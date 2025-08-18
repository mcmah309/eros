use std::{
    borrow::Cow,
    fmt::{self, Display},
};

/// An Error type that can hold a string in either a static or owned form. No unnecessary allocation for static strings compared to `String`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StrError {
    Static(&'static str),
    Owned(String),
}

impl std::error::Error for StrError {}

impl StrError {
    pub fn as_str(&self) -> &str {
        match self {
            StrError::Static(s) => s,
            StrError::Owned(s) => s,
        }
    }
}

impl Display for StrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrError::Static(s) => write!(formatter, "{}", s),
            StrError::Owned(s) => write!(formatter, "{}", s),
        }
    }
}

impl Clone for StrError {
    fn clone(&self) -> Self {
        match self {
            StrError::Static(s) => StrError::Static(s),
            StrError::Owned(s) => StrError::Owned(s.clone()),
        }
    }
}

impl From<&'static str> for StrError {
    fn from(s: &'static str) -> StrError {
        StrError::Static(s)
    }
}

impl From<String> for StrError {
    fn from(s: String) -> StrError {
        StrError::Owned(s)
    }
}

impl From<Cow<'static, str>> for StrError {
    fn from(s: Cow<'static, str>) -> StrError {
        match s {
            Cow::Borrowed(s) => StrError::Static(s),
            Cow::Owned(s) => StrError::Owned(s),
        }
    }
}
