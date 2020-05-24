//! A module containing traits designed to provide args-based convenience methods.
//!
//! # Example
//!
//! Using `HasArgs` and `HasParsedArgs` in conjunction, the original example can
//! be re-written like so:
//!
//! ```rust
//! extern crate args;
//!
//! use args::{Args,ArgsError};
//! use args::traits::{HasArgs,HasParsedArgs};
//! use args::validations::{Order,OrderValidation};
//! use args::getopts::Occur;
//! use std::process::exit;
//!
//! const PROGRAM_DESC: &'static str = "Run this program";
//! const PROGRAM_NAME: &'static str = "program";
//!
//! struct Program { parsed_args: Args }
//!
//! impl Program {
//!     pub fn new() -> Result<Self, ArgsError> {
//!         let mut args = Self::args();
//!         try!(args.parse(vec!("-i", "5")));
//!         Ok(Program { parsed_args: args })
//!     }
//!
//!     pub fn run(&self) -> Result<(), ArgsError> {
//!         if try!(self.value_of("help")) {
//!             println!("{}", Self::full_usage());
//!             return Ok(());
//!         }
//!
//!         let gt_0 = Box::new(OrderValidation::new(Order::GreaterThan, 0u32));
//!         let lt_10 = Box::new(OrderValidation::new(Order::LessThanOrEqual, 10u32));
//!
//!         let iters = try!(self.validated_value_of("iter", &[gt_0, lt_10]));
//!         for iter in 0..iters {
//!             println!("Working on iteration {}", iter);
//!         }
//!         println!("All done!");
//!
//!         Ok(())
//!     }
//! }
//!
//! impl HasArgs for Program {
//!     fn args() -> Args {
//!         let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
//!         args.flag("h", "help", "Print the usage menu");
//!         args.option("i",
//!             "iter",
//!             "The number of times to run this program",
//!             "TIMES",
//!             Occur::Req,
//!             None);
//!         args.option("l",
//!             "log_file",
//!             "The name of the log file",
//!             "NAME",
//!             Occur::Optional,
//!             None);
//!
//!         args
//!     }
//! }
//!
//! impl HasParsedArgs for Program {
//!     fn parsed_args(&self) -> &Args { &self.parsed_args }
//! }
//!
//! fn main() {
//!     match Program::new().map(|program| { program.run() }) {
//!         Ok(_) => println!("Program ran successfully"),
//!         Err(error) => {
//!             println!("{}", error);
//!             exit(1);
//!         }
//!     }
//! }
//! ```

pub use self::has_args::HasArgs;
pub use self::has_parsed_args::HasParsedArgs;

mod has_args;
mod has_parsed_args;

