use core::any::{Any, TypeId};
use eros::SendSyncError;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity {
    kind: TypeId,
    heap_address: Option<*const ()>,
}

/// Heap roots retain their address through reshaping. Inline roots can move.
pub fn identity(error: &dyn SendSyncError) -> Identity {
    let diagnostics = cfg!(any(
        feature = "context",
        feature = "backtrace",
        feature = "error_location",
        feature = "error_span_trace"
    ));
    let heap = cfg!(feature = "alloc")
        && (diagnostics
            || (size_of_val(error) != 0
                && (size_of_val(error) > size_of::<usize>()
                    || core::mem::align_of_val(error) > align_of::<usize>())));
    Identity {
        kind: Any::type_id(error),
        heap_address: heap.then_some(error as *const dyn SendSyncError as *const ()),
    }
}
