use std::fmt::Display;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Display)]
pub enum LoaderError {
    IOError(std::io::Error),
    InvalidURL(crate::url_helpers::UrlError),
    FetchURLFailed(reqwest::Error),
    // We're not saving the real error instance, but only it's Display representation
    // in order to simplify the interface of the LoaderTrait trait
    FormatError(String),
    UnknownError,
}

impl<T: Display> From<&T> for LoaderError {
    #[must_use]
    fn from(error: &T) -> Self {
        Self::FormatError(format!("{}", error))
    }
}

impl From<std::io::Error> for LoaderError {
    #[must_use]
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<url::ParseError> for LoaderError {
    #[must_use]
    fn from(error: url::ParseError) -> Self {
        Self::InvalidURL(crate::url_helpers::UrlError::ParseError(error))
    }
}

impl From<url::SyntaxViolation> for LoaderError {
    #[must_use]
    fn from(error: url::SyntaxViolation) -> Self {
        Self::InvalidURL(crate::url_helpers::UrlError::SyntaxViolation(error))
    }
}

impl From<crate::url_helpers::UrlError> for LoaderError {
    #[must_use]
    fn from(error: crate::url_helpers::UrlError) -> Self {
        Self::InvalidURL(error)
    }
}

impl From<reqwest::Error> for LoaderError {
    #[must_use]
    fn from(error: reqwest::Error) -> Self {
        Self::FetchURLFailed(error)
    }
}

impl Default for LoaderError {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::UnknownError
    }
}

#[cfg(test)]
mod tests {
    use super::LoaderError;

    #[test]
    fn test_default_loader_error() {
        let loader_error_enum = LoaderError::default();
        if let LoaderError::UnknownError = loader_error_enum {
        } else {
            panic!("Expected LoaderError::UnknownError, received {:?}", loader_error_enum);
        }
    }
}
