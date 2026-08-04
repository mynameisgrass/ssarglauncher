---
title: Technical Features & Architecture
description: Comprehensive overview of the core systems and capabilities engineered into Ssarg Launcher.
sidebar:
  order: 1
---

Ssarg Launcher is engineered from the ground up to deliver a high-performance, clutter-free Minecraft modding environment. By utilizing systems-level Rust programming and a lightweight web view frontend, the application avoids the memory overhead, advertising frameworks, and telemetry SDKs prevalent in standard desktop launchers.

---

## Multi-Ecosystem Content Federation

Rather than isolating users within a single mod repository, Ssarg Launcher queries both **Modrinth** and **CurseForge** concurrently:

- **Native Systems HTTP Layer**: Search requests bypass browser Content Security Policy (CSP) restrictions and CORS limits by executing asynchronous, multi-threaded requests in the Rust core (`reqwest`).
- **Clear Provenance Badges**: Search results display an unambiguous repository badge on each card, providing immediate clarity on whether a modpack or mod originates from Modrinth or CurseForge.

---

## Normalized Download Sorting

When aggregating content across distinct mod databases, accurate sorting is critical for discovery. Ssarg Launcher normalizes download metrics across both ecosystems and ranks results by **total download count descending**:

- Premier modpacks and libraries with high community adoption (`20M+ → 5M+ → 500K+`) consistently appear at the top of query results.
- Unverified clone projects and low-relevance artifacts are naturally pushed down the ranking hierarchy.

---

## Strict Privacy & Zero-Ad Guarantee

Ssarg Launcher is built around a strict commitment to user privacy and system integrity:

- **Zero Advertisement Containers**: The interface contains no banner ads, sponsored placements, or external marketing frames.
- **Stripped Analytics**: Third-party analytics and data collection SDKs are completely excluded from the runtime.
- **Clean Network Profile**: Background network activity is strictly limited to content repository queries, Java runtime bootstrapping, and official Mojang authentication.

---

## Lightweight Tauri & Rust Core

Unlike traditional Electron-based desktop applications that package a redundant Chromium browser and Node.js runtime, Ssarg Launcher is built on **Tauri 2.0** and **Rust**:

- **Minimal Memory Overhead**: System RAM consumption is minimized, preserving host resources for JVM execution and high-memory Minecraft modpacks.
- **Instant Bootstrapping**: Cold application start times are optimized across Linux, Windows, and macOS.
