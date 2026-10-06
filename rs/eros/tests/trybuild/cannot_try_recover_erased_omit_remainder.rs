use eros::{MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<()> = Ok(());
    let _: eros::Result<(), ()> = result.try_recover::<MsgError, _, _, _>(|_| Ok(()));
}
