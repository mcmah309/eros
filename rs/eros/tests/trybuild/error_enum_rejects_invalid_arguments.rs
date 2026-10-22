#[eros::error_enum(CustomError, 123)]
type NotString = (std::fmt::Error,);

#[eros::error_enum(CustomError "message")]
type MissingComma = (std::fmt::Error,);

#[eros::error_enum("message", CustomError)]
type WrongOrder = (std::fmt::Error,);

#[eros::error_enum(CustomError, "message", "extra")]
type ExtraArgument = (std::fmt::Error,);

#[eros::error_enum(SameName)]
type SameName = (std::fmt::Error,);

fn main() {}
