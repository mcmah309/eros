use eros::{MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::fmt::Error)> = Ok(());
    let _: eros::Result<(), (MsgError,)> = result.widen();
}
