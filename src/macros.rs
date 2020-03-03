#[cfg(any(test))]
#[macro_export]
macro_rules! mock_loader_request {
    ($loader:ident, $status_code:expr, $content_type:expr, $file_path:expr,) => {{
        use crate::url_helpers::test_data_file_path;
        use mockito::{mock, server_url};

        let abs_file_path = test_data_file_path($file_path);
        let url_path = String::from(url::Url::parse(&test_data_file_url($file_path)).unwrap().path());
        let mocked_request = mock("GET", url_path.as_str())
            .with_status($status_code)
            .with_header("content-type", $content_type)
            .with_body_from_file(&abs_file_path)
            .create();
        let url = url::Url::parse(&server_url()).unwrap().join(url_path.as_str()).unwrap();

        let value = $loader.load(url.as_ref());
        mocked_request.expect(1).assert();

        value
    }};
    ($loader:ident, $status_code:expr, $content_type:expr, $file_path:expr) => {{
        mock_loader_request!($loader, $status_code, $content_type, $file_path,)
    }};
    ($loader:ident, $status_code:expr, $file_path:expr,) => {{
        mock_loader_request!($loader, $status_code, "application/octet-stream", $file_path,)
    }};
    ($loader:ident, $status_code:expr, $file_path:expr) => {{
        mock_loader_request!($loader, $status_code, $file_path,)
    }};
    ($loader:ident, $file_path:expr,) => {{
        mock_loader_request!($loader, 200, $file_path,)
    }};
    ($loader:ident, $file_path:expr) => {{
        mock_loader_request!($loader, $file_path,)
    }};
}

#[cfg(any(test, feature = "trait_json"))]
#[macro_export]
macro_rules! rust_json {
    ($($json:tt)+) => {{
        use serde_json;
        use json;
        let thing: json::JsonValue = json::parse(
            serde_json::to_string(&json![$($json)+]).unwrap().as_str(),
        ).unwrap();
        thing
    }};
}

#[cfg(any(test, feature = "trait_serde_yaml"))]
#[macro_export]
macro_rules! yaml {
    ($($json:tt)+) => {{
        use serde_json;
        use serde_yaml;
        let thing: serde_yaml::Value = serde_yaml::from_str(
            serde_json::to_string(&json![$($json)+]).unwrap().as_str(),
        ).unwrap();
        thing
    }};
}
