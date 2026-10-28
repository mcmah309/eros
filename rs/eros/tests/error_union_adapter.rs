use core::{fmt, num::ParseIntError};
use eros::{ErrorUnion, ErrorUnionAdapter, IntoUnion, ReshapeUnion, TypeSet};

#[test]
fn result_adapter_preserves_error_sets_and_success_values() {
    fn adapt<T, E: TypeSet>(result: eros::Result<T, E>) -> Result<T, ErrorUnionAdapter<E>> {
        result.adapter()
    }

    let typed: eros::Result<(), (fmt::Error, ParseIntError)> = Err(ErrorUnion::new(fmt::Error));
    let error: ErrorUnion<(fmt::Error, ParseIntError)> = adapt(typed).unwrap_err().union();
    assert!(error.is_inner::<fmt::Error>());

    let erased: eros::Result<()> = Err(ErrorUnion::new(fmt::Error));
    let error: ErrorUnion = adapt(erased).unwrap_err().union();
    assert!(error.is_inner::<fmt::Error>());

    let mut value = 7;
    let typed: eros::Result<_, (fmt::Error,)> = Ok(&mut value);
    *adapt(typed).unwrap() += 1;
    let erased: eros::Result<_> = Ok(&mut value);
    *adapt(erased).unwrap() += 1;
    let empty: eros::Result<_, ()> = Ok(&mut value);
    *adapt(empty).unwrap() += 1;
    assert_eq!(value, 10);
}

#[test]
fn result_adapter_infers_error_sets_in_return_expressions() {
    fn typed() -> Result<(), ErrorUnionAdapter<(fmt::Error, ParseIntError)>> {
        return Err(fmt::Error).union().adapter();
    }

    fn erased() -> Result<(), ErrorUnionAdapter> {
        return Err(fmt::Error).union().adapter();
    }

    fn widened<T>(
        result: eros::Result<T, (fmt::Error,)>,
    ) -> Result<T, ErrorUnionAdapter<(ParseIntError, fmt::Error)>> {
        result.widen().adapter()
    }

    assert!(typed().unwrap_err().union().is_inner::<fmt::Error>());
    assert!(erased().unwrap_err().union().is_inner::<fmt::Error>());
    assert!(
        widened(Err::<(), _>(ErrorUnion::new(fmt::Error)))
            .unwrap_err()
            .union()
            .is_inner::<fmt::Error>()
    );
    assert_eq!(widened(Ok(7)).unwrap(), 7);
}

#[test]
fn result_adapter_preserves_diagnostics() {
    use eros::prelude::*;

    let error: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
    let result = Err::<(), _>(error).context("format response");
    let before = format!("{:?}", result.as_ref().unwrap_err());
    #[cfg(feature = "error_location")]
    let location = result.as_ref().unwrap_err().location();

    let error = result.adapter().unwrap_err().union();
    assert!(error.is_inner::<fmt::Error>());
    assert_eq!(format!("{error:?}"), before);
    #[cfg(feature = "context")]
    assert!(before.contains("format response"));
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, error.location()));
}

#[test]
fn union_then_adapter_question_mark_infers_typed_and_erased_error_sets() {
    fn typed<T>(
        result: Result<T, fmt::Error>,
    ) -> Result<T, ErrorUnionAdapter<(fmt::Error, ParseIntError)>> {
        Ok(result.union().adapter()?)
    }

    fn erased<T>(result: Result<T, fmt::Error>) -> Result<T, ErrorUnionAdapter> {
        Ok(result.union().adapter()?)
    }

    let error: ErrorUnion<(fmt::Error, ParseIntError)> =
        typed(Err::<(), _>(fmt::Error)).unwrap_err().union();
    assert!(error.is_inner::<fmt::Error>());
    let error: ErrorUnion = erased(Err::<(), _>(fmt::Error)).unwrap_err().union();
    assert!(error.is_inner::<fmt::Error>());

    let mut value = 7;
    *typed(Ok(&mut value)).unwrap() += 1;
    *erased(Ok(&mut value)).unwrap() += 1;
    assert_eq!(value, 9);
}

#[cfg(feature = "error_location")]
#[test]
fn union_then_adapter_preserves_call_location() {
    let line = line!() + 1;
    let result: Result<(), ErrorUnionAdapter> = Err(fmt::Error).union().adapter();
    let error = result.unwrap_err().union();
    assert_eq!(error.location().file(), file!());
    assert_eq!(error.location().line(), line);
}

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
    let _: ErrorUnion<(fmt::Error, ParseIntError)> = error.union();
    let error = erased(Err::<(), _>(fmt::Error)).unwrap_err();
    assert!(error.union().is_inner::<fmt::Error>());

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
    let error: ErrorUnion<(ParseIntError, fmt::Error)> = error.union();
    assert!(error.is_inner::<fmt::Error>());
    let error = erased(Err::<(), _>(ErrorUnion::new(fmt::Error))).unwrap_err();
    assert!(error.union().is_inner::<fmt::Error>());

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
    let _: fmt::Error = error.union().into_single();

    let erased: eros::Result<()> = Err(ErrorUnion::new(fmt::Error));
    let error: ErrorUnionAdapter = propagate(erased).unwrap_err();
    assert!(error.union().is_inner::<fmt::Error>());

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

        let error = adapter.union();
        assert_eq!(error.downcast_inner_ref::<Code>().unwrap().0, 8);
        error
    }

    let typed: ErrorUnion<(Code, fmt::Error)> = ErrorUnion::new(Code(7));
    let _: ErrorUnion<(Code, fmt::Error)> = mutate(typed.adapter());

    let erased: ErrorUnion = ErrorUnion::new(Code(7));
    let _: ErrorUnion = mutate(erased.adapter());
}
