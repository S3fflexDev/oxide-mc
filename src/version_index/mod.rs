use crate::net::get_http_client;
use crate::version_index::models::ManifestIndex;

mod models;

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

pub async fn find_version_manifest_url(version_id: &str) -> anyhow::Result<String> {
    println!("Fetching global version manifest...");
    let client = get_http_client();

    let response_text = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .text()
        .await?;

    let version_index: ManifestIndex = serde_json::from_str(&response_text)?;
    println!("Found {} versions in total.", version_index.versions.len());

    let found_version = version_index.versions.iter().find(|v| v.id == version_id);

    match found_version {
        Some(version_info) => {
            println!(
                "Version '{}' found. Manifest URL: {}",
                version_id, version_info.url
            );
            Ok(version_info.url.clone())
        }
        None => Err(anyhow::anyhow!(
            "Version '{}' not found in the official manifest.",
            version_id
        )),
    }
}
