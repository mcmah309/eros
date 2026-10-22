use eros::ErrorUnion;
use std::{fmt, io};

#[eros::error_enum_kind(AppKind)]
type App = (fmt::Error,);

fn main() {
    let mut union: ErrorUnion<(fmt::Error, io::Error)> = ErrorUnion::new(fmt::Error);
    let _: AppKind = (&union).into();
    let _: AppKind = (&mut union).into();
    let _: AppKind = union.into();
}
