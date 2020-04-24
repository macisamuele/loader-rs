use crate::{
    json::ConcreteJsonLoader,
    loader::{error::LoaderError, trait_::LoaderTrait},
};
use serde_yaml::Value;

#[allow(clippy::module_name_repetitions)]
pub type SerdeYamlLoader = ConcreteJsonLoader<Value>;

impl LoaderTrait<Value> for SerdeYamlLoader {
    fn load_from_bytes(&self, content: &[u8]) -> Result<Value, LoaderError>
    where
        Self: Sized,
    {
        serde_yaml::from_slice(content).or_else(|ref serde_error| Err(LoaderError::from(serde_error)))
    }
}

#[cfg(test)]
mod tests {
    use super::SerdeYamlLoader;
    use crate::{
        loader::{error::LoaderError, trait_::LoaderTrait},
        traits::check_loader,
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
        match SerdeYamlLoader::default().load("this-is-a-wrong-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        match SerdeYamlLoader::default().load("http:/this-is-syntactically-invalid-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let mut non_exiting_file_url = test_data_file_url("serde_yaml/Null.yaml").to_string();
        non_exiting_file_url.push_str("_not_existing");
        match SerdeYamlLoader::default().load(&non_exiting_file_url).unwrap_err() {
            LoaderError::IOError(value) => assert_eq!(value.kind(), io::ErrorKind::NotFound),
            loader_error => panic!("Expected LoaderError::IOError(...), received {:?}", loader_error),
        }
    }

    #[test_case("serde_yaml/Boolean.yaml", yaml![false])]
    #[test_case("serde_yaml/Integer.yaml", yaml![1])]
    #[test_case("serde_yaml/Null.yaml", yaml![null])]
    #[test_case("serde_yaml/String.yaml", yaml!["Some Text"])]
    #[test_case("serde_yaml/Object.yaml", yaml![{"key": "Some Text"}])]
    #[test_case("serde_yaml/Object.yaml#/key", yaml!["Some Text"])]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: Value) {
        assert_eq!(
            SerdeYamlLoader::default().get_or_fetch_with_result(&test_data_file_url(file_path)).ok().unwrap(),
            Arc::new(expected_loaded_object),
        );
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        match SerdeYamlLoader::default().load(&test_data_file_url("serde_yaml/Invalid.yaml").to_string()).unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("while parsing a node, did not find expected node content at line 2 column 1", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
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
        match mock_loader_request!(loader, "serde_yaml/Invalid.yaml").unwrap_err() {
            LoaderError::FormatError(value) => assert_eq!("while parsing a node, did not find expected node content at line 2 column 1", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = SerdeYamlLoader::default();
        match mock_loader_request!(loader, 404, "serde_yaml/Null.yaml").unwrap_err() {
            LoaderError::FetchURLFailed(value) => assert_eq!(value.status().map(|value| value.as_u16()), Some(404)),
            loader_error => panic!("Expected LoaderError::FetchURLFailed(...), received {:?}", loader_error),
        }
    }
}
