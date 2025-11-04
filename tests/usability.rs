use std::fmt::Display;

use eros::{traced, IntoUnion, SendSyncError, TracedUnion, Union};

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
        let _allocation = match allocates() {
            Ok(a) => a,
            Err(e) => return Err(e.widen()),
        };

        let _chat = match chats() {
            Ok(c) => c,
            Err(e) => return Err(TracedUnion::error(e)),
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

        Err(TracedUnion::error(RetriesExhausted))
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
    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(NotEnoughMemory);
    let _narrowed_1: NotEnoughMemory = o_1.narrow::<NotEnoughMemory, _>().unwrap();

    let o_2: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::error(NotEnoughMemory);
    let _narrowed_2: NotEnoughMemory = o_2.narrow::<NotEnoughMemory, _>().unwrap();

    let o_3: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(Timeout);
    let _narrowed_3: TracedUnion<(Timeout,)> = o_3.narrow::<NotEnoughMemory, _>().unwrap_err();

    let o_4: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(Timeout);

    let _: Timeout = o_4.narrow().unwrap();

    let o_5: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::error(Timeout);
    o_5.narrow::<Timeout, _>().unwrap();

    let o_6: TracedUnion<(Timeout, NotEnoughMemory)> = TracedUnion::error(Timeout);
    let o_7: TracedUnion<(NotEnoughMemory, Timeout)> = o_6.widen();
    let o_8: TracedUnion<(Timeout, NotEnoughMemory)> = o_7.subset().unwrap();
    let _: TracedUnion<(NotEnoughMemory, Timeout)> = o_8.subset().unwrap();

    let o_9: TracedUnion<(u8, u16, NotEnoughMemory)> = TracedUnion::error(NotEnoughMemory);
    let _: Result<TracedUnion<(u16,)>, TracedUnion<(u8, NotEnoughMemory)>> = o_9.subset();
    let o_10: TracedUnion<(u8, u16, NotEnoughMemory)> = TracedUnion::error(NotEnoughMemory);
    let _: Result<u16, TracedUnion<(u8, NotEnoughMemory)>> = o_10.narrow();
}

#[test]
fn debug() {
    use std::error::Error;
    use std::io;

    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(NotEnoughMemory);

    // Debug is implemented if all types in the type set implement Debug
    dbg!(&o_1);

    // Display is implemented if all types in the type set implement Display
    println!("{}", o_1);

    type E = io::Error;
    let e = io::Error::new(io::ErrorKind::Other, "wuaaaaahhhzzaaaaaaaa");

    let o_2: TracedUnion<(E,)> = TracedUnion::error(e);

    // std::error::Error is implemented if all types in the type set implement it
    dbg!(o_2.source());

    let o_3: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(Timeout);
    dbg!(o_3);
}

#[test]
fn multi_match() {
    use eros::E2;

    let o_1: TracedUnion<(NotEnoughMemory, Timeout)> = TracedUnion::error(NotEnoughMemory);
    let mut is_hit = false;
    match o_1.ref_enum() {
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

    let o_1: TracedUnion<(u8, u16, NotEnoughMemory, u64, u128)> =
        TracedUnion::error(NotEnoughMemory);

    let _narrow_res: Result<TracedUnion<(u8, u128)>, TracedUnion<(u16, NotEnoughMemory, u64)>> =
        o_1.subset();

    let o_2: TracedUnion<(u8, u16, Timeout, NotEnoughMemory, u64, u128)> =
        TracedUnion::error(Timeout);

    let mut is_hit: bool = false;
    match o_2
        .subset::<(Timeout, NotEnoughMemory), _>()
        .unwrap()
        .to_enum()
    {
        E2::A(Timeout {}) => {
            is_hit = true;
        }
        E2::B(NotEnoughMemory {}) => {
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
struct MyErrorType(Box<dyn SendSyncError>);

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
    let error: TracedUnion<(std::io::Error,)> =
        TracedUnion::error(std::io::Error::new(std::io::ErrorKind::Other, "wuaaaaahhh"));
    let error: TracedUnion<(IoErrorWrapper,)> = error.map(|e| IoErrorWrapper(e));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedUnion<(MyErrorType,)> = error.map(|e| MyErrorType(Box::new(e)));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
    let error: TracedUnion =
        TracedUnion::any_error(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
    let error: TracedUnion<(MyErrorType,)> = error.map(|e| MyErrorType(e));
    let message = format!("{:?}", error);
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
}

#[test]
fn source_lives_long_enough() {
    enum Wrapper {
        TracedUnion(TracedUnion),
    }

    let error = traced!("Error");
    let wrapper_binding = Wrapper::TracedUnion(error);
    let wrapper = &wrapper_binding;
    let source = match wrapper {
        // Wrapper::TracedUnion(traced_union) => std::error::Error::source(&traced_union), // does not work
        Wrapper::TracedUnion(traced_error) => traced_error.source(),
    };
    let _source = source;
}

// //************************************************************************//

#[cfg(test)]
mod into_union {
    use eros::{Context, IntoUnion, IntoUnionSingle, TracedUnion, Union};

    #[derive(Debug)]
    pub enum OurError {
        IoError(std::io::Error),
    }
    #[allow(unused_qualifications)]
    impl core::error::Error for OurError {
        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
            match self {
                OurError::IoError(source) => source.source(),
                #[allow(unreachable_patterns)]
                _ => None,
            }
        }
    }
    impl core::fmt::Display for OurError {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            match &*self {
                OurError::IoError(source) => write!(f, "{}", source),
            }
        }
    }
    impl From<std::io::Error> for OurError {
        fn from(error: std::io::Error) -> Self {
            OurError::IoError(error)
        }
    }
    #[derive(Debug)]
    pub enum AnotherError {
        AnotherErrorVariant,
        IoError(std::io::Error),
    }
    #[allow(unused_qualifications)]
    impl core::error::Error for AnotherError {
        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
            match self {
                AnotherError::IoError(source) => source.source(),
                #[allow(unreachable_patterns)]
                _ => None,
            }
        }
    }
    impl core::fmt::Display for AnotherError {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            match &*self {
                AnotherError::AnotherErrorVariant => write!(
                    f,
                    "{}",
                    concat!(
                        stringify!(AnotherError),
                        "::",
                        stringify!(AnotherErrorVariant)
                    )
                ),
                AnotherError::IoError(source) => write!(f, "{}", source),
            }
        }
    }
    impl From<OurError> for AnotherError {
        fn from(error: OurError) -> Self {
            match error {
                OurError::IoError(source) => AnotherError::IoError(source),
            }
        }
    }
    impl From<std::io::Error> for AnotherError {
        fn from(error: std::io::Error) -> Self {
            AnotherError::IoError(error)
        }
    }

    fn raw_error_result() -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "this is a raw io error",
        ))
    }

    fn traced_error_result() -> eros::Result<(), (std::io::Error,)> {
        raw_error_result().union().context("Here is some context")
    }

    fn traced_our_error_enum_result1() -> eros::Result<(), (OurError,)> {
        let _ = traced_error_result().into_union().context("More context")?; // `.traced()` does not work here
        let _ = raw_error_result()
            .into_union()
            .context("Different context")?; // `.traced()` does not work here
        Ok(())
    }

    fn traced_our_error_enum_result2() -> eros::Result<(), (AnotherError,)> {
        let _ = traced_our_error_enum_result1().into_union()?; // `.traced()` does not work here
        Ok(())
    }

    #[test]
    fn into_traced() {
        let error = traced_our_error_enum_result2().unwrap_err();
        assert!(matches!(error.inner(), AnotherError::IoError(_)));
    }
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
        return error.into();
    }

    fn result_union() -> Result<(), TracedUnion<(std::io::Error,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        return Err(error).union();
    }

    fn mapped_result_union() -> Result<(), TracedUnion<(MyCustomError,)>> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        return Err(error).into_union();
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
