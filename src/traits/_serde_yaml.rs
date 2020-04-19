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
    use crate::{loader::error::LoaderError, testing_helpers::MockLoaderRequestBuilder, traits::check_loader};
    use serde_yaml::Value;
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

    #[test_case("Boolean.yaml", "", &yaml![false])]
    #[test_case("Integer.yaml", "", &yaml![1])]
    #[test_case("Null.yaml", "", &yaml![null])]
    #[test_case("String.yaml", "", &yaml!["Some Text"])]
    #[test_case("Object.yaml", "", &yaml![{"key": "Some Text"}])]
    #[test_case("Object.yaml", "/key", &yaml!["Some Text"])]
    fn test_load_from_file_valid_content(file_name: &'static str, fragment: &str, expected_loaded_object: &Value) {
        assert_eq!(
            &*MockLoaderRequestBuilder::default()
                .http_path(format!("/#{}", fragment))
                .resp_body_file_path(vec!["serde_yaml", file_name])
                .build()
                .unwrap()
                .send_request(&SerdeYamlLoader::default())
                .unwrap(),
            expected_loaded_object,
        );
    }

    #[test]
    fn test_load_invalid_content() {
        match MockLoaderRequestBuilder::default()
            .resp_body_file_path(vec!["serde_yaml", "Invalid.yaml"])
            .build()
            .unwrap()
            .send_request(&SerdeYamlLoader::default())
            .unwrap_err()
        {
            LoaderError::FormatError(value) => assert_eq!("while parsing a node, did not find expected node content at line 2 column 1", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }
}
