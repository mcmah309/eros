#[eros::error_enum(FirstOwned)]
#[eros::error_enum(SecondOwned)]
type DuplicateOwned = (std::fmt::Error,);

#[eros::error_enum_ref(FirstShared)]
#[eros::error_enum_ref(SecondShared)]
type DuplicateShared = (std::fmt::Error,);

#[eros::error_enum_mut(FirstMutable)]
#[eros::error_enum_mut(SecondMutable)]
type DuplicateMutable = (std::fmt::Error,);

#[eros::error_enum_kind(FirstKind)]
#[eros::error_enum_kind(SecondKind)]
type DuplicateKind = (std::fmt::Error,);

#[eros::error_enum(SameName)]
#[eros::error_enum_kind(SameName)]
type ConflictingNames = (std::fmt::Error,);

#[eros::error_enum_kind(SameAlias)]
type SameAlias = (std::fmt::Error,);

fn main() {}
