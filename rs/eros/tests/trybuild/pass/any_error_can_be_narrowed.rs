use eros::{ErrorUnion, MsgError, ReshapeUnion};
use std::fmt;

fn main() {
    let error: ErrorUnion = ErrorUnion::new(fmt::Error);
    let _: Result<fmt::Error, ErrorUnion> = error.narrow::<fmt::Error, _>();

    let error: ErrorUnion = ErrorUnion::new(fmt::Error);
    let _: Result<ErrorUnion<(MsgError, fmt::Error)>, ErrorUnion> =
        error.narrow::<(MsgError, fmt::Error), _>();

    let result: eros::Result<()> = Ok(());
    let _: eros::Result<()> = result.recover::<MsgError, _>(|_| ());

    let result: eros::Result<()> = Ok(());
    let _: eros::Result<()> = result.try_recover::<MsgError, _, _, _>(|_| Ok(()));
}
