use eros::{AnyError, E26, ErrorUnion, SendSyncError, TypeSet};
use std::fmt;

#[derive(Debug)]
struct TestError<const N: u8>;

impl<const N: u8> fmt::Display for TestError<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {N}")
    }
}

impl<const N: u8> std::error::Error for TestError<N> {}

type AllErrors = (
    TestError<0>,
    TestError<1>,
    TestError<2>,
    TestError<3>,
    TestError<4>,
    TestError<5>,
    TestError<6>,
    TestError<7>,
    TestError<8>,
    TestError<9>,
    TestError<10>,
    TestError<11>,
    TestError<12>,
    TestError<13>,
    TestError<14>,
    TestError<15>,
    TestError<16>,
    TestError<17>,
    TestError<18>,
    TestError<19>,
    TestError<20>,
    TestError<21>,
    TestError<22>,
    TestError<23>,
    TestError<24>,
    TestError<25>,
);

#[test]
fn error_sets_support_generic_and_empty_unions() {
    fn assert_union<E: TypeSet>() {
        let _: Option<ErrorUnion<E>> = None;
    }

    fn singleton<E: SendSyncError>(error: E) -> ErrorUnion<(E,)> {
        ErrorUnion::new(error)
    }

    assert_union::<()>();
    assert_union::<AnyError>();
    assert_union::<(fmt::Error,)>();

    let error = singleton(fmt::Error).narrow::<fmt::Error, _>().unwrap();
    let erased: ErrorUnion = singleton(error).into();
    assert!(erased.is_inner::<fmt::Error>());
}

#[test]
fn largest_error_set_supports_enum_conversions_and_reshaping() {
    let error: ErrorUnion<(TestError<25>,)> = ErrorUnion::new(TestError::<25>);
    let mut error: ErrorUnion<AllErrors> = error.widen();
    assert!(matches!(error.as_enum(), E26::Z(_)));
    assert!(matches!(error.as_mut_enum(), E26::Z(_)));
    assert!(matches!(error.into_enum(), E26::Z(_)));

    let error: ErrorUnion<AllErrors> = ErrorUnion::new(TestError::<25>);
    let remainder = error.narrow::<TestError<0>, _>().unwrap_err();
    assert!(remainder.narrow::<TestError<25>, _>().is_ok());
}
