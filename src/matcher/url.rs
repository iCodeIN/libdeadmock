// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request URL matching
use crate::config::Request as RequestConfig;
use crate::error::Error;
use crate::matcher::RequestMatch;
use http::Request;
use slog::{b, kv, log, record, record_static, trace, Logger};
use slog_try::try_trace;
use std::fmt;

/// Exactly match a url
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
        request_config: &RequestConfig,
    ) -> Result<Option<bool>, Error> {
        if let Some(url) = request_config.url() {
            try_trace!(
                self.stdout,
                "Checking {} against {}",
                url,
                request.uri().path()
            );
            Ok(Some(request.uri().path() == &url[..]))
        } else {
            Ok(None)
        }
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match On Url")
    }
}
