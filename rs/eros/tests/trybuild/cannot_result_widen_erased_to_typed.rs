use eros::{MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<()> = Ok(());
    let _: eros::Result<(), (MsgError,)> = result.widen();
}
