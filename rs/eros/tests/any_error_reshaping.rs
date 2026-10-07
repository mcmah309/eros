use eros::{ErrorUnion, MsgError, ReshapeUnion, SendSyncError, TypeSet};
use std::{fmt, io};

#[eros::error_enum_ref(SelectedErrorRef)]
type Selected = (fmt::Error, MsgError);

struct Snapshot {
    address: *const (),
    report: String,
    #[cfg(feature = "diagnostic")]
    diagnostic: serde_json::Value,
}

impl Snapshot {
    fn new(error: &ErrorUnion) -> Self {
        Self {
            address: error.inner() as *const dyn SendSyncError as *const (),
            report: format!("{error:?}"),
            #[cfg(feature = "diagnostic")]
            diagnostic: error.to_debug_json(),
        }
    }

    fn assert_preserved<E: TypeSet>(&self, error: &ErrorUnion<E>) {
        assert_eq!(
            error.inner() as *const dyn SendSyncError as *const (),
            self.address
        );
        assert_eq!(format!("{error:?}"), self.report);
        #[cfg(feature = "diagnostic")]
        assert_eq!(error.to_debug_json(), self.diagnostic);
    }
}

fn errors() -> [ErrorUnion; 3] {
    let errors: [ErrorUnion; 3] = [
        ErrorUnion::new(MsgError::from(String::from("message"))),
        ErrorUnion::new(fmt::Error),
        ErrorUnion::new(io::Error::other("unhandled")),
    ];
    errors.map(|error| {
        let error = error.context("inner operation").context("outer operation");
        #[cfg(feature = "user_context")]
        let error = error.user_context("user message");
        error
    })
}

#[test]
fn erased_single_narrow_extracts_matches_and_keeps_an_erased_remainder() {
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let selected: Result<MsgError, ErrorUnion> = error.narrow::<MsgError, _>();
        match selected {
            Ok(error) => {
                assert_eq!(index, 0);
                assert_eq!(error.as_str(), "message");
            }
            Err(remainder) => {
                assert_ne!(index, 0);
                snapshot.assert_preserved(&remainder);
                // A failed selection must leave the error available for another attempt.
                let remainder: ErrorUnion = remainder.narrow::<MsgError, _>().unwrap_err();
                snapshot.assert_preserved(&remainder);
                if index == 1 {
                    assert_eq!(remainder.narrow::<fmt::Error, _>().unwrap(), fmt::Error);
                } else {
                    assert_eq!(
                        remainder.narrow::<io::Error, _>().unwrap().to_string(),
                        "unhandled"
                    );
                }
            }
        }
    }
}

#[test]
fn erased_group_narrow_builds_typed_unions_and_preserves_both_branches() {
    for singleton in [false, true] {
        for (index, error) in errors().into_iter().enumerate() {
            let snapshot = Snapshot::new(&error);
            let partition: Result<ErrorUnion<Selected>, ErrorUnion> = if singleton {
                error.narrow::<(MsgError,), _>().map(|error| error.widen())
            } else {
                error.narrow::<(fmt::Error, MsgError), _>()
            };
            // Check the branch before using enum conversions, which rely on typed membership.
            assert_eq!(partition.is_ok(), index == 0 || (!singleton && index == 1));
            let error: ErrorUnion = match partition {
                Ok(selected) => {
                    snapshot.assert_preserved(&selected);
                    match (index, SelectedErrorRef::from(&selected)) {
                        (0, SelectedErrorRef::MsgError(error)) => {
                            assert_eq!(error.as_str(), "message")
                        }
                        (1, SelectedErrorRef::FmtError(_)) => {}
                        _ => panic!("selected the wrong typed variant"),
                    }
                    selected.into()
                }
                Err(remainder) => remainder,
            };
            snapshot.assert_preserved(&error);
        }
    }
}

#[test]
fn erased_empty_group_narrow_always_retains_the_error() {
    for error in errors() {
        let snapshot = Snapshot::new(&error);
        let selected: Result<ErrorUnion<()>, ErrorUnion> = error.narrow::<(), _>();
        assert!(selected.is_err());
        snapshot.assert_preserved(&selected.unwrap_err());
    }
}

#[test]
fn erased_narrow_only_matches_the_inner_error() {
    let error: ErrorUnion = ErrorUnion::new(io::Error::other(MsgError::from("source error")));
    let context: Box<dyn SendSyncError> = Box::new(MsgError::from("context error"));
    let error = error.context(context);
    let snapshot = Snapshot::new(&error);
    let error: ErrorUnion = error.narrow::<MsgError, _>().unwrap_err();
    snapshot.assert_preserved(&error);
    let error: ErrorUnion = error.narrow::<(fmt::Error, MsgError), _>().unwrap_err();
    snapshot.assert_preserved(&error);
    assert_eq!(
        error.narrow::<io::Error, _>().unwrap().to_string(),
        "source error"
    );
}

#[test]
fn erased_result_narrow_preserves_success_and_routes_each_error() {
    let value = String::from("success");
    let address = value.as_ptr();
    let result: eros::Result<String> = Ok(value);
    let result: eros::Result<String> = result.narrow::<MsgError, _>().unwrap_err();
    let result: eros::Result<String> = result.narrow::<(fmt::Error, MsgError), _>().unwrap_err();
    let value = result.unwrap();
    assert_eq!(value, "success");
    assert_eq!(value.as_ptr(), address);

    for group in [false, true] {
        for (index, error) in errors().into_iter().enumerate() {
            let snapshot = Snapshot::new(&error);
            let result: eros::Result<String> = Err(error);
            let remainder: eros::Result<String> = if group {
                let selected: Result<ErrorUnion<Selected>, eros::Result<String>> =
                    result.narrow::<(fmt::Error, MsgError), _>();
                assert_eq!(selected.is_ok(), index < 2);
                match selected {
                    Ok(error) => {
                        snapshot.assert_preserved(&error);
                        assert!(matches!(
                            (index, SelectedErrorRef::from(&error)),
                            (0, SelectedErrorRef::MsgError(_)) | (1, SelectedErrorRef::FmtError(_))
                        ));
                        continue;
                    }
                    Err(remainder) => remainder,
                }
            } else {
                let selected: Result<MsgError, eros::Result<String>> =
                    result.narrow::<MsgError, _>();
                assert_eq!(selected.is_ok(), index == 0);
                match selected {
                    Ok(error) => {
                        assert_eq!(error.as_str(), "message");
                        continue;
                    }
                    Err(remainder) => remainder,
                }
            };
            snapshot.assert_preserved(&remainder.unwrap_err());
        }
    }
}

#[test]
fn erased_recover_handles_single_and_group_targets_without_losing_unknown_errors() {
    for group in [false, true] {
        let result: eros::Result<String> = Ok(String::from("success"));
        let address = result.as_ref().unwrap().as_ptr();
        let result: eros::Result<String> = result
            .recover::<MsgError, _>(|_| panic!("must not handle success"))
            .recover::<(fmt::Error, MsgError), _>(|_| panic!("must not handle success"));
        let value = result.unwrap();
        assert_eq!(value, "success");
        assert_eq!(value.as_ptr(), address);

        for (index, error) in errors().into_iter().enumerate() {
            let snapshot = Snapshot::new(&error);
            let result: eros::Result<String> = Err(error);
            let mut calls = 0;
            let result: eros::Result<String> = if group {
                result.recover(|error: ErrorUnion<Selected>| {
                    calls += 1;
                    snapshot.assert_preserved(&error);
                    assert!(matches!(
                        (index, SelectedErrorRef::from(&error)),
                        (0, SelectedErrorRef::MsgError(_)) | (1, SelectedErrorRef::FmtError(_))
                    ));
                    String::from("recovered")
                })
            } else {
                result.recover(|error: ErrorUnion<(MsgError,)>| {
                    calls += 1;
                    assert_eq!(index, 0);
                    snapshot.assert_preserved(&error);
                    assert_eq!(error.into_single().as_str(), "message");
                    String::from("recovered")
                })
            };
            let matches = index == 0 || (group && index == 1);
            assert_eq!(calls, usize::from(matches));
            if matches {
                assert_eq!(result.unwrap(), "recovered");
            } else {
                snapshot.assert_preserved(&result.unwrap_err());
            }
        }
    }
}

#[test]
fn erased_try_recover_preserves_success_remainders_and_handler_errors() {
    let result: eros::Result<String> = Ok(String::from("success"));
    let address = result.as_ref().unwrap().as_ptr();
    let result: eros::Result<String> =
        result.try_recover::<MsgError, _, _, _>(|_| panic!("must not handle success"));
    let value = result.unwrap();
    assert_eq!(value, "success");
    assert_eq!(value.as_ptr(), address);

    for fails in [false, true] {
        for (index, error) in errors().into_iter().enumerate() {
            let snapshot = Snapshot::new(&error);
            let replacement: ErrorUnion = ErrorUnion::new(io::Error::other("replacement"));
            let replacement = replacement.context("handler context");
            let replacement_snapshot = Snapshot::new(&replacement);
            let result: eros::Result<String> = Err(error);
            let mut calls = 0;
            let result: eros::Result<String> = result.try_recover(|error: ErrorUnion<Selected>| {
                calls += 1;
                snapshot.assert_preserved(&error);
                assert!(matches!(
                    (index, SelectedErrorRef::from(&error)),
                    (0, SelectedErrorRef::MsgError(_)) | (1, SelectedErrorRef::FmtError(_))
                ));
                if fails {
                    Err(replacement)
                } else {
                    Ok(String::from("recovered"))
                }
            });
            assert_eq!(calls, usize::from(index < 2));
            if index == 2 {
                snapshot.assert_preserved(&result.unwrap_err());
            } else if fails {
                replacement_snapshot.assert_preserved(&result.unwrap_err());
            } else {
                assert_eq!(result.unwrap(), "recovered");
            }
        }
    }
}
