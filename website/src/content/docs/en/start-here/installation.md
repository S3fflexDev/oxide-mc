---
title: Installation
---

Oxide MC is an async library designed for creating launchers of Minecraft without headaches, we manage libraries, assets and JVM for you. Designed for use in [Tauri](https://v2.tauri.app/es/).

Add it with this command

```bash
cargo add oxide_mc
```
This tool is focused for series launchers, individual modpack launchers, etc..., not intended for large-scale launchers like Modrinth or GDLauncher. Meanwhile, you can do it, but it's not the objective of this project.

# Use guide

## Step 1: Beginning

Oxide MC is not an object, it's what saves the session.

Because you need to put the name of the session we recommend save it in a `useState` of React and pass it to Tauri.

```rust
let launcher = OxideLauncher::new("Steve");
```

At this moment the library didn't write anything in disk, it only has saved the necessary configs.

## Step 2: Installation process

- Check: **Is installed?** (check_game)

```rust
async fn check_game_installed() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    let result = launcher.check_game("1.20.1", "vanilla").await?;
    println!("{:#?}", result);
    Ok(())
}
```

- Download: If not, **download everything** (full_install).

This function downloads all Minecraft assets and libraries, (And Fabric if is it selected)

```rust
async fn test_install() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.full_install(None, "1.20.1", oxide_mc::state::models::ModLoader::Fabric, false).await?;
    Ok(())
}
```

- Runtime: **Install java** (java_download).

Oxide MC manages Java, but is portable, for making it more easy and more automatic.

If you don't download the right version of Java for each version, the start could fail.

```rust
async fn java_donwload() -> anyhow::Result<()> {
    let mut launcher = OxideLauncher::new("TestUser");

    launcher.java_download(17).await?;
    Ok(())
}

```

- Recommendations

Try to do all in one function (`install_minecraft`) so everything unifies, and you are not passing all the time the variable `name` and overwriting the values of the config.

## Step 3: Start game

This is for making it more easy, the library auto-detects the version you've installed with the command `test_install()`, so you don't need to put any parameters, this is for making it unified, aesthetic and simple.

```rust
async fn run() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.start().await?;
    Ok(())
}
```

## Check commands

```rust
async fn check_java_version_test() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    launcher.check_java().await?;
    Ok(())
}

async fn check_game_installed() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    let result = launcher.check_game("1.20.1", "vanilla").await?;
    println!("{:#?}", result);
    Ok(())
}
```

This commands stands here for returning `true` or `false` depending on if something is installed or not.

Do not forget to import it! ;)

```rust
use oxide_mc::OxideLauncher;
```

---

*Made with ❤️ by a 16 years old dev.*
