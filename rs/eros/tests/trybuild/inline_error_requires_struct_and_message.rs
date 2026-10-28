#[eros::inline_error("union")]
union Union { field: u8 }

#[eros::inline_error(42)]
struct Number;

#[eros::inline_error("message", "extra")]
struct Extra;

fn main() {}
