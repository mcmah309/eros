use core::{fmt, num::ParseIntError};
use eros::{ErrorUnion, IntoUnion, ReshapeUnion, StdError, TypeSet};

#[test]
fn union_question_mark_infers_typed_and_erased_error_sets() {
    fn typed<T>(result: Result<T, fmt::Error>) -> Result<T, StdError<(fmt::Error, ParseIntError)>> {
        Ok(result.union()?)
    }

    fn erased<T>(result: Result<T, fmt::Error>) -> Result<T, StdError> {
        Ok(result.union()?)
    }

    let error = typed(Err::<(), _>(fmt::Error)).unwrap_err();
    let _: ErrorUnion<(fmt::Error, ParseIntError)> = error.into_union();
    let error = erased(Err::<(), _>(fmt::Error)).unwrap_err();
    assert!(error.into_union().is_inner::<fmt::Error>());

    let mut value = 7;
    *typed(Ok(&mut value)).unwrap() += 1;
    *erased(Ok(&mut value)).unwrap() += 1;
    assert_eq!(value, 9);
}

#[test]
fn widen_question_mark_infers_larger_and_erased_error_sets() {
    fn typed<T>(
        result: eros::Result<T, (fmt::Error,)>,
    ) -> Result<T, StdError<(ParseIntError, fmt::Error)>> {
        Ok(result.widen()?)
    }

    fn erased<T>(result: eros::Result<T, (fmt::Error,)>) -> Result<T, StdError> {
        Ok(result.widen()?)
    }

    let error = typed(Err::<(), _>(ErrorUnion::new(fmt::Error))).unwrap_err();
    let error: ErrorUnion<(ParseIntError, fmt::Error)> = error.into_union();
    assert!(error.is_inner::<fmt::Error>());
    let error = erased(Err::<(), _>(ErrorUnion::new(fmt::Error))).unwrap_err();
    assert!(error.into_union().is_inner::<fmt::Error>());

    assert_eq!(typed(Ok(7)).unwrap(), 7);
    assert_eq!(erased(Ok(8)).unwrap(), 8);
}

#[test]
fn question_mark_converts_matching_unions_without_reshaping() {
    fn propagate<T, E: TypeSet>(result: eros::Result<T, E>) -> Result<T, StdError<E>> {
        Ok(result?)
    }

    let typed: eros::Result<(), (fmt::Error,)> = Err(ErrorUnion::new(fmt::Error));
    let error: StdError<(fmt::Error,)> = propagate(typed).unwrap_err();
    let _: fmt::Error = error.into_union().into_single();

    let erased: eros::Result<()> = Err(ErrorUnion::new(fmt::Error));
    let error: StdError = propagate(erased).unwrap_err();
    assert!(error.into_union().is_inner::<fmt::Error>());

    let empty: eros::Result<_, ()> = Ok(7);
    assert_eq!(propagate(empty).unwrap(), 7);
}
