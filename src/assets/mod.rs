use tokio::fs;
use crate::mc::models::{AssetIndexContent, VersionManifest};
use crate::net::get_http_client;

pub async fn download_assets(
    manifest: &VersionManifest,
    base_path: &std::path::Path,
) -> anyhow::Result<()> {
    let client = get_http_client();
    let assets_dir = base_path.join("assets");
    let objects_dir = assets_dir.join("objects");
    let indexes_dir = assets_dir.join("indexes");

    fs::create_dir_all(&indexes_dir).await?;
    fs::create_dir_all(&objects_dir).await?;

    let index_url = &manifest.asset_index.url;
    let index_path = indexes_dir.join(format!("{}.json", manifest.asset_index.id));

    let index_content = if index_path.exists() {
        fs::read_to_string(&index_path).await?
    } else {
        println!(
            "Downloading assets index ({}.json)...",
            manifest.asset_index.id
        );
        let content = client.get(index_url).send().await?.text().await?;
        fs::write(&index_path, &content).await?;
        content
    };

    let index_data: AssetIndexContent = serde_json::from_str(&index_content)
        .map_err(|e| anyhow::anyhow!("Error parsing assets index: {}", e))?;

    println!("Verifying {} assets...", index_data.objects.len());

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(64));
    let mut set = tokio::task::JoinSet::new();

    for (_name, obj) in index_data.objects {
        let client = client.clone();
        let objects_dir = objects_dir.clone();
        let semaphore = semaphore.clone();
        set.spawn(async move {
            let _permit = semaphore.acquire().await;
            let prefix = &obj.hash[..2];
            let folder = objects_dir.join(prefix);
            let file_path = folder.join(&obj.hash);

            if file_path.exists() {
                return;
            }

            let _ = fs::create_dir_all(&folder).await;
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                prefix, obj.hash
            );
            let Ok(res) = client.get(&url).send().await else {
                return;
            };
            let Ok(bytes) = res.bytes().await else {
                return;
            };
            let _ = fs::write(&file_path, &bytes).await;
        });
    }

    while set.join_next().await.is_some() {}

    println!("All assets are ready to load up.");
    Ok(())
}
