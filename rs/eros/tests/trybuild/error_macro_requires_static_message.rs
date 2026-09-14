#[allow(non_snake_case)]
fn main() {
    let message = String::from("borrowed message");
    let ERROR = message.as_str();
    let _ = eros::error!(ERROR);
}
