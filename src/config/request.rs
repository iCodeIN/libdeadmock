// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` request matching configuration
use crate::config::Header;
use getset::Getters;
use serde_derive::{Deserialize, Serialize};

/// `libdeadmock` request matching configuration.
#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Request {
    /// The HTTP request method to match.
    #[get = "pub"]
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    /// The url to exact match.
    #[get = "pub"]
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// The url pattern to match (regex).
    #[get = "pub"]
    #[serde(skip_serializing_if = "Option::is_none")]
    url_pattern: Option<String>,
    /// The HTTP headers to match (exact).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    headers: Option<Vec<Header>>,
    /// The HTTP header to match (exact).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    header: Option<Header>,
}

#[cfg(test)]
crate mod test {
    use super::Request;
    use crate::config::header::test::content_type_header;

    const EMPTY_REQUEST: &str = "{}";
    const PARTIAL_REQUEST: &str = r#"{"method":"GET","url":"http://a.url.com"}"#;
    const FULL_REQUEST: &str = r#"{"method":"GET","url":"http://a.url.com","url_pattern":".*jasonozias.*","headers":[{"key":"Content-Type","value":"application/json"}],"header":{"key":"Content-Type","value":"application/json"}}"#;
    const BAD_REQUEST: &str = r#"{"method":}"#;

    crate fn partial_request() -> Request {
        let mut partial_request = Request::default();
        partial_request.method = Some("GET".to_string());
        partial_request.url = Some("http://a.url.com".to_string());
        partial_request
    }

    crate fn full_request() -> Request {
        let mut request = partial_request();
        request.url_pattern = Some(".*jasonozias.*".to_string());
        request.headers = Some(vec![content_type_header()]);
        request.header = Some(content_type_header());
        request
    }

    #[test]
    fn serialize_empty_reqeust() {
        if let Ok(req_str) = serde_json::to_string(&Request::default()) {
            assert_eq!(req_str, EMPTY_REQUEST);
        } else {
            assert!(false, "Expected serialization of empty request to succeed!");
        }
    }

    #[test]
    fn serialize_partial_reqeust() {
        if let Ok(req_str) = serde_json::to_string(&partial_request()) {
            assert_eq!(req_str, PARTIAL_REQUEST);
        } else {
            assert!(
                false,
                "Expected serialization of partial request to succeed!"
            );
        }
    }

    #[test]
    fn serialize_request() {
        if let Ok(req_str) = serde_json::to_string(&full_request()) {
            assert_eq!(req_str, FULL_REQUEST);
        } else {
            assert!(false, "Expected serialization of full request to succeed!");
        }
    }

    #[test]
    fn deserialize_empty_request() {
        if let Ok(deserialized) = serde_json::from_str::<Request>(EMPTY_REQUEST) {
            assert_eq!(deserialized, Request::default());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Request to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_partial_request() {
        if let Ok(deserialized) = serde_json::from_str::<Request>(PARTIAL_REQUEST) {
            assert_eq!(deserialized, partial_request());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Request to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_request() {
        if let Ok(deserialized) = serde_json::from_str::<Request>(FULL_REQUEST) {
            assert_eq!(deserialized, full_request());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Request to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_bad_request() {
        assert!(
            serde_json::from_str::<Request>(BAD_REQUEST).is_err(),
            "Expected the deserialization to fail!"
        );
    }
}
