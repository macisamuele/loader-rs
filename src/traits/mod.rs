#[cfg(feature = "trait_testing")]
pub mod _testing;

pub mod loaders {
    #[allow(unused_imports)]
    use crate::Loader;

    #[cfg(feature = "trait_testing")]
    pub type TestingLoader = Loader<json_trait_rs::testing::TestingType, ()>;
}
