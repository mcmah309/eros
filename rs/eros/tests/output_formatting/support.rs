use eros::ErrorReport;

pub struct Fixtures {
    pub display: &'static str,
    pub display_special: &'static str,
    pub debug: &'static str,
    pub debug_special: &'static str,
    pub debug_unlocated: Option<&'static str>,
    #[cfg(feature = "json")]
    pub display_json: &'static str,
    #[cfg(feature = "json")]
    pub debug_json: &'static str,
}

fn trace_status(report: ErrorReport<'_>) -> &'static str {
    #[cfg(feature = "backtrace")]
    {
        match report.backtrace().status() {
            std::backtrace::BacktraceStatus::Captured => "captured",
            std::backtrace::BacktraceStatus::Disabled => "disabled",
            std::backtrace::BacktraceStatus::Unsupported => "unsupported",
            _ => "unknown",
        }
    }
    #[cfg(not(feature = "backtrace"))]
    {
        let _ = report;
        "feature_disabled"
    }
}

fn visit_reports(report: ErrorReport<'_>, visitor: &mut impl FnMut(ErrorReport<'_>)) {
    visitor(report);
    report.visit_children(&mut |child| visit_reports(child, visitor));
}

#[cfg(any(
    feature = "error_span_trace",
    all(feature = "context", feature = "context_span_trace")
))]
fn normalize_span_locations(trace: &tracing_error::SpanTrace, mut text: String) -> String {
    // Keep span names, fields, frame order, and indentation in the fixtures;
    // only source locations change when the test file is edited or moved.
    trace.with_spans(|metadata, _fields| {
        if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
            let location = format!("location: {file}:{line}");
            text = text
                .split('\n')
                .map(|line| {
                    if line.trim_start() == location {
                        line.replace(&location, "location: <location>")
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
        true
    });
    text
}

#[allow(unused_mut)] // Location replacement is feature-dependent.
fn normalize_debug(report: ErrorReport<'_>, mut text: String) -> String {
    let mut statuses = Vec::new();
    visit_reports(report, &mut |report| {
        statuses.push(trace_status(report));
        #[cfg(feature = "error_span_trace")]
        {
            text = normalize_span_locations(report.span_trace(), std::mem::take(&mut text));
        }
        #[cfg(all(feature = "context", feature = "context_span_trace"))]
        for frame in report.contexts() {
            text = normalize_span_locations(frame.span_trace(), std::mem::take(&mut text));
        }
        #[cfg(feature = "error_location")]
        {
            text = text.replace(
                &format!("location: {}", report.location()),
                "location: <location>",
            );
        }
        #[cfg(all(feature = "context", feature = "context_location"))]
        for frame in report.contexts() {
            text = text.replace(
                &format!("location: {}", frame.location()),
                "location: <location>",
            );
        }
    });
    let lines: Vec<_> = text.split('\n').collect();
    let mut normalized = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if let Some(status) = trimmed.strip_prefix("backtrace:") {
            let status = match status.trim() {
                "" => "captured",
                "not captured" => "disabled",
                status => status,
            };
            assert!(
                statuses.contains(&status),
                "unexpected backtrace status: {status}"
            );
            let indent = line.len() - trimmed.len();
            normalized.push(format!("{}backtrace: <status>", " ".repeat(indent)));
            index += 1;
            // Captured frames are volatile. Only the indented trace body is
            // omitted; blank separators and subsequent report fields remain.
            while index < lines.len()
                && lines[index]
                    .bytes()
                    .take_while(|byte| *byte == b' ')
                    .count()
                    > indent
            {
                index += 1;
            }
        } else {
            normalized.push(line.to_owned());
            index += 1;
        }
    }
    normalized.join("\n")
}

fn expected_debug(text: &str) -> String {
    let mut context_indent = None;
    let mut span_indent = None;
    text.split('\n')
        .filter(|line| {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            if trimmed == "context:" {
                context_indent = Some(indent);
            } else if context_indent.is_some_and(|context| indent <= context) {
                context_indent = None;
            }
            if trimmed == "span trace:" {
                span_indent = Some(indent);
            } else if span_indent.is_some_and(|span| indent <= span) {
                span_indent = None;
            }
            (!trimmed.starts_with("location: <location>")
                || span_indent.is_some()
                || if context_indent.is_some_and(|context| indent == context + 6) {
                    cfg!(feature = "context_location")
                } else {
                    cfg!(feature = "error_location")
                })
                && (!trimmed.starts_with("span trace:")
                    || if context_indent.is_some_and(|context| indent == context + 6) {
                        cfg!(feature = "context_span_trace")
                    } else {
                        cfg!(feature = "error_span_trace")
                    })
                && (cfg!(feature = "backtrace") || !line.trim_start().starts_with("backtrace:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn canonical_text(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            if line.bytes().all(|byte| byte == b' ') {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn record(case: &str, file: &str, actual: &str) -> bool {
    let update = std::env::var("EROS_UPDATE_OUTPUT_FIXTURES").unwrap_or_default();
    let should_write = match update.as_str() {
        "1" => {
            assert!(
                cfg!(all(
                    feature = "json",
                    feature = "context",
                    feature = "error_location",
                    feature = "context_location",
                    feature = "error_span_trace",
                    feature = "context_span_trace"
                )),
                "record fixtures with --all-features"
            );
            true
        }
        "debug-unlocated" => file == "debug.unlocated.txt",
        "" => false,
        _ => panic!("unknown EROS_UPDATE_OUTPUT_FIXTURES value"),
    };
    if should_write {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/output_formatting")
            .join(case);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(file), actual).unwrap();
    }
    should_write
}

fn assert_text(case: &str, file: &str, actual: &str, expected: &str) {
    let actual = canonical_text(actual) + "\n";
    println!("\n--- {case}/{file} ---\n{actual}");
    if !record(case, file, &actual) {
        assert!(
            actual == expected,
            "Output differs from output_formatting/{case}/{file}\n\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
        );
    }
}

pub fn assert_case(case: &str, report: ErrorReport<'_>, fixtures: Fixtures) {
    assert_text(case, "display.txt", &format!("{report}"), fixtures.display);
    assert_text(
        case,
        "display.special.txt",
        &format!("{report:#}"),
        fixtures.display_special,
    );
    assert_text(
        case,
        "debug.special.txt",
        &format!("{report:#?}"),
        fixtures.debug_special,
    );
    let basic = !cfg!(feature = "error_location") && trace_status(report) != "captured";
    let (file, expected) = if basic {
        fixtures
            .debug_unlocated
            .map(|text| ("debug.unlocated.txt", text))
            .unwrap_or(("debug.txt", fixtures.debug))
    } else {
        ("debug.txt", fixtures.debug)
    };
    assert_text(
        case,
        file,
        &normalize_debug(report, format!("{report:?}")),
        &expected_debug(expected),
    );
    #[cfg(feature = "json")]
    assert_json(case, report, fixtures.display_json, fixtures.debug_json);
}

#[cfg(all(
    feature = "json",
    any(
        feature = "error_location",
        all(feature = "context", feature = "context_location")
    )
))]
fn assert_location(value: &mut serde_json::Value, location: &core::panic::Location<'_>) {
    assert_eq!(
        *value,
        serde_json::json!({
            "file": location.file(), "line": location.line(), "column": location.column(),
        })
    );
    *value = serde_json::json!({"file": "<file>", "line": 0, "column": 0});
}

#[cfg(all(
    feature = "json",
    any(
        feature = "error_span_trace",
        all(feature = "context", feature = "context_span_trace")
    )
))]
fn normalize_span_json(trace: &tracing_error::SpanTrace, value: &mut serde_json::Value) {
    use tracing_error::SpanTraceStatus;
    if trace.status() == SpanTraceStatus::CAPTURED {
        let text = value["span_trace"]
            .as_str()
            .expect("captured span trace text");
        trace.with_spans(|metadata, fields| {
            assert!(text.contains(&format!("- {}::{}", metadata.target(), metadata.name())));
            if !fields.is_empty() {
                let expected = if fields.lines().nth(1).is_some() {
                    let lines = fields
                        .lines()
                        .map(|line| format!("    {line}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("  with:\n{lines}")
                } else {
                    format!("  with: {fields}")
                };
                assert!(text.contains(&expected));
            }
            if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
                assert!(text.contains(&format!("  location: {file}:{line}")));
            }
            true
        });
        value["span_trace"] = normalize_span_locations(trace, text.to_owned()).into();
    } else {
        assert_eq!(value.get("span_trace"), Some(&serde_json::Value::Null));
    }
}

#[cfg(feature = "json")]
fn normalize_json(report: ErrorReport<'_>, value: &mut serde_json::Value) {
    #[cfg(feature = "error_location")]
    assert_location(&mut value["location"], report.location());
    #[cfg(not(feature = "error_location"))]
    assert!(value.get("location").is_none());
    let contexts = value["contexts"]
        .as_array_mut()
        .expect("debug contexts array");
    #[cfg(feature = "context")]
    {
        assert_eq!(contexts.len(), report.contexts().len());
        for (value, frame) in contexts.iter_mut().zip(report.contexts()) {
            assert_eq!(value["message"], frame.to_string());
            #[cfg(feature = "user_context")]
            assert_eq!(value["user_facing"], frame.is_user_context());
            #[cfg(not(feature = "user_context"))]
            assert_eq!(value["user_facing"], false);
            #[cfg(feature = "context_location")]
            assert_location(&mut value["location"], frame.location());
            #[cfg(not(feature = "context_location"))]
            assert!(value.get("location").is_none());
            #[cfg(feature = "context_span_trace")]
            normalize_span_json(frame.span_trace(), value);
            #[cfg(not(feature = "context_span_trace"))]
            assert!(value.get("span_trace").is_none());
        }
    }
    #[cfg(not(feature = "context"))]
    assert!(contexts.is_empty());
    #[cfg(feature = "backtrace")]
    {
        let trace = value.get("backtrace").expect("enabled backtrace field");
        if trace_status(report) == "captured" {
            let frames = trace.as_str().expect("captured frames must be text");
            assert!(!frames.trim().is_empty());
            assert!(
                !frames.contains('\u{1b}'),
                "JSON traces must not contain ANSI escapes"
            );
            let human = format!("{report:?}");
            let heading = human
                .lines()
                .rfind(|line| line.trim_start() == "backtrace:")
                .expect("captured backtrace heading");
            let indent = heading.len() - heading.trim_start().len() + 4;
            let indented = frames
                .lines()
                .map(|line| format!("{:indent$}{line}", ""))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                human.ends_with(&indented),
                "JSON must retain the same captured frames as ordinary Debug"
            );
        } else {
            assert!(trace.is_null());
        }
        // Captured stack frames are volatile and checked above.
        value["backtrace"] = serde_json::Value::Null;
    }
    #[cfg(not(feature = "backtrace"))]
    assert!(value.get("backtrace").is_none());
    #[cfg(feature = "error_span_trace")]
    normalize_span_json(report.span_trace(), value);
    #[cfg(not(feature = "error_span_trace"))]
    assert!(value.get("span_trace").is_none());
    let mut children = Vec::new();
    report.visit_children(&mut |child| children.push(child));
    if children.is_empty() {
        assert!(value.get("children").is_none());
    } else {
        let output = value["children"]
            .as_array_mut()
            .expect("JSON children array");
        assert_eq!(output.len(), children.len());
        for (child, output) in children.into_iter().zip(output) {
            normalize_json(child, output);
        }
    }
}

#[cfg(feature = "json")]
fn adapt_expected_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            let include_location = if object.contains_key("message") {
                cfg!(feature = "context_location")
            } else {
                cfg!(feature = "error_location")
            };
            if !include_location {
                object.remove("location");
            }
            let include_span_trace = if object.contains_key("message") {
                cfg!(feature = "context_span_trace")
            } else {
                cfg!(feature = "error_span_trace")
            };
            if !include_span_trace {
                object.remove("span_trace");
            }
            if !cfg!(feature = "backtrace") {
                object.remove("backtrace");
            }
            for value in object.values_mut() {
                adapt_expected_json(value);
            }
        }
        serde_json::Value::Array(array) => {
            for value in array {
                adapt_expected_json(value);
            }
        }
        _ => {}
    }
}

#[cfg(feature = "json")]
fn display_projection(mut value: serde_json::Value) -> serde_json::Value {
    let object = value.as_object_mut().unwrap();
    for key in ["contexts", "location", "backtrace", "span_trace"] {
        object.remove(key);
    }
    if let Some(children) = object.get_mut("children") {
        for child in children.as_array_mut().unwrap() {
            *child = display_projection(child.take());
        }
    }
    value
}

#[cfg(feature = "json")]
fn assert_json(case: &str, report: ErrorReport<'_>, display_fixture: &str, debug_fixture: &str) {
    let display = report.to_display_json();
    let mut debug = report.to_debug_json();
    assert_eq!(
        display_projection(debug.clone()),
        display,
        "debug JSON must enrich display JSON recursively"
    );
    normalize_json(report, &mut debug);
    for (file, actual, fixture) in [
        ("display.json", display, display_fixture),
        ("debug.json", debug, debug_fixture),
    ] {
        let rendered = serde_json::to_string_pretty(&actual).unwrap() + "\n";
        println!("\n--- {case}/{file} ---\n{rendered}");
        if !record(case, file, &rendered) {
            let mut expected: serde_json::Value = serde_json::from_str(fixture)
                .unwrap_or_else(|error| panic!("invalid JSON in {case}/{file}: {error}"));
            adapt_expected_json(&mut expected);
            assert_eq!(
                actual, expected,
                "JSON differs from output_formatting/{case}/{file}"
            );
        }
    }
}
