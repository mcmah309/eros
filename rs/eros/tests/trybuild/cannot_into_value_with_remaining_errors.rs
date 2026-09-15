use eros::{MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error, MsgError)> = Ok(());
    let _ = result.recover::<MsgError, _>(|_| ()).into_value();
}
