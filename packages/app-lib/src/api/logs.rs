use std::fmt::Write as _;
use std::io::{BufRead, SeekFrom};
use std::time::SystemTime;

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    State,
    prelude::Credentials,
    util::io::{self, IOError},
};

#[derive(Serialize, Debug)]
pub struct Logs {
    pub log_type: LogType,
    pub filename: String,
    pub age: u64,
    pub output: Option<CensoredString>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LogType {
    InfoLog,
    CrashReport,
}

const LOG_COMPACTION_THRESHOLD: usize = 20;

#[derive(Serialize, Debug)]
pub struct LatestLogCursor {
    pub cursor: u64,
    pub output: CensoredString,
    pub new_file: bool,
}

#[derive(Serialize, Debug)] // Not deserialize
#[serde(transparent)]
pub struct CensoredString(String);
impl CensoredString {
    pub fn censor(mut s: String, credentials_list: &[Credentials]) -> Self {
        let username = whoami::username();
        s = s
            .replace(&format!("/{username}/"), "/{COMPUTER_USERNAME}/")
            .replace(&format!("\\{username}\\"), "\\{COMPUTER_USERNAME}\\");
        for credentials in credentials_list {
            // Use the offline profile to guarantee that this function does not cause
            // Mojang API request, and is never delayed by a network request. The offline
            // profile is optimistically updated on upsert from time to time anyway
            s = s
                .replace(&credentials.access_token, "{MINECRAFT_ACCESS_TOKEN}")
                .replace(
                    &credentials.offline_profile.name,
                    "{MINECRAFT_USERNAME}",
                )
                .replace(
                    &credentials.offline_profile.id.as_simple().to_string(),
                    "{MINECRAFT_UUID}",
                )
                .replace(
                    &credentials.offline_profile.id.as_hyphenated().to_string(),
                    "{MINECRAFT_UUID}",
                );
        }

        Self(s)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LogCompactionStats {
    compacted_runs: usize,
    compacted_lines: usize,
}

struct CompactedLog {
    output: String,
    stats: LogCompactionStats,
}

async fn resolve_instance_path(
    instance: &str,
    state: &State,
) -> crate::Result<String> {
    sqlx::query_scalar!(
        "
        SELECT path
        FROM instances
        WHERE id = ? OR path = ?
        ORDER BY CASE WHEN id = ? THEN 0 ELSE 1 END
        LIMIT 1
        ",
        instance,
        instance,
        instance,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unknown instance id or path: {instance}"
        ))
        .as_error()
    })
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(line) = line.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = line.strip_suffix('\n') {
        (line, "\n")
    } else if let Some(line) = line.strip_suffix('\r') {
        (line, "\r")
    } else {
        (line, "")
    }
}

fn push_compacted_log_run(
    output: &mut String,
    stats: &mut LogCompactionStats,
    line: &str,
    line_ending: &str,
    count: usize,
) {
    if count >= LOG_COMPACTION_THRESHOLD {
        output.push_str(line);
        let _ = write!(output, " (x{count} times - compacted by Modrinth App)");
        output.push_str(line_ending);
        stats.compacted_runs += 1;
        stats.compacted_lines += count;
    } else {
        for _ in 0..count {
            output.push_str(line);
            output.push_str(line_ending);
        }
    }
}

fn read_compacted_log<R: BufRead>(
    reader: &mut R,
) -> std::io::Result<CompactedLog> {
    let mut output = String::new();
    let mut stats = LogCompactionStats::default();
    let mut buffer = Vec::new();
    let mut current_line: Option<String> = None;
    let mut current_line_ending = String::new();
    let mut current_count = 0usize;

    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&buffer);
        let (line, line_ending) = split_line_ending(&line);

        match current_line.as_deref() {
            Some(current) if current == line => {
                current_count += 1;
                if current_line_ending.is_empty() && !line_ending.is_empty() {
                    current_line_ending = line_ending.to_string();
                }
            }
            _ => {
                if let Some(current) = current_line.take() {
                    push_compacted_log_run(
                        &mut output,
                        &mut stats,
                        &current,
                        &current_line_ending,
                        current_count,
                    );
                }

                current_line = Some(line.to_string());
                current_line_ending = line_ending.to_string();
                current_count = 1;
            }
        }
    }

    if let Some(current) = current_line {
        push_compacted_log_run(
            &mut output,
            &mut stats,
            &current,
            &current_line_ending,
            current_count,
        );
    }

    Ok(CompactedLog { output, stats })
}

fn compact_duplicate_lines(input: &str) -> CompactedLog {
    let mut reader = std::io::Cursor::new(input.as_bytes());
    read_compacted_log(&mut reader)
        .expect("compacting an in-memory log should not fail")
}

fn format_count(count: usize) -> String {
    let raw = count.to_string();
    let mut formatted = String::with_capacity(raw.len() + raw.len() / 3);
    for (index, character) in raw.chars().enumerate() {
        if index > 0 && (raw.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(character);
    }
    formatted
}

async fn maybe_emit_log_compaction_warning(
    file_name: &str,
    stats: LogCompactionStats,
) {
    if stats.compacted_runs == 0 {
        return;
    }

    let _ = crate::event::emit::emit_warning(&format!(
        "Modrinth App has compacted {} repeated log lines in {} before displaying it for performance reasons.",
        format_count(stats.compacted_lines),
        file_name,
    ))
    .await;
}

impl Logs {
    async fn build(
        log_type: LogType,
        age: SystemTime,
        instance_path: &str,
        filename: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            log_type,
            age: age
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                let state = State::get().await?;
                Some(
                    get_output_by_filename_from_path(
                        &state,
                        instance_path,
                        log_type,
                        &filename,
                    )
                    .await?,
                )
            },
            filename,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs_from_type(
    instance_id: &str,
    log_type: LogType,
    clear_contents: Option<bool>,
    logs: &mut Vec<crate::Result<Logs>>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    };

    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry: std::fs::DirEntry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let age = entry
                .metadata()?
                .created()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                logs.push(
                    Logs::build(
                        log_type,
                        age,
                        &instance_path,
                        file_name,
                        clear_contents,
                    )
                    .await,
                );
            }
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn get_logs(
    instance_id: &str,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let mut logs = Vec::new();
    get_logs_from_type(
        instance_id,
        LogType::InfoLog,
        clear_contents,
        &mut logs,
    )
    .await?;
    get_logs_from_type(
        instance_id,
        LogType::CrashReport,
        clear_contents,
        &mut logs,
    )
    .await?;

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    logs.sort_by(|a, b| b.age.cmp(&a.age).then(b.filename.cmp(&a.filename)));
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: String,
) -> crate::Result<Logs> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let path = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    }
    .join(&filename);

    let metadata = std::fs::metadata(&path)?;
    let age = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);

    Logs::build(log_type, age, &instance_path, filename, Some(true)).await
}

async fn get_output_by_filename_from_path(
    state: &State,
    instance_path: &str,
    log_type: LogType,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(instance_path)
        }
    };

    let path = logs_folder.join(file_name);

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();

    if let Some(ext) = path.extension() {
        if ext == "gz" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let gz =
                flate2::read::GzDecoder::new(std::io::BufReader::new(file));
            let mut reader = std::io::BufReader::new(gz);
            let compacted = read_compacted_log(&mut reader)
                .map_err(|e| IOError::with_path(e, &path))?;
            maybe_emit_log_compaction_warning(file_name, compacted.stats).await;
            return Ok(CensoredString::censor(compacted.output, &credentials));
        } else if ext == "log" || ext == "txt" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut reader = std::io::BufReader::new(file);
            let compacted = read_compacted_log(&mut reader)
                .map_err(|e| IOError::with_path(e, &path))?;
            maybe_emit_log_compaction_warning(file_name, compacted.stats).await;
            return Ok(CensoredString::censor(compacted.output, &credentials));
        }
    }
    Err(crate::ErrorKind::OtherError(format!(
        "File extension not supported: {}",
        path.display()
    ))
    .into())
}

#[tracing::instrument]
pub async fn get_output_by_filename(
    instance_id: &str,
    log_type: LogType,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;
    get_output_by_filename_from_path(
        &state,
        &instance_path,
        log_type,
        file_name,
    )
    .await
}

#[tracing::instrument]
pub async fn delete_logs(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = state.directories.instance_logs_dir(&instance_path);
    for entry in std::fs::read_dir(&logs_folder)
        .map_err(|e| IOError::with_path(e, &logs_folder))?
    {
        let entry = entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
        let path = entry.path();
        if path.is_dir() {
            io::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn delete_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    };

    let path = logs_folder.join(filename);
    io::remove_file(&path).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn get_live_log_buffer(
    instance_id: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;
    let lines = crate::state::get_log_buffer(instance_id);
    let joined = lines.join("\n");
    let compacted = compact_duplicate_lines(&joined);

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();
    maybe_emit_log_compaction_warning("live log", compacted.stats).await;
    Ok(CensoredString::censor(compacted.output, &credentials))
}

pub fn clear_live_log_buffer(instance_id: &str) {
    crate::state::remove_log_buffer(instance_id);
}

#[tracing::instrument]
pub async fn get_latest_log_cursor(
    instance_id: &str,
    cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    get_generic_live_log_cursor(instance_id, "launcher_log.txt", cursor).await
}

#[tracing::instrument]
pub async fn get_generic_live_log_cursor(
    instance_id: &str,
    log_file_name: &str,
    mut cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;
    let logs_folder = state.directories.instance_logs_dir(&instance_path);
    let path = logs_folder.join(log_file_name);
    if !path.exists() {
        // Allow silent failure if latest.log doesn't exist (as the instance may have been launched, but not yet created the file)
        return Ok(LatestLogCursor {
            cursor: 0,
            new_file: false,
            output: CensoredString("".to_string()),
        });
    }

    let mut file = File::open(&path)
        .await
        .map_err(|e| IOError::with_path(e, &path))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, &path))?;

    let mut new_file = false;
    if cursor > metadata.len() {
        // Cursor is greater than file length, reset cursor to 0
        // Likely cause is that the file was rotated while the log was being read
        cursor = 0;
        new_file = true;
    }

    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Seek to cursor
    let bytes_read = file
        .read_to_end(&mut buffer)
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Read to end of file
    let output = String::from_utf8_lossy(&buffer); // Convert to String
    let compacted = compact_duplicate_lines(&output);
    let cursor = cursor + bytes_read as u64; // Update cursor

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();
    maybe_emit_log_compaction_warning(log_file_name, compacted.stats).await;
    let output = CensoredString::censor(compacted.output, &credentials);
    Ok(LatestLogCursor {
        cursor,
        new_file,
        output,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrashDiagnosticReport {
    pub has_crashed: bool,
    pub error_type: String,
    pub summary: String,
    pub recommendation: String,
    pub target_mod_id: Option<String>,
    pub target_mod_name: Option<String>,
    pub required_version: Option<String>,
    pub recommended_ram_mb: Option<u32>,
    pub recommended_java_version: Option<u32>,
    pub log_snippet: Option<String>,
}

fn extract_log_snippet(content: &str, keyword: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let lower_kw = keyword.to_lowercase();
    for (i, line) in lines.iter().enumerate() {
        if line.to_lowercase().contains(&lower_kw) {
            let start = i.saturating_sub(4);
            let end = (i + 10).min(lines.len());
            return Some(lines[start..end].join("\n"));
        }
    }
    None
}

fn extract_missing_dependency_details(content: &str) -> (String, Option<String>, Option<String>) {
    for line in content.lines() {
        let l_lower = line.to_lowercase();
        if l_lower.contains("requires") {
            if l_lower.contains("indium") {
                return (
                    "Crash Detected: Sodium / Sodium Options is missing the Indium dependency.".to_string(),
                    Some("indium".to_string()),
                    None,
                );
            }
            if l_lower.contains("fabric-api") || l_lower.contains("fabric api") {
                return (
                    "Crash Detected: Mod requires Fabric API dependency.".to_string(),
                    Some("fabric-api".to_string()),
                    None,
                );
            }
            return (
                format!("Crash Detected: Missing required mod dependency. ({line})"),
                None,
                None,
            );
        }
    }
    ("Crash Detected: Missing required mod dependency.".to_string(), None, None)
}

fn extract_duplicate_mod_id(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.to_lowercase().contains("duplicate") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_mixin_details(content: &str) -> (Option<String>, String) {
    for line in content.lines() {
        if line.contains("Mixin") || line.contains("mixin") {
            return (
                None,
                format!("Mixin Transformer Error: Mod injection conflict detected. ({line})"),
            );
        }
    }
    (None, "Mixin Transformer Error: Conflict during mod injection.".to_string())
}

#[tracing::instrument]
pub async fn parse_instance_crash_diagnostics(instance_id: &str) -> crate::Result<CrashDiagnosticReport> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;
    
    // 1. Check crash-reports folder first for latest crash report
    let crash_reports_dir = std::path::Path::new(&instance_path).join("crash-reports");
    if crash_reports_dir.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&crash_reports_dir).await {
            let mut latest_file = None;
            let mut latest_time = SystemTime::UNIX_EPOCH;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(meta) = entry.metadata().await {
                    if let Ok(mod_time) = meta.modified() {
                        if mod_time > latest_time {
                            latest_time = mod_time;
                            latest_file = Some(entry.path());
                        }
                    }
                }
            }
            if let Some(crash_file) = latest_file {
                if let Ok(content) = tokio::fs::read_to_string(&crash_file).await {
                    let report = parse_crash_diagnostics_from_text(&content);
                    if report.has_crashed {
                        return Ok(report);
                    }
                }
            }
        }
    }

    // 2. Check latest.log
    let logs_dir = state.directories.instance_logs_dir(&instance_path);
    let latest_log_path = logs_dir.join("latest.log");
    if latest_log_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&latest_log_path).await {
            let report = parse_crash_diagnostics_from_text(&content);
            return Ok(report);
        }
    }

    Ok(CrashDiagnosticReport {
        has_crashed: false,
        error_type: "none".to_string(),
        summary: "No crash detected.".to_string(),
        recommendation: "".to_string(),
        target_mod_id: None,
        target_mod_name: None,
        required_version: None,
        recommended_ram_mb: None,
        recommended_java_version: None,
        log_snippet: None,
    })
}

pub fn parse_crash_diagnostics_from_text(log_content: &str) -> CrashDiagnosticReport {
    let lower = log_content.to_lowercase();

    if lower.contains("java.lang.outofmemoryerror") || lower.contains("gc overhead limit exceeded") || lower.contains("out of memory") {
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "out_of_memory".to_string(),
            summary: "Out of Memory: Java heap space exhausted during game execution.".to_string(),
            recommendation: "Increase allocated RAM memory to at least 6GB (6144 MB) in instance settings.".to_string(),
            target_mod_id: None,
            target_mod_name: None,
            required_version: None,
            recommended_ram_mb: Some(6144),
            recommended_java_version: None,
            log_snippet: extract_log_snippet(log_content, "OutOfMemoryError"),
        };
    }

    if lower.contains("requires") && (lower.contains("mod") || lower.contains("dependency") || lower.contains("indium") || lower.contains("fabric") || lower.contains("forge")) {
        let (summary, mod_id, req_ver) = extract_missing_dependency_details(log_content);
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "missing_dependency".to_string(),
            summary: summary.clone(),
            recommendation: format!("Install missing dependency mod '{}' from the Browse tab.", mod_id.as_deref().unwrap_or("required mod")),
            target_mod_id: mod_id,
            target_mod_name: None,
            required_version: req_ver,
            recommended_ram_mb: None,
            recommended_java_version: None,
            log_snippet: extract_log_snippet(log_content, "requires"),
        };
    }

    if lower.contains("duplicate mod") || lower.contains("found duplicate") || lower.contains("modidalreadyexistsexception") {
        let dup_id = extract_duplicate_mod_id(log_content);
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "duplicate_mod".to_string(),
            summary: format!("Duplicate Mod Detected: Multiple copies of mod '{}' installed.", dup_id.as_deref().unwrap_or("mod")),
            recommendation: "Remove or disable the duplicate mod version in the instance mods tab.".to_string(),
            target_mod_id: dup_id,
            target_mod_name: None,
            required_version: None,
            recommended_ram_mb: None,
            recommended_java_version: None,
            log_snippet: extract_log_snippet(log_content, "duplicate"),
        };
    }

    if lower.contains("unsupportedclassversionerror") || lower.contains("has been compiled by a more recent version of the java runtime") || lower.contains("class file version 61.0") || lower.contains("class file version 65.0") {
        let (needed_ver, text) = if lower.contains("version 65.0") { (21, "Java 21") } else { (17, "Java 17") };
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "java_mismatch".to_string(),
            summary: format!("Java Version Mismatch: Game requires {text} or higher."),
            recommendation: format!("Switch instance Java version to {text} using 1-Click Auto JDK."),
            target_mod_id: None,
            target_mod_name: None,
            required_version: None,
            recommended_ram_mb: None,
            recommended_java_version: Some(needed_ver),
            log_snippet: extract_log_snippet(log_content, "UnsupportedClassVersionError"),
        };
    }

    if lower.contains("mixintransformererror") || lower.contains("mixin apply failed") || lower.contains("critical injection failure") {
        let (failing_mixin, summary) = extract_mixin_details(log_content);
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "mixin_conflict".to_string(),
            summary,
            recommendation: "Mod mixin conflict detected. Try updating or disabling the conflicting mod.".to_string(),
            target_mod_id: failing_mixin,
            target_mod_name: None,
            required_version: None,
            recommended_ram_mb: None,
            recommended_java_version: None,
            log_snippet: extract_log_snippet(log_content, "Mixin"),
        };
    }

    if lower.contains("crash") || lower.contains("fatal") || lower.contains("exception") {
        return CrashDiagnosticReport {
            has_crashed: true,
            error_type: "unknown".to_string(),
            summary: "Minecraft crashed unexpectedly during execution.".to_string(),
            recommendation: "Check the full instance log for details or repair the instance.".to_string(),
            target_mod_id: None,
            target_mod_name: None,
            required_version: None,
            recommended_ram_mb: None,
            recommended_java_version: None,
            log_snippet: extract_log_snippet(log_content, "Exception"),
        };
    }

    CrashDiagnosticReport {
        has_crashed: false,
        error_type: "none".to_string(),
        summary: "No crash detected.".to_string(),
        recommendation: "".to_string(),
        target_mod_id: None,
        target_mod_name: None,
        required_version: None,
        recommended_ram_mb: None,
        recommended_java_version: None,
        log_snippet: None,
    }
}

