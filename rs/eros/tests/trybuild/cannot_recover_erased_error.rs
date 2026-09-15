use eros::{MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<()> = Ok(());
    let _ = result.recover::<MsgError, _>(|_| ());
}
