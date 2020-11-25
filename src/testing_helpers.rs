#[cfg(test)]
use crate::{LoaderError, LoaderTrait};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;
use url::Url;

pub(in crate) fn test_data_file_path(path_components: &[&str]) -> Result<PathBuf, std::io::Error> {
    let repository_path = Path::new(file!()).canonicalize().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    Ok(path_components
        .iter()
        .map(ToString::to_string)
        .fold(repository_path.join("test-data"), |iter_path, path_path| iter_path.join(path_path))
        .canonicalize()?)
}

fn validate_builder(value: &MockLoaderRequestBuilder) -> Result<(), String> {
    match (value.resp_body.as_ref(), value.resp_body_file_path.as_ref()) {
        (Some(Some(_)), Some(Some(_))) => Err("Only one between resp_body and resp_body_file_path should be defined".to_string()),
        (None, Some(Some(resp_body_file_path))) => match test_data_file_path(resp_body_file_path) {
            Err(io_error) => Err(io_error.to_string()),
            Ok(absolute_path) if !absolute_path.is_file() => Err(format!("absolute_path={} is not a file", absolute_path.to_str().unwrap())),
            Ok(_) => Ok(()),
        },
        _ => Ok(()),
    }
}

#[derive(Debug, Builder)]
#[builder(build_fn(validate = "validate_builder"))]
#[builder(setter(strip_option))]
pub struct MockLoaderRequest {
    #[builder(default = "true")]
    ensure_mock_calls: bool,
    #[builder(default = "1")]
    expected_mock_calls: usize,
    #[builder(default = "\"/\".to_string()")]
    #[builder(setter(into))]
    http_path: String,
    #[builder(default = "\"GET\".to_string()")]
    #[builder(setter(into))]
    http_verb: String,
    #[builder(default = "None")]
    #[builder(setter(into))]
    resp_content_type: Option<String>,
    #[builder(default = "200")]
    resp_status_code: usize,
    #[builder(default = "None")]
    #[builder(setter(into))]
    resp_body_file_path: Option<Vec<&'static str>>,
    #[builder(default = "None")]
    #[builder(setter(into))]
    resp_body: Option<String>,
}

impl MockLoaderRequest {
    #[allow(clippy::inefficient_to_string)]
    fn build_mock_request(&self) -> mockito::Mock {
        let mut mocked_request_builder = mockito::mock(
            &self.http_verb,
            // Remove fragment from http path
            self.http_path.split('#').collect::<Vec<_>>().first().unwrap().to_string().as_str(),
        )
        .with_status(self.resp_status_code);

        if let Some(content_type) = &self.resp_content_type {
            mocked_request_builder = mocked_request_builder.with_header("content-type", content_type);
        }

        if let Some(resp_body) = &self.resp_body {
            mocked_request_builder = mocked_request_builder.with_body(&resp_body);
        } else if let Some(resp_file_path) = self.resp_body_file_path.as_ref() {
            mocked_request_builder = mocked_request_builder.with_body_from_file(test_data_file_path(resp_file_path).unwrap());
        }

        mocked_request_builder.create()
    }

    pub fn run_in_mock_context<R>(&self, callback: &dyn Fn(&Url) -> R) -> R {
        let mocked_request = self.build_mock_request();

        let url = Url::parse(&mockito::server_url()).and_then(|url| url.join(&self.http_path)).unwrap();

        let value = callback(&url);

        if self.ensure_mock_calls {
            mocked_request.expect(self.expected_mock_calls).assert();
        }
        value
    }

    #[cfg(test)]
    pub(in crate) fn send_request<T, L: LoaderTrait<T>>(&self, loader: &L) -> Result<Arc<T>, LoaderError> {
        self.run_in_mock_context(&|url| loader.get_or_fetch_with_result(url))
    }
}

#[cfg(test)]
mod tests {
    use super::MockLoaderRequestBuilder;
    use crate::loader::{testing::TestStringLoader, trait_::LoaderTrait};
    use std::sync::Arc;
    use test_case::test_case;

    #[test]
    fn test_mock_loader_request_resp_body() {
        assert_eq!(
            MockLoaderRequestBuilder::default()
                .resp_body("Content")
                .build()
                .unwrap()
                .send_request(&TestStringLoader::default())
                .unwrap(),
            Arc::new("Content".to_string())
        );
    }

    #[test]
    fn test_mock_loader_request_resp_body_file_path_not_existing() {
        assert_eq!(
            MockLoaderRequestBuilder::default().resp_body_file_path(vec!["not-existing"]).build().unwrap_err(),
            std::io::Error::from_raw_os_error(2).to_string(),
        );
    }

    #[test]
    fn test_mock_loader_request_resp_body_file_path_existing() {
        assert_eq!(
            MockLoaderRequestBuilder::default()
                .resp_body_file_path(vec!["empty"])
                .build()
                .unwrap()
                .send_request(&TestStringLoader::default())
                .unwrap(),
            Arc::new("".to_string()),
        );
    }

    #[test_case(true => panics "Expected 1 request")]
    #[test_case(false)]
    fn test_mock_loader_honors_ensure_mock_calls(ensure_mock_calls: bool) {
        MockLoaderRequestBuilder::default()
            .ensure_mock_calls(ensure_mock_calls)
            .resp_body("")
            .build()
            .unwrap()
            .run_in_mock_context(&|_url| {
                // Not using the _url as we want to ensure that no requests are issued to the mock
            });
    }

    #[test_case(0 => panics "Expected 0 request")]
    #[test_case(1)]
    #[test_case(2 => panics "Expected 2 request")]
    fn test_mock_loader_honors_expected_mock_calls(expected_mock_calls: usize) {
        MockLoaderRequestBuilder::default()
            .expected_mock_calls(expected_mock_calls)
            .resp_body("")
            .build()
            .unwrap()
            .run_in_mock_context(&|url| {
                let _d = TestStringLoader::default().load(url.as_str());
            });
    }

    #[test]
    fn test_mock_loader_ensure_test_not_called() {
        MockLoaderRequestBuilder::default()
            .expected_mock_calls(0)
            .resp_body("")
            .build()
            .unwrap()
            .run_in_mock_context(&|_url| {
                // Not using the _url as we want to ensure that no requests are issued to the mock
            });
    }
}
