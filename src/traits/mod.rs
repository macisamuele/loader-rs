#[cfg(feature = "trait_serde_json")]
pub mod _serde_json;

#[cfg(feature = "trait_testing")]
pub mod _testing;

pub mod loaders {
    #[allow(unused_imports)]
    use crate::Loader;

    #[cfg(feature = "trait_serde_json")]
    pub type SerdeJsonLoader = Loader<serde_json::Value, serde_json::Error>;

    #[cfg(feature = "trait_testing")]
    pub type TestingLoader = Loader<json_trait_rs::testing::TestingType, ()>;
}
