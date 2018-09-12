// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` response templating configuration
use crate::config::Header;
use getset::Getters;
use serde_derive::{Deserialize, Serialize};

/// `libdeadmock` response configuration
#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Response {
    /// The http status code to send on the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    status: Option<u16>,
    /// The http headers to send on the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    headers: Option<Vec<Header>>,
    /// The file to use as the http response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    body_file_name: Option<String>,
    /// The base url of the proxy you wish to generate the response from.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    proxy_base_url: Option<String>,
    /// Additional headers to send along with the request to the proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[get = "pub"]
    additional_proxy_request_headers: Option<Vec<Header>>,
}

#[cfg(test)]
crate mod test {
    use super::Response;
    use crate::config::header::test::{additional_proxy_request_headers, content_type_header};

    const EMPTY_RESPONSE: &str = "{}";
    const PARTIAL_RESPONSE: &str = r#"{"status":200,"headers":[{"key":"Content-Type","value":"application/json"}],"proxy_base_url":"http://cdcproxy.kroger.com"}"#;
    const FULL_RESPONSE: &str = r#"{"status":200,"headers":[{"key":"Content-Type","value":"application/json"}],"body_file_name":"test.json","proxy_base_url":"http://cdcproxy.kroger.com","additional_proxy_request_headers":[{"key":"Authorization","value":"Basic abcdef123"}]}"#;
    const BAD_RESPONSE: &str = r#"{"status":"abc"}"#;

    crate fn partial_response() -> Response {
        let mut response = Response::default();
        response.status = Some(200);
        response.proxy_base_url = Some("http://cdcproxy.kroger.com".to_string());
        response.headers = Some(vec![content_type_header()]);
        response
    }

    crate fn full_response() -> Response {
        let mut response = partial_response();
        response.body_file_name = Some("test.json".to_string());
        response.additional_proxy_request_headers = Some(vec![additional_proxy_request_headers()]);
        response
    }

    #[test]
    fn serialize_empty_response() {
        if let Ok(req_str) = serde_json::to_string(&Response::default()) {
            assert_eq!(req_str, EMPTY_RESPONSE);
        } else {
            assert!(
                false,
                "Expected serialization of empty response to succeed!"
            );
        }
    }

    #[test]
    fn serialize_partial_response() {
        if let Ok(req_str) = serde_json::to_string(&partial_response()) {
            assert_eq!(req_str, PARTIAL_RESPONSE);
        } else {
            assert!(
                false,
                "Expected serialization of partial response to succeed!"
            );
        }
    }

    #[test]
    fn serialize_full_response() {
        if let Ok(req_str) = serde_json::to_string(&full_response()) {
            assert_eq!(req_str, FULL_RESPONSE);
        } else {
            assert!(false, "Expected serialization of full response to succeed!");
        }
    }

    #[test]
    fn deserialize_empty_response() {
        if let Ok(deserialized) = serde_json::from_str::<Response>(EMPTY_RESPONSE) {
            assert_eq!(deserialized, Response::default());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Response to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_partial_response() {
        if let Ok(deserialized) = serde_json::from_str::<Response>(PARTIAL_RESPONSE) {
            assert_eq!(deserialized, partial_response());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Response to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_full_response() {
        if let Ok(deserialized) = serde_json::from_str::<Response>(FULL_RESPONSE) {
            assert_eq!(deserialized, full_response());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Response to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_bad_response() {
        assert!(
            serde_json::from_str::<Response>(BAD_RESPONSE).is_err(),
            "Expected the deserialization to fail!"
        );
    }
}
