use eros::{traced, Traced, TracedError};



fn main() {
    let error: eros::TracedError = traced!("Error");
    let result: Result<(), TracedError> = Err(error);
    let _error = result.traced();
}
