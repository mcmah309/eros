use eros::{IntoUnion};



fn main() {
    let error: eros::ErrorUnion = eros::traced!("Error");
    let _error = error.into_union();
}
