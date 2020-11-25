use crate::{
    loader::{
        error::LoaderError,
        trait_::{GetCache, GetClient},
        Loader,
    },
    thread_safe_cache::ThreadSafeCacheTrait,
    url_helpers::UrlError,
};
use json_trait_rs::{get_fragment, JsonType};
use reqwest::blocking::Client;
use std::sync::Arc;
use url::Url;

#[derive(Debug)]
pub struct ConcreteJsonLoader<T: JsonType>(Loader<T>);

impl<T: JsonType> Default for ConcreteJsonLoader<T> {
    fn default() -> Self {
        Self(Loader::default())
    }
}

impl<T: JsonType> GetCache<T> for ConcreteJsonLoader<T> {
    fn get_cache(&self) -> &dyn ThreadSafeCacheTrait<Url, T> {
        self.0.get_cache()
    }
}

impl<T: JsonType> GetClient<T> for ConcreteJsonLoader<T> {
    fn get_client(&self) -> &Client {
        self.0.get_client()
    }
}

pub trait ToOwnedJsonType: JsonType {
    fn to_owned_json_type(&self) -> Self;
}

impl<T: JsonType + Clone> ToOwnedJsonType for T {
    fn to_owned_json_type(&self) -> Self {
        self.clone()
    }
}

#[inline]
pub(in crate) fn extract_fragment_json_loader<T: ToOwnedJsonType>(fragment: &str, value: &T) -> Result<Arc<T>, LoaderError> {
    if let Some(fragment) = get_fragment(&*value, fragment) {
        Ok(Arc::new(fragment.to_owned_json_type()))
    } else {
        Err(LoaderError::InvalidURL(UrlError::JsonFragmentError(format!(
            "Fragment '{}' not found in {}",
            fragment,
            value.to_json_string()
        ))))
    }
}
