---
title: Download & Install
description: Native installation packages and deployment commands for Linux, Windows, and macOS.
sidebar:
  order: 1
---

Ssarg Launcher provides native installation artifacts for all supported desktop operating systems. Choose your target platform below for package downloads or automated command-line installation.

---

## Linux

Ssarg Launcher is distributed for Linux via universal AppImage archives, Debian packages, and RPM packages.

### Universal AppImage

The `.AppImage` distribution executes natively across modern Linux environments including Fedora, Arch Linux, Ubuntu, Debian, and Steam OS:

```bash
mkdir -p ~/Applications && curl -L -o ~/Applications/SsargLauncher.AppImage "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_amd64.AppImage" && chmod +x ~/Applications/SsargLauncher.AppImage && ~/Applications/SsargLauncher.AppImage
```

> **Fedora Systems Notice**: If your Fedora environment lacks user-space FUSE libraries, install `fuse-libs` before executing the AppImage:
> ```bash
> sudo dnf install -y fuse-libs
> ```

### Debian & Ubuntu (.deb)

Install the Debian package directly via `apt`:

```bash
curl -L -o /tmp/ssarg.deb "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_amd64.deb" && sudo apt install -y /tmp/ssarg.deb
```

### Fedora & RHEL (.rpm)

Install the RPM package via `dnf`:

```bash
sudo dnf install -y --nogpgcheck "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher-1.1.0-1.x86_64.rpm"
```

---

## Windows (10 & 11)

### Standard Setup Executable

1. Download **`Ssarg.Launcher_1.1.0_x64-setup.exe`** from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest).
2. Execute the setup installer to deploy Ssarg Launcher to your system.
3. *If Microsoft Defender SmartScreen displays an unrecognized publisher prompt, select **More info → Run anyway**.*

### PowerShell Automated Deployment

Run this command in an elevated Windows PowerShell terminal to download and launch the installer automatically:

```powershell
Invoke-WebRequest -Uri "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_x64-setup.exe" -OutFile "$env:TEMP\SsargLauncherSetup.exe"; & "$env:TEMP\SsargLauncherSetup.exe"
```

---

## macOS (Apple Silicon & Intel)

### Universal Disk Image (.dmg)

1. Download **`Ssarg.Launcher_1.1.0_universal.dmg`** from [GitHub Releases](https://github.com/mynameisgrass/ssarglauncher/releases/latest).
2. Mount the DMG volume and drag **Ssarg Launcher** into your `/Applications` directory.
3. *If macOS Gatekeeper prevents execution of unnotarized binaries, right-click (or Control-click) the application in `/Applications` and select **Open → Open**.*

### Terminal Automated Deployment

Execute this command in Terminal to download, mount, and deploy Ssarg Launcher to `/Applications` automatically:

```bash
curl -L -o /tmp/SsargLauncher.dmg "https://github.com/mynameisgrass/ssarglauncher/releases/latest/download/Ssarg.Launcher_1.1.0_universal.dmg" && hdiutil attach -nobrowse -quiet /tmp/SsargLauncher.dmg && ditto "/Volumes/Ssarg Launcher/Ssarg Launcher.app" "/Applications/Ssarg Launcher.app" && hdiutil detach -quiet "/Volumes/Ssarg Launcher" && rm /tmp/SsargLauncher.dmg
```

---

## Verification & Integrity

All release builds are automatically compiled and published by our GitHub Actions release pipeline. Check sums and SHA-256 signatures are provided on the [Releases page](https://github.com/mynameisgrass/ssarglauncher/releases/latest) for cryptographic verification.
