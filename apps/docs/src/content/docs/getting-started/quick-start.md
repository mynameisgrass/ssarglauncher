---
title: Quick Start Guide
description: Configure your first Minecraft instance and deploy modpacks across Modrinth and CurseForge.
sidebar:
  order: 2
---

This guide provides step-by-step instructions for deploying Minecraft instances, searching multi-ecosystem modpack repositories, and managing automated Java runtime environments within Ssarg Launcher.

---

## 1. Searching & Installing Modpacks

Ssarg Launcher integrates both the **Modrinth** and **CurseForge** modpack repositories into a single unified search interface.

1. Open **Ssarg Launcher** and select the **Browse** or **Search** view from the primary navigation bar.
2. Enter your target modpack query (for example, *All the Mods 10*, *RLCraft*, or *Fabrishred*).
3. Identify the origin repository via the platform indicator on each project card:
   - **Modrinth**: Native `.mrpack` archive format.
   - **CurseForge**: Standard `.zip` archive format.
4. Click **Install** on your target modpack. Ssarg Launcher will resolve dependencies, download project archives, and initialize an isolated Minecraft instance directory.

---

## 2. Automated Runtime (JDK) Bootstrapping

Ssarg Launcher manages Java Virtual Machine (JVM) requirements automatically without requiring global system Java installations or manual path configurations.

- When an instance is launched, the core engine detects the exact Java runtime version mandated by the Minecraft version (Java 8 for Minecraft 1.12.2 and earlier, Java 17 for 1.18 through 1.20.4, or Java 21 for 1.20.5+).
- If the required JDK is missing from the local runtime cache, the engine initiates a multi-threaded 8x parallel download of the official Eclipse Temurin runtime.
- Download speed, chunk verification, and archive extraction progress are reported in real time.

---

## 3. Account Authentication & Custom Skins

Ssarg Launcher supports both online Microsoft authentication and offline profile sessions.

- **Microsoft Accounts**: Select **Add Microsoft Account** to authenticate securely via OAuth2 and access official online multiplayer servers.
- **Offline Profiles & Ely.by Skins**: For offline play or non-Microsoft sessions, create a local profile name. Offline profiles integrate with the third-party **Ely.by** skin ecosystem when the lightweight Ely.by Skins mod is present in your modpack.

---

## 4. Launching Instances

1. Navigate to the **Library** view.
2. Select your configured instance and click **Play**.
3. Monitor console output and memory allocation in real time via the built-in diagnostic console.
