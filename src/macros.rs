
#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err(eros::ErrorUnion::new(eros::GenericError::new($msg)));
    };
}

#[macro_export]
macro_rules! generic {
    ($msg:expr) => {
        eros::ErrorUnion::new(eros::GenericError::new($msg));
    };
}

#[macro_export]
macro_rules! new {
    ($error:expr) => {
        eros::ErrorUnion::new(error);
    };
}

