use eros::{ErrorUnion, OtherError};

fn misplaced() -> Option<ErrorUnion<(OtherError, std::fmt::Error)>> {
    None
}

fn repeated() -> Option<ErrorUnion<(OtherError, OtherError)>> {
    None
}

fn main() {}
