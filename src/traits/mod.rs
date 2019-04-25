#[cfg(feature = "trait_serde_json")]
pub mod _serde_json;

#[cfg(feature = "trait_serde_yaml")]
pub mod _serde_yaml;

#[cfg(feature = "trait_testing")]
pub mod _testing;

pub mod loaders {
    #[allow(unused_imports)]
    use crate::Loader;

    #[cfg(feature = "trait_serde_json")]
    pub type SerdeJsonLoader = Loader<serde_json::Value, serde_json::Error>;

    #[cfg(feature = "trait_serde_yaml")]
    pub type SerdeYamlLoader = Loader<serde_yaml::Value, serde_yaml::Error>;

    #[cfg(feature = "trait_testing")]
    pub type TestingLoader = Loader<json_trait_rs::testing::TestingType, ()>;
}
