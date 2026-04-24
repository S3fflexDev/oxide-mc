use std::io::Cursor;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zip_extract::extract;
use crate::fabic_manifest_model::{FabricLibrary, FabricProfile};
use crate::models::{AssetIndex, AssetIndexContent, VersionManifest}; // Importamos tus Structs

pub async fn get_manifest() -> anyhow::Result<VersionManifest> {
    let url = "https://piston-meta.mojang.com/v1/packages/c9811ffdbcd77d79c12412836f21ed4e3c592102/1.20.1.json";

    // 1. Creamos un cliente con un User-Agent de un navegador real
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // 2. Hacemos la petición
    let response = client.get(url).send().await?;

    // 3. Obtenemos el texto y verificamos qué hay dentro
    let texto = response.text().await?;

    if texto.is_empty() {
        return Err(anyhow::anyhow!("El servidor devolvió una respuesta vacía"));
    }

    // IMPRIMIR PARA DEBUG (esto te ayudará a ver qué llega realmente)
    println!("Primeros 50 caracteres recibidos: {}", &texto[..std::cmp::min(50, texto.len())]);

    // 4. Intentamos parsear
    let manifest: VersionManifest = serde_json::from_str(&texto).map_err(|e| {
        // Si falla, mostramos el error y el inicio del texto que causó el fallo
        anyhow::anyhow!("Error al parsear JSON: {} | Contenido: {}", e, &texto[..std::cmp::min(100, texto.len())])
    })?;

    Ok(manifest)
}


pub async fn listar_librerias() -> anyhow::Result<()> {
    let manifest = get_manifest().await?;

    for lib in manifest.libraries {
        // Ahora usamos "if let Some" porque artifact es opcional
        if let Some(artifact) = lib.downloads.artifact {
            println!("Librería: {}", lib.name);
            println!("  -> URL: {}", artifact.url);
            if let Some(path) = artifact.path {
                println!("  -> Path: {}", path);
            }
        } else {
            // Esto imprimirá las librerías que no tienen descarga directa (como las de sistema)
            println!("Librería sin artefacto directo: {}", lib.name);
        }
    }
    Ok(())
}

pub async fn download_libraries(manifest: &VersionManifest, base_path: &Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let libraries_dir = base_path.join("libraries");

    println!("Iniciando descarga de librerías en: {:?}", libraries_dir);

    for lib in &manifest.libraries {
        // Solo descargamos si tiene un "artifact" (algunas librerías no lo tienen)
        if let Some(artifact) = &lib.downloads.artifact {
            // 1. Obtener la ruta relativa (ej: "ca/weblite/java-objc-bridge/1.1/...")
            let relative_path = artifact.path.as_ref().expect("Falta el path en la librería");
            let target_path = libraries_dir.join(relative_path);

            // 2. Crear las carpetas necesarias
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            // 3. Descargar si el archivo no existe ya
            if !target_path.exists() {
                println!("Descargando: {}", lib.name);
                let bytes = client.get(&artifact.url).send().await?.bytes().await?;
                let mut file = fs::File::create(&target_path).await?;
                file.write_all(&bytes).await?;
            }
        }
    }

    println!("¡Todas las librerías están listas!");
    Ok(())
}

pub async fn download_client(manifest: &VersionManifest, base_path: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    // 1. Definir la ruta: test_minecraft/versions/1.20.1/1.20.1.jar
    let version_dir = base_path.join("versions").join(&manifest.id);
    let target_path = version_dir.join(format!("{}.jar", manifest.id));

    // 2. Crear las carpetas si no existen
    fs::create_dir_all(&version_dir).await?;

    // 3. Descargar el archivo
    if !target_path.exists() {
        println!("Descargando el motor del juego (client.jar)...");

        // La URL está en downloads.client.url de tu estructura
        let url = &manifest.downloads.client.url;
        let bytes = client.get(url).send().await?.bytes().await?;

        let mut file = fs::File::create(&target_path).await?;
        file.write_all(&bytes).await?;

        println!("✅ client.jar descargado con éxito ({} MB)", bytes.len() / 1_024 / 1_024);
    } else {
        println!("El client.jar ya existe, saltando descarga.");
    }

    Ok(())
}

pub fn gen_classpath(manifest: &VersionManifest, base_path: &std::path::Path) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    let separador_cp = if cfg!(target_os = "windows") { ";" } else { ":" };

    // 1. Añadimos todas las librerías que tienen un path
    for lib in &manifest.libraries {
        if let Some(artifact) = &lib.downloads.artifact {
            if let Some(rel_path) = &artifact.path {
                let absolute_path = libraries_dir.join(rel_path);
                if let Some(path_str) = absolute_path.to_str() {
                    cp_parts.push(path_str.to_string());
                }
            }
        }
    }

    // 2. Añadimos el client.jar al final
    let client_jar = base_path.join("versions").join(&manifest.id).join(format!("{}.jar", manifest.id));
    if let Some(path_str) = client_jar.to_str() {
        cp_parts.push(path_str.to_string());
    }

    cp_parts.join(separador_cp)
}

// ------------------------------------- ASSETS

pub async fn download_assets(manifest: &VersionManifest, base_path: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let assets_dir = base_path.join("assets");
    let objects_dir = assets_dir.join("objects");
    let indexes_dir = assets_dir.join("indexes");

    // 1. Crear carpetas de los assets
    fs::create_dir_all(&indexes_dir).await?;
    fs::create_dir_all(&objects_dir).await?;

    let index_url = &manifest.asset_index.url;
    let index_path = indexes_dir.join(format!("{}.json", manifest.asset_index.id));

    // 2. Obtener el contenido del índice (descargar o leer si ya existe)
    let index_content = if index_path.exists() {
        fs::read_to_string(&index_path).await?
    } else {
        println!("Downloading assets index (5.json)...");
        let content = client.get(index_url).send().await?.text().await?;
        fs::write(&index_path, &content).await?;
        content
    };

    // 3. Parsear el contenido usando el nuevo nombre AssetIndexContent
    let index_data: AssetIndexContent = serde_json::from_str(&index_content)
        .map_err(|e| anyhow::anyhow!("Error parsing assets index: {}", e))?;

    println!("Verifing {} assets...", index_data.objects.len());

    // 4. Bucle de descarga
    for (name, obj) in index_data.objects {
        let prefix = &obj.hash[..2];
        let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, obj.hash);
        let folder = objects_dir.join(prefix);
        let file_path = folder.join(&obj.hash);

        if !file_path.exists() {
            fs::create_dir_all(&folder).await?;

            // Usamos un pequeño print para saber que está trabajando
            // pero solo cada 100 archivos para no saturar la consola
            // println!("Descargando asset: {}", name);

            if let Ok(res) = client.get(&url).send().await {
                if let Ok(bytes) = res.bytes().await {
                    let _ = fs::write(&file_path, &bytes).await;
                }
            }
        }
    }

    println!("All assets are ready to load up.");
    Ok(())
}



// ------------------------------------------ FABRIC

pub async fn get_fabric_manifest() -> anyhow::Result<FabricProfile> {
    let url = "https://meta.fabricmc.net/v2/versions/loader/1.20.1/0.19.2/profile/json";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;

    // Parseamos directamente a la nueva estructura de Fabric
    let manifest: FabricProfile = response.json().await?;

    Ok(manifest)
}

pub fn gen_fabric_path(lib: &FabricLibrary) -> String {
    let parts: Vec<&str> = lib.name.split(':').collect();
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    let jar_name = format!("{}-{}.jar", artifact, version);
    // DEVOLVEMOS SOLO LA RUTA: "net/fabricmc/..."
    format!("{}/{}/{}/{}", group, artifact, version, jar_name)
}

pub async fn download_fabric_libraries(manifest_fabric: &FabricProfile, base_path: &Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let libraries_dir = base_path.join("libraries");

    println!("Iniciando descarga de librerías de Fabric...");

    for lib in &manifest_fabric.libraries {
        // 1. Obtenemos solo la parte relativa (la carpeta)
        let relative_path = gen_fabric_path(&lib);

        // 2. Ruta en el DISCO (test_minecraft/libraries/net/fabricmc/...)
        let target_path = libraries_dir.join(&relative_path.replace('/', "\\")); // Cambiamos / por \ para Windows

        // 3. URL de INTERNET (https://maven.fabricmc.net/net/fabricmc/...)
        let download_url = format!("{}{}", lib.url, relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if !target_path.exists() {
            println!("Descargando Fabric Lib: {}", lib.name);

            let response = client.get(&download_url).send().await?;
            if response.status().is_success() {
                let bytes = response.bytes().await?;
                let mut file = fs::File::create(&target_path).await?;
                file.write_all(&bytes).await?;
            } else {
                println!("⚠️ Error 404 en Fabric Lib: {}", download_url);
            }
        }
    }

    println!("¡Librerías de Fabric descargadas con éxito!");
    Ok(())
}

pub fn gen_cp_fabric(
    manifest_mc: &VersionManifest,
    manifest_fabric: &FabricProfile,
    base_path: &std::path::Path
) -> String {
    let mut cp_parts = Vec::new();
    let libraries_dir = base_path.join("libraries");

    let separador_cp = if cfg!(target_os = "windows") { ";" } else { ":" };

    // 1. LIBS DE MINECRAFT (Ya lo tienes bien)
    for lib in &manifest_mc.libraries {
        if let Some(artifact) = &lib.downloads.artifact {
            if let Some(path) = &artifact.path {
                cp_parts.push(libraries_dir.join(path).to_str().unwrap().to_string());
            }
        }
    }

    // 2. LIBS DE FABRIC (Usamos la nueva función)
    for lib in &manifest_fabric.libraries {
        let rel_path = gen_fabric_path(lib);
        let full_path = libraries_dir.join(rel_path.replace('/', "\\"));
        cp_parts.push(full_path.to_str().unwrap().to_string());
    }

    // 3. CLIENT.JAR
    let client_jar = base_path.join("versions").join(&manifest_mc.id).join(format!("{}.jar", manifest_mc.id));
    cp_parts.push(client_jar.to_str().unwrap().to_string());

    cp_parts.join(separador_cp)
}

// --------------------------------- INYECT MODPACK

pub async fn inject_modpack(url: &str, base_path: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    println!("📥 Descargando modpack desde: {}", url);

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let mods_dir = base_path.join("mods");
    let config_dir = base_path.join("config");

    if mods_dir.exists() { fs::remove_dir_all(&mods_dir).await?; }
    if config_dir.exists() { fs::remove_dir_all(&config_dir).await?; }

    let target_dir = base_path;
    let buffer = Cursor::new(bytes);

    println!("📦 Extrayendo archivos en {:?}...", target_dir);

    extract(buffer, target_dir, false)?;

    println!("✅ Modpack inyectado correctamente.");
    Ok(())
}


// --------------------------------- MULTIPLATFORM

use directories::ProjectDirs;
use std::process::Command;

pub fn obtener_ruta_base() -> std::path::PathBuf {
    // Esto crea una ruta tipo:
    // Win: C:\Users\Nombre\AppData\Roaming\OxideMC\data
    // Lin: /home/nombre/.local/share/oxidemc
    // Mac: /Users/Nombre/Library/Application Support/OxideMC
    if let Some(proj_dirs) = ProjectDirs::from("com", "s3fflex", "oxidemc") {
        return proj_dirs.data_dir().to_path_buf();
    }
    // Si algo falla, usamos una carpeta local como "salvavidas"
    std::path::PathBuf::from(".minecraft")
}

// ----------------------------------------- JAVA

// En src/functions/mod.rs (necesitarás la crate 'zip' o 'zip-extract')
pub async fn download_java_runtime(base_path: &std::path::Path) -> anyhow::Result<()> {
    let runtime_dir = base_path.join("runtime");
    let java_exe = runtime_dir.join("bin/java.exe");

    if !java_exe.exists() {
        println!("☕ Java no encontrado. Descargando JRE 17 portable...");

        let url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_windows_hotspot_17.0.10_7.zip";

        let client = reqwest::Client::new();
        let bytes = client.get(url).send().await?.bytes().await?;

        // Extraemos el zip directamente en la carpeta runtime
        let cursor = std::io::Cursor::new(bytes);
        zip_extract::extract(cursor, &runtime_dir, true)?;

        println!("✅ Java instalado correctamente en AppData.");
    }

    Ok(())
}

pub fn check_java_version(target_version: u32) -> bool {
    // 1. Intentamos ejecutar "java -version"
    let output = Command::new("java")
        .arg("-version")
        .output();

    match output {
        Ok(out) => {
            // Java imprime la versión en stderr (errores), no en stdout
            let version_info = String::from_utf8_lossy(&out.stderr);

            // Buscamos el número de versión en el texto
            // El formato suele ser: java version "17.0.10" ...
            let version_string = format!("\"{}\"", target_version); // Buscamos "17"
            let alt_version_string = format!(" {}", target_version); // O buscamos 17 suelto

            if version_info.contains(&version_string) || version_info.contains(&alt_version_string) {
                println!("✅ Se detectó Java {} en el sistema.", target_version);
                return true;
            }

            println!("⚠️ Se encontró Java, pero no es la versión {}.", target_version);
            false
        }
        Err(_) => {
            println!("Java no está instalado en el sistema (no se encontró en el PATH).");
            false
        }
    }
}