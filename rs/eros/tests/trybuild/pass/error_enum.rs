#![no_implicit_prelude]

extern crate eros;

#[eros::error_enum(NamedError, "{0}")]
#[non_exhaustive]
#[eros::error_enum_ref(NamedErrorRef, "{0}")]
#[derive(::core::clone::Clone, ::core::marker::Copy)]
#[doc = "Shared view"]
#[eros::error_enum_mut(NamedErrorMut, "{0}")]
#[non_exhaustive]
#[doc = "Mutable view"]
pub type Named = (::core::fmt::Error, eros::MsgError);

fn main() {
    let mut error: eros::ErrorUnion<Named> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: NamedErrorRef<'_> = ::core::convert::From::from(&error);
    let _: NamedErrorMut<'_> = ::core::convert::From::from(&mut error);
    let _: NamedError = ::core::convert::From::from(error);
}
