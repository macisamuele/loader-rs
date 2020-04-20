use crate::loader::{error::LoaderError, trait_::LoaderTrait, Loader};
use serde_yaml::Value;

pub type SerdeYamlLoader = Loader<Value>;

impl LoaderTrait<Value> for Loader<Value> {
    fn load_from_bytes(&self, content: &[u8]) -> Result<Value, LoaderError>
    where
        Self: Sized,
    {
        serde_yaml::from_slice(content).or_else(|ref serde_error| Err(LoaderError::from(serde_error)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        loader::{error::LoaderError, trait_::LoaderTrait},
        traits::{check_loader, loaders::SerdeYamlLoader},
        url_helpers::{test_data_file_url, UrlError},
    };
    use serde_yaml::Value;
    use std::{io, sync::Arc};
    use test_case::test_case;

    macro_rules! yaml {
        ($($json:tt)+) => {{
            serde_yaml::from_str(
                serde_json::to_string(&json![$($json)+]).unwrap().as_str(),
            ).unwrap()
        }};
    }

    #[test]
    fn test_is_loader() {
        check_loader::<_, SerdeYamlLoader>()
    }

    #[test]
    fn test_load_wrong_url_parse_error() {
        let expression_result = SerdeYamlLoader::default().load("this-is-a-wrong-url");
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
        let load_result = SerdeYamlLoader::default().load("http:/this-is-syntactically-invalid-url");
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
        let loader = SerdeYamlLoader::default();
        let mut non_exiting_file_url = test_data_file_url("serde_yaml/Null.yaml");
        non_exiting_file_url.push_str("_not_existing");
        let load_result = loader.load(&non_exiting_file_url);
        if let Err(LoaderError::IOError(value)) = load_result {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        } else {
            panic!("Expected LoaderError::IOError(...), received {:?}", load_result);
        }
    }

    #[test_case("serde_yaml/Boolean.yaml", yaml![false])]
    #[test_case("serde_yaml/Integer.yaml", yaml![1])]
    #[test_case("serde_yaml/Null.yaml", yaml![null])]
    #[test_case("serde_yaml/String.yaml", yaml!["Some Text"])]
    #[test_case("serde_yaml/Object.yaml", yaml![{"key": "Some Text"}])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: Value) {
        let loader = SerdeYamlLoader::default();
        assert_eq!(loader.load(&test_data_file_url(file_path)).ok().unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        let loader = SerdeYamlLoader::default();
        let load_result = loader.load(&test_data_file_url("serde_yaml/Invalid.yaml"));
        if let Err(LoaderError::FormatError(value)) = load_result {
            assert_eq!("while parsing a node, did not find expected node content at line 2 column 1", &value);
        } else {
            panic!("Expected LoaderError::FormatError(...), received {:?}", load_result);
        }
    }

    #[test_case("serde_yaml/Boolean.yaml", yaml![false])]
    #[test_case("serde_yaml/Integer.yaml", yaml![1])]
    #[test_case("serde_yaml/Null.yaml", yaml![null])]
    #[test_case("serde_yaml/String.yaml", yaml!["Some Text"])]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: Value) {
        let loader = SerdeYamlLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), Arc::new(expected_loaded_object));
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = SerdeYamlLoader::default();
        let load_result = mock_loader_request!(loader, "serde_yaml/Invalid.yaml");
        if let Err(LoaderError::FormatError(value)) = load_result {
            assert_eq!("while parsing a node, did not find expected node content at line 2 column 1", &value);
        } else {
            panic!("Expected LoaderError::FormatError(...), received {:?}", load_result);
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = SerdeYamlLoader::default();
        let load_result = mock_loader_request!(loader, 404, "serde_yaml/Null.yaml");
        if let Err(LoaderError::FetchURLFailed(value)) = load_result {
            assert_eq!(value.status().map(|value| value.as_u16()), Some(404))
        } else {
            panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", load_result);
        }
    }
}
