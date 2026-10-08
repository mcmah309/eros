use eros::ErrorUnion;

#[eros::error_enum_kind(AppKind)]
type App = (std::fmt::Error,);

fn main() {
    let mut union: ErrorUnion = ErrorUnion::new(std::fmt::Error);
    let _: AppKind = (&union).into();
    let _: AppKind = (&mut union).into();
    let _: AppKind = union.into();
}
