#[cfg(feature = "alloc")]
use core::error::Error;
use core::{
    cell::Cell,
    fmt::{self, Write},
    hash::{Hash, Hasher},
};
#[cfg(feature = "alloc")]
use eros::{Erase, ReshapeUnion};
use eros::{ErrorUnion, MsgError};

#[cfg(feature = "alloc")]
#[eros::error_enum(Owned)]
#[eros::error_enum_ref(Shared)]
#[eros::error_enum_mut(Mutable)]
type Messages = (MsgError, fmt::Error);

#[cfg(feature = "alloc")]
#[cfg_attr(test, test)]
pub fn static_error_references_preserve_identity() {
    #[derive(Debug)]
    struct Message(&'static str);
    impl fmt::Display for Message {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }
    impl Error for Message {}

    static MESSAGE: Message = Message("message");
    assert_eq!(size_of::<Message>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<&Message>(), size_of::<usize>());

    let error: ErrorUnion<(&'static Message, fmt::Error)> = ErrorUnion::new(&MESSAGE);
    let erased: ErrorUnion = error.into();
    assert!(!erased.is_inner::<Message>());
    assert!(core::ptr::eq(
        *erased.downcast_inner_ref::<&'static Message>().unwrap(),
        &MESSAGE,
    ));
    let message = erased
        .narrow::<(&'static Message,), _>()
        .unwrap()
        .into_single();
    assert!(core::ptr::eq(message, &MESSAGE));
    assert_eq!(message.0, "message");

    // A borrowed constant expression can also be promoted to static storage.
    let error: ErrorUnion<(&'static Message,)> = ErrorUnion::new(&Message("promoted"));
    assert_eq!(error.into_single().0, "promoted");
}

// Fixed output storage keeps these checks usable in a freestanding executable.
fn assert_message(error: &ErrorUnion, expected: &str) {
    struct Buffer {
        bytes: [u8; 128],
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
    let mut buffer = Buffer {
        bytes: [0; 128],
        len: 0,
    };
    write!(&mut buffer, "{error}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], expected.as_bytes());
    assert!(error.inner().source().is_none());
    #[cfg(not(feature = "alloc"))]
    assert_eq!(size_of_val(error.inner()), 0);
}

#[cfg_attr(test, test)]
pub fn literals_support_escaping_and_forwarding() {
    macro_rules! forwarded {
        ($message:expr) => {
            eros::error!($message)
        };
    }
    for (mut error, expected) in [
        (eros::error!("literal"), "literal"),
        (eros::error!("literal",), "literal"),
        (eros::error!(""), ""),
        (eros::error!("{{id}}"), "{id}"),
        (
            eros::error!(r#"{{{{"id": "日本語"}}}}"#),
            "{{\"id\": \"日本語\"}}",
        ),
        (eros::error!("\u{7b}\u{7b}id\u{7d}\u{7d}"), "{id}"),
        (forwarded!("forwarded {{message}}"), "forwarded {message}"),
    ] {
        assert_message(&error, expected);
        #[cfg(feature = "alloc")]
        assert_eq!(
            error.downcast_inner_ref::<MsgError>().unwrap().as_str(),
            expected
        );
        #[cfg(not(feature = "alloc"))]
        assert!(!error.is_inner::<MsgError>());
        let _ = error.inner_mut();
        // A failed narrowing must retain the anonymous concrete error.
        let error = error.narrow::<(fmt::Error,), _>().unwrap_err();
        assert_message(&error, expected);
    }
}

#[cfg_attr(test, test)]
pub fn literal_sites_keep_independent_messages() {
    fn first() -> ErrorUnion {
        eros::error!("first")
    }
    fn second() -> ErrorUnion {
        eros::error!("second")
    }
    let a = first();
    let b = second();
    let c = first();
    assert_message(&a, "first");
    assert_message(&b, "second");
    assert_message(&c, "first");
    let kind = |e: &ErrorUnion| core::any::Any::type_id(e.inner());
    assert_eq!(kind(&a), kind(&c));
    #[cfg(not(feature = "alloc"))]
    assert_ne!(kind(&a), kind(&b));
}

#[cfg_attr(test, test)]
pub fn bail_and_ensure_support_literal_messages() {
    fn bail() -> eros::Result<()> {
        eros::bail!("literal",);
    }
    fn ensure(ok: bool, calls: &Cell<u8>) -> eros::Result<u8> {
        eros::ensure!(
            {
                calls.set(calls.get() + 1);
                ok
            },
            "condition {{failed}}",
        );
        Ok(42)
    }
    let calls = Cell::new(0);
    assert_eq!(ensure(true, &calls).unwrap(), 42);
    assert_message(&ensure(false, &calls).unwrap_err(), "condition {failed}");
    assert_eq!(calls.get(), 2);
    assert_message(&bail().unwrap_err(), "literal");
}

#[cfg(feature = "alloc")]
#[cfg_attr(test, test)]
pub fn message_errors_compose_with_other_errors_and_generated_enums() {
    fn operation() -> eros::Result<(), (MsgError,)> {
        eros::bail!("original");
    }
    let result: eros::Result<(), Messages> = operation().widen();
    let erased = result.erase().unwrap_err();
    let mut error = erased.narrow::<Messages, _>().unwrap();
    assert!(matches!(Shared::from(&error), Shared::Msg(message) if message.as_str() == "original"));
    static REPLACEMENT: &str = "replacement";
    match Mutable::from(&mut error) {
        Mutable::Msg(message) => *message = MsgError::from_static_ref(&REPLACEMENT),
        _ => panic!("wrong variant"),
    }
    let error = match Owned::from(error) {
        Owned::Msg(message) => message,
        _ => panic!("wrong variant"),
    };
    let error: ErrorUnion<(MsgError,)> = error.into();
    let result: eros::Result<u8, Messages> = Err(error.widen());
    let result: eros::Result<u8, (fmt::Error,)> =
        result.try_recover(|error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.as_single().as_str(), "replacement");
            Err(error.map_single(|_| fmt::Error))
        });
    assert_eq!(result.recover::<fmt::Error, _>(|_| 7).into_value(), 7);
}

#[cfg_attr(test, test)]
pub fn static_messages_compare_and_hash_by_text_and_format_without_a_heap() {
    static FIRST: &str = "same";
    static SECOND: &str = "same";
    static OTHER: &str = "z";
    let first = MsgError::from_static_ref(&FIRST);
    let second = MsgError::from_static_ref(&SECOND);
    assert_eq!(first, second);
    assert!(first < MsgError::from_static_ref(&OTHER));
    struct Digest(u64);
    impl Hasher for Digest {
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes {
                self.0 = self.0.wrapping_mul(31).wrapping_add(u64::from(*byte));
            }
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }
    let mut a = Digest(0);
    let mut b = Digest(0);
    first.hash(&mut a);
    second.hash(&mut b);
    assert_eq!(a.finish(), b.finish());
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
    let error = eros::error!("message");
    let mut buffer = Buffer {
        bytes: [0; 64],
        len: 0,
    };
    write!(&mut buffer, "{error} {error:#?}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], b"message error: message");
}

#[cfg(feature = "error_location")]
#[cfg_attr(test, test)]
pub fn literal_macros_capture_the_call_site_and_preserve_it_through_reshaping() {
    let expected = line!() + 1;
    let error = eros::error!("located");
    assert_eq!(error.location().line(), expected);
    let location = error.location();
    let error = error.narrow::<(fmt::Error,), _>().unwrap_err();
    assert!(core::ptr::eq(error.location(), location));
    fn bail(line: &Cell<u32>) -> eros::Result<()> {
        line.set(line!() + 1);
        eros::bail!("located bail");
    }
    fn ensure(line: &Cell<u32>) -> eros::Result<()> {
        line.set(line!() + 1);
        eros::ensure!(false, "located ensure");
        Ok(())
    }
    for operation in [bail, ensure] {
        let line = Cell::new(0);
        let error = operation(&line).unwrap_err();
        assert_eq!(error.location().file(), file!());
        assert_eq!(error.location().line(), line.get());
    }
}

#[cfg_attr(test, allow(dead_code))]
pub fn run_checks() {
    #[cfg(feature = "alloc")]
    static_error_references_preserve_identity();
    literals_support_escaping_and_forwarding();
    literal_sites_keep_independent_messages();
    bail_and_ensure_support_literal_messages();
    #[cfg(feature = "alloc")]
    message_errors_compose_with_other_errors_and_generated_enums();
    static_messages_compare_and_hash_by_text_and_format_without_a_heap();
    #[cfg(feature = "error_location")]
    literal_macros_capture_the_call_site_and_preserve_it_through_reshaping();
}
