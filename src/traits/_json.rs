use crate::{
    json::{extract_fragment_json_loader, ConcreteJsonLoader},
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use json::{Error, JsonValue};
use std::sync::Arc;

#[allow(clippy::module_name_repetitions)]
pub type JsonLoader = ConcreteJsonLoader<JsonValue>;

impl From<Error> for LoaderError {
    #[must_use]
    fn from(value: Error) -> Self {
        Self::from(&value)
    }
}

impl LoaderTrait<JsonValue> for JsonLoader {
    fn extract_fragment(&self, fragment: &str, value: Arc<JsonValue>) -> Result<Arc<JsonValue>, LoaderError> {
        extract_fragment_json_loader(fragment, &value)
    }

    fn load_from_string(&self, content: &str) -> Result<JsonValue, LoaderError>
    where
        Self: Sized,
    {
        json::parse(content).map_err(|json_error| json_error.into())
    }

    fn load_from_bytes(&self, content: &[u8]) -> Result<JsonValue, LoaderError>
    where
        Self: Sized,
    {
        match std::str::from_utf8(content) {
            Ok(string_value) => self.load_from_string(string_value),
            Err(_) => Err(Error::FailedUtf8Parsing.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsonLoader;
    use crate::{loader::error::LoaderError, testing_helpers::MockLoaderRequestBuilder, traits::check_loader};
    use json::{Error, JsonValue};
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, JsonLoader>()
    }

    macro_rules! rust_json {
        ($($json:tt)+) => {{
            json::parse(
                serde_json::to_string(&json![$($json)+]).unwrap().as_str(),
            ).unwrap()
        }};
    }

    #[test_case("Boolean.json", "", &rust_json![false])]
    #[test_case("Integer.json", "", &rust_json![1])]
    #[test_case("Null.json", "", &rust_json![null])]
    #[test_case("String.json", "", &rust_json!["Some Text"])]
    #[test_case("Object.json", "", &rust_json![{"key": "Some Text"}])]
    #[test_case("Object.json", "/key", &rust_json!["Some Text"])]
    fn test_load_from_file_valid_content(file_name: &'static str, fragment: &str, expected_loaded_object: &JsonValue) {
        assert_eq!(
            &*MockLoaderRequestBuilder::default()
                .http_path(format!("/#{}", fragment))
                .resp_body_file_path(vec![file_name])
                .build()
                .unwrap()
                .send_request(&JsonLoader::default())
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
                .send_request(&JsonLoader::default())
                .unwrap_err(),
            LoaderError::FormatError(value) if Error::UnexpectedEndOfJson.to_string() == value
        ));
    }
}
