use eros::{ErrorUnion, StrError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(std::fmt::Error);
    let _ = error.subset::<(StrError,), _>();
}
