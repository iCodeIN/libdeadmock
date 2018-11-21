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
use crate::matcher::{RequestMatch, Slogger};
use cached::{cached_key_result, UnboundCache};
use http::Request;
use regex::Regex;
use slog::{trace, Logger};
use slog_try::try_trace;
use std::fmt;

/// Exactly match a url
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

impl RequestMatch for ExactMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &RequestConfig,
    ) -> Result<Option<bool>, Error> {
        if let Some(url) = request_config.url() {
            try_trace!(
                self.stdout,
                "Exact Match (URL) - Checking {} against {}",
                url,
                request.uri().path()
            );
            Ok(Some(request.uri().path() == &url[..]))
        } else {
            try_trace!(self.stdout, "Exact Match (URL) - No check performed");
            Ok(None)
        }
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match On Url")
    }
}

/// Pattern match a url
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

cached_key_result! {
    REGEX: UnboundCache<String, Regex> = UnboundCache::new();
    Key = { path.to_string() };
    fn generate_regex(path: &str, url_pattern: &str) -> Result<Regex, String> = {
        let regex_result = Regex::new(url_pattern);

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
        if let Some(url_pattern) = request_config.url_pattern() {
            let path = request.uri().path();
            try_trace!(
                self.stdout,
                "Pattern Match (URL) - Checking {} against {}",
                url_pattern,
                path
            );
            if let Ok(regex) = generate_regex(path, url_pattern) {
                Ok(Some(regex.is_match(path)))
            } else {
                Ok(Some(false))
            }
        } else {
            try_trace!(self.stdout, "Patten Match (URL) - No check performed");
            Ok(None)
        }
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pattern Match On Url")
    }
}
