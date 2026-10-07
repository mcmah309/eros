#![no_implicit_prelude]

extern crate eros;

#[eros::error_enum(CustomError)]
#[non_exhaustive]
pub type DefaultDisplay = (::core::fmt::Error, eros::MsgError);

#[eros::error_enum(FormattedError, "operation failed: {0}")]
pub type CustomDisplay = (::core::fmt::Error,);

#[eros::error_enum(r#type)]
#[allow(non_camel_case_types)]
pub type RawName = (::core::fmt::Error,);

// The chosen enum name must not be shadowed by a payload parameter.
#[eros::error_enum(E0)]
pub type ParameterName = (::core::fmt::Error,);

fn main() {
    let mut error: eros::ErrorUnion<DefaultDisplay> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: CustomError<&::core::fmt::Error, &eros::MsgError> = ::core::convert::From::from(&error);
    let _: CustomError<&mut ::core::fmt::Error, &mut eros::MsgError> =
        ::core::convert::From::from(&mut error);
    let _: CustomError = ::core::convert::From::from(error);

    let error: eros::ErrorUnion<CustomDisplay> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FormattedError = ::core::convert::From::from(error);
    let error: eros::ErrorUnion<RawName> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: r#type = ::core::convert::From::from(error);
    let error: eros::ErrorUnion<ParameterName> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: E0 = ::core::convert::From::from(error);
}
