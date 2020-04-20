use crate::loader::error::LoaderError;
use reqwest::blocking::Client;
use std::{sync::Arc, time::Duration};
use url::Url;

use crate::{
    thread_safe_cache::ThreadSafeCacheTrait,
    url_helpers::{normalize_url_for_cache, parse_and_normalize_url},
};

pub trait GetCache<T> {
    fn get_cache(&self) -> &dyn ThreadSafeCacheTrait<Url, T>;
}

pub trait GetClient<T> {
    fn get_client(&self) -> &Client;
}

fn remove_fragment_from_url(url: &Url) -> Url {
    let mut fragment_less_key = url.clone();
    fragment_less_key.set_fragment(None);
    fragment_less_key
}

#[allow(clippy::module_name_repetitions)]
pub trait LoaderTrait<T>: GetClient<T> + GetCache<T> {
    fn get_from_cache(&self, key: &Url) -> Option<Arc<T>> {
        self.get_cache().get(key)
    }

    fn save_in_cache(&self, key: &Url, value: &Arc<T>) {
        self.get_cache().set(key, value.clone())
    }

    fn load_from_string(&self, content: &str) -> Result<T, LoaderError> {
        self.load_from_bytes(content.as_bytes())
    }

    fn load_from_bytes(&self, content: &[u8]) -> Result<T, LoaderError>;

    fn load(&self, url: &str) -> Result<Arc<T>, LoaderError> {
        self.load_with_timeout(url, Duration::from_millis(30_000))
    }

    fn load_with_timeout(&self, url: &str, timeout: Duration) -> Result<Arc<T>, LoaderError> {
        let url = parse_and_normalize_url(url)?;

        Ok(Arc::new(if url.scheme() == "file" {
            self.load_from_bytes(std::fs::read(url.to_file_path().unwrap())?.as_slice())
        } else {
            self.load_from_bytes(self.get_client().get(url.as_ref()).timeout(timeout).send()?.error_for_status()?.bytes()?.as_ref())
        }?))
    }

    fn get_or_fetch_with_result(&self, key: &Url) -> Result<Arc<T>, LoaderError> {
        let fragmentless_url = &remove_fragment_from_url(key);
        let value = if let Some(arc_value) = self.get_from_cache(fragmentless_url) {
            arc_value
        } else {
            let arc_value = self.load(key.as_str())?;
            self.save_in_cache(&normalize_url_for_cache(fragmentless_url), &arc_value);
            arc_value
        };
        if let Some(fragment) = key.fragment() {
            self.extract_fragment(fragment, value)
        } else {
            Ok(value)
        }
    }

    fn extract_fragment(&self, _fragment: &str, value: Arc<T>) -> Result<Arc<T>, LoaderError> {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{remove_fragment_from_url, LoaderTrait};
    use test_case::test_case;
    use url::Url;

    #[test_case("memory:///no/fragment" => "memory:///no/fragment" ; "No fragment")]
    #[test_case("memory:///#" => "memory:///" ; "With empty fragment")]
    #[test_case("memory:///#/fragment" => "memory:///" ; "With non empty fragment")]
    fn test_remove_fragment_from_url(url: &str) -> String {
        remove_fragment_from_url(&Url::parse(url).unwrap()).as_str().to_string()
    }

    // The code will fail to compile if LoaderTrait cannot be made into an object
    // Adding `fn foo() {}` into the trait will result into
    // error[E0038]: the trait `LoaderTrait` cannot be made into an object
    //     associated function `foo` has no `self` parameter
    #[allow(dead_code)]
    fn loader_trait_can_be_made_into_an_object<T>(_: &dyn LoaderTrait<T>) {}
}
