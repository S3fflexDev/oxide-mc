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

## Quick Start

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

## Features

---

- Installation: Downloads libraries, assets, and the game client.
- Fabric Injection: Seamlessly integrates Fabric Loader (+1.8.7).
- Modpack Support: Inject custom mods, configs, and shaders via URL.
- Smart Runtime: Automatic Java version detection and portable installation.
- Coming Soon: NeoForge support and legacy vanilla versions.

## Installation

---

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxide-mc = "0.2.1"
```

or use command: `cargo add oxide-mc`

<p align="center"> Made with ❤️ by <strong>S3fflexDev</strong> (16y/o dev) </p> 