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
        fn assert_traits<T: fmt::Display + fmt::Debug + Send + Sync>() {}
        assert_traits::<ErrorUnion<E>>();
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
fn generic_code_can_construct_and_reshape_sets_with_public_bounds() {
    use eros::MsgError;
    use eros::type_set::{Contains, GroupNarrow, Narrow, SingleNarrow, SupersetOf, TupleForm};

    fn wrap<T: SendSyncError, E: TypeSet, I>(error: T) -> ErrorUnion<E>
    where
        E::Variants: Contains<T, I>,
    {
        ErrorUnion::new(error)
    }

    fn widen<E: TypeSet, Other: TypeSet, I>(error: ErrorUnion<E>) -> ErrorUnion<Other>
    where
        Other::Variants: SupersetOf<E::Variants, I>,
    {
        error.widen()
    }

    fn narrow<T: 'static, E: TypeSet, I>(
        error: ErrorUnion<E>,
    ) -> Result<T, ErrorUnion<<<E::Variants as Narrow<T, I>>::Remainder as TupleForm>::Tuple>>
    where
        E::Variants: Narrow<T, I>,
    {
        error.narrow::<T, SingleNarrow<I>>()
    }

    fn narrow_group<Other: TypeSet, E: TypeSet, I>(
        error: ErrorUnion<E>,
    ) -> Result<
        ErrorUnion<Other>,
        ErrorUnion<
            <<E::Variants as SupersetOf<Other::Variants, I>>::Remainder as TupleForm>::Tuple,
        >,
    >
    where
        E::Variants: SupersetOf<Other::Variants, I>,
    {
        error.narrow::<Other, GroupNarrow<I>>()
    }

    let error: ErrorUnion<(MsgError,)> = wrap(MsgError::from_static("root"));
    let error: ErrorUnion<(fmt::Error, MsgError)> = widen(error);
    let error = narrow::<fmt::Error, _, _>(error).unwrap_err();
    let error = narrow_group::<(MsgError,), _, _>(error).unwrap();
    assert_eq!(error.into_single().as_str(), "root");
}

#[test]
fn generic_dyn_error_roundtrip_supports_typed_and_erased_sets() {
    fn roundtrip<E: TypeSet>(error: ErrorUnion<E>) -> ErrorUnion<E> {
        let display = format!("{error}");
        let debug = format!("{error:?}");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let error = ErrorUnion::<E>::try_from_dyn_error(Box::new(error.into_std_error())).unwrap();
        assert_eq!(
            error.inner() as *const dyn SendSyncError as *const (),
            original
        );
        assert_eq!(format!("{error}"), display);
        assert_eq!(format!("{error:?}"), debug);
        error
    }

    let typed: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
    let typed = roundtrip(typed.context("typed context"));
    assert_eq!(typed.into_single(), fmt::Error);

    let erased = roundtrip(eros::error!("root").context("erased context"));
    assert_eq!(
        erased.downcast_inner::<eros::MsgError>().unwrap().as_str(),
        "root"
    );

    let unrelated: Box<dyn SendSyncError> = Box::new(eros::MsgError::from_static("unrelated"));
    let original = unrelated.as_ref() as *const dyn SendSyncError as *const ();
    let unrelated = ErrorUnion::<AnyError>::try_from_dyn_error(unrelated).unwrap_err();
    assert_eq!(
        unrelated.as_ref() as *const dyn SendSyncError as *const (),
        original
    );
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

    let error: ErrorUnion<AllErrors> = ErrorUnion::new(TestError::<25>);
    let selected: Result<ErrorUnion<AllErrors>, ErrorUnion<()>> =
        error.narrow::<AllErrors, _>();
    assert!(matches!(selected.unwrap().as_enum(), E26::Z(_)));
}
