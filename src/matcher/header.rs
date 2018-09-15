// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request header matching
use crate::config;
use crate::error::Error;
use crate::matcher::RequestMatch;
use http::header::{HeaderName, HeaderValue};
use http::Request;
use slog::{b, kv, log, record, record_static, trace, Logger};
use slog_try::try_trace;
use std::fmt;

/// Exactly match all headers on a HTTP request.
#[derive(Clone, Debug, Default)]
pub struct ExactMatch {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
}

type HeaderTuple = (HeaderName, HeaderValue);
type HeaderTupleRef<'a> = (&'a HeaderName, &'a HeaderValue);

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

    fn to_header_tuple(&self, header: &config::Header) -> Result<HeaderTuple, failure::Error> {
        Ok((
            HeaderName::from_bytes(header.key().as_bytes())?,
            HeaderValue::from_bytes(header.value().as_bytes())?,
        ))
    }

    fn equal_headers(&self, actual: HeaderTupleRef<'_>, expected: HeaderTupleRef<'_>) -> bool {
        try_trace!(self.stdout, "Checking {:?} against {:?}", expected, actual);
        actual == expected
    }

    fn actual_has_match(&self, request: &Request<()>, header: &config::Header) -> Option<bool> {
        if let Ok((ref expected_name, ref expected_value)) = self.to_header_tuple(header) {
            let expected = (expected_name, expected_value);
            Some(
                request
                    .headers()
                    .iter()
                    .map(|actual| self.equal_headers(actual, expected))
                    .any(|x| x),
            )
        } else {
            None
        }
    }
}

impl fmt::Display for ExactMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Exact Match All Headers")
    }
}

impl RequestMatch for ExactMatch {
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &config::Request,
    ) -> Result<Option<bool>, Error> {
        if let Some(headers) = request_config.headers() {
            Ok(Some(
                headers
                    .iter()
                    .filter_map(|header| self.actual_has_match(request, header))
                    .all(|v| v),
            ))
        } else {
            Ok(None)
        }
    }
}
