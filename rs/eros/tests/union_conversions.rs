use eros::{AnyError, ErrorUnion, MsgError, SendSyncError, prelude::*};
use std::{fmt, io, num::ParseIntError};

type Input = (MsgError, fmt::Error, io::Error);
type Reordered = (io::Error, MsgError, fmt::Error);
type Expanded = (ParseIntError, fmt::Error, MsgError, io::Error);

#[test]
fn union_inserts_each_concrete_error_at_its_declared_position() {
    let errors: [eros::Result<(), Input>; 3] = [
        Err(MsgError::from(String::from("owned message"))).union(),
        Err(fmt::Error).union::<Input, _>().context("format response"),
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")).union(),
    ];
    for (index, result) in errors.into_iter().enumerate() {
        let error = result.unwrap_err();
        #[cfg(feature = "context")]
        assert_eq!(error.contexts().len(), usize::from(index == 1));
        match (index, error.into_enum()) {
            (0, eros::E3::A(error)) => assert_eq!(error.as_str(), "owned message"),
            (1, eros::E3::B(fmt::Error)) => {}
            (2, eros::E3::C(error)) => assert_eq!(error.kind(), io::ErrorKind::PermissionDenied),
            _ => panic!("union changed the concrete error or its position"),
        }
    }
}

#[test]
fn union_supports_singleton_and_erased_destinations_without_losing_owned_payloads() {
    let message = String::from("owned error");
    let original = message.as_ptr();
    let result: eros::Result<(), (MsgError,)> = Err(MsgError::from(message)).union();
    assert_eq!(
        result.unwrap_err().into_single().as_str().as_ptr(),
        original
    );

    let source = Box::new(MsgError::from("source"));
    let original = source.as_ref() as *const MsgError;
    let source: Box<dyn std::error::Error + Send + Sync> = source;
    let result: eros::Result<()> = Err(io::Error::other(source)).union();
    let error = result.unwrap_err().downcast_inner::<io::Error>().unwrap();
    let source = error.get_ref().unwrap().downcast_ref::<MsgError>().unwrap();
    assert_eq!(source as *const MsgError, original);
    assert_eq!(source.as_str(), "source");
}

#[test]
fn widen_preserves_every_variant_through_identity_reordering_expansion_and_erasure() {
    for as_result in [false, true] {
        let errors: [ErrorUnion<Input>; 3] = [
            ErrorUnion::new(MsgError::from(String::from("message"))),
            ErrorUnion::new(fmt::Error),
            ErrorUnion::new(io::Error::other(MsgError::from("source"))),
        ];
        for (index, error) in errors.into_iter().enumerate() {
            let error = error.context("inner operation").context("outer operation");
            #[cfg(feature = "user_context")]
            let error = error.user_context("user message");
            let original = error.inner() as *const dyn SendSyncError as *const ();
            let report = format!("{error:?}");
            #[cfg(feature = "diagnostic")]
            let diagnostic = error.to_debug_json();

            let error: ErrorUnion<Expanded> = if as_result {
                let result: eros::Result<(), Input> = Err(error);
                let result: eros::Result<(), Input> = result.widen();
                let result: eros::Result<(), Reordered> = result.widen();
                let result: eros::Result<(), Expanded> = result.widen();
                result.unwrap_err()
            } else {
                let error: ErrorUnion<Input> = error.widen();
                let error: ErrorUnion<Reordered> = error.widen();
                error.widen()
            };
            match (index, error.as_enum()) {
                (0, eros::E4::C(error)) => assert_eq!(error.as_str(), "message"),
                (1, eros::E4::B(_)) => {}
                (2, eros::E4::D(error)) => assert_eq!(error.kind(), io::ErrorKind::Other),
                _ => panic!("widen changed the active variant"),
            }
            let error: ErrorUnion<AnyError> = if as_result {
                let result: eros::Result<(), Expanded> = Err(error);
                let result: eros::Result<()> = result.any_union();
                result.widen::<AnyError, _>().unwrap_err()
            } else {
                let error: ErrorUnion = error.into();
                error.widen()
            };
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), report);
            #[cfg(feature = "diagnostic")]
            assert_eq!(error.to_debug_json(), diagnostic);
        }
    }
}

#[test]
fn successful_conversions_accept_owned_and_borrowed_values_without_extra_bounds() {
    // Deliberately no Debug, Clone, Copy, Send, Sync, or 'static bounds.
    struct Value<'a>(&'a mut String, std::rc::Rc<()>);
    let mut storage = String::from("borrowed");
    let token = std::rc::Rc::new(());
    let result: Result<Value<'_>, MsgError> = Ok(Value(&mut storage, token.clone()));
    let result: eros::Result<_, Input> = result.union();
    let result: eros::Result<_, Reordered> = result.widen();
    let result: eros::Result<_, Expanded> = result.widen();
    let result: eros::Result<_> = result.any_union();
    let value = result.unwrap();
    assert!(std::rc::Rc::ptr_eq(&value.1, &token));
    value.0.push_str(" success");
    assert_eq!(storage, "borrowed success");

    let result: eros::Result<Box<u8>, ()> = Ok(Box::new(7));
    let original = result.as_ref().unwrap().as_ref() as *const u8;
    let result: eros::Result<_, ()> = result.widen();
    let result: eros::Result<_, Input> = result.widen();
    let value = result.unwrap();
    assert_eq!(value.as_ref() as *const u8, original);
    assert_eq!(*value, 7);

    let result: eros::Result<&str, ()> = Ok(&storage);
    let result: eros::Result<_> = result.widen();
    assert_eq!(result.unwrap(), "borrowed success");
}

#[cfg(feature = "location")]
#[test]
fn union_captures_the_call_site_and_widen_keeps_it() {
    let original: Result<(), MsgError> = Err(MsgError::from("root"));
    let call_line = line!() + 1;
    let result: eros::Result<(), Input> = original.union();
    let error = result.unwrap_err();
    assert_eq!(error.location().file(), file!());
    assert_eq!(error.location().line(), call_line);
    let error: ErrorUnion<Expanded> = error.widen();
    assert_eq!(error.location().line(), call_line);
    let result: eros::Result<(), Expanded> = Err(error);
    let result: eros::Result<(), Expanded> = result.widen();
    assert_eq!(result.unwrap_err().location().line(), call_line);
}

#[derive(Debug)]
struct Payload<const N: u8>(String);

impl<const N: u8> fmt::Display for Payload<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {N}: {}", self.0)
    }
}
impl<const N: u8> std::error::Error for Payload<N> {}

macro_rules! check_large_set {
    ($($n:literal),+) => {
        #[test]
        fn every_position_in_a_maximum_size_union_can_be_widened_and_narrowed() {
            type Set = ($(Payload<$n>,)+);
            $(
                let payload = String::from("owned payload");
                let original = payload.as_ptr();
                let result: eros::Result<(), (Payload<$n>,)> = Err(Payload::<$n>(payload)).union();
                let result: eros::Result<(), Set> = result.widen();
                let selected: ErrorUnion<(Payload<$n>,)> = result.narrow::<(Payload<$n>,), _>().unwrap();
                assert_eq!(selected.0.as_ptr(), original);
                let error: ErrorUnion<Set> = selected.widen();
                let value: Payload<$n> = error.narrow::<Payload<$n>, _>().unwrap();
                assert_eq!(value.0, "owned payload");
                assert_eq!(value.0.as_ptr(), original);
            )+
        }
    };
}

check_large_set!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25
);
