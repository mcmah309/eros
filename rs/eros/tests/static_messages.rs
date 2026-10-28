#[path = "no_alloc/messages.rs"]
mod messages;

#[cfg(not(feature = "alloc"))]
#[test]
fn no_alloc_literal_has_no_payload() {
    let error = eros::error!("message");
    assert_eq!(size_of_val(error.inner()), 0);
    assert_eq!(align_of_val(error.inner()), 1);
}
