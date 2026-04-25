use oxide_mc::OxideLauncher;

#[tokio::test]
#[ignore]
async fn test_install() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.full_install(None).await?;
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

    launcher.java_download(21).await?;
    Ok(())
}

#[tokio::test]
async fn check_java_version_test() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    launcher.check_java().await?;
    Ok(())
}
