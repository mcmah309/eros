use alloc::{borrow::Cow, string::String};
use core::fmt::{self, Debug, Display};

/// An error containing a message.
///
/// Static messages do not allocate. Equality, ordering, and hashing depend only
/// on the message text, regardless of how it is stored.
/// Use [`Self::from_static`] or [`Self::from_owned`] to choose storage explicitly,
/// or convert from a string with [`From`].
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsgError(Cow<'static, str>);

impl core::error::Error for MsgError {}

impl MsgError {
    /// Borrows static text without allocating.
    #[inline]
    pub const fn from_static(message: &'static str) -> Self {
        Self(Cow::Borrowed(message))
    }

    /// Takes ownership of a string without allocating or copying its contents.
    #[inline]
    pub fn from_owned(message: String) -> Self {
        Self(Cow::Owned(message))
    }

    /// Returns the message text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Display for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&'static str> for MsgError {
    fn from(s: &'static str) -> Self {
        Self::from_static(s)
    }
}

impl From<String> for MsgError {
    fn from(s: String) -> Self {
        Self::from_owned(s)
    }
}

impl From<Cow<'static, str>> for MsgError {
    fn from(s: Cow<'static, str>) -> Self {
        Self(s)
    }
}
