use crate::mc::models::VersionManifest;
use std::process::{Child, Command};
use anyhow::Result;

pub fn launch_game(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
    java_bin_path: &std::path::Path,
    username: &str,
    classpath: String,
    main_class: String,
    natives: bool,
    max_ram: &str
) -> Result<Child> {
    let mut cmd = Command::new(java_bin_path);

    let natives_path = base_path.join("versions").join(&manifest.id).join("natives");

    // Memory arguments
    cmd.arg(format!("-Xmx{}", max_ram));
    cmd.arg("-Xms512M");

    if natives {
        cmd.arg(format!("-Djava.library.path={}", natives_path.to_string_lossy()));
    }

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
    cmd.arg("-Dorg.lwjgl.util.Debug=true");
    cmd.arg("-Dorg.lwjgl.util.DebugLoader=true");

    // Launch!!
    println!("Launching Minecraft {}...", &manifest.id);
    let process = cmd.spawn()?;
    Ok(process)
}
