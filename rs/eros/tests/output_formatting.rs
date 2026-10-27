//! Readable examples of the progressively enriched output formats.
//!
//! Expected reports and instructions live in `output_formatting/README.md`.
//! Run with `-- --nocapture --test-threads=1` to print the examples in order.

use eros::ErrorUnion;
#[cfg(feature = "alloc")]
use eros::ErrorUnionVec;

#[path = "output_formatting/support.rs"]
mod support;

macro_rules! assert_outputs {
    ($name:literal, $report:expr) => {
        assert_outputs!($name, $report, None)
    };
    ($name:literal, $report:expr, $unlocated:expr) => {
        support::assert_case(
            $name,
            $report,
            support::Fixtures {
                display: include_str!(concat!("output_formatting/", $name, "/display.txt")),
                display_special: include_str!(concat!(
                    "output_formatting/",
                    $name,
                    "/display.special.txt"
                )),
                debug: include_str!(concat!("output_formatting/", $name, "/debug.txt")),
                debug_special: include_str!(concat!(
                    "output_formatting/",
                    $name,
                    "/debug.special.txt"
                )),
                debug_unlocated: $unlocated,
                #[cfg(feature = "diagnostic")]
                display_json: include_str!(concat!("output_formatting/", $name, "/display.json")),
                #[cfg(feature = "diagnostic")]
                debug_json: include_str!(concat!("output_formatting/", $name, "/debug.json")),
            },
        )
    };
}

#[derive(Debug, thiserror::Error)]
#[error("invalid input")]
struct InvalidInput;

#[derive(Debug, thiserror::Error)]
#[error("invalid record\r\nretry exhausted")]
struct Multiline;

#[test]
fn bare_error() {
    let error: ErrorUnion<(InvalidInput,)> = ErrorUnion::new(InvalidInput);
    assert_outputs!("bare_error", error.report());
    assert_outputs!("bare_error", error.erase().into_adapter().as_ref().report());
}

#[test]
fn multiline_root() {
    let error: ErrorUnion<(Multiline,)> = ErrorUnion::new(Multiline);
    assert_outputs!("multiline_root", error.report());
}

#[cfg(feature = "alloc")]
mod allocated {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("{message}")]
    struct Failure {
        message: &'static str,
        #[source]
        source: Option<Box<Failure>>,
    }

    fn error(message: &'static str) -> ErrorUnion<(std::io::Error,)> {
        ErrorUnion::new(std::io::Error::other(message))
    }

    #[test]
    fn source_chain() {
        let error: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
            message: "startup failed",
            source: Some(Box::new(Failure {
                message: "cannot open configuration",
                source: Some(Box::new(Failure {
                    message: "permission denied",
                    source: None,
                })),
            })),
        });
        assert_outputs!("source_chain", error.report());
        assert_outputs!("source_chain", error.into_adapter().as_ref().report());
    }

    #[test]
    fn multiline_sources() {
        let error: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
            message: "cannot open configuration\nretry exhausted",
            source: Some(Box::new(Failure {
                message: "permission denied\naccess policy rejected",
                source: None,
            })),
        });
        assert_outputs!("multiline_sources", error.report());
    }

    #[test]
    fn empty_vector() {
        let errors = ErrorUnionVec::<(std::io::Error,)>::default();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("empty_vector", error.report());
    }

    #[test]
    fn single_child() {
        let errors = ErrorUnionVec::new(vec![error("invalid input")]);
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>, std::fmt::Error)> =
            errors.into_union();
        assert_outputs!("single_child", error.report());
    }

    #[test]
    fn distinct_children() {
        let errors: ErrorUnionVec<(std::io::Error,)> =
            ["invalid input", "permission denied", "file missing"]
                .into_iter()
                .map(error)
                .collect();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("distinct_children", error.report());
    }

    #[test]
    fn identical_children() {
        let errors: ErrorUnionVec<(std::io::Error,)> =
            (0..4).map(|_| error("invalid input")).collect();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("identical_children", error.report());
    }

    #[test]
    fn grouped_ranges() {
        let errors: ErrorUnionVec<(std::io::Error,)> = [
            "zebra", "zebra", "zebra", "alpha", "zebra", "beta", "beta", "alpha", "alpha", "zebra",
        ]
        .into_iter()
        .map(error)
        .collect();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("grouped_ranges", error.report());
        assert_outputs!("grouped_ranges", error.report());
        assert_outputs!(
            "grouped_ranges",
            error.erase().into_adapter().as_ref().report()
        );
    }

    #[test]
    fn matching_messages_from_different_types() {
        let first: ErrorUnion<(InvalidInput,)> = ErrorUnion::new(InvalidInput);
        let errors = ErrorUnionVec::new(vec![first.erase(), error("invalid input").erase()]);
        let error: ErrorUnion<(ErrorUnionVec<eros::AnyError>,)> = errors.into_union();
        assert_outputs!(
            "matching_messages_from_different_types",
            error.report(),
            Some(include_str!(
                "output_formatting/matching_messages_from_different_types/debug.unlocated.txt"
            ))
        );
    }

    #[test]
    fn grouped_multiline_messages() {
        let errors: ErrorUnionVec<(std::io::Error,)> = (0..3)
            .map(|_| error("invalid record\r\nretry exhausted"))
            .collect();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("grouped_multiline_messages", error.report());
    }

    #[test]
    fn matching_roots_with_different_sources() {
        let errors: ErrorUnionVec<(Failure,)> = ["permission denied", "file missing"]
            .into_iter()
            .map(|message| {
                ErrorUnion::new(Failure {
                    message: "import failed",
                    source: Some(Box::new(Failure {
                        message,
                        source: None,
                    })),
                })
            })
            .collect();
        let error: ErrorUnion<(ErrorUnionVec<(Failure,)>,)> = errors.into_union();
        assert_outputs!("matching_roots_with_different_sources", error.report());
    }

    #[test]
    fn nested_distinct_children() {
        let errors: ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)> = ["alpha", "beta"]
            .into_iter()
            .map(|message| ErrorUnionVec::new(vec![error(message)]).into_union())
            .collect();
        let error: ErrorUnion<(ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)>,)> =
            errors.into_union();
        assert_outputs!("nested_distinct_children", error.report());
    }

    #[test]
    fn deeply_nested_groups() {
        let errors: ErrorUnionVec<(std::io::Error,)> =
            (0..2).map(|_| error("invalid input")).collect();
        let inner: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        let middle: ErrorUnion<(ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)>,)> =
            ErrorUnionVec::new(vec![inner]).into_union();
        let outer = ErrorUnionVec::new(vec![middle]);
        let error: ErrorUnion<(
            ErrorUnionVec<(ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)>,)>,
        )> = outer.into_union();
        assert_outputs!("deeply_nested_groups", error.report());
    }

    #[test]
    fn nested_multiline_sources() {
        let child: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
            message: "invalid record\r\nretry exhausted",
            source: Some(Box::new(Failure {
                message: "permission denied\naccess policy rejected",
                source: None,
            })),
        });
        let inner: ErrorUnion<(ErrorUnionVec<(Failure,)>,)> =
            ErrorUnionVec::new(vec![child]).into_union();
        let outer = ErrorUnionVec::new(vec![inner]);
        let error: ErrorUnion<(ErrorUnionVec<(ErrorUnionVec<(Failure,)>,)>,)> = outer.into_union();
        assert_outputs!("nested_multiline_sources", error.report());
    }

    #[test]
    fn unicode_multiline_child() {
        let errors = ErrorUnionVec::new(vec![error("🦀 invalid café\n检查输入")]);
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("unicode_multiline_child", error.report());
    }

    #[cfg(feature = "context")]
    #[test]
    fn multiline_context() {
        let error = error("invalid input").context("validate record\nrow 17");
        assert_outputs!("multiline_context", error.report());
    }

    #[cfg(feature = "context")]
    #[test]
    fn multiple_context_frames() {
        let mut error = error("invalid input");
        for number in 1..=12 {
            error = error.context(format!("step {number}\ncontinuation"));
        }
        assert_outputs!("multiple_context_frames", error.report());
    }

    #[cfg(feature = "context")]
    #[test]
    fn grouped_distinct_contexts() {
        let errors: ErrorUnionVec<(std::io::Error,)> = ["row 17", "row 29", "row 17"]
            .into_iter()
            .map(|context| error("invalid input").context(context))
            .collect();
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.into_union();
        assert_outputs!("grouped_distinct_contexts", error.report());
    }

    #[cfg(feature = "context")]
    #[test]
    fn nested_parent_contexts() {
        let child = error("invalid input").context("validate row 17");
        let inner: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> =
            ErrorUnionVec::new(vec![child]).into_union();
        let inner = inner.context("import page");
        let outer: ErrorUnion<(ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)>,)> =
            ErrorUnionVec::new(vec![inner]).into_union();
        let outer = outer.context("import file");
        assert_outputs!("nested_parent_contexts", outer.report());
        assert_outputs!(
            "nested_parent_contexts",
            outer.erase().into_adapter().as_ref().report()
        );
    }

    #[cfg(feature = "context")]
    #[test]
    fn format_levels() {
        let error: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
            message: "cannot read configuration",
            source: Some(Box::new(Failure {
                message: "permission denied",
                source: None,
            })),
        });
        let error = error.context("read /etc/app.toml").context("start service");
        assert_outputs!("format_levels", error.report());
        let rich = format!("{error:?}");
        // Removing added location/backtrace fields restores the context level;
        // richer formats must not restyle the existing message or cause list.
        let without_metadata = rich
            .lines()
            .take_while(|line| !line.trim_start().starts_with("backtrace ("))
            .filter(|line| !line.trim_start().starts_with("location:"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(without_metadata, format!("{error:#?}"));
        assert!(format!("{error:#?}").starts_with(&format!("{error}")));
        assert!(format!("{error}").starts_with(&format!("{error:#}")));
    }

    #[cfg(feature = "context")]
    #[test]
    fn parent_and_child_contexts() {
        let first = error("invalid email").context("validate customer 104");
        let second = error("missing field\ncustomer name").context("validate customer 209");
        let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> =
            ErrorUnionVec::new(vec![first, second]).into_union();
        let error = error.context("import customers.csv");
        assert_outputs!("parent_and_child_contexts", error.report());
    }
}
