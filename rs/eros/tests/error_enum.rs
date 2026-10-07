use eros::{ErrorUnion, MsgError};
use std::{
    fmt, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[eros::error_enum("operation failed: {0}")]
pub type Name = (std::io::Error, fmt::Error);

#[eros::error_enum(PublicError)]
#[non_exhaustive]
pub type Internal = (io::Error, fmt::Error);

#[eros::error_enum(FormattedError, "operation failed: {0}")]
type InternalFormatted = (io::Error, fmt::Error);

#[test]
fn custom_names_support_all_conversions_and_optional_display() {
    let errors: [ErrorUnion<Internal>; 2] = [
        ErrorUnion::new(io::Error::other("disk failed")),
        ErrorUnion::new(fmt::Error),
    ];
    for mut union in errors {
        let message = union.inner().to_string();
        let shared: PublicError<&io::Error, &fmt::Error> = (&union).into();
        assert_eq!(shared.to_string(), message);
        let shared: FormattedError<&io::Error, &fmt::Error> = (&union).into();
        assert_eq!(shared.to_string(), format!("operation failed: {message}"));
        let mutable: PublicError<&mut io::Error, &mut fmt::Error> = (&mut union).into();
        assert_eq!(mutable.to_string(), message);
        let mutable: FormattedError<&mut io::Error, &mut fmt::Error> = (&mut union).into();
        assert_eq!(mutable.to_string(), format!("operation failed: {message}"));
        let owned: PublicError = union.into();
        assert_eq!(owned.to_string(), message);
        assert_eq!(
            std::error::Error::source(&owned).unwrap().to_string(),
            message
        );
        assert!(!format!("{owned:?}").is_empty());
    }

    let errors: [ErrorUnion<InternalFormatted>; 2] = [
        ErrorUnion::new(io::Error::other("disk failed")),
        ErrorUnion::new(fmt::Error),
    ];
    for union in errors {
        let message = union.inner().to_string();
        let owned: FormattedError = union.into();
        assert_eq!(owned.to_string(), format!("operation failed: {message}"));
        assert_eq!(
            std::error::Error::source(&owned).unwrap().to_string(),
            message
        );
    }
}

#[derive(Debug)]
struct FormatAwareError;

impl fmt::Display for FormatAwareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(if f.alternate() { "alternate" } else { "normal" })
    }
}

impl std::error::Error for FormatAwareError {}

#[eros::error_enum]
type Delegated = (FormatAwareError, fmt::Error);

#[eros::error_enum()]
type EmptyArgs = (FormatAwareError, fmt::Error);

#[test]
fn default_display_delegates_for_all_variants_and_preserves_formatting_flags() {
    fn assert_display(error: &impl fmt::Display, inner: &dyn fmt::Display) {
        assert_eq!(error.to_string(), inner.to_string());
        assert_eq!(format!("{error:#}"), format!("{inner:#}"));
        assert_eq!(format!("{error:*>12.4}"), format!("{inner:*>12.4}"));
    }

    let errors: [ErrorUnion<Delegated>; 2] = [
        ErrorUnion::new(FormatAwareError),
        ErrorUnion::new(fmt::Error),
    ];
    for mut union in errors {
        let borrowed: DelegatedError<&FormatAwareError, &fmt::Error> = (&union).into();
        assert_display(&borrowed, union.inner());
        let expected = union.inner().to_string();
        let mutable: DelegatedError<&mut FormatAwareError, &mut fmt::Error> = (&mut union).into();
        assert_eq!(mutable.to_string(), expected);
        let owned: DelegatedError = union.into();
        assert_display(&owned, std::error::Error::source(&owned).unwrap());
    }

    let errors: [ErrorUnion<EmptyArgs>; 2] = [
        ErrorUnion::new(FormatAwareError),
        ErrorUnion::new(fmt::Error),
    ];
    for union in errors {
        let owned: EmptyArgsError = union.into();
        assert_display(&owned, std::error::Error::source(&owned).unwrap());
    }
}

#[test]
fn converts_every_named_variant_owned_shared_and_mutable() {
    let errors: [ErrorUnion<Name>; 2] = [
        ErrorUnion::new(io::Error::new(io::ErrorKind::PermissionDenied, "original")),
        ErrorUnion::new(fmt::Error),
    ];
    for (index, mut union) in errors.into_iter().enumerate() {
        let original = union.inner() as *const dyn eros::SendSyncError as *const ();
        let borrowed: NameError<&io::Error, &fmt::Error> = (&union).into();
        match (index, borrowed) {
            (0, NameError::StdIoError(error)) => {
                assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
                assert_eq!(error as *const io::Error as *const (), original);
            }
            (1, NameError::FmtError(error)) => assert_eq!(error, &fmt::Error),
            _ => panic!("wrong borrowed variant"),
        }
        let mutable: NameError<&mut io::Error, &mut fmt::Error> = (&mut union).into();
        match (index, mutable) {
            (0, NameError::StdIoError(error)) => *error = io::Error::other("updated"),
            (1, NameError::FmtError(error)) => *error = fmt::Error,
            _ => panic!("wrong mutable variant"),
        }
        let owned: NameError = union.into();
        assert!(!format!("{owned:?}").is_empty());
        match (index, owned) {
            (0, NameError::StdIoError(error)) => assert_eq!(error.to_string(), "updated"),
            (1, NameError::FmtError(fmt::Error)) => {}
            _ => panic!("wrong owned variant"),
        }
    }
}

#[test]
fn automatic_error_traits_format_and_expose_each_contained_source() {
    fn require_error<E: std::error::Error>() {}
    require_error::<NameError>();
    let errors: [ErrorUnion<Name>; 2] = [
        ErrorUnion::new(io::Error::other("disk failed")),
        ErrorUnion::new(fmt::Error),
    ];
    for union in errors {
        let inner_message = union.inner().to_string();
        let borrowed: NameError<&io::Error, &fmt::Error> = (&union).into();
        assert_eq!(
            borrowed.to_string(),
            format!("operation failed: {inner_message}")
        );
        let owned: NameError = union.into();
        assert_eq!(
            owned.to_string(),
            format!("operation failed: {inner_message}")
        );
        let source = std::error::Error::source(&owned).unwrap();
        assert_eq!(source.to_string(), inner_message);
        match &owned {
            NameError::StdIoError(error) => assert!(std::ptr::eq(
                source.downcast_ref::<io::Error>().unwrap(),
                error,
            )),
            NameError::FmtError(error) => assert!(std::ptr::eq(
                source.downcast_ref::<fmt::Error>().unwrap(),
                error,
            )),
        }
    }
}

#[eros::error_enum("{0}")]
type Single = (MsgError,);

#[test]
fn singleton_preserves_owned_storage_and_tuple_alias() {
    let message = String::from("owned payload");
    let original = message.as_ptr();
    let result: eros::Result<(), Single> = Err(ErrorUnion::new(MsgError::from(message)));
    let SingleError::MsgError(message) = SingleError::from(result.unwrap_err());
    assert_eq!(message.as_str().as_ptr(), original);
}

type E0 = MsgError;

#[eros::error_enum("{0}")]
type Shadowed = (E0, fmt::Error);

#[test]
fn payload_parameters_do_not_shadow_error_types() {
    let union: ErrorUnion<Shadowed> = ErrorUnion::new(MsgError::from("original type"));
    let enum_error: ShadowedError = union.into();
    assert!(matches!(enum_error, ShadowedError::E0(_)));
}

#[derive(Debug)]
struct DropError(Arc<AtomicUsize>);

impl fmt::Display for DropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("drop error")
    }
}

impl std::error::Error for DropError {}

impl Drop for DropError {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[eros::error_enum("{0}")]
type Tracked = (DropError, MsgError);

#[test]
fn conversion_moves_payload_and_drops_metadata_once() {
    let payload_drops = Arc::new(AtomicUsize::new(0));
    let context_drops = Arc::new(AtomicUsize::new(0));
    let union: ErrorUnion<Tracked> = ErrorUnion::new(DropError(payload_drops.clone()));
    let context: Box<dyn eros::SendSyncError> = Box::new(DropError(context_drops.clone()));
    let mut union = union.context(context);
    let _: TrackedError<&DropError, &MsgError> = (&union).into();
    let _: TrackedError<&mut DropError, &mut MsgError> = (&mut union).into();
    assert_eq!(payload_drops.load(Ordering::SeqCst), 0);
    let owned: TrackedError = union.into();
    assert_eq!(payload_drops.load(Ordering::SeqCst), 0);
    assert_eq!(context_drops.load(Ordering::SeqCst), 1);
    drop(owned);
    assert_eq!(payload_drops.load(Ordering::SeqCst), 1);
}

mod snake_case {
    pub use std::fmt::Error as r#type;
}

#[eros::error_enum("fixed {{message}}")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
type Raw = (snake_case::r#type,);

#[test]
fn names_handle_snake_case_and_raw_identifiers() {
    let union: ErrorUnion<Raw> = ErrorUnion::new(fmt::Error);
    let error = RawError::from(union);
    assert!(matches!(error, RawError::SnakeCaseType(_)));
    assert_eq!(error.to_string(), "fixed {message}");
    assert_eq!(error, error.clone());
}

#[eros::error_enum("{0}")]
#[cfg_attr(all(), cfg(any()), derive(Clone))]
type DisabledAlias = (fmt::Error,);

mod exported {
    #[eros::error_enum("{0}")]
    pub type Public = (core::fmt::Error,);
}

#[test]
fn generated_enum_preserves_alias_visibility() {
    let union: ErrorUnion<exported::Public> = ErrorUnion::new(fmt::Error);
    assert!(matches!(
        exported::PublicError::from(union),
        exported::PublicError::CoreFmtError(_)
    ));
}
