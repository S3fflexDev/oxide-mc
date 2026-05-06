use tokio::fs;
use tracing::info;
use crate::functions::download_file;
use crate::mc::models::{AssetIndexContent, VersionManifest};
use crate::net::get_http_client;
use anyhow::Result;

pub async fn download_assets(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
) -> Result<()> {
    let client = get_http_client();
    let assets_dir = base_path.join("assets");
    let objects_dir = assets_dir.join("objects");
    let indexes_dir = assets_dir.join("indexes");

    fs::create_dir_all(&indexes_dir).await?;
    fs::create_dir_all(&objects_dir).await?;

    let index_url = &manifest.asset_index.url;
    let index_path = indexes_dir.join(format!("{}.json", manifest.asset_index.id));

    if !index_path.exists() {
        println!("Downloading assets index: {}.json", manifest.asset_index.id);
        download_file(&client, index_url, &index_path).await?;
    }

    let index_content = tokio::fs::read_to_string(&index_path).await?;
    let index_data: AssetIndexContent = serde_json::from_str(&index_content)?;

    println!("Verifying {} assets...", index_data.objects.len());

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(20));
    let mut set = tokio::task::JoinSet::new();

    for (_name, obj) in index_data.objects {
        let client = client.clone();
        let objects_dir = objects_dir.clone();
        let semaphore = semaphore.clone();
        let hash = obj.hash.clone();

        set.spawn(async move {
            let _permit = semaphore.acquire_owned().await?;

            let prefix = &hash[..2];
            let folder = objects_dir.join(prefix);
            let file_path = folder.join(&hash);

            if file_path.exists() {
                return Ok(());
            }

            let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);

            info!("Downloading asset: {}", url);

            download_file(&client, &url, &file_path).await?;

            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(res) = set.join_next().await {
        res??;
    }

    println!("All assets are ready.");
    Ok(())
}
