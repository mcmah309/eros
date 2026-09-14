use eros::{ErrorUnion, IntoUnion, ReshapeUnion};

fn main() {
    let _: ErrorUnion<(u16, std::io::Error)> = ErrorUnion::new(std::io::Error::other("error"));

    let error: ErrorUnion<(std::io::Error,)> = ErrorUnion::new(std::io::Error::other("error"));
    let _ = error.widen::<(std::io::Error, u16), _>();

    let result: Result<(), std::io::Error> = Ok(());
    let _ = result.union::<_, (std::io::Error, u16)>();

    let result: Result<(), ErrorUnion<(std::io::Error,)>> = Ok(());
    let _ = result.widen::<(std::io::Error, u16), _>();
}
