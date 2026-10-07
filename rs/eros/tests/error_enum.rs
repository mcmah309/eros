use eros::{ErrorUnion, MsgError};
use std::{
    fmt, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[eros::error_enums(NameError, "operation failed: {0}")]
pub type Name = (std::io::Error, fmt::Error);

#[eros::error_enum(PublicError)]
#[non_exhaustive]
#[eros::error_enum_ref(PublicErrorRef)]
#[eros::error_enum_mut(PublicErrorMut)]
pub type Internal = (io::Error, fmt::Error);

#[eros::error_enums(FormattedError, "operation failed: {0}")]
type InternalFormatted = (io::Error, fmt::Error);

#[test]
fn custom_names_support_all_conversions_and_optional_display() {
    let errors: [ErrorUnion<Internal>; 2] = [
        ErrorUnion::new(io::Error::other("disk failed")),
        ErrorUnion::new(fmt::Error),
    ];
    for mut union in errors {
        let message = union.inner().to_string();
        let shared: PublicErrorRef<'_> = (&union).into();
        assert_eq!(shared.to_string(), message);
        let shared: FormattedErrorRef<'_> = (&union).into();
        assert_eq!(shared.to_string(), format!("operation failed: {message}"));
        let mutable: PublicErrorMut<'_> = (&mut union).into();
        assert_eq!(mutable.to_string(), message);
        let mutable: FormattedErrorMut<'_> = (&mut union).into();
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

#[eros::error_enums(DelegatedError)]
type Delegated = (FormatAwareError, fmt::Error);

#[eros::error_enum(TrailingCommaError)]
type TrailingComma = (FormatAwareError, fmt::Error);

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
        let borrowed: DelegatedErrorRef<'_> = (&union).into();
        assert_display(&borrowed, union.inner());
        let expected = union.inner().to_string();
        let mutable: DelegatedErrorMut<'_> = (&mut union).into();
        assert_eq!(mutable.to_string(), expected);
        let owned: DelegatedError = union.into();
        assert_display(&owned, std::error::Error::source(&owned).unwrap());
    }

    let errors: [ErrorUnion<TrailingComma>; 2] = [
        ErrorUnion::new(FormatAwareError),
        ErrorUnion::new(fmt::Error),
    ];
    for union in errors {
        let owned: TrailingCommaError = union.into();
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
        let borrowed: NameErrorRef<'_> = (&union).into();
        match (index, borrowed) {
            (0, NameErrorRef::StdIoError(error)) => {
                assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
                assert_eq!(error as *const io::Error as *const (), original);
            }
            (1, NameErrorRef::FmtError(error)) => assert_eq!(error, &fmt::Error),
            _ => panic!("wrong borrowed variant"),
        }
        let mutable: NameErrorMut<'_> = (&mut union).into();
        match (index, mutable) {
            (0, NameErrorMut::StdIoError(error)) => *error = io::Error::other("updated"),
            (1, NameErrorMut::FmtError(error)) => *error = fmt::Error,
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
    require_error::<NameErrorRef<'_>>();
    require_error::<NameErrorMut<'_>>();
    let errors: [ErrorUnion<Name>; 2] = [
        ErrorUnion::new(io::Error::other("disk failed")),
        ErrorUnion::new(fmt::Error),
    ];
    for mut union in errors {
        let inner_message = union.inner().to_string();
        let original = union.inner() as *const dyn eros::SendSyncError as *const ();
        fn assert_source(error: &dyn std::error::Error, original: *const ()) {
            let source = error.source().unwrap();
            assert_eq!(
                source as *const dyn std::error::Error as *const (),
                original
            );
            assert!(source.is::<io::Error>() || source.is::<fmt::Error>());
        }
        let borrowed = NameErrorRef::from(&union);
        assert_eq!(
            borrowed.to_string(),
            format!("operation failed: {inner_message}")
        );
        assert_source(&borrowed, original);
        assert!(!format!("{borrowed:?}").is_empty());
        let mutable = NameErrorMut::from(&mut union);
        assert_eq!(
            mutable.to_string(),
            format!("operation failed: {inner_message}")
        );
        assert_source(&mutable, original);
        assert!(!format!("{mutable:?}").is_empty());
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

#[eros::error_enum(SingleError, "{0}")]
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

#[eros::error_enum(ShadowedError, "{0}")]
type Shadowed = (E0, fmt::Error);

#[test]
fn preserves_error_types_named_like_former_payload_parameters() {
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

#[eros::error_enums(TrackedError, "{0}")]
type Tracked = (DropError, MsgError);

#[test]
fn conversion_moves_payload_and_drops_metadata_once() {
    let payload_drops = Arc::new(AtomicUsize::new(0));
    let context_drops = Arc::new(AtomicUsize::new(0));
    let union: ErrorUnion<Tracked> = ErrorUnion::new(DropError(payload_drops.clone()));
    let context: Box<dyn eros::SendSyncError> = Box::new(DropError(context_drops.clone()));
    let mut union = union.context(context);
    let _: TrackedErrorRef<'_> = (&union).into();
    let _: TrackedErrorMut<'_> = (&mut union).into();
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

#[eros::error_enum(RawError, "fixed {{message}}")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[eros::error_enum_ref(RawErrorRef, "fixed {{message}}")]
#[eros::error_enum_mut(RawErrorMut, "fixed {{message}}")]
type Raw = (snake_case::r#type,);

#[test]
fn names_handle_snake_case_and_raw_identifiers() {
    let mut union: ErrorUnion<Raw> = ErrorUnion::new(fmt::Error);
    // Clone/Copy on the owned enum must not be copied to mutable references.
    let RawErrorRef::SnakeCaseType(error) = RawErrorRef::from(&union);
    assert_eq!(error, &fmt::Error);
    let RawErrorMut::SnakeCaseType(error) = RawErrorMut::from(&mut union);
    assert_eq!(error, &mut fmt::Error);
    let error = RawError::from(union);
    assert!(matches!(error, RawError::SnakeCaseType(_)));
    assert_eq!(error.to_string(), "fixed {message}");
    assert_eq!(error, error.clone());
}

#[eros::error_enum(DisabledAliasError, "{0}")]
#[cfg_attr(all(), cfg(any()), derive(Clone))]
type DisabledAlias = (fmt::Error,);

#[eros::error_enum(ScopedError)]
#[eros::error_enum_ref(ScopedErrorRef)]
#[derive(Clone, Copy)]
#[non_exhaustive]
#[doc = "Shared error view"]
#[eros::error_enum_mut(ScopedErrorMut)]
#[non_exhaustive]
type Scoped = (io::Error, fmt::Error);

#[eros::error_enum(ComparableError)]
#[eros::error_enum_mut(ComparableErrorMut)]
#[derive(PartialEq, Eq)]
#[doc = "Mutable error view"]
#[eros::error_enum_ref(ComparableErrorRef)]
#[cfg_attr(all(), derive(Clone, Copy))]
type Comparable = (fmt::Error,);

#[test]
fn annotations_apply_to_the_selected_borrowed_enum() {
    let mut union_of: ErrorUnion<Scoped> = ErrorUnion::new(io::Error::other("borrowed"));
    let shared = ScopedErrorRef::from(&union_of);
    let shared_copy = shared;
    let shared_clone = shared.clone();
    assert_eq!(shared.to_string(), shared_copy.to_string());
    assert_eq!(shared.to_string(), shared_clone.to_string());
    let mutable = ScopedErrorMut::from(&mut union_of);
    assert_eq!(mutable.to_string(), "borrowed");
    let owned = ScopedError::from(union_of);
    assert_eq!(owned.to_string(), "borrowed");

    let mut first: ErrorUnion<Comparable> = ErrorUnion::new(fmt::Error);
    let mut second: ErrorUnion<Comparable> = ErrorUnion::new(fmt::Error);
    assert_eq!(
        ComparableErrorMut::from(&mut first),
        ComparableErrorMut::from(&mut second)
    );
}

#[eros::error_enum_ref(SharedFirstErrorRef, "shared-first: {0}")]
#[derive(Clone, Copy)]
#[eros::error_enum(SharedFirstError, "shared-first: {0}")]
#[eros::error_enum_mut(SharedFirstErrorMut, "shared-first: {0}")]
#[derive(PartialEq, Eq)]
type SharedFirst = (fmt::Error,);

#[eros::error_enum_mut(MutableFirstErrorMut, "mutable-first: {0}")]
#[derive(PartialEq, Eq)]
#[eros::error_enum_ref(MutableFirstErrorRef, "mutable-first: {0}")]
#[derive(Clone, Copy)]
#[eros::error_enum(MutableFirstError, "mutable-first: {0}")]
type MutableFirst = (fmt::Error,);

#[test]
fn borrowed_markers_can_appear_before_the_owned_marker() {
    let mut first: ErrorUnion<SharedFirst> = ErrorUnion::new(fmt::Error);
    let mut second: ErrorUnion<SharedFirst> = ErrorUnion::new(fmt::Error);
    let shared = SharedFirstErrorRef::from(&first);
    let shared_copy = shared;
    assert_eq!(shared.to_string(), shared_copy.to_string());
    assert_eq!(shared.to_string(), format!("shared-first: {}", fmt::Error));
    let expected = shared_copy.to_string();
    assert_eq!(
        SharedFirstErrorMut::from(&mut first),
        SharedFirstErrorMut::from(&mut second)
    );
    assert_eq!(SharedFirstError::from(first).to_string(), expected);

    let mut first: ErrorUnion<MutableFirst> = ErrorUnion::new(fmt::Error);
    let mut second: ErrorUnion<MutableFirst> = ErrorUnion::new(fmt::Error);
    let shared = MutableFirstErrorRef::from(&first);
    let shared_copy = shared;
    assert_eq!(shared.to_string(), shared_copy.to_string());
    assert_eq!(shared.to_string(), format!("mutable-first: {}", fmt::Error));
    let expected = shared_copy.to_string();
    assert_eq!(
        MutableFirstErrorMut::from(&mut first),
        MutableFirstErrorMut::from(&mut second)
    );
    assert_eq!(MutableFirstError::from(first).to_string(), expected);
}

mod exported {
    #[eros::error_enums(PublicError, "{0}")]
    pub type Public = (core::fmt::Error,);
}

#[test]
fn generated_enum_preserves_alias_visibility() {
    let mut union: ErrorUnion<exported::Public> = ErrorUnion::new(fmt::Error);
    assert!(matches!(
        exported::PublicErrorRef::from(&union),
        exported::PublicErrorRef::CoreFmtError(_)
    ));
    assert!(matches!(
        exported::PublicErrorMut::from(&mut union),
        exported::PublicErrorMut::CoreFmtError(_)
    ));
    assert!(matches!(
        exported::PublicError::from(union),
        exported::PublicError::CoreFmtError(_)
    ));
}
