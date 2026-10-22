use core::error::Error;
use core::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use eros::{ErrorUnion, SendSyncError, MsgError};

#[derive(Debug)]
struct ConfigError {
    cause: MsgError,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cannot open configuration")
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

fn config_error() -> ConfigError {
    ConfigError {
        cause: MsgError::from("permission denied"),
    }
}

#[derive(Debug)]
struct StartupError {
    message: String,
    source: Box<dyn SendSyncError>,
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for StartupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

#[test]
fn closure_owns_the_typed_root_and_can_explicitly_retain_it_as_source() {
    let error: ErrorUnion<(ConfigError, fmt::Error)> = ErrorUnion::new(config_error());
    let message = String::from("startup failed");
    let mut calls = 0;
    let error: ErrorUnion<(StartupError,)> = error.map_inner(|old| {
        calls += 1;
        assert!(old.as_ref().as_any().is::<ConfigError>());
        StartupError {
            message,
            source: old,
        }
    });

    assert_eq!(calls, 1);
    assert!(error.is_inner::<StartupError>());
    assert_eq!(
        error.downcast_inner_ref::<StartupError>().unwrap().message,
        "startup failed"
    );
    assert_eq!(
        error.to_string(),
        "startup failed <- cannot open configuration <- permission denied"
    );
    let previous = error
        .source()
        .unwrap()
        .downcast_ref::<ConfigError>()
        .unwrap();
    assert_eq!(previous.cause.as_str(), "permission denied");
    assert!(previous.source().unwrap().is::<MsgError>());
}

#[test]
fn closure_can_replace_the_old_root_with_an_unrelated_error_and_chain() {
    let error = eros::error!("original failure").map_inner(|_old| config_error());

    assert!(error.is_inner::<ConfigError>());
    assert_eq!(
        error.to_string(),
        "cannot open configuration <- permission denied"
    );
    assert_eq!(error.source().unwrap().to_string(), "permission denied");
    assert!(!format!("{error:#?}").contains("original failure"));
    #[cfg(feature = "diagnostic")]
    assert_eq!(
        error.to_display_json(),
        serde_json::json!({
            "root": "cannot open configuration",
            "sources": ["permission denied"]
        })
    );
}

#[test]
fn closure_can_drop_the_source_chain_completely() {
    let error: ErrorUnion = ErrorUnion::new(config_error());
    let error = error.map_inner(|_| MsgError::from("configuration unavailable"));

    assert_eq!(error.to_string(), "configuration unavailable");
    assert!(error.source().is_none());
    assert_eq!(format!("{error:#?}"), "configuration unavailable");
}

#[test]
fn identity_and_repeated_replacements_receive_the_actual_root() {
    let error: ErrorUnion = ErrorUnion::new(config_error());
    let report = format!("{error:?}");
    let error = error.map_inner(|old| {
        let old: Box<dyn core::any::Any> = old;
        *old.downcast::<ConfigError>().unwrap()
    });
    assert!(error.is_inner::<ConfigError>());
    assert_eq!(format!("{error:?}"), report);

    let error = error.map_inner(|old| {
        assert!(old.as_ref().as_any().is::<ConfigError>());
        StartupError {
            message: "startup failed".into(),
            source: old,
        }
    });
    let error = error.map_inner(|old| {
        assert!(old.as_ref().as_any().is::<StartupError>());
        let old: Box<dyn core::any::Any> = old;
        *old.downcast::<StartupError>().unwrap()
    });
    assert!(error.is_inner::<StartupError>());
    assert_eq!(
        error.to_string(),
        "startup failed <- cannot open configuration <- permission denied"
    );

    let error = error.map_inner(|old| {
        assert!(old.as_ref().as_any().is::<StartupError>());
        MsgError::from("replacement leaf")
    });
    assert!(error.is_inner::<MsgError>());
    assert_eq!(
        error.downcast_inner::<MsgError>().unwrap().as_str(),
        "replacement leaf"
    );
}

#[test]
fn closure_can_downcast_and_consume_the_old_error() {
    let error: ErrorUnion = ErrorUnion::new(config_error());
    let error = error.map_inner(|old| {
        let old: Box<dyn core::any::Any> = old;
        let old = old.downcast::<ConfigError>().unwrap();
        old.cause
    });

    assert!(error.is_inner::<MsgError>());
    assert_eq!(error.to_string(), "permission denied");
    assert!(error.source().is_none());
}

#[test]
fn extracted_box_has_the_actual_returned_root_type() {
    let error = eros::error!("original failure").map_inner(|_| config_error());
    let boxed = error.into_inner();
    assert!(boxed.as_ref().as_any().is::<ConfigError>());
    let boxed: Box<dyn core::any::Any> = boxed;
    let root = boxed.downcast::<ConfigError>().unwrap();
    assert_eq!(root.cause.as_str(), "permission denied");
}

#[test]
fn intentionally_boxed_root_keeps_its_type_and_owned_allocation() {
    let root = Box::new(config_error());
    let original_pointer = &*root as *const ConfigError;
    let error = eros::error!("original failure").map_inner(|_| root);
    assert!(error.is_inner::<Box<ConfigError>>());
    assert!(!error.is_inner::<ConfigError>());

    let error = error.map_inner(|old| {
        assert!(old.as_ref().as_any().is::<Box<ConfigError>>());
        let old: Box<dyn core::any::Any> = old;
        *old.downcast::<Box<ConfigError>>().unwrap()
    });
    let boxed: Box<dyn core::any::Any> = error.into_inner();
    let root = *boxed.downcast::<Box<ConfigError>>().unwrap();
    assert_eq!(&*root as *const ConfigError, original_pointer);
    assert_eq!(root.cause.as_str(), "permission denied");
}

#[repr(align(256))]
#[derive(Debug)]
struct AlignedError {
    payload: Vec<u8>,
    drops: Arc<AtomicUsize>,
}

impl fmt::Display for AlignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "aligned error ({} bytes)", self.payload.len())
    }
}

impl Error for AlignedError {}

impl Drop for AlignedError {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn aligned_owned_root_supports_reference_mutable_and_owned_downcasts() {
    let drops = Arc::new(AtomicUsize::new(0));
    let payload = vec![37; 4097];
    let original_pointer = payload.as_ptr();
    let mut error = eros::error!("original failure").map_inner(|_| AlignedError {
        payload,
        drops: drops.clone(),
    });

    assert!(error.is_inner::<AlignedError>());
    assert!(error.downcast_inner_ref::<ConfigError>().is_none());
    assert!(error.downcast_inner_mut::<ConfigError>().is_none());
    let root = error.downcast_inner_ref::<AlignedError>().unwrap();
    assert_eq!((root as *const AlignedError).addr() % 256, 0);
    assert_eq!(root.payload.as_ptr(), original_pointer);
    assert!(root.payload.iter().all(|byte| *byte == 37));
    assert_eq!(drops.load(Ordering::SeqCst), 0);

    error.downcast_inner_mut::<AlignedError>().unwrap().payload[4096] = 91;
    assert_eq!(error.to_string(), "aligned error (4097 bytes)");
    let root = error.downcast_inner::<AlignedError>().unwrap();
    assert_eq!(root.payload.as_ptr(), original_pointer);
    assert_eq!(root.payload[4096], 91);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(root);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn aligned_anyerror_root_can_be_retained_as_source_then_replaced() {
    let drops = Arc::new(AtomicUsize::new(0));
    let payload = vec![37; 4097];
    let original_pointer = payload.as_ptr();
    let error: ErrorUnion<eros::AnyError> = ErrorUnion::new(AlignedError {
        payload,
        drops: drops.clone(),
    });
    #[cfg(feature = "context")]
    let context_drops = Arc::new(AtomicUsize::new(0));
    #[cfg(feature = "context")]
    let error = error.context(Box::new(DropError(context_drops.clone())) as Box<dyn SendSyncError>);

    let error = error.map_inner(|old| {
        let root = old
            .as_ref()
            .as_any()
            .downcast_ref::<AlignedError>()
            .unwrap();
        assert_eq!((root as *const AlignedError).addr() % 256, 0);
        assert_eq!(root.payload.as_ptr(), original_pointer);
        assert!(root.payload.iter().all(|byte| *byte == 37));
        StartupError {
            message: "startup failed".into(),
            source: old,
        }
    });
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<AlignedError>()
        .unwrap();
    assert_eq!(source.payload.as_ptr(), original_pointer);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 0);

    let error: ErrorUnion<eros::AnyError> = error.into();
    let error = error.map_inner(|old| {
        drop(old);
        MsgError::from("replacement leaf")
    });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 0);
    drop(error);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn wrong_type_owned_downcast_retains_root_and_context_until_drop() {
    let drops = Arc::new(AtomicUsize::new(0));
    let error = eros::error!("original failure").map_inner(|_| AlignedError {
        payload: vec![73; 8193],
        drops: drops.clone(),
    });
    #[cfg(feature = "context")]
    let context_drops = Arc::new(AtomicUsize::new(0));
    #[cfg(feature = "context")]
    let error = error.context(Box::new(DropError(context_drops.clone())) as Box<dyn SendSyncError>);

    let original = error.inner() as *const dyn SendSyncError as *const ();
    let report = format!("{error:?}");
    let error = error.downcast_inner::<ConfigError>().unwrap_err();
    assert_eq!(
        error.inner() as *const dyn SendSyncError as *const (),
        original
    );
    assert_eq!(format!("{error:?}"), report);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 0);
    drop(error);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 1);
}

static ZERO_SIZED_DROPS: AtomicUsize = AtomicUsize::new(0);

#[repr(align(64))]
#[derive(Debug)]
struct ZeroSizedError;

impl fmt::Display for ZeroSizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("zero sized error")
    }
}

impl Error for ZeroSizedError {}

impl Drop for ZeroSizedError {
    fn drop(&mut self) {
        ZERO_SIZED_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn zero_sized_root_moves_through_identity_extraction_and_drop_once() {
    assert_eq!(core::mem::size_of::<ZeroSizedError>(), 0);
    assert_eq!(core::mem::align_of::<ZeroSizedError>(), 64);
    let before = ZERO_SIZED_DROPS.load(Ordering::SeqCst);
    let error = eros::error!("original failure").map_inner(|_| ZeroSizedError);
    assert!(error.is_inner::<ZeroSizedError>());
    let root = error.downcast_inner_ref::<ZeroSizedError>().unwrap();
    assert_eq!((root as *const ZeroSizedError).addr() % 64, 0);
    assert_eq!(ZERO_SIZED_DROPS.load(Ordering::SeqCst), before);

    let error = error.map_inner(|old| {
        assert!(old.as_ref().as_any().is::<ZeroSizedError>());
        let old: Box<dyn core::any::Any> = old;
        *old.downcast::<ZeroSizedError>().unwrap()
    });
    let boxed: Box<dyn core::any::Any> = error.into_inner();
    let root = boxed.downcast::<ZeroSizedError>().unwrap();
    assert_eq!(ZERO_SIZED_DROPS.load(Ordering::SeqCst), before);
    drop(root);
    assert_eq!(ZERO_SIZED_DROPS.load(Ordering::SeqCst), before + 1);

    let error = eros::error!("original failure").map_inner(|_| ZeroSizedError);
    drop(error);
    assert_eq!(ZERO_SIZED_DROPS.load(Ordering::SeqCst), before + 2);
}

#[derive(Debug)]
struct DropError(Arc<AtomicUsize>);

impl fmt::Display for DropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("counted failure")
    }
}
impl Error for DropError {}
impl Drop for DropError {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn replacement_drops_old_and_new_roots_exactly_once() {
    let old_drops = Arc::new(AtomicUsize::new(0));
    let new_drops = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion = ErrorUnion::new(DropError(old_drops.clone()));
    let error = error.map_inner(|old| {
        drop(old);
        DropError(new_drops.clone())
    });

    assert_eq!(old_drops.load(Ordering::SeqCst), 1);
    assert_eq!(new_drops.load(Ordering::SeqCst), 0);
    drop(error);
    assert_eq!(old_drops.load(Ordering::SeqCst), 1);
    assert_eq!(new_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn panicking_closure_drops_root_and_context_without_leaking() {
    let root_drops = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion = ErrorUnion::new(DropError(root_drops.clone()));
    #[cfg(feature = "context")]
    let context_drops = Arc::new(AtomicUsize::new(0));
    #[cfg(feature = "context")]
    let error = error.context(Box::new(DropError(context_drops.clone())) as Box<dyn SendSyncError>);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: ErrorUnion<(MsgError,)> = error.map_inner(|_old| panic!("replacement failed"));
    }));

    assert!(result.is_err());
    assert_eq!(root_drops.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(context_drops.load(Ordering::SeqCst), 1);
}
