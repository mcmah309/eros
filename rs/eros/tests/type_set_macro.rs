use core::{any::TypeId, fmt, num::ParseIntError};

use eros::{ErrorUnion, IntoUnion, ReshapeUnion};

#[derive(Debug)]
struct TestError<const ID: usize>;

impl<const ID: usize> fmt::Display for TestError<ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {ID}")
    }
}

impl<const ID: usize> core::error::Error for TestError<ID> {}

eros::type_set! {
    // A forward reference and overlapping sets exercise recursive expansion.
    type AppErrors = ParseErrors || OtherErrors || ParseErrors;
    type ParseErrors = (fmt::Error, ParseIntError, fmt::Error);
    type OtherErrors = (TestError<0>,) || ParseErrors || (TestError<1>, TestError<0>);

    type Empty = ();
    type AlsoEmpty = Empty || ();
    type Singleton = Empty || (fmt::Error,) || (fmt::Error,);
    type Copy = Singleton;

    #[eros::error_enum(AppError)]
    #[eros::error_enum_ref(AppErrorRef)]
    #[eros::error_enum_kind(AppErrorKind)]
    #[derive(PartialEq, Eq)]
    type EnumErrors = ParseErrors || (fmt::Error,);

    #[cfg(any())]
    type Disabled = (DoesNotExist,);
}

#[test]
fn composed_aliases_are_exact_flat_tuples() {
    assert_eq!(
        TypeId::of::<AppErrors>(),
        TypeId::of::<(fmt::Error, ParseIntError, TestError<0>, TestError<1>)>()
    );
    assert_eq!(TypeId::of::<AlsoEmpty>(), TypeId::of::<()>());
    assert_eq!(TypeId::of::<Singleton>(), TypeId::of::<(fmt::Error,)>());
    assert_eq!(TypeId::of::<Copy>(), TypeId::of::<Singleton>());
}

#[test]
fn overlapping_sets_support_construction_widening_and_narrowing() {
    let result: eros::Result<(), ParseErrors> = Err(fmt::Error).union();
    let result: eros::Result<(), AppErrors> = result.widen();
    let error = result.unwrap_err().narrow::<fmt::Error, _>().unwrap();
    assert_eq!(error, fmt::Error);

    let error: ErrorUnion<AppErrors> = ErrorUnion::new(TestError::<0>);
    let remaining: ErrorUnion<(TestError<0>, TestError<1>)> =
        error.narrow::<ParseErrors, _>().unwrap_err();
    assert!(remaining.is_inner::<TestError<0>>());
}

#[test]
fn attributes_receive_the_expanded_tuple() {
    let error: ErrorUnion<EnumErrors> = ErrorUnion::new(fmt::Error);
    assert_eq!(AppErrorKind::from(&error), AppErrorKind::Fmt);
    assert!(matches!(AppErrorRef::from(&error), AppErrorRef::Fmt(_)));
    assert!(matches!(AppError::from(error), AppError::Fmt(_)));
}

#[test]
fn preserves_visibility_and_accepts_complex_member_types() {
    mod sets {
        eros::type_set! {
            /// A public alias assembled from a private definition.
            pub(super) type Public = Private || (core::fmt::Error,);
            type Private = (super::TestError<0>,);
        }
    }
    let _: ErrorUnion<sets::Public> = ErrorUnion::new(TestError::<0>);

    // Type syntax, including commas in generic arguments and qualified paths,
    // belongs to Rust. The macro preserves it when copying tuple members.
    trait Associated {
        type Error;
    }
    impl Associated for () {
        type Error = TestError<1>;
    }
    eros::type_set! {
        type GenericMembers = (
            core::result::Result<u8, u16>,
            <() as Associated>::Error,
        ) || (<() as Associated>::Error,);
    }
    assert_eq!(
        TypeId::of::<GenericMembers>(),
        TypeId::of::<(core::result::Result<u8, u16>, TestError<1>)>()
    );
}
