#[cfg(feature = "trait_json")]
pub mod _json;

#[cfg(feature = "trait_serde_json")]
pub mod _serde_json;

#[cfg(feature = "trait_serde_yaml")]
pub mod _serde_yaml;

pub mod rust_type;

pub mod loaders {
    use crate::Loader;

    #[cfg(feature = "trait_json")]
    pub type JsonLoader = Loader<json::JsonValue>;

    #[cfg(feature = "trait_serde_json")]
    pub type SerdeJsonLoader = Loader<serde_json::Value>;

    #[cfg(feature = "trait_serde_yaml")]
    pub type SerdeYamlLoader = Loader<serde_yaml::Value>;

    pub type RustTypeLoader = Loader<::json_trait_rs::RustType>;
}

#[cfg(test)]
mod test_loaders_do_implement_loader_trait {
    #[cfg(feature = "trait_json")]
    use crate::traits::loaders::JsonLoader;
    #[cfg(feature = "trait_serde_json")]
    use crate::traits::loaders::SerdeJsonLoader;
    #[cfg(feature = "trait_serde_yaml")]
    use crate::traits::loaders::SerdeYamlLoader;
    use crate::{traits::loaders::RustTypeLoader, LoaderTrait};
    use json_trait_rs::{JsonType, RustType};

    #[allow(dead_code)]
    fn check<T: JsonType<T> + Into<RustType>, L: LoaderTrait<T>>(_loader: &L) {}

    #[cfg(feature = "trait_json")]
    #[test]
    fn test_json_loader() {
        check(&JsonLoader::default());
    }

    #[cfg(feature = "trait_serde_json")]
    #[test]
    fn test_serde_json_loader() {
        check(&SerdeJsonLoader::default());
    }

    #[cfg(feature = "trait_serde_yaml")]
    #[test]
    fn test_serde_yaml_loader() {
        check(&SerdeYamlLoader::default());
    }

    #[test]
    fn test_rust_type_loader() {
        check(&RustTypeLoader::default());
    }
}
