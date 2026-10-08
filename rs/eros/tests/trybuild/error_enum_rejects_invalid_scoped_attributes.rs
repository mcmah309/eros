#[eros::error_enum(BareError)]
#[eros::error_enum_ref(BareRef, derive(Clone))]
type Bare = (std::fmt::Error,);

#[eros::error_enum(AssignedError)]
#[eros::error_enum_mut = "non_exhaustive"]
type Assigned = (std::fmt::Error,);

#[eros::error_enum(UnexpectedError)]
#[eros::error_enum_mut(UnexpectedMut, non_exhaustive)]
type UnexpectedArgument = (std::fmt::Error,);

#[eros::error_enum_ref(FirstRef)]
#[eros::error_enum_ref(SecondRef)]
type DuplicateRef = (std::fmt::Error,);

#[eros::error_enum_mut(FirstMut)]
#[eros::error_enum_mut(SecondMut)]
type DuplicateMut = (std::fmt::Error,);

#[eros::error_enum(FirstOwned)]
#[eros::error_enum(SecondOwned)]
type DuplicateOwned = (std::fmt::Error,);

fn main() {}
