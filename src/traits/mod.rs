#[cfg(feature = "trait_json")]
mod _json;

#[cfg(feature = "trait_serde_json")]
mod _serde_json;

#[cfg(feature = "trait_serde_yaml")]
mod _serde_yaml;

#[cfg(all(feature = "json-loader", feature = "testing-helpers"))]
mod rust_type;

#[cfg(all(test, feature = "json-loader"))]
use crate::loader::trait_::LoaderTrait;

#[cfg(all(test, feature = "json-loader"))]
use json_trait_rs::JsonType;

pub mod loaders {
    #[cfg(feature = "trait_json")]
    pub use super::_json::JsonLoader;

    #[cfg(feature = "trait_serde_json")]
    pub use super::_serde_json::SerdeJsonLoader;

    #[cfg(feature = "trait_serde_yaml")]
    pub use super::_serde_yaml::SerdeYamlLoader;

    // RustTypeLoader is exposed only if testing-helpers feature is enabled
    // because the Loader depends on serde_json loading capabilities (which for
    // example are not able to cover the i128 case, but for testing is good enough)
    #[cfg(all(feature = "json-loader", feature = "testing-helpers"))]
    pub use super::rust_type::RustTypeLoader;
}

#[allow(dead_code)]
#[cfg(all(test, feature = "json-loader"))]
fn check_loader<T: JsonType, L: Default + LoaderTrait<T>>() {}
