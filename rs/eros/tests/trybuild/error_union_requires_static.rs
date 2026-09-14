use eros::ErrorUnion;
use std::fmt;

#[derive(Debug)]
struct BorrowedError<'a>(&'a str);

impl fmt::Display for BorrowedError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for BorrowedError<'_> {}

struct Invalid<'a> {
    error: Option<ErrorUnion<(std::io::Error, BorrowedError<'a>)>>,
}

fn main() {}
