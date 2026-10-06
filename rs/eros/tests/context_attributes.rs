use std::{cell::RefCell, fmt};

struct CloneProbe<'a> {
    value: u32,
    events: &'a RefCell<Vec<&'static str>>,
}

impl Clone for CloneProbe<'_> {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push("clone");
        Self {
            value: self.value + 1,
            events: self.events,
        }
    }
}

impl fmt::Display for CloneProbe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[eros::context("first={} again={}", value.clone(), value.clone())]
fn lazy_clones(value: &CloneProbe<'_>, fail: bool) -> eros::Result<()> {
    assert_eq!(value.value, 7);
    value.events.borrow_mut().push("body");
    eros::ensure!(!fail, "failed");
    Ok(())
}

#[eros::eager_context("first={} again={}", value.clone(), value.clone())]
fn eager_clones(value: CloneProbe<'_>, fail: bool) -> eros::Result<()> {
    assert_eq!(value.value, 7);
    value.events.borrow_mut().push("body");
    eros::ensure!(!fail, "failed");
    Ok(())
}

#[test]
fn context_evaluates_clones_as_written_only_on_error() {
    let events = RefCell::new(Vec::new());
    let value = CloneProbe {
        value: 7,
        events: &events,
    };
    lazy_clones(&value, false).unwrap();
    assert_eq!(*events.borrow(), ["body"]);
    events.borrow_mut().clear();

    let error = lazy_clones(&value, true).unwrap_err();
    #[cfg(feature = "context")]
    {
        assert_eq!(*events.borrow(), ["body", "clone", "clone"]);
        assert_eq!(
            error.contexts().next().unwrap().to_string(),
            "first=8 again=8"
        );
    }
    #[cfg(not(feature = "context"))]
    {
        assert_eq!(*events.borrow(), ["body"]);
        assert_eq!(error.inner().to_string(), "failed");
    }
}

#[test]
fn eager_context_evaluates_clones_before_body_on_success_and_error() {
    let events = RefCell::new(Vec::new());
    for fail in [false, true] {
        events.borrow_mut().clear();
        let value = CloneProbe {
            value: 7,
            events: &events,
        };
        let result = eager_clones(value, fail);
        assert_eq!(*events.borrow(), ["clone", "clone", "body"]);
        if fail {
            let error = result.unwrap_err();
            assert_eq!(error.inner().to_string(), "failed");
            #[cfg(feature = "context")]
            assert_eq!(
                error.contexts().next().unwrap().to_string(),
                "first=8 again=8"
            );
        } else {
            result.unwrap();
        }
    }
}

#[eros::eager_context("input={__eros_context}")]
fn implicit_capture(__eros_context: String) -> eros::Result<String> {
    Ok(__eros_context)
}

#[test]
fn eager_context_local_does_not_shadow_parameters() {
    assert_eq!(implicit_capture("original".to_owned()).unwrap(), "original");
}
