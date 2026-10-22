#[eros::error_enum(LocalError)]
#[eros::error_enum_ref(LocalErrorRef)]
#[eros::error_enum_mut(LocalErrorMut)]
type Local = (std::fmt::Error, eros::MsgError);

fn shared() -> LocalErrorRef<'static> {
    let union_of: eros::ErrorUnion<(std::fmt::Error,)> = eros::ErrorUnion::new(std::fmt::Error);
    LocalErrorRef::from(&union_of)
}

fn mutable() -> LocalErrorMut<'static> {
    let mut union_of: eros::ErrorUnion<(std::fmt::Error,)> = eros::ErrorUnion::new(std::fmt::Error);
    LocalErrorMut::from(&mut union_of)
}

fn main() {}
