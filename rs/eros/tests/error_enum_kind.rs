use eros::{AnyError, ErrorUnion, MsgError, TypeSet};
use std::{
    fmt, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[eros::error_enum_kind(AppKind)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
type App = (io::Error, fmt::Error, MsgError);

fn check_typed<S>(mut union: ErrorUnion<S>, expected: AppKind)
where
    S: TypeSet,
    AppKind: From<ErrorUnion<S>>,
    for<'a> AppKind: From<&'a ErrorUnion<S>> + From<&'a mut ErrorUnion<S>>,
{
    union = union.context("retained diagnostics");
    let original = union.inner() as *const dyn eros::SendSyncError as *const ();
    let report = format!("{union:?}");
    assert_eq!(AppKind::from(&union), expected);
    assert_eq!(AppKind::from(&mut union), expected);
    assert_eq!(format!("{union:?}"), report);
    assert_eq!(
        union.inner() as *const dyn eros::SendSyncError as *const (),
        original
    );
    assert_eq!(AppKind::from(union), expected);
}

#[test]
fn typed_conversions_classify_every_branch_and_accept_subsets_and_reordering() {
    check_typed::<App>(
        ErrorUnion::new(io::Error::other("disk failed")),
        AppKind::Io,
    );
    check_typed::<App>(ErrorUnion::new(fmt::Error), AppKind::Fmt);
    check_typed::<App>(ErrorUnion::new(MsgError::from("message")), AppKind::Msg);
    check_typed::<(io::Error,)>(
        ErrorUnion::new(io::Error::other("disk failed")),
        AppKind::Io,
    );
    check_typed::<(fmt::Error,)>(ErrorUnion::new(fmt::Error), AppKind::Fmt);
    check_typed::<(MsgError,)>(ErrorUnion::new(MsgError::from("message")), AppKind::Msg);
    check_typed::<(MsgError, fmt::Error)>(ErrorUnion::new(fmt::Error), AppKind::Fmt);
    check_typed::<(MsgError, fmt::Error, io::Error)>(
        ErrorUnion::new(io::Error::other("disk failed")),
        AppKind::Io,
    );

    fn classify_empty(mut union: ErrorUnion<()>) {
        let _: AppKind = (&union).into();
        let _: AppKind = (&mut union).into();
        let _: AppKind = union.into();
    }
    let _: fn(ErrorUnion<()>) = classify_empty;
}

#[test]
fn erased_conversions_classify_every_branch_without_disturbing_borrowed_errors() {
    let errors: [(ErrorUnion<AnyError>, AppKind); 3] = [
        (
            ErrorUnion::new(io::Error::other("disk failed")),
            AppKind::Io,
        ),
        (ErrorUnion::new(fmt::Error), AppKind::Fmt),
        (ErrorUnion::new(MsgError::from("message")), AppKind::Msg),
    ];
    for (union, expected) in errors {
        let mut union = union.context("retained diagnostics");
        let report = format!("{union:?}");
        assert_eq!(AppKind::try_from(&union).unwrap(), expected);
        assert_eq!(AppKind::try_from(&mut union).unwrap(), expected);
        assert_eq!(format!("{union:?}"), report);
        assert_eq!(AppKind::try_from(union).unwrap(), expected);
    }
}

#[derive(Debug)]
struct UnlistedError(fmt::Error);

impl fmt::Display for UnlistedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unlisted error")
    }
}

impl std::error::Error for UnlistedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn erased_mismatches_return_original_values_and_ignore_sources_and_contexts() {
    let mut union: ErrorUnion = ErrorUnion::new(UnlistedError(fmt::Error));
    union = union.context(Box::new(fmt::Error) as Box<dyn eros::SendSyncError>);
    let original = union.inner() as *const dyn eros::SendSyncError as *const ();
    let report = format!("{union:?}");
    let returned = AppKind::try_from(&union).unwrap_err();
    assert!(std::ptr::eq(returned, &union));
    let original_union = &mut union as *mut ErrorUnion;
    let returned = AppKind::try_from(&mut union).unwrap_err();
    assert_eq!(returned as *mut ErrorUnion, original_union);
    let returned = AppKind::try_from(union).unwrap_err();
    assert_eq!(format!("{returned:?}"), report);
    assert_eq!(
        returned.inner() as *const dyn eros::SendSyncError as *const (),
        original
    );
    assert!(returned.is_inner::<UnlistedError>());
}

#[derive(Debug)]
struct DropError(Arc<AtomicUsize>);

impl fmt::Display for DropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tracked error")
    }
}

impl std::error::Error for DropError {}

impl Drop for DropError {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[eros::error_enum_kind(TrackedKind)]
#[derive(PartialEq, Eq)]
type Tracked = (DropError, fmt::Error);

#[test]
fn owned_classification_drops_payload_and_diagnostics_exactly_once() {
    for erased in [false, true] {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut union: ErrorUnion<Tracked> = ErrorUnion::new(DropError(drops.clone()));
        union = union.context(Box::new(DropError(drops.clone())) as Box<dyn eros::SendSyncError>);
        assert_eq!(TrackedKind::from(&union), TrackedKind::Drop);
        assert_eq!(TrackedKind::from(&mut union), TrackedKind::Drop);
        let retained = if cfg!(feature = "context") { 0 } else { 1 };
        assert_eq!(drops.load(Ordering::SeqCst), retained);
        let kind = if erased {
            let union: ErrorUnion = union.into();
            TrackedKind::try_from(union).unwrap()
        } else {
            TrackedKind::from(union)
        };
        assert_eq!(kind, TrackedKind::Drop);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}

#[eros::error_enum_ref(Shared)]
#[eros::error_enum_kind(Kind)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(align(32))]
#[eros::error_enum_mut(Mutable)]
#[eros::error_enum(Owned)]
type Stacked = (fmt::Error,);

#[eros::error_enum_kind(DisabledKind)]
#[cfg_attr(all(), cfg(any()))]
type DisabledSet = (fmt::Error,);

struct DisabledKind;
struct DisabledSet;

#[test]
fn kind_stacks_with_payload_macros_and_keeps_its_own_attributes() {
    let mut union: ErrorUnion<Stacked> = ErrorUnion::new(fmt::Error);
    let kind: Kind = (&union).into();
    let copied = kind;
    assert_eq!(kind, copied);
    assert_eq!(std::mem::align_of::<Kind>(), 32);
    assert!(matches!(Shared::from(&union), Shared::Fmt(_)));
    assert!(matches!(Mutable::from(&mut union), Mutable::Fmt(_)));
    assert!(matches!(Owned::from(union), Owned::Fmt(_)));
    let _disabled = (DisabledKind, DisabledSet);
}

mod exported {
    type Error = std::fmt::Error;

    #[eros::error_enum_kind(PublicKind)]
    pub type PublicSet = (Error,);
}

#[test]
fn singleton_kind_preserves_visibility_and_bare_error_variant_name() {
    let mut union: ErrorUnion<exported::PublicSet> = ErrorUnion::new(fmt::Error);
    assert!(matches!(
        exported::PublicKind::from(&union),
        exported::PublicKind::Error
    ));
    let _: exported::PublicKind = (&mut union).into();
    let _: exported::PublicKind = union.into();
}

#[allow(dead_code)]
trait SpoofTypeCheck {
    fn is_inner<T: 'static>(&self) -> bool {
        true
    }
}

impl<E: TypeSet> SpoofTypeCheck for &mut ErrorUnion<E> {}

#[test]
fn kind_dispatch_ignores_downstream_methods() {
    let mut typed: ErrorUnion<App> = ErrorUnion::new(fmt::Error);
    assert_eq!(AppKind::from(&mut typed), AppKind::Fmt);
    let mut erased: ErrorUnion = typed.into();
    assert_eq!(AppKind::try_from(&mut erased).unwrap(), AppKind::Fmt);
    let mut unlisted: ErrorUnion = ErrorUnion::new(UnlistedError(fmt::Error));
    assert!(AppKind::try_from(&mut unlisted).is_err());
}
