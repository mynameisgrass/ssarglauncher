---
title: Why Ssarg Launcher?
description: Explore the core features that make Ssarg Launcher the best Minecraft launcher.
sidebar:
  order: 1
---

**Ssarg Launcher** was engineered from the ground up to provide the ultimate Minecraft modding experience without the clutter, advertisements, or tracking scripts found in typical third-party launchers.

---

## 🌐 Unified Modrinth + CurseForge Search

Instead of forcing you to choose between platforms or switch back and forth between different apps, Ssarg Launcher queries both **Modrinth** and **CurseForge** simultaneously:

- **Native Rust IPC**: Searches bypass browser Content Security Policy (CSP) limitations by performing fast concurrent HTTP requests in native Rust (`reqwest`).
- **Platform Badges**: Every project card prominently displays its platform badge (**Orange `CurseForge`** or **Green `Modrinth`**) at tag position `0`, so you always know where a project originates.

---

## ⭐ Smart Download Ranking

When searching across two massive mod databases, ranking matters. Ssarg Launcher automatically merges results from both platforms and sorts them by **total download count descending**:

- Popular modpacks with millions of downloads (`20M+ → 5M+ → 500K+...`) always rise to the top.
- Duplicate or low-quality clone projects are pushed to the bottom of search results.

---

## 🚫 100% Ad-Free & Zero Telemetry

We believe your gaming launcher should respect your computer and your privacy:

- **No Banner Ads or Video Ads**: The UI contains zero advertisement containers or promoted sponsored blocks.
- **Zero Telemetry**: Third-party analytics SDKs (PostHog, Sentry, Google Analytics) have been completely stripped and disabled.
- **Clean Console**: No CORS errors, network timeouts, or tracking scripts in the developer console.

---

## ⚡ Native Rust & Tauri Speed

Unlike traditional Electron-based apps that consume hundreds of megabytes of RAM just to sit idle, Ssarg Launcher is built on **Tauri** and **Rust**:

- **Lightweight Memory Footprint**: Minimal overhead so more RAM is reserved for your actual Minecraft game.
- **Fast Startup**: Opens instantly on Linux, Windows, and macOS.
