#![allow(dead_code, clippy::panic)]
use crate::prelude::*;
use std::fmt::Debug;

pub trait AssertOkDebug<T, E: Debug> {
    fn assert_ok_debug(self) -> T;
}

pub trait AssertOkDisplay<T, E: Display> {
    fn assert_ok(self) -> T;
}

pub trait AssertErrDebug<T: Debug, E> {
    fn assert_err_debug(self) -> E;
}

pub trait AssertErrDisplay<T: Display, E> {
    fn assert_err(self) -> E;
}

impl<T, E: Debug> AssertOkDebug<T, E> for Result<T, E> {
    fn assert_ok_debug(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => panic!("Result was an error:\n{e:?}"),
        }
    }
}

impl<T, E: Display> AssertOkDisplay<T, E> for Result<T, E> {
    fn assert_ok(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => panic!("Result was an error:\n{e}"),
        }
    }
}

impl<T: Debug, E> AssertErrDebug<T, E> for Result<T, E> {
    fn assert_err_debug(self) -> E {
        match self {
            Ok(value) => panic!("Result was not an error:\n{value:?}"),
            Err(e) => e,
        }
    }
}

impl<T: Display, E> AssertErrDisplay<T, E> for Result<T, E> {
    fn assert_err(self) -> E {
        match self {
            Ok(value) => panic!("Result was not an error:\n{value}"),
            Err(e) => e,
        }
    }
}
