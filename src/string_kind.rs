use std::{borrow::Cow, fmt::{self, Display}};

#[derive(Debug)]
pub enum StringKind {
    Static(&'static str),
    Owned(String),
}

impl StringKind {
    pub fn as_str(&self) -> &str {
        match self {
            StringKind::Static(s) => s,
            StringKind::Owned(s) => s,
        }
    }
}

impl Display for StringKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringKind::Static(s) => write!(formatter, "{}", s),
            StringKind::Owned(s) => write!(formatter, "{}", s),
        }
    }
}

impl Clone for StringKind {
    fn clone(&self) -> Self {
        match self {
            StringKind::Static(s) => StringKind::Static(s),
            StringKind::Owned(s) => StringKind::Owned(s.clone()),
        }
    }
}

impl From<&'static str> for StringKind {
    fn from(s: &'static str) -> StringKind {
        StringKind::Static(s)
    }
}

impl From<String> for StringKind {
    fn from(s: String) -> StringKind {
        StringKind::Owned(s)
    }
}

impl From<Cow<'static, str>> for StringKind {
    fn from(s: Cow<'static, str>) -> StringKind {
        match s {
            Cow::Borrowed(s) => StringKind::Static(s),
            Cow::Owned(s) => StringKind::Owned(s),
        }
    }
}