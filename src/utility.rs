use std::fmt::Arguments;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn eprintln_error_impl(args: &Arguments) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    let supports_color = stderr.supports_color();

    if supports_color {
        let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
    }

    writeln!(stderr, "{}", args).unwrap();

    if supports_color {
        let _ = stderr.reset();
    }
}

macro_rules! eprintln_error {
    ($fmt: expr) => {
        crate::utility::eprintln_error_impl(&std::format_args!($fmt))
    };

    ($fmt: expr, $($args: tt), *) => {
        crate::utility::eprintln_error_impl(&std::format_args!($fmt, $($args),*))
    };
}

pub(crate) use eprintln_error;
