#![cfg(feature = "alloc")]

use eros::{ErrorUnion, ErrorUnionVec};

// Omitted children must never be formatted, even for grouping or JSON.
#[derive(Debug, thiserror::Error)]
struct Failure {
    index: usize,
    may_format: bool,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.may_format, "omitted error was formatted");
        write!(f, "failure {}", self.index)
    }
}

type Batch = ErrorUnionVec<(Failure,)>;

fn batch(count: usize, visible: usize) -> Batch {
    (0..count)
        .map(|index| {
            ErrorUnion::new(Failure {
                index,
                may_format: index < visible,
            })
        })
        .collect()
}

#[test]
fn large_batches_limit_formatting_and_preserve_all_errors() {
    for count in [0, 9, 10, 11, 10_000] {
        let batch = batch(count, 10);
        for text in [format!("{batch:?}"), format!("{batch:#?}")] {
            assert!(text.starts_with(&format!("errors ({count} total):")));
            assert_eq!(text.matches("- error: failure ").count(), count.min(10));
            if count > 10 {
                assert!(text.ends_with(&format!(
                    "- ... {} errors omitted (report limit reached)",
                    count - 10
                )));
                assert!(text.contains("at: [9]"));
            } else {
                assert!(!text.contains("omitted"));
            }
        }
        let error: ErrorUnion<(Batch,)> = batch.into_union();
        assert_eq!(error.to_string(), format!("errors ({count} total):"));
        for text in [format!("{error:?}"), format!("{error:#?}")] {
            assert_eq!(text.matches("- error: failure ").count(), count.min(10));
            assert_eq!(text.contains("omitted"), count > 10);
        }
        let mut visited = 0;
        error.report().visit_children(&mut |_| visited += 1);
        assert_eq!(visited, count);
        let recovered = error.into_single().into_errors();
        assert_eq!(recovered.len(), count);
        if let Some(last) = recovered.into_iter().last() {
            assert_eq!(last.into_single().index, count - 1);
        }
    }
}

#[test]
fn more_than_ten_errors_show_the_full_count_and_omission_notice() {
    let error: ErrorUnion<(Batch,)> = batch(12, 10).into_union();
    let output = format!("{error:#?}");
    println!("{output}");
    assert_eq!(
        output,
        r#"error:
    errors (12 total):
        - error: failure 0
          at: [0]
        - error: failure 1
          at: [1]
        - error: failure 2
          at: [2]
        - error: failure 3
          at: [3]
        - error: failure 4
          at: [4]
        - error: failure 5
          at: [5]
        - error: failure 6
          at: [6]
        - error: failure 7
          at: [7]
        - error: failure 8
          at: [8]
        - error: failure 9
          at: [9]
        - ... 2 errors omitted (report limit reached)"#
    );
}

#[test]
fn report_limit_applies_before_grouping_duplicates() {
    let batch: Batch = (0..10_000)
        .map(|_| {
            ErrorUnion::new(Failure {
                index: 0,
                may_format: true,
            })
        })
        .collect();
    assert_eq!(
        format!("{batch:#?}"),
        "errors (10000 total):\n    - error: failure 0\n      at: [0..10]\n    - ... 9990 errors omitted (report limit reached)"
    );
}

fn nested_batch() -> ErrorUnionVec<(Batch,)> {
    // The first batch consumes 7 slots (itself plus 6 children), leaving
    // 3 for the second batch and its first 2 children. The third is omitted.
    [(6, 6), (6, 2), (6, 0)]
        .into_iter()
        .map(|(count, visible)| batch(count, visible).into_union())
        .collect()
}

#[test]
fn nested_batches_share_one_report_limit() {
    let batch = nested_batch();
    let error: ErrorUnion<(ErrorUnionVec<(Batch,)>,)> = nested_batch().into_union();
    for text in [
        format!("{batch:?}"),
        format!("{batch:#?}"),
        format!("{error:?}"),
        format!("{error:#?}"),
    ] {
        assert_eq!(text.matches("- error:").count(), 10);
        assert_eq!(text.matches("errors (6 total):").count(), 2);
        assert!(text.contains("... 4 errors omitted (report limit reached)"));
        assert!(text.contains("... 1 errors omitted (report limit reached)"));
    }
}

#[test]
fn deeply_nested_batches_stop_expanding_at_the_report_limit() {
    let mut error: ErrorUnion = ErrorUnion::new(Failure {
        index: 0,
        may_format: false,
    });
    for _ in 0..150 {
        error = ErrorUnionVec::new(vec![error]).into_union();
    }
    let text = format!("{error:#?}");
    assert_eq!(text.matches("- error:").count(), 10);
    assert_eq!(text.matches("errors (1 total):").count(), 11);
    assert!(text.contains("... 1 errors omitted (report limit reached)"));
    #[cfg(feature = "json")]
    for mut value in [error.to_display_json(), error.to_debug_json()] {
        for _ in 0..10 {
            assert!(value.get("children_omitted").is_none());
            let children = value["children"].as_array_mut().unwrap();
            assert_eq!(children.len(), 1);
            value = children.pop().unwrap();
        }
        assert!(value.get("children").is_none());
        assert_eq!(value["children_omitted"], 1);
    }
}

#[cfg(feature = "json")]
#[test]
fn json_reports_limit_children_and_mark_omissions() {
    for count in [0, 9, 10, 11, 10_000] {
        let error: ErrorUnion<(Batch,)> = batch(count, 10).into_union();
        for value in [
            error.to_display_json(),
            error.to_debug_json(),
            error.report().to_display_json(),
            error.report().to_debug_json(),
        ] {
            assert_eq!(value["root"], format!("errors ({count} total):"));
            if count == 0 {
                assert!(value.get("children").is_none());
            } else {
                assert_eq!(value["children"].as_array().unwrap().len(), count.min(10));
                assert_eq!(
                    value["children"][count.min(10) - 1]["root"],
                    format!("failure {}", count.min(10) - 1)
                );
            }
            if count > 10 {
                assert_eq!(value["children_omitted"], count - 10);
            } else {
                assert!(value.get("children_omitted").is_none());
            }
        }
    }
}

#[cfg(feature = "json")]
#[test]
fn nested_json_reports_share_one_report_limit() {
    let error: ErrorUnion<(ErrorUnionVec<(Batch,)>,)> = nested_batch().into_union();
    for value in [error.to_display_json(), error.to_debug_json()] {
        assert_eq!(value["children_omitted"], 1);
        let children = value["children"].as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0]["children"].as_array().unwrap().len(), 6);
        assert!(children[0].get("children_omitted").is_none());
        assert_eq!(children[1]["children"].as_array().unwrap().len(), 2);
        assert_eq!(children[1]["children_omitted"], 4);
    }
}
