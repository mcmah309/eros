use eros::{traced, IntoConcreteTracedError};



fn main() {
    let error: eros::TracedError = traced!("Error");
    let _error = error.traced();
}
