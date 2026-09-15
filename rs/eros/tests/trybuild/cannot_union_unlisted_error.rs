use eros::{IntoUnion, MsgError};

fn main() {
    let result: Result<(), MsgError> = Err(MsgError::from("unlisted"));
    let _: eros::Result<(), (std::fmt::Error,)> = result.union();
}
