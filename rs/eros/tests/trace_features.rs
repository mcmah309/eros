use eros::ErrorUnion;

#[test]
fn debug_trace_fields_follow_their_own_feature_flags() {
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    for text in [format!("{error:?}"), format!("{:?}", error.report())] {
        assert_eq!(
            text.contains("\nspan trace:"),
            cfg!(feature = "error_spantrace"),
            "{text}"
        );
        assert_eq!(
            text.contains("\nbacktrace:"),
            cfg!(feature = "backtrace"),
            "{text}"
        );
        #[cfg(feature = "error_spantrace")]
        assert!(text.contains("\nspan trace: not captured"), "{text}");
        assert!(!text.contains("(captured)"), "{text}");
    }
}

#[cfg(feature = "context")]
#[test]
fn context_trace_fields_follow_the_context_feature_flag() {
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    let error = error.context("attached");
    for text in [format!("{error:?}"), format!("{:?}", error.report())] {
        assert_eq!(
            text.contains("\n      span trace:"),
            cfg!(feature = "context_spantrace"),
            "{text}"
        );
    }
    #[cfg(feature = "diagnostic")]
    {
        let json = error.to_debug_json();
        assert_eq!(
            json.get("span_trace").is_some(),
            cfg!(feature = "error_spantrace")
        );
        assert_eq!(
            json["contexts"][0].get("span_trace").is_some(),
            cfg!(feature = "context_spantrace")
        );
    }
}
