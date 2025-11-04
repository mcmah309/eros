use eros::{Union};



fn main() {
    let error: eros::TracedUnion = eros::traced!("Error");
    let _error = error.union();
}
