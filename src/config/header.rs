// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` header configuration
use getset::{Getters, MutGetters, Setters};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

/// `libdeadmock` header configuration
#[derive(
    Clone, Debug, Default, Deserialize, Eq, Getters, Hash, MutGetters, PartialEq, Serialize, Setters,
)]
pub struct Header {
    /// The header key, i.e. 'Content-Type'
    #[get = "pub"]
    #[get_mut]
    key: String,
    /// The header value, i.e. 'application/json'
    #[get = "pub"]
    #[get_mut]
    value: String,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[cfg(test)]
crate mod test {
    use super::Header;

    const EMPTY_HEADER: &str = r#"{"key":"","value":""}"#;
    const CONTENT_TYPE_JSON: &str = r#"{"key":"Content-Type","value":"application/json"}"#;
    const BAD_HEADER_JSON: &str = r#"{"key":"blah"}"#;

    crate fn content_type_header() -> Header {
        Header {
            key: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }
    }

    crate fn additional_proxy_request_headers() -> Header {
        let mut header = Header::default();
        (*header.key_mut()) = "Authorization".to_string();
        (*header.value_mut()) = "Basic abcdef123".to_string();
        header
    }

    #[test]
    fn serialize_empty_header() {
        if let Ok(serialized) = serde_json::to_string(&Header::default()) {
            assert_eq!(serialized, EMPTY_HEADER);
        } else {
            assert!(false, "Serialization not expected to fail!");
        }
    }

    #[test]
    fn serialize_header() {
        if let Ok(serialized) = serde_json::to_string(&content_type_header()) {
            assert_eq!(serialized, CONTENT_TYPE_JSON);
        } else {
            assert!(false, "Serialization not expected to fail!");
        }
    }

    #[test]
    fn deserialize_empty_header() {
        if let Ok(deserialized) = serde_json::from_str::<Header>(EMPTY_HEADER) {
            assert_eq!(deserialized, Header::default());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Header to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_header() {
        if let Ok(deserialized) = serde_json::from_str::<Header>(CONTENT_TYPE_JSON) {
            assert_eq!(deserialized, content_type_header());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Header to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_bad_header() {
        assert!(
            serde_json::from_str::<Header>(BAD_HEADER_JSON).is_err(),
            "Expected the deserialization to fail!"
        );
    }
}
