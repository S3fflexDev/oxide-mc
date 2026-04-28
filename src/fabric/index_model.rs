use serde::{Deserialize};

pub type FabricLoaderVersions = Vec<FabricLoaderVersion>;

#[derive(Deserialize, Debug, Clone)]
pub struct FabricLoaderVersion {
    pub loader: LoaderInfo,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoaderInfo {
    pub version: String,
    pub stable: bool,
}