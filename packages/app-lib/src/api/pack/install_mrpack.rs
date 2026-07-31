use crate::State;
use crate::event::emit::loading_try_for_each_concurrent;
use crate::install::{
    InstallErrorContext, InstallJobEventKind, InstallPhaseDetails,
    InstallPhaseId, InstallProgress, InstallProgressReporter,
    InstallProgressSecondary,
};
use crate::pack::install_from::{
    EnvType, PackFile, PackFileHash, set_instance_information,
};
use crate::state::instances::ContentSourceKind;
use crate::state::{
    CachedEntry, CachedFile, EditInstance, InstanceInstallStage, SideType,
    cache_file_hash,
};
use crate::util::fetch::{
    DownloadMeta, DownloadReason, FetchProgressFn, fetch_mirrors_with_progress,
    write,
};
use crate::util::io;
use async_zip::base::read::seek::ZipFileReader as SeekZipFileReader;
use async_zip::base::read::{WithEntry, ZipEntryReader};
use async_zip::tokio::read::fs::ZipFileReader as FsZipFileReader;
use futures::StreamExt;
use path_util::SafeRelativeUtf8UnixPathBuf;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use super::install_from::{CreatePack, CreatePackFile, PackDependency, PackFormat};
use crate::data::ProjectType;
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

type ExtractProgressFn<'a> = dyn FnMut(u64) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'a>>
    + Send
    + 'a;
const MODPACK_CONTENT_DOWNLOAD_CONCURRENCY: usize = 16;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifest {
    pub minecraft: CurseForgeMinecraft,
    pub name: String,
    pub version: String,
    pub files: Option<Vec<CurseForgeManifestFile>>,
    pub overrides: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMinecraft {
    pub version: String,
    pub mod_loaders: Option<Vec<CurseForgeModLoaderInfo>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeModLoaderInfo {
    pub id: String,
    pub primary: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    pub required: Option<bool>,
}

#[derive(serde::Deserialize, Debug)]
struct CfFileApiResponse {
    data: Option<CfFileApiData>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CfFileApiData {
    file_name: String,
    download_url: Option<String>,
    file_length: Option<u32>,
    hashes: Option<Vec<CfFileHash>>,
}

#[derive(serde::Deserialize, Debug)]
struct CfFileHash {
    value: String,
    algo: u32,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CfModInfoData {
    files: Option<Vec<CfFileApiData>>,
    latest_files: Option<Vec<CfFileApiData>>,
}

#[derive(serde::Deserialize, Debug)]
struct CfModInfoResponse {
    data: Option<CfModInfoData>,
}

pub async fn convert_cf_manifest_to_pack_format(
    cf_manifest: CurseForgeManifest,
    reporter: Option<&InstallProgressReporter>,
    modpack_details: Option<&InstallPhaseDetails>,
) -> crate::Result<PackFormat> {
    tracing::info!(
        "[CURSEFORGE IMPORT] Converting CurseForge manifest '{}' (version {}) for Minecraft {}",
        cf_manifest.name,
        cf_manifest.version,
        cf_manifest.minecraft.version
    );

    let mut dependencies = HashMap::new();
    dependencies.insert(
        PackDependency::Minecraft,
        cf_manifest.minecraft.version.clone(),
    );

    if let Some(ref loaders) = cf_manifest.minecraft.mod_loaders {
        if let Some(loader) = loaders.iter().find(|l| l.primary.unwrap_or(false)).or_else(|| loaders.first()) {
            let id = &loader.id;
            if let Some((loader_type, loader_ver)) = id.split_once('-') {
                match loader_type.to_lowercase().as_str() {
                    "forge" => {
                        dependencies.insert(PackDependency::Forge, loader_ver.to_string());
                    }
                    "fabric" => {
                        dependencies.insert(PackDependency::FabricLoader, loader_ver.to_string());
                    }
                    "quilt" => {
                        dependencies.insert(PackDependency::QuiltLoader, loader_ver.to_string());
                    }
                    "neoforge" => {
                        dependencies.insert(PackDependency::NeoForge, loader_ver.to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-api-key",
        reqwest::header::HeaderValue::from_static("$2a$10$3kFa9lBWciEK.lsp7NyCSupZ3XmlAYixZQ9fTczqsz1/.W9QDnLUy"),
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .default_headers(headers)
        .build()?;

    let mut pack_files = Vec::new();
    if let Some(cf_files) = cf_manifest.files {
        let total_files = cf_files.len();
        tracing::info!("[CURSEFORGE IMPORT] Resolving {} mod files in parallel (16 workers)...", total_files);

        let completed = Arc::new(AtomicU64::new(0));
        let reporter_clone = reporter.cloned();
        let details_clone = modpack_details.cloned();

        let resolved_files: Vec<Option<PackFile>> = futures::stream::iter(cf_files)
            .map(|file_ref| {
                let client = client.clone();
                let completed = completed.clone();
                let reporter_clone = reporter_clone.clone();
                let details_clone = details_clone.clone();

                async move {
                    let official_url = format!(
                        "https://api.curseforge.com/v1/mods/{}/files/{}",
                        file_ref.project_id, file_ref.file_id
                    );
                    let mirror_url = format!(
                        "https://api.curse.tools/v1/cf/mods/{}/files/{}",
                        file_ref.project_id, file_ref.file_id
                    );
                    let mut resolved_data: Option<CfFileApiData> = None;

                    // Try official CurseForge API first
                    if let Ok(res) = client.get(&official_url).send().await {
                        if let Ok(json) = res.json::<CfFileApiResponse>().await {
                            if let Some(data) = json.data {
                                if data.download_url.is_some() {
                                    resolved_data = Some(data);
                                }
                            }
                        }
                    }

                    // Fallback to curse.tools mirror file API
                    if resolved_data.is_none() {
                        if let Ok(res) = client.get(&mirror_url).send().await {
                            if let Ok(json) = res.json::<CfFileApiResponse>().await {
                                if let Some(data) = json.data {
                                    if data.download_url.is_some() {
                                        resolved_data = Some(data);
                                    }
                                }
                            }
                        }
                    }

                    // Fallback: Query mod info if file API fails or has missing downloadUrl
                    if resolved_data.is_none() {
                        let mod_url = format!("https://api.curseforge.com/v1/mods/{}", file_ref.project_id);
                        if let Ok(res) = client.get(&mod_url).send().await {
                            if let Ok(json) = res.json::<CfModInfoResponse>().await {
                                if let Some(mod_data) = json.data {
                                    let all_files = mod_data.files.or(mod_data.latest_files).unwrap_or_default();
                                    if let Some(matched) = all_files.into_iter().find(|f| f.file_name.contains(&file_ref.file_id.to_string()) || f.download_url.is_some()) {
                                        resolved_data = Some(matched);
                                    }
                                }
                            }
                        }
                    }

                    let cur = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    if let (Some(rep), Some(det)) = (&reporter_clone, &details_clone) {
                        let _ = rep.update(
                            InstallPhaseId::ReadingPackManifest,
                            Some(InstallProgress {
                                current: cur,
                                total: total_files as u64,
                                secondary: None,
                            }),
                            det.clone(),
                        ).await;
                    }

                    if let Some(data) = resolved_data {
                        if let Some(dl_url) = data.download_url {
                            let mut hashes = HashMap::new();
                            if let Some(h_list) = data.hashes {
                                for h in h_list {
                                    if h.algo == 1 {
                                        hashes.insert(PackFileHash::Sha1, h.value);
                                    }
                                }
                            }

                            if let Ok(path_buf) = SafeRelativeUtf8UnixPathBuf::try_from(format!("mods/{}", data.file_name)) {
                                return Some(PackFile {
                                    path: path_buf,
                                    hashes,
                                    env: None,
                                    downloads: vec![dl_url],
                                    file_size: data.file_length.unwrap_or(0),
                                });
                            }
                        }
                    }
                    None
                }
            })
            .buffer_unordered(16)
            .collect()
            .await;

        for f in resolved_files {
            if let Some(pf) = f {
                pack_files.push(pf);
            }
        }
    }

    tracing::info!(
        "[CURSEFORGE IMPORT] Successfully resolved {} mod files for '{}'",
        pack_files.len(),
        cf_manifest.name
    );

    Ok(PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id: cf_manifest.version,
        name: cf_manifest.name,
        summary: None,
        files: pack_files,
        dependencies,
    })
}

#[derive(Clone)]
struct ModpackContentInstallContext {
    instance_id: String,
    instance_path: String,
    instance_full_path: PathBuf,
    download_meta: DownloadMeta,
    pack_version_id: Option<String>,
    pack_project_id: Option<String>,
    reporter: InstallProgressReporter,
    modpack_details: InstallPhaseDetails,
    content_progress: Arc<AtomicU64>,
    content_bytes_progress: Arc<AtomicU64>,
    active_download_bytes: Arc<Mutex<HashMap<String, u64>>>,
    file_infos_by_hash: Arc<HashMap<String, CachedFile>>,
    num_files: usize,
    content_total_bytes: u64,
}

impl ModpackContentInstallContext {
    async fn mark_downloaded(
        &self,
        file_size: u64,
        event: InstallJobEventKind,
    ) -> crate::Result<()> {
        let current = self.content_progress.fetch_add(1, Ordering::Relaxed) + 1;
        let current_bytes = self
            .content_bytes_progress
            .fetch_add(file_size, Ordering::Relaxed)
            + file_size;

        self.reporter
            .update_with_events(
                InstallPhaseId::DownloadingContent,
                Some(InstallProgress {
                    current,
                    total: self.num_files as u64,
                    secondary: (self.content_total_bytes > 0).then_some(
                        InstallProgressSecondary {
                            current: current_bytes
                                .min(self.content_total_bytes),
                            total: self.content_total_bytes,
                        },
                    ),
                }),
                self.modpack_details.clone(),
                vec![event],
            )
            .await
    }

    async fn remove_active_download(&self, path: &str) {
        let mut active_download_bytes = self.active_download_bytes.lock().await;
        active_download_bytes.remove(path);
    }

    async fn update_active_download(
        &self,
        path: String,
        downloaded: u64,
    ) -> u64 {
        let mut active_download_bytes = self.active_download_bytes.lock().await;
        active_download_bytes.insert(path, downloaded);
        active_download_bytes.values().sum::<u64>()
    }
}

enum MrpackZipReader {
    Memory(async_zip::tokio::read::seek::ZipFileReader<Cursor<bytes::Bytes>>),
    // Local imports stay on disk so large .mrpacks do not have to fit in memory.
    File(FsZipFileReader),
}

impl MrpackZipReader {
    async fn new(file: &CreatePackFile) -> crate::Result<Self> {
        match file {
            CreatePackFile::Bytes(file) => Ok(Self::Memory(
                SeekZipFileReader::with_tokio(Cursor::new(file.clone()))
                    .await
                    .map_err(|_| {
                        crate::Error::from(crate::ErrorKind::InputError(
                            "Failed to read input modpack zip".to_string(),
                        ))
                    })?,
            )),
            CreatePackFile::Path(path) => Ok(Self::File(
                FsZipFileReader::new(path).await.map_err(|_| {
                    crate::Error::from(crate::ErrorKind::InputError(
                        "Failed to read input modpack zip".to_string(),
                    ))
                })?,
            )),
        }
    }

    fn file(&self) -> &async_zip::ZipFile {
        match self {
            Self::Memory(reader) => reader.file(),
            Self::File(reader) => reader.file(),
        }
    }

    async fn read_entry_to_string(
        &mut self,
        index: usize,
    ) -> crate::Result<String> {
        let mut value = String::new();
        match self {
            Self::Memory(reader) => {
                let mut reader = reader.reader_with_entry(index).await?;
                reader.read_to_string_checked(&mut value).await?;
            }
            Self::File(reader) => {
                let mut reader = reader.reader_with_entry(index).await?;
                reader.read_to_string_checked(&mut value).await?;
            }
        }

        Ok(value)
    }

    async fn hash_entry(
        &mut self,
        index: usize,
    ) -> crate::Result<(u64, String)> {
        match self {
            Self::Memory(reader) => {
                hash_zip_entry(reader.reader_with_entry(index).await?).await
            }
            Self::File(reader) => {
                hash_zip_entry(reader.reader_with_entry(index).await?).await
            }
        }
    }

    async fn extract_entry(
        &mut self,
        index: usize,
        path: &Path,
        semaphore: &crate::util::fetch::IoSemaphore,
        progress: Option<&mut ExtractProgressFn<'_>>,
    ) -> crate::Result<(u64, String)> {
        match self {
            Self::Memory(reader) => {
                extract_zip_entry(
                    reader.reader_with_entry(index).await?,
                    path,
                    semaphore,
                    progress,
                )
                .await
            }
            Self::File(reader) => {
                extract_zip_entry(
                    reader.reader_with_entry(index).await?,
                    path,
                    semaphore,
                    progress,
                )
                .await
            }
        }
    }
}

async fn hash_zip_entry<R>(
    mut reader: ZipEntryReader<'_, R, WithEntry<'_>>,
) -> crate::Result<(u64, String)>
where
    R: futures_lite::io::AsyncBufRead + Unpin,
{
    let expected_crc32 = reader.entry().crc32();
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];

    loop {
        let bytes_read =
            futures_lite::io::AsyncReadExt::read(&mut reader, &mut buffer)
                .await?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
    }

    if reader.compute_hash() != expected_crc32 {
        return Err(async_zip::error::ZipError::CRC32CheckError.into());
    }

    Ok((size, hasher.digest().to_string()))
}

pub(crate) async fn get_external_files_from_mrpack(
    file: &CreatePackFile,
) -> crate::Result<Vec<String>> {
    let mut zip_reader = MrpackZipReader::new(file).await?;
    let mr_manifest_idx = zip_reader.file().entries().iter().position(|entry| {
        matches!(entry.filename().as_str(), Ok("modrinth.index.json"))
    });
    let cf_manifest_idx = zip_reader.file().entries().iter().position(|entry| {
        matches!(entry.filename().as_str(), Ok("manifest.json"))
    });

    let pack: PackFormat = if let Some(manifest_idx) = mr_manifest_idx {
        let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;
        serde_json::from_str(&manifest)?
    } else if let Some(manifest_idx) = cf_manifest_idx {
        let manifest_str = zip_reader.read_entry_to_string(manifest_idx).await?;
        let cf_manifest: CurseForgeManifest = serde_json::from_str(&manifest_str)?;
        convert_cf_manifest_to_pack_format(cf_manifest, None, None).await?
    } else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in modpack archive".to_string(),
        )));
    };
    let mut candidates = pack
        .files
        .into_iter()
        .filter_map(|file| {
            let path = file.path.as_str();
            let hash = file.hashes.get(&PackFileHash::Sha1)?.clone();
            let file_name = path.rsplit('/').next()?.to_string();
            Some((file_name, hash))
        })
        .collect::<Vec<_>>();

    let override_entries = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let path = entry.filename().as_str().ok()?;
            let relative_path = path
                .strip_prefix("overrides/")
                .or_else(|| path.strip_prefix("client-overrides/"))?;
            if path.ends_with('/')
                || ProjectType::get_from_parent_folder(relative_path).is_none()
            {
                return None;
            }
            let file_name = relative_path.rsplit('/').next()?.to_string();
            Some((index, file_name))
        })
        .collect::<Vec<_>>();

    for (index, file_name) in override_entries {
        let (_, hash) = zip_reader.hash_entry(index).await?;
        candidates.push((file_name, hash));
    }

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let state = State::get().await?;
    let hashes = candidates
        .iter()
        .map(|(_, hash)| hash.as_str())
        .collect::<Vec<_>>();
    let recognized_hashes = match CachedEntry::get_file_many(
        &hashes,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await
    {
        Ok(files) => files
            .into_iter()
            .map(|file| file.hash)
            .collect::<HashSet<_>>(),
        Err(err) => {
            tracing::warn!("Failed to look up files in imported mrpack: {err}");
            HashSet::new()
        }
    };

    let mut external_files = candidates
        .into_iter()
        .filter_map(|(file_name, hash)| {
            (!recognized_hashes.contains(&hash)).then_some(file_name)
        })
        .collect::<Vec<_>>();
    external_files.sort();
    external_files.dedup();
    Ok(external_files)
}

async fn extract_zip_entry<R>(
    mut reader: ZipEntryReader<'_, R, WithEntry<'_>>,
    path: &Path,
    semaphore: &crate::util::fetch::IoSemaphore,
    mut progress: Option<&mut ExtractProgressFn<'_>>,
) -> crate::Result<(u64, String)>
where
    R: futures_lite::io::AsyncBufRead + Unpin,
{
    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = path.parent() {
        io::create_dir_all(parent).await?;
    }

    let parent = path.parent().ok_or_else(|| {
        io::IOError::from(std::io::Error::other(
            "could not get parent directory for temporary file",
        ))
    })?;
    let temp_path = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| io::IOError::with_path(e, parent))?
        .into_temp_path();

    // Only replace the profile file after the ZIP entry has passed its CRC check.
    let expected_crc32 = reader.entry().crc32();
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| io::IOError::with_path(e, &temp_path))?;
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];

    loop {
        let bytes_read =
            futures_lite::io::AsyncReadExt::read(&mut reader, &mut buffer)
                .await?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .await
            .map_err(|e| io::IOError::with_path(e, &temp_path))?;
        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
        if let Some(progress) = progress.as_mut() {
            progress(bytes_read as u64).await?;
        }
    }

    file.flush()
        .await
        .map_err(|e| io::IOError::with_path(e, &temp_path))?;
    drop(file);

    if reader.compute_hash() != expected_crc32 {
        return Err(async_zip::error::ZipError::CRC32CheckError.into());
    }

    temp_path.persist(path).map_err(|e| {
        let tempfile::PathPersistError { error, .. } = e;
        io::IOError::with_path(error, path)
    })?;

    Ok((size, hasher.digest().to_string()))
}

pub(crate) async fn install_zipped_mrpack_files_with_reporter(
    create_pack: CreatePack,
    ignore_lock: bool,
    reason: DownloadReason,
    reporter: InstallProgressReporter,
) -> crate::Result<String> {
    let state = &State::get().await?;

    let file = create_pack.file;
    let description = create_pack.description.clone();
    let icon = create_pack.description.icon;
    let project_id = create_pack.description.project_id;
    let version_id = create_pack.description.version_id;
    let instance_id = create_pack.description.instance_id;
    let mut icon_exists = icon.is_some();
    let source_path = pack_source_path(&file);

    reporter
        .set_context(
            InstallErrorContext::new("read modpack archive")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .build(),
        )
        .await?;
    let mut zip_reader = MrpackZipReader::new(&file).await?;
    let instance_full_path =
        crate::api::instance::get_full_path(&instance_id).await?;
    let modpack_details = InstallPhaseDetails::Modpack {
        project_id: project_id.clone(),
        version_id: version_id.clone(),
        title: description.override_title.clone(),
    };
    reporter
        .update(
            InstallPhaseId::ReadingPackManifest,
            None,
            modpack_details.clone(),
        )
        .await?;
    reporter
        .set_context(
            InstallErrorContext::new("read modpack manifest")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .entry_path("modrinth.index.json")
                .build(),
        )
        .await?;

    let mr_manifest_idx = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("modrinth.index.json"))
    });
    let cf_manifest_idx = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("manifest.json"))
    });

    let pack: PackFormat = if let Some(manifest_idx) = mr_manifest_idx {
        let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;
        serde_json::from_str(&manifest)?
    } else if let Some(manifest_idx) = cf_manifest_idx {
        let manifest_str = zip_reader.read_entry_to_string(manifest_idx).await?;
        let cf_manifest: CurseForgeManifest = serde_json::from_str(&manifest_str)?;
        convert_cf_manifest_to_pack_format(cf_manifest, Some(&reporter), Some(&modpack_details)).await?
    } else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in modpack archive".to_string(),
        )));
    };
    if &*pack.game != "minecraft" {
        return Err(crate::ErrorKind::InputError(
            "Pack does not support Minecraft".to_string(),
        )
        .into());
    }

    reporter
        .update(InstallPhaseId::ResolvingPack, None, modpack_details.clone())
        .await?;

    if !icon_exists {
        let icon_entry =
            zip_reader.file().entries().iter().enumerate().find_map(
                |(index, entry)| {
                    matches!(
                        entry.filename().as_str(),
                        Ok("icon.png"
                            | "overrides/icon.png"
                            | "client-overrides/icon.png")
                    )
                    .then_some(index)
                },
            );

        if let Some(icon_entry) = icon_entry {
            let icon_path = instance_full_path.join("icon.png");
            zip_reader
                .extract_entry(
                    icon_entry,
                    &icon_path,
                    &state.io_semaphore,
                    None,
                )
                .await?;
            crate::api::instance::edit_icon(&instance_id, Some(&icon_path))
                .await?;
            icon_exists = true;
        }
    }

    // Cache the modpack file hashes for later filtering of user-added content
    // Includes both manifest file hashes and computed hashes for override files
    if let Some(ref version_id) = version_id {
        let mut file_hashes: Vec<String> = pack
            .files
            .iter()
            .filter_map(|f| f.hashes.get(&PackFileHash::Sha1).cloned())
            .collect();

        // Also hash files from overrides folders (these aren't in modrinth.index.json)
        let override_entries: Vec<usize> = zip_reader
            .file()
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let filename = entry.filename().as_str().ok()?;
                let is_override = (filename.starts_with("overrides/")
                    || filename.starts_with("client-overrides/")
                    || filename.starts_with("server-overrides/"))
                    && !filename.ends_with('/');
                is_override.then_some(index)
            })
            .collect();

        for index in override_entries {
            let (_, hash) = zip_reader.hash_entry(index).await?;
            file_hashes.push(hash);
        }

        let project_ids: Vec<String> = pack
            .files
            .iter()
            .filter_map(|f| {
                f.downloads.iter().find_map(|url| {
                    let parts: Vec<&str> = url.split('/').collect();
                    let data_idx = parts.iter().position(|&p| p == "data")?;
                    parts.get(data_idx + 1).map(|s| s.to_string())
                })
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        tracing::info!(
            "Caching {} modpack file hashes and {} project IDs for version {}",
            file_hashes.len(),
            project_ids.len(),
            version_id
        );
        CachedEntry::cache_modpack_files(
            version_id,
            file_hashes,
            project_ids,
            &state.pool,
        )
        .await?;
    } else {
        tracing::warn!(
            "No version_id available, skipping modpack file hash caching"
        );
    }

    set_instance_information(
        instance_id.clone(),
        &description,
        &pack.name,
        Some(&pack.version_id),
        &pack.dependencies,
        ignore_lock,
    )
    .await?;

    let metadata =
        crate::api::instance::get(&instance_id)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown instance {instance_id}"
                ))
            })?;
    let instance_path = metadata.instance.path.clone();
    let download_meta = DownloadMeta {
        reason,
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: metadata.applied_content_set.loader.as_str().to_string(),
        dependent_on: version_id.clone(),
    };

    let num_files = pack.files.len();
    let content_total_bytes = pack
        .files
        .iter()
        .map(|file| file.file_size as u64)
        .sum::<u64>();
    reporter
        .update_with_events(
            InstallPhaseId::DownloadingContent,
            Some(InstallProgress {
                current: 0,
                total: num_files as u64,
                secondary: (content_total_bytes > 0).then_some(
                    InstallProgressSecondary {
                        current: 0,
                        total: content_total_bytes,
                    },
                ),
            }),
            modpack_details.clone(),
            vec![InstallJobEventKind::ContentDownloadStarted {
                files: num_files as u64,
                bytes: (content_total_bytes > 0).then_some(content_total_bytes),
            }],
        )
        .await?;
    let content_progress = Arc::new(AtomicU64::new(0));
    let content_bytes_progress = Arc::new(AtomicU64::new(0));
    let active_download_bytes =
        Arc::new(Mutex::new(HashMap::<String, u64>::new()));
    let file_info_hashes = pack
        .files
        .iter()
        .filter_map(|file| {
            file.hashes.get(&PackFileHash::Sha1).map(String::as_str)
        })
        .collect::<Vec<_>>();
    let file_infos_by_hash = Arc::new(
        CachedEntry::get_file_many(
            &file_info_hashes,
            None,
            &state.pool,
            &state.api_semaphore,
        )
        .await?
        .into_iter()
        .map(|file| (file.hash.clone(), file))
        .collect::<HashMap<_, _>>(),
    );
    let content_context = ModpackContentInstallContext {
        instance_id: instance_id.clone(),
        instance_path: instance_path.clone(),
        instance_full_path: instance_full_path.clone(),
        download_meta,
        pack_version_id: version_id.clone(),
        pack_project_id: project_id.clone(),
        reporter: reporter.clone(),
        modpack_details: modpack_details.clone(),
        content_progress,
        content_bytes_progress,
        active_download_bytes,
        file_infos_by_hash,
        num_files,
        content_total_bytes,
    };
    loading_try_for_each_concurrent(
        futures::stream::iter(pack.files).map(Ok::<PackFile, crate::Error>),
        Some(MODPACK_CONTENT_DOWNLOAD_CONCURRENCY),
        None,
        70.0,
        num_files,
        None,
        |project| {
            let content_context = content_context.clone();
            async move {
                let project_size = project.file_size as u64;
                let project_path = project.path.as_str().to_string();
                let target_path = content_context
                    .instance_full_path
                    .join(project.path.as_str());

                //TODO: Future update: prompt user for optional files in a modpack
                if let Some(env) = project.env.as_ref()
                    && env
                        .get(&EnvType::Client)
                        .is_some_and(|x| x == &SideType::Unsupported)
                {
                    content_context
                        .mark_downloaded(
                            project_size,
                            InstallJobEventKind::ContentFileSkipped {
                                path: project_path,
                                reason: "unsupported on client".to_string(),
                            },
                        )
                        .await?;
                    return Ok(());
                }

                let context =
                    InstallErrorContext::new("download modpack content file")
                        .maybe_project_id(
                            content_context.pack_project_id.clone(),
                        )
                        .maybe_version_id(
                            content_context.pack_version_id.clone(),
                        )
                        .file_path(project_path.clone())
                        .target_path(target_path.display().to_string())
                        .urls(project.downloads.clone())
                        .maybe_expected_hash(
                            project.hashes.get(&PackFileHash::Sha1).cloned(),
                        )
                        .expected_size(project_size)
                        .build();
                content_context
                    .reporter
                    .set_transient_context(context.clone())
                    .await?;

                let progress_key = project_path.clone();
                let progress_context = content_context.clone();
                let min_download_progress_delta =
                    (project_size / 200).max(256 * 1024);
                let mut last_reported_downloaded = 0_u64;
                let mut report_download_progress = move |downloaded: u64,
                                                         _total_size: u64|
                      -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send>> {
                    if downloaded < project_size
                        && downloaded.saturating_sub(last_reported_downloaded)
                            < min_download_progress_delta
                    {
                        return Box::pin(async { Ok(()) });
                    }

                    last_reported_downloaded = downloaded;
                    let progress_context = progress_context.clone();
                    let progress_key = progress_key.clone();
                    Box::pin(async move {
                        let active_bytes = progress_context
                            .update_active_download(progress_key, downloaded)
                            .await;
                        let current_bytes = progress_context
                            .content_bytes_progress
                            .load(Ordering::Relaxed)
                            .saturating_add(active_bytes)
                            .min(progress_context.content_total_bytes);
                        progress_context
                            .reporter
                            .update(
                                InstallPhaseId::DownloadingContent,
                                Some(InstallProgress {
                                    current: progress_context
                                        .content_progress
                                        .load(Ordering::Relaxed),
                                    total: progress_context.num_files as u64,
                                    secondary: (progress_context
                                        .content_total_bytes
                                        > 0)
                                        .then_some(InstallProgressSecondary {
                                            current: current_bytes,
                                            total: progress_context
                                                .content_total_bytes,
                                        }),
                                }),
                                progress_context.modpack_details.clone(),
                            )
                            .await?;
                        Ok(())
                    })
                };
                let progress =
                    &mut report_download_progress as &mut FetchProgressFn<'_>;
                let file = match fetch_mirrors_with_progress(
                    &project
                        .downloads
                        .iter()
                        .map(|x| &**x)
                        .collect::<Vec<&str>>(),
                    project.hashes.get(&PackFileHash::Sha1).map(|x| &**x),
                    Some(&content_context.download_meta),
                    None,
                    &state.fetch_semaphore,
                    &state.pool,
                    Some(progress),
                )
                .await
                {
                    Ok(file) => {
                        content_context
                            .remove_active_download(&project_path)
                            .await;
                        file
                    }
                    Err(error) => {
                        content_context
                            .remove_active_download(&project_path)
                            .await;
                        content_context
                            .reporter
                            .persist_failure_context(context)
                            .await;
                        return Err(error);
                    }
                };
                let downloaded_bytes = file.len() as u64;

                let path = target_path;
				content_context
					.reporter
					.preserve_failure_context(
						context.clone(),
						write(&path, &file, &state.io_semaphore).await,
					)
					.await?;
				let modified_at_ns = crate::state::file_modified_at_ns(
					&io::metadata(&path).await?,
				)?;

                {
                    let _permit = state.install_db_semaphore.acquire().await?;
                    content_context
                        .reporter
                        .preserve_failure_context(
                            context.clone(),
                            cache_file_hash(
                                file.clone(),
                                &content_context.instance_path,
                                project.path.as_str(),
								modified_at_ns,
                                project
                                    .hashes
                                    .get(&PackFileHash::Sha1)
                                    .map(|x| &**x),
                                ProjectType::get_from_parent_folder(&path),
                                None,
                                &state.pool,
                            )
                            .await,
                        )
                        .await?;
                }

                if let Some(project_type) =
                    ProjectType::get_from_parent_folder(project.path.as_str())
                {
                    let hash =
                        project.hashes.get(&PackFileHash::Sha1).map(|x| &**x);
                    let file_info =
                        hash.and_then(|hash| {
                            content_context.file_infos_by_hash.get(hash)
                        });
                    if let Some(hash) = hash {
                        let _permit =
                            state.install_db_semaphore.acquire().await?;
                        content_context
                            .reporter
                            .preserve_failure_context(
                                context.clone(),
                                crate::state::instances::commands::record_project_file(
                                    &content_context.instance_id,
                                    project.path.as_str(),
                                    hash,
                                    project.file_size as u64,
                                    project_type,
                                    modpack_source_kind(
                                        content_context
                                            .pack_version_id
                                            .as_deref(),
                                    ),
                                    file_info.map(|file| {
                                        file.project_id.as_str()
                                    }),
                                    file_info.map(|file| {
                                        file.version_id.as_str()
                                    }),
                                    state,
                                )
                                .await,
                            )
                            .await?;
                    }
                }

                content_context
                    .mark_downloaded(
                        project_size,
                        InstallJobEventKind::ContentFileCompleted {
                            path: project_path,
                            bytes: downloaded_bytes,
                        },
                    )
                    .await?;
                Ok(())
            }
        },
    )
    .await?;

    let override_file_entries = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, file)| {
            let filename = file.filename().as_str().unwrap_or_default();
            ((filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/"))
                && !filename.ends_with('/'))
            .then(|| (index, file.clone()))
        })
        .collect::<Vec<_>>();
    let override_total_bytes = override_file_entries
        .iter()
        .map(|(_, file)| file.uncompressed_size())
        .sum::<u64>();
    let progress = (override_total_bytes > 0).then_some(InstallProgress {
        current: 0,
        total: override_total_bytes,
        secondary: None,
    });
    reporter
        .update(
            InstallPhaseId::ExtractingOverrides,
            progress,
            modpack_details.clone(),
        )
        .await?;

    let extracted_override_bytes = Arc::new(AtomicU64::new(0));
    let mut last_reported_override_bytes = 0_u64;
    let reporter_for_overrides = reporter.clone();
    let details_for_overrides = modpack_details.clone();
    let mut report_override_progress = |bytes_read: u64| -> Pin<
        Box<dyn Future<Output = crate::Result<()>> + Send>,
    > {
        let current = extracted_override_bytes
            .fetch_add(bytes_read, Ordering::Relaxed)
            + bytes_read;
        let min_delta = (override_total_bytes / 200).max(256 * 1024);
        if current < override_total_bytes
            && current.saturating_sub(last_reported_override_bytes) < min_delta
        {
            return Box::pin(async { Ok(()) });
        }

        last_reported_override_bytes = current;
        let reporter = reporter_for_overrides.clone();
        let details = details_for_overrides.clone();
        Box::pin(async move {
            reporter
                .update(
                    InstallPhaseId::ExtractingOverrides,
                    Some(InstallProgress {
                        current: current.min(override_total_bytes),
                        total: override_total_bytes,
                        secondary: None,
                    }),
                    details,
                )
                .await?;
            Ok(())
        })
    };

    for (index, file) in override_file_entries {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(
                file.filename().as_str().unwrap().to_string(),
            )?;
        let relative_override_file_path = relative_override_file_path
            .strip_prefix("overrides")
            .or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::OtherError(
                    format!("Failed to strip override prefix from override file path: {relative_override_file_path}")
                ))
            })?;

        let path =
            instance_full_path.join(relative_override_file_path.as_str());
        let override_context =
            InstallErrorContext::new("extract modpack override")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .entry_path(file.filename().as_str().unwrap_or_default())
                .target_path(path.display().to_string())
                .build();
        reporter
            .set_transient_context(override_context.clone())
            .await?;
        let extract_result = if override_total_bytes > 0 {
            let progress =
                &mut report_override_progress as &mut ExtractProgressFn<'_>;
            zip_reader
                .extract_entry(
                    index,
                    &path,
                    &state.io_semaphore,
                    Some(progress),
                )
                .await
        } else {
            zip_reader
                .extract_entry(index, &path, &state.io_semaphore, None)
                .await
        };
        let (size, hash) = reporter
            .preserve_failure_context(override_context, extract_result)
            .await?;
        let modified_at_ns =
            crate::state::file_modified_at_ns(&io::metadata(&path).await?)?;

        {
            let _permit = state.install_db_semaphore.acquire().await?;
            let record_context =
                InstallErrorContext::new("record modpack override")
                    .maybe_project_id(project_id.clone())
                    .maybe_version_id(version_id.clone())
                    .source_path(source_path.clone())
                    .entry_path(file.filename().as_str().unwrap_or_default())
                    .target_path(path.display().to_string())
                    .build();
            reporter
                .preserve_failure_context(
                    record_context.clone(),
                    crate::state::cache_file_hash_metadata(
                        &instance_path,
                        relative_override_file_path.as_str(),
                        size,
                        modified_at_ns,
                        hash.clone(),
                        ProjectType::get_from_parent_folder(
                            relative_override_file_path.as_str(),
                        ),
                        None,
                        &state.pool,
                    )
                    .await,
                )
                .await?;

            if let Some(project_type) = ProjectType::get_from_parent_folder(
                relative_override_file_path.as_str(),
            ) {
                reporter
                    .preserve_failure_context(
                        record_context,
                        crate::state::instances::commands::record_project_file(
                            &instance_id,
                            relative_override_file_path.as_str(),
                            &hash,
                            size,
                            project_type,
                            modpack_source_kind(version_id.as_deref()),
                            None,
                            None,
                            state,
                        )
                        .await,
                    )
                    .await?;
            }
        }
    }

    // If the icon doesn't exist, we expect icon.png to be a potential icon.
    // If it doesn't exist, and an override to icon.png exists, cache and use that
    let potential_icon = instance_full_path.join("icon.png");
    if !icon_exists && potential_icon.exists() {
        crate::api::instance::edit_icon(&instance_id, Some(&potential_icon))
            .await?;
    }

    crate::launcher::install_minecraft_for_instance_id_with_reporter(
        &instance_id,
        false,
        Some(reporter.clone()),
    )
    .await?;
    reporter.clear_context().await?;

    Ok::<String, crate::Error>(instance_id.clone())
}

fn pack_source_path(file: &CreatePackFile) -> String {
    match file {
        CreatePackFile::Bytes(_) => "downloaded mrpack bytes".to_string(),
        CreatePackFile::Path(path) => path.display().to_string(),
    }
}

fn modpack_source_kind(version_id: Option<&str>) -> ContentSourceKind {
    if version_id.is_some() {
        ContentSourceKind::ModrinthModpack
    } else {
        ContentSourceKind::ImportedModpack
    }
}

#[tracing::instrument(skip(mrpack_file))]

pub async fn remove_all_related_files(
    instance_id: String,
    mrpack_file: CreatePackFile,
) -> crate::Result<()> {
    // Updates can remove files from a locally imported or downloaded pack, so share the same reader path.
    let mut zip_reader = MrpackZipReader::new(&mrpack_file).await?;

    let mr_manifest_idx = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("modrinth.index.json"))
    });
    let cf_manifest_idx = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("manifest.json"))
    });

    let pack: PackFormat = if let Some(manifest_idx) = mr_manifest_idx {
        let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;
        serde_json::from_str(&manifest)?
    } else if let Some(manifest_idx) = cf_manifest_idx {
        let manifest_str = zip_reader.read_entry_to_string(manifest_idx).await?;
        let cf_manifest: CurseForgeManifest = serde_json::from_str(&manifest_str)?;
        convert_cf_manifest_to_pack_format(cf_manifest, None, None).await?
    } else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in modpack archive".to_string(),
        )));
    };

    if &*pack.game != "minecraft" {
        return Err(crate::ErrorKind::InputError(
            "Pack does not support Minecraft".to_string(),
        )
        .into());
    }

    crate::api::instance::edit(
        &instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            ..EditInstance::default()
        },
    )
    .await?;

    // First, remove all modrinth projects by their version hashes
    // Remove all modrinth projects by their version hashes
    // We need to do a fetch to get the project ids from Modrinth
    let state = State::get().await?;
    let metadata =
        crate::api::instance::get(&instance_id)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown instance {instance_id}"
                ))
            })?;
    let instance_full_path =
        crate::api::instance::get_full_path(&instance_id).await?;
    let all_hashes = pack
        .files
        .iter()
        .filter_map(|f| Some(f.hashes.get(&PackFileHash::Sha1)?.clone()))
        .collect::<Vec<_>>();

    // First, get project info by hash
    let file_infos = CachedEntry::get_file_many(
        &all_hashes.iter().map(|x| &**x).collect::<Vec<_>>(),
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    let to_remove = file_infos
        .into_iter()
        .map(|p| p.project_id)
        .collect::<Vec<_>>();

    for file in crate::state::instances::commands::list_project_files(
        &metadata.instance.id,
        &state,
    )
    .await?
    {
        if let Some(project_id) = &file.project_id
            && to_remove.contains(project_id)
        {
            crate::state::instances::commands::remove_project(
                &metadata.instance.id,
                &file.relative_path,
                &state,
            )
            .await?;
        }
    }

    // Iterate over all Modrinth project file paths in the json, and remove them
    // (There should be few, but this removes any files the .mrpack intended as Modrinth projects but were unrecognized)
    for file in pack.files {
        match io::remove_file(instance_full_path.join(file.path.as_str())).await
        {
            Ok(_) => (),
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }
    }

    // Iterate over each 'overrides' file and remove it
    let override_file_entries =
        zip_reader.file().entries().iter().filter(|file| {
            let filename = file.filename().as_str().unwrap_or_default();
            (filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/"))
                && !filename.ends_with('/')
        });

    for file in override_file_entries {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(
                file.filename().as_str().unwrap().to_string(),
            )?;
        let relative_override_file_path = relative_override_file_path
            .strip_prefix("overrides")
            .or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::OtherError(
                    format!("Failed to strip override prefix from override file path: {relative_override_file_path}")
                ))
            })?;

        // Remove this file if a corresponding one exists in the filesystem
        match io::remove_file(
            instance_full_path.join(relative_override_file_path.as_str()),
        )
        .await
        {
            Ok(_) => (),
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
