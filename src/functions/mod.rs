use std::fs;
use std::path::Path;

pub(crate) fn extract_zip(data: &[u8], target_dir: &Path, strip_toplevel: bool) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(enclosed_name) = file.enclosed_name() else {
            continue;
        };
        let path = if strip_toplevel {
            let mut components = enclosed_name.components();
            components.next();
            components.as_path().to_path_buf()
        } else {
            enclosed_name.to_path_buf()
        };
        if path.as_os_str().is_empty() {
            continue;
        }
        let out_path = target_dir.join(&path);
        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out_file = fs::File::create(&out_path)?;
        std::io::copy(&mut file, &mut out_file)?;
    }
    Ok(())
}

// --------------------------------- MULTIPLATFORM

use directories::ProjectDirs;

pub fn base_path() -> std::path::PathBuf {
    // Win: C:\Users\Nombre\AppData\Roaming\OxideMC\data
    // Lin: /home/nombre/.local/share/oxidemc
    // Mac: /Users/Nombre/Library/Application Support/OxideMC
    if let Some(proj_dirs) = ProjectDirs::from("com", "s3fflex", "oxidemc") {
        return proj_dirs.data_dir().to_path_buf();
    }
    std::path::PathBuf::from(".minecraft")
}

pub fn clean_data_directory(base_path: &Path) -> std::io::Result<()> {
    if !base_path.exists() || !base_path.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            
            if file_name == "install_profile.json" || file_name == "runtime" {
                println!("Saltando: {}", file_name);
                continue;
            }
            
            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    eprintln!("No se pudo borrar la carpeta {:?}: {}", path, e);
                } else {
                    println!("Carpeta borrada: {}", file_name);
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("No se pudo borrar el archivo {:?}: {}", path, e);
                } else {
                    println!("Archivo borrado: {}", file_name);
                }
            }
        }
    }

    Ok(())
}