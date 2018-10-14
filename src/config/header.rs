// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! HTTP header configuration
use getset::{Getters, MutGetters, Setters};
use libeither::Either;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

/// HTTP header configuration
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

/// HTTP header pattern configuration
#[derive(
    Clone, Debug, Deserialize, Eq, Getters, Hash, MutGetters, PartialEq, Serialize, Setters,
)]
pub struct HeaderPattern {
    /// Either the header key, i.e. 'Content-Type' or a header key pattern, i.e. '^X-.*'
    #[get = "pub"]
    #[get_mut]
    key: Either<String, String>,
    /// Either the header value, i.e. 'application/json' or a header key pattern, i.e. '^application/.*'
    #[get = "pub"]
    #[get_mut]
    value: Either<String, String>,
}

impl fmt::Display for HeaderPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

#[cfg(test)]
crate mod test {
    use super::{Header, HeaderPattern};
    use libeither::Either;

    const EMPTY_HEADER: &str = r#"{"key":"","value":""}"#;
    const CONTENT_TYPE_JSON: &str = r#"{"key":"Content-Type","value":"application/json"}"#;
    const CONTENT_TYPE_TOML: &str = r#"key = "Content-Type"
value = "application/json"
"#;
    const CONTENT_TYPE_EITHER_JSON: &str = r#"{"key":{"left":"Content-Type","right":null},"value":{"left":null,"right":"^application/.*"}}"#;
    const CONTENT_TYPE_EITHER_TOML: &str = r#"[key]
left = "Content-Type"

[value]
right = "^application/.*"
"#;
    const BAD_HEADER_JSON: &str = r#"{"key":"blah"}"#;

    crate fn content_type_header_pattern() -> HeaderPattern {
        HeaderPattern {
            key: Either::new_left("Content-Type".to_string()),
            value: Either::new_right("^application/.*".to_string()),
        }
    }

    crate fn content_type_header() -> Header {
        Header {
            key: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }
    }

    crate fn content_type_star_pattern() -> HeaderPattern {
        HeaderPattern {
            key: Either::new_left("Content-Type".to_string()),
            value: Either::new_right("*".to_string()),
        }
    }

    crate fn accept_star_pattern() -> HeaderPattern {
        HeaderPattern {
            key: Either::new_left("Accept".to_string()),
            value: Either::new_right("*".to_string()),
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
    fn serialize_header_json() {
        if let Ok(serialized) = serde_json::to_string(&content_type_header()) {
            assert_eq!(serialized, CONTENT_TYPE_JSON);
        } else {
            assert!(false, "Serialization not expected to fail!");
        }
    }

    #[test]
    fn serialize_header_toml() {
        if let Ok(serialized) = toml::to_string(&content_type_header()) {
            assert_eq!(serialized, CONTENT_TYPE_TOML);
        } else {
            assert!(false, "Serialization not expected to fail!");
        }
    }

    #[test]
    fn serialize_header_pattern() {
        if let Ok(serialized) = serde_json::to_string(&content_type_header_pattern()) {
            assert_eq!(serialized, CONTENT_TYPE_EITHER_JSON);
        } else {
            assert!(false, "Serialization not expected to fail!");
        }
    }

    #[test]
    fn serialize_header_pattern_toml() {
        if let Ok(serialized) = toml::to_string(&content_type_header_pattern()) {
            assert_eq!(serialized, CONTENT_TYPE_EITHER_TOML);
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
    fn deserialize_header_json() {
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
    fn deserialize_header_pattern_json() {
        if let Ok(deserialized) = serde_json::from_str::<HeaderPattern>(CONTENT_TYPE_EITHER_JSON) {
            assert_eq!(deserialized, content_type_header_pattern());
        } else {
            assert!(
                false,
                "Expected deserialization of string into Header to succeed!"
            );
        }
    }

    #[test]
    fn deserialize_header_pattern_toml() {
        if let Ok(deserialized) = toml::from_str::<HeaderPattern>(CONTENT_TYPE_EITHER_TOML) {
            assert_eq!(deserialized, content_type_header_pattern());
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
