use std::fmt;

use crate::string_kind::StringKind;

/// A generic error for when you wish to propagate information about an issue, but the caller would not care about
/// the type of issue.
#[derive(Debug)]
pub struct GenericError {
    message: StringKind,
}

impl GenericError {
    #[allow(private_bounds)]
    pub fn new(message: impl Into<StringKind>) -> Self {
        GenericError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        match &self.message {
            StringKind::Static(s) => s,
            StringKind::Owned(s) => s,
        }
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl std::error::Error for GenericError {}