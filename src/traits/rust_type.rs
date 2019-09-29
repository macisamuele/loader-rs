use crate::{Loader, LoaderError, LoaderTrait};
use json_trait_rs::RustType;

impl From<()> for LoaderError<()> {
    fn from(_: ()) -> Self {
        LoaderError::FormatError(())
    }
}

impl LoaderTrait<RustType, ()> for Loader<RustType, ()> {
    fn load_from_string(content: String) -> Result<RustType, LoaderError<()>>
    where
        Self: Sized,
    {
        let content = content.trim();
        if content.is_empty() {
            Ok(RustType::Null)
        } else if "ERR" == content {
            Err(())?
        } else if let Ok(value) = content.parse::<i32>() {
            Ok(RustType::from(value))
        } else if let Ok(value) = content.parse::<bool>() {
            Ok(RustType::from(value))
        } else {
            Ok(RustType::from(content))
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
    use json_trait_rs::RustType;
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
        let load_result = loader.load(non_exiting_file_url);
        if let Err(LoaderError::IOError(value)) = load_result {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected LoaderError::IOError(...), received {:?}", load_result);
        }
    }

    #[test_case("testing/Boolean.txt", RustType::from(false))]
    #[test_case("testing/Integer.txt", RustType::from(1))]
    #[test_case("testing/Null.txt", RustType::from(()))]
    #[test_case("testing/String.txt", RustType::from("Some Text"))]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: RustType) {
        let loader = RustTypeLoader::default();
        assert_eq!(loader.load(test_data_file_url(file_path)).ok().unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let loader = RustTypeLoader::default();
        let load_result = loader.load(test_data_file_url("testing/Invalid.txt"));
        if let Err(LoaderError::FormatError(())) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(()), received {:?}", load_result);
        }
    }

    #[test_case("testing/Boolean.txt", RustType::from(false))]
    #[test_case("testing/Integer.txt", RustType::from(1))]
    #[test_case("testing/Null.txt", RustType::from(()))]
    #[test_case("testing/String.txt", RustType::from("Some Text"))]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: RustType) {
        let loader = RustTypeLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = RustTypeLoader::default();
        let load_result = mock_loader_request!(loader, "testing/Invalid.txt");
        if let Err(LoaderError::FormatError(())) = load_result {
        } else {
            panic!("Expected LoaderError::FormatError(()), received {:?}", load_result);
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = RustTypeLoader::default();
        let load_result = mock_loader_request!(loader, 404, "testing/Null.txt");
        if let Err(LoaderError::FetchURLFailed(value)) = load_result {
            assert_eq!(value.status().and_then(|value| Some(value.as_u16())), Some(404))
        } else {
            panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", load_result);
        }
    }
}
