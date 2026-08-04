# Ssarg Launcher

[![Typing SVG](https://readme-typing-svg.demolab.com?font=Fira+Code&pause=1000&color=F75C7E&width=435&lines=Welcome+to+Ssarg+Launcher!;Fast,+scalable,+and+open+source.)](https://git.io/typing-svg)

![GitHub repo stars](https://img.shields.io/github/stars/mynameisgrass/ssarglauncher?style=for-the-badge&logo=github&color=ff69b4)
![GitHub forks](https://img.shields.io/github/forks/mynameisgrass/ssarglauncher?style=for-the-badge&logo=github&color=blue)
![GitHub last commit](https://img.shields.io/github/last-commit/mynameisgrass/ssarglauncher?style=for-the-badge&color=green)

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![Vue.js](https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)

---

## Executive Summary

**Ssarg Launcher** is an advanced, high-performance, open-source desktop application designed for managing Minecraft instances, modpacks, mods, resource packs, and shaders across multiple operating systems. Built on a native **Rust and Tauri** core architecture with a responsive **Vue 3 and TypeScript** user interface, Ssarg Launcher delivers native execution speed and minimal resource overhead.

The platform natively integrates both the **Modrinth** and **CurseForge** ecosystems into a single, cohesive workflow while eliminating advertisements, tracking telemetry, and bloatware.

---

## Core Capabilities

| Feature | Technical Implementation | Benefit |
| :--- | :--- | :--- |
| **Unified Ecosystem Access** | Integrated Modrinth & CurseForge GraphQL and REST APIs | Seamless discovery and one-click installation of content from major mod repositories |
| **High-Performance Core** | Native Rust systems programming via Tauri 2.0 runtime | Fast instance bootstrapping, minimal memory footprint, and rapid file verification |
| **Zero Ad & Telemetry Policy** | Strictly local execution without analytics or ad network integrations | Maximum privacy, reduced bandwidth usage, and clutter-free interface |
| **Automated Crash Diagnostics** | Real-time log parsing with heuristic pattern matching | Immediate root-cause analysis for mod conflicts, missing dependencies, and JVM exceptions |
| **Cross-Platform Compatibility** | Native builds for Linux (AppImage, deb, rpm), Windows (exe), and macOS (dmg) | Consistent, first-class user experience across all supported desktop platforms |
| **Offline Custom Skins** | Seamless compatibility with Ely.by and offline authentication modes | Support for custom player skins even when playing without an active online profile |

---

## Technical Architecture

Ssarg Launcher separates systems-level resource management from user interface rendering:

1. **Backend Engine (Rust + Tauri)**:
   - Handles JVM downloading and process orchestration.
   - Executes multi-threaded modpack downloading, hashing, and delta patching.
   - Manages secure credential storage and local filesystem transactions.
2. **Frontend Interface (Vue 3 + TypeScript)**:
   - Responsive, component-driven UI built for high frame rates and low input latency.
   - Fully type-safe state management across the Tauri IPC bridge.

---

## Installation

### Linux

Download and run the recommended `.AppImage`, or install via package manager:

- **Universal Linux (.AppImage - Recommended)**
  ```bash
  mkdir -p ~/Applications && curl -L -o ~/Applications/SsargLauncher.AppImage "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_amd64.AppImage" && chmod +x ~/Applications/SsargLauncher.AppImage && ~/Applications/SsargLauncher.AppImage
  ```
  *(Note for fresh Fedora installations: Install FUSE support first via `sudo dnf install -y fuse-libs` if not present).*

- **Debian / Ubuntu (.deb)**
  ```bash
  curl -L -o /tmp/ssarg.deb "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_amd64.deb" && sudo apt install -y /tmp/ssarg.deb
  ```

- **Fedora / RHEL (.rpm)**
  ```bash
  sudo dnf install -y --nogpgcheck "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher-1.1.0-1.x86_64.rpm"
  ```

### Windows

- **Standard Installer**: Download `Ssarg.Launcher_<version>_x64-setup.exe` from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest).
- **PowerShell One-Command Setup**:
  ```powershell
  Invoke-WebRequest -Uri "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_x64-setup.exe" -OutFile "$env:TEMP\SsargLauncherSetup.exe"; & "$env:TEMP\SsargLauncherSetup.exe"
  ```

### macOS

- **Universal DMG**: Download `Ssarg.Launcher_<version>_universal.dmg` from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest) and drag to `/Applications`.
- **Terminal One-Command Setup**:
  ```bash
  curl -L -o /tmp/SsargLauncher.dmg "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_universal.dmg" && hdiutil attach -nobrowse -quiet /tmp/SsargLauncher.dmg && ditto "/Volumes/Ssarg Launcher/Ssarg Launcher.app" "/Applications/Ssarg Launcher.app" && hdiutil detach -quiet "/Volumes/Ssarg Launcher" && rm /tmp/SsargLauncher.dmg
  ```

---

## Repository Analytics

### Project Statistics

<p align="center">
  <a href="https://github.com/mynameisgrass/ssarglauncher">
    <img src="https://github-readme-stats.anuraghazra1.vercel.app/api/pin/?username=mynameisgrass&repo=ssarglauncher&theme=dark" alt="Repository Stats" />
  </a>
</p>

### Developer Statistics & Activity

<p align="center">
  <a href="https://github.com/mynameisgrass">
    <img src="https://github-readme-stats.anuraghazra1.vercel.app/api?username=mynameisgrass&show_icons=true&theme=dark" alt="GitHub Stats" />
  </a>
  <a href="https://github.com/mynameisgrass">
    <img src="https://github-readme-streak-stats.herokuapp.com/?user=mynameisgrass&theme=dark" alt="GitHub Streak" />
  </a>
</p>

### Commit Pulse

<p align="center">
  <img src="https://repobeats.axiom.co/api/embed/a1df13d38a7d21ac7de9ae1f146e16681ad5eba8.svg" alt="Repobeats analytics image" />
</p>

---

## Documentation & Support

- **Official Website & Docs**: [launcher.grassist.me](https://launcher.grassist.me)
- **Issue Tracker**: [GitHub Issues](https://github.com/mynameisgrass/ssarglauncher/issues)

---

## License

This software is distributed under the terms of the [GNU General Public License v3.0](COPYING.md). See `COPYING.md` for complete licensing information.
