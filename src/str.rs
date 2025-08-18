use std::{borrow::Cow, fmt::{self, Display}};

/// A type that can hold a string in either a static or owned form. No unnecessary allocation for static strings compared to `String`.
#[derive(Debug)]
pub enum Str {
    Static(&'static str),
    Owned(String),
}

impl Str {
    pub fn as_str(&self) -> &str {
        match self {
            Str::Static(s) => s,
            Str::Owned(s) => s,
        }
    }
}

impl Display for Str {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Str::Static(s) => write!(formatter, "{}", s),
            Str::Owned(s) => write!(formatter, "{}", s),
        }
    }
}

impl Clone for Str {
    fn clone(&self) -> Self {
        match self {
            Str::Static(s) => Str::Static(s),
            Str::Owned(s) => Str::Owned(s.clone()),
        }
    }
}

impl From<&'static str> for Str {
    fn from(s: &'static str) -> Str {
        Str::Static(s)
    }
}

impl From<String> for Str {
    fn from(s: String) -> Str {
        Str::Owned(s)
    }
}

impl From<Cow<'static, str>> for Str {
    fn from(s: Cow<'static, str>) -> Str {
        match s {
            Cow::Borrowed(s) => Str::Static(s),
            Cow::Owned(s) => Str::Owned(s),
        }
    }
}