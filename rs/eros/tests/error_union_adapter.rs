use core::{fmt, num::ParseIntError};
use eros::{ErrorUnion, ErrorUnionAdapter, IntoUnion, ReshapeUnion, TypeSet};

#[test]
fn union_question_mark_infers_typed_and_erased_error_sets() {
    fn typed<T>(
        result: Result<T, fmt::Error>,
    ) -> Result<T, ErrorUnionAdapter<(fmt::Error, ParseIntError)>> {
        Ok(result.union()?)
    }

    fn erased<T>(result: Result<T, fmt::Error>) -> Result<T, ErrorUnionAdapter> {
        Ok(result.union()?)
    }

    let error = typed(Err::<(), _>(fmt::Error)).unwrap_err();
    let _: ErrorUnion<(fmt::Error, ParseIntError)> = error.into_unadapted();
    let error = erased(Err::<(), _>(fmt::Error)).unwrap_err();
    assert!(error.into_unadapted().is_inner::<fmt::Error>());

    let mut value = 7;
    *typed(Ok(&mut value)).unwrap() += 1;
    *erased(Ok(&mut value)).unwrap() += 1;
    assert_eq!(value, 9);
}

#[test]
fn widen_question_mark_infers_larger_and_erased_error_sets() {
    fn typed<T>(
        result: eros::Result<T, (fmt::Error,)>,
    ) -> Result<T, ErrorUnionAdapter<(ParseIntError, fmt::Error)>> {
        Ok(result.widen()?)
    }

    fn erased<T>(result: eros::Result<T, (fmt::Error,)>) -> Result<T, ErrorUnionAdapter> {
        Ok(result.widen()?)
    }

    let error = typed(Err::<(), _>(ErrorUnion::new(fmt::Error))).unwrap_err();
    let error: ErrorUnion<(ParseIntError, fmt::Error)> = error.into_unadapted();
    assert!(error.is_inner::<fmt::Error>());
    let error = erased(Err::<(), _>(ErrorUnion::new(fmt::Error))).unwrap_err();
    assert!(error.into_unadapted().is_inner::<fmt::Error>());

    assert_eq!(typed(Ok(7)).unwrap(), 7);
    assert_eq!(erased(Ok(8)).unwrap(), 8);
}

#[test]
fn question_mark_converts_matching_unions_without_reshaping() {
    fn propagate<T, E: TypeSet>(result: eros::Result<T, E>) -> Result<T, ErrorUnionAdapter<E>> {
        Ok(result?)
    }

    let typed: eros::Result<(), (fmt::Error,)> = Err(ErrorUnion::new(fmt::Error));
    let error: ErrorUnionAdapter<(fmt::Error,)> = propagate(typed).unwrap_err();
    let _: fmt::Error = error.into_unadapted().into_single();

    let erased: eros::Result<()> = Err(ErrorUnion::new(fmt::Error));
    let error: ErrorUnionAdapter = propagate(erased).unwrap_err();
    assert!(error.into_unadapted().is_inner::<fmt::Error>());

    let empty: eros::Result<_, ()> = Ok(7);
    assert_eq!(propagate(empty).unwrap(), 7);
}

#[test]
fn borrowing_and_recovery_preserve_typed_and_erased_roots() {
    #[derive(Debug)]
    struct Code(u8);

    impl fmt::Display for Code {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "code {}", self.0)
        }
    }

    impl core::error::Error for Code {}

    fn mutate<E: TypeSet>(mut adapter: ErrorUnionAdapter<E>) -> ErrorUnion<E> {
        let borrowed: &ErrorUnion<E> = adapter.as_ref();
        assert_eq!(borrowed.downcast_inner_ref::<Code>().unwrap().0, 7);

        let borrowed: &mut ErrorUnion<E> = adapter.as_mut();
        borrowed.downcast_inner_mut::<Code>().unwrap().0 += 1;

        let error = adapter.into_unadapted();
        assert_eq!(error.downcast_inner_ref::<Code>().unwrap().0, 8);
        error
    }

    let typed: ErrorUnion<(Code, fmt::Error)> = ErrorUnion::new(Code(7));
    let _: ErrorUnion<(Code, fmt::Error)> = mutate(typed.into_adapter());

    let erased: ErrorUnion = ErrorUnion::new(Code(7));
    let _: ErrorUnion = mutate(erased.into_adapter());
}
