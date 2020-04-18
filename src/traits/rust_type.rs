use crate::{Loader, LoaderError, LoaderTrait};

impl LoaderTrait<json_trait_rs::RustType> for Loader<json_trait_rs::RustType> {
    fn load_from_bytes(&self, content: &[u8]) -> Result<json_trait_rs::RustType, LoaderError>
    where
        Self: Sized,
    {
        let tm = String::from_utf8_lossy(content);
        let string_content = tm.trim();
        if string_content.is_empty() {
            Ok(json_trait_rs::RustType::Null)
        } else if "ERR" == string_content {
            Err(LoaderError::from(&"ERR"))
        } else if let Ok(value) = string_content.parse::<i32>() {
            Ok(json_trait_rs::RustType::from(value))
        } else if let Ok(value) = string_content.parse::<bool>() {
            Ok(json_trait_rs::RustType::from(value))
        } else {
            Ok(json_trait_rs::RustType::from(string_content))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::loaders::RustTypeLoader,
        url_helpers::{test_data_file_url, UrlError},
        LoaderError, LoaderTrait,
    };
    use std::{io, sync::Arc};
    use test_case::test_case;

    #[test]
    fn test_load_wrong_url_parse_error() {
        let expression_result = RustTypeLoader::default().load("this-is-a-wrong-url");
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
        let load_result = RustTypeLoader::default().load("http:/this-is-syntactically-invalid-url");
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
        let loader = RustTypeLoader::default();
        let mut non_exiting_file_url = test_data_file_url("testing/Null.txt");
        non_exiting_file_url.push_str("_not_existing");
        let load_result = loader.load(&non_exiting_file_url);
        if let Err(LoaderError::IOError(value)) = load_result {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected LoaderError::IOError(...), received {:?}", load_result);
        }
    }

    #[test_case("testing/Boolean.txt", json_trait_rs::RustType::from(false))]
    #[test_case("testing/Integer.txt", json_trait_rs::RustType::from(1))]
    #[test_case("testing/Null.txt", json_trait_rs::RustType::from(()))]
    #[test_case("testing/String.txt", json_trait_rs::RustType::from("Some Text"))]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: json_trait_rs::RustType) {
        let loader = RustTypeLoader::default();
        assert_eq!(loader.load(&test_data_file_url(file_path)).ok().unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let loader = RustTypeLoader::default();
        let load_result = loader.load(&test_data_file_url("testing/Invalid.txt"));
        if let Err(LoaderError::FormatError(value)) = load_result {
            assert_eq!("ERR", &value);
        } else {
            panic!("Expected LoaderError::FormatError(...), received {:?}", load_result);
        }
    }

    #[test_case("testing/Boolean.txt", json_trait_rs::RustType::from(false))]
    #[test_case("testing/Integer.txt", json_trait_rs::RustType::from(1))]
    #[test_case("testing/Null.txt", json_trait_rs::RustType::from(()))]
    #[test_case("testing/String.txt", json_trait_rs::RustType::from("Some Text"))]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: json_trait_rs::RustType) {
        let loader = RustTypeLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = RustTypeLoader::default();
        let load_result = mock_loader_request!(loader, "testing/Invalid.txt");
        if let Err(LoaderError::FormatError(value)) = load_result {
            assert_eq!("ERR", &value);
        } else {
            panic!("Expected LoaderError::FormatError(...), received {:?}", load_result);
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = RustTypeLoader::default();
        let load_result = mock_loader_request!(loader, 404, "testing/Null.txt");
        if let Err(LoaderError::FetchURLFailed(value)) = load_result {
            assert_eq!(value.status().map(|value| value.as_u16()), Some(404))
        } else {
            panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", load_result);
        }
    }
}
