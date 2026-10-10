use core::{error::Error, fmt, fmt::Write};
use eros::{ErrorUnion, IntoAnyUnion, IntoUnion, OtherError, ReshapeUnion, SendSyncError, TypeSet};

macro_rules! unit_error {
    ($name:ident, $message:literal) => {
        #[derive(Debug, PartialEq, Eq)]
        struct $name;
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str($message)
            }
        }
        impl Error for $name {}
    };
}

unit_error!(Connect, "connection failed");
unit_error!(Read, "read failed");
unit_error!(Decode, "decode failed");
unit_error!(Validate, "validation failed");
unit_error!(Fallback, "fallback failed");

#[eros::error_enum_ref(AllRef)]
type All = (Connect, Read, Decode, Validate);
#[eros::error_enum(SelectedError)]
type Selected = (Decode, Connect);
#[eros::error_enum(RemainingError)]
type Remaining = (Read, Validate);

fn errors() -> [ErrorUnion<All>; 4] {
    [
        ErrorUnion::new(Connect),
        ErrorUnion::new(Read),
        ErrorUnion::new(Decode),
        ErrorUnion::new(Validate),
    ]
}

fn assert_variant<E: TypeSet>(error: &ErrorUnion<E>, index: usize) {
    assert_eq!(error.is_inner::<Connect>(), index == 0);
    assert_eq!(error.is_inner::<Read>(), index == 1);
    assert_eq!(error.is_inner::<Decode>(), index == 2);
    assert_eq!(error.is_inner::<Validate>(), index == 3);
    assert!(error.downcast_inner_ref::<u64>().is_none());
}

// Addresses alone cannot distinguish ZST owners, so also verify concrete types
// in every scenario. The snapshot checks that reshaping keeps the data pointer
// and (when enabled) the original call-site location.
struct Snapshot {
    data: *const (),
    #[cfg(feature = "location")]
    location: &'static core::panic::Location<'static>,
}
impl Snapshot {
    fn new<E: TypeSet>(error: &ErrorUnion<E>) -> Self {
        Self {
            data: error.inner() as *const dyn SendSyncError as *const (),
            #[cfg(feature = "location")]
            location: error.location(),
        }
    }
    fn assert_preserved<E: TypeSet>(&self, error: &ErrorUnion<E>) {
        assert_eq!(
            self.data,
            error.inner() as *const dyn SendSyncError as *const ()
        );
        #[cfg(feature = "location")]
        assert!(core::ptr::eq(self.location, error.location()));
    }
}

fn connect(mode: usize) -> Result<u8, Connect> {
    if mode == 0 { Err(Connect) } else { Ok(10) }
}

fn read(mode: usize) -> eros::Result<u8, (Connect, Read)> {
    let value = connect(mode).union()?;
    if mode == 1 {
        Err(Read).union()
    } else {
        Ok(value + 1)
    }
}

fn decode(mode: usize) -> eros::Result<u8, (Decode, Read, Connect)> {
    let value = read(mode).widen()?;
    if mode == 2 {
        Err(Decode).union()
    } else {
        Ok(value + 1)
    }
}

fn pipeline(mode: usize) -> eros::Result<u8, All> {
    let value = decode(mode).widen()?;
    if mode == 3 {
        Err(Validate).union()
    } else {
        Ok(value + 1)
    }
}

#[cfg_attr(test, test)]
pub fn union_widen_and_question_mark_compose_across_functions() {
    for mode in 0..4 {
        let error = pipeline(mode).unwrap_err();
        assert_variant(&error, mode);
        assert!(matches!(
            (mode, AllRef::from(&error)),
            (0, AllRef::Connect(_))
                | (1, AllRef::Read(_))
                | (2, AllRef::Decode(_))
                | (3, AllRef::Validate(_))
        ));
    }
    assert_eq!(pipeline(4).unwrap(), 13);
}

#[cfg_attr(test, test)]
pub fn group_narrow_partitions_every_variant_then_widens_back() {
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let split: Result<ErrorUnion<Selected>, ErrorUnion<Remaining>> =
            error.narrow::<Selected, _>();
        assert_eq!(split.is_ok(), matches!(index, 0 | 2));
        let error: ErrorUnion<All> = match split {
            Ok(selected) => {
                snapshot.assert_preserved(&selected);
                assert_variant(&selected, index);
                selected.widen()
            }
            Err(remaining) => {
                snapshot.assert_preserved(&remaining);
                assert_variant(&remaining, index);
                remaining.widen()
            }
        };
        snapshot.assert_preserved(&error);
        assert_variant(&error, index);
    }
}

#[cfg_attr(test, test)]
pub fn empty_full_and_single_narrow_targets_compose() {
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let error: ErrorUnion<All> = error.narrow::<(), _>().unwrap_err();
        let split: Result<ErrorUnion<(Validate, Decode, Read, Connect)>, ErrorUnion<()>> =
            error.narrow::<(Validate, Decode, Read, Connect), _>();
        let reordered = split.unwrap();
        snapshot.assert_preserved(&reordered);
        assert_variant(&reordered, index);
        match index {
            0 => assert_eq!(reordered.narrow::<Connect, _>().unwrap(), Connect),
            1 => assert_eq!(reordered.narrow::<Read, _>().unwrap(), Read),
            2 => assert_eq!(reordered.narrow::<Decode, _>().unwrap(), Decode),
            3 => assert_eq!(reordered.narrow::<Validate, _>().unwrap(), Validate),
            _ => unreachable!(),
        }
    }
    let error: ErrorUnion<All> = ErrorUnion::new(Read);
    let remaining: ErrorUnion<(Read, Decode, Validate)> = error.narrow::<Connect, _>().unwrap_err();
    let single = remaining.narrow::<(Read,), _>().unwrap();
    assert_eq!(single.into_single(), Read);
}

#[cfg_attr(test, test)]
pub fn result_narrow_routes_selected_remaining_and_success_branches() {
    for mode in 0..=4 {
        let split: Result<ErrorUnion<Selected>, eros::Result<u8, Remaining>> =
            pipeline(mode).narrow::<Selected, _>();
        assert_eq!(split.is_ok(), matches!(mode, 0 | 2));
        match split {
            Ok(selected) => assert!(matches!(
                (mode, SelectedError::from(selected)),
                (0, SelectedError::Connect(_)) | (2, SelectedError::Decode(_))
            )),
            Err(Err(remaining)) => assert!(matches!(
                (mode, RemainingError::from(remaining)),
                (1, RemainingError::Read(_)) | (3, RemainingError::Validate(_))
            )),
            Err(Ok(value)) => {
                assert_eq!(mode, 4);
                assert_eq!(value, 13);
            }
        }
    }
    let success: eros::Result<u8, All> = Ok(13);
    let remaining: eros::Result<u8, (Read, Decode, Validate)> =
        success.narrow::<Connect, _>().unwrap_err();
    assert_eq!(remaining.unwrap(), 13);
}

#[cfg_attr(test, test)]
pub fn recover_chains_remove_multiple_errors_and_finish_with_into_value() {
    for mode in 0..=4 {
        let mut calls = [0; 3];
        let remaining: eros::Result<u8, Remaining> =
            pipeline(mode).recover(|selected: ErrorUnion<Selected>| {
                calls[0] += 1;
                assert_variant(&selected, mode);
                20 + mode as u8
            });
        let remaining: eros::Result<u8, (Validate,)> = remaining.recover::<Read, _>(|error| {
            calls[1] += 1;
            assert_eq!(error.into_single(), Read);
            21
        });
        let value = remaining
            .recover::<Validate, _>(|error| {
                calls[2] += 1;
                assert_eq!(error.into_single(), Validate);
                23
            })
            .into_value();
        assert_eq!(
            calls,
            [
                usize::from(matches!(mode, 0 | 2)),
                usize::from(mode == 1),
                usize::from(mode == 3)
            ]
        );
        assert_eq!(value, if mode == 4 { 13 } else { 20 + mode as u8 });
    }
}

#[cfg_attr(test, test)]
pub fn try_recover_groups_replace_errors_or_keep_reordered_remainders() {
    type Output = (Validate, Fallback, Read);
    for succeeds in [false, true] {
        for (index, error) in errors().into_iter().enumerate() {
            let original = Snapshot::new(&error);
            let fallback: ErrorUnion<(Fallback,)> = ErrorUnion::new(Fallback);
            let replacement = Snapshot::new(&fallback);
            let mut calls = 0;
            let input: eros::Result<u8, All> = Err(error);
            let output: eros::Result<u8, Output> =
                input.try_recover(|selected: ErrorUnion<Selected>| {
                    calls += 1;
                    original.assert_preserved(&selected);
                    assert_variant(&selected, index);
                    if succeeds {
                        Ok(42)
                    } else {
                        Err(fallback.widen())
                    }
                });
            let selected = matches!(index, 0 | 2);
            assert_eq!(calls, usize::from(selected));
            if selected && succeeds {
                assert_eq!(output.unwrap(), 42);
            } else {
                let error = output.unwrap_err();
                if selected {
                    assert!(error.is_inner::<Fallback>());
                    replacement.assert_preserved(&error);
                } else {
                    assert_variant(&error, index);
                    original.assert_preserved(&error);
                }
            }
        }
    }
    let success: eros::Result<u8, All> = Ok(13);
    let output: eros::Result<u8, Output> =
        success.try_recover::<Selected, _, _, _>(|_| panic!("handled success"));
    assert_eq!(output.unwrap(), 13);
}

#[cfg_attr(test, test)]
pub fn fallible_chain_replaces_reintroduces_maps_and_recovers() {
    let input: eros::Result<u8, (Connect, Read)> = Err(Connect).union();
    let fallback: eros::Result<u8, (Read, Fallback)> =
        input.try_recover::<Connect, _, _, _>(|error| {
            assert_eq!(error.into_single(), Connect);
            Err(Fallback).union()
        });
    let original = Snapshot::new(fallback.as_ref().unwrap_err());
    let retried: eros::Result<u8, (Fallback, Read)> =
        fallback.try_recover(|error: ErrorUnion<(Fallback,)>| {
            original.assert_preserved(&error);
            Err(error.widen())
        });
    let single: eros::Result<u8, (Fallback,)> =
        retried.recover::<Read, _>(|_| panic!("wrong handler"));
    let mapped: eros::Result<u8, (Decode,)> = single.map_err(|error| {
        original.assert_preserved(&error);
        error.map_single(|_: Fallback| Decode)
    });
    #[cfg(feature = "location")]
    assert!(core::ptr::eq(
        original.location,
        mapped.as_ref().unwrap_err().location()
    ));
    assert_eq!(
        mapped
            .recover::<Decode, _>(|error| {
                assert_eq!(error.into_single(), Decode);
                42
            })
            .into_value(),
        42
    );
}

#[cfg_attr(test, test)]
pub fn any_union_handles_plain_typed_and_already_erased_results() {
    let plain = [
        Err::<u8, _>(Connect).any_union(),
        Err::<u8, _>(Read).any_union(),
        Err::<u8, _>(Decode).any_union(),
        Err::<u8, _>(Validate).any_union(),
    ];
    for (index, result) in plain.into_iter().enumerate() {
        assert_variant(&result.unwrap_err(), index);
    }
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let typed: eros::Result<u8, All> = Err(error);
        let error = typed.any_union().any_union().unwrap_err();
        snapshot.assert_preserved(&error);
        assert_variant(&error, index);
        // Erasing a union must preserve its root rather than wrap an adapter.
        assert!(error.inner().source().is_none());
    }
    assert_eq!(Ok::<u8, Connect>(13).any_union().unwrap(), 13);
    let success: eros::Result<u8, All> = Ok(13);
    assert_eq!(success.any_union().any_union().unwrap(), 13);
}

#[cfg_attr(test, test)]
pub fn erased_narrow_can_recover_a_typed_set_then_erase_again() {
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let erased: ErrorUnion = error.into();
        let erased = erased.downcast_inner::<u64>().unwrap_err();
        let erased = erased.narrow::<Fallback, _>().unwrap_err();
        let erased = erased.narrow::<(), _>().unwrap_err();
        let split: Result<ErrorUnion<Selected>, ErrorUnion> = erased.narrow::<Selected, _>();
        assert_eq!(split.is_ok(), matches!(index, 0 | 2));
        let erased: ErrorUnion = match split {
            Ok(selected) => {
                snapshot.assert_preserved(&selected);
                assert_variant(&selected, index);
                selected.into()
            }
            Err(remainder) => remainder,
        };
        snapshot.assert_preserved(&erased);
        assert_variant(&erased, index);
        let typed: ErrorUnion<(Validate, Decode, Read, Connect)> = erased
            .narrow::<(Validate, Decode, Read, Connect), _>()
            .unwrap();
        snapshot.assert_preserved(&typed);
        assert_variant(&typed, index);
    }
}

#[cfg_attr(test, test)]
pub fn erased_result_narrow_and_recovery_route_every_variant() {
    for mode in 0..=4 {
        let result = pipeline(mode).any_union();
        let split: Result<ErrorUnion<Selected>, eros::Result<u8>> = result.narrow::<Selected, _>();
        assert_eq!(split.is_ok(), matches!(mode, 0 | 2));
        match split {
            Ok(selected) => assert_variant(&selected, mode),
            Err(Err(remaining)) => assert_variant(&remaining, mode),
            Err(Ok(value)) => {
                assert_eq!(mode, 4);
                assert_eq!(value, 13);
            }
        }
        let mut calls = [0; 2];
        let value = pipeline(mode)
            .any_union()
            .recover(|error: ErrorUnion<Selected>| {
                calls[0] += 1;
                assert_variant(&error, mode);
                20 + mode as u8
            })
            .recover(|error: ErrorUnion<Remaining>| {
                calls[1] += 1;
                assert_variant(&error, mode);
                20 + mode as u8
            })
            .unwrap();
        assert_eq!(
            calls,
            [
                usize::from(matches!(mode, 0 | 2)),
                usize::from(matches!(mode, 1 | 3))
            ]
        );
        assert_eq!(value, if mode == 4 { 13 } else { 20 + mode as u8 });
    }
}

#[cfg_attr(test, test)]
pub fn erased_try_recover_keeps_unknown_errors_and_replacement_types() {
    for succeeds in [false, true] {
        for mode in 0..=4 {
            let mut calls = 0;
            let result: eros::Result<u8> =
                pipeline(mode)
                    .any_union()
                    .try_recover(|error: ErrorUnion<Selected>| {
                        calls += 1;
                        assert_variant(&error, mode);
                        if succeeds {
                            Ok(42)
                        } else {
                            Err(Fallback).any_union()
                        }
                    });
            let selected = matches!(mode, 0 | 2);
            assert_eq!(calls, usize::from(selected));
            match result {
                Ok(value) => assert_eq!(
                    value,
                    if mode == 4 {
                        13
                    } else {
                        assert!(selected && succeeds);
                        42
                    }
                ),
                Err(error) if selected => {
                    assert!(!succeeds);
                    assert!(error.is_inner::<Fallback>());
                }
                Err(error) => assert_variant(&error, mode),
            }
        }
    }
}

#[cfg_attr(test, test)]
pub fn other_error_partitions_and_recovers_without_an_allocator() {
    type Open = (Connect, OtherError);
    for (index, error) in errors().into_iter().enumerate() {
        let snapshot = Snapshot::new(&error);
        let error: ErrorUnion<Open> = error.widen();
        let split: Result<ErrorUnion, ErrorUnion<(Connect,)>> = error.narrow::<OtherError, _>();
        assert_eq!(split.is_ok(), index != 0);
        match split {
            Ok(other) => {
                snapshot.assert_preserved(&other);
                assert_variant(&other, index);
            }
            Err(named) => {
                snapshot.assert_preserved(&named);
                assert_eq!(named.into_single(), Connect);
            }
        }
    }
    for mode in 0..=4 {
        let result: eros::Result<u8, Open> = pipeline(mode).widen();
        let result: eros::Result<u8, (Connect,)> = result.recover::<OtherError, _>(|other| {
            assert_variant(&other, mode);
            42
        });
        let value = result.recover::<Connect, _>(|_| 20).into_value();
        assert_eq!(
            value,
            if mode == 0 {
                20
            } else if mode == 4 {
                13
            } else {
                42
            }
        );
    }
    let result: eros::Result<u8, Open> = Err(Read).union();
    let result: eros::Result<u8, (Fallback, Connect)> =
        result.try_recover::<OtherError, _, _, _>(|_| Err(Fallback).union());
    let error = result.unwrap_err().narrow::<Fallback, _>().unwrap();
    assert_eq!(error, Fallback);
}

#[cfg_attr(test, test)]
pub fn unit_errors_can_display_static_text_without_a_string_payload() {
    struct Buffer {
        bytes: [u8; 64],
        len: usize,
    }
    impl Write for Buffer {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let end = self.len + text.len();
            if end > self.bytes.len() {
                return Err(fmt::Error);
            }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len = end;
            Ok(())
        }
    }
    assert_eq!(core::mem::size_of::<Connect>(), 0);
    assert!(core::mem::size_of::<&'static str>() > 0);
    let error: ErrorUnion<All> = ErrorUnion::new(Connect);
    let mut buffer = Buffer {
        bytes: [0; 64],
        len: 0,
    };
    write!(&mut buffer, "{error}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], b"connection failed");
}

// Called by a no_std executable with no allocator, in addition to individual
// host/Miri tests. Keep every scenario on this path as coverage grows.
#[cfg_attr(test, allow(dead_code))]
pub fn run_checks() {
    union_widen_and_question_mark_compose_across_functions();
    group_narrow_partitions_every_variant_then_widens_back();
    empty_full_and_single_narrow_targets_compose();
    result_narrow_routes_selected_remaining_and_success_branches();
    recover_chains_remove_multiple_errors_and_finish_with_into_value();
    try_recover_groups_replace_errors_or_keep_reordered_remainders();
    fallible_chain_replaces_reintroduces_maps_and_recovers();
    any_union_handles_plain_typed_and_already_erased_results();
    erased_narrow_can_recover_a_typed_set_then_erase_again();
    erased_result_narrow_and_recovery_route_every_variant();
    erased_try_recover_keeps_unknown_errors_and_replacement_types();
    other_error_partitions_and_recovers_without_an_allocator();
    unit_errors_can_display_static_text_without_a_string_payload();
}
