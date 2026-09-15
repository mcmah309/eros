use eros::{ErrorUnion, MsgError, ReshapeUnion, SendSyncError};
use std::{fmt, io, num::ParseIntError};

type Input = (MsgError, fmt::Error, io::Error);

fn inputs() -> [eros::Result<String, Input>; 4] {
    [
        Ok(String::from("original success")),
        Err(ErrorUnion::new::<_, Input, _>(MsgError::from("message")).context("message context")),
        Err(ErrorUnion::new::<_, Input, _>(fmt::Error).context("format context")),
        Err(ErrorUnion::new::<_, Input, _>(io::Error::other("io")).context("io context")),
    ]
}

fn address<E: eros::TypeSet>(error: &ErrorUnion<E>) -> *const () {
    error.inner() as *const dyn SendSyncError as *const ()
}

// Check each possible input against a target at the beginning, middle, and end.
// Naming both remaining types checks that removing a member retains tuple order.
macro_rules! check_target {
    ($module:ident, $target:ty, $matches:literal, $first:ty, $second:ty) => {
        mod $module {
            use super::*;
            type Remaining = ($first, $second);

            #[test]
            fn bare_and_tuple_narrow_route_every_input() {
                for tuple in [false, true] {
                    for (index, result) in inputs().into_iter().enumerate() {
                        let report = format!("{result:?}");
                        let original = result.as_ref().err().map(address);
                        let success = result.as_ref().ok().map(|value| value.as_ptr());
                        let remainder: eros::Result<String, Remaining> = if tuple {
                            match result.narrow::<($target,), _>() {
                                Ok(error) => {
                                    assert_eq!(index, $matches);
                                    assert_eq!(Some(address(&error)), original);
                                    assert_eq!(format!("Err({error:?})"), report);
                                    continue;
                                }
                                Err(remainder) => remainder,
                            }
                        } else {
                            match result.narrow::<$target, _>() {
                                Ok(error) => {
                                    assert_eq!(index, $matches);
                                    assert_eq!(
                                        error.to_string(),
                                        match index {
                                            1 => "message".to_string(),
                                            2 => fmt::Error.to_string(),
                                            3 => "io".to_string(),
                                            _ => unreachable!(),
                                        }
                                    );
                                    continue;
                                }
                                Err(remainder) => remainder,
                            }
                        };
                        assert_ne!(index, $matches);
                        assert_eq!(format!("{remainder:?}"), report);
                        match remainder {
                            Ok(value) => {
                                assert_eq!(index, 0);
                                assert_eq!(Some(value.as_ptr()), success);
                            }
                            Err(error) => assert_eq!(Some(address(&error)), original),
                        }
                    }
                }
            }

            #[test]
            fn recover_and_try_recover_route_every_input_and_handler_outcome() {
                type Output = (ParseIntError, $first, $second);
                for (fallible, fails) in [(false, false), (true, false), (true, true)] {
                    for (index, result) in inputs().into_iter().enumerate() {
                        let report = format!("{result:?}");
                        let original = result.as_ref().err().map(address);
                        let success = result.as_ref().ok().map(|value| value.as_ptr());
                        let replacement = String::from("recovered");
                        let replacement_ptr = replacement.as_ptr();
                        let mut calls = 0;
                        let handler = |error: ErrorUnion<($target,)>| {
                            calls += 1;
                            assert_eq!(index, $matches);
                            assert_eq!(Some(address(&error)), original);
                            assert_eq!(format!("Err({error:?})"), report);
                            replacement
                        };
                        let result: eros::Result<String, Output> = if !fallible {
                            let recovered: eros::Result<String, Remaining> =
                                result.recover::<$target, _>(handler);
                            recovered.widen()
                        } else {
                            result.try_recover::<$target, _, _, _>(|error| {
                                let value = handler(error);
                                if fails {
                                    Err(ErrorUnion::new("invalid".parse::<u8>().unwrap_err()))
                                } else {
                                    Ok(value)
                                }
                            })
                        };
                        assert_eq!(calls, usize::from(index == $matches));
                        if index == $matches {
                            if fails {
                                assert!(matches!(result.unwrap_err().into_enum(), eros::E3::A(_)));
                            } else {
                                let value = result.unwrap();
                                assert_eq!(value, "recovered");
                                assert_eq!(value.as_ptr(), replacement_ptr);
                            }
                        } else {
                            assert_eq!(format!("{result:?}"), report);
                            match result {
                                Ok(value) => {
                                    assert_eq!(index, 0);
                                    assert_eq!(Some(value.as_ptr()), success);
                                }
                                Err(error) => assert_eq!(Some(address(&error)), original),
                            }
                        }
                    }
                }
            }
        }
    };
}

check_target!(first, MsgError, 1, fmt::Error, io::Error);
check_target!(middle, fmt::Error, 2, MsgError, io::Error);
check_target!(last, io::Error, 3, MsgError, fmt::Error);
