// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request single header matching
use cached::{cached_key_result, UnboundCache};
use crate::config::{self, Request as RequestConfig};
use crate::error::Error;
use crate::matcher::{self, RequestMatch};
use http::Request;
use lazy_static::lazy_static;
use regex::Regex;
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
            try_trace!(self.stdout, "Checking header: '{}'", header);
            if let Ok((ref expected_name, ref expected_value)) = matcher::to_header_tuple(header) {
                let expected = (expected_name, expected_value);
                let results: Vec<bool> = request
                    .headers()
                    .iter()
                    .map(|actual| matcher::equal_headers(actual, expected))
                    .filter(|v| *v)
                    .collect();
                try_trace!(self.stdout, "Found {} header matches", results.len());
                Ok(Some(results.len() == 1 && results[0]))
            } else {
                try_trace!(
                    self.stdout,
                    "Unable to convert header config to http::Header"
                );
                Ok(None)
            }
        } else {
            try_trace!(self.stdout, "Exact header match not configured!");
            Ok(None)
        }
    }
}

/// Pattern match a header
#[derive(Clone, Debug, Default)]
pub struct PatternMatch {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
}

impl PatternMatch {
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

cached_key_result!{
    REGEX: UnboundCache<String, Regex> = UnboundCache::new();
    Key = { actual.to_string() };
    fn generate_regex(actual: &str, header_pattern: &str) -> Result<Regex, String> = {
        let regex_result = Regex::new(header_pattern);

        match regex_result {
            Ok(regex) => Ok(regex),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl RequestMatch for PatternMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &RequestConfig,
    ) -> Result<Option<bool>, Error> {
        if let Some(header) = request_config.header() {
            try_trace!(self.stdout, "Checking header: '{}'", header);
            let _headers_str: Vec<(&str, &str)> = request
                .headers()
                .iter()
                .map(|(key, value)| (key.as_str(), value.to_str()))
                .filter_map(|(key, result)| match result {
                    Ok(value) => Some((key, value)),
                    Err(_) => None,
                })
                .collect();
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pattern Match On Header")
    }
}
