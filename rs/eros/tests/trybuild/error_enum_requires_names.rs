#[eros::error_enum]
type MissingOwned = (std::fmt::Error,);

#[eros::error_enum_ref]
type MissingShared = (std::fmt::Error,);

#[eros::error_enum_mut]
type MissingMutable = (std::fmt::Error,);

#[eros::error_enum_kind]
type MissingKind = (std::fmt::Error,);

#[eros::error_enum()]
type EmptyOwned = (std::fmt::Error,);

#[eros::error_enum_ref()]
type EmptyShared = (std::fmt::Error,);

#[eros::error_enum_mut()]
type EmptyMutable = (std::fmt::Error,);

#[eros::error_enum_kind()]
type EmptyKind = (std::fmt::Error,);

fn main() {}
