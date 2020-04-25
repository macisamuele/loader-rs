pub mod error;
pub mod trait_;

use crate::thread_safe_cache::{ThreadSafeCacheImpl, ThreadSafeCacheTrait};
use reqwest::blocking::Client;
use trait_::{GetCache, GetClient};
use url::Url;

lazy_static::lazy_static! {
    pub(in crate) static ref DEFAULT_CLIENT: Client = Client::new();
}

#[derive(Debug)]
pub struct Loader<T> {
    cache: ThreadSafeCacheImpl<Url, T>,
}

impl<T> Default for Loader<T> {
    fn default() -> Self {
        Self {
            cache: ThreadSafeCacheImpl::default(),
        }
    }
}

impl<T> GetCache<T> for Loader<T> {
    fn get_cache(&self) -> &dyn ThreadSafeCacheTrait<Url, T> {
        &self.cache
    }
}

impl<T> GetClient<T> for Loader<T> {
    fn get_client(&self) -> &Client {
        &*DEFAULT_CLIENT
    }
}

#[cfg(test)]
pub(in crate) mod testing {
    use super::{
        error::LoaderError,
        trait_::{GetCache, GetClient, LoaderTrait},
        Loader,
    };
    use crate::thread_safe_cache::ThreadSafeCacheTrait;
    use reqwest::blocking::Client;
    use url::Url;

    #[derive(Debug, Default)]
    pub(in crate) struct TestStringLoader(Loader<String>);

    impl GetClient<String> for TestStringLoader {
        fn get_client(&self) -> &Client {
            self.0.get_client()
        }
    }

    impl GetCache<String> for TestStringLoader {
        fn get_cache(&self) -> &dyn ThreadSafeCacheTrait<Url, String> {
            self.0.get_cache()
        }
    }

    impl LoaderTrait<String> for TestStringLoader {
        fn load_from_bytes(&self, content: &[u8]) -> Result<String, LoaderError> {
            match std::str::from_utf8(content) {
                Ok(str_ref) => Ok(str_ref.to_string()),
                Err(utf8_error) => Err(LoaderError::FormatError(utf8_error.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{error::LoaderError, testing::TestStringLoader, trait_::LoaderTrait};
    use crate::{
        testing_helpers::{test_data_file_path, MockLoaderRequestBuilder},
        url_helpers::UrlError,
    };
    use url::Url;

    #[test]
    fn test_load_wrong_url_parse_error() {
        match TestStringLoader::default().load("this-is-a-wrong-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        match TestStringLoader::default().load("http:/this-is-syntactically-invalid-url").unwrap_err() {
            LoaderError::InvalidURL(UrlError::SyntaxViolation(url::SyntaxViolation::ExpectedDoubleSlash)) => (),
            loader_error => panic!(
                "Expected LoaderError::InvalidURL(UrlError::ParseError(url::ParseError::RelativeUrlWithoutBase)), received {:?}",
                loader_error,
            ),
        }
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let mut non_exiting_file_url = Url::from_file_path(test_data_file_path(&["empty"]).unwrap().as_path()).unwrap().to_string();
        non_exiting_file_url.push_str("_not_existing");

        assert!(matches!(
            TestStringLoader::default().load(&non_exiting_file_url).unwrap_err(),
            LoaderError::IOError(value) if value.kind() == std::io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn test_load_from_not_existing_url() {
        assert!(matches!(
            MockLoaderRequestBuilder::default()
                .resp_status_code(404)
                .build()
                .unwrap()
                .send_request(&TestStringLoader::default())
                .unwrap_err(),
            LoaderError::FetchURLFailed(value) if value.status().map(|value| value.as_u16()) == Some(404)
        ));
    }
}
