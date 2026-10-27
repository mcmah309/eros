#![cfg(feature = "alloc")]

use eros::{ErrorUnion, ErrorUnionVec, IntoUnion, ReshapeUnion};

#[derive(Debug, thiserror::Error)]
#[error("odd number")]
struct Odd;

#[derive(Debug, thiserror::Error)]
#[error("calculation overflowed")]
struct Overflow;

type ItemErrors = (Odd, Overflow);
type Batch = ErrorUnionVec<ItemErrors>;

fn double_even(value: u8) -> eros::Result<u8, ItemErrors> {
    let result = if value % 2 != 0 {
        Err(Odd).union()
    } else {
        value.checked_mul(2).ok_or(Overflow).union()
    };
    result.map_err(|error: ErrorUnion<ItemErrors>| error.context(format!("process {value}")))
}

fn check_nums(nums: impl IntoIterator<Item = u8>) -> eros::Result<Vec<u8>, (Batch,)> {
    let mut values = Vec::new();
    let mut errors = Batch::default();
    for value in nums {
        match double_even(value) {
            Ok(value) => values.push(value),
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(values)
    } else {
        Err(errors.into_union())
    }
}

#[test]
fn batch_collection_continues_after_failures_and_preserves_typed_children() {
    let mut visited = Vec::new();
    let result = check_nums([0, 1, 128, 3, 2].into_iter().inspect(|n| visited.push(*n)));
    assert_eq!(visited, [0, 1, 128, 3, 2]);
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "errors (3 total):");
    let children = error.into_single().into_errors();
    assert_eq!(children.len(), 3);
    assert!(children[0].is_inner::<Odd>());
    assert!(children[1].is_inner::<Overflow>());
    assert!(children[2].is_inner::<Odd>());
    for child in children {
        Err::<(), _>(child)
            .recover(|_: ErrorUnion<(Odd,)>| ())
            .recover(|_: ErrorUnion<(Overflow,)>| ())
            .into_value();
    }
    assert_eq!(check_nums([0, 2, 126]).unwrap(), [0, 4, 252]);
    assert!(check_nums([]).unwrap().is_empty());
    assert_eq!(check_nums(0..5).unwrap_err().as_single().len(), 2);
}

#[test]
fn wrapping_preserves_child_reports_and_registers_human_and_json_traversal() {
    let first = double_even(1).unwrap_err();
    let second = double_even(128).unwrap_err();
    let expected_reports = [format!("{first:?}"), format!("{second:?}")];
    #[cfg(feature = "diagnostic")]
    let expected_json = [first.to_debug_json(), second.to_debug_json()];
    let errors = vec![first, second];
    let original_allocation = errors.as_ptr();
    let batch = Batch::new(errors);
    assert_eq!(batch.iter().as_slice().as_ptr(), original_allocation);
    let error: ErrorUnion<(Batch, std::fmt::Error)> = batch.into_union();
    let error = error.context("process batch");
    let report = format!("{error:#?}");
    assert!(report.contains("- message: odd number\n          at: [0]"));
    assert!(report.contains("- message: calculation overflowed\n          at: [1]"));
    #[cfg(feature = "context")]
    assert!(report.contains("process batch"));
    let mut children = Vec::new();
    error
        .report()
        .visit_children(&mut |child| children.push(child));
    assert_eq!(format!("{:?}", children[0]), expected_reports[0]);
    assert_eq!(format!("{:?}", children[1]), expected_reports[1]);
    #[cfg(feature = "diagnostic")]
    assert_eq!(
        error.to_debug_json()["children"],
        serde_json::json!(expected_json)
    );
    let error: ErrorUnion = error.erase();
    assert_eq!(format!("{error:#?}"), report);
    let recovered = error.narrow::<(Batch,), _>().unwrap().into_single();
    let errors = recovered.into_errors();
    assert_eq!(errors.as_ptr(), original_allocation);
}

#[test]
fn batches_support_collect_extend_mutable_iteration_and_owned_iteration() {
    let mut batch: Batch = [double_even(1).unwrap_err()].into_iter().collect();
    batch.extend([double_even(128).unwrap_err()]);
    assert_eq!(batch.len(), 2);
    for child in &mut batch {
        *child = double_even(3).unwrap_err();
    }
    assert!(batch.iter().all(ErrorUnion::is_inner::<Odd>));
    assert_eq!((&batch).into_iter().count(), 2);
    batch
        .iter_mut()
        .for_each(|child| *child = double_even(254).unwrap_err());
    assert!(batch.into_iter().all(|child| child.is_inner::<Overflow>()));
    let batch: Batch = Vec::new().into();
    assert!(batch.is_empty());
    assert_eq!(batch.to_string(), "errors (0 total):");
    let error: ErrorUnion<(Batch,)> = batch.into_union();
    assert_eq!(format!("{error:#?}"), "errors (0 total):");
}

#[test]
fn nested_convenience_batches_report_each_level() {
    let child_batch = check_nums(0..3).unwrap_err();
    let batch = ErrorUnionVec::new(vec![child_batch]);
    let error: ErrorUnion<(ErrorUnionVec<(Batch,)>,)> = batch.into_union();
    let report = format!("{error:#?}");
    assert!(report.starts_with(
        "errors (1 total):\n    groups:\n        - message: errors (1 total)\n          at: [0]"
    ));
    assert!(report.contains("              - message: odd number\n                at: [0]"));
}

#[derive(Debug, thiserror::Error)]
#[error("invalid record\r\nretry exhausted")]
struct Multiline {
    #[source]
    source: std::io::Error,
}

#[test]
fn standalone_and_wrapped_vectors_share_nested_multiline_indentation() {
    let child: ErrorUnion<(Multiline,)> = ErrorUnion::new(Multiline {
        source: std::io::Error::other("permission denied\npolicy rejected"),
    });
    let child = child.context("validate record\nrow 17");
    let batch = ErrorUnionVec::new(vec![child]);
    let mut expected = "errors (1 total):\n    groups:\n        - message: |\n              invalid record\n              retry exhausted\n          at: [0]\n          caused by:\n              - permission denied\n                policy rejected".to_owned();
    if cfg!(feature = "context") {
        expected.push_str(
            "\n          context:\n              - validate record\n                row 17",
        );
    }
    assert_eq!(format!("{batch:#?}"), expected);

    let standalone = format!("{batch:?}");
    // The vector itself has no captured metadata; its child has one report.
    assert_eq!(standalone.matches("backtrace (").count(), 1);
    let wrapped: ErrorUnion<(ErrorUnionVec<(Multiline,)>, std::fmt::Error)> = batch.into_union();
    assert_eq!(format!("{wrapped:#?}"), expected);
    let wrapped_detailed = format!("{wrapped:?}");
    assert_eq!(wrapped_detailed.matches("backtrace (").count(), 2);
    assert!(wrapped_detailed.starts_with(&standalone));

    let outer = ErrorUnionVec::new(vec![wrapped]);
    let mut nested_expected = "errors (1 total):\n    groups:\n        - message: errors (1 total)\n          at: [0]\n          groups:\n              - message: |\n                    invalid record\n                    retry exhausted\n                at: [0]\n                caused by:\n                    - permission denied\n                      policy rejected".to_owned();
    if cfg!(feature = "context") {
        nested_expected.push_str(
            "\n                context:\n                    - validate record\n                      row 17",
        );
    }
    assert_eq!(format!("{outer:#?}"), nested_expected);
    assert_eq!(format!("{outer:?}").matches("backtrace (").count(), 2);
    let outer: ErrorUnion<(ErrorUnionVec<(ErrorUnionVec<(Multiline,)>, std::fmt::Error)>,)> =
        outer.into_union();
    assert_eq!(format!("{outer:#?}"), nested_expected);
    assert_eq!(format!("{outer:?}").matches("backtrace (").count(), 3);
}

#[test]
fn empty_vectors_have_the_same_summary_in_every_format() {
    let batch = ErrorUnionVec::<ItemErrors>::default();
    assert_eq!(format!("{batch}"), "errors (0 total):");
    assert_eq!(format!("{batch:#}"), "errors (0 total):");
    assert_eq!(format!("{batch:?}"), "errors (0 total):");
    assert_eq!(format!("{batch:#?}"), "errors (0 total):");
}

#[test]
fn matching_messages_group_nonconsecutive_ranges_in_first_occurrence_order() {
    let batch: ErrorUnionVec<(std::io::Error,)> = [
        "zebra", "zebra", "zebra", "alpha", "zebra", "beta", "beta", "alpha", "alpha", "zebra",
    ]
    .into_iter()
    .map(|message| ErrorUnion::new(std::io::Error::other(message)))
    .collect();
    let expected =
        include_str!("output_formatting/grouped_ranges/debug.special.txt").trim_end_matches('\n');
    assert_eq!(format!("{batch:#?}"), expected);
    let error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = batch.into_union();
    assert_eq!(format!("{error:#?}"), expected);
    assert_eq!(error.to_string(), "errors (10 total):");
    let mut children = Vec::new();
    error
        .report()
        .visit_children(&mut |child| children.push(child));
    assert_eq!(children.len(), 10);
    assert_eq!(children[3].to_string(), "alpha");
    assert_eq!(children[4].to_string(), "zebra");
    #[cfg(feature = "diagnostic")]
    {
        let json = error.to_debug_json();
        assert_eq!(json["children"].as_array().unwrap().len(), 10);
        assert_eq!(json["children"][9]["root"], "zebra");
    }
    assert_eq!(error.into_single().into_errors().len(), 10);
}

#[test]
fn matching_messages_group_different_concrete_error_types() {
    let first: ErrorUnion<(Odd,)> = ErrorUnion::new(Odd);
    let second: ErrorUnion<(std::io::Error,)> =
        ErrorUnion::new(std::io::Error::other("odd number"));
    let batch = ErrorUnionVec::new(vec![first.erase(), second.erase()]);
    assert_eq!(
        format!("{batch:#?}"),
        "errors (2 total):\n    groups:\n        - message: odd number\n          at: [0..2]"
    );
    let mut children = batch.into_iter();
    assert!(children.next().unwrap().is_inner::<Odd>());
    assert!(children.next().unwrap().is_inner::<std::io::Error>());
}

#[cfg(feature = "context")]
#[test]
fn equal_messages_with_distinct_contexts_form_separate_groups() {
    let batch: ErrorUnionVec<(std::io::Error,)> = ["row 17", "row 29", "row 17"]
        .into_iter()
        .map(|context| {
            let child: ErrorUnion<(std::io::Error,)> =
                ErrorUnion::new(std::io::Error::other("invalid input"));
            child.context(context)
        })
        .collect();
    assert_eq!(
        format!("{batch:#?}"),
        include_str!("output_formatting/grouped_distinct_contexts/debug.special.txt")
            .trim_end_matches('\n')
    );
}

#[cfg(feature = "location")]
#[test]
fn equal_messages_retain_distinct_locations_in_detailed_reports() {
    let first: ErrorUnion<(std::io::Error,)> =
        ErrorUnion::new(std::io::Error::other("invalid input"));
    let second: ErrorUnion<(std::io::Error,)> =
        ErrorUnion::new(std::io::Error::other("invalid input"));
    let first_location = first.location().to_string();
    let second_location = second.location().to_string();
    let batch = ErrorUnionVec::new(vec![first, second]);
    assert_eq!(
        format!("{batch:#?}"),
        "errors (2 total):\n    groups:\n        - message: invalid input\n          at: [0..2]"
    );
    let detailed = format!("{batch:?}");
    assert_eq!(detailed.matches("- message: invalid input\n").count(), 2);
    assert!(detailed.contains("          at: [0]"));
    assert!(detailed.contains("          at: [1]"));
    assert!(detailed.contains(&first_location));
    assert!(detailed.contains(&second_location));
    assert_eq!(detailed.matches("backtrace (").count(), 2);
}

#[test]
fn equal_roots_with_different_sources_remain_separate_groups() {
    let batch: ErrorUnionVec<(Multiline,)> = ["permission denied", "file missing"]
        .into_iter()
        .map(|message| {
            ErrorUnion::new(Multiline {
                source: std::io::Error::other(message),
            })
        })
        .collect();
    let report = format!("{batch:#?}");
    assert!(!report.contains("at: [0..2]"));
    assert!(report.contains(
        "- message: |\n              invalid record\n              retry exhausted\n          at: [0]\n          caused by:\n              - permission denied"
    ));
    assert!(
        report.contains(
            "- message: |\n              invalid record\n              retry exhausted\n          at: [1]\n          caused by:\n              - file missing"
        )
    );
}

#[test]
fn multiline_roots_cannot_impersonate_source_fields_when_grouping() {
    #[derive(Debug, thiserror::Error)]
    #[error("{message}")]
    struct Failure {
        message: &'static str,
        #[source]
        source: Option<std::io::Error>,
    }

    for reverse in [false, true] {
        for nested in [false, true] {
            let literal: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
                message: "foo\n    caused by:\n        - bar",
                source: None,
            });
            let sourced: ErrorUnion<(Failure,)> = ErrorUnion::new(Failure {
                message: "foo",
                source: Some(std::io::Error::other("bar")),
            });
            // The standalone text is ambiguous, but the underlying fields differ.
            assert_eq!(format!("{literal:#?}"), format!("{sourced:#?}"));
            let mut children = vec![literal.erase(), sourced.erase()];
            if reverse {
                children.reverse();
            }
            if nested {
                children = children
                    .into_iter()
                    .map(|child| ErrorUnionVec::new(vec![child]).into_union())
                    .collect();
            }
            let batch = ErrorUnionVec::new(children);
            let report = format!("{batch:#?}");
            assert_eq!(
                report.matches("- message:").count(),
                if nested { 4 } else { 2 }
            );
            assert!(!report.contains("at: [0..2]"), "{report}");
            let indent = " ".repeat(if nested { 16 } else { 10 });
            assert!(
                report.contains(&format!("\n{indent}caused by:\n{indent}    - bar")),
                "the actual source chain must remain a separate field: {report}"
            );
            let error: ErrorUnion = batch.into_union();
            assert_eq!(format!("{error:#?}"), report);
            assert_eq!(
                format!("{error:?}").matches("- message:").count(),
                if nested { 4 } else { 2 }
            );
        }
    }
}

#[test]
fn equal_batch_summaries_preserve_different_nested_children() {
    let batch: ErrorUnionVec<(ErrorUnionVec<(std::io::Error,)>,)> = ["alpha", "beta"]
        .into_iter()
        .map(|message| {
            let child: ErrorUnion<(std::io::Error,)> =
                ErrorUnion::new(std::io::Error::other(message));
            ErrorUnionVec::new(vec![child]).into_union()
        })
        .collect();
    assert_eq!(
        format!("{batch:#?}"),
        include_str!("output_formatting/nested_distinct_children/debug.special.txt")
            .trim_end_matches('\n')
    );
}
