// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request headers matching
use crate::config;
use crate::error::Error;
use crate::matcher::{self, RequestMatch, Slogger};
use http::Request;
use slog::{trace, Logger};
use slog_try::try_trace;
use std::fmt;

/// Exactly match all headers on a HTTP request.
#[derive(Clone, Debug, Default)]
pub struct ExactMatch {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
}

impl ExactMatch {
    fn actual_has_match(&self, request: &Request<()>, header: &config::Header) -> Option<bool> {
        if let Ok((ref expected_name, ref expected_value)) = matcher::to_header_tuple(header) {
            let expected = (expected_name, expected_value);
            Some(
                request
                    .headers()
                    .iter()
                    .map(|actual| matcher::equal_headers(actual, expected))
                    .any(|x| x),
            )
        } else {
            None
        }
    }
}

impl Slogger for ExactMatch {
    /// Add a stdout logger
    fn set_stdout(mut self, stdout: Option<Logger>) -> Self {
        self.stdout = stdout;
        self
    }

    /// Add a stderr logger
    fn set_stderr(mut self, stderr: Option<Logger>) -> Self {
        self.stderr = stderr;
        self
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match Headers")
    }
}

impl RequestMatch for ExactMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &config::Request,
    ) -> Result<Option<bool>, Error> {
        if request_config.headers().is_empty() {
            try_trace!(self.stdout, "Exact Match (Headers) - No check performed");
            Ok(None)
        } else {
            try_trace!(self.stdout, "Exact Match (Headers) - Checking...");
            Ok(Some(
                request_config
                    .headers()
                    .iter()
                    .filter_map(|header| self.actual_has_match(request, header))
                    .all(|v| v),
            ))
        }
    }
}

/// Pattern match all headers on an HTTP request.
#[derive(Clone, Debug, Default)]
pub struct PatternMatch {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
}

impl Slogger for PatternMatch {
    /// Add a stdout logger
    fn set_stdout(mut self, stdout: Option<Logger>) -> Self {
        self.stdout = stdout;
        self
    }

    /// Add a stderr logger
    fn set_stderr(mut self, stderr: Option<Logger>) -> Self {
        self.stderr = stderr;
        self
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match Headers")
    }
}

impl RequestMatch for PatternMatch {
    fn is_match(
        &self,
        _request: &Request<()>,
        request_config: &config::Request,
    ) -> Result<Option<bool>, Error> {
        if request_config.headers().is_empty() {
            try_trace!(self.stdout, "Pattern Match (Headers) - No check performed");
            Ok(None)
        } else {
            try_trace!(self.stdout, "Pattern Match (Headers) - Not Implemented!!");
            Ok(None)
        }
    }
}
