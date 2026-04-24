###
<p align="center">
  <img src="img/logo.png" width="300" alt="OxideMC Logo">
  <br>
  <b>A high-performance, lightweight Minecraft engine core written in Rust.</b>
</p>

---

OxideMC is a lightweight Minecraft manager designed for efficiency and performance. Built from the ground up in **Rust**, it is specifically crafted to be compatible with the **Tauri framework**.

Currently, it serves as a full installer and injector for **Fabric 1.20.1**, with support for **NeoForge** coming soon.

## 🚀 Why OxideMC?

Most Minecraft launchers are resource-heavy. OxideMC is different:

- **Low RAM Footprint:** Designed exclusively for low-end devices (like 4GB RAM systems).
- **Native Speed:** Written in Rust to ensure the launcher doesn't steal resources from the game.
- **Maximum Performance:** Leaves more CPU and RAM available for Minecraft and the JVM to perform at their best.
- **Tauri Ready:** Optimized to work as a backend for modern, lightweight desktop GUIs.

## ✨ Features

- ✅ **Full Installation:** Downloads libraries, assets, and the game client.
- ✅ **Fabric Injection:** Seamlessly integrates Fabric Loader.
- ✅ **Modpack Support:** Inject custom mods, configs, and shaders via URL.
- ✅ **Smart Runtime:** Automatic Java version detection and portable installation.

## 🛠️ Installation (Dependency)

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxide-mc = { git = "https://github.com/S3fflexDev/oxide-mc.git" }