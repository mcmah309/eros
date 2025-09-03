use eros::{traced, IntoConcreteTracedError, TracedError};



fn main() {
    let error: eros::TracedError = traced!("Error");
    let result: Result<(), TracedError> = Err(error);
    let _error = result.traced();
}
