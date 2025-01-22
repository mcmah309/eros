
#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err(U::new(GenericError::new($msg)));
    };
}

#[macro_export]
macro_rules! generic {
    ($msg:expr) => {
        U::new(GenericError::new($msg));
    };
}

