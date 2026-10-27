use std::{error::Error, fmt};

use eros::ErrorUnion;

#[derive(Debug)]
struct Empty;

impl fmt::Display for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no failures")
    }
}

impl Error for Empty {}

#[test]
fn leaf_reports_work_without_alloc_and_preserve_formats() {
    let error: ErrorUnion<(Empty,)> = ErrorUnion::new(Empty);
    assert_eq!(format!("{}", error.report()), format!("{error}"));
    assert_eq!(format!("{:#?}", error.report()), format!("{error:#?}"));
    assert!((error.report().inner() as &dyn Error).is::<Empty>());
    error
        .report()
        .visit_children(&mut |_| panic!("no children"));
    #[cfg(feature = "diagnostic")]
    assert!(error.to_debug_json().get("children").is_none());
}

#[cfg(feature = "alloc")]
mod allocated {
    use super::*;
    use eros::{ErrorUnionVec, ReshapeUnion};

    #[derive(Debug, thiserror::Error)]
    #[error("invalid email")]
    struct InvalidEmail;

    #[derive(Debug, thiserror::Error)]
    #[error("missing field\ncustomer name")]
    struct MissingField;

    type ItemError = ErrorUnion<(InvalidEmail, MissingField)>;

    type BatchErrors = ErrorUnionVec<(InvalidEmail, MissingField)>;

    #[inline(never)]
    fn email_failure() -> ItemError {
        let error: ItemError = ErrorUnion::new(InvalidEmail);
        error.context("validate customer 104")
    }

    #[inline(never)]
    fn field_failure() -> ItemError {
        let error: ItemError = ErrorUnion::new(MissingField);
        error.context("validate customer 209")
    }

    fn batch() -> ErrorUnion<(BatchErrors,)> {
        let error: ErrorUnion<(BatchErrors,)> =
            BatchErrors::new(vec![email_failure(), field_failure()]).into_union();
        error.context("import customers.csv")
    }

    #[test]
    fn human_reports_keep_parent_and_child_diagnostics_separate() {
        let error = batch();
        assert_eq!(error.to_string(), "errors (2 total):");
        assert_eq!(format!("{error:#}"), "errors (2 total):");
        let expected = if cfg!(feature = "context") {
            include_str!("output_formatting/parent_and_child_contexts/debug.special.txt")
                .trim_end_matches('\n')
        } else {
            "errors (2 total):\n    groups:\n        - message: invalid email\n          at: [0]\n        - message: |\n              missing field\n              customer name\n          at: [1]"
        };
        assert_eq!(format!("{error:#?}"), expected);
        assert_eq!(format!("{:#?}", error.report()), expected);
        let detailed = format!("{error:?}");
        assert_eq!(detailed.matches("backtrace (").count(), 3);
        assert!(!detailed.ends_with('\n'));

        #[cfg(feature = "location")]
        {
            let parent = error.location();
            assert!(detailed.contains(&parent.to_string()));
            error.report().visit_children(&mut |child| {
                assert!(detailed.contains(&child.location().to_string()));
            });
        }
    }

    #[test]
    fn borrowed_children_keep_identity_order_and_original_metadata() {
        let error = batch();
        let original = error.as_single().iter();
        let mut children = Vec::new();
        // Views can outlive the visitor call; they remain tied to the parent.
        error
            .report()
            .visit_children(&mut |child| children.push(child));
        assert_eq!(children.len(), 2);
        for (child, original) in children.iter().zip(original) {
            // Trait-object vtables can be duplicated across codegen units.
            assert!(core::ptr::addr_eq(child.inner(), original.inner()));
            #[cfg(feature = "backtrace")]
            assert!(core::ptr::eq(child.backtrace(), original.backtrace()));
            #[cfg(feature = "location")]
            assert!(core::ptr::eq(child.location(), original.location()));
            #[cfg(feature = "context")]
            assert!(core::ptr::eq(
                child.contexts().next().unwrap(),
                original.contexts().next().unwrap(),
            ));
            assert_eq!(format!("{child:?}"), format!("{original:?}"));
        }
        assert!((children[0].inner() as &dyn Error).is::<InvalidEmail>());
        assert!((children[1].inner() as &dyn Error).is::<MissingField>());
        #[cfg(feature = "backtrace")]
        if children[0].backtrace().status() == std::backtrace::BacktraceStatus::Captured {
            assert!(
                children[0]
                    .backtrace()
                    .to_string()
                    .contains("email_failure")
            );
            assert!(
                children[1]
                    .backtrace()
                    .to_string()
                    .contains("field_failure")
            );
        }
    }

    #[test]
    fn traversal_survives_reshaping_adapters_and_typed_recovery() {
        let error = batch();
        let expected = format!("{error:?}");
        let error: ErrorUnion<(fmt::Error, BatchErrors)> = error.widen();
        let error: ErrorUnion = error.erase();
        let error = error.narrow::<(BatchErrors,), _>().unwrap();
        let error = error.into_adapter().into_unadapted();
        assert_eq!(format!("{error:?}"), expected);
        let result = Err::<(), _>(error);
        result
            .recover(|batch: ErrorUnion<(BatchErrors,)>| {
                for child in batch.into_single() {
                    Err::<(), _>(child)
                        .recover(|_: ErrorUnion<(InvalidEmail,)>| ())
                        .recover(|_: ErrorUnion<(MissingField,)>| ())
                        .into_value();
                }
            })
            .into_value();
    }

    #[test]
    fn registration_is_explicit_and_reads_live_children() {
        let plain: ErrorUnion<(BatchErrors,)> =
            ErrorUnion::new(BatchErrors::new(vec![email_failure()]));
        assert_eq!(format!("{plain:#?}"), "errors (1 total):");
        let mut error = batch();
        *error.as_single_mut().iter_mut().next().unwrap() = field_failure();
        assert!(
            format!("{error:#?}")
                .contains("- message: |\n              missing field\n              customer name\n          at: [0..2]")
        );
        assert!(!format!("{error:#?}").contains("invalid email"));
        *error.as_single_mut() = BatchErrors::default();
        assert!(!format!("{error:#?}").contains("at:"));
        #[cfg(feature = "diagnostic")]
        assert!(error.to_debug_json().get("children").is_none());

        let replacement = batch().map_single(|_| Empty);
        assert_eq!(replacement.inner().to_string(), "no failures");
        replacement
            .report()
            .visit_children(&mut |_| panic!("stale visitor"));
    }

    #[derive(Debug, thiserror::Error)]
    #[error("import failed")]
    struct ImportFailure {
        #[source]
        source: std::io::Error,
    }

    #[test]
    fn child_reports_preserve_the_native_source_chain() {
        let child: ErrorUnion<(ImportFailure,)> = ErrorUnion::new(ImportFailure {
            source: std::io::Error::other("permission denied"),
        });
        let error: ErrorUnion<(ErrorUnionVec<(ImportFailure,)>,)> =
            ErrorUnionVec::new(vec![child]).into_union();
        assert_eq!(error.to_string(), "errors (1 total):");
        let report = format!("{error:#?}");
        assert!(report.contains(
            "- message: import failed\n          at: [0]\n          caused by:\n              - permission denied"
        ));
        assert_eq!(report.matches("caused by:").count(), 1);
        #[cfg(feature = "diagnostic")]
        {
            let json = error.to_debug_json();
            assert_eq!(json["sources"], serde_json::json!([]));
            assert_eq!(json["children"][0]["root"], "import failed");
            assert_eq!(
                json["children"][0]["sources"],
                serde_json::json!(["permission denied"])
            );
        }
    }

    #[cfg(feature = "diagnostic")]
    #[test]
    fn json_preserves_each_child_report_without_flattening_context_or_backtraces() {
        let first = email_failure();
        let second = field_failure();
        let expected_display = vec![first.to_display_json(), second.to_display_json()];
        let expected_debug = vec![first.to_debug_json(), second.to_debug_json()];
        let error: ErrorUnion<(BatchErrors,)> = BatchErrors::new(vec![first, second]).into_union();
        let error = error.context("import customers.csv");
        assert_eq!(
            error.to_display_json()["children"],
            serde_json::json!(expected_display)
        );
        let debug = error.to_debug_json();
        assert_eq!(debug["children"], serde_json::json!(expected_debug));
        assert_eq!(error.report().to_debug_json(), debug);
        if cfg!(feature = "context") {
            assert_eq!(debug["contexts"].as_array().unwrap().len(), 1);
            assert_eq!(debug["contexts"][0]["message"], "import customers.csv");
            assert_eq!(
                debug["children"][0]["contexts"][0]["message"],
                "validate customer 104"
            );
            assert_eq!(
                debug["children"][1]["contexts"][0]["message"],
                "validate customer 209"
            );
        }
        assert!(debug["children"][0].get("children").is_none());
        let error: ErrorUnion = error.erase();
        assert_eq!(error.to_debug_json(), debug);
    }

    #[test]
    fn child_formatter_errors_propagate_to_the_caller() {
        #[derive(Debug)]
        struct Broken;
        impl fmt::Display for Broken {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Err(fmt::Error)
            }
        }
        impl Error for Broken {}

        let child: ErrorUnion<(Broken,)> = ErrorUnion::new(Broken);
        let batch = ErrorUnionVec::new(vec![child]);
        let mut output = String::new();
        assert!(fmt::write(&mut output, format_args!("{batch:?}")).is_err());
        assert!(fmt::write(&mut output, format_args!("{batch:#?}")).is_err());
        let error: ErrorUnion<(ErrorUnionVec<(Broken,)>,)> = batch.into_union();
        let mut output = String::new();
        assert!(fmt::write(&mut output, format_args!("{error:#?}")).is_err());
    }
}
