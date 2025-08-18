use std::fmt::Display;

use eros::ErrorUnion;

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
fn retry() {
    fn does_stuff() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout)>> {
        let _allocation = match allocates() {
            Ok(a) => a,
            Err(e) => return Err(e.inflate()),
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

            match err.deflate::<Timeout, _>() {
                Ok(_timeout) => continue,
                Err(not_enough_memory_union) => {
                    return Err(not_enough_memory_union.inflate());
                }
            }
        }

        Err(ErrorUnion::new(RetriesExhausted))
    }

    let Err(err) = inner() else {
        panic!("Should be error");
    };
    let err = match err.deflate::<RetriesExhausted, _>() {
        Ok(_) => panic!("Should not have RetriesExhausted"),
        Err(err) => err,
    };
    let err = match err.deflate::<NotEnoughMemory, _>() {
        Ok(err) => err,
        Err(_) => panic!("Should have NotEnoughMemory"),
    };
    assert_eq!(err, NotEnoughMemory);
}

#[test]
fn smoke() {
    let o_1: ErrorUnion<(u32, String)> = ErrorUnion::new(5_u32);
    let _narrowed_1: u32 = o_1.deflate::<u32, _>().unwrap();

    let o_2: ErrorUnion<(String, u32)> = ErrorUnion::new(5_u32);
    let _narrowed_2: u32 = o_2.deflate::<u32, _>().unwrap();

    let o_3: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    let _narrowed_3: ErrorUnion<(String,)> = o_3.deflate::<u32, _>().unwrap_err();

    let o_4: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());

    let _: String = o_4.deflate().unwrap();

    let o_5: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    o_5.deflate::<String, _>().unwrap();

    let o_6: ErrorUnion<(String, u32)> = ErrorUnion::new("5".to_string());
    let o_7: ErrorUnion<(u32, String)> = o_6.inflate();
    let o_8: ErrorUnion<(String, u32)> = o_7.subset().unwrap();
    let _: ErrorUnion<(u32, String)> = o_8.subset().unwrap();

    let o_9: ErrorUnion<(u8, u16, u32)> = ErrorUnion::new(3_u32);
    let _: Result<ErrorUnion<(u16,)>, ErrorUnion<(u8, u32)>> = o_9.subset();
    let o_10: ErrorUnion<(u8, u16, u32)> = ErrorUnion::new(3_u32);
    let _: Result<u16, ErrorUnion<(u8, u32)>> = o_10.deflate();
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

    match o_1.as_enum() {
        E2::A(u) => {
            println!("handling {u}: u32")
        }
        E2::B(s) => {
            println!("handling {s}: String")
        }
    }

    match o_1.to_enum() {
        E2::A(u) => {
            println!("handling {u}: u32")
        }
        E2::B(s) => {
            println!("handling {s}: String")
        }
    }
}

#[test]
fn multi_narrow() {
    use eros::E2;

    struct Timeout {}
    struct Backoff {}

    let o_1: ErrorUnion<(u8, u16, u32, u64, u128)> = ErrorUnion::new(5_u32);

    let _narrow_res: Result<ErrorUnion<(u8, u128)>, ErrorUnion<(u16, u32, u64)>> = o_1.subset();

    let o_2: ErrorUnion<(u8, u16, Backoff, Timeout, u32, u64, u128)> = ErrorUnion::new(Timeout {});

    match o_2.subset::<(Timeout, Backoff), _>().unwrap().to_enum() {
        E2::A(Timeout {}) => {
            println!(":)");
        }
        E2::B(Backoff {}) => {
            unreachable!()
        }
    }
}
