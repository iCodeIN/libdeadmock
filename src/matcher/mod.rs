// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request matching for the server.
use bitflags::bitflags;
use crate::config::{self, Mapping, Mappings, Request as RequestConfig};
use crate::error::Error;
use crate::error::ErrorKind::MappingNotFound;
use http::header::{HeaderName, HeaderValue};
use http::Request;
use slog::{trace, Logger};
use slog_try::try_trace;
use std::fmt;

#[cfg(feature = "header")]
crate mod header;
#[cfg(feature = "headers")]
crate mod headers;
#[cfg(feature = "method")]
crate mod method;
#[cfg(feature = "url")]
crate mod url;

#[cfg(all(feature = "exact_match", feature = "header"))]
pub use self::header::ExactMatch as ExactMatchHeader;
#[cfg(all(feature = "pattern_match", feature = "header"))]
pub use self::header::PatternMatch as PatternMatchHeader;
#[cfg(all(feature = "exact_match", feature = "headers"))]
pub use self::headers::ExactMatch as ExactMatchHeaders;
#[cfg(all(feature = "pattern_match", feature = "headers"))]
pub use self::headers::PatternMatch as PatternMatchHeaders;
#[cfg(all(feature = "exact_match", feature = "method"))]
pub use self::method::ExactMatch as ExactMatchMethod;
#[cfg(all(feature = "pattern_match", feature = "method"))]
pub use self::method::PatternMatch as PatternMatchMethod;
#[cfg(all(feature = "exact_match", feature = "url"))]
pub use self::url::ExactMatch as ExactMatchUrl;
#[cfg(all(feature = "pattern_match", feature = "url"))]
pub use self::url::PatternMatch as PatternMatchUrl;

bitflags!{
    /// Enabled flags for request matching types
    pub struct Enabled: u32 {
        /// Enable the exact matching on url
        #[cfg(all(feature = "exact_match", feature = "url"))]
        const EXACT_URL       = 0b0000_0000_0001;
        /// Enable the exact matching on method
        #[cfg(all(feature = "exact_match", feature = "method"))]
        const EXACT_METHOD    = 0b0000_0000_0010;
        /// Enable the exact matching on all headers
        #[cfg(all(feature = "exact_match", feature = "headers"))]
        const EXACT_HEADERS   = 0b0000_0000_0100;
        /// Enable the exact matching on one header
        #[cfg(all(feature = "exact_match", feature = "header"))]
        const EXACT_HEADER    = 0b0000_0000_1000;
        /// Enable the pattern matching on url
        #[cfg(all(feature = "pattern_match", feature = "url"))]
        const PATTERN_URL     = 0b0000_0001_0000;
        /// Enable the pattern matching on one header
        #[cfg(all(feature = "pattern_match", feature = "header"))]
        const PATTERN_HEADER  = 0b0000_1000_0000;
        /// Enable the pattern matching on method
        #[cfg(all(feature = "pattern_match", feature = "method"))]
        const PATTERN_METHOD  = 0b0001_0000_0000;
        /// Enable the pattern matching on all headers
        #[cfg(all(feature = "pattern_match", feature = "headers"))]
        const PATTERN_HEADERS = 0b0010_0000_0000;
    }
}

crate type HeaderTuple = (HeaderName, HeaderValue);
crate type HeaderTupleRef<'a> = (&'a HeaderName, &'a HeaderValue);

crate fn to_header_tuple(header: &config::Header) -> Result<HeaderTuple, failure::Error> {
    Ok((
        HeaderName::from_bytes(header.key().as_bytes())?,
        HeaderValue::from_bytes(header.value().as_bytes())?,
    ))
}

crate fn equal_headers(actual: HeaderTupleRef<'_>, expected: HeaderTupleRef<'_>) -> bool {
    actual == expected
}

/// A struct that supports slog logging
pub trait Slogger {
    /// Add an optional stdout `slog` logger to the struct.
    fn set_stdout(self, stdout: Option<Logger>) -> Self;
    /// Add an optional stderr `slog` logger to the struct.
    fn set_stderr(self, stderr: Option<Logger>) -> Self;
}

/// A request matcher
pub trait RequestMatch: fmt::Debug + fmt::Display {
    /// Does the incoming request match the request configuration from a mapping.
    ///
    /// If the matcher has configuration, then `is_match` must return `Some(bool)`.
    /// Otherwise, `is_match` must return `None`
    fn is_match(
        &self,
        request: &Request<()>,
        request_config: &RequestConfig,
    ) -> Result<Option<bool>, Error>;
}

/// Try to match an incoming request to a mapping.
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

#[cfg(all(feature = "exact_match", feature = "url"))]
fn enable_exact_match_url(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<ExactMatchUrl>(enabled, Enabled::EXACT_URL, matcher);
}

#[cfg(not(all(feature = "exact_match", feature = "url")))]
fn enable_exact_match_url(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "pattern_match", feature = "url"))]
fn enable_pattern_match_url(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<PatternMatchUrl>(enabled, Enabled::PATTERN_URL, matcher);
}

#[cfg(not(all(feature = "pattern_match", feature = "url")))]
fn enable_pattern_match_url(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "exact_match", feature = "method"))]
fn enable_exact_match_mehod(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<ExactMatchMethod>(enabled, Enabled::EXACT_METHOD, matcher);
}

#[cfg(not(all(feature = "exact_match", feature = "method")))]
fn enable_exact_match_method(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "pattern_match", feature = "method"))]
fn enable_pattern_match_method(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<PatternMatchMethod>(enabled, Enabled::PATTERN_METHOD, matcher);
}

#[cfg(not(all(feature = "pattern_match", feature = "method")))]
fn enable_pattern_match_method(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "exact_match", feature = "header"))]
fn enable_exact_match_header(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<ExactMatchHeader>(enabled, Enabled::EXACT_HEADER, matcher);
}

#[cfg(not(all(feature = "exact_match", feature = "header")))]
fn enable_exact_match_header(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "pattern_match", feature = "header"))]
fn enable_pattern_match_header(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<PatternMatchHeader>(enabled, Enabled::PATTERN_HEADER, matcher);
}

#[cfg(not(all(feature = "pattern_match", feature = "header")))]
fn enable_pattern_match_header(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "exact_match", feature = "headers"))]
fn enable_exact_match_headers(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<ExactMatchHeaders>(enabled, Enabled::EXACT_HEADERS, matcher);
}

#[cfg(not(all(feature = "exact_match", feature = "headers")))]
fn enable_exact_match_headers(_enabled: Enabled, _matcher: &mut Matcher) {}

#[cfg(all(feature = "pattern_match", feature = "headers"))]
fn enable_pattern_match_headers(enabled: Enabled, matcher: &mut Matcher) {
    enable_matcher::<PatternMatchHeaders>(enabled, Enabled::PATTERN_HEADERS, matcher);
}

#[cfg(not(all(feature = "pattern_match", feature = "headers")))]
fn enable_pattern_match_headers(_enabled: Enabled, _matcher: &mut Matcher) {}

fn enable_matcher<T>(enabled: Enabled, contains: Enabled, matcher: &mut Matcher)
where
    T: 'static + RequestMatch + Default + Slogger,
{
    if enabled.contains(contains) {
        let _ = matcher.push(
            T::default()
                .set_stdout(matcher.stdout.clone())
                .set_stderr(matcher.stderr.clone()),
        );
    }
}

#[allow(box_pointers)]
impl Matcher {
    /// Create a new `Matcher`
    pub fn new(enabled: Enabled, stdout: Option<Logger>, stderr: Option<Logger>) -> Self {
        let mut matcher = Self {
            matchers: vec![],
            stdout,
            stderr,
        };

        enable_exact_match_url(enabled, &mut matcher);
        enable_pattern_match_url(enabled, &mut matcher);
        enable_exact_match_mehod(enabled, &mut matcher);
        enable_pattern_match_method(enabled, &mut matcher);
        enable_exact_match_header(enabled, &mut matcher);
        enable_pattern_match_header(enabled, &mut matcher);
        enable_exact_match_headers(enabled, &mut matcher);
        enable_pattern_match_headers(enabled, &mut matcher);

        matcher
    }

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
    fn push<T: RequestMatch + 'static>(&mut self, request_match: T) -> &mut Self {
        self.matchers.push(Box::new(request_match));
        self
    }

    /// Get a mapping that matches the given request.
    pub fn get_match(&self, request: &Request<()>, mappings: &Mappings) -> Result<Mapping, Error> {
        mappings
            .inner()
            .iter()
            .inspect(|(_uuid, mapping)| {
                try_trace!(self.stdout, "");
                try_trace!(
                    self.stdout,
                    "{:#^1$}",
                    format!(" Checking '{}' ", mapping.name()),
                    80
                );
            })
            .filter_map(|(_uuid, mapping)| self.is_match(request, mapping))
            .min()
            .ok_or_else(|| MappingNotFound.into())
    }

    fn is_match(&self, request: &Request<()>, mapping: &Mapping) -> Option<Mapping> {
        let matches = self
            .matchers
            .iter()
            // Generate a list of matches
            // * If the matcher was configured and matches, returns `Some(true)`
            // * If the matcher was configured and doesn't match, returns `Some(false)`
            // * If the matcher was not configured, returns `None`
            .map(|matcher| matcher.is_match(request, mapping.request()))
            // Filter out any Errors
            .filter_map(|res| res.ok())
            // Filter out the `None` from matchers that weren't configured
            .filter_map(|x| x)
            .collect::<Vec<bool>>();

        let all_true = matches.iter().all(|x| *x);
        try_trace!(self.stdout, "Matches: {:?}, All: {}", matches, all_true);

        // Is the remaining list non-empty and all true?
        if !matches.is_empty() && all_true {
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
    use crate::matcher::Enabled;
    use http::request::Builder;
    use http::Request;

    #[test]
    #[allow(box_pointers)]
    fn matching() {
        let mappings = test_mappings().expect("Unable to setup mappings!");

        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/json");
        let _ = request_builder.header("Content-Type", "application/json");

        let enabled = Enabled::EXACT_URL | Enabled::EXACT_METHOD | Enabled::EXACT_HEADERS;
        let matcher = Matcher::new(enabled, None, None);
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

    #[test]
    #[allow(box_pointers)]
    fn matching_one_header() {
        let mappings = test_mappings().expect("Unable to setup mappings!");

        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/json");
        let _ = request_builder.header("Content-Type", "application/json");

        let enabled = Enabled::EXACT_URL | Enabled::EXACT_METHOD | Enabled::EXACT_HEADER;
        let matcher = Matcher::new(enabled, None, None);
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

    #[test]
    #[allow(box_pointers)]
    fn match_header_pattern() {
        let mappings = test_mappings().expect("Unable to setup mappings!");

        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/json");
        let _ = request_builder.header("Content-Type", "application/json");
        let _ = request_builder.header("X-Pattern-Match", "yoda-darth");

        let enabled = Enabled::PATTERN_HEADER;
        let matcher = Matcher::new(enabled, None, None);
        assert!(!matcher.matchers.is_empty());

        if let Ok(request) = request_builder.body(()) {
            if let Ok(mapping) = matcher.get_match(&request, &mappings) {
                assert_eq!(*mapping.priority(), 2);
                assert!(mapping.response().body_file_name().is_some());
            } else {
                assert!(false, "Unable to find a matching mapping!");
            }
        } else {
            assert!(false, "Unable to build the request to test!");
        }
    }

    #[allow(box_pointers)]
    fn check_request(enabled: Enabled, request_builder: &mut Builder, priority: u8, name: &str) {
        let mappings = test_mappings().expect("Unable to setup mappings!");
        let matcher = Matcher::new(enabled, None, None);
        assert!(!matcher.matchers.is_empty());

        if let Ok(request) = request_builder.body(()) {
            if let Ok(mapping) = matcher.get_match(&request, &mappings) {
                assert_eq!(*mapping.priority(), priority);
                assert_eq!(mapping.name(), name);
                assert!(mapping.response().body_file_name().is_some());
            } else {
                assert!(false, "Unable to find a matching mapping!");
            }
        } else {
            assert!(false, "Unable to build the request to test!");
        }
    }

    #[test]
    fn match_method_pattern() {
        let mut put_request = Request::builder();
        let _ = put_request.uri("/toodles");
        let _ = put_request.header("Content-Type", "application/json");
        let _ = put_request.method("PUT");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut put_request,
            3,
            "Method Pattern Match",
        );

        let mut post_request = Request::builder();
        let _ = post_request.uri("/poodles");
        let _ = post_request.header("Content-Type", "application/json");
        let _ = post_request.method("POST");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut post_request,
            3,
            "Method Pattern Match",
        );

        let mut patch_request = Request::builder();
        let _ = patch_request.uri("/noodles");
        let _ = patch_request.header("Content-Type", "application/json");
        let _ = patch_request.method("PATCH");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut patch_request,
            3,
            "Method Pattern Match",
        );
    }
}
