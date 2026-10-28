static MESSAGE: &str = "message";
fn main() { let _ = eros::error!(eros::MsgError::from_static_ref(&MESSAGE)); }
