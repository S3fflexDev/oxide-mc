use oxide_mc::OxideLauncher;

#[tokio::test]
#[ignore]
async fn test_install() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.full_install(None, "1.16.5", oxide_mc::state::models::ModLoader::Fabric, false).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn run() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.start().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn java_donwload() -> anyhow::Result<()> {
    let mut launcher = OxideLauncher::new("TestUser");

    launcher.java_download(8).await?;
    Ok(())
}

#[tokio::test]
async fn check_java_version_test() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    launcher.check_java().await?;
    Ok(())
}

#[tokio::test]
async fn check_game_installed() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    let result = launcher.check_game("1.20.1", "vanilla").await?;
    println!("{:#?}", result);
    Ok(())
}