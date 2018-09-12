// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` request/response mappings
use clap::ArgMatches;
use crate::config::Mapping;
use crate::error::Error::MappingKeyCollision;
use crate::util;
use failure::Error;
use getset::Getters;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use toml;
use uuid::Uuid;

/// A map of `Mappings`.   Each is stored by `Uuid`.
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Mappings {
    /// The private inner hashmap.
    #[get = "pub"]
    inner: HashMap<Uuid, Mapping>,
}

impl<'a> TryFrom<&'a ArgMatches<'a>> for Mappings {
    type Error = Error;

    fn try_from(matches: &'a ArgMatches<'_>) -> Result<Self, Error> {
        let mut mappings = Self::default();

        let mappings_path = if let Some(mappings_path) = matches.value_of("mappings_path") {
            PathBuf::from(mappings_path).join("mappings")
        } else {
            PathBuf::from("mappings")
        };

        util::visit_dirs(&mappings_path, &mut |entry| -> Result<(), Error> {
            let f = File::open(entry.path())?;
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::new();
            let _bytes_read = reader.read_to_end(&mut buffer)?;
            let mapping: Mapping = toml::from_slice(&buffer)?;
            if let Some(_v) = mappings.inner.insert(Uuid::new_v4(), mapping) {
                Err(MappingKeyCollision.into())
            } else {
                Ok(())
            }
        })?;
        Ok(mappings)
    }
}
