# args

A dead simple implementation of command line argument parsing and validation
built on top of the [getopts](https://crates.io/crates/getopts) crate.

In order to use the `args` crate simply create an `Args` object and begin
registering possible command line options via the `flag(...)` and `option(...)`
methods. Once all options have been registered, parse arguments directly from the
command line, or provide a vector of your own arguments.

Any errors encountered during parsing will be returned wrapped in an `ArgsError`.
If there are no errors during parsing values may be retrieved from the `args`
object by simply calling `value_of(...)` and `validated_value_of(...)`.

That's it!

## Usage

This crate is [on crates.io](https://crates.io/crates/args) and can be
used by adding `args` to the dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
args = "2.0"
```

and this to your crate root:

```rust
extern crate args;
```

## Example

The following example shows simple command line parsing for an application that
requires a number of iterations between zero *(0)* and ten *(10)* to be specified,
accepts an optional log file name and responds to the help flag.

```rust
extern crate args;
extern crate getopts;

use getopts::Occur;
use std::process::exit;

use args::{Args,ArgsError};
use args::validations::{Order,OrderValidation};

const PROGRAM_DESC: &'static str = "Run this program";
const PROGRAM_NAME: &'static str = "program";

fn main() {
    match parse(&vec!("-i", "5")) {
        Ok(_) => println!("Successfully parsed args"),
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };
}

fn parse(input: &Vec<&str>) -> Result<(), ArgsError> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
    args.option("i",
        "iter",
        "The number of times to run this program",
        "TIMES",
        Occur::Req,
        None);
    args.option("l",
        "log_file",
        "The name of the log file",
        "NAME",
        Occur::Optional,
        Some(String::from("output.log")));

    try!(args.parse(input));

    let help = try!(args.value_of("help"));
    if help {
        println!("{}", args.full_usage());
        return Ok(());
    }

    let gt_0 = Box::new(OrderValidation::new(Order::GreaterThan, 0u32));
    let lt_10 = Box::new(OrderValidation::new(Order::LessThanOrEqual, 10u32));

    let iters = try!(args.validated_value_of("iter", &[gt_0, lt_10]));
    for iter in 0..iters {
        println!("Working on iteration {}", iter);
    }

    let optional_val = try!(Self::optional_value_of::<String>(&args, SERVICE));
    if let Some(val) = optional_val {
        // val is `Some`
    } else {
        // val is `None`
    }

    println!("All done.");

    Ok(())
}
```

