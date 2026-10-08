#![no_implicit_prelude]

extern crate eros;

#[eros::error_enum_kind(FailureKind)]
#[non_exhaustive]
#[repr(align(32))]
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq)]
#[eros::error_enum(Failure)]
#[eros::error_enum_ref(FailureRef)]
#[eros::error_enum_mut(FailureMut)]
pub type Errors = (::core::fmt::Error,);

#[eros::error_enum_kind(r#type)]
#[allow(non_camel_case_types)]
type RawErrors = (::core::fmt::Error,);

fn main() {
    let mut union: eros::ErrorUnion<Errors> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FailureKind = ::core::convert::From::from(&union);
    let _: FailureKind = ::core::convert::From::from(&mut union);
    let _: FailureKind = ::core::convert::From::from(union);

    let mut union: eros::ErrorUnion<RawErrors> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: r#type = ::core::convert::From::from(&union);
    let _: r#type = ::core::convert::From::from(&mut union);
    let _: r#type = ::core::convert::From::from(union);

    let mut union: eros::ErrorUnion<eros::AnyError> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FailureKind = ::core::convert::TryFrom::try_from(&union).unwrap();
    let _: FailureKind = ::core::convert::TryFrom::try_from(&mut union).unwrap();
    let _: FailureKind = ::core::convert::TryFrom::try_from(union).unwrap();
}
