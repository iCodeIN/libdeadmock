// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP request matching for the server.
#[cfg(feature = "headers")]
use crate::config::Header;
use crate::config::{Mapping, Mappings, Request as RequestConfig};
use crate::error::Error::{self, MappingNotFound};
use bitflags::bitflags;
#[cfg(feature = "headers")]
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

bitflags! {
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

impl Enabled {
    /// Enable all of the exact matching.
    pub fn exact() -> Self {
        Self::exact_url() | Self::exact_method() | Self::exact_header() | Self::exact_headers()
    }

    /// Enable all of the pattern matching.
    pub fn pattern() -> Self {
        Self::pattern_url()
            | Self::pattern_method()
            | Self::pattern_header()
            | Self::pattern_headers()
    }

    #[cfg(all(feature = "exact_match", feature = "url"))]
    fn exact_url() -> Self {
        Self::EXACT_URL
    }

    #[cfg(not(all(feature = "exact_match", feature = "url")))]
    fn exact_url() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "exact_match", feature = "method"))]
    fn exact_method() -> Self {
        Self::EXACT_METHOD
    }

    #[cfg(not(all(feature = "exact_match", feature = "method")))]
    fn exact_method() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "exact_match", feature = "header"))]
    fn exact_header() -> Self {
        Self::EXACT_HEADER
    }

    #[cfg(not(all(feature = "exact_match", feature = "header")))]
    fn exact_header() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "exact_match", feature = "headers"))]
    fn exact_headers() -> Self {
        Self::EXACT_HEADERS
    }

    #[cfg(not(all(feature = "exact_match", feature = "headers")))]
    fn exact_headers() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "pattern_match", feature = "url"))]
    fn pattern_url() -> Self {
        Self::PATTERN_URL
    }

    #[cfg(not(all(feature = "pattern_match", feature = "url")))]
    fn pattern_url() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "pattern_match", feature = "method"))]
    fn pattern_method() -> Self {
        Self::PATTERN_METHOD
    }

    #[cfg(not(all(feature = "pattern_match", feature = "method")))]
    fn pattern_method() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "pattern_match", feature = "header"))]
    fn pattern_header() -> Self {
        Self::PATTERN_HEADER
    }

    #[cfg(not(all(feature = "pattern_match", feature = "header")))]
    fn pattern_header() -> Self {
        Self::empty()
    }

    #[cfg(all(feature = "pattern_match", feature = "headers"))]
    fn pattern_headers() -> Self {
        Self::PATTERN_HEADERS
    }

    #[cfg(not(all(feature = "pattern_match", feature = "headers")))]
    fn pattern_headers() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Enabled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "headers")]
crate type HeaderTuple = (HeaderName, HeaderValue);
#[cfg(feature = "headers")]
crate type HeaderTupleRef<'a> = (&'a HeaderName, &'a HeaderValue);

#[cfg(feature = "headers")]
crate fn to_header_tuple(header: &Header) -> Result<HeaderTuple, Error> {
    Ok((
        HeaderName::from_bytes(header.key().as_bytes())?,
        HeaderValue::from_bytes(header.value().as_bytes())?,
    ))
}

#[cfg(feature = "headers")]
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
fn enable_exact_match_method(enabled: Enabled, matcher: &mut Matcher) {
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
        enable_exact_match_method(enabled, &mut matcher);
        enable_pattern_match_method(enabled, &mut matcher);
        enable_exact_match_header(enabled, &mut matcher);
        enable_pattern_match_header(enabled, &mut matcher);
        enable_exact_match_headers(enabled, &mut matcher);
        enable_pattern_match_headers(enabled, &mut matcher);

        matcher
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
            .ok_or_else(|| MappingNotFound)
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

impl Slogger for Matcher {
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

#[cfg(test)]
mod test {
    use super::Matcher;
    use crate::config::files::test::test_files;
    use crate::config::mappings::test::test_mappings;
    use crate::matcher::Enabled;
    use http::request::Builder;
    use http::Request;
    // use slog::{o, Drain};
    // use slog_term;

    #[test]
    fn enable_pattern() {
        let all_pattern = Enabled::pattern();
        assert!(!all_pattern.is_empty());
        assert!(all_pattern.contains(
            Enabled::PATTERN_URL
                | Enabled::PATTERN_METHOD
                | Enabled::PATTERN_HEADER
                | Enabled::PATTERN_HEADERS
        ));
        assert!(!all_pattern.contains(Enabled::EXACT_URL));
        assert!(!all_pattern.contains(Enabled::EXACT_METHOD));
        assert!(!all_pattern.contains(Enabled::EXACT_HEADER));
        assert!(!all_pattern.contains(Enabled::EXACT_HEADERS));
    }

    #[test]
    fn enable_exact() {
        let all_exact = Enabled::exact();
        assert!(!all_exact.is_empty());
        assert!(all_exact.contains(
            Enabled::EXACT_URL
                | Enabled::EXACT_METHOD
                | Enabled::EXACT_HEADER
                | Enabled::EXACT_HEADERS
        ));
        assert!(!all_exact.contains(Enabled::PATTERN_URL));
        assert!(!all_exact.contains(Enabled::PATTERN_METHOD));
        assert!(!all_exact.contains(Enabled::PATTERN_HEADER));
        assert!(!all_exact.contains(Enabled::PATTERN_HEADERS));
    }

    #[allow(box_pointers)]
    fn check_request(enabled: Enabled, request_builder: &mut Builder, priority: u8, name: &str) {
        let mappings = test_mappings().expect("Unable to setup mappings!");
        // let decorator = slog_term::PlainDecorator::new(std::io::stderr());
        // let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        // let drain = slog_async::Async::new(drain).build().fuse();
        // let log = slog::Logger::root(drain, o!("test" => "test"));

        let matcher = Matcher::new(enabled, None, None);
        assert!(!matcher.matchers.is_empty());

        if let Ok(request) = request_builder.body(()) {
            if let Ok(mapping) = matcher.get_match(&request, &mappings) {
                assert_eq!(mapping.name(), name);
                assert_eq!(*mapping.priority(), priority);
                assert!(mapping.response().body_file_name().is_some());
            } else {
                assert!(false, "Unable to find a matching mapping!");
            }
        } else {
            assert!(false, "Unable to build the request to test!");
        }
    }

    #[allow(box_pointers)]
    fn check_no_match(enabled: Enabled, request_builder: &mut Builder) {
        let mappings = test_mappings().expect("Unable to setup mappings!");
        let matcher = Matcher::new(enabled, None, None);
        assert!(!matcher.matchers.is_empty());

        if let Ok(request) = request_builder.body(()) {
            assert!(matcher.get_match(&request, &mappings).is_err());
        } else {
            assert!(false, "Unable to build the request to test!");
        }
    }

    #[test]
    #[allow(box_pointers)]
    fn load_test_files() {
        assert!(test_files().is_ok());
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_header() {
        let mut request_builder = Request::builder();
        let _ = request_builder.header("X-Exact-Match", "header");

        check_request(
            Enabled::EXACT_HEADER,
            &mut request_builder,
            1,
            "Exact Match - Header",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_headers() {
        let mut request_builder = Request::builder();
        let _ = request_builder.header("Content-Type", "application/json");
        let _ = request_builder.header("X-Exact-Headers", "true");

        check_request(
            Enabled::EXACT_HEADERS,
            &mut request_builder,
            1,
            "Exact Match - Headers",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_method() {
        let mut request_builder = Request::builder();
        let _ = request_builder.method("PATCH");

        check_request(
            Enabled::EXACT_METHOD,
            &mut request_builder,
            1,
            "Exact Match - Method",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_url() {
        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/plaintext");

        check_request(
            Enabled::EXACT_URL,
            &mut request_builder,
            1,
            "Exact Match - URL",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_method_and_url() {
        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/json");
        let _ = request_builder.method("GET");

        check_request(
            Enabled::EXACT_URL | Enabled::EXACT_METHOD,
            &mut request_builder,
            2,
            "Exact Match - Method & URL",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn exact_match_method_url_and_header() {
        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/header-method-url");
        let _ = request_builder.method("GET");
        let _ = request_builder.header("X-Exact-Match", "header-method-url");

        check_request(
            Enabled::EXACT_HEADER | Enabled::EXACT_METHOD | Enabled::EXACT_URL,
            &mut request_builder,
            3,
            "Exact Match - Header, Method, & URL",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn pattern_match_header() {
        let mut request_builder = Request::builder();
        let _ = request_builder.header("X-Pattern-Match", "yoda-darth");

        check_request(
            Enabled::PATTERN_HEADER,
            &mut request_builder,
            1,
            "Pattern Match - Header",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn pattern_match_headers() {
        let mut headers_request = Request::builder();
        let _ = headers_request.header("X-Correlation-Id", "12345");
        let _ = headers_request.header("X-Loyalty-Id", "abcd-1234");

        check_request(
            Enabled::PATTERN_HEADERS,
            &mut headers_request,
            2,
            "Pattern Match - Headers",
        );

        let mut long_corr_id = Request::builder();
        let _ = long_corr_id.header("X-Correlation-Id", "123456");
        let _ = long_corr_id.header("X-Loyalty-Id", "abcd-1234");

        check_no_match(Enabled::PATTERN_HEADERS, &mut long_corr_id);

        let mut short_corr_id = Request::builder();
        let _ = short_corr_id.header("X-Correlation-Id", "1234");
        let _ = short_corr_id.header("X-Loyalty-Id", "abcd-1234");

        check_no_match(Enabled::PATTERN_HEADERS, &mut short_corr_id);

        let mut invalid_loy_id = Request::builder();
        let _ = invalid_loy_id.header("X-Correlation-Id", "12345");
        let _ = invalid_loy_id.header("X-Loyalty-Id", "Abcd-1234");

        check_no_match(Enabled::PATTERN_HEADERS, &mut invalid_loy_id);
    }

    #[test]
    fn pattern_match_method() {
        let mut put_request = Request::builder();
        let _ = put_request.uri("/toodles");
        let _ = put_request.header("Content-Type", "application/json");
        let _ = put_request.method("PUT");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut put_request,
            3,
            "Pattern Match - Method",
        );

        let mut post_request = Request::builder();
        let _ = post_request.uri("/poodles");
        let _ = post_request.header("Content-Type", "application/json");
        let _ = post_request.method("POST");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut post_request,
            3,
            "Pattern Match - Method",
        );

        let mut patch_request = Request::builder();
        let _ = patch_request.uri("/noodles");
        let _ = patch_request.header("Content-Type", "application/json");
        let _ = patch_request.method("PATCH");

        check_request(
            Enabled::PATTERN_METHOD,
            &mut patch_request,
            3,
            "Pattern Match - Method",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn pattern_match_url() {
        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/admin/list");

        check_request(
            Enabled::PATTERN_URL,
            &mut request_builder,
            4,
            "Pattern Match - URL",
        );
    }

    #[test]
    #[allow(box_pointers)]
    fn mixed_match_header() {
        let mut request_builder = Request::builder();
        let _ = request_builder.uri("/mixed-match");
        let _ = request_builder.header("X-Pattern-Match", "mixed-match");

        check_request(
            Enabled::EXACT_URL | Enabled::PATTERN_HEADER,
            &mut request_builder,
            2,
            "Mixed Match - Header & URL",
        );
    }
}
