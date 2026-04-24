use mc_lib::OxideLauncher; // Cambia mc_lib por el nombre de tu paquete

#[tokio::test]
async fn test_install() {
    let launcher = OxideLauncher::new("TestUser");

    let result = launcher.full_install(None).await;

    assert!(result.is_ok(), "Installation process done successfully");
}

#[tokio::test]
async fn run() {
    let launcher = OxideLauncher::new("TestUser");

    let result = launcher.start().await;

    assert!(result.is_ok(), "Game closed");
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

    assert!(result.is_ok(), "La función de chequeo ha fallado técnicamente");

    let is_installed = result.unwrap();
    println!("¿Java 17 detectado?: {}", is_installed);

}