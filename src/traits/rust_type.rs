use crate::{
    json::ConcreteJsonLoader,
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use json_trait_rs::{RustType, ToRustType};

#[allow(clippy::module_name_repetitions)]
pub type RustTypeLoader = ConcreteJsonLoader<RustType>;

impl LoaderTrait<RustType> for RustTypeLoader {
    fn load_from_bytes(&self, content: &[u8]) -> Result<RustType, LoaderError>
    where
        Self: Sized,
    {
        // Sub optimal loader, as it essentially delegates the JSON parsing to serde_json
        // but it's good enough for testing, at least for the time being
        serde_json::from_slice::<serde_json::Value>(content)
            .map(|value| value.to_rust_type())
            .or_else(|ref serde_error| Err(LoaderError::from(serde_error)))
    }
}

#[cfg(test)]
mod tests {
    use super::RustTypeLoader;
    use crate::{loader::error::LoaderError, testing_helpers::MockLoaderRequestBuilder, traits::check_loader};
    use json_trait_rs::RustType;
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, RustTypeLoader>()
    }

    #[test_case("Boolean.json", &RustType::from(false))]
    #[test_case("Integer.json", &RustType::from(1))]
    #[test_case("Null.json", &RustType::from(()))]
    #[test_case("String.json", &RustType::from("Some Text"))]
    fn test_load_valid_content(file_name: &'static str, expected_loaded_object: &RustType) {
        assert_eq!(
            &*MockLoaderRequestBuilder::default()
                .resp_body_file_path(vec![file_name])
                .build()
                .unwrap()
                .send_request(&RustTypeLoader::default())
                .unwrap(),
            expected_loaded_object,
        );
    }

    #[test]
    fn test_load_invalid_content() {
        assert!(matches!(
            MockLoaderRequestBuilder::default()
                .resp_body_file_path(vec!["Invalid.json"])
                .build()
                .unwrap()
                .send_request(&RustTypeLoader::default())
                .unwrap_err(),
            LoaderError::FormatError(value) if "EOF while parsing an object at line 2 column 0" == &value
        ));
    }
}
