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
#![feature(tool_lints, try_from)]
#![deny(
    clippy::all,
    clippy::pedantic,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
#![allow(clippy::stutter)]
#![doc(html_root_url = "https://docs.rs/libdeadmock/0.1.0")]

// Macro imports
#[macro_use]
extern crate failure;
#[macro_use]
extern crate getset;
#[macro_use]
extern crate serde_derive;

// Library Modules
mod config;
mod error;
mod util;

// Public API
pub use crate::config::Header as HeaderConfig;
pub use crate::config::Mapping as MappingConfig;
pub use crate::config::Mappings as MappingsConfig;
pub use crate::config::Proxy as ProxyConfig;
pub use crate::config::Request as RequestConfig;
pub use crate::config::Response as ResponseConfig;
pub use crate::config::Runtime as RuntimeConfig;
pub use crate::error::Error as DeadmockError;
