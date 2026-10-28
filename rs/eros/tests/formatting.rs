#![cfg(feature = "alloc")]

//! The public formatting contract, including the values received by tracing.
//!
//! Run this target with `RUST_LIB_BACKTRACE=0` and `RUST_LIB_BACKTRACE=1` in
//! separate processes: Rust caches whether environment-controlled capture is on.

use std::{
    error::Error,
    fmt,
    io::{self, Write},
    sync::{Arc, Mutex},
};

use eros::{Context, ErrorUnion};

struct NativeError {
    message: &'static str,
    source: Option<Box<NativeError>>,
}

impl NativeError {
    fn leaf(message: &'static str) -> Self {
        Self {
            message,
            source: None,
        }
    }

    fn caused_by(message: &'static str, source: Self) -> Self {
        Self {
            message,
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("INNER ALTERNATE DISPLAY MUST NOT LEAK")
        } else {
            f.write_str(self.message)
        }
    }
}

impl fmt::Debug for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug is allowed to omit the source and use an entirely different form.
        // Eros's human report must read Display and Error::source() explicitly.
        f.write_str("INTERNAL DEBUG MUST NOT LEAK")
    }
}

impl Error for NativeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

#[inline(never)]
fn open_config_file() -> Result<(), ErrorUnion<(NativeError,)>> {
    Err(ErrorUnion::new(NativeError::caused_by(
        "cannot open configuration",
        NativeError::leaf("permission denied"),
    )))
}

fn read_config() -> Result<(), ErrorUnion<(NativeError,)>> {
    open_config_file().context("read /etc/app.toml")
}

fn start_service() -> Result<(), ErrorUnion<(NativeError,)>> {
    read_config().context("start service")
}

fn report_without_metadata() -> &'static str {
    if cfg!(feature = "context") {
        "error: cannot open configuration\ncaused by:\n    - permission denied\ncontext:\n    - read /etc/app.toml\n    - start service"
    } else {
        "error: cannot open configuration\ncaused by:\n    - permission denied"
    }
}

#[test]
fn display_shows_root_and_native_sources_while_alternate_shows_root_only() {
    let error = start_service().unwrap_err();

    assert_eq!(
        error.to_string(),
        "cannot open configuration\n    caused by:\n        - permission denied"
    );
    assert_eq!(format!("{error:#}"), "cannot open configuration");
    assert_eq!(format!("{error:#?}"), report_without_metadata());
}

#[test]
fn display_and_debug_walk_every_native_source_in_order() {
    let error: ErrorUnion<(NativeError,)> = ErrorUnion::new(NativeError::caused_by(
        "cannot open configuration",
        NativeError::caused_by(
            "permission denied",
            NativeError::leaf("access policy rejected"),
        ),
    ));

    assert_eq!(
        error.to_string(),
        "cannot open configuration\n    caused by:\n        - permission denied\n        - access policy rejected"
    );
    assert_eq!(
        format!("{error:#?}"),
        "error: cannot open configuration\ncaused by:\n    - permission denied\n    - access policy rejected"
    );
}

#[test]
fn bare_errors_have_no_empty_source_or_context_sections() {
    let error: ErrorUnion<(NativeError,)> =
        ErrorUnion::new(NativeError::leaf("configuration missing"));

    assert_eq!(error.to_string(), "configuration missing");
    assert_eq!(format!("{error:#?}"), "error: configuration missing");
    let report = format!("{error:?}");
    assert_eq!(report.lines().next(), Some("error: configuration missing"));
    assert!(!report.contains("caused by:"));
    assert!(!report.contains("context:"));
    assert!(!report.ends_with('\n'));
}

#[test]
fn erasing_the_union_preserves_every_format_and_captured_metadata() {
    let typed = start_service().unwrap_err();
    let expected = [
        format!("{typed}"),
        format!("{typed:#}"),
        format!("{typed:?}"),
        format!("{typed:#?}"),
    ];
    #[cfg(feature = "json")]
    let diagnostic_expected = [typed.to_display_json(), typed.to_debug_json()];
    let erased: ErrorUnion = typed.into();

    assert_eq!(
        [
            format!("{erased}"),
            format!("{erased:#}"),
            format!("{erased:?}"),
            format!("{erased:#?}"),
        ],
        expected
    );
    #[cfg(feature = "json")]
    assert_eq!(
        [erased.to_display_json(), erased.to_debug_json()],
        diagnostic_expected
    );
}

fn native_chain_messages(error: &(dyn Error + 'static)) -> Vec<String> {
    let mut messages = vec![error.to_string()];
    let mut source = error.source();
    while let Some(error) = source {
        messages.push(error.to_string());
        source = error.source();
    }
    messages
}

fn assert_native_adapter_roundtrip(error: ErrorUnion<(NativeError,)>) {
    let expected_display = error.to_string();
    let expected_debug = format!("{error:?}");
    let expected_alternate_debug = format!("{error:#?}");
    #[cfg(feature = "json")]
    let expected_diagnostic = error.to_debug_json();
    let original_root = error.inner() as *const dyn eros::SendSyncError as *const ();
    #[cfg(feature = "error_location")]
    let original_location = error.location();

    // The concrete adapter works at generic Error boundaries and keeps the
    // existing allocation, diagnostics, and exact typed set when recovered.
    fn accepts_error<E: Error + Send + Sync + 'static>(error: E) -> E {
        error
    }
    let native: eros::ErrorUnionAdapter<(NativeError,)> = accepts_error(error.into_adapter());
    assert_eq!(
        native.as_ref().inner() as *const dyn eros::SendSyncError as *const (),
        original_root
    );
    assert_eq!(native.to_string(), "cannot open configuration");
    assert_eq!(format!("{native:#}"), "cannot open configuration");
    assert_eq!(
        native_chain_messages(&native),
        ["cannot open configuration", "permission denied"]
    );
    assert_eq!(format!("{native:?}"), expected_debug);
    assert_eq!(format!("{native:#?}"), expected_alternate_debug);
    let error: ErrorUnion<(NativeError,)> = native.into_unadapted();
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(error.location(), original_location));
    let native: Box<dyn Error + Send + Sync> = Box::new(error.into_adapter());

    assert!(native.is::<eros::ErrorUnionAdapter<(NativeError,)>>());

    // A native Error reporter visits source() itself. Its root must therefore
    // format only the root message, or it would print each cause twice.
    assert_eq!(native.to_string(), "cannot open configuration");
    assert_eq!(format!("{native:#}"), "cannot open configuration");
    assert_eq!(
        native_chain_messages(native.as_ref()),
        ["cannot open configuration", "permission denied"]
    );
    assert_eq!(format!("{native:?}"), expected_debug);
    assert_eq!(format!("{native:#?}"), expected_alternate_debug);

    let recovered = native
        .downcast::<eros::ErrorUnionAdapter<(NativeError,)>>()
        .unwrap()
        .into_unadapted();
    assert!(recovered.is_inner::<NativeError>());
    assert_eq!(recovered.to_string(), expected_display);
    assert_eq!(format!("{recovered:?}"), expected_debug);
    assert_eq!(format!("{recovered:#?}"), expected_alternate_debug);
    #[cfg(feature = "json")]
    assert_eq!(recovered.to_debug_json(), expected_diagnostic);
}

#[test]
fn typed_native_error_adapter_has_a_single_root_and_roundtrips_metadata() {
    assert_native_adapter_roundtrip(start_service().unwrap_err());
}

#[derive(Debug)]
struct StartupError(Box<dyn eros::SendSyncError>);

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("startup failed")
    }
}

impl Error for StartupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

#[test]
fn replacing_an_anyerror_root_preserves_metadata_in_every_format() {
    let error: ErrorUnion<eros::AnyError> = start_service().unwrap_err().into();
    let original_report = format!("{error:?}");
    let (_, contexts_and_backtrace) = original_report
        .split_once("\ncaused by:\n    - permission denied")
        .unwrap();
    #[cfg(feature = "json")]
    let mut expected_diagnostic = error.to_debug_json();

    let error: ErrorUnion<(StartupError,)> = error.map_inner(|old| {
        assert!((old.as_ref() as &dyn core::any::Any).is::<NativeError>());
        StartupError(old)
    });
    // The new root keeps the old root as a cause. All saved metadata stays in
    // its original place, including the exact captured frames and locations.
    let expected = [
        "startup failed\n    caused by:\n        - cannot open configuration\n        - permission denied".to_owned(),
        "startup failed".to_owned(),
        format!(
            "error: startup failed\ncaused by:\n    - cannot open configuration\n    - permission denied{contexts_and_backtrace}"
        ),
        format!(
            "error: startup failed\ncaused by:\n    - cannot open configuration\n    - permission denied{}",
            if cfg!(feature = "context") {
                "\ncontext:\n    - read /etc/app.toml\n    - start service"
            } else {
                ""
            }
        ),
    ];
    assert_eq!(
        [
            format!("{error}"),
            format!("{error:#}"),
            format!("{error:?}"),
            format!("{error:#?}"),
        ],
        expected
    );
    #[cfg(feature = "json")]
    let expected_display = serde_json::json!({
        "root": "startup failed",
        "sources": ["cannot open configuration", "permission denied"],
    });
    #[cfg(feature = "json")]
    {
        expected_diagnostic["root"] = expected_display["root"].clone();
        expected_diagnostic["sources"] = expected_display["sources"].clone();
        assert_eq!(error.to_display_json(), expected_display);
        assert_eq!(error.to_debug_json(), expected_diagnostic);
    }

    let error: ErrorUnion<eros::AnyError> = error.into();
    assert_eq!(
        [
            format!("{error}"),
            format!("{error:#}"),
            format!("{error:?}"),
            format!("{error:#?}"),
        ],
        expected
    );
    #[cfg(feature = "json")]
    {
        assert_eq!(error.to_display_json(), expected_display);
        assert_eq!(error.to_debug_json(), expected_diagnostic);
    }
}

#[cfg(feature = "anyhow")]
#[inline(never)]
fn anyhow_failure_origin() -> anyhow::Error {
    anyhow::Error::new(NativeError::caused_by(
        "cannot open configuration",
        NativeError::leaf("permission denied"),
    ))
    .context("load configuration")
    .context("initialize application")
}

#[cfg(feature = "anyhow")]
#[test]
fn anyhow_owned_and_shared_adapters_preserve_source_order_after_new_root() {
    for error in [
        ErrorUnion::from_anyhow(anyhow_failure_origin()),
        ErrorUnion::from_anyhow_arc(Arc::new(anyhow_failure_origin())),
    ] {
        let error = error.context("start service");
        let expected_sources = [
            "initialize application",
            "load configuration",
            "cannot open configuration",
            "permission denied",
        ];
        assert_eq!(
            error.to_string(),
            "initialize application\n    caused by:\n        - load configuration\n        - cannot open configuration\n        - permission denied"
        );
        assert_eq!(format!("{error:#}"), "initialize application");
        assert_eq!(native_chain_messages(error.inner()), expected_sources);
        let mut expected_report = "error: initialize application\ncaused by:\n    - load configuration\n    - cannot open configuration\n    - permission denied".to_owned();
        if cfg!(feature = "context") {
            expected_report.push_str("\ncontext:\n    - start service");
        }
        assert_eq!(format!("{error:#?}"), expected_report);

        let error = error.map_inner(StartupError);
        assert_eq!(
            error.to_string(),
            "startup failed\n    caused by:\n        - initialize application\n        - load configuration\n        - cannot open configuration\n        - permission denied"
        );
        assert_eq!(
            native_chain_messages(error.source().unwrap()),
            expected_sources
        );
        #[cfg(feature = "json")]
        assert_eq!(
            error.to_display_json(),
            serde_json::json!({"root": "startup failed", "sources": expected_sources})
        );
    }
}

#[cfg(all(feature = "anyhow", feature = "backtrace"))]
#[test]
fn anyhow_owned_and_shared_reports_keep_the_original_capture() {
    for shared in [false, true] {
        let original = anyhow_failure_origin();
        let original_status = original.backtrace().status();
        #[cfg(not(feature = "better_backtrace"))]
        let original_frames = original.backtrace().to_string();
        let error = if shared {
            ErrorUnion::from_anyhow_arc(Arc::new(original))
        } else {
            ErrorUnion::from_anyhow(original)
        };
        let report = format!("{error:?}");
        let (_, original_section) = report.split_once("\nbacktrace:").unwrap();
        #[cfg(feature = "json")]
        if original_status == std::backtrace::BacktraceStatus::Captured {
            assert!(
                error.to_debug_json()["backtrace"]
                    .as_str()
                    .unwrap()
                    .contains("anyhow_failure_origin")
            );
        }

        match original_status {
            std::backtrace::BacktraceStatus::Captured => {
                let frames = original_section.strip_prefix('\n').unwrap();
                assert!(frames.contains("anyhow_failure_origin"), "{report}");
                #[cfg(not(feature = "better_backtrace"))]
                assert_eq!(
                    frames,
                    original_frames
                        .trim_end_matches('\n')
                        .lines()
                        .map(|line| format!("    {line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
            std::backtrace::BacktraceStatus::Disabled => {
                assert_eq!(original_section, " not captured");
            }
            std::backtrace::BacktraceStatus::Unsupported => {
                assert_eq!(original_section, " unsupported");
            }
            _ => panic!("unexpected backtrace status"),
        }
    }
}

#[test]
fn multiline_report_messages_keep_their_continuations_indented() {
    let error: ErrorUnion<(NativeError,)> = ErrorUnion::new(NativeError::caused_by(
        "cannot open configuration\nretry exhausted",
        NativeError::leaf("permission denied\naccess policy rejected"),
    ));
    let error = error.context("read /etc/app.toml\nselected by CONFIG_PATH");

    assert_eq!(
        error.to_string(),
        "cannot open configuration\nretry exhausted\n    caused by:\n        - permission denied\n          access policy rejected"
    );
    let mut expected = "error:\n  cannot open configuration\n  retry exhausted\ncaused by:\n    - permission denied\n      access policy rejected".to_owned();
    if cfg!(feature = "context") {
        expected.push_str("\ncontext:\n    - read /etc/app.toml\n      selected by CONFIG_PATH");
    }
    assert_eq!(format!("{error:#?}"), expected);
}

#[test]
fn option_context_keeps_the_absent_value_root() {
    let error = None::<()>
        .context("configuration path required")
        .unwrap_err();

    assert_eq!(error.to_string(), "An `Option` was unexpectedly `None`");
    let report = format!("{error:#?}");
    if cfg!(feature = "context") {
        assert_eq!(
            report,
            "error: An `Option` was unexpectedly `None`\ncontext:\n    - configuration path required"
        );
    } else {
        assert_eq!(report, "error: An `Option` was unexpectedly `None`");
    }
}

#[cfg(feature = "context")]
#[test]
fn context_follows_stack_order_and_error_context_uses_its_display() {
    let context: Box<dyn eros::SendSyncError> = Box::new(NativeError::leaf("read /etc/app.toml"));
    let error = open_config_file()
        .unwrap_err()
        .context(context)
        .context("start service");

    assert_eq!(format!("{error:#?}"), report_without_metadata());
}

#[cfg(any(feature = "error_location", feature = "context_location"))]
#[test]
fn debug_places_captured_locations_below_the_root_and_each_context() {
    let native = NativeError::leaf("configuration missing");
    let _root_line = line!() + 1;
    let error: ErrorUnion<(NativeError,)> = ErrorUnion::new(native);
    let _inner_line = line!() + 1;
    let error = error.context("read /etc/app.toml");
    let _outer_line = line!() + 1;
    let error = error.context("start service");
    let report = format!("{error:?}");

    assert_eq!(
        report.contains(&format!("\nlocation: {}:{_root_line}:", file!())),
        cfg!(feature = "error_location"),
        "{report}"
    );
    if cfg!(all(feature = "context", feature = "context_location")) {
        assert!(
            report.contains(&format!(
                "    - read /etc/app.toml\n      location: {}:{_inner_line}:",
                file!()
            )),
            "{report}"
        );
        assert!(
            report.contains(&format!(
                "    - start service\n      location: {}:{_outer_line}:",
                file!()
            )),
            "{report}"
        );
        assert!(report.find("read /etc/app.toml").unwrap() < report.find("start service").unwrap());
    }

    let alternate = format!("{error:#?}");
    assert!(!alternate.contains(file!()));
    assert!(!alternate.contains("backtrace"));
}

#[test]
fn debug_reports_the_actual_backtrace_capture_status() {
    let error = start_service().unwrap_err();
    let report = format!("{error:?}");

    #[cfg(feature = "backtrace")]
    match error.backtrace().status() {
        std::backtrace::BacktraceStatus::Captured => {
            let (_, frames) = report
                .split_once("\nbacktrace:\n")
                .expect("captured backtraces must be printed in the human report");
            assert!(!frames.trim().is_empty());
            assert!(frames.contains("open_config_file"), "{report}");
        }
        std::backtrace::BacktraceStatus::Disabled => {
            assert!(report.ends_with("\nbacktrace: not captured"), "{report}");
        }
        std::backtrace::BacktraceStatus::Unsupported => {
            assert!(report.ends_with("\nbacktrace: unsupported"), "{report}");
        }
        _ => panic!("unexpected backtrace status"),
    }
    #[cfg(not(feature = "backtrace"))]
    assert!(!report.contains("backtrace:"), "{report}");

    assert!(!report.ends_with('\n'));
    assert_eq!(format!("{error:#?}"), report_without_metadata());
}

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn tracing_percent_and_question_mark_use_the_public_human_formats() {
    let error = start_service().unwrap_err();
    let output = Buffer::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .without_time()
        .with_writer(move || writer.clone())
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        tracing::error!(
            error_display = %error,
            error_debug = ?error,
            error_short = %format_args!("{error:#}"),
            error_report_without_metadata = %format_args!("{error:#?}"),
            "startup failed"
        );
    });

    let bytes = output.0.lock().unwrap();
    let event: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(event["level"], "ERROR");
    assert_eq!(event["fields"]["message"], "startup failed");
    assert_eq!(
        event["fields"]["error_display"],
        "cannot open configuration\n    caused by:\n        - permission denied"
    );
    assert_eq!(event["fields"]["error_debug"], format!("{error:?}"));
    assert_eq!(event["fields"]["error_short"], "cannot open configuration");
    assert_eq!(
        event["fields"]["error_report_without_metadata"],
        report_without_metadata()
    );
}

#[cfg(feature = "json")]
#[test]
fn to_display_json_is_data_with_the_same_root_and_source_messages() {
    let error = start_service().unwrap_err();

    assert_eq!(
        error.to_display_json(),
        serde_json::json!({
            "root": "cannot open configuration",
            "sources": ["permission denied"],
        })
    );

    let bare = eros::error!("configuration missing");
    assert_eq!(
        bare.to_display_json(),
        serde_json::json!({"root": "configuration missing", "sources": []})
    );
}

#[cfg(feature = "json")]
#[test]
fn to_debug_json_includes_contexts_locations_and_enabled_traces() {
    let native = NativeError::caused_by(
        "cannot open configuration",
        NativeError::leaf("permission denied"),
    );
    let _root_line = line!() + 1;
    let error: ErrorUnion<(NativeError,)> = ErrorUnion::new(native);
    let _inner_line = line!() + 1;
    let error = error.context("read /etc/app.toml");
    let _outer_line = line!() + 1;
    let error = error.context("start service");
    let diagnostic = error.to_debug_json();

    assert_eq!(diagnostic["root"], "cannot open configuration");
    assert_eq!(
        diagnostic["sources"],
        serde_json::json!(["permission denied"])
    );
    assert_eq!(
        diagnostic.as_object().unwrap().len(),
        3 + usize::from(cfg!(feature = "error_location"))
            + usize::from(cfg!(feature = "error_spantrace"))
            + usize::from(cfg!(feature = "backtrace"))
    );
    #[cfg(feature = "error_spantrace")]
    assert_eq!(diagnostic.get("span_trace"), Some(&serde_json::Value::Null));
    #[cfg(not(feature = "error_spantrace"))]
    assert!(diagnostic.get("span_trace").is_none());

    #[cfg(feature = "error_location")]
    assert_json_location(&diagnostic["location"], _root_line);
    #[cfg(not(feature = "error_location"))]
    assert!(diagnostic.get("location").is_none());

    let contexts = diagnostic["contexts"].as_array().unwrap();
    if cfg!(feature = "context") {
        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0]["message"], "read /etc/app.toml");
        assert_eq!(contexts[1]["message"], "start service");
        for context in contexts {
            assert_eq!(context["user_facing"], false);
            assert_eq!(
                context.as_object().unwrap().len(),
                2 + usize::from(cfg!(feature = "context_location"))
                    + usize::from(cfg!(feature = "context_spantrace"))
            );
        }
        #[cfg(feature = "context_location")]
        {
            assert_json_location(&contexts[0]["location"], _inner_line);
            assert_json_location(&contexts[1]["location"], _outer_line);
        }
    } else {
        assert!(contexts.is_empty());
    }

    #[cfg(feature = "backtrace")]
    match error.backtrace().status() {
        std::backtrace::BacktraceStatus::Captured => {
            let text = diagnostic["backtrace"].as_str().unwrap();
            assert!(!text.trim().is_empty());
            let indented = text
                .lines()
                .map(|line| format!("    {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(format!("{error:?}").ends_with(&indented));
        }
        std::backtrace::BacktraceStatus::Disabled
        | std::backtrace::BacktraceStatus::Unsupported => {
            assert_eq!(diagnostic.get("backtrace"), Some(&serde_json::Value::Null));
        }
        _ => panic!("unexpected backtrace status"),
    }
    #[cfg(not(feature = "backtrace"))]
    assert!(diagnostic.get("backtrace").is_none());
}

#[cfg(all(
    feature = "json",
    any(feature = "error_location", feature = "context_location")
))]
fn assert_json_location(location: &serde_json::Value, line: u32) {
    assert_eq!(location.as_object().unwrap().len(), 3);
    assert_eq!(location["file"], file!());
    assert_eq!(location["line"], line);
    assert!(location["column"].as_u64().unwrap() > 0);
}

#[cfg(all(feature = "json", feature = "user_context"))]
#[test]
fn diagnostic_contexts_preserve_user_facing_markers_in_stack_order() {
    let error = eros::error!("permission denied")
        .context("read /etc/app.toml")
        .user_context("Choose a readable configuration file.");
    let diagnostic = error.to_debug_json();
    let contexts = diagnostic["contexts"].as_array().unwrap();

    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0]["message"], "read /etc/app.toml");
    assert_eq!(contexts[0]["user_facing"], false);
    assert_eq!(
        contexts[1]["message"],
        "Choose a readable configuration file."
    );
    assert_eq!(contexts[1]["user_facing"], true);
}
