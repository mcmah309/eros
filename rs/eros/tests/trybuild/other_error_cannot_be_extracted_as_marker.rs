use eros::{ErrorUnion, OtherError};

fn extract(error: ErrorUnion<(OtherError,)>) -> OtherError {
    error.narrow::<OtherError, _>().unwrap()
}

fn singleton(error: ErrorUnion<(OtherError,)>) -> OtherError {
    error.into_single()
}

fn main() {}
