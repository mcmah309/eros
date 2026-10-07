#[eros::error_enums(AllFirst)]
#[eros::error_enum(OwnedAfter)]
type AllThenOwned = (std::fmt::Error,);

#[eros::error_enum(OwnedFirst)]
#[eros::error_enums(AllAfter)]
type OwnedThenAll = (std::fmt::Error,);

#[eros::error_enums(AllShared)]
#[eros::error_enum_ref(SharedAfter)]
type AllThenShared = (std::fmt::Error,);

#[eros::error_enum_mut(MutableFirst)]
#[eros::error_enums(AllMutable)]
type MutableThenAll = (std::fmt::Error,);

#[eros::error_enums(First)]
#[eros::error_enums(Second)]
type DuplicateAll = (std::fmt::Error,);

#[eros::error_enum(SameName)]
#[eros::error_enum_ref(SameName)]
type ConflictingNames = (std::fmt::Error,);

#[eros::error_enums(Failure)]
type FailureRef = (std::fmt::Error,);

fn main() {}
