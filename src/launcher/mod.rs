use crate::models::VersionManifest;
use std::process::{Child, Command};

pub fn lanzar_juego(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
    java_bin_path: &std::path::Path,
    username: &str,
    classpath: String,
    main_class: &String,
) -> anyhow::Result<Child> {
    let mut cmd = Command::new(java_bin_path);

    // Memory arguments
    cmd.arg("-Xmx2G");
    cmd.arg("-Xms512M");

    // Classpath
    cmd.arg("-cp").arg(classpath);

    // Main class
    cmd.arg(main_class);

    // Minecraft arguments
    cmd.arg("--username").arg(username);
    cmd.arg("--version").arg(&manifest.id);
    cmd.arg("--gameDir").arg(base_path);
    cmd.arg("--assetsDir").arg(base_path.join("assets"));
    cmd.arg("--assetIndex").arg(&manifest.asset_index.id);
    cmd.arg("--uuid")
        .arg("00000000-0000-0000-0000-000000000000");
    cmd.arg("--accessToken").arg("0");
    cmd.arg("--userType").arg("mojang");
    cmd.arg("--versionType").arg("release");

    // Lanzar!
    println!("Launching Minecraft 1.20.1...");
    let process = cmd.spawn()?;
    Ok(process)
}
