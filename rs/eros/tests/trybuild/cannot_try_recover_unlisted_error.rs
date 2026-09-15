use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error,)> = Err(ErrorUnion::new(std::fmt::Error));
    let _: eros::Result<(), (std::fmt::Error,)> =
        result.try_recover::<MsgError, _, _, _>(|_| Ok(()));
}
