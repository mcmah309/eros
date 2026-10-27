use eros::ErrorUnion;

#[test]
fn debug_trace_fields_follow_their_own_feature_flags() {
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    for text in [format!("{error:?}"), format!("{:?}", error.report())] {
        assert_eq!(
            text.contains("\nspan trace:"),
            cfg!(feature = "spantrace"),
            "{text}"
        );
        assert_eq!(
            text.contains("\nbacktrace:"),
            cfg!(feature = "backtrace"),
            "{text}"
        );
        #[cfg(feature = "spantrace")]
        assert!(text.contains("\nspan trace: not captured"), "{text}");
        assert!(!text.contains("(captured)"), "{text}");
    }
}
