use crate::{Loader, LoaderError, LoaderTrait};
use json;

impl From<json::Error> for LoaderError<json::Error> {
    fn from(value: json::Error) -> Self {
        Self::FormatError(value)
    }
}

impl LoaderTrait<json::JsonValue, json::Error> for Loader<json::JsonValue, json::Error> {
    fn load_from_string(content: String) -> Result<json::JsonValue, LoaderError<json::Error>>
    where
        Self: Sized,
    {
        json::parse(&content).or_else(|json_error| Err(json_error)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::loaders::JsonLoader,
        url_helpers::{test_data_file_url, UrlError},
        LoaderError, LoaderTrait,
    };
    use json;
    use std::{io, sync::Arc};
    use test_case::test_case;

    #[test]
    fn test_load_wrong_url_parse_error() {
        let expression_result = JsonLoader::default().load("this-is-a-wrong-url");
        if let Err(LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase))) = expression_result {
        } else {
            panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                expression_result
            );
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        let load_result = JsonLoader::default().load("http:/this-is-syntactically-invalid-url");
        if let Err(LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash))) = load_result {
        } else {
            panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                load_result
            );
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let loader = JsonLoader::default();
        let mut non_exiting_file_url = test_data_file_url("json/Null.json");
        non_exiting_file_url.push_str("_not_existing");
        let load_result = loader.load(non_exiting_file_url);
        if let Err(LoaderError::IOError(value)) = load_result {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected LoaderError::IOError(...), received {:?}", load_result);
        }
    }

    #[test_case("json/Boolean.json", rust_json![false])]
    #[test_case("json/Integer.json", rust_json![1])]
    #[test_case("json/Null.json", rust_json![null])]
    #[test_case("json/String.json", rust_json!["Some Text"])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: json::JsonValue) {
        let loader = JsonLoader::default();
        assert_eq!(loader.load(test_data_file_url(file_path)).ok().unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let loader = JsonLoader::default();
        let load_result = loader.load(test_data_file_url("json/Invalid.json"));
        if let Err(LoaderError::FormatError(json::Error::UnexpectedEndOfJson)) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(json::Error::UnexpectedEndOfJson), received {:?}", load_result);
        }
    }

    #[test_case("json/Boolean.json", rust_json![false])]
    #[test_case("json/Integer.json", rust_json![1])]
    #[test_case("json/Null.json", rust_json![null])]
    #[test_case("json/String.json", rust_json!["Some Text"])]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: json::JsonValue) {
        let loader = JsonLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = JsonLoader::default();
        let load_result = mock_loader_request!(loader, "json/Invalid.json");
        if let Err(LoaderError::FormatError(json::Error::UnexpectedEndOfJson)) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(json::Error::UnexpectedEndOfJson), received {:?}", load_result);
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = JsonLoader::default();
        let load_result = mock_loader_request!(loader, 404, "json/Null.json");
        if let Err(LoaderError::FetchURLFailed(value)) = load_result {
            assert_eq!(value.status().and_then(|value| Some(value.as_u16())), Some(404))
        } else {
            panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", load_result);
        }
    }
}
