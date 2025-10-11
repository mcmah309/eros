use eros::{traced, Traced};



fn main() {
    let error: eros::TracedError = traced!("Error");
    let _error = error.traced();
}
