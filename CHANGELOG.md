Changelog
=========

0.7.0 (2020-05-10)
------------------

- Provide dedicated Json loader (`ConcreteJsonLoader`) to allow the loader to deal with Json fragments - [PR #26](https://github.com/macisamuele/loader-rs/pull/26)
- Export testing helpers, via `testing-helpers`, to provide a structure to easily mock requests for the `LoaderTrait` interactions - [PR #27](https://github.com/macisamuele/loader-rs/pull/27) and [PR #33](https://github.com/macisamuele/loader-rs/pull/33)
- Improve `RustTypeLoader` implementaton in order to actually parse JSON. - [PR #35](https://github.com/macisamuele/loader-rs/pull/35)

  WARNING: `RustTypeLoader` should be used only for testing as the parsing depends on `serde_json` which might be a dependency that you'd like not to have and is not accurate around `i128` integers.
- Fix loader cache usage - [PR #36](https://github.com/macisamuele/loader-rs/pull/36)

0.6.0 (2020-04-18)
------------------

- Internal cache update, use `cached` instead of self-implemented solution - [PR #17](https://github.com/macisamuele/loader-rs/pull/17)
- Remove FormatError generic type from `LoaderTrait`.
- Miscellaneous updates

0.5.0 (2020-01-13)
------------------

- Support reqwest 0.10+ - [PR #14](https://github.com/macisamuele/loader-rs/pull/14)
- Add `must_use` annotation to trait methods (silence lints) - [PR #12](https://github.com/macisamuele/loader-rs/pull/12)

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
