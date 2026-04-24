use std::path::PathBuf;
use anyhow::Result;
use crate::functions::{check_java_version, download_java_runtime};

pub mod models;
pub mod functions;
pub mod launcher;
pub mod fabic_manifest_model;

pub struct LauncherConfig {
    pub game_path: PathBuf,
    pub java_path: PathBuf,
    pub username: String,
}

pub struct OxideLauncher {
    pub settings: LauncherConfig,
}

impl OxideLauncher {
    pub fn new(username: &str) -> Self {
        let base = functions::base_path();

        let java_path = if functions::check_java_version(17) {
            std::path::PathBuf::from("java")
        } else {
            base.join("runtime/jdk-17.0.10+7/bin/java.exe")
        };

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
        Self {
            settings: LauncherConfig {
                java_path: path.join("runtime/jdk-17.0.10+7/bin/java.exe"),
                game_path: path,
                username: username.to_string(),
            },
        }
    }

    pub async fn full_install(&self, modpack_url: Option<&str>) -> Result<()> {
        println!("Beggining installation on: {:?}", self.settings.game_path);

        // Get manifests
        let manifest = functions::get_manifest().await?;
        let fabric_manifest = functions::get_fabric_manifest().await?;

        // Downloads
        functions::download_libraries(&manifest, &self.settings.game_path).await?;
        functions::download_fabric_libraries(&fabric_manifest, &self.settings.game_path).await?;
        functions::download_client(&manifest, &self.settings.game_path).await?;
        functions::download_assets(&manifest, &self.settings.game_path).await?;

        // Inyect modpack
        if let Some(url) = modpack_url {
            functions::inject_modpack(url, &self.settings.game_path).await?;
        }

        println!("All done successfully!.");
        Ok(())
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
            main_class
        )
    }

    pub async fn java_download(&mut self) -> Result<()> {
        
        println!("Java download started...");
        
        functions::download_java_runtime(&self.settings.game_path).await?;

        let nuevo_path = self.settings.game_path
            .join("runtime")
            .join("jdk-17.0.10+7")
            .join("bin")
            .join(if cfg!(target_os = "windows") { "java.exe" } else { "java" });

        self.settings.java_path = nuevo_path;

        println!("Java path updated to: {:?}", self.settings.java_path);
        Ok(())
    }

    pub async fn check_java(&self, version: u32) -> anyhow::Result<bool> {

        Ok(functions::check_java_version(version))

    }
}