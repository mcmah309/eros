#[eros::error_enum]
#[eros::error_enum_ref(derive(Clone))]
type Bare = (std::fmt::Error,);

#[eros::error_enum]
#[eros::error_enum_mut = "non_exhaustive"]
type Assigned = (std::fmt::Error,);

#[eros::error_enum]
#[eros::error_enum_mut(non_exhaustive)]
type UnexpectedArgument = (std::fmt::Error,);

#[eros::error_enum_ref]
type MissingOwned = (std::fmt::Error,);

#[eros::error_enum_mut]
type MissingOwnedMut = (std::fmt::Error,);

#[eros::error_enum]
#[eros::error_enum]
type DuplicateOwned = (std::fmt::Error,);

fn main() {}
