// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Files configuration
use crate::error::Error;
use clap::ArgMatches;
use getset::{Getters, Setters};
use std::convert::TryFrom;
use std::path::PathBuf;

/// Files configuration.
///
/// This represents a path to the static response files.
#[derive(Clone, Debug, Default, Getters, Hash, Eq, PartialEq, Setters)]
pub struct Files {
    /// The path to the response files
    #[get = "pub"]
    #[set = "pub"]
    path: PathBuf,
}

impl<'a> TryFrom<&'a ArgMatches<'a>> for Files {
    type Error = Error;

    fn try_from(matches: &'a ArgMatches<'a>) -> Result<Self, Error> {
        let files_path = if let Some(files_path) = matches.value_of("files_path") {
            PathBuf::from(files_path).join("files")
        } else {
            PathBuf::from("files")
        };
        Ok(Self { path: files_path })
    }
}

#[cfg(test)]
crate mod test {
    use super::Files;
    use crate::error::Error;
    use clap::{App, Arg};
    use std::convert::TryFrom;

    crate fn test_files() -> Result<Files, Error> {
        let args = vec!["test", "-f", "files"];

        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Proxy server for hosting mocked responses on match criteria")
            .arg(
                Arg::with_name("files_path")
                    .short("f")
                    .long("files_path")
                    .takes_value(true)
                    .value_name("FILES_PATH"),
            )
            .get_matches_from(args);

        Ok(Files::try_from(&matches)?)
    }
}
