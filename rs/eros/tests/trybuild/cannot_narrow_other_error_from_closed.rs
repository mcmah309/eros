use eros::{ErrorUnion, OtherError, ReshapeUnion};

fn main() {
    let error: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(std::fmt::Error);
    let _ = error.narrow::<OtherError, _>();

    let result: eros::Result<(), (std::fmt::Error,)> = Err(ErrorUnion::new(std::fmt::Error));
    let _ = result.recover::<OtherError, _>(|_| ());
}
