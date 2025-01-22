
#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err(OneOf::new(GenericError::new($msg)));
    };
}

#[macro_export]
macro_rules! generic {
    ($msg:expr) => {
        OneOf::new(GenericError::new($msg));
    };
}

