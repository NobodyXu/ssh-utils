use clap_verbosity_flag::Verbosity;
use owo_colors::{
    OwoColorize,
    Stream::{Stderr, Stdout},
};
use std::fmt::Arguments;

pub use log::Level;

pub fn eprintln_error_impl(args: &Arguments<'_>) {
    eprintln!("{}", args.if_supports_color(Stderr, |args| args.red()));
}

macro_rules! eprintln_error {
    ($fmt: expr) => {
        crate::utility::eprintln_error_impl(&std::format_args!($fmt))
    };

    ($fmt: expr, $($args: expr), *) => {
        crate::utility::eprintln_error_impl(&std::format_args!($fmt, $($args),*))
    };
}

pub(crate) use eprintln_error;

pub trait PrintBasedOnVerbosity {
    /// Print to stdout only if the current log level is higher than `level`.
    fn print(&self, level: Level, args: &Arguments<'_>);

    /// Println to stdout only if the current log level is higher than `level`.
    fn println(&self, level: Level, args: &Arguments<'_>);

    /// Print to stdout only if the current log level is not quiet.
    fn print_if_not_quiet(&self, args: &Arguments<'_>);

    /// Println to stdout only if the current log level is not quiet.
    fn println_if_not_quiet(&self, args: &Arguments<'_>);
}

impl PrintBasedOnVerbosity for Verbosity {
    fn print(&self, level: Level, args: &Arguments<'_>) {
        match self.log_level() {
            Some(curr_level) if curr_level >= level => {
                if level == Level::Error {
                    eprintln_error_impl(args)
                } else if level == Level::Warn {
                    print!("{}", args.if_supports_color(Stdout, |args| args.yellow()));
                } else {
                    print!("{}", args);
                }
            }
            _ => (),
        }
    }

    fn println(&self, level: Level, args: &Arguments<'_>) {
        self.print(level, &std::format_args!("{args}\n"))
    }

    fn print_if_not_quiet(&self, args: &Arguments<'_>) {
        if self.log_level().is_some() {
            print!("{}", args);
        }
    }

    fn println_if_not_quiet(&self, args: &Arguments<'_>) {
        self.print_if_not_quiet(&std::format_args!("{args}\n"))
    }
}

macro_rules! println_on_level {
    ($verbosity:expr, $level:expr, $fmt: expr) => {
        crate::utility::PrintBasedOnVerbosity::println(&$verbosity, $level, &std::format_args!($fmt))
    };

    ($verbosity:expr, $level:expr, $fmt: expr, $($args: expr), *) => {
        crate::utility::PrintBasedOnVerbosity::println(&$verbosity, $level, &std::format_args!($fmt, $($args),*))
    };
}

pub(crate) use println_on_level;

macro_rules! println_if_not_quiet {
    ($verbosity:expr, $fmt: expr) => {
        crate::utility::PrintBasedOnVerbosity::println_if_not_quiet(&$verbosity, &std::format_args!($fmt))
    };

    ($verbosity:expr, $fmt: expr, $($args: expr), *) => {
        crate::utility::PrintBasedOnVerbosity::println_if_not_quiet(&$verbosity, &std::format_args!($fmt, $($args),*))
    };
}

pub(crate) use println_if_not_quiet;
