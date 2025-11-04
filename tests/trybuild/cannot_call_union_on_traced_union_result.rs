use eros::{Union};



fn main() {
    let error: eros::TracedUnion = eros::traced!("Error");
    let result: Result<(), eros::TracedUnion> = Err(error);
    let _error = result.union();
}
