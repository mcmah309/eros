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
            AppErrorRef::Io(error) => assert_eq!(error.to_string(), $message),
            AppErrorRef::Fmt(error) => assert_eq!(error.to_string(), $message),
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
            AppErrorMut::Io(error) => {
                *error = io::Error::other("updated");
                $message = String::from("updated");
            }
            AppErrorMut::Fmt(error) => *error = fmt::Error,
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
            AppError::Io(error) => assert_eq!(error.to_string(), $message),
            AppError::Fmt(error) => assert_eq!(error.to_string(), $message),
        }
    };
}

macro_rules! enum_case {
    ($name:ident, [$($attribute:meta),+], [$($kind:ident),+], [$($absent:ident),*]) => {
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

            #[test]
            fn converts_singleton_subsets_for_requested_enums() {
                #[allow(unused_mut)]
                let mut union: ErrorUnion<(io::Error,)> = ErrorUnion::new(io::Error::other("io error"));
                #[allow(unused_mut)]
                let mut message = union.inner().to_string();
                $(check_conversion!($kind, union, message);)*

                #[allow(unused_mut)]
                let mut union: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
                #[allow(unused_mut)]
                let mut message = union.inner().to_string();
                $(check_conversion!($kind, union, message);)*
            }
        }
    };
}

enum_case!(
    owned_only,
    [eros::error_enum(AppError)],
    [owned],
    [AppErrorRef, AppErrorMut]
);
enum_case!(
    shared_only,
    [eros::error_enum_ref(AppErrorRef)],
    [shared],
    [AppError, AppErrorMut]
);
enum_case!(
    mutable_only,
    [eros::error_enum_mut(AppErrorMut)],
    [mutable],
    [AppError, AppErrorRef]
);

enum_case!(
    owned_shared,
    [
        eros::error_enum(AppError),
        eros::error_enum_ref(AppErrorRef)
    ],
    [shared, owned],
    [AppErrorMut]
);
enum_case!(
    shared_owned,
    [
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum(AppError)
    ],
    [shared, owned],
    [AppErrorMut]
);
enum_case!(
    owned_mutable,
    [
        eros::error_enum(AppError),
        eros::error_enum_mut(AppErrorMut)
    ],
    [mutable, owned],
    [AppErrorRef]
);
enum_case!(
    mutable_owned,
    [
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum(AppError)
    ],
    [mutable, owned],
    [AppErrorRef]
);
enum_case!(
    shared_mutable,
    [
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum_mut(AppErrorMut)
    ],
    [shared, mutable],
    [AppError]
);
enum_case!(
    mutable_shared,
    [
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum_ref(AppErrorRef)
    ],
    [shared, mutable],
    [AppError]
);

enum_case!(
    owned_shared_mutable,
    [
        eros::error_enum(AppError),
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum_mut(AppErrorMut)
    ],
    [shared, mutable, owned],
    []
);
enum_case!(
    owned_mutable_shared,
    [
        eros::error_enum(AppError),
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum_ref(AppErrorRef)
    ],
    [shared, mutable, owned],
    []
);
enum_case!(
    shared_owned_mutable,
    [
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum(AppError),
        eros::error_enum_mut(AppErrorMut)
    ],
    [shared, mutable, owned],
    []
);
enum_case!(
    shared_mutable_owned,
    [
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum(AppError)
    ],
    [shared, mutable, owned],
    []
);
enum_case!(
    mutable_owned_shared,
    [
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum(AppError),
        eros::error_enum_ref(AppErrorRef)
    ],
    [shared, mutable, owned],
    []
);
enum_case!(
    mutable_shared_owned,
    [
        eros::error_enum_mut(AppErrorMut),
        eros::error_enum_ref(AppErrorRef),
        eros::error_enum(AppError)
    ],
    [shared, mutable, owned],
    []
);

#[eros::error_enum(Failure, "all: {0}")]
#[non_exhaustive]
#[repr(align(64))]
#[cfg_attr(all(), derive(PartialEq, Eq))]
#[doc = "An owned error with explicit attributes."]
#[eros::error_enum_ref(FailureRef, "all: {0}")]
#[non_exhaustive]
#[repr(align(64))]
#[cfg_attr(all(), derive(PartialEq, Eq))]
#[doc = "A shared error with explicit attributes."]
#[eros::error_enum_mut(FailureMut, "all: {0}")]
#[non_exhaustive]
#[repr(align(64))]
#[cfg_attr(all(), derive(PartialEq, Eq))]
#[doc = "A mutable error with explicit attributes."]
type AllErrors = (fmt::Error,);

#[eros::error_enum(DisabledFailure)]
#[eros::error_enum_ref(DisabledFailureRef)]
#[eros::error_enum_mut(DisabledFailureMut)]
#[cfg_attr(all(), cfg(any()))]
type DisabledErrors = (fmt::Error,);

// Disabled declarations must not emit enums or conversions.
struct DisabledFailure;
struct DisabledFailureRef;
struct DisabledFailureMut;
struct DisabledErrors;

#[test]
fn stacked_macros_preserve_explicit_attributes_and_display() {
    assert_eq!(std::mem::align_of::<Failure>(), 64);
    assert_eq!(std::mem::align_of::<FailureRef<'_>>(), 64);
    assert_eq!(std::mem::align_of::<FailureMut<'_>>(), 64);

    let mut first: ErrorUnion<AllErrors> = ErrorUnion::new(fmt::Error);
    let mut second: ErrorUnion<AllErrors> = ErrorUnion::new(fmt::Error);
    let expected = format!("all: {}", fmt::Error);

    let shared: FailureRef<'_> = (&first).into();
    assert_eq!(shared, (&second).into());
    assert_eq!(shared.to_string(), expected);
    assert_eq!(
        std::error::Error::source(&shared).unwrap().to_string(),
        fmt::Error.to_string()
    );

    let mutable: FailureMut<'_> = (&mut first).into();
    assert_eq!(mutable, (&mut second).into());
    assert_eq!(mutable.to_string(), expected);

    let owned: Failure = first.into();
    assert_eq!(owned, second.into());
    assert_eq!(owned.to_string(), expected);

    let _disabled = (
        DisabledFailure,
        DisabledFailureRef,
        DisabledFailureMut,
        DisabledErrors,
    );
}

#[eros::error_enum_mut(Editable, "mutable: {0}")]
#[eros::error_enum_ref(Borrowed, "shared: {0}")]
#[eros::error_enum(Owned, "owned: {0}")]
type IndependentlyNamed = (io::Error, fmt::Error);

#[test]
fn individual_macros_use_exact_names_and_independent_display_formats() {
    let mut union: ErrorUnion<IndependentlyNamed> = ErrorUnion::new(io::Error::other("before"));
    let shared: Borrowed<'_> = (&union).into();
    assert_eq!(shared.to_string(), "shared: before");
    let mutable: Editable<'_> = (&mut union).into();
    assert_eq!(mutable.to_string(), "mutable: before");
    match mutable {
        Editable::Io(error) => *error = io::Error::other("after"),
        Editable::Fmt(_) => panic!("wrong variant"),
    }
    let owned: Owned = union.into();
    assert_eq!(owned.to_string(), "owned: after");
}
