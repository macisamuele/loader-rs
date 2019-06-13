#![allow(
    unreachable_pub,
    anonymous_parameters,
    bad_style,
    const_err,
    dead_code,
    deprecated,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    late_bound_lifetime_arguments,
    missing_copy_implementations,
    missing_debug_implementations,
    // missing_docs,
    non_shorthand_field_patterns,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unreachable_code,
    unreachable_patterns,
    unsafe_code,
    unused_allocation,
    unused_assignments,
    unused_comparisons,
    unused_doc_comments,
    unused_extern_crates,
    unused_extern_crates,
    unused_import_braces,
    unused_imports,
    unused_macros,
    unused_parens,
    unused_qualifications,
    unused_results,
    unused_unsafe,
    unused_variables,
    warnings,
)]
// Ignore missing_const_for_fn clippy linter (it's too noisy in regards const fn in traits)
#![allow(clippy::missing_const_for_fn)]

#[macro_use]
extern crate strum_macros;
#[cfg(test)]
#[macro_use]
extern crate serde_json;

use crate::{
    cache::{Cache, Cached},
    private::LoaderInternal,
    url_helpers::{normalize_url_for_cache, parse_and_normalize_url, UrlError},
};
use failure::Fail;
use std::{fmt::Debug, fs::read_to_string, io, marker::PhantomData, ops::Deref, sync::Arc, time::Duration};
use url::Url;

#[cfg(test)]
#[macro_use]
mod macros;

pub mod cache;
pub mod traits;
pub mod url_helpers;

pub use traits::loaders;

#[derive(Debug, Display, Fail)]
pub enum LoaderError<FE>
where
    FE: 'static + Debug + Sync + Send,
{
    IOError(io::Error),
    InvalidURL(UrlError),
    FetchURLFailed(reqwest::Error),
    FormatError(FE),
    UnknownError,
}

impl<FE> From<io::Error> for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    fn from(error: io::Error) -> Self {
        LoaderError::IOError(error)
    }
}

impl<FE> From<url::ParseError> for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    fn from(error: url::ParseError) -> Self {
        LoaderError::InvalidURL(UrlError::ParseError(error))
    }
}

impl<FE> From<url::SyntaxViolation> for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    fn from(error: url::SyntaxViolation) -> Self {
        LoaderError::InvalidURL(UrlError::SyntaxViolation(error))
    }
}

impl<FE> From<UrlError> for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    fn from(error: UrlError) -> Self {
        LoaderError::InvalidURL(error)
    }
}

impl<FE> From<reqwest::Error> for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    fn from(error: reqwest::Error) -> Self {
        LoaderError::FetchURLFailed(error)
    }
}

impl<FE> Default for LoaderError<FE>
where
    FE: Debug + Sync + Send,
{
    #[inline]
    fn default() -> Self {
        LoaderError::UnknownError
    }
}

// Prevent users from implementing the LoaderInternal trait. (Idea extrapolated from libcore/slice/mod.rs)
mod private {
    use crate::LoaderError;
    use std::{fmt::Debug, sync::Arc};
    use url::Url;

    pub trait LoaderInternal<T, FE>
    where
        T: Debug,
        FE: 'static + Debug + Sync + Send,
        LoaderError<FE>: From<FE>,
    {
        fn set<R: AsRef<str>>(&self, url: R, value: T) -> Result<(), LoaderError<FE>>;
        fn internal_get_or_fetch_with_result<F: FnOnce(&Url) -> Result<T, LoaderError<FE>>>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>>;
    }
}

pub trait LoaderTrait<T, FE>: Default + Sync + Send + LoaderInternal<T, FE>
where
    T: Clone + Debug,
    FE: 'static + Debug + Sync + Send,
    LoaderError<FE>: From<FE>,
{
    fn load_from_string(content: String) -> Result<T, LoaderError<FE>>
    where
        Self: Sized;

    fn load<R: AsRef<str>>(&self, url: R) -> Result<T, LoaderError<FE>> {
        self.load_with_timeout(url, Duration::from_millis(30_000))
    }

    #[allow(unused_variables)]
    fn extract_fragment(value: Arc<T>, url: &Url) -> T
    where
        Self: Sized,
    {
        value.deref().clone()
    }

    fn load_with_timeout<R: AsRef<str>>(&self, url: R, timeout: Duration) -> Result<T, LoaderError<FE>> {
        let url = parse_and_normalize_url(url)?;

        let normalized_url = normalize_url_for_cache(&url);

        let cached_value = {
            let thing: Result<Arc<T>, LoaderError<FE>> = self.get_or_fetch_with_result(&normalized_url, |url_to_fetch| {
                // Value was not available on cache
                Ok(Self::load_from_string(if url_to_fetch.scheme() == "file" {
                    read_to_string(url_to_fetch.to_file_path().unwrap())?
                } else {
                    let client_builder = reqwest::Client::builder();
                    let client = client_builder.gzip(true).timeout(timeout).build()?;
                    client.get(url_to_fetch.as_ref()).send()?.error_for_status()?.text()?
                })?)
            });
            thing?
        };
        Ok(Self::extract_fragment(cached_value, &url))
    }

    // This method is needed to extract internal_get_or_fetch_with_result from the internal trait
    #[inline(always)]
    fn get_or_fetch_with_result<F: FnOnce(&Url) -> Result<T, LoaderError<FE>>>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>> {
        self.internal_get_or_fetch_with_result(key, fetcher)
    }
}

#[derive(Debug)]
pub struct Loader<T, FE>
where
    T: Clone + Debug,
    FE: 'static + Debug + Sync + Send,
    LoaderError<FE>: From<FE>,
{
    cache: Cache<Url, T>,
    format_error: PhantomData<FE>,
}

impl<T, FE> Default for Loader<T, FE>
where
    T: Clone + Debug,
    FE: 'static + Debug + Sync + Send,
    LoaderError<FE>: From<FE>,
{
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            format_error: PhantomData,
        }
    }
}

impl<T, FE> LoaderInternal<T, FE> for Loader<T, FE>
where
    T: Clone + Debug,
    FE: 'static + Debug + Sync + Send,
    LoaderError<FE>: From<FE>,
{
    #[inline]
    fn set<R: AsRef<str>>(&self, url: R, value: T) -> Result<(), LoaderError<FE>> {
        self.cache.set(normalize_url_for_cache(&parse_and_normalize_url(url)?), value);
        Ok(())
    }

    #[inline]
    fn internal_get_or_fetch_with_result<F: FnOnce(&Url) -> Result<T, LoaderError<FE>>>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>> {
        self.cache.get_or_fetch_with_result(key, fetcher)
    }
}

#[cfg(test)]
mod tests {
    use crate::LoaderError;

    #[test]
    fn test_default_loader_error() {
        let loader_error_enum = LoaderError::<()>::default();
        if let LoaderError::UnknownError = loader_error_enum {
        } else {
            panic!("Expected LoaderError::UnknownError, received {:?}", loader_error_enum);
        }
    }
}
