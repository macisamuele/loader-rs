#[cfg(feature = "regular_expression")]
use regex::Regex;
use std::cell::RefCell;
use url::{ParseError, SyntaxViolation, Url};

#[derive(Clone, Debug, Display, PartialEq)]
pub enum UrlError {
    ParseError(ParseError),
    SyntaxViolation(SyntaxViolation),
    JsonFragmentError(String),
}

impl From<ParseError> for UrlError {
    #[must_use]
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl From<SyntaxViolation> for UrlError {
    #[must_use]
    fn from(error: SyntaxViolation) -> Self {
        Self::SyntaxViolation(error)
    }
}

pub(in crate) fn parse_and_normalize_url(url: &str) -> Result<Url, UrlError> {
    let mut maybe_syntax_violation: RefCell<Option<SyntaxViolation>> = RefCell::new(None);
    let mut url = Url::options()
        .syntax_violation_callback(Some(&|syntax_violation| {
            maybe_syntax_violation.swap(&RefCell::new(Some(syntax_violation)));
        }))
        .parse(url)?;
    if let Some(ref violation) = maybe_syntax_violation.get_mut() {
        return Err(UrlError::from(*violation));
    }

    let fragments = url.fragment().unwrap_or("").trim_end_matches('/');
    let owned_fragments = if fragments.is_empty() {
        "/".to_string()
    } else if fragments.starts_with('/') {
        fragments.to_string()
    } else {
        format!("/{}", fragments)
    };
    url.set_fragment(Some(owned_fragments.as_str()));

    if url.path().is_empty() {
        url.set_path("/");
    }

    Ok(url)
}

pub(in crate) fn normalize_url_for_cache(url: &Url) -> Url {
    let mut clone_url = url.clone();
    clone_url.set_fragment(Some("/"));
    clone_url
}

#[cfg(test)]
mod tests {
    use super::{parse_and_normalize_url, UrlError};
    use test_case::test_case;
    use url::{ParseError, SyntaxViolation};

    #[test_case("memory://", "memory:///#/" ; "url_with_no_path_no_fragment")]
    #[test_case("memory://#", "memory:///#/" ; "url_with_no_path")]
    #[test_case("memory:///", "memory:///#/" ; "url_with_no_fragment")]
    #[test_case("memory:///#", "memory:///#/" ; "url_with_path_and_fragment")]
    #[test_case("memory:///#/", "memory:///#/" ; "url_with_path_and_fragment_normalized")]
    #[test_case("memory:///#fragment", "memory:///#/fragment" ; "url_with_path_and_not_empty_fragment_1")]
    #[test_case("memory:///#/fragment", "memory:///#/fragment" ; "url_with_path_and_not_empty_fragment_2")]
    #[test_case("memory:///#/fragment/", "memory:///#/fragment" ; "url_with_path_and_not_empty_fragment_3")]
    fn test_parse_and_normalize_url_valid_case(url_str: &str, expected_result_str: &str) {
        assert_eq!(parse_and_normalize_url(url_str).unwrap().as_str(), expected_result_str);
    }

    #[test_case("http:///", &UrlError::ParseError(ParseError::EmptyHost))]
    #[test_case("http://300.0.0.0/", &UrlError::ParseError(ParseError::InvalidIpv4Address))]
    #[test_case("memory://#/\0a", &UrlError::SyntaxViolation(SyntaxViolation::NullInFragment))]
    #[test_case("http:/example", &UrlError::SyntaxViolation(SyntaxViolation::ExpectedDoubleSlash))]
    fn test_parse_and_normalize_url_invalid_case(url_str: &str, expected_err: &UrlError) {
        assert_eq!(&parse_and_normalize_url(url_str).unwrap_err(), expected_err);
    }
}
