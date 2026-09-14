use eros::{ErrorUnion, StrError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error, StrError)> = ErrorUnion::new(std::fmt::Error);
    let _: ErrorUnion<(std::fmt::Error,)> = error.widen();
}
