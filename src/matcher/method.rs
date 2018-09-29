// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request method matching
use crate::config;
use crate::error::Error;
use crate::matcher::RequestMatch;
use http::Request;
use slog::{trace, Logger};
use slog_try::try_trace;
use std::fmt;

/// Exactly match an HTTP method.
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

impl RequestMatch for ExactMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &config::Request,
    ) -> Result<Option<bool>, Error> {
        if let Some(method) = request_config.method() {
            try_trace!(
                self.stdout,
                "Checking {} against {}",
                method,
                request.method().as_str()
            );
            Ok(Some(request.method().as_str() == &method[..]))
        } else {
            Ok(None)
        }
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match On Method")
    }
}
