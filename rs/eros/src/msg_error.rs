#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};
use core::fmt::{self, Debug, Display};

/// An error containing a message.
///
/// Static messages do not allocate. Equality, ordering, and hashing depend only
/// on the message text, regardless of how it is stored.
/// [`Self::from_static_ref`] borrows a static string descriptor and works without
/// `alloc`. With `alloc`, `from_static`, `from_owned`, and string conversions are
/// also available.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsgError(
    #[cfg(feature = "alloc")] Cow<'static, str>,
    #[cfg(not(feature = "alloc"))] &'static &'static str,
);

impl core::error::Error for MsgError {}

impl MsgError {
    /// Borrows a static string descriptor without allocating.
    ///
    /// Without `alloc`, this stores one thin pointer, fitting Eros's inline word.
    /// The descriptor must also be static; a local variable holding a static
    /// string is insufficient. Literal macros create the descriptor automatically.
    ///
    /// ```
    /// static MESSAGE: &str = "connection failed";
    /// const ERROR: eros::MsgError = eros::MsgError::from_static_ref(&MESSAGE);
    /// assert_eq!(ERROR.as_str(), "connection failed");
    /// ```
    #[inline]
    pub const fn from_static_ref(message: &'static &'static str) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::from_static(*message)
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self(message)
        }
    }

    /// Borrows static text without allocating.
    #[cfg(feature = "alloc")]
    #[inline]
    pub const fn from_static(message: &'static str) -> Self {
        Self(Cow::Borrowed(message))
    }

    /// Takes ownership of a string without allocating or copying its contents.
    #[cfg(feature = "alloc")]
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

#[cfg(feature = "alloc")]
impl From<&'static str> for MsgError {
    fn from(s: &'static str) -> Self {
        Self::from_static(s)
    }
}

#[cfg(feature = "alloc")]
impl From<String> for MsgError {
    fn from(s: String) -> Self {
        Self::from_owned(s)
    }
}

#[cfg(feature = "alloc")]
impl From<Cow<'static, str>> for MsgError {
    fn from(s: Cow<'static, str>) -> Self {
        Self(s)
    }
}
