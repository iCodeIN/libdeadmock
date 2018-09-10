// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` runtime environment configuration
use std::env;

const DM_ENV: &str = "dmenv";
const LOCAL_ENV: &str = "local";

/// The runtime environment configuration for deadmock.
#[derive(Clone, Copy, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Runtime<'a> {
    /// The IP address to listen on.
    #[get = "pub"]
    ip: Option<&'a str>,
    /// The port to listen on.
    #[get = "pub"]
    port: Option<u32>,
    /// The path to the mappings and templates
    #[get = "pub"]
    path: Option<&'a str>,
}

impl<'a> Runtime<'a> {
    /// Get the `dmenv` environment variable, setting it to `local` if the variable is not found or set already.
    ///
    /// # Example
    ///
    /// ```
    /// # use libdeadmock::RuntimeConfig;
    /// #
    /// # fn main() {
    /// assert_eq!("local", RuntimeConfig::dmenv());
    /// # }
    /// ```
    pub fn dmenv() -> String {
        env::var(DM_ENV).unwrap_or_else(|_| {
            env::set_var(DM_ENV, LOCAL_ENV);
            LOCAL_ENV.to_string()
        })
    }
}
