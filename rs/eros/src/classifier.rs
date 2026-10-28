use crate::{InnerError, SendSyncError};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A callback or replacement error accepted by [`crate::ErrorUnion::classify`].
///
/// Implemented for callbacks taking [`InnerError`] and, with `alloc`, callbacks
/// taking `Box<dyn SendSyncError>`. This includes tuple struct constructors.
/// Error values also implement this trait and discard the original error when
/// classification is needed. This supports unit structs such as `.classify(MyError)`.
/// `Input` distinguishes these forms and is inferred from the classifier.
/// Closure parameters may need an explicit type, such as `|inner: InnerError|`.
pub trait Classifier<Output, Input> {
    /// Classifies the original owned error, converting its storage if needed.
    fn classify_error(self, error: InnerError) -> Output;
}

impl<E: SendSyncError> Classifier<E, ()> for E {
    fn classify_error(self, error: InnerError) -> E {
        // Keep the replacement owned here if the original destructor panics.
        drop(error);
        self
    }
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
