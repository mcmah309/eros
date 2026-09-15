fn main() {
    const MESSAGE: &str = "message";
    let message = MESSAGE;
    let _ = eros::error!(MESSAGE);
    let _ = eros::error!(message);
}
