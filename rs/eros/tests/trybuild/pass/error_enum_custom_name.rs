#![no_implicit_prelude]

extern crate eros;

#[eros::error_enum(CustomError)]
#[non_exhaustive]
#[eros::error_enum_ref(CustomErrorRef)]
#[eros::error_enum_mut(CustomErrorMut)]
pub type DefaultDisplay = (::core::fmt::Error, eros::MsgError);

#[eros::error_enum(FormattedError, "operation failed: {0}")]
pub type CustomDisplay = (::core::fmt::Error,);

#[eros::error_enum(r#type)]
#[allow(non_camel_case_types)]
pub type RawName = (::core::fmt::Error,);

// A name matching the former payload parameters remains valid.
#[eros::error_enum(E0)]
pub type ParameterName = (::core::fmt::Error,);

// Unrequested borrowed names do not conflict with the tuple alias.
#[eros::error_enum(SharedError)]
type SharedErrorRef = (::core::fmt::Error,);

#[eros::error_enum(MutableError)]
type MutableErrorMut = (::core::fmt::Error,);

fn main() {
    let mut error: eros::ErrorUnion<DefaultDisplay> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: CustomErrorRef<'_> = ::core::convert::From::from(&error);
    let _: CustomErrorMut<'_> = ::core::convert::From::from(&mut error);
    let _: CustomError = ::core::convert::From::from(error);

    let error: eros::ErrorUnion<CustomDisplay> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FormattedError = ::core::convert::From::from(error);
    let error: eros::ErrorUnion<RawName> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: r#type = ::core::convert::From::from(error);
    let error: eros::ErrorUnion<ParameterName> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: E0 = ::core::convert::From::from(error);

    let error: eros::ErrorUnion<SharedErrorRef> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: SharedError = ::core::convert::From::from(error);
    let error: eros::ErrorUnion<MutableErrorMut> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: MutableError = ::core::convert::From::from(error);
}
