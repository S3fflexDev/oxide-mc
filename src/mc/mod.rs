pub(crate) mod models;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;
use crate::mc::models::{Action, Library, Name, VersionManifest};
use crate::net::get_http_client;
use crate::{state, version_index};
use crate::functions::extract_zip;

pub async fn get_manifest(version: &str) -> anyhow::Result<VersionManifest> {
    println!("Getting manifest for version: {}", version);

    let url = version_index::find_version_manifest_url(version).await?;

    println!("{}", url.as_str());

    let client = get_http_client();

    let response = client.get(url).send().await?;

    let text = response.text().await?;

    if text.is_empty() {
        return Err(anyhow::anyhow!("Server did not return text"));
    }

    println!(
        "Firsts 50 characters received: {}",
        &text[..std::cmp::min(50, text.len())]
    );

    let manifest: VersionManifest = serde_json::from_str(&text).map_err(|e| {
        anyhow::anyhow!(
            "Error parsing JSON: {} | Content: {}",
            e,
            &text[..std::cmp::min(100, text.len())]
        )
    })?;

    Ok(manifest)
}

pub async fn download_libraries(
    manifest: &VersionManifest,
    base_path: &Path,
) -> anyhow::Result<()> {
    let client = get_http_client();
    let libraries_dir = base_path.join("libraries");
    let mut set = JoinSet::new();

    for lib in &manifest.libraries {
        let Some(artifact) = &lib.downloads.artifact else { continue; };
        let relative_path = artifact.path.as_ref().ok_or_else(|| anyhow::anyhow!("Missing path"))?.clone();
        let target_path = libraries_dir.join(&relative_path);
        let url = artifact.url.clone();
        let client = client.clone();
        
        set.spawn(async move {
            if target_path.exists() { return Ok(()); }

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let bytes = client.get(url).send().await?.bytes().await?;
            fs::write(&target_path, &bytes).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(res) = set.join_next().await {
        res??;
    }

    println!("All libraries downloaded in parallel!");
    Ok(())
}

pub async fn download_client(
    manifest: &VersionManifest,
    base_path: &Path,
) -> anyhow::Result<()> {
    let client = get_http_client();

    let version_dir = base_path.join("versions").join(&manifest.id);
    let target_path = version_dir.join(format!("{}.jar", manifest.id));

    fs::create_dir_all(&version_dir).await?;

    if target_path.exists() {
        println!("Client.jar already exists, skipping download.");
        return Ok(());
    }

    println!("Downloading game code (client.jar)...");
    let url = &manifest.downloads.client.url;
    let bytes = client.get(url).send().await?.bytes().await?;

    let mut file = fs::File::create(&target_path).await?;
    file.write_all(&bytes).await?;

    println!(
        "client.jar downloaded successfully ({} MB)",
        bytes.len() / 1_024 / 1_024
    );

    Ok(())
}


pub(crate) fn collect_vanilla_cp(
    libraries: &[Library],
    libraries_dir: &Path,
    cp_parts: &mut Vec<String>,
) {
    for lib in libraries {
        if !should_use_library(lib) {
            continue;
        }
        
        if let Some(artifact) = &lib.downloads.artifact {
            if let Some(rel_path) = &artifact.path {
                let full_path = libraries_dir.join(rel_path);
                if let Some(path_str) = full_path.to_str() {
                    let p = path_str.to_string();
                    if !cp_parts.contains(&p) {
                        cp_parts.push(p);
                    }
                }
            }
        }
        
        if let Some(native_artifact) = lib.downloads.classifiers.get("natives-windows") {
            if let Some(rel_path) = &native_artifact.path {
                let full_path = libraries_dir.join(rel_path);
                if let Some(path_str) = full_path.to_str() {
                    let p = path_str.to_string();
                    if !cp_parts.contains(&p) {
                        cp_parts.push(p);
                    }
                }
            }
        }
    }
}
pub(crate) fn should_use_library(lib: &Library) -> bool {
    if let Some(rules) = &lib.rules {
        let mut allow = false;
        for rule in rules {
            let os_matches = if let Some(os) = &rule.os {
                os.name == Name::Windows
            } else {
                true
            };

            if os_matches {
                allow = rule.action == Action::Allow;
            }
        }
        return allow;
    }
    true
}

/*
fn get_library_base_name(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 2 {
        format!("{}:{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}
*/

pub fn gen_classpath(manifest: &VersionManifest, base_path: &Path) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    collect_vanilla_cp(&manifest.libraries, &libraries_dir, &mut cp_parts);

    let client_jar = base_path
        .join("versions")
        .join(&manifest.id)
        .join(format!("{}.jar", manifest.id));
    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(crate::platform::CLASSPATH_SEPARATOR)
}

pub fn get_native_classifier(lib: &Library) -> Option<String> {
    let os_key = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "osx"
    };

    lib.natives.as_ref()?.get(os_key).cloned()
}

pub async fn download_and_extract_natives(manifest: &VersionManifest, base_path: &Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let natives_dir = base_path.join("versions").join(&manifest.id).join("natives");
    fs::create_dir_all(&natives_dir).await?;

    for lib in &manifest.libraries {
        if let Some(classifier) = get_native_classifier(lib) {
            if let Some(native_artifact) = lib.downloads.classifiers.get(&classifier) {
                println!("Downloading and extracting native: {}", lib.name);

                let response = client.get(&native_artifact.url).send().await?;
                let bytes = response.bytes().await?;

                extract_zip(&bytes, &natives_dir, false)?;
            }
        }
    }


    Ok(())
}

pub async fn check_game_installed(version: &str, mod_loader: &str) -> anyhow::Result<bool> {

    let profile = match state::load_profile()? {
        Some(p) => {
            println!("Found installation profile: {}", p.minecraft_version);
            p
        },
        None => {
            return Ok(false);
        }
    };

    if profile.minecraft_version != version {
        return Ok(false)
    }

    if profile.modloader_type != mod_loader {
        return Ok(false)
    }

    Ok(true)
}