use eros::{ErrorUnion, OtherError};

fn close(error: ErrorUnion<(std::fmt::Error, OtherError)>) -> ErrorUnion<(std::fmt::Error,)> {
    error.widen()
}

fn retype_other(
    error: ErrorUnion<(std::fmt::Error, OtherError)>,
) -> ErrorUnion<(std::fmt::Error,)> {
    error.narrow::<OtherError, _>().unwrap().widen()
}

fn main() {}
