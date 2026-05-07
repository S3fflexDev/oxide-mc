use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstallationProfile {
    pub minecraft_version: String,
    pub modloader_type: String,
    pub modloader_version: Option<String>,
    pub main_class: String,
    pub classpath: String,
    pub native_libraries: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum ModLoader {
    #[default]
    Vanilla,
    Fabric,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &str {
        match self {
            ModLoader::Vanilla => "vanilla",
            ModLoader::Fabric { .. } => "fabric",
            ModLoader::NeoForge { .. } => "neoforge",
        }
    }
}
