use crate::java::{check_java_version, download_java_runtime};
use anyhow::Result;
use std::path::PathBuf;
use sha1::digest::typenum::Mod;
use crate::functions::base_path;
use crate::platform::JAVA_EXECUTABLE;
use crate::state::load_profile;
use crate::state::models::{InstallationProfile, ModLoader};

pub mod fabric;
pub mod functions;
pub mod launcher;
mod version_index;
pub mod net;
pub mod mc;
mod assets;
mod modpack;
mod java;
mod platform;
pub mod state;

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

        let _java_installed = java::check_java_version().unwrap();

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

    pub async fn full_install(
        &self,
        modpack_url: Option<&str>,
        version: &str,
        modloader: ModLoader
    ) -> Result<i64> {
        println!("Beggining installation on: {:?}", self.settings.game_path);

        // Get manifests
        let manifest = mc::get_manifest(version).await?;
        let fabric_manifest = fabric::get_fabric_manifest(version).await?;

        let base_path = base_path();

        let _game_version = &manifest.id.clone();
        let _loader_version = &fabric_manifest.inherits_from.clone();

        let java_version: &i64 = &manifest.java_version.major_version.clone();

        // Vanilla
        mc::download_libraries(&manifest, &self.settings.game_path).await?;
        mc::download_client(&manifest, &self.settings.game_path).await?;

        // Assets
        assets::download_assets(&manifest, &self.settings.game_path).await?;

        let mut main_class: String = manifest.main_class.clone();

        let mut final_classpath: String = mc::gen_classpath(&manifest, &self.settings.game_path);
        
        match modloader {
            ModLoader::Vanilla => {
                println!("Vanilla installation complete.");
            }
            ModLoader::Fabric => {
                println!("Installing Fabric...");
                fabric::download_fabric_libraries(&fabric_manifest, &self.settings.game_path).await?;
                main_class = fabric_manifest.main_class.clone();
                final_classpath = fabric::gen_cp_fabric(&manifest, &fabric_manifest, &base_path);
                println!("Fabric installation complete.");
            }
            ModLoader::NeoForge => {
                println!("Installing NeoForge...");
                println!("Neoforge is a placeholder :).");
            }
        }

        let profile = InstallationProfile {
            minecraft_version: manifest.id.clone(),
            modloader_type: "vanilla".to_string(),
            modloader_version: None,
            main_class,
            classpath: final_classpath
        };

        state::save_profile(&profile)?;

        // Inyect modpack
        if let Some(url) = modpack_url {
            modpack::inject_modpack(url, &self.settings.game_path).await?;
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

        let profile = match state::load_profile()? {
            Some(p) => {
                println!("Found installation profile: {}", p.minecraft_version);
                p
            },
            None => {
                // Si no hay perfil, no hay nada que hacer. Error claro.
                return Err(anyhow::anyhow!("No installation found. Please run the 'install' command first."));
            }
        };


        let manifest = mc::get_manifest(&*profile.minecraft_version).await?;
        let fabric_manifest = fabric::get_fabric_manifest(&profile.minecraft_version).await?;

        // let cp = mc::gen_classpath(&manifest, &self.settings.game_path);
        // let main_class = &manifest.main_class;

        launcher::launch_game(
            &manifest,
            &self.settings.game_path,
            &self.settings.java_path,
            &self.settings.username,
            profile.classpath,
            &profile.main_class,
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
