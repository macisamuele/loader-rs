use crate::{
    json::ConcreteJsonLoader,
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use json::{Error, JsonValue};

#[allow(clippy::module_name_repetitions)]
pub type JsonLoader = ConcreteJsonLoader<JsonValue>;

impl From<Error> for LoaderError {
    #[must_use]
    fn from(value: Error) -> Self {
        Self::from(&value)
    }
}

impl LoaderTrait<JsonValue> for JsonLoader {
    fn load_from_string(&self, content: &str) -> Result<JsonValue, LoaderError>
    where
        Self: Sized,
    {
        json::parse(content).or_else(|json_error| Err(json_error.into()))
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
    use crate::{
        loader::{error::LoaderError, trait_::LoaderTrait},
        traits::check_loader,
        url_helpers::{test_data_file_url, UrlError},
    };
    use json::{Error, JsonValue};
    use std::{io, sync::Arc};
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

    #[test]
    fn test_load_wrong_url_parse_error() {
        match JsonLoader::default().load("this-is-a-wrong-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        match JsonLoader::default().load("http:/this-is-syntactically-invalid-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let mut non_exiting_file_url = test_data_file_url("json/Null.json").to_string();
        non_exiting_file_url.push_str("_not_existing");
        match JsonLoader::default().load(&non_exiting_file_url).unwrap_err() {
            LoaderError::IOError(value) => assert_eq!(value.kind(), io::ErrorKind::NotFound),
            loader_error => panic!("Expected LoaderError::IOError(...), received {:?}", loader_error),
        }
    }

    #[test_case("json/Boolean.json", rust_json![false])]
    #[test_case("json/Integer.json", rust_json![1])]
    #[test_case("json/Null.json", rust_json![null])]
    #[test_case("json/String.json", rust_json!["Some Text"])]
    #[test_case("json/Object.json", rust_json![{"key": "Some Text"}])]
    #[test_case("json/Object.json#/key", rust_json!["Some Text"])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: JsonValue) {
        assert_eq!(
            JsonLoader::default().get_or_fetch_with_result(&test_data_file_url(file_path)).ok().unwrap(),
            Arc::new(expected_loaded_object)
        );
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        match JsonLoader::default().load(&test_data_file_url("json/Invalid.json").to_string()).unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!(value, Error::UnexpectedEndOfJson.to_string()),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test_case("json/Boolean.json", rust_json![false])]
    #[test_case("json/Integer.json", rust_json![1])]
    #[test_case("json/Null.json", rust_json![null])]
    #[test_case("json/String.json", rust_json!["Some Text"])]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: JsonValue) {
        let loader = JsonLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = JsonLoader::default();
        match mock_loader_request!(loader, "json/Invalid.json").unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!(value, Error::UnexpectedEndOfJson.to_string()),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = JsonLoader::default();
        match mock_loader_request!(loader, 404, "json/Null.json").unwrap_err() {
            LoaderError::FetchURLFailed(value) => assert_eq!(value.status().map(|value| value.as_u16()), Some(404)),
            loader_error => panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", loader_error),
        }
    }
}
