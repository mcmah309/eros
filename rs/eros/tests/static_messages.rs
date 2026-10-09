#[path = "no_alloc/messages.rs"]
mod messages;

#[cfg(not(feature = "alloc"))]
#[test]
fn no_alloc_message_error_uses_one_word() {
    assert_eq!(size_of::<eros::MsgError>(), size_of::<usize>());
}
