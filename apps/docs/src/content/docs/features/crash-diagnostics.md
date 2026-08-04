---
title: Automated Crash Diagnostics
description: Real-time stack trace analysis and heuristic error identification implemented in Rust.
sidebar:
  order: 2
---

Modded Minecraft environments frequently encounter complex dependency conflicts, duplicate class loaders, and bytecode injection failures. Ssarg Launcher embeds an automated **Crash Diagnostics Engine** written in Rust that parses log files and crash traces to provide immediate root-cause remediation.

---

## Technical Mechanism

When a Minecraft process terminates unexpectedly, the Ssarg Launcher core engine performs a structured diagnostic sweep:

1. **Crash Report Parsing**: The engine scans the instance's `crash-reports` directory for the latest generated `.txt` report produced by the Forge, Fabric, or NeoForge bootstrapper.
2. **Log File Inspection**: If a formal crash report was not written to disk, the engine falls back to parsing `logs/latest.log` to identify fatal JVM exceptions, thread panics, and stack traces.

All parsing and heuristic matching execute locally on the host machine. Log contents are never transmitted to external servers.

---

## Diagnostic Heuristics & Advice

Rather than requiring users to manually analyze verbose Java stack traces, the engine renders an inline diagnostic banner on the instance control panel with specific remediation guidance:

### 1. Missing Library Dependencies

When a mod fails to initialize because a required dependency is absent from the classpath (for example, **Sodium** requiring **Indium** when rendering add-ons are installed), the engine identifies the missing artifact:

> **Diagnostic Finding**: Missing required dependency `indium` for mod `sodium`. Install the missing library to proceed.

### 2. Duplicate Mod Identifiers

When redundant JAR files or conflicting mod versions are present in the instance `mods/` directory:

> **Diagnostic Finding**: Duplicate mod ID detected (`fabric-api`). Remove redundant JAR archives from the instance directory.

### 3. Mixin Bytecode Transformation Conflicts

When multiple mods attempt incompatible bytecode transformations on the same internal Minecraft class:

> **Diagnostic Finding**: Mixin transformation conflict in target class `net.minecraft.client.render.WorldRenderer`. Check recently installed rendering or optimization mods.
