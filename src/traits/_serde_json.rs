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
    use crate::{
        loader::{error::LoaderError, trait_::LoaderTrait},
        traits::check_loader,
        url_helpers::{test_data_file_url, UrlError},
    };
    use serde_json::Value;
    use std::{io, sync::Arc};
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, SerdeJsonLoader>()
    }

    #[test]
    fn test_load_wrong_url_parse_error() {
        match SerdeJsonLoader::default().load("this-is-a-wrong-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        match SerdeJsonLoader::default().load("http:/this-is-syntactically-invalid-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let mut non_exiting_file_url = test_data_file_url("serde_json/Null.json").to_string();
        non_exiting_file_url.push_str("_not_existing");
        match SerdeJsonLoader::default().load(&non_exiting_file_url).unwrap_err() {
            LoaderError::IOError(value) => assert_eq!(value.kind(), io::ErrorKind::NotFound),
            loader_error => panic!("Expected LoaderError::IOError(...), received {:?}", loader_error),
        }
    }

    #[test_case("serde_json/Boolean.json", json![false])]
    #[test_case("serde_json/Integer.json", json![1])]
    #[test_case("serde_json/Null.json", json![null])]
    #[test_case("serde_json/String.json", json!["Some Text"])]
    #[test_case("serde_json/Object.json", json![{"key": "Some Text"}])]
    #[test_case("serde_json/Object.json#/key", json!["Some Text"])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: Value) {
        assert_eq!(
            SerdeJsonLoader::default().get_or_fetch_with_result(&test_data_file_url(file_path)).ok().unwrap(),
            Arc::new(expected_loaded_object),
        );
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        match SerdeJsonLoader::default().load(&test_data_file_url("serde_json/Invalid.json").to_string()).unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("EOF while parsing an object at line 2 column 0", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test_case("serde_json/Boolean.json", json![false])]
    #[test_case("serde_json/Integer.json", json![1])]
    #[test_case("serde_json/Null.json", json![null])]
    #[test_case("serde_json/String.json", json!["Some Text"])]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: Value) {
        let loader = SerdeJsonLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = SerdeJsonLoader::default();
        match mock_loader_request!(loader, "serde_json/Invalid.json").unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("EOF while parsing an object at line 2 column 0", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = SerdeJsonLoader::default();
        match mock_loader_request!(loader, 404, "serde_json/Null.json").unwrap_err() {
            LoaderError::FetchURLFailed(value) => assert_eq!(value.status().map(|value| value.as_u16()), Some(404)),
            loader_error => panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", loader_error),
        }
    }
}
