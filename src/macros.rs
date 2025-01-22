
#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err(ErrorUnion::new(GenericError::new($msg)));
    };
}

#[macro_export]
macro_rules! generic {
    ($msg:expr) => {
        ErrorUnion::new(GenericError::new($msg));
    };
}

#[macro_export]
macro_rules! new {
    ($error:expr) => {
        ErrorUnion::new(error);
    };
}

