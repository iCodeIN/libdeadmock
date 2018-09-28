// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request single header matching
use crate::config;
use crate::error::Error;
use crate::matcher::{self, RequestMatch};
use http::Request;
use slog::Logger;
use std::fmt;

/// Exactly match all headers on a HTTP request.
#[derive(Clone, Debug, Default)]
pub struct ExactMatch {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
}

impl ExactMatch {
    /// Add a stdout logger
    pub fn set_stdout(mut self, stdout: Option<Logger>) -> Self {
        self.stdout = stdout;
        self
    }

    /// Add a stderr logger
    pub fn set_stderr(mut self, stderr: Option<Logger>) -> Self {
        self.stderr = stderr;
        self
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match Header")
    }
}

impl RequestMatch for ExactMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &config::Request,
    ) -> Result<Option<bool>, Error> {
        if let Some(header) = request_config.header() {
            if let Ok((ref expected_name, ref expected_value)) = matcher::to_header_tuple(header) {
                let expected = (expected_name, expected_value);
                let results: Vec<bool> = request
                    .headers()
                    .iter()
                    .map(|actual| matcher::equal_headers(actual, expected))
                    .collect();
                Ok(Some(results.len() == 1 && results[0]))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
