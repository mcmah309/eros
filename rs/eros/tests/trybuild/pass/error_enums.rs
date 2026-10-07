#![no_implicit_prelude]

extern crate eros;

#[eros::error_enums(Failure, "operation failed: {0}")]
#[non_exhaustive]
#[repr(align(32))]
#[derive(::core::cmp::PartialEq, ::core::cmp::Eq)]
pub type Errors = (::core::fmt::Error,);

#[eros::error_enums(r#type)]
#[allow(non_camel_case_types)]
type RawErrors = (::core::fmt::Error,);

fn main() {
    let mut union: eros::ErrorUnion<Errors> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FailureRef<'_> = ::core::convert::From::from(&union);
    let _: FailureMut<'_> = ::core::convert::From::from(&mut union);
    let _: Failure = ::core::convert::From::from(union);

    let mut union: eros::ErrorUnion<RawErrors> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: typeRef<'_> = ::core::convert::From::from(&union);
    let _: typeMut<'_> = ::core::convert::From::from(&mut union);
    let _: r#type = ::core::convert::From::from(union);

    let mut union: eros::ErrorUnion<eros::AnyError> = eros::ErrorUnion::new(::core::fmt::Error);
    let _: FailureRef<'_> = ::core::convert::TryFrom::try_from(&union).unwrap();
    let _: FailureMut<'_> = ::core::convert::TryFrom::try_from(&mut union).unwrap();
    let _: Failure = ::core::convert::TryFrom::try_from(union).unwrap();
}
