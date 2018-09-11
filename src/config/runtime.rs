// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` runtime environment configuration
use std::env;
use tomlenv::Environment;

const ENV: &str = "env";

/// The runtime environment configuration for deadmock.
#[derive(Clone, Debug, Default, Deserialize, Getters, Hash, Eq, PartialEq, Serialize)]
pub struct Runtime {
    /// The IP address to listen on.
    #[get = "pub"]
    ip: Option<String>,
    /// The port to listen on.
    #[get = "pub"]
    port: Option<u32>,
    /// The path to the mappings and templates
    #[get = "pub"]
    path: Option<String>,
}

impl Runtime {
    /// Get the `env` environment variable, setting it to `local` if the variable is not found or set already.
    ///
    /// # Example
    ///
    /// ```
    /// # use libdeadmock::RuntimeConfig;
    /// #
    /// # fn main() {
    /// assert_eq!("local", RuntimeConfig::env());
    /// # }
    /// ```
    pub fn env() -> String {
        env::var(ENV).unwrap_or_else(|_| {
            let env_str = Environment::Local.to_string();
            env::set_var(ENV, &env_str);
            env_str
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Runtime, ENV};
    use std::env;
    use tomlenv::Environment;

    fn validate_env(currenv: &str) {
        env::set_var(ENV, &currenv);
        assert_eq!(Runtime::env(), currenv);
        assert!(env::var(ENV).is_ok());
        env::remove_var(ENV);
    }
    #[test]
    fn local_env_when_not_set() {
        let local_env = Environment::Local.to_string();
        env::remove_var(ENV);
        assert_eq!(&Runtime::env(), &local_env);
        assert!(env::var(ENV).is_ok());
        env::remove_var(ENV);

        validate_env(&local_env);
        validate_env(&Environment::Dev.to_string());
        validate_env(&Environment::Test.to_string());
        validate_env(&Environment::Stage.to_string());
        validate_env(&Environment::Prod.to_string());
    }
}
