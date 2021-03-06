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
    unused_imports,
    unused_macros,
    unused_parens,
    unused_qualifications,
    unused_results,
    unused_unsafe,
    unused_variables,
    warnings,
)]
// Enable very pendantic clippy linting
#![deny(clippy::pedantic, clippy::nursery)]
// This is not great, but the library is not stable enough to write documentation
#![allow(clippy::missing_errors_doc)]
// Specialization needed in order to accommodate partial LoaderTrait implementation for ConcreteJsonLoader
#![feature(min_specialization)]

#[cfg(any(test, feature = "testing-helpers"))]
#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate strum_macros;
#[cfg(all(test, any(feature = "trait_json", feature = "trait_serde_json", feature = "trait_serde_yaml")))]
#[macro_use]
extern crate serde_json;

#[cfg(any(test, feature = "testing-helpers"))]
pub mod testing_helpers;

#[cfg(feature = "json-loader")]
pub mod json;
pub mod loader;
mod thread_safe_cache;
pub mod traits;
pub mod url_helpers;

#[cfg(feature = "json-loader")]
pub use crate::json::ConcreteJsonLoader;
pub use crate::{
    loader::{error::LoaderError, trait_::LoaderTrait, Loader},
    traits::loaders,
};
