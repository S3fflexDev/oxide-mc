use crate::net::get_http_client;
use tokio::fs;

pub async fn inject_modpack(url: &str, base_path: &std::path::Path) -> anyhow::Result<()> {
    let client = get_http_client();

    println!("Downloading modpack from: {}", url);

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let mods_dir = base_path.join("mods");
    let config_dir = base_path.join("config");

    if mods_dir.exists() {
        fs::remove_dir_all(&mods_dir).await?;
    }
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir).await?;
    }

    let target_dir = base_path;

    println!("Extracting files in {:?}...", target_dir);

    crate::functions::extract_zip(&bytes, target_dir, false)?;

    println!("Modpack injected successfully.");
    Ok(())
}
