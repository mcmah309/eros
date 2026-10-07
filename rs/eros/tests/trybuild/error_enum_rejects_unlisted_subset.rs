use eros::{ErrorUnion, MsgError};
use std::{fmt, io};

#[eros::error_enums(AppError)]
type App = (fmt::Error, MsgError);

fn main() {
    // The inner error matches, but the source set also permits an unlisted type.
    let mut union: ErrorUnion<(fmt::Error, io::Error)> = ErrorUnion::new(fmt::Error);
    let _: AppErrorRef<'_> = (&union).into();
    let _: AppErrorMut<'_> = (&mut union).into();
    let _: AppError = union.into();
}
