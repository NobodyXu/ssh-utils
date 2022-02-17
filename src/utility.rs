use std::cell::Cell;
use std::fmt::Arguments;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
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

#[repr(transparent)]
pub(crate) struct BorrowCell<T>(Cell<T>);

impl<T> BorrowCell<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self(Cell::new(value))
    }

    pub(crate) fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: Default> BorrowCell<T> {
    pub(crate) fn borrow(&self) -> BorrowedCell<'_, T> {
        BorrowedCell(self, ManuallyDrop::new(self.0.take()))
    }
}

pub(crate) struct BorrowedCell<'a, T>(&'a BorrowCell<T>, ManuallyDrop<T>);

impl<T> Deref for BorrowedCell<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.1
    }
}

impl<T> DerefMut for BorrowedCell<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.1
    }
}

impl<T> Drop for BorrowedCell<'_, T> {
    fn drop(&mut self) {
        self.0 .0.set(unsafe { ManuallyDrop::take(&mut self.1) });
    }
}
