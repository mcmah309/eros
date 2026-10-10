mod common;

use core::{fmt, num::ParseIntError};
use eros::{ErrorUnion, IntoUnion, OtherError, ReshapeUnion};

type Open = (fmt::Error, ParseIntError, OtherError);
type Typed = (fmt::Error, ParseIntError);

#[derive(Debug)]
struct Unlisted(usize);

impl fmt::Display for Unlisted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unlisted {}", self.0)
    }
}

impl core::error::Error for Unlisted {}

fn errors() -> [ErrorUnion<Open>; 3] {
    [
        ErrorUnion::new(fmt::Error),
        ErrorUnion::new("invalid".parse::<u8>().unwrap_err()),
        ErrorUnion::new(Unlisted(7)),
    ]
}

#[test]
fn other_selects_only_unlisted_inner_types_and_preserves_both_branches() {
    for (index, error) in errors().into_iter().enumerate() {
        let error = error.context("operation");
        #[cfg(feature = "user_context")]
        let error = error.user_context("user message");
        let identity = common::identity(error.inner());
        let report = format!("{error:?}");
        #[cfg(feature = "location")]
        let location = error.location();
        #[cfg(feature = "diagnostic")]
        let diagnostic = error.to_debug_json();
        let partition: Result<ErrorUnion, ErrorUnion<Typed>> = error.narrow::<OtherError, _>();
        assert_eq!(partition.is_ok(), index == 2);
        let error: ErrorUnion = match partition {
            Ok(other) => {
                assert_eq!(other.downcast_inner_ref::<Unlisted>().unwrap().0, 7);
                other
            }
            Err(typed) => {
                assert_eq!(typed.is_inner::<fmt::Error>(), index == 0);
                assert_eq!(typed.is_inner::<ParseIntError>(), index == 1);
                typed.into()
            }
        };
        assert_eq!(common::identity(error.inner()), identity);
        assert_eq!(format!("{error:?}"), report);
        #[cfg(feature = "location")]
        assert!(core::ptr::eq(error.location(), location));
        #[cfg(feature = "diagnostic")]
        assert_eq!(error.to_debug_json(), diagnostic);
    }
}

#[test]
fn recovery_infers_other_target_and_leaves_only_the_named_errors() {
    for fallible in [false, true] {
        for (index, error) in errors().into_iter().enumerate() {
            let report = format!("{error:?}");
            let result: eros::Result<usize, Open> = Err(error);
            let mut calls = 0;
            let result: eros::Result<usize, Typed> = if fallible {
                result.try_recover(|other: ErrorUnion| {
                    calls += 1;
                    assert_eq!(format!("{other:?}"), report);
                    assert!(other.is_inner::<Unlisted>());
                    Ok(42)
                })
            } else {
                result.recover(|other: ErrorUnion| {
                    calls += 1;
                    assert_eq!(format!("{other:?}"), report);
                    assert!(other.is_inner::<Unlisted>());
                    42
                })
            };
            assert_eq!(calls, usize::from(index == 2));
            if index == 2 {
                assert_eq!(result.unwrap(), 42);
            } else {
                assert_eq!(format!("{:?}", result.unwrap_err()), report);
            }
        }
    }
}

#[test]
fn typed_recovery_removes_names_before_later_other_checks() {
    for (index, error) in errors().into_iter().enumerate() {
        let result: eros::Result<usize, Open> = Err(error);
        let result: eros::Result<usize, (ParseIntError, OtherError)> =
            result.recover::<fmt::Error, _>(|_| 10);
        let result: eros::Result<usize, (ParseIntError,)> = result.recover::<OtherError, _>(|_| 30);
        let value = result.recover::<ParseIntError, _>(|_| 20).into_value();
        assert_eq!(value, (index + 1) * 10);
    }

    // Reintroducing a previously handled type is allowed by the open marker.
    // It is now unnamed, so OtherError must select it.
    let error: ErrorUnion<Open> = ErrorUnion::new(fmt::Error);
    let result: eros::Result<(), Open> = Err(error);
    let result: eros::Result<(), (ParseIntError, OtherError)> =
        result.try_recover::<fmt::Error, _, _, _>(|_| Err(fmt::Error).union());
    assert!(
        result
            .unwrap_err()
            .narrow::<OtherError, _>()
            .unwrap()
            .is_inner::<fmt::Error>()
    );
}

#[test]
fn typed_group_narrowing_and_recovery_retain_the_marker() {
    for (index, error) in errors().into_iter().enumerate() {
        let partition: Result<ErrorUnion<Typed>, ErrorUnion<(OtherError,)>> =
            error.narrow::<Typed, _>();
        assert_eq!(partition.is_ok(), index < 2);
        if let Err(other) = partition {
            assert!(
                other
                    .narrow::<OtherError, _>()
                    .unwrap()
                    .is_inner::<Unlisted>()
            );
        }
    }
    let result: eros::Result<usize, Open> = Err(ErrorUnion::new(Unlisted(7)));
    let result: eros::Result<usize, (OtherError,)> =
        result.recover(|_: ErrorUnion<Typed>| panic!("unlisted error must remain"));
    assert_eq!(result.recover::<OtherError, _>(|_| 42).into_value(), 42);
}

#[test]
fn other_only_sets_and_open_tuple_targets_accept_all_errors() {
    for error in errors() {
        let other: ErrorUnion<(OtherError,)> = error.widen();
        let all: Result<ErrorUnion<Open>, ErrorUnion<()>> = other.narrow::<Open, _>();
        assert!(all.is_ok());
    }

    let error: ErrorUnion<(OtherError,)> = ErrorUnion::new(fmt::Error);
    let selected: Result<ErrorUnion, ErrorUnion<()>> = error.narrow::<OtherError, _>();
    assert!(selected.unwrap().is_inner::<fmt::Error>());

    let result: eros::Result<usize, Open> = Err(ErrorUnion::new(fmt::Error));
    let value = result.recover(|_: ErrorUnion<Open>| 42).into_value();
    assert_eq!(value, 42);
    let result: eros::Result<usize, Open> = Err(ErrorUnion::new(fmt::Error));
    let value = result
        .recover(|_: ErrorUnion<(OtherError,)>| 42)
        .into_value();
    assert_eq!(value, 42);
}

#[test]
fn conversions_reuse_the_root_and_its_diagnostics() {
    let error: ErrorUnion = ErrorUnion::new(Unlisted(7));
    let error = error.context("conversion");
    let report = format!("{error:?}");
    let error: ErrorUnion<Open> = error.widen();
    assert_eq!(format!("{error:?}"), report);
    let error: ErrorUnion = error.into();
    assert_eq!(format!("{error:?}"), report);
    let error: ErrorUnion<Open> = error.into();
    assert_eq!(format!("{error:?}"), report);

    let result: eros::Result<(), (Unlisted,)> = Err(ErrorUnion::new(Unlisted(7)));
    let result: eros::Result<(), Open> = result.widen();
    assert!(result.unwrap_err().narrow::<OtherError, _>().is_ok());

    let result: eros::Result<(), Open> = Err(Unlisted(7)).union();
    assert!(result.unwrap_err().narrow::<OtherError, _>().is_ok());
    let typed: ErrorUnion<Typed> = ErrorUnion::new(fmt::Error);
    let open: ErrorUnion<Open> = typed.widen();
    assert!(open.narrow::<OtherError, _>().is_err());

    fn propagate_raw() -> eros::Result<(), Open> {
        Err(Unlisted(7))?;
        Ok(())
    }
    fn propagate_erased() -> eros::Result<(), Open> {
        let result: eros::Result<()> = Err(ErrorUnion::new(Unlisted(7)));
        result?;
        Ok(())
    }
    assert!(
        propagate_raw()
            .unwrap_err()
            .narrow::<OtherError, _>()
            .is_ok()
    );
    assert!(
        propagate_erased()
            .unwrap_err()
            .narrow::<OtherError, _>()
            .is_ok()
    );
}

#[test]
fn success_values_skip_other_handlers_and_narrowing() {
    let result: eros::Result<String, Open> = Ok(String::from("success"));
    let address = result.as_ref().unwrap().as_ptr();
    let result: eros::Result<String, Typed> = result.narrow::<OtherError, _>().unwrap_err();
    let value = result
        .recover(|_: ErrorUnion<Typed>| panic!("must not handle success"))
        .into_value();
    assert_eq!(value.as_ptr(), address);

    for fallible in [false, true] {
        let result: eros::Result<String, Open> = Ok(String::from("success"));
        let address = result.as_ref().unwrap().as_ptr();
        let result: eros::Result<String, Typed> = if fallible {
            result.try_recover::<OtherError, _, _, _>(|_| panic!("must not handle success"))
        } else {
            result.recover::<OtherError, _>(|_| panic!("must not handle success"))
        };
        let value = result.unwrap();
        assert_eq!(value.as_ptr(), address);
    }
}

#[test]
fn fallible_other_handler_can_reintroduce_typed_or_unnamed_errors() {
    for typed in [false, true] {
        let result: eros::Result<(), Open> = Err(ErrorUnion::new(Unlisted(7)));
        let replacement: ErrorUnion<Open> = if typed {
            ErrorUnion::new(fmt::Error)
        } else {
            ErrorUnion::new(Unlisted(42))
        };
        let replacement = replacement.context("fallback");
        let report = format!("{replacement:?}");
        let result: eros::Result<(), Open> =
            result.try_recover::<OtherError, _, _, _>(|_| Err(replacement));
        let error = result.unwrap_err();
        assert_eq!(format!("{error:?}"), report);
        assert_eq!(error.narrow::<OtherError, _>().is_err(), typed);
    }
}

#[cfg(feature = "alloc")]
#[test]
fn other_does_not_exclude_source_or_context_types() {
    use eros::SendSyncError;
    use std::io;

    let error: ErrorUnion<(fmt::Error, OtherError)> = ErrorUnion::new(io::Error::other(fmt::Error));
    let context: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let error = error.context(context);
    let identity = common::identity(error.inner());
    let report = format!("{error:?}");
    let other = error.narrow::<OtherError, _>().unwrap();
    assert!(other.is_inner::<io::Error>());
    assert_eq!(common::identity(other.inner()), identity);
    assert_eq!(format!("{other:?}"), report);
}

#[derive(Debug)]
struct Named<const N: u8>;

impl<const N: u8> fmt::Display for Named<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "named {N}")
    }
}

impl<const N: u8> core::error::Error for Named<N> {}

#[test]
fn maximum_open_tuple_supports_the_last_name_and_marker() {
    type Names = (
        Named<0>,
        Named<1>,
        Named<2>,
        Named<3>,
        Named<4>,
        Named<5>,
        Named<6>,
        Named<7>,
        Named<8>,
        Named<9>,
        Named<10>,
        Named<11>,
        Named<12>,
        Named<13>,
        Named<14>,
        Named<15>,
        Named<16>,
        Named<17>,
        Named<18>,
        Named<19>,
        Named<20>,
        Named<21>,
        Named<22>,
        Named<23>,
        Named<24>,
    );
    type Errors = (
        Named<0>,
        Named<1>,
        Named<2>,
        Named<3>,
        Named<4>,
        Named<5>,
        Named<6>,
        Named<7>,
        Named<8>,
        Named<9>,
        Named<10>,
        Named<11>,
        Named<12>,
        Named<13>,
        Named<14>,
        Named<15>,
        Named<16>,
        Named<17>,
        Named<18>,
        Named<19>,
        Named<20>,
        Named<21>,
        Named<22>,
        Named<23>,
        Named<24>,
        OtherError,
    );
    let error: ErrorUnion<Errors> = Named::<24>.into();
    let typed: ErrorUnion<Names> = error.narrow::<OtherError, _>().unwrap_err();
    assert!(typed.narrow::<Named<24>, _>().is_ok());
    let error: ErrorUnion<Errors> = ErrorUnion::new(Unlisted(7));
    let error = error.narrow::<Named<0>, _>().unwrap_err();
    let error = error.narrow::<Named<24>, _>().unwrap_err();
    assert!(error.narrow::<OtherError, _>().is_ok());
    let error: ErrorUnion<Errors> = ErrorUnion::new(Unlisted(7));
    let result: eros::Result<(), Errors> = Err(error);
    let result: eros::Result<(), (OtherError,)> =
        result.recover(|_: ErrorUnion<Names>| panic!("must retain unnamed error"));
    result.recover::<OtherError, _>(|_| ()).into_value();
}
