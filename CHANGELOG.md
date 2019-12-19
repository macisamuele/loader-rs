Changelog
=========

0.4.1 (2019-12-19)
------------------
- Internal fixes, no changes to lib code

0.4.0 (2019-09-30)
------------------
- `LoaderTrait` traits cleanup (remove not needed Sync, Send, Clone, Debug, etc.) - [PR #5](https://github.com/macisamuele/loader-rs/pull/5)
- Update `LoaderTrait` to support loading via bytes instead of string - [PR #6](https://github.com/macisamuele/loader-rs/pull/6)

0.3.0 (2019-06-02)
------------------
- Update `json-trait-rs` dependency and rename `TestingLoader` into `RustTypeLoader`

0.2.0 (2019-05-26)
------------------
- Update code-base to deal with `json-trait-rs` update - [PR #1](https://github.com/macisamuele/loader-rs/pull/1)

0.1.0 (2019-04-28)
------------------
- Initial project release
- Definition of generic `Loader` (and `LoaderTrait` trait)
- Implementation of trait for [`json::JsonValue`](https://github.com/maciejhirsz/json-rust/), [`serde_json::Value`](https://github.com/serde-rs/json/) and [`serde_yaml::Value`](https://github.com/dtolnay/serde-yaml).
