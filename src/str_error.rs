use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
};

/// An Error type that is just a message.
/// It can hold a string in either a static or owned form.
/// No unnecessary allocation for static strings compared to `String`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StrContext {
    Static(&'static str),
    Owned(String),
}

impl std::error::Error for StrContext {}

impl StrContext {
    pub fn as_str(&self) -> &str {
        match self {
            StrContext::Static(s) => s,
            StrContext::Owned(s) => s,
        }
    }
}

impl Debug for StrContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrContext::Static(s) => f.write_str(&s),
            StrContext::Owned(s) => f.write_str(&s),
        }
    }
}

impl Display for StrContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrContext::Static(s) => write!(formatter, "{}", s),
            StrContext::Owned(s) => write!(formatter, "{}", s),
        }
    }
}

impl Clone for StrContext {
    fn clone(&self) -> Self {
        match self {
            StrContext::Static(s) => StrContext::Static(s),
            StrContext::Owned(s) => StrContext::Owned(s.clone()),
        }
    }
}

impl From<&'static str> for StrContext {
    fn from(s: &'static str) -> StrContext {
        StrContext::Static(s)
    }
}

impl From<String> for StrContext {
    fn from(s: String) -> StrContext {
        StrContext::Owned(s)
    }
}

impl From<Cow<'static, str>> for StrContext {
    fn from(s: Cow<'static, str>) -> StrContext {
        match s {
            Cow::Borrowed(s) => StrContext::Static(s),
            Cow::Owned(s) => StrContext::Owned(s),
        }
    }
}
