// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request single header matching
use crate::config::{self, HeaderPattern, Request as RequestConfig};
use crate::error::Error;
use crate::matcher::{self, RequestMatch, Slogger};
use cached::{cached_key_result, UnboundCache};
use http::Request;
use libeither::Either;
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
            try_trace!(
                self.stdout,
                "Exact Match (Header) - Checking header: '{}'",
                header
            );
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
                Ok(Some(false))
            }
        } else {
            try_trace!(self.stdout, "Exact Match (Header) - No check performed");
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
    fn is_match_either(
        &self,
        actual: &str,
        either: &Either<String, String>,
        case_insensitive: bool,
    ) -> bool {
        if let Ok(expected) = either.left_ref() {
            if case_insensitive {
                actual == expected.to_lowercase()
            } else {
                actual == expected
            }
        } else if let Ok(expected) = either.right_ref() {
            try_trace!(self.stdout, "Checking {} against {}", actual, expected);
            if let Ok(regex) = generate_regex(expected) {
                try_trace!(self.stdout, "Regex: {:?}", regex);
                regex.is_match(actual)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_header_match(&self, actual: &(&str, &str), expected: &HeaderPattern) -> Option<bool> {
        Some(
            self.is_match_either(actual.0, expected.key(), true)
                && self.is_match_either(actual.1, expected.value(), false),
        )
    }
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

cached_key_result! {
    REGEX: UnboundCache<String, Regex> = UnboundCache::new();
    Key = { header_pattern.to_string() };
    fn generate_regex(header_pattern: &str) -> Result<Regex, String> = {
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
        if let Some(header_pattern) = request_config.header_pattern() {
            try_trace!(
                self.stdout,
                "Pattern Match (Header) - Checking header pattern: '{}'",
                header_pattern
            );
            let matched_header: Vec<bool> = request
                .headers()
                .iter()
                .map(|(key, value)| (key.as_str(), value.to_str()))
                .filter_map(|(key, result)| match result {
                    Ok(value) => Some((key, value)),
                    Err(_) => None,
                })
                .filter_map(|actual_header| self.is_header_match(&actual_header, header_pattern))
                .filter(|x| *x)
                .collect();

            if matched_header.len() == 1 && matched_header[0] {
                try_trace!(
                    self.stdout,
                    "Matched Header: {} - {}",
                    matched_header.len(),
                    matched_header[0]
                );
                Ok(Some(true))
            } else {
                try_trace!(self.stdout, "Matched Header: {}", matched_header.len());
                Ok(Some(false))
            }
        } else {
            try_trace!(self.stdout, "Pattern Match (Header) - No check performed");
            Ok(None)
        }
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pattern Match On Header")
    }
}
