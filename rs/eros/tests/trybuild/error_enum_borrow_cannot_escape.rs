#[eros::error_enum]
#[eros::error_enum_ref]
#[eros::error_enum_mut]
type Local = (std::fmt::Error,);

fn shared() -> LocalErrorRef<'static> {
    let union_of: eros::ErrorUnion<Local> = eros::ErrorUnion::new(std::fmt::Error);
    LocalErrorRef::from(&union_of)
}

fn mutable() -> LocalErrorMut<'static> {
    let mut union_of: eros::ErrorUnion<Local> = eros::ErrorUnion::new(std::fmt::Error);
    LocalErrorMut::from(&mut union_of)
}

fn main() {}
