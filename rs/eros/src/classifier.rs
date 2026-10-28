use crate::InnerError;
#[cfg(feature = "alloc")]
use crate::SendSyncError;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A callback accepted by [`crate::ErrorUnion::classify`].
///
/// Implemented for callbacks taking [`InnerError`] and, with `alloc`, callbacks
/// taking `Box<dyn SendSyncError>`. This includes tuple struct constructors.
/// `Input` distinguishes the two forms and is inferred from the callback.
/// Closure parameters may need an explicit type, such as `|inner: InnerError|`.
pub trait Classifier<Output, Input> {
    /// Classifies the original owned error, converting its storage if needed.
    fn classify_error(self, error: InnerError) -> Output;
}

impl<F, Output> Classifier<Output, InnerError> for F
where
    F: FnOnce(InnerError) -> Output,
{
    fn classify_error(self, error: InnerError) -> Output {
        self(error)
    }
}

#[cfg(feature = "alloc")]
impl<F, Output> Classifier<Output, Box<dyn SendSyncError>> for F
where
    F: FnOnce(Box<dyn SendSyncError>) -> Output,
{
    fn classify_error(self, error: InnerError) -> Output {
        self(error.into_boxed())
    }
}
