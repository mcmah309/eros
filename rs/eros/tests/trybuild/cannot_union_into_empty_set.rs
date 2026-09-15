use eros::{IntoUnion, MsgError};

fn main() {
    let result: Result<(), MsgError> = Ok(());
    let _: eros::Result<(), ()> = result.union();
}
