use eros::{AnyError, ErrorUnion};

fn owned(error: ErrorUnion) -> AnyError {
    error.into_enum()
}

fn borrowed(error: &ErrorUnion) -> &AnyError {
    error.as_enum()
}

fn mutable(error: &mut ErrorUnion) -> &mut AnyError {
    error.as_mut_enum()
}

fn narrow(error: ErrorUnion) {
    let _ = error.narrow::<AnyError, _>();
}

fn singleton_marker_is_not_an_error_set() -> Option<ErrorUnion<(AnyError,)>> {
    None
}

fn main() {}
