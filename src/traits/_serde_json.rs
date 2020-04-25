use crate::{
    json::ConcreteJsonLoader,
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use serde_json::Value;

#[allow(clippy::module_name_repetitions)]
pub type SerdeJsonLoader = ConcreteJsonLoader<Value>;

impl LoaderTrait<Value> for SerdeJsonLoader {
    fn load_from_bytes(&self, content: &[u8]) -> Result<Value, LoaderError>
    where
        Self: Sized,
    {
        serde_json::from_slice(content).or_else(|ref serde_error| Err(LoaderError::from(serde_error)))
    }
}

#[cfg(test)]
mod tests {
    use super::SerdeJsonLoader;
    use crate::{loader::error::LoaderError, testing_helpers::MockLoaderRequestBuilder, traits::check_loader};
    use serde_json::Value;
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, SerdeJsonLoader>()
    }

    #[test_case("Boolean.json", "", &json![false])]
    #[test_case("Integer.json", "", &json![1])]
    #[test_case("Null.json", "", &json![null])]
    #[test_case("String.json", "", &json!["Some Text"])]
    #[test_case("Object.json", "", &json![{"key": "Some Text"}])]
    #[test_case("Object.json", "/key", &json!["Some Text"])]
    fn test_load_from_file_valid_content(file_name: &'static str, fragment: &str, expected_loaded_object: &Value) {
        assert_eq!(
            &*MockLoaderRequestBuilder::default()
                .http_path(format!("/#{}", fragment))
                .resp_body_file_path(vec!["serde_json", file_name])
                .build()
                .unwrap()
                .send_request(&SerdeJsonLoader::default())
                .unwrap(),
            expected_loaded_object,
        );
    }

    #[test]
    fn test_load_invalid_content() {
        assert!(matches!(
            MockLoaderRequestBuilder::default()
                .resp_body_file_path(vec!["serde_json", "Invalid.json"])
                .build()
                .unwrap()
                .send_request(&SerdeJsonLoader::default())
                .unwrap_err(),
            LoaderError::FormatError(value) if "EOF while parsing an object at line 2 column 0" == &value
        ));
    }
}
