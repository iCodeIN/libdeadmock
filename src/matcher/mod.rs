// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request matching for the server.
use crate::config::{Mapping, Mappings, Request as RequestConfig};
use crate::error::Error;
use crate::error::ErrorKind::MappingNotFound;
use http::Request;
use slog::{b, kv, log, record, record_static, trace, Logger};
use slog_try::try_trace;
use std::fmt;

crate mod header;
crate mod method;
crate mod url;

pub use self::header::ExactMatchAllHeaders;
pub use self::method::ExactMatchMethod;
pub use self::url::ExactMatchUrl;

/// A request matcher
pub trait RequestMatch: fmt::Debug + fmt::Display {
    /// Does the incoming request match the request configuration from a mapping.
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &RequestConfig,
    ) -> Result<Option<bool>, Error>;
}

/// Try to match an incoming request to a mapping.
#[derive(Default)]
#[allow(box_pointers)]
pub struct Matcher {
    /// The matchers setup for request matching.
    matchers: Vec<Box<dyn RequestMatch>>,
    /// stdout slog logger
    stdout: Option<Logger>,
    /// stderr slog logger
    stderr: Option<Logger>,
}

#[allow(box_pointers)]
impl fmt::Debug for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.matchers
            .iter()
            .map(|matcher| write!(f, "{:?},", matcher))
            .collect()
    }
}

#[allow(box_pointers)]
impl Matcher {
    /// Add a stdout logger
    pub fn set_stdout(&mut self, stdout: Option<Logger>) -> &mut Self {
        self.stdout = stdout;
        self
    }

    /// Add a stderr logger
    pub fn set_stderr(&mut self, stderr: Option<Logger>) -> &mut Self {
        self.stderr = stderr;
        self
    }

    /// Add a request matcher to the list.
    pub fn push<T: RequestMatch + 'static>(&mut self, request_match: T) -> &mut Self {
        self.matchers.push(Box::new(request_match));
        self
    }

    /// Get a mapping that matches the given request.
    pub fn get_match(&self, request: &Request<()>, mappings: &Mappings) -> Result<Mapping, Error> {
        mappings
            .inner()
            .iter()
            .inspect(|(_uuid, mapping)| try_trace!(self.stdout, "Checking mapping '{}'", mapping))
            .filter_map(|(_uuid, mapping)| self.is_match(request, mapping))
            .min()
            .ok_or_else(|| MappingNotFound.into())
    }

    fn is_match(&self, request: &Request<()>, mapping: &Mapping) -> Option<Mapping> {
        let matches = self
            .matchers
            .iter()
            .inspect(|x| try_trace!(self.stdout, "Checking for '{}'", x))
            .map(|matcher| matcher.is_match(request, mapping.request()))
            // Filter out the Err
            .filter_map(|res| res.ok())
            // Filter out the None
            .filter_map(|x| x)
            .collect::<Vec<bool>>();

        // Is the remaining list non-empty and all true?
        if !matches.is_empty() && matches.iter().all(|x| *x) {
            Some(mapping.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::Matcher;
    use crate::config::mappings::test::test_mappings;
    use crate::matcher::{ExactMatchAllHeaders, ExactMatchMethod, ExactMatchUrl};
    use http::Request;

    #[test]
    #[allow(box_pointers)]
    fn matching() {
        let mappings = test_mappings().expect("Unable to setup mappings!");

        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/json");
        let _ = request_builder.header("Content-Type", "application/json");

        let mut matcher = Matcher::default();
        let _ = matcher.push(ExactMatchMethod::default());
        let _ = matcher.push(ExactMatchUrl::default());
        let _ = matcher.push(ExactMatchAllHeaders::default());
        assert!(!matcher.matchers.is_empty());

        if let Ok(request) = request_builder.body(()) {
            if let Ok(mapping) = matcher.get_match(&request, &mappings) {
                assert_eq!(*mapping.priority(), 1);
                assert!(mapping.response().body_file_name().is_some());
            } else {
                assert!(false, "Unable to find a matching mapping!");
            }
        } else {
            assert!(false, "Unable to build the request to test!");
        }
    }
}
