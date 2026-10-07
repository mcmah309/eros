#[eros::error_enum]
type Bare = (std::fmt::Error,);

#[eros::error_enum()]
type Empty = (std::fmt::Error,);

fn main() {
    let union: eros::ErrorUnion<Bare> = eros::ErrorUnion::new(std::fmt::Error);
    let error = BareError::from(union);
    assert_eq!(error.to_string(), std::fmt::Error.to_string());
    let union: eros::ErrorUnion<Empty> = eros::ErrorUnion::new(std::fmt::Error);
    let error = EmptyError::from(union);
    assert_eq!(error.to_string(), std::fmt::Error.to_string());
}
