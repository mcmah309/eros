use eros::ErrorUnion;
use std::{fmt, io};

macro_rules! check_conversion {
    (shared, $union:ident, $message:ident) => {
        let error: AppErrorRef<'_> = (&$union).into();
        assert_eq!(error.to_string(), $message);
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            $message
        );
        match error {
            AppErrorRef::IoError(error) => assert_eq!(error.to_string(), $message),
            AppErrorRef::FmtError(error) => assert_eq!(error.to_string(), $message),
        }
    };
    (mutable, $union:ident, $message:ident) => {
        let error: AppErrorMut<'_> = (&mut $union).into();
        assert_eq!(error.to_string(), $message);
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            $message
        );
        match error {
            AppErrorMut::IoError(error) => {
                *error = io::Error::other("updated");
                $message = String::from("updated");
            }
            AppErrorMut::FmtError(error) => *error = fmt::Error,
        }
        assert_eq!($union.inner().to_string(), $message);
    };
    (owned, $union:ident, $message:ident) => {
        let error: AppError = $union.into();
        assert_eq!(error.to_string(), $message);
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            $message
        );
        match error {
            AppError::IoError(error) => assert_eq!(error.to_string(), $message),
            AppError::FmtError(error) => assert_eq!(error.to_string(), $message),
        }
    };
}

macro_rules! enum_case {
    ($name:ident, [$($attribute:path),+], [$($kind:ident),+], [$($absent:ident),*]) => {
        mod $name {
            use super::*;

            $(#[$attribute])*
            type App = (io::Error, fmt::Error);

            // These names must remain available when their attributes are absent.
            $(#[allow(dead_code)] struct $absent;)*

            #[test]
            fn converts_only_requested_enums() {
                let errors: [ErrorUnion<App>; 2] = [
                    ErrorUnion::new(io::Error::other("io error")),
                    ErrorUnion::new(fmt::Error),
                ];
                #[allow(unused_mut)]
                for mut union in errors {
                    #[allow(unused_mut)]
                    let mut message = union.inner().to_string();
                    $(check_conversion!($kind, union, message);)*
                }
            }
        }
    };
}

enum_case!(
    owned_only,
    [eros::error_enum],
    [owned],
    [AppErrorRef, AppErrorMut]
);
enum_case!(
    shared_only,
    [eros::error_enum_ref],
    [shared],
    [AppError, AppErrorMut]
);
enum_case!(
    mutable_only,
    [eros::error_enum_mut],
    [mutable],
    [AppError, AppErrorRef]
);

enum_case!(
    owned_shared,
    [eros::error_enum, eros::error_enum_ref],
    [shared, owned],
    [AppErrorMut]
);
enum_case!(
    shared_owned,
    [eros::error_enum_ref, eros::error_enum],
    [shared, owned],
    [AppErrorMut]
);
enum_case!(
    owned_mutable,
    [eros::error_enum, eros::error_enum_mut],
    [mutable, owned],
    [AppErrorRef]
);
enum_case!(
    mutable_owned,
    [eros::error_enum_mut, eros::error_enum],
    [mutable, owned],
    [AppErrorRef]
);
enum_case!(
    shared_mutable,
    [eros::error_enum_ref, eros::error_enum_mut],
    [shared, mutable],
    [AppError]
);
enum_case!(
    mutable_shared,
    [eros::error_enum_mut, eros::error_enum_ref],
    [shared, mutable],
    [AppError]
);

enum_case!(
    owned_shared_mutable,
    [eros::error_enum, eros::error_enum_ref, eros::error_enum_mut],
    [shared, mutable, owned],
    []
);
enum_case!(
    owned_mutable_shared,
    [eros::error_enum, eros::error_enum_mut, eros::error_enum_ref],
    [shared, mutable, owned],
    []
);
enum_case!(
    shared_owned_mutable,
    [eros::error_enum_ref, eros::error_enum, eros::error_enum_mut],
    [shared, mutable, owned],
    []
);
enum_case!(
    shared_mutable_owned,
    [eros::error_enum_ref, eros::error_enum_mut, eros::error_enum],
    [shared, mutable, owned],
    []
);
enum_case!(
    mutable_owned_shared,
    [eros::error_enum_mut, eros::error_enum, eros::error_enum_ref],
    [shared, mutable, owned],
    []
);
enum_case!(
    mutable_shared_owned,
    [eros::error_enum_mut, eros::error_enum_ref, eros::error_enum],
    [shared, mutable, owned],
    []
);
