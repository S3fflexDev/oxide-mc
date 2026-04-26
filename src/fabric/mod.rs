use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use crate::fabric::models::{FabricLibrary, FabricProfile};
use crate::platform::CLASSPATH_SEPARATOR;
use crate::mc;
use crate::mc::models::VersionManifest;
use crate::net::get_http_client;

pub(crate) mod models;

pub async fn get_fabric_manifest() -> anyhow::Result<FabricProfile> {

    // https://meta.fabricmc.net/v2/versions/loader/1.20.1/0.19.2/profile/json
    let url = "https://meta.fabricmc.net/v2/versions/loader/1.21.1/0.19.2/profile/json";

    let client = get_http_client();

    let response = client.get(url).send().await?;

    let manifest: FabricProfile = response.json().await?;

    Ok(manifest)
}

pub fn gen_fabric_path(lib: &FabricLibrary) -> std::path::PathBuf {
    let parts: Vec<&str> = lib.name.split(':').collect();
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let jar_name = format!("{}-{}.jar", artifact, version);

    // Build path natively using PathBuf components
    let mut path = std::path::PathBuf::new();
    for part in group.split('/') {
        path.push(part);
    }
    path.push(artifact);
    path.push(version);
    path.push(jar_name);
    path
}

pub async fn download_fabric_libraries(
    manifest_fabric: &FabricProfile,
    base_path: &Path,
) -> anyhow::Result<()> {
    let client = get_http_client();
    let libraries_dir = base_path.join("libraries");

    println!("Starting download of Fabric libraries...");

    for lib in &manifest_fabric.libraries {
        let relative_path_buf = gen_fabric_path(lib);

        let target_path = libraries_dir.join(&relative_path_buf);

        let url_path = relative_path_buf.to_string_lossy().replace('\\', "/");
        let download_url = format!("{}{}", lib.url, url_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if target_path.exists() {
            continue;
        }

        println!("Downloading Fabric Lib: {}", lib.name);
        let response = client.get(&download_url).send().await?;
        if !response.status().is_success() {
            println!("Error 404 in Fabric Lib: {}", download_url);
            continue;
        }
        let bytes = response.bytes().await?;
        let mut file = fs::File::create(&target_path).await?;
        file.write_all(&bytes).await?;
    }

    println!("Fabric libraries downloaded successfully!");
    Ok(())
}

pub fn gen_cp_fabric(
    manifest_mc: &VersionManifest,
    manifest_fabric: &FabricProfile,
    base_path: &std::path::Path,
) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    mc::collect_vanilla_cp(&manifest_mc.libraries, &libraries_dir, &mut cp_parts);

    for lib in &manifest_fabric.libraries {
        let full_path = libraries_dir.join(gen_fabric_path(lib));
        if let Some(path_str) = full_path.to_str() {
            cp_parts.push(path_str.to_string());
        }
    }

    let client_jar = base_path
        .join("versions")
        .join(&manifest_mc.id)
        .join(format!("{}.jar", manifest_mc.id));
    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(CLASSPATH_SEPARATOR)
}
