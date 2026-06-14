use eros::{IntoUnion};



fn main() {
    let error: eros::ErrorUnion = eros::error!("Error");
    let _error = error.into_union();
}
