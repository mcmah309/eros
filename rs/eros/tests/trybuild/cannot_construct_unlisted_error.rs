use eros::{ErrorUnion, MsgError};

fn main() {
    let _: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(MsgError::from("unlisted"));
}
