# 🎮 Ssarg Launcher

### The Ultimate Ad-Free, Cross-Platform Minecraft & Modpack Launcher

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](COPYING.md)
[![Built with Rust & Tauri](https://img.shields.io/badge/Built%20with-Rust%20%7C%20Tauri-orange.svg)](https://tauri.app)
[![Vue 3](https://img.shields.io/badge/Frontend-Vue%203%20%7C%20TypeScript-4FC08D.svg)](https://vuejs.org)
[![Zero Telemetry](https://img.shields.io/badge/Telemetry-100%25%20Disabled-success.svg)](#)
[![Zero Ads](https://img.shields.io/badge/Ads-0%25-success.svg)](#)

---

## 🌟 Overview

**Ssarg Launcher** is a lightning-fast, modern, open-source Minecraft launcher designed to deliver the best possible user experience without compromise. By integrating both **Modrinth** and **CurseForge** into a single unified interface, Ssarg Launcher lets you discover, install, and manage modpacks, mods, shaders, and resource packs seamlessly—all with **zero advertisements, zero telemetry, and a clean console**.

---

## ✨ Key Features

### 🌐 Unified Modrinth + CurseForge Search
- **Best of Both Worlds**: Search and explore content across both Modrinth and CurseForge simultaneously.
- **Smart Download Ranking**: Mixed search results are automatically sorted by download count descending (`20M+ → 3M+ → 100K+...`), ensuring the most popular packs (e.g., *All the Mods 10*, *FTB*, *RLCraft*) are always at the top.
- **Native Rust IPC**: Searches bypass browser Content Security Policy (CSP) limitations using native Rust `reqwest` integration.

### 🎨 Vibrant Modern UI & Platform Badges
- **Visual Excellence**: Clean dark-mode aesthetic with custom HSL platform badges.
- **Priority Badge Placement**: The vibrant **Orange `CurseForge`** and **Green `Modrinth`** badges are guaranteed to always appear at index 0—the very first tag on every project card.

### ☕ 1-Click Auto-JDK Installer
- **Flawless Java Setup**: Never worry about matching Java versions again.
- **Parallel Chunk Downloading**: Downloads and extracts required Java Development Kits (Java 8, 17, 21) with 8x parallel downloading and real-time UI progress bars.
- **Automatic Instance Configuration**: Automatically assigns the downloaded JDK path to your Minecraft instance.

### 🚫 100% Ad-Free & Zero Telemetry
- **Complete Privacy**: All third-party telemetry (PostHog, Sentry) is completely disabled.
- **Clean Console**: No network CORS errors, no tracking scripts, and dummy no-op ad loaders.

### 📥 Seamless Modpack Installation
- **Universal Archive Support**: Installs both Modrinth `.mrpack` files and CurseForge `.zip` modpacks seamlessly by downloading archives to a system temporary path before invoking the Rust instance builder.

---

## 🚀 Getting Started (Local Development & Building)

### Prerequisites
- **Node.js**: v20 or higher
- **pnpm**: v9 or higher (`npm install -g pnpm`)
- **Rust & Cargo**: Latest stable toolchain ([rustup.rs](https://rustup.rs/))
- **Windows Build Tools** *(Windows only)*: Visual Studio C++ build tools / WiX toolset (for MSI/NSIS installers)

### 1. Clone the Repository
```bash
git clone https://github.com/your-username/ssarg-launcher.git
cd ssarg-launcher
```

### 2. Install Dependencies
```bash
pnpm install
```

### 3. Run in Development Mode
Launch the Vite hot-reload frontend alongside the native Tauri desktop runtime:
```bash
pnpm app:dev
```

### 4. Build Production Release (Multi-Platform Binary & Installers)
Because Tauri compiles native desktop applications, running the build command generates the bundle for your current operating system:

```bash
pnpm --filter @modrinth/app run build
```

Once finished, your compiled build artifacts appear in `target/release/` and `target/release/bundle/`:

| Operating System | Standalone Binary | Setup / Installer Bundle |
| :--- | :--- | :--- |
| **Windows (x64)** | `target/release/Ssarg Launcher.exe` | `target/release/bundle/nsis/Ssarg Launcher_<version>_x64-setup.exe` |
| **macOS (Universal / Apple Silicon & Intel)** | `target/release/Ssarg Launcher` | `target/release/bundle/dmg/Ssarg Launcher_<version>_universal.dmg` |
| **Linux (x64 / Debian & RPM)** | `target/release/Ssarg Launcher` | `target/release/bundle/appimage/Ssarg Launcher_<version>_amd64.AppImage` |

> [!TIP]
> **How to build for all 3 platforms (Windows, macOS, Linux) without owning a Mac or Linux PC:**  
> You don't need separate computers! When you push a version tag to GitHub (e.g., `git tag v1.0.0 && git push --tags`), our pre-configured **GitHub Actions CI/CD Release Workflow** (`.github/workflows/build-release.yml`) automatically spins up Windows, macOS, and Linux cloud runners, compiles all three platforms in parallel, and publishes the `.exe`, `.dmg`, and `.AppImage` files directly to your official GitHub Releases page!

---

## 📦 Automated GitHub Actions CI/CD Releases

This repository includes a pre-configured GitHub Actions workflow (`.github/workflows/build-release.yml`) for automated multi-platform builds and releases:

- **Automated Releases**: When you push a version tag (e.g., `git tag v1.0.0 && git push --tags`), GitHub Actions automatically compiles production bundles for **Windows (x64)**, **macOS (Universal Apple Silicon & Intel)**, and **Linux (x64 AppImage/deb/rpm)** and publishes them as an official GitHub Release.
- **Manual Trigger**: You can also trigger a build anytime by clicking the **"Run workflow"** button on the `Build and Release Ssarg Launcher` workflow in the GitHub Actions tab.

---

## 🤝 Contributing

We welcome community contributions!
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m "feat: add amazing feature"`)
4. Push to your branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is open-source and licensed under the **GPL-3.0 License**. See the [COPYING.md](COPYING.md) file for details.
