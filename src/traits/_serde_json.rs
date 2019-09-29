use crate::{Loader, LoaderError, LoaderTrait};
use serde_json;

impl From<serde_json::Error> for LoaderError<serde_json::Error> {
    fn from(value: serde_json::Error) -> Self {
        LoaderError::FormatError(value)
    }
}

impl LoaderTrait<serde_json::Value, serde_json::Error> for Loader<serde_json::Value, serde_json::Error> {
    fn load_from_string(content: String) -> Result<serde_json::Value, LoaderError<serde_json::Error>>
    where
        Self: Sized,
    {
        match serde_json::from_str(&content) {
            Ok(value) => Ok(value),
            Err(serde_error) => Err(serde_error)?,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::loaders::SerdeJsonLoader,
        url_helpers::{test_data_file_url, UrlError},
        LoaderError, LoaderTrait,
    };
    use serde_json;
    use std::{io, sync::Arc};
    use test_case_derive::test_case;

    #[test]
    fn test_load_wrong_url_parse_error() {
        let expression_result = SerdeJsonLoader::default().load("this-is-a-wrong-url");
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
        let load_result = SerdeJsonLoader::default().load("http:/this-is-syntactically-invalid-url");
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
        let loader = SerdeJsonLoader::default();
        let mut non_exiting_file_url = test_data_file_url("serde_json/Null.json");
        non_exiting_file_url.push_str("_not_existing");
        let load_result = loader.load(non_exiting_file_url);
        if let Err(LoaderError::IOError(value)) = load_result {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected LoaderError::IOError(...), received {:?}", load_result);
        }
    }

    #[test_case("serde_json/Boolean.json", json![false])]
    #[test_case("serde_json/Integer.json", json![1])]
    #[test_case("serde_json/Null.json", json![null])]
    #[test_case("serde_json/String.json", json!["Some Text"])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: serde_json::Value) {
        let loader = SerdeJsonLoader::default();
        assert_eq!(loader.load(test_data_file_url(file_path)).ok().unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let loader = SerdeJsonLoader::default();
        let load_result = loader.load(test_data_file_url("serde_json/Invalid.json"));
        if let Err(LoaderError::FormatError(serde_json::Error { .. })) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(serde_json::Error {{ .. }}), received {:?}", load_result);
        }
    }

    #[test_case("serde_json/Boolean.json", json![false])]
    #[test_case("serde_json/Integer.json", json![1])]
    #[test_case("serde_json/Null.json", json![null])]
    #[test_case("serde_json/String.json", json!["Some Text"])]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: serde_json::Value) {
        let loader = SerdeJsonLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = SerdeJsonLoader::default();
        let load_result = mock_loader_request!(loader, "serde_json/Invalid.json");
        if let Err(LoaderError::FormatError(serde_json::Error { .. })) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(serde_json::Error {{ .. }}), received {:?}", load_result);
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = SerdeJsonLoader::default();
        let load_result = mock_loader_request!(loader, 404, "serde_json/Null.json");
        if let Err(LoaderError::FetchURLFailed(value)) = load_result {
            assert_eq!(value.status().and_then(|value| Some(value.as_u16())), Some(404))
        } else {
            panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", load_result);
        }
    }
}
