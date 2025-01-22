
#[macro_export]
macro_rules! bail {
    ($error:expr) => {
        return Err(eros::ErrorUnion::new(std::convert::identity::<eros::GenericError>($error.into())));
    };
}

#[macro_export]
macro_rules! generic {
    ($error:expr) => {
        eros::ErrorUnion::new(std::convert::identity::<eros::GenericError>($error.into()))
    };
}