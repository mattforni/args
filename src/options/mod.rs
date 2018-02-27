use getopts::{HasArg,Matches,Occur,Options};
use std::fmt::{self,Debug,Display,Error,Formatter};

#[cfg(test)] mod tst;

use super::SEPARATOR as SEPARATOR;

macro_rules! unsupported {
    ( $str:expr ) => ( panic!("{} is not supported yet", $str) );
}

pub fn new(short_name: &str,
        long_name: &str,
        desc: &str,
        hint: &str,
        has_arg: HasArg,
        occur: Occur,
        default: Option<String>) -> Box<Opt> {
    if has_arg == HasArg::Maybe { unsupported!("HasArg::Maybe"); }

    if occur != Occur::Multi {
        Box::new(Single::new(short_name, long_name, desc, hint, has_arg, occur, default))
    } else {
        Box::new(Multi::new(short_name, long_name, desc, hint))
    }
}

pub trait Opt: Send {
    fn flag(&self) -> String;
    fn is_multi(&self) -> bool;
    fn is_required(&self) -> bool;
    fn name(&self) -> String;
    fn parse(&self, matches: &Matches) -> Option<String>;
    fn register(&self, options: &mut Options);
    fn box_clone(&self) -> Box<Opt>;
}

impl Clone for Box<Opt>
{
    fn clone(&self) -> Box<Opt> {
        self.box_clone()
    }
}

#[derive(Clone)]
struct Multi {
    short_name: String,
    long_name: String,
    desc: String,
    hint: String,
}

impl Multi {
    fn new(short_name: &str,
            long_name: &str,
            desc: &str,
            hint: &str) -> Self {
        Multi {
            short_name: short_name.to_string(),
            long_name: long_name.to_string(),
            desc: desc.to_string(),
            hint: hint.to_string(),
        }
    }
}

impl Opt for Multi {
    fn flag(&self) -> String {
        self.short_name.to_string()
    }

    fn is_required(&self) -> bool {
        false
    }

    fn is_multi(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        self.long_name.to_string()
    }

    fn parse(&self, matches: &Matches) -> Option<String> {
        let strs = matches.opt_strs(&self.long_name);
        if strs.is_empty() { None } else { Some(strs.join(SEPARATOR)) }
    }

    fn register(&self, options: &mut Options) {
        options.optmulti(&self.short_name,
            &self.long_name,
            &self.desc,
            &self.hint);
    }

    fn box_clone(&self) -> Box<Opt> {
        Box::new((*self).clone())
    }
}

#[derive(Clone)]
struct Single {
    short_name: String,
    long_name: String,
    desc: String,
    hint: String,
    has_arg: HasArg,
    occur: Occur,
    default: Option<String>
}

impl Single {
    fn new(short_name: &str,
            long_name: &str,
            desc: &str,
            hint: &str,
            has_arg: HasArg,
            occur: Occur,
            default: Option<String>) -> Self {
        // If there is a default occurence becomes optional
        let occur = if default.is_some() { Occur::Optional } else { occur };

        Single {
            short_name: short_name.to_string(),
            long_name: long_name.to_string(),
            desc: desc.to_string(),
            hint: hint.to_string(),
            has_arg: has_arg,
            occur: occur,
            default: default
        }
    }
}

impl Opt for Single {
    fn flag(&self) -> String {
        self.short_name.to_string()
    }

    fn is_required(&self) -> bool {
        self.occur == Occur::Req
    }

    fn is_multi(&self) -> bool {
        false
    }

    fn name(&self) -> String {
        self.long_name.to_string()
    }

    fn parse(&self, matches: &Matches) -> Option<String> {
        // If the option does not have an argument, return presence
        if self.has_arg == HasArg::No {
            return Some(matches.opt_present(&self.long_name).to_string());
        }

        // If the option does have an arugment, parse it or get the default
        matches.opt_str(&self.long_name).or_else(|| {
            // Return the default if it is defined and there is no match
            if self.default.is_some() { return self.default.clone(); }
            None
        })
    }

    fn register(&self, options: &mut Options) {
        options.opt(&self.short_name,
            &self.long_name,
            &self.desc,
            &self.hint,
            self.has_arg,
            self.occur);
    }

    fn box_clone(&self) -> Box<Opt> {
        Box::new((*self).clone())
    }
}

impl Display for Opt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "option '-{} --{}'", self.flag(), self.name())
    }
}

impl Debug for Opt {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "option '-{} --{}'", self.flag(), self.name())
    }
}
