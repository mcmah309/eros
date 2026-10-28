use core::any::{Any, TypeId};
use eros::SendSyncError;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity {
    kind: TypeId,
    heap_address: Option<*const ()>,
}

/// Heap roots retain their address through reshaping. ZSTs carry no bytes.
pub fn identity(error: &dyn SendSyncError) -> Identity {
    Identity {
        kind: Any::type_id(error),
        heap_address: cfg!(feature = "alloc")
            .then_some(error as *const dyn SendSyncError as *const ()),
    }
}
