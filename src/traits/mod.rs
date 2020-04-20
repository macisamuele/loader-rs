#[cfg(feature = "trait_json")]
mod _json;

#[cfg(feature = "trait_serde_json")]
mod _serde_json;

#[cfg(feature = "trait_serde_yaml")]
mod _serde_yaml;

#[cfg(feature = "trait_json_trait_rs")]
mod rust_type;

#[cfg(all(test, feature = "trait_json_trait_rs"))]
use crate::loader::trait_::LoaderTrait;

#[cfg(all(test, feature = "trait_json_trait_rs"))]
use json_trait_rs::JsonType;

pub mod loaders {
    #[cfg(feature = "trait_json")]
    pub use super::_json::JsonLoader;

    #[cfg(feature = "trait_serde_json")]
    pub use super::_serde_json::SerdeJsonLoader;

    #[cfg(feature = "trait_serde_yaml")]
    pub use super::_serde_yaml::SerdeYamlLoader;

    #[cfg(feature = "trait_json_trait_rs")]
    pub use super::rust_type::RustTypeLoader;
}

#[cfg(all(test, feature = "trait_json_trait_rs"))]
fn check_loader<T: JsonType, L: Default + LoaderTrait<T>>() {}
