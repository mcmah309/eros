use alloc::vec::{self, Vec as AllocVec};
use core::{fmt, slice};

use crate::{AnyError, ErrorUnion, TypeSet, storage::ErrorUnionInner, type_set::Contains};

// TODO: When specialization is stabilized, revisit replacing this wrapper with
// Vec<ErrorUnion<E>> using a local union-member trait, while retaining blanket
// support for ordinary errors and recursive child reporting.
/// A vector of independent failures, each retaining its typed error and diagnostics.
///
/// Stores [`ErrorUnion<E>`] values. Display prints `errors (N total):`;
/// Debug groups identical visible reports with zero-based positions and
/// half-open ranges. Alternate Debug omits locations, backtraces, and span traces.
/// Grouping preserves the stored errors and their insertion order.
/// Debug and JSON reporting expand at most 10 child reports across the whole
/// tree, in depth-first insertion order, and indicate omitted children. The
/// limit applies before grouping and does not affect iteration or recovery.
///
/// Requires `alloc`. An empty batch is allowed while accumulating failures;
/// check [`is_empty`](Self::is_empty) before returning it as an error.
/// Convert to or from a `Vec<ErrorUnion<E>>` with [`From`] without reallocating.
///
/// Use [`union`](Self::union) on the batch to register recursive human and JSON
/// reporting. Converting a result with [`IntoUnion::union`](crate::IntoUnion::union)
/// or wrapping the batch with [`ErrorUnion::new`] does not register children.
///
/// With the `json` feature, the returned union's
/// `to_display_json()` and `to_debug_json()` methods include registered children
/// recursively in a `children` array, omitted when empty. Each child has its own
/// root/source messages and, for Debug JSON, diagnostic metadata. The same
/// 10-child limit applies; truncated batches include `children_omitted`, the
/// number of immediate children skipped.
///
/// ```
/// use eros::{ErrorUnion, ErrorUnionVec};
///
/// let child: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
/// let batch: ErrorUnionVec<(core::fmt::Error,)> =
///     ErrorUnionVec::new(vec![child.context("format item 3")]);
/// let error: ErrorUnion<(ErrorUnionVec<(core::fmt::Error,)>,)> = batch.union();
/// let error = error.context("process batch");
/// println!("{error:?}");
///
/// // Recover the typed children for individual handling.
/// for child in error.into_single() {
///     let _: core::fmt::Error = child.into_single();
/// }
/// ```
pub struct ErrorUnionVec<E: TypeSet = AnyError> {
    errors: AllocVec<ErrorUnion<E>>,
}

impl<E: TypeSet> ErrorUnionVec<E> {
    /// Takes ownership of existing unions without reallocating or recapturing diagnostics.
    #[inline]
    pub const fn new(errors: AllocVec<ErrorUnion<E>>) -> Self {
        Self { errors }
    }

    /// Returns the number of failures.
    #[inline]
    pub const fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns whether the batch has no failures.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Appends an existing union without changing its diagnostics.
    #[inline]
    pub fn push(&mut self, error: ErrorUnion<E>) {
        self.errors.push(error);
    }

    /// Borrows the typed child unions in insertion order.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, ErrorUnion<E>> {
        self.errors.iter()
    }

    /// Mutably borrows the typed child unions in insertion order.
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, ErrorUnion<E>> {
        self.errors.iter_mut()
    }

    /// Returns the original vector, preserving its allocation and child diagnostics.
    #[inline]
    pub fn into_inner(self) -> AllocVec<ErrorUnion<E>> {
        self.errors
    }

    /// Wraps this batch and enables recursive child reporting.
    ///
    /// The destination set can contain other error types alongside this batch.
    /// Context added to the returned union describes the batch; child contexts
    /// and captured traces remain attached to their original unions.
    #[cfg_attr(feature = "error_location", track_caller)]
    pub fn union<OutSet, Index>(self) -> ErrorUnion<OutSet>
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

impl<E: TypeSet> Default for ErrorUnionVec<E> {
    #[inline]
    fn default() -> Self {
        Self::new(AllocVec::new())
    }
}

impl<E: TypeSet> From<AllocVec<ErrorUnion<E>>> for ErrorUnionVec<E> {
    #[inline]
    fn from(errors: AllocVec<ErrorUnion<E>>) -> Self {
        Self::new(errors)
    }
}

impl<E: TypeSet> From<ErrorUnionVec<E>> for AllocVec<ErrorUnion<E>> {
    #[inline]
    fn from(errors: ErrorUnionVec<E>) -> Self {
        errors.into_inner()
    }
}

impl<E: TypeSet> FromIterator<ErrorUnion<E>> for ErrorUnionVec<E> {
    fn from_iter<T: IntoIterator<Item = ErrorUnion<E>>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<E: TypeSet> Extend<ErrorUnion<E>> for ErrorUnionVec<E> {
    fn extend<T: IntoIterator<Item = ErrorUnion<E>>>(&mut self, iter: T) {
        self.errors.extend(iter);
    }
}

impl<E: TypeSet> IntoIterator for ErrorUnionVec<E> {
    type Item = ErrorUnion<E>;
    type IntoIter = vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: TypeSet> IntoIterator for &'a ErrorUnionVec<E> {
    type Item = &'a ErrorUnion<E>;
    type IntoIter = slice::Iter<'a, ErrorUnion<E>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, E: TypeSet> IntoIterator for &'a mut ErrorUnionVec<E> {
    type Item = &'a mut ErrorUnion<E>;
    type IntoIter = slice::IterMut<'a, ErrorUnion<E>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<E: TypeSet> fmt::Display for ErrorUnionVec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "errors ({} total):", self.len())
    }
}

impl<E: TypeSet> fmt::Debug for ErrorUnionVec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::formatting::debug_vector(f, |visitor| {
            for child in self {
                visitor(child.report());
            }
        })
    }
}

impl<E: TypeSet> core::error::Error for ErrorUnionVec<E> {}
