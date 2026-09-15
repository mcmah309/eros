use eros::{E2, ErrorUnion, MsgError, ReshapeUnion, SendSyncError};
use std::{fmt, io, net::AddrParseError, num::ParseIntError};

type Input = (MsgError, io::Error, fmt::Error, ParseIntError);
type Selected = (fmt::Error, MsgError);
type Remaining = (io::Error, ParseIntError);

fn input_errors() -> [ErrorUnion<Input>; 4] {
    [
        ErrorUnion::new(MsgError::from("message")),
        ErrorUnion::new(io::Error::other("io")),
        ErrorUnion::new(fmt::Error),
        ErrorUnion::new("invalid".parse::<u8>().unwrap_err()),
    ]
}

#[test]
fn recover_selects_every_group_member_and_preserves_the_remainder() {
    for (index, error) in input_errors().into_iter().enumerate() {
        let error = error.context("inner").context("outer");
        #[cfg(feature = "user_context")]
        let error = error.user_context("user message");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let report = format!("{error:?}");
        #[cfg(feature = "diagnostic")]
        let diagnostic = error.to_debug_json();
        let result: eros::Result<u8, Input> = Err(error);
        let mut calls = 0;
        let result: eros::Result<u8, Remaining> = result.recover(|error: ErrorUnion<Selected>| {
            calls += 1;
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), report);
            #[cfg(feature = "diagnostic")]
            assert_eq!(error.to_debug_json(), diagnostic);
            match (index, error.into_enum()) {
                (0, E2::B(error)) => assert_eq!(error.as_str(), "message"),
                (2, E2::A(fmt::Error)) => {}
                _ => panic!("wrong selected variant or tuple order"),
            }
            7
        });
        let selected = matches!(index, 0 | 2);
        assert_eq!(calls, usize::from(selected));
        if selected {
            assert_eq!(result.unwrap(), 7);
        } else {
            let error = result.unwrap_err();
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), report);
            #[cfg(feature = "diagnostic")]
            assert_eq!(error.to_debug_json(), diagnostic);
            match (index, error.into_enum()) {
                (1, E2::A(_)) | (3, E2::B(_)) => {}
                _ => panic!("wrong remaining variant or tuple order"),
            }
        }
    }
}

#[test]
fn try_recover_groups_keep_original_and_fallback_diagnostics() {
    type Output = (ParseIntError, AddrParseError, io::Error);
    for (index, error) in input_errors().into_iter().enumerate() {
        let error = error.context("original operation");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let original_report = format!("{error:?}");
        let fallback: ErrorUnion<(AddrParseError,)> =
            ErrorUnion::new("invalid".parse::<std::net::IpAddr>().unwrap_err());
        let fallback = fallback.context("fallback operation");
        let fallback_ptr = fallback.inner() as *const dyn SendSyncError as *const ();
        let fallback_report = format!("{fallback:?}");
        let result: eros::Result<u8, Input> = Err(error);
        let mut calls = 0;
        let result: eros::Result<u8, Output> = result.try_recover(|error: ErrorUnion<Selected>| {
            calls += 1;
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), original_report);
            // The handler owns the fallback, and widening infers Output from
            // the enclosing result. This exercises an FnOnce group handler.
            Err(fallback.widen())
        });
        let selected = matches!(index, 0 | 2);
        assert_eq!(calls, usize::from(selected));
        let error = result.unwrap_err();
        let (expected_ptr, expected_report) = if selected {
            (fallback_ptr, fallback_report)
        } else {
            (original, original_report)
        };
        assert_eq!(
            error.inner() as *const dyn SendSyncError as *const (),
            expected_ptr
        );
        assert_eq!(format!("{error:?}"), expected_report);
        match (index, error.into_enum()) {
            (0 | 2, eros::E3::B(_)) | (1, eros::E3::C(_)) | (3, eros::E3::A(_)) => {}
            _ => panic!("wrong output variant"),
        }
    }
}

#[test]
fn successes_pass_through_group_handlers_without_calling_them() {
    for fallible in [false, true] {
        let result: eros::Result<String, Input> = Ok(String::from("owned"));
        let original = result.as_ref().unwrap().as_ptr();
        let result: eros::Result<String, Remaining> = if fallible {
            result.try_recover::<Selected, _, _, _>(|_| panic!("must not handle success"))
        } else {
            result.recover::<Selected, _>(|_| panic!("must not handle success"))
        };
        let value = result
            .recover(|_: ErrorUnion<Remaining>| panic!("must not handle success"))
            .into_value();
        assert_eq!(value.as_ptr(), original);
    }
}

#[test]
fn fallible_group_handler_can_reintroduce_handled_types() {
    let error: ErrorUnion<(io::Error, MsgError, fmt::Error)> = ErrorUnion::new(fmt::Error);
    let error = error.context("retained");
    let original = error.inner() as *const dyn SendSyncError as *const ();
    let report = format!("{error:?}");
    let result: eros::Result<(), (io::Error, MsgError, fmt::Error)> = Err(error);
    let result: eros::Result<(), (MsgError, fmt::Error, io::Error)> = result
        .try_recover::<Selected, _, _, _>(|error| {
            assert!(matches!(error.as_enum(), E2::A(_)));
            Err(error.widen())
        });
    let error = result.unwrap_err();
    assert_eq!(
        error.inner() as *const dyn SendSyncError as *const (),
        original
    );
    assert_eq!(format!("{error:?}"), report);
}

#[test]
fn group_handlers_do_not_match_sources_or_contexts() {
    for fallible in [false, true] {
        let error: ErrorUnion<(io::Error, MsgError, fmt::Error)> =
            ErrorUnion::new(io::Error::other(MsgError::from("source")));
        let context: Box<dyn SendSyncError> = Box::new(fmt::Error);
        let error = error.context(context);
        let report = format!("{error:?}");
        let result: eros::Result<(), (io::Error, MsgError, fmt::Error)> = Err(error);
        let result: eros::Result<(), (io::Error,)> = if fallible {
            result.try_recover::<Selected, _, _, _>(|_| panic!("must only match the inner error"))
        } else {
            result.recover::<Selected, _>(|_| panic!("must only match the inner error"))
        };
        assert_eq!(format!("{:?}", result.unwrap_err()), report);
    }
}

#[derive(Debug)]
struct Payload<const N: u8>;

impl<const N: u8> fmt::Display for Payload<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {N}")
    }
}
impl<const N: u8> std::error::Error for Payload<N> {}

macro_rules! check_group_arity {
    ($test:ident, $($n:literal),+) => {
        #[test]
        fn $test() {
            type Errors = ($(Payload<$n>,)+);
            // Each generated handler implementation must support inference and
            // remove the full group, for both recovery methods.
            // Exercise the whole runtime type search, including the last member.
            let errors: [ErrorUnion<Errors>; [$($n),+].len()] = [
                $(ErrorUnion::new(Payload::<$n>),)+
            ];
            for (index, error) in errors.into_iter().enumerate() {
                let expected = format!("error {index}");
                let result: eros::Result<u8, Errors> = Err(error);
                let mut calls = 0;
                let value = result.recover(|error: ErrorUnion<Errors>| {
                    calls += 1;
                    assert_eq!(error.to_string(), expected);
                    7
                }).into_value();
                assert_eq!(calls, 1);
                assert_eq!(value, 7);
            }
            let errors: [ErrorUnion<Errors>; [$($n),+].len()] = [
                $(ErrorUnion::new(Payload::<$n>),)+
            ];
            for (index, error) in errors.into_iter().enumerate() {
                let expected = format!("error {index}");
                let result: eros::Result<u8, Errors> = Err(error);
                let mut calls = 0;
                let result: eros::Result<u8, ()> = result.try_recover(|error: ErrorUnion<Errors>| {
                    calls += 1;
                    assert_eq!(error.to_string(), expected);
                    Ok(7)
                });
                assert_eq!(calls, 1);
                assert_eq!(result.into_value(), 7);
            }
        }
    };
}

check_group_arity!(arity_2, 0, 1);
check_group_arity!(arity_3, 0, 1, 2);
check_group_arity!(arity_4, 0, 1, 2, 3);
check_group_arity!(arity_5, 0, 1, 2, 3, 4);
check_group_arity!(arity_6, 0, 1, 2, 3, 4, 5);
check_group_arity!(arity_7, 0, 1, 2, 3, 4, 5, 6);
check_group_arity!(arity_8, 0, 1, 2, 3, 4, 5, 6, 7);
check_group_arity!(arity_9, 0, 1, 2, 3, 4, 5, 6, 7, 8);
check_group_arity!(arity_10, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
check_group_arity!(arity_11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
check_group_arity!(arity_12, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
check_group_arity!(arity_13, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
check_group_arity!(arity_14, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
check_group_arity!(arity_15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
check_group_arity!(
    arity_16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
);
check_group_arity!(
    arity_17, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
);
check_group_arity!(
    arity_18, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
);
check_group_arity!(
    arity_19, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18
);
check_group_arity!(
    arity_20, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
);
check_group_arity!(
    arity_21, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
);
check_group_arity!(
    arity_22, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21
);
check_group_arity!(
    arity_23, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22
);
check_group_arity!(
    arity_24, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23
);
check_group_arity!(
    arity_25, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24
);
check_group_arity!(
    arity_26, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25
);
