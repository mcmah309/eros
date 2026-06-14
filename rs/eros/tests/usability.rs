use std::fmt::Display;

use eros::{ErrorUnion, IntoUnion, SendSyncError, error};

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
    fn does_stuff() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout)>> {
        let _allocation: () = match allocates() {
            Ok(a) => a,
            Err(e) => return Err(e.widen()),
        };

        let _chat: () = match chats() {
            Ok(c) => c,
            Err(e) => return Err(ErrorUnion::new(e)),
        };

        Ok(())
    }

    fn allocates() -> Result<(), ErrorUnion<(NotEnoughMemory,)>> {
        let result: Result<(), NotEnoughMemory> = Err(NotEnoughMemory);

        result?;

        Ok(())
    }

    fn chats() -> Result<(), Timeout> {
        Err(Timeout)
    }

    fn inner() -> Result<(), ErrorUnion<(NotEnoughMemory, RetriesExhausted)>> {
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

        Err(ErrorUnion::new(RetriesExhausted))
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
    let o_1: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);
    let _narrowed_1: NotEnoughMemory = o_1.narrow::<NotEnoughMemory, _>().unwrap();

    let o_2: ErrorUnion<(Timeout, NotEnoughMemory)> = ErrorUnion::new(NotEnoughMemory);
    let _narrowed_2: NotEnoughMemory = o_2.narrow::<NotEnoughMemory, _>().unwrap();

    let o_3: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
    let _narrowed_3: ErrorUnion<(Timeout,)> = o_3.narrow::<NotEnoughMemory, _>().unwrap_err();

    let o_4: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);

    let _: Timeout = o_4.narrow().unwrap();

    let o_5: ErrorUnion<(Timeout, NotEnoughMemory)> = ErrorUnion::new(Timeout);
    o_5.narrow::<Timeout, _>().unwrap();

    let o_6: ErrorUnion<(Timeout, NotEnoughMemory)> = ErrorUnion::new(Timeout);
    let o_7: ErrorUnion<(NotEnoughMemory, Timeout)> = o_6.widen();
    let o_8: ErrorUnion<(Timeout, NotEnoughMemory)> = o_7.subset().unwrap();
    let _: ErrorUnion<(NotEnoughMemory, Timeout)> = o_8.subset().unwrap();

    let o_9: ErrorUnion<(std::sync::mpsc::RecvError, std::fmt::Error, NotEnoughMemory)> =
        ErrorUnion::new(NotEnoughMemory);
    let _: Result<
        ErrorUnion<(std::fmt::Error,)>,
        ErrorUnion<(std::sync::mpsc::RecvError, NotEnoughMemory)>,
    > = o_9.subset();
    let o_10: ErrorUnion<(std::sync::mpsc::RecvError, std::fmt::Error, NotEnoughMemory)> =
        ErrorUnion::new(NotEnoughMemory);
    let _: Result<std::fmt::Error, ErrorUnion<(std::sync::mpsc::RecvError, NotEnoughMemory)>> =
        o_10.narrow();
}

#[test]
fn debug() {
    use std::io;

    let o_1: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);

    // Debug is implemented if all types in the type set implement Debug
    dbg!(&o_1);

    // Display is implemented if all types in the type set implement Display
    println!("{}", o_1);

    type E = io::Error;
    let e = io::Error::other("wuaaaaahhhzzaaaaaaaa");

    let o_2: ErrorUnion<(E,)> = ErrorUnion::new(e);

    // std::error::Error is implemented if all types in the type set implement it
    dbg!(o_2.source());

    let o_3: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
    dbg!(o_3);
}

#[test]
fn multi_match() {
    use eros::E2;

    let o_1: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);
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

    let o_1: ErrorUnion<(
        std::sync::mpsc::RecvError,
        std::fmt::Error,
        NotEnoughMemory,
        std::io::Error,
        std::cell::BorrowError,
    )> = ErrorUnion::new(NotEnoughMemory);

    #[allow(clippy::type_complexity)]
    let _narrow_res: Result<
        ErrorUnion<(std::sync::mpsc::RecvError, std::cell::BorrowError)>,
        ErrorUnion<(std::fmt::Error, NotEnoughMemory, std::io::Error)>,
    > = o_1.subset();

    let o_2: ErrorUnion<(
        std::sync::mpsc::RecvError,
        std::fmt::Error,
        Timeout,
        NotEnoughMemory,
        std::io::Error,
        std::cell::BorrowError,
    )> = ErrorUnion::new(Timeout);

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
    let error: ErrorUnion<(std::io::Error,)> = ErrorUnion::new(std::io::Error::other("wuaaaaahhh"));
    let error: ErrorUnion<(IoErrorWrapper,)> = error.map(IoErrorWrapper);
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: ErrorUnion<(MyErrorType,)> = error.map(|e| MyErrorType(Box::new(e.0)));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: ErrorUnion<(std::io::Error,)> = ErrorUnion::new(std::io::Error::other("io error"));
    let error: ErrorUnion<(MyErrorType,)> = error.map(|e| MyErrorType(Box::new(e)));
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
        ErrorUnion(ErrorUnion),
    }

    let error = error!("Error");
    let wrapper_binding = Wrapper::ErrorUnion(error);
    let wrapper = &wrapper_binding;
    let source = match wrapper {
        // Wrapper::ErrorUnion(error_union) => std::error::Error::source(&error_union), // does not work since does not implement error directly
        Wrapper::ErrorUnion(error_union) => error_union.source(),
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

    fn regular_union() -> ErrorUnion<(std::io::Error,)> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        error.into()
    }

    fn result_union() -> Result<(), ErrorUnion<(std::io::Error,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        Err(error).into_union()
    }

    fn mapped_result_union() -> Result<(), ErrorUnion<(MyCustomError,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        Err(error).map_err(|e| ErrorUnion::new(e.into()))
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
