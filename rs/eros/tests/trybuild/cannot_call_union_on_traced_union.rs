use eros::{Union};



fn main() {
    let error: eros::ErrorUnion = eros::traced!("Error");
    let _error = error.union();
}
