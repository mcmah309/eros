use eros::{ErrorUnion, OtherError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error, OtherError)> =
        Err(ErrorUnion::new(std::fmt::Error));
    result.recover::<std::fmt::Error, _>(|_| ()).into_value();
}
