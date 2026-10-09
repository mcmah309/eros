#![cfg(not(feature = "alloc"))]

// Shared with the freestanding executable: every scenario must work using core.
#[path = "no_alloc/composition.rs"]
mod composition;
