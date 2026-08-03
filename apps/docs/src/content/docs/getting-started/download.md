---
title: Download & Install
description: Download Ssarg Launcher for Linux, Windows, and macOS.
sidebar:
  order: 1
---

Get started with **Ssarg Launcher** in seconds! Choose your operating system below for 1-click downloads or 1-command terminal installers.

---

## 🐧 Linux (Recommended)

Ssarg Launcher provides native packages for all major Linux distributions.

### Universal AppImage (1-Command Install)
The `.AppImage` works universally across Linux distributions including **Fedora, Arch Linux, Ubuntu, Debian, and Steam Deck**:

```bash
mkdir -p ~/Applications && curl -L -o ~/Applications/SsargLauncher.AppImage "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.0.0_amd64.AppImage" && chmod +x ~/Applications/SsargLauncher.AppImage && ~/Applications/SsargLauncher.AppImage
```

> **Note for Fedora Users**: If your Fedora system is fresh and lacks FUSE support, install `fuse-libs` first by running:
> ```bash
> sudo dnf install -y fuse-libs
> ```

### Ubuntu / Debian / Linux Mint (`.deb`)
Download and install the Debian package in one command:

```bash
curl -L -o /tmp/ssarg.deb "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.0.0_amd64.deb" && sudo apt install -y /tmp/ssarg.deb
```

### Fedora / RHEL / CentOS (`.rpm`)
Install the RPM package directly via `dnf`:

```bash
sudo dnf install -y --nogpgcheck "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher-1.0.0-1.x86_64.rpm"
```

---

## 🪟 Windows (10 & 11)

### Option 1: 1-Click Setup Executable
1. Download **`Ssarg.Launcher_<version>_x64-setup.exe`** from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest).
2. Double-click the installer to install Ssarg Launcher to your system.
3. *If Microsoft SmartScreen says "Windows protected your PC", click **More info → Run anyway**.*

### Option 2: PowerShell 1-Command Install
Run this command in Windows PowerShell to download and launch the installer automatically:

```powershell
Invoke-WebRequest -Uri "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.0.0_x64-setup.exe" -OutFile "$env:TEMP\SsargLauncherSetup.exe"; & "$env:TEMP\SsargLauncherSetup.exe"
```

---

## 🍎 macOS (Apple Silicon & Intel)

### Option 1: Universal DMG Installer
1. Download **`Ssarg.Launcher_<version>_universal.dmg`** from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest).
2. Open the DMG file and drag **Ssarg Launcher** into your `Applications` folder.
3. *When launching for the first time, if macOS Gatekeeper says "developer cannot be verified", right-click (or two-finger tap) the app in Applications and select **Open → Open**.*

### Option 2: Terminal 1-Command Install
Run this command in Terminal to download, mount, and copy Ssarg Launcher to `/Applications` automatically:

```bash
curl -L -o /tmp/SsargLauncher.dmg "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.0.0_universal.dmg" && hdiutil attach /tmp/SsargLauncher.dmg && cp -R "/Volumes/Ssarg Launcher/Ssarg Launcher.app" /Applications/ && hdiutil detach "/Volumes/Ssarg Launcher"
```

---

## 📦 Verifying Your Download

All official builds of Ssarg Launcher are compiled automatically via GitHub Actions from our [public repository](https://github.com/mynameisgrass/ssarglauncher). We never bundle adware, toolbars, or telemetry.
