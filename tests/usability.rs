use std::fmt::Display;

use eros::{traced, AnyError, ErrorUnion, IntoDynTracedError, TracedError};

#[derive(Debug, PartialEq, Eq)]
struct NotEnoughMemory;

impl Display for NotEnoughMemory {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Not enough memory")
    }
}

// #[derive(Debug)]
struct Timeout;

impl Display for Timeout {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Timeout")
    }
}

// #[derive(Debug)]
struct RetriesExhausted;

impl Display for RetriesExhausted {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Retries exhausted")
    }
}

#[test]
fn retry_example() {
    fn does_stuff() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout)>> {
        let _allocation = match allocates() {
            Ok(a) => a,
            Err(e) => return Err(e.widen()),
        };

        let _chat = match chats() {
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
    let o_1: ErrorUnion<(u32, String)> = ErrorUnion::new(5_u32);
    let _narrowed_1: u32 = o_1.narrow::<u32, _>().unwrap();

    let o_2: ErrorUnion<(String, u32)> = ErrorUnion::new(5_u32);
    let _narrowed_2: u32 = o_2.narrow::<u32, _>().unwrap();

    let o_3: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    let _narrowed_3: ErrorUnion<(String,)> = o_3.narrow::<u32, _>().unwrap_err();

    let o_4: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());

    let _: String = o_4.narrow().unwrap();

    let o_5: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    o_5.narrow::<String, _>().unwrap();

    let o_6: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    let o_7: ErrorUnion<(u32, String)> = o_6.widen();
    let o_8: ErrorUnion<(String, u32)> = o_7.subset().unwrap();
    let _: ErrorUnion<(u32, String)> = o_8.subset().unwrap();

    let o_9: ErrorUnion<(u8, u16, u32)> = ErrorUnion::new(3_u32);
    let _: Result<ErrorUnion<(u16,)>, ErrorUnion<(u8, u32)>> = o_9.subset();
    let o_10: ErrorUnion<(u8, u16, u32)> = ErrorUnion::new(3_u32);
    let _: Result<u16, ErrorUnion<(u8, u32)>> = o_10.narrow();
}

#[test]
fn debug() {
    use std::error::Error;
    use std::io;

    let o_1: ErrorUnion<(u32, String)> = ErrorUnion::new(5_u32);

    // Debug is implemented if all types in the type set implement Debug
    dbg!(&o_1);

    // Display is implemented if all types in the type set implement Display
    println!("{}", o_1);

    type E = io::Error;
    let e = io::Error::new(io::ErrorKind::Other, "wuaaaaahhhzzaaaaaaaa");

    let o_2: ErrorUnion<(E,)> = ErrorUnion::new(e);

    // std::error::Error is implemented if all types in the type set implement it
    dbg!(o_2.source());

    let o_3: ErrorUnion<(u32, String)> = ErrorUnion::new("hey".to_string());
    dbg!(o_3);
}

#[test]
fn multi_match() {
    use eros::E2;

    let o_1: ErrorUnion<(u32, String)> = ErrorUnion::new(5_u32);
    let mut is_hit = false;
    match o_1.as_enum() {
        E2::A(u) => {
            is_hit = true;
        }
        E2::B(s) => {
            unreachable!()
        }
    }
    assert!(is_hit);
    is_hit = false;
    match o_1.to_enum() {
        E2::A(u) => {
            is_hit = true;
        }
        E2::B(s) => {
            unreachable!()
        }
    }
    assert!(is_hit);
}

#[test]
fn multi_narrow() {
    use eros::E2;

    struct Timeout {}
    struct Backoff {}

    let o_1: ErrorUnion<(u8, u16, u32, u64, u128)> = ErrorUnion::new(5_u32);

    let _narrow_res: Result<ErrorUnion<(u8, u128)>, ErrorUnion<(u16, u32, u64)>> = o_1.subset();

    let o_2: ErrorUnion<(u8, u16, Backoff, Timeout, u32, u64, u128)> = ErrorUnion::new(Timeout {});

    let mut is_hit: bool = false;
    match o_2.subset::<(Timeout, Backoff), _>().unwrap().to_enum() {
        E2::A(Timeout {}) => {
            is_hit = true;
        }
        E2::B(Backoff {}) => {
            unreachable!()
        }
    }
    assert!(is_hit);
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
struct MyErrorType(Box<dyn AnyError>);

impl Display for MyErrorType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "MyErrorType: {}", self.0)
    }
}

impl std::error::Error for MyErrorType {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn map_inner() {
    let error: TracedError<std::io::Error> =
        TracedError::new(std::io::Error::new(std::io::ErrorKind::Other, "wuaaaaahhh"));
    let error: TracedError<IoErrorWrapper> = error.map(|e| IoErrorWrapper(e));
    println!("{error}");
    let message = error.to_string();
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedError<MyErrorType> = error.map(|e| MyErrorType(Box::new(e)));
    println!("{error}");
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedError =
        TracedError::boxed(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
    let error: TracedError<MyErrorType> = error.map(|e| MyErrorType(e));
    println!("{error}");
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
}

#[test]
#[should_panic]
fn double_traced_dyn_error() {
    let error = traced!("Error");
    let _error = error.traced_dyn();
}

#[test]
#[cfg_attr(not(feature = "min_specialization"), should_panic)]
fn double_traced_dyn_result() {
    let error = traced!("Error");
    let result: Result<(), TracedError> = Err(error);
    let _result = result.traced_dyn();
}

#[test]
fn source_lives_long_enough() {
    enum Wrapper {
        TracedError(TracedError),
    }

    let error = traced!("Error");
    let wrapper_binding = Wrapper::TracedError(error);
    let wrapper = &wrapper_binding;
    let source = match wrapper {
        // Wrapper::TracedError(traced_error) => std::error::Error::source(traced_error), // does not work
        Wrapper::TracedError(traced_error) => traced_error.source(),
    };
    let _source = source;
}

#[test]
fn x() {
    let error = traced!("Root");
    let error = error.context("the context before");
    let error = error.into_any_error();
    let error = TracedError::from_dyn_error(error).unwrap();
    let error = error.context("The context after");
    let message = error.to_string();
    assert!(
        !message.contains(
            r#"Context:
- the context before
- The context after"#
        ),
        "Expected context to be correct:\n{}",
        message
    );
}
