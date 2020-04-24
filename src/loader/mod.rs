pub mod error;
pub mod trait_;

use crate::thread_safe_cache::{ThreadSafeCacheImpl, ThreadSafeCacheTrait};
use trait_::GetCache;
use url::Url;

#[cfg(test)]
use error::LoaderError;
use reqwest::blocking::Client;
use trait_::GetClient;
#[cfg(test)]
use trait_::LoaderTrait;

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
#[derive(Debug, Default)]
pub(in crate) struct TestStringLoader(Loader<String>);

#[cfg(test)]
impl From<std::str::Utf8Error> for LoaderError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::FormatError(format!("{:?}", error))
    }
}

#[cfg(test)]
impl GetClient<String> for TestStringLoader {
    fn get_client(&self) -> &Client {
        self.0.get_client()
    }
}

#[cfg(test)]
impl GetCache<String> for TestStringLoader {
    fn get_cache(&self) -> &dyn ThreadSafeCacheTrait<Url, String> {
        self.0.get_cache()
    }
}

#[cfg(test)]
impl LoaderTrait<String> for TestStringLoader {
    fn load_from_bytes(&self, content: &[u8]) -> Result<String, LoaderError> {
        Ok(std::str::from_utf8(content)?.to_string())
    }
}
