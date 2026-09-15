use alloc::{borrow::Cow, string::String};
use core::fmt::{self, Debug, Display};

/// An Error type that is just a message.
/// It can hold a string in either a static or owned form.
/// No unnecessary allocation for static strings compared to `String`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgError {
    Static(&'static str),
    Owned(String),
}

impl core::error::Error for MsgError {}

impl MsgError {
    pub fn as_str(&self) -> &str {
        match self {
            MsgError::Static(s) => s,
            MsgError::Owned(s) => s,
        }
    }
}

impl Debug for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MsgError::Static(s) => f.write_str(s),
            MsgError::Owned(s) => f.write_str(s),
        }
    }
}

impl Display for MsgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MsgError::Static(s) => write!(formatter, "{}", s),
            MsgError::Owned(s) => write!(formatter, "{}", s),
        }
    }
}

impl Clone for MsgError {
    fn clone(&self) -> Self {
        match self {
            MsgError::Static(s) => MsgError::Static(s),
            MsgError::Owned(s) => MsgError::Owned(s.clone()),
        }
    }
}

impl From<&'static str> for MsgError {
    fn from(s: &'static str) -> MsgError {
        MsgError::Static(s)
    }
}

impl From<String> for MsgError {
    fn from(s: String) -> MsgError {
        MsgError::Owned(s)
    }
}

impl From<Cow<'static, str>> for MsgError {
    fn from(s: Cow<'static, str>) -> MsgError {
        match s {
            Cow::Borrowed(s) => MsgError::Static(s),
            Cow::Owned(s) => MsgError::Owned(s),
        }
    }
}
