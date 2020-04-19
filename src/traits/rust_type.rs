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
    use crate::{loader::error::LoaderError, testing_helpers::MockLoaderRequestBuilder, traits::check_loader};
    use json_trait_rs::RustType;
    use test_case::test_case;

    #[test]
    fn test_is_loader() {
        check_loader::<_, RustTypeLoader>()
    }

    #[test_case("Boolean.txt", &RustType::from(false))]
    #[test_case("Integer.txt", &RustType::from(1))]
    #[test_case("Null.txt", &RustType::from(()))]
    #[test_case("String.txt", &RustType::from("Some Text"))]
    fn test_load_valid_content(file_name: &'static str, expected_loaded_object: &RustType) {
        assert_eq!(
            &*MockLoaderRequestBuilder::default()
                .resp_body_file_path(vec!["rust_type", file_name])
                .build()
                .unwrap()
                .send_request(&RustTypeLoader::default())
                .unwrap(),
            expected_loaded_object,
        );
    }

    #[test]
    fn test_load_invalid_content() {
        match MockLoaderRequestBuilder::default()
            .resp_body_file_path(vec!["rust_type", "Invalid.txt"])
            .build()
            .unwrap()
            .send_request(&RustTypeLoader::default())
            .unwrap_err()
        {
            LoaderError::FormatError(value) => assert_eq!("ERR", &value),
            loader_error => panic!("Expected LoaderError::FormatError(...), received {:?}", loader_error),
        }
    }
}
