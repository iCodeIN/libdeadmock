// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` errors

/// `deadmock` errors
#[derive(Copy, Clone, Debug, Fail)]
pub enum DeadmockError {
    /// If `use-proxy` is true, a `proxy-url` must also be given.
    #[fail(display = "invalid proxy configuration! proxy url is required")]
    InvalidProxyConfig,
}
