// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` 0.1.0
//!
//! Configuration for the deadmock server.
#![feature(crate_visibility_modifier, tool_lints, try_from)]
#![deny(
    clippy::all,
    clippy::pedantic,
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused
)]
#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bare_trait_objects,
    box_pointers,
    elided_lifetimes_in_paths,
    ellipsis_inclusive_range_patterns,
    keyword_idents,
    question_mark_macro_sep,
    single_use_lifetimes,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![allow(clippy::stutter)]
#![doc(html_root_url = "https://docs.rs/libdeadmock/0.1.0")]

// Library Modules
mod config;
mod error;
mod logs;
mod util;

// Public API
pub use crate::config::header::Header as HeaderConfig;
pub use crate::config::mapping::Mapping as MappingConfig;
pub use crate::config::mappings::Mappings as MappingsConfig;
pub use crate::config::proxy::Proxy as ProxyConfig;
pub use crate::config::request::Request as RequestConfig;
pub use crate::config::response::Response as ResponseConfig;
pub use crate::config::runtime::Runtime as RuntimeConfig;
pub use crate::error::Error as DeadmockError;
pub use crate::logs::Loggers;
