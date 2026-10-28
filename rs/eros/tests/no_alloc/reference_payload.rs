static ERROR: core::fmt::Error = core::fmt::Error;
fn main() { let _: eros::ErrorUnion<(&'static core::fmt::Error,)> = eros::ErrorUnion::new(&ERROR); }
