// Copyright 2016 Matthew Fornaciari <mattforni@gmail.com>
//! A dead simple implementation of command line argument parsing and validation
//! built on top of the [getopts](https://crates.io/crates/getopts) crate.
//!
//! In order to use the `args` crate simply create an `Args` object and begin
//! registering possible command line options via the `flag(...)` and `option(...)`
//! methods. Once all options have been registered, parse arguments directly from the
//! command line, or provide a vector of your own arguments.
//!
//! Any errors encountered during parsing will be returned wrapped in an `ArgsError`.
//! If there are no errors during parsing values may be retrieved from the `args`
//! object by simply calling `value_of(...)` and `validated_value_of(...)`.
//!
//! That's it!
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/args) and can be
//! used by adding `args` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! args = "2.0"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate args;
//! ```
//!
//! # Example
//!
//! The following example shows simple command line parsing for an application that
//! requires a number of iterations between zero *(0)* and ten *(10)* to be specified,
//! accepts an optional log file name and responds to the help flag.
//!
//! ```rust
//! extern crate args;
//! extern crate getopts;
//!
//! use getopts::Occur;
//! use std::process::exit;
//!
//! use args::{Args,ArgsError};
//! use args::validations::{Order,OrderValidation};
//!
//! const PROGRAM_DESC: &'static str = "Run this program";
//! const PROGRAM_NAME: &'static str = "program";
//!
//! fn main() {
//!     match parse(&vec!("-i", "5")) {
//!         Ok(_) => println!("Successfully parsed args"),
//!         Err(error) => {
//!             println!("{}", error);
//!             exit(1);
//!         }
//!     };
//! }
//!
//! fn parse(input: &Vec<&str>) -> Result<(), ArgsError> {
//!     let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
//!     args.flag("h", "help", "Print the usage menu");
//!     args.option("i",
//!         "iter",
//!         "The number of times to run this program",
//!         "TIMES",
//!         Occur::Req,
//!         None);
//!     args.option("l",
//!         "log_file",
//!         "The name of the log file",
//!         "NAME",
//!         Occur::Optional,
//!         Some(String::from("output.log")));
//!
//!     args.parse(input)?;
//!
//!     let help = args.value_of("help")?;
//!     if help {
//!         args.full_usage();
//!         return Ok(());
//!     }
//!
//!     let gt_0 = Box::new(OrderValidation::new(Order::GreaterThan, 0u32));
//!     let lt_10 = Box::new(OrderValidation::new(Order::LessThanOrEqual, 10u32));
//!
//!     let iters = args.validated_value_of("iter", &[gt_0, lt_10])?;
//!     for iter in 0..iters {
//!         println!("Working on iteration {}", iter);
//!     }
//!     println!("All done.");
//!
//!     Ok(())
//! }
//! ```
//!

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://doc.rust-lang.org/")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

#[macro_use] extern crate log;
extern crate getopts;

use getopts::{Fail,HasArg,Occur,Options};
use std::collections::BTreeMap;
use std::collections::btree_map::Iter;
use std::env;
use std::ffi::OsStr;
use std::fmt::{self,Display,Formatter};
use std::iter::IntoIterator;
use std::str::FromStr;

pub use self::errors::ArgsError;

use self::options::Opt;
use self::validations::Validation;

pub mod traits;
pub mod validations;

mod errors;
mod options;
#[cfg(test)] mod tst;

const COLUMN_WIDTH: usize = 20;
const SCOPE_PARSE: &'static str = "parse";
const SEPARATOR: &'static str = ",";

/// A dead simple implementation of command line argument parsing and validation.
pub struct Args {
    description: String,
    options: Options,
    opts: BTreeMap<String, Box<dyn Opt>>,
    opt_names: Vec<String>,
    program_name: String,
    values: BTreeMap<String, String>
}

impl Args {
    // Public associated methods
    /// Creates an empty set of command line options.
    pub fn new(program_name: &str, description: &str) -> Args {
        debug!("Creating new args object for '{}'", program_name);

        Args {
            description: description.to_string(),
            options: Options::new(),
            opts: BTreeMap::new(),
            opt_names: Vec::new(),
            program_name: program_name.to_string(),
            values: BTreeMap::new()
        }
    }

    // Public instance methods
    /// Registers an optional flag argument that does not take an argument and defaults to false.
    ///
    /// * `short_name` - e.g. `"h"` for a `-h` option, or `""` for none
    /// * `long_name` - e.g. `"help"` for a `--help` option, or `""` for none
    /// * `desc` - A description of the flag for the usage message
    pub fn flag(&mut self,
            short_name: &str,
            long_name: &str,
            desc: &str) -> &mut Args {
        self.register_opt(
            options::new(short_name,
                long_name,
                desc,
                "",
                HasArg::No,
                Occur::Optional,
                None
            )
        );

        self
    }

    /// Generates a combination of the short and verbose usage messages.
    pub fn full_usage(&self) -> String {
        format!("{}\n\n{}", self.short_usage(), self.usage())
    }

    /// Returns a `bool` indicating whether or not any options are registered.
    pub fn has_options(&self) -> bool {
        !self.opts.is_empty()
    }

    /// Returns a `bool` indicating whether or not a argument is present.
    pub fn has_value(&self, opt_name: &str) -> bool {
        self.values.get(opt_name).is_some()
    }

    /// Returns an iterator visiting all key-value pairs in alphabetical order.
    pub fn iter(&self) -> Iter<String, String> {
        self.values.iter()
    }

    /// Registers an option explicitly.
    ///
    /// * `short_name` - e.g. `"h"` for a `-h` option, or `""` for none
    /// * `long_name` - e.g. `"help"` for a `--help` option, or `""` for none
    /// * `desc` - A description of the flag for the usage message
    /// * `hint` - A hint to be used in place of the argument in the usage message,
    /// e.g. `"FILE"` for a `-o FILE` option
    /// * `occur` - An enum representing whether the option is required or not
    /// * `default` - The default value for this option if there should be one
    pub fn option(&mut self,
            short_name: &str,
            long_name: &str,
            desc: &str,
            hint: &str,
            occur: Occur,
            default: Option<String>) -> &mut Args {
        self.register_opt(
            options::new(short_name,
                long_name,
                desc,
                hint,
                HasArg::Yes,
                occur,
                default
            )
        );

        self
    }

    /// Parses arguments according to the registered options.
    ///
    /// # Failures
    /// Fails if any errors are encountered during parsing.
    pub fn parse<C: IntoIterator>(&mut self, raw_args: C) -> Result<(), ArgsError> where C::Item: AsRef<OsStr> {
        debug!("Parsing args for '{}'", self.program_name);

        // Get matches and return an error if there is a problem parsing
        let matches = match self.options.parse(raw_args) {
            Ok(matches) => { matches },
            Err(error) => { return Err(ArgsError::new(SCOPE_PARSE, &error.to_string())) }
        };

        // Find matches and store the values (or a default)
        for opt_name in &self.opt_names {
            let option = self.opts.get(opt_name);
            if option.is_none() {
                return Err(ArgsError::new(SCOPE_PARSE, &Fail::UnrecognizedOption(opt_name.to_string()).to_string()));
            }

            let opt = option.unwrap();
            let value = opt.parse(&matches).unwrap_or("".to_string());
            if !value.is_empty() {
                self.values.insert(opt_name.to_string(), value);
            } else {
                if opt.is_required() {
                    return Err(ArgsError::new(SCOPE_PARSE, &Fail::ArgumentMissing(opt_name.to_string()).to_string()));
                }
            }
        }

        debug!("Args: {:?}", self.values);
        Ok(())
    }

    /// Parses arguments directly from the command line according to the registered options.
    ///
    /// # Failures
    /// Fails if any errors are encountered during parsing.
    pub fn parse_from_cli(&mut self) -> Result<(), ArgsError> {
        // Retrieve the cli args and throw out the program name
        let mut raw_args: Vec<String> = env::args().collect();
        if !raw_args.is_empty() { raw_args.remove(0); }

        self.parse(&mut raw_args)
    }

    /// Generates a one-line usage summary from the registered options.
    pub fn short_usage(&self) -> String {
        self.options.short_usage(&self.program_name)
    }

    /// Generates a verbose usage summary from the registered options.
    pub fn usage(&self) -> String {
        if !self.has_options() { return format!("{}\n", self.description); }
        self.options.usage(&self.description)
    }

    /// Retrieves the optional value of the `Opt` identified by `opt_name`, casts it to
    /// the type specified by `T`, runs all provided `Validation`s, and wraps it in an Option<T>.
    ///
    /// # Failures
    ///
    /// See `validated_value_of`
    pub fn optional_validated_value_of<T>(&self, opt_name: &str, validations: &[Box<dyn Validation<T=T>>])
                                          -> Result<Option<T>, ArgsError> where T: FromStr {
        if self.has_value(opt_name) {
            Ok(Some(self.validated_value_of::<T>(opt_name, validations)?))
        } else {
            Ok(None)
        }
    }

    /// Retrieves the optional value of the `Opt` identified by `opt_name`, casts it to
    /// the type specified by `T` and wraps it in an optional.
    ///
    /// # Failures
    ///
    /// See `value_of`
    pub fn optional_value_of<T: FromStr>(&self, opt_name: &str) -> Result<Option<T>, ArgsError> {
        if self.has_value(opt_name) {
            Ok(Some(self.value_of::<T>(opt_name)?))
        } else {
            Ok(None)
        }
    }

    /// Retrieves the value of the `Opt` identified by `opt_name`, casts it to
    /// the type specified by `T` and then runs all provided `Validation`s.
    ///
    /// # Failures
    ///
    /// Returns `Err(ArgsError)` if no `Opt` correspond to `opt_name`, if the value cannot
    /// be cast to type `T` or if any validation is considered invalid.
    pub fn validated_value_of<T>(&self, opt_name: &str, validations: &[Box<dyn Validation<T=T>>])
        -> Result<T, ArgsError> where T: FromStr {
        // If the value does not have an error, run validations
        self.value_of::<T>(opt_name).and_then(|value| {
            for validation in validations {
                // If any validations fail, break the loop and return the error
                if validation.is_invalid(&value) { return Err(validation.error(&value)); }
            }

            Ok(value)
        })
    }

    /// Retrieves the value for the `Opt` identified by `opt_name` and casts it to
    /// the type specified by `T`.
    ///
    /// # Failures
    ///
    /// Returns `Err(ArgsError)` if no `Opt` corresponds to `opt_name` or if the
    /// value cannot be cast to type `T`.
    pub fn value_of<T: FromStr>(&self, opt_name: &str) -> Result<T, ArgsError> {
        self.values.get(opt_name).ok_or(
            ArgsError::new(opt_name, "does not have a value")
        ).and_then(|value_string| {
            T::from_str(value_string).or(
                Err(ArgsError::new(opt_name, &format!("unable to parse '{}'", value_string)))
            )
        })
    }

    /// Retrieves a vector of values for the `Opt` identified by `opt_name` and
    /// casts each of them to the type specified by `T`.
    ///
    /// # Failures
    ///
    /// Returns `Err(ArgsError)` if no `Opt` corresponds to `opt_name` or if any
    /// of the values cannot be cast to type `T`.
    pub fn values_of<T: FromStr>(&self, opt_name: &str) -> Result<Vec<T>, ArgsError> {
        self.values.get(opt_name).ok_or(
            ArgsError::new(opt_name, "does not have a value")
        ).and_then(|values_str| {
            values_str.split(SEPARATOR).map(|value| {
                T::from_str(value).or(
                    Err(ArgsError::new(opt_name, &format!("unable to parse '{}'", value)))
                )
            }).collect()
        })
    }

    /// Retreives an `Option<Opt>` for the `Opt` identified by `opt_name`
    ///
    /// # Failures
    ///
    /// Returns `None` if no `Opt` corresponds to `opt_name`.
    pub fn get_option(&self, opt_name :&str) -> Option<Box<dyn Opt>> {
        self.opts.get(opt_name).map(|opt| opt.clone())
    }

    // Private instance methods
    fn register_opt(&mut self, opt: Box<dyn Opt>) {
        if !self.opt_names.contains(&opt.name()) {
            debug!("Registering {}", opt);
            opt.register(&mut self.options);
            self.opt_names.push(opt.name().to_string());
            self.opts.insert(opt.name().to_string(), opt);
        } else {
            warn!("{} is already registered, ignoring", opt.name());
        }
    }
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut display = String::new();
        display.push_str(&format!("{}\n{}",
            to_column("Args"), column_underline()));
        for (key, value) in self.values.clone() {
            display.push_str(&format!("\n{}\t{}",
                to_column(&key), to_column(&value)));
        }
        write!(f, "{}", display)
    }
}

// Private associated methods
fn column_underline() -> String {
    let mut underline = String::new();
    for _ in 0..COLUMN_WIDTH { underline.push_str("="); }
    underline
}

fn to_column(string: &str) -> String {
    let mut string = string.to_string();
    string = if string.len() > COLUMN_WIDTH {
        string.truncate(COLUMN_WIDTH- 3);
        format!("{}...", string)
    } else { string };
    let mut spaces = String::new();
    for _ in 0..(COLUMN_WIDTH - string.len()) { spaces.push_str(" "); }
    format!("{}{}", string, spaces)
}

