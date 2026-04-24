use mc_lib::OxideLauncher; // Cambia mc_lib por el nombre de tu paquete

#[tokio::test]
async fn test_install() {
    let launcher = OxideLauncher::new("TestUser");

    let result = launcher.full_install(None).await;

    assert!(result.is_ok(), "La instalación debería haber funcionado");
}

#[tokio::test]
async fn run() {
    let launcher = OxideLauncher::new("TestUser");

    let result = launcher.start().await;

    assert!(result.is_ok(), "Juego cerrado");
}

#[tokio::test]
async fn java_donwload() {
    let mut launcher = OxideLauncher::new("TestUser");

    let result = launcher.java_download().await;

    assert!(result.is_ok(), "Java installed.");
}

#[tokio::test]
async fn check_java_version_test() {

    let launcher = OxideLauncher::new("TestUser");

    let result = launcher.check_java(17).await;

    let is_installed = result.expect("Fallo al ejecutar el comando java");

    println!("¿Java 17 detectado?: {}", is_installed);

    assert!(is_installed, "No se detectó Java 17 en el sistema");

}