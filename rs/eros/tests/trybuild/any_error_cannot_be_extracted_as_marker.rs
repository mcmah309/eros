use eros::{AnyError, ErrorUnion};

fn narrow(error: ErrorUnion) -> AnyError {
    error.narrow::<AnyError, _>().unwrap()
}

fn singleton_marker_is_not_an_error_set() -> Option<ErrorUnion<(AnyError,)>> {
    None
}

fn main() {}
