use core::{
    any::{Any, TypeId},
    mem::{align_of_val, size_of_val},
};
use eros::SendSyncError;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity {
    kind: TypeId,
    heap_address: Option<*const ()>,
}

/// Moving an inline root changes its address. Heap roots must keep theirs;
/// both representations must preserve the concrete type through reshaping.
pub fn identity(error: &dyn SendSyncError) -> Identity {
    let size = size_of_val(error);
    let heap =
        size != 0 && (size > size_of::<usize>() || align_of_val(error) > align_of::<usize>());
    Identity {
        kind: Any::type_id(error),
        heap_address: heap.then_some(error as *const dyn SendSyncError as *const ()),
    }
}
