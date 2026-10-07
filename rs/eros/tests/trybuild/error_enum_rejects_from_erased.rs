use eros::{AnyError, ErrorUnion};
use std::fmt;

#[eros::error_enums(AppError)]
type App = (fmt::Error,);

fn main() {
    // An erased error requires TryFrom even when its inner type matches.
    let mut union: ErrorUnion<AnyError> = ErrorUnion::new(fmt::Error);
    let _: AppErrorRef<'_> = (&union).into();
    let _: AppErrorMut<'_> = (&mut union).into();
    let _: AppError = union.into();
}
