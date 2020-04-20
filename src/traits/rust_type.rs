use crate::{
    json::ConcreteJsonLoader,
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use json_trait_rs::RustType;

#[allow(clippy::module_name_repetitions)]
pub type RustTypeLoader = ConcreteJsonLoader<RustType>;

impl LoaderTrait<RustType> for RustTypeLoader {
    fn load_from_bytes(&self, content: &[u8]) -> Result<RustType, LoaderError>
    where
        Self: Sized,
    {
        let tm = String::from_utf8_lossy(content);
        let string_content = tm.trim();
        if string_content.is_empty() {
            Ok(RustType::Null)
        } else if "ERR" == string_content {
            Err(LoaderError::from(&"ERR"))
        } else if let Ok(value) = string_content.parse::<i32>() {
            Ok(RustType::from(value))
        } else if let Ok(value) = string_content.parse::<bool>() {
            Ok(RustType::from(value))
        } else {
            Ok(RustType::from(string_content))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RustTypeLoader;
    use crate::{
        loader::{error::LoaderError, trait_::LoaderTrait},
        traits::check_loader,
        url_helpers::{test_data_file_url, UrlError},
    };
    use json_trait_rs::RustType;
    use std::{io, sync::Arc};
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, RustTypeLoader>()
    }

    #[test]
    fn test_load_wrong_url_parse_error() {
        match RustTypeLoader::default().load("this-is-a-wrong-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        match RustTypeLoader::default().load("http:/this-is-syntactically-invalid-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let mut non_exiting_file_url = test_data_file_url("rust_type/Null.txt").to_string();
        non_exiting_file_url.push_str("_not_existing");
        match RustTypeLoader::default().load(&non_exiting_file_url).unwrap_err() {
            LoaderError::IOError(value) => assert_eq!(value.kind(), io::ErrorKind::NotFound),
            loader_error => panic!("Expected LoaderError::IOError(...), received {:?}", loader_error),
        }
    }

    #[test_case("rust_type/Boolean.txt", RustType::from(false))]
    #[test_case("rust_type/Integer.txt", RustType::from(1))]
    #[test_case("rust_type/Null.txt", RustType::from(()))]
    #[test_case("rust_type/String.txt", RustType::from("Some Text"))]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: RustType) {
        assert_eq!(
            RustTypeLoader::default().load(&test_data_file_url(file_path).to_string()).ok().unwrap(),
            Arc::new(expected_loaded_object),
        );
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        match RustTypeLoader::default().load(&test_data_file_url("rust_type/Invalid.txt").to_string()).unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("ERR", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test_case("rust_type/Boolean.txt", RustType::from(false))]
    #[test_case("rust_type/Integer.txt", RustType::from(1))]
    #[test_case("rust_type/Null.txt", RustType::from(()))]
    #[test_case("rust_type/String.txt", RustType::from("Some Text"))]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: RustType) {
        let loader = RustTypeLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = RustTypeLoader::default();
        match mock_loader_request!(loader, "rust_type/Invalid.txt").unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("ERR", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = RustTypeLoader::default();
        match mock_loader_request!(loader, 404, "rust_type/Null.txt").unwrap_err() {
            LoaderError::FetchURLFailed(value) => assert_eq!(value.status().map(|value| value.as_u16()), Some(404)),
            loader_error => panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", loader_error),
        }
    }
}
