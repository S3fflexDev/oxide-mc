use crate::functions;
use anyhow::Result;

pub mod models;
use models::InstallationProfile;

const PROFILE_FILE_NAME: &str = "install_profile.json";

pub fn load_profile() -> Result<Option<InstallationProfile>> {
    let base_path = functions::base_path();
    let file_path = base_path.join(PROFILE_FILE_NAME);

    if !file_path.exists() {
        return Ok(None);
    }

    println!("Loading installation profile from: {:?}", file_path);
    let content = std::fs::read_to_string(file_path)?;

    let profile: InstallationProfile = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse profile file: {}", e))?;

    Ok(Some(profile))
}

pub fn save_profile(profile: &InstallationProfile) -> Result<()> {
    let base_path = functions::base_path();
    let file_path = base_path.join(PROFILE_FILE_NAME);

    let content = serde_json::to_string_pretty(profile)?;

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&file_path, content)?;
    println!("Installation profile saved to: {:?}", file_path);
    Ok(())
}

pub fn delete_profile() -> Result<()> {
    let base_path = functions::base_path();
    let file_path = base_path.join(PROFILE_FILE_NAME);

    if file_path.exists() {
        std::fs::remove_file(file_path)?;
        println!("Deleted existing installation profile.");
    }

    Ok(())
}
