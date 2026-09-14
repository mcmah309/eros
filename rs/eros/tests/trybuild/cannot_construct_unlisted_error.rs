use eros::{ErrorUnion, StrError};

fn main() {
    let _: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(StrError::from("unlisted"));
}
