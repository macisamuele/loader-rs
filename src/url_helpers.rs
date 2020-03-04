#[cfg(feature = "regular_expression")]
use regex::Regex;
use std::cell::RefCell;
use url::{ParseError, SyntaxViolation, Url};

#[cfg(test)]
use std::path::Path;

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

#[cfg(feature = "regular_expression")]
fn get_invalid_fragment_part_according_to_json_pointer_rules(url: &Url) -> Option<String> {
    // Checks https://tools.ietf.org/html/rfc6901 rules
    let regex = Regex::new("#.*(~([^01]|$))").unwrap();
    regex.captures(url.as_str()).map(|captures| String::from(&captures[1]))
}

#[cfg(not(feature = "regular_expression"))]
fn get_invalid_fragment_part_according_to_json_pointer_rules(url: &Url) -> Option<String> {
    // Checks https://tools.ietf.org/html/rfc6901 rules
    // This check could have been done with a Regex but this requires regex dependency which adds ~20KB in the
    // final built library, which looks a bit too much as we are not using regex for anything else.
    //
    // As we're not sure about the real impact of having regex as dependency of this library
    // we're implementing a non-regex based solution, the code is a bit more verbose and so it's possible that it will
    // be selected a single solution.
    let fragment = url.fragment().unwrap_or("/");
    let mut next_character_error: Option<String> = None;

    let _ = fragment.chars().collect::<Vec<char>>().windows(2).any(|window| {
        let current = window[0];
        let next = window[1];

        if current == '~' && next != '0' && next != '1' {
            next_character_error = Some(format!("{}{}", current, next));
            return true;
        }
        false
    });

    if next_character_error.is_some() {
        next_character_error
    } else if fragment.ends_with('~') {
        Some("~".to_string())
    } else {
        None
    }
}

pub(crate) fn parse_and_normalize_url<R: AsRef<str>>(url: R) -> Result<Url, UrlError> {
    let syntax_violations = RefCell::new(Vec::<SyntaxViolation>::new());
    let mut url = Url::options()
        .syntax_violation_callback(Some(&|v| syntax_violations.borrow_mut().push(v)))
        .parse(url.as_ref())?;
    if let Some(violation) = syntax_violations.borrow().first() {
        return Err(Into::into(*violation));
    }

    let cloned_url = url.clone();
    let fragments = cloned_url.fragment().unwrap_or("").trim_end_matches('/');

    let owned_fragment = if fragments.is_empty() {
        "/".to_string()
    } else if fragments.starts_with('/') {
        fragments.to_string()
    } else {
        format!("/{}", fragments)
    };

    url.set_fragment(Some(owned_fragment.as_str()));

    if let Some(invalid_fragment) = get_invalid_fragment_part_according_to_json_pointer_rules(&url) {
        return Err(UrlError::JsonFragmentError(invalid_fragment));
    }

    Ok(url)
}

pub(crate) fn normalize_url_for_cache(url: &Url) -> Url {
    let mut clone_url = url.clone();
    clone_url.set_fragment(Some("/"));
    clone_url
}

#[cfg(test)]
pub(crate) fn test_data_file_path(path: &str) -> String {
    let repository_path = Path::new(file!()).canonicalize().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    String::from(
        path.split('/')
            .collect::<Vec<_>>()
            .iter()
            .fold(repository_path.join("test-data"), |iter_path, &path_path| iter_path.join(path_path))
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    )
}

#[cfg(test)]
pub(crate) fn test_data_file_url(path: &str) -> String {
    Url::from_file_path(test_data_file_path(path)).unwrap().to_string()
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
    #[test_case("memory://#/~", &UrlError::JsonFragmentError(String::from("~")))]
    #[test_case("memory://#/~a", &UrlError::JsonFragmentError(String::from("~a")))]
    #[test_case("memory://#/~0/~1/~c", &UrlError::JsonFragmentError(String::from("~c")))]
    fn test_parse_and_normalize_url_invalid_case(url_str: &str, expected_err: &UrlError) {
        assert_eq!(&parse_and_normalize_url(url_str).unwrap_err(), expected_err);
    }
}
