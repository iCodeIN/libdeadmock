// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Logging for the server.
use crate::config::Runtime;
use clap::ArgMatches;
use failure::Error;
use getset::Getters;
use slog::{o, Drain, Level, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};
use std::convert::TryFrom;

/// `slog` loggers for stdout/stderr.
#[derive(Clone, Debug, Default, Getters)]
pub struct Loggers {
    /// An optional stdout logger.
    #[get = "pub"]
    stdout: Option<Logger>,
    /// An optional stderr logger.
    #[get = "pub"]
    stderr: Option<Logger>,
}

impl Loggers {
    /// Split this `Loggers` into the stdout and stderr components.
    pub fn split(&self) -> (Option<Logger>, Option<Logger>) {
        (self.stdout.clone(), self.stderr.clone())
    }
}

impl<'a> TryFrom<&'a ArgMatches<'a>> for Loggers {
    type Error = Error;

    fn try_from(matches: &'a ArgMatches<'a>) -> Result<Self, Error> {
        let level = match matches.occurrences_of("v") {
            0 => Level::Warning,
            1 => Level::Info,
            2 => Level::Debug,
            3 | _ => Level::Trace,
        };

        let dm_env = Runtime::env();

        let stdout_decorator = TermDecorator::new().stdout().build();
        let stdout_drain = CompactFormat::new(stdout_decorator).build().fuse();
        let stdout_async_drain = Async::new(stdout_drain).build().filter_level(level).fuse();
        let stdout = Logger::root(stdout_async_drain, o!("env" => dm_env.clone()));

        let stderr_decorator = TermDecorator::new().stdout().build();
        let stderr_drain = CompactFormat::new(stderr_decorator).build().fuse();
        let stderr_async_drain = Async::new(stderr_drain)
            .build()
            .filter_level(Level::Error)
            .fuse();
        let stderr = Logger::root(stderr_async_drain, o!("env" => dm_env.clone()));

        Ok(Self {
            stdout: Some(stdout),
            stderr: Some(stderr),
        })
    }
}
