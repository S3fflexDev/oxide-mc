###
<p align="center">
  <img src="img/logo.png" width="450" alt="OxideMC Logo">
</p>

<p align="center">
  <strong>A fast and minimal Minecraft launcher library for Rust.</strong>
  <br />
  Designed for speed, low-end devices, and seamless Tauri integration.
</p>

<p align="center">
  <a href="https://crates.io/crates/oxide-mc"><img src="https://img.shields.io/crates/v/oxide-mc.svg?style=flat-square&color=E99F56" alt="Crates.io"></a>
  <a href="https://github.com/S3fflexDev/oxide-mc/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square&color=242424" alt="License"></a>
  <img src="https://img.shields.io/badge/rustc-1.75+-black?style=flat-square&logo=rust&color=CC7F33" alt="Rust Version">
</p>


---

OxideMC is a lightweight Minecraft manager designed for efficiency and performance. Built from the ground up in **Rust**, it is specifically crafted to be compatible with the **Tauri framework**.

Currently, it serves as a full installer of **any versions of Minecraft and Fabric** (+1.8.7), with support for **NeoForge** coming soon.

### 💫 Focus
Most launchers are heavy. **Oxide** is a core library that handles the "dirty work" (assets, libraries, JVM, Fabric) while staying out of your way.

- **Memory Efficient**: Built for systems with 4GB RAM or less.
- **Tauri-First**: Optimized for async execution in modern desktop backends.
- **Native**: 0ms overhead, pure Rust performance.

## ⚡ Quick Start

```rust
use oxide_mc::OxideLauncher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize the core
    let mut launcher = OxideLauncher::new("Steve");

    // 2. Install game & Java
    launcher.full_install(None, "1.20.1", ModLoader::Fabric, false).await?;
    launcher.java_download(17).await?;

    // 3. Launch!
    launcher.start("8G").await?;

    Ok(())
}
```

## 🚀 Why OxideMC?

Most Minecraft launchers are resource-heavy. OxideMC is different:

- **Low RAM Footprint:** Designed exclusively for low-end devices (like 4GB RAM systems).
- **Native Speed:** Written in Rust to ensure the launcher doesn't steal resources from the game.
- **Maximum Performance:** Leaves more CPU and RAM available for Minecraft and the JVM to perform at their best.
- **Tauri Ready:** Optimized to work as a backend for modern, lightweight desktop GUIs.

✨ Features

- ✅ Full Installation: Downloads libraries, assets, and the game client.
- ✅ Fabric Injection: Seamlessly integrates Fabric Loader (+1.8.7).
- ✅ Modpack Support: Inject custom mods, configs, and shaders via URL.
- ✅ Smart Runtime: Automatic Java version detection and portable installation.
- 🚧 Coming Soon: NeoForge support and legacy vanilla versions.

## 🛠️ Installation (Dependency)

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxide-mc = { git = "https://github.com/S3fflexDev/oxide-mc.git" }
```

or use command: `cargo add oxide-mc`

<p align="center"> Made with ❤️ by <strong>S3fflexDev</strong> (16y/o dev) </p> 