// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` utilities
use failure::Error;
use futures::{future, Future};
use http::header::{HeaderValue, CONTENT_TYPE};
use http::{Response, StatusCode};
use serde_derive::Serialize;
use std::fs::{self, DirEntry};
use std::path::Path;

#[allow(box_pointers)]
crate type FutResponse = Box<dyn Future<Item = Response<String>, Error = String> + Send>;

crate fn visit_dirs<F>(dir: &Path, cb: &mut F) -> Result<(), Error>
where
    F: FnMut(&DirEntry) -> Result<(), Error>,
{
    if fs::metadata(dir)?.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if fs::metadata(entry.path())?.is_dir() {
                visit_dirs(&entry.path(), cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

#[allow(box_pointers)]
crate fn error_response_fut(body: String, status_code: StatusCode) -> FutResponse {
    Box::new(future::ok(error_response(body, status_code)))
}

crate fn error_response(message: String, status_code: StatusCode) -> Response<String> {
    let mut response = Response::builder();
    let _ = response
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .status(status_code);

    if let Ok(message) = serde_json::to_string(&ErrorMessage { message }) {
        if let Ok(response) = response.body(message) {
            return response;
        }
    }

    Response::new(r#"{ "message": "Unable to process body" }"#.to_string())
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}
