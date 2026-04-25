use crate::functions::{check_java_version, download_java_runtime, JAVA_EXECUTABLE};
use anyhow::Result;
use std::path::PathBuf;

pub mod fabric_manifest_model;
pub mod functions;
pub mod launcher;
mod manifest_indexes;
pub mod models;

pub struct LauncherConfig {
    pub game_path: PathBuf,
    pub java_path: PathBuf,
    pub username: String,
}

pub struct OxideLauncher {
    pub settings: LauncherConfig,
}

pub struct InstallationSummary {
    pub game_version: String,
    pub fabric_loader: String,
}

pub struct JavaInfo {
    pub major: u32,
    pub full_name: String,
}

impl OxideLauncher {
    pub fn new(username: &str) -> Self {
        let base = functions::base_path();
        println!("base path: {}", base.display());

        let _java_installed = functions::check_java_version().unwrap();

        let java_path = base.join("runtime").join("bin").join(JAVA_EXECUTABLE);

        Self {
            settings: LauncherConfig {
                game_path: base.clone(),
                java_path,
                username: username.to_string(),
            },
        }
    }

    pub fn new_at_path(username: &str, custom_path: PathBuf) -> Self {
        Self::create_with_path(username, custom_path)
    }

    fn create_with_path(username: &str, path: PathBuf) -> Self {
        let java_path = path.join("runtime").join("bin").join(JAVA_EXECUTABLE);
        Self {
            settings: LauncherConfig {
                java_path,
                game_path: path,
                username: username.to_string(),
            },
        }
    }

    pub async fn full_install(&self, modpack_url: Option<&str>) -> Result<i64> {
        println!("Beggining installation on: {:?}", self.settings.game_path);

        // Get manifests
        let manifest = functions::get_manifest().await?;
        let fabric_manifest = functions::get_fabric_manifest().await?;

        let _game_version = &manifest.id.clone();
        let _loader_version = &fabric_manifest.inherits_from.clone();

        let java_version: &i64 = &manifest.java_version.major_version.clone();

        // Downloads
        functions::download_libraries(&manifest, &self.settings.game_path).await?;
        functions::download_fabric_libraries(&fabric_manifest, &self.settings.game_path).await?;
        functions::download_client(&manifest, &self.settings.game_path).await?;
        functions::download_assets(&manifest, &self.settings.game_path).await?;

        // Inyect modpack
        if let Some(url) = modpack_url {
            functions::inject_modpack(url, &self.settings.game_path).await?;
        }

        println!(
            "Install java {} before running with command java_download!.",
            java_version
        );

        println!("All done successfully!.");
        Ok(*java_version)
    }

    pub async fn start(&self) -> Result<std::process::Child> {
        if !self.settings.java_path.exists() {
            return Err(anyhow::anyhow!(
                "Java not found on path: {:?}",
                self.settings.java_path
            ));
        }

        let manifest = functions::get_manifest().await?;
        let fabric_manifest = functions::get_fabric_manifest().await?;

        let cp = functions::gen_cp_fabric(&manifest, &fabric_manifest, &self.settings.game_path);
        let main_class = &fabric_manifest.main_class;

        launcher::lanzar_juego(
            &manifest,
            &self.settings.game_path,
            &self.settings.java_path,
            &self.settings.username,
            cp,
            main_class,
        )
    }

    pub async fn java_download(&mut self, version: i64) -> Result<()> {
        println!("Java download started...");

        let _full_name = download_java_runtime(&self.settings.game_path, version).await?;

        self.settings.java_path = self
            .settings
            .game_path
            .join("runtime")
            .join("bin")
            .join(JAVA_EXECUTABLE);

        println!("Java path updated to: {:?}", self.settings.java_path);
        Ok(())
    }

    pub async fn check_java(&self) -> anyhow::Result<i32> {
        check_java_version()
    }
}
