#![cfg(not(feature = "alloc"))]

#[path = "no_alloc/classification.rs"]
mod classification;

#[test]
fn classification_preserves_known_types_and_replaces_unknown_types_without_alloc() {
    classification::run_checks();
}

#[test]
fn panicking_classifier_drops_the_original_once() {
    use core::{
        error::Error,
        fmt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct Root;
    impl fmt::Display for Root {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("root")
        }
    }
    impl Error for Root {}
    impl Drop for Root {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    let error: eros::ErrorUnion = eros::ErrorUnion::new(Root);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.classify::<(fmt::Error,), _, _>(|_| -> fmt::Error { panic!("classification failed") })
    }));
    assert!(outcome.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}
