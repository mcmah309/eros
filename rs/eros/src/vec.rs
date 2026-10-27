use alloc::vec::{self, Vec as AllocVec};
use core::{fmt, slice};

use crate::{AnyError, ErrorUnion, TypeSet, storage::ErrorUnionInner, type_set::Contains};

/// A vector of independent failures, each retaining its typed error and diagnostics.
///
/// Stores [`ErrorUnion<E>`] values. Display prints `errors (N total):`;
/// Debug groups identical visible reports with zero-based positions and
/// half-open ranges. Alternate Debug omits locations and backtraces.
/// Grouping preserves the stored errors and their insertion order.
///
/// Requires `alloc`. An empty batch is allowed while accumulating failures;
/// check [`is_empty`](Self::is_empty) before returning it as an error.
///
/// Use [`into_union`](Self::into_union) to register recursive human and JSON
/// reporting. Ordinary `.union()` and [`ErrorUnion::new`] do not register children.
///
/// ```
/// use eros::ErrorUnion;
///
/// let child: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
/// let batch: eros::Vec<(core::fmt::Error,)> =
///     eros::Vec::new(vec![child.context("format item 3")]);
/// let error: ErrorUnion<(eros::Vec<(core::fmt::Error,)>,)> = batch.into_union();
/// let error = error.context("process batch");
/// println!("{error:?}");
///
/// // Recover the typed children for individual handling.
/// for child in error.into_single() {
///     let _: core::fmt::Error = child.into_single();
/// }
/// ```
pub struct Vec<E: TypeSet = AnyError> {
    errors: AllocVec<ErrorUnion<E>>,
}

impl<E: TypeSet> Vec<E> {
    /// Takes ownership of existing unions without reallocating or recapturing diagnostics.
    pub fn new(errors: AllocVec<ErrorUnion<E>>) -> Self {
        Self { errors }
    }

    /// Returns the number of failures.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns whether the batch has no failures.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Appends an existing union without changing its diagnostics.
    pub fn push(&mut self, error: ErrorUnion<E>) {
        self.errors.push(error);
    }

    /// Borrows the typed child unions in insertion order.
    pub fn iter(&self) -> slice::Iter<'_, ErrorUnion<E>> {
        self.errors.iter()
    }

    /// Mutably borrows the typed child unions in insertion order.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, ErrorUnion<E>> {
        self.errors.iter_mut()
    }

    /// Returns the original vector and its typed unions without reallocating.
    pub fn into_errors(self) -> AllocVec<ErrorUnion<E>> {
        self.errors
    }

    /// Wraps this batch and enables recursive child reporting.
    ///
    /// The destination set can contain other error types alongside this batch.
    /// Context added to the returned union describes the batch; child contexts
    /// and captured backtraces remain attached to their original unions.
    #[cfg_attr(feature = "location", track_caller)]
    pub fn into_union<OutSet, Index>(self) -> ErrorUnion<OutSet>
    where
        OutSet: TypeSet,
        OutSet::Variants: Contains<Self, Index>,
    {
        ErrorUnion {
            inner: ErrorUnionInner::new_vec(self),
            _pd: core::marker::PhantomData,
        }
    }
}

impl<E: TypeSet> Default for Vec<E> {
    fn default() -> Self {
        Self::new(AllocVec::new())
    }
}

impl<E: TypeSet> From<AllocVec<ErrorUnion<E>>> for Vec<E> {
    fn from(errors: AllocVec<ErrorUnion<E>>) -> Self {
        Self::new(errors)
    }
}

impl<E: TypeSet> FromIterator<ErrorUnion<E>> for Vec<E> {
    fn from_iter<T: IntoIterator<Item = ErrorUnion<E>>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<E: TypeSet> Extend<ErrorUnion<E>> for Vec<E> {
    fn extend<T: IntoIterator<Item = ErrorUnion<E>>>(&mut self, iter: T) {
        self.errors.extend(iter);
    }
}

impl<E: TypeSet> IntoIterator for Vec<E> {
    type Item = ErrorUnion<E>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: TypeSet> IntoIterator for &'a Vec<E> {
    type Item = &'a ErrorUnion<E>;
    type IntoIter = slice::Iter<'a, ErrorUnion<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, E: TypeSet> IntoIterator for &'a mut Vec<E> {
    type Item = &'a mut ErrorUnion<E>;
    type IntoIter = slice::IterMut<'a, ErrorUnion<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<E: TypeSet> fmt::Display for Vec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "errors ({} total):", self.len())
    }
}

impl<E: TypeSet> fmt::Debug for Vec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::formatting::debug_vector(f, |visitor| {
            for child in self {
                visitor(child.report());
            }
        })
    }
}

impl<E: TypeSet> core::error::Error for Vec<E> {}
