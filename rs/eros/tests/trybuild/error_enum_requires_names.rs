#[eros::error_enum]
type MissingOwned = (std::fmt::Error,);

#[eros::error_enum_ref]
type MissingShared = (std::fmt::Error,);

#[eros::error_enum_mut]
type MissingMutable = (std::fmt::Error,);

#[eros::error_enums]
type MissingAll = (std::fmt::Error,);

#[eros::error_enum()]
type EmptyOwned = (std::fmt::Error,);

#[eros::error_enum_ref()]
type EmptyShared = (std::fmt::Error,);

#[eros::error_enum_mut()]
type EmptyMutable = (std::fmt::Error,);

#[eros::error_enums()]
type EmptyAll = (std::fmt::Error,);

fn main() {}
