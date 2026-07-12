use core::{fmt, ops::Deref};
use std::sync::Arc;

use crate::{
    AnyError, ErrorUnion, TypeSet,
    type_set::{DebugFold, DisplayFold},
};

pub type Result<T, E = AnyError> = core::result::Result<T, ArcErrorUnion<E>>;

#[derive(Clone)]
pub struct ArcErrorUnion<T = AnyError>(std::sync::Arc<ErrorUnion<T>>)
where
    T: TypeSet;

impl<T> Deref for ArcErrorUnion<T>
where
    T: TypeSet,
{
    type Target = ErrorUnion<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Debug for ArcErrorUnion<T>
where
    T: TypeSet,
    T::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl<T> fmt::Display for ArcErrorUnion<T>
where
    T: TypeSet,
    T::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl<T> ArcErrorUnion<T>
where
    T: TypeSet,
{
    pub fn new(error_union: ErrorUnion<T>) -> Self {
        ArcErrorUnion(std::sync::Arc::new(error_union))
    }
}

impl<T> PartialEq for ArcErrorUnion<T>
where
    T: TypeSet,
    T::Variants: PartialEq + DebugFold,
{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T, E> From<E> for ArcErrorUnion<T>
where
    T: TypeSet,
    E: Into<ErrorUnion<T>>,
{
    fn from(value: E) -> Self {
        ArcErrorUnion::new(value.into())
    }
}
