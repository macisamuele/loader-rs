#![deny(
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
// This is not great, but the library is not stable enough to write documentation
#![allow(clippy::missing_errors_doc)]
// Ignore missing_const_for_fn clippy linter (it's too noisy in regards const fn in traits)
#![allow(clippy::missing_const_for_fn)]

#[macro_use]
extern crate strum_macros;
#[cfg(all(test, any(feature = "trait_json", feature = "trait_serde_json", feature = "trait_serde_yaml")))]
#[macro_use]
extern crate serde_json;

use crate::{
    private::LoaderInternal,
    url_helpers::{normalize_url_for_cache, parse_and_normalize_url, UrlError},
};
use std::{io, marker::PhantomData, time::Duration};
use url::Url;

#[cfg(test)]
#[macro_use]
mod macros;

mod arc_cache;
pub mod traits;
pub mod url_helpers;

use crate::arc_cache::ThreadSafeCache;
use std::{fs::read, io::Read, sync::Arc};
pub use traits::loaders;

#[derive(Debug, Display)]
pub enum LoaderError<FE> {
    IOError(io::Error),
    InvalidURL(UrlError),
    FetchURLFailed(reqwest::Error),
    FormatError(FE),
    UnknownError,
}

impl<FE> From<io::Error> for LoaderError<FE> {
    #[must_use]
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}

impl<FE> From<url::ParseError> for LoaderError<FE> {
    #[must_use]
    fn from(error: url::ParseError) -> Self {
        Self::InvalidURL(UrlError::ParseError(error))
    }
}

impl<FE> From<url::SyntaxViolation> for LoaderError<FE> {
    #[must_use]
    fn from(error: url::SyntaxViolation) -> Self {
        Self::InvalidURL(UrlError::SyntaxViolation(error))
    }
}

impl<FE> From<UrlError> for LoaderError<FE> {
    #[must_use]
    fn from(error: UrlError) -> Self {
        Self::InvalidURL(error)
    }
}

impl<FE> From<reqwest::Error> for LoaderError<FE> {
    #[must_use]
    fn from(error: reqwest::Error) -> Self {
        Self::FetchURLFailed(error)
    }
}

impl<FE> Default for LoaderError<FE> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::UnknownError
    }
}

// Prevent users from implementing the LoaderInternal trait. (Idea extrapolated from libcore/slice/mod.rs)
mod private {
    use crate::LoaderError;
    use std::sync::Arc;
    use url::Url;

    pub trait LoaderInternal<T, FE>
    where
        LoaderError<FE>: From<FE>,
    {
        fn set(&self, url: &str, value: T) -> Result<(), LoaderError<FE>>;
        fn internal_get_or_fetch_with_result<F>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>>
        where
            F: FnOnce(&Url) -> Result<T, LoaderError<FE>>;
    }
}

pub trait LoaderTrait<T, FE>: Default + LoaderInternal<T, FE>
where
    LoaderError<FE>: From<FE>,
{
    fn load_from_string(content: &str) -> Result<T, LoaderError<FE>>
    where
        Self: Sized,
    {
        Self::load_from_bytes(content.as_bytes())
    }

    fn load_from_bytes(content: &[u8]) -> Result<T, LoaderError<FE>>
    where
        Self: Sized;

    fn load<R: AsRef<str>>(&self, url: R) -> Result<Arc<T>, LoaderError<FE>> {
        self.load_with_timeout(url, Duration::from_millis(30_000))
    }

    fn load_with_timeout<R: AsRef<str>>(&self, url: R, timeout: Duration) -> Result<Arc<T>, LoaderError<FE>> {
        let url = parse_and_normalize_url(url)?;

        Ok(self.get_or_fetch_with_result(&normalize_url_for_cache(&url), |url_to_fetch| {
            // Value was not available on cache
            let bytes_content: Vec<u8> = if url_to_fetch.scheme() == "file" {
                read(url_to_fetch.to_file_path().unwrap())?
            } else {
                let client_builder = reqwest::blocking::Client::builder();
                let client = client_builder.gzip(true).timeout(timeout).build()?;
                let response = client.get(url_to_fetch.as_ref()).send()?.error_for_status()?;
                response.bytes().filter_map(Result::ok).collect::<_>()
            };
            Self::load_from_bytes(bytes_content.as_slice())
        })?)
    }

    // This method is needed to extract internal_get_or_fetch_with_result from the internal trait
    fn get_or_fetch_with_result<F: FnOnce(&Url) -> Result<T, LoaderError<FE>>>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>> {
        self.internal_get_or_fetch_with_result(key, fetcher)
    }
}

#[derive(Debug)]
pub struct Loader<T, FE>
where
    LoaderError<FE>: From<FE>,
{
    cache: ThreadSafeCache<Url, T>,
    format_error: PhantomData<FE>,
}

impl<T, FE> Default for Loader<T, FE>
where
    LoaderError<FE>: From<FE>,
{
    #[must_use]
    fn default() -> Self {
        Self {
            cache: ThreadSafeCache::default(),
            format_error: PhantomData,
        }
    }
}

impl<T, FE> LoaderInternal<T, FE> for Loader<T, FE>
where
    LoaderError<FE>: From<FE>,
{
    #[inline]
    fn set(&self, url: &str, value: T) -> Result<(), LoaderError<FE>> {
        self.cache.set(&normalize_url_for_cache(&parse_and_normalize_url(url)?), value);
        Ok(())
    }

    #[inline]
    fn internal_get_or_fetch_with_result<F: FnOnce(&Url) -> Result<T, LoaderError<FE>>>(&self, key: &Url, fetcher: F) -> Result<Arc<T>, LoaderError<FE>> {
        if let Some(value) = self.cache.get(key) {
            Ok(value)
        } else {
            match fetcher(key) {
                Ok(fetched_value) => {
                    let arc_fetched_value = Arc::new(fetched_value);
                    self.cache.set_arc(key, arc_fetched_value.clone());
                    Ok(arc_fetched_value)
                }
                Err(loader_error) => Err(loader_error),
            }
        }
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
