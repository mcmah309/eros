use eros::{IntoUnion};



fn main() {
    let error: eros::ErrorUnion = eros::traced!("Error");
    let result: Result<(), eros::ErrorUnion> = Err(error);
    let _error = result.into_union();
}
