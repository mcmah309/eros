use std::fmt::Display;

use eros::{traced, SendSyncError, TracedUnion, IntoUnion};

#[derive(Debug, PartialEq, Eq)]
struct NotEnoughMemory;

impl Display for NotEnoughMemory {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Not enough memory")
    }
}

impl std::error::Error for NotEnoughMemory {}

#[derive(Debug)]
struct Timeout;

impl Display for Timeout {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Timeout")
    }
}

impl std::error::Error for Timeout {}

#[derive(Debug)]
struct RetriesExhausted;

impl Display for RetriesExhausted {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Retries exhausted")
    }
}

impl std::error::Error for RetriesExhausted {}

#[test]
fn retry_example() {
    fn does_stuff() -> Result<(), TracedUnion<(NotEnoughMemory, Timeout)>> {
        let _allocation: () = match allocates() {
            Ok(a) => a,
            Err(e) => return Err(e.widen()),
        };

        let _chat: () = match chats() {
            Ok(c) => c,
            Err(e) => return Err(TracedUnion::new(e)),
        };

        Ok(())
    }

    fn allocates() -> Result<(), TracedUnion<(NotEnoughMemory,)>> {
        let result: Result<(), NotEnoughMemory> = Err(NotEnoughMemory);

        result?;

        Ok(())
    }

    fn chats() -> Result<(), Timeout> {
        Err(Timeout)
    }

    fn inner() -> Result<(), TracedUnion<(NotEnoughMemory, RetriesExhausted)>> {
        for _ in 0..3 {
            let Err(err) = does_stuff() else {
                return Ok(());
            };

            match err.narrow::<Timeout, _>() {
                Ok(_timeout) => continue,
                Err(not_enough_memory_union) => {
                    return Err(not_enough_memory_union.widen());
                }
            }
        }

        Err(TracedUnion::new(RetriesExhausted))
    }

    let Err(err) = inner() else {
        panic!("Should be error");
    };
    let err = match err.narrow::<RetriesExhausted, _>() {
        Ok(_) => panic!("Should not have RetriesExhausted"),
        Err(err) => err,
    };
    let err = match err.narrow::<NotEnoughMemory, _>() {
        Ok(err) => err,
        Err(_) => panic!("Should have NotEnoughMemory"),
    };
    assert_eq!(err, NotEnoughMemory);
}

#[test]
fn widen_narrow() {
    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(NotEnoughMemory);
    let _narrowed_1: NotEnoughMemory = o_1.narrow::<NotEnoughMemory, _>().unwrap();

    let o_2: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::new(NotEnoughMemory);
    let _narrowed_2: NotEnoughMemory = o_2.narrow::<NotEnoughMemory, _>().unwrap();

    let o_3: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(Timeout);
    let _narrowed_3: TracedUnion<(Timeout,)> = o_3.narrow::<NotEnoughMemory, _>().unwrap_err();

    let o_4: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(Timeout);

    let _: Timeout = o_4.narrow().unwrap();

    let o_5: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::new(Timeout);
    o_5.narrow::<Timeout, _>().unwrap();

    let o_6: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::new(Timeout);
    let o_7: TracedUnion<(NotEnoughMemory, Timeout)> = o_6.widen();
    let o_8: TracedUnion<(Timeout, NotEnoughMemory)> = o_7.subset().unwrap();
    let _: TracedUnion<(NotEnoughMemory, Timeout)> = o_8.subset().unwrap();

    let o_9: TracedUnion<(u8, u16, NotEnoughMemory)> = TracedUnion::new(NotEnoughMemory);
    let _: Result<TracedUnion<(u16,)>, TracedUnion<(u8, NotEnoughMemory)>> = o_9.subset();
    let o_10: TracedUnion<(u8, u16, NotEnoughMemory)> = TracedUnion::new(NotEnoughMemory);
    let _: Result<u16, TracedUnion<(u8, NotEnoughMemory)>> = o_10.narrow();
}

#[test]
fn debug() {
    use std::io;

    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(NotEnoughMemory);

    // Debug is implemented if all types in the type set implement Debug
    dbg!(&o_1);

    // Display is implemented if all types in the type set implement Display
    println!("{}", o_1);

    type E = io::Error;
    let e = io::Error::other("wuaaaaahhhzzaaaaaaaa");

    let o_2: TracedUnion<(E,)> = TracedUnion::new(e);

    // std::error::Error is implemented if all types in the type set implement it
    dbg!(o_2.source());

    let o_3: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(Timeout);
    dbg!(o_3);
}

#[test]
fn multi_match() {
    use eros::E2;

    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::new(NotEnoughMemory);
    match o_1.ref_enum() {
        E2::A(_u) => {}
        E2::B(_s) => {
            unreachable!()
        }
    }
    match o_1.to_enum() {
        E2::A(_u) => {}
        E2::B(_s) => {
            unreachable!()
        }
    }
}

#[test]
fn multi_narrow() {
    use eros::E2;

    let o_1: TracedUnion<(u8, u16, NotEnoughMemory, u64, u128)> = TracedUnion::new(NotEnoughMemory);

    #[allow(clippy::type_complexity)]
    let _narrow_res: Result<TracedUnion<(u8, u128)>, TracedUnion<(u16, NotEnoughMemory, u64)>> =
        o_1.subset();

    let o_2: TracedUnion<(u8, u16, Timeout, NotEnoughMemory, u64, u128)> =
        TracedUnion::new(Timeout);

    match o_2
        .subset::<(Timeout, NotEnoughMemory), _>()
        .unwrap()
        .to_enum()
    {
        E2::A(Timeout {}) => {}
        E2::B(NotEnoughMemory {}) => {
            unreachable!()
        }
    }
}

#[derive(Debug)]
struct IoErrorWrapper(std::io::Error);

impl Display for IoErrorWrapper {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "IoErrorWrapper: {}", self.0)
    }
}

impl std::error::Error for IoErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Debug)]
struct MyErrorType(Box<dyn SendSyncError>);

impl Display for MyErrorType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "MyErrorType")
    }
}

impl std::error::Error for MyErrorType {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn map_inner() {
    let error: TracedUnion<(std::io::Error,)> =
        TracedUnion::new(std::io::Error::other("wuaaaaahhh"));
    let error: TracedUnion<(IoErrorWrapper,)> = error.map(IoErrorWrapper);
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedUnion<(MyErrorType,)> = error.map(|e| MyErrorType(Box::new(e.0)));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedUnion<(std::io::Error,)> = TracedUnion::new(std::io::Error::other("io error"));
    let error: TracedUnion<(MyErrorType,)> = error.map(|e| MyErrorType(Box::new(e)));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
}

// This source check is used to ensure that This works with the `error_set` crate, which uses source.
// We expose the same source method as `std::error::Error` even though the type does not implement it.
#[test]
fn source_lives_long_enough() {
    enum Wrapper {
        TracedUnion(TracedUnion),
    }

    let error = traced!("Error");
    let wrapper_binding = Wrapper::TracedUnion(error);
    let wrapper = &wrapper_binding;
    let source = match wrapper {
        // Wrapper::TracedUnion(traced_union) => std::error::Error::source(&traced_union), // does not work since does not implement error directly
        Wrapper::TracedUnion(traced_error) => traced_error.source(),
    };
    let _source = source;
}

// //************************************************************************//

#[test]
fn union() {
    #[derive(Debug)]
    struct MyCustomError(std::io::Error);

    impl Display for MyCustomError {
        fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(fmt, "MyCustomError: {}", self.0)
        }
    }

    impl std::error::Error for MyCustomError {}

    impl From<std::io::Error> for MyCustomError {
        fn from(value: std::io::Error) -> Self {
            MyCustomError(value)
        }
    }

    fn regular_union() -> TracedUnion<(std::io::Error,)> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        error.into()
    }

    fn result_union() -> Result<(), TracedUnion<(std::io::Error,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        Err(error).into_union()
    }

    fn mapped_result_union() -> Result<(), TracedUnion<(MyCustomError,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        Err(error).map_err(|e| TracedUnion::new(e.into()))
    }

    let error = regular_union();
    assert_eq!(error.into_inner().kind(), std::io::ErrorKind::AddrInUse);
    let result = result_union();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.into_inner().kind(), std::io::ErrorKind::AddrInUse);
    let result = mapped_result_union();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.into_inner().0.kind(), std::io::ErrorKind::AddrInUse);
}
