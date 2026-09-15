use eros::ErrorUnion;
use std::{fmt, string::FromUtf8Error};

struct Callback<T>(T);

impl<T> fmt::Debug for Callback<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("callback")
    }
}

impl<T> fmt::Display for Callback<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("callback")
    }
}

impl<T> std::error::Error for Callback<T> {}

// A function accepting any lifetime can normally coerce to one accepting only
// 'static, but that changes its TypeId. ErrorUnion must forbid that coercion:
// into_enum() would miss the stored Callback and read it as FromUtf8Error.
fn coerce(
    error: ErrorUnion<(Callback<fn(&())>, FromUtf8Error)>,
) -> ErrorUnion<(Callback<fn(&'static ())>, FromUtf8Error)> {
    error
}

fn main() {}
