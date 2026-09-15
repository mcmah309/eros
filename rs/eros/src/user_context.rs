#[cfg(test)]
mod tests {
    use crate::{ErrorUnion, SendSyncError};
    #[cfg(not(feature = "std"))]
    use std::prelude::v1::*;

    #[derive(Debug)]
    struct SystemDiskError;
    impl std::fmt::Display for SystemDiskError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Critial IO failure at block 0x7FA3")
        }
    }
    impl std::error::Error for SystemDiskError {}

    #[derive(Debug)]
    struct InvalidPasswordError;
    impl std::fmt::Display for InvalidPasswordError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Your password must be at least 8 characters long.")
        }
    }
    impl std::error::Error for InvalidPasswordError {}

    fn create_user_error_message(error: &dyn SendSyncError) -> Option<String> {
        #[allow(clippy::manual_map)]
        if let Some(user_error) = error.as_any().downcast_ref::<InvalidPasswordError>() {
            Some(user_error.to_string())
        } else {
            None
        }
    }

    #[test]
    fn test_use_case() {
        let system_err = SystemDiskError;
        let union_a: ErrorUnion<(SystemDiskError,)> = ErrorUnion::new(system_err);
        let union_a = union_a.context("Normal context");
        let union_a = union_a.user_context("User context");

        let user_context = union_a
            .contexts()
            .filter(|frame| frame.is_user_facing())
            .collect::<Vec<_>>();

        assert_eq!(user_context.len(), 1);
        assert_eq!(user_context[0].to_string(), "User context");

        let user_error_message = create_user_error_message(union_a.inner());
        assert_eq!(user_error_message, None);

        let user_err = InvalidPasswordError;
        let union_b: ErrorUnion<(InvalidPasswordError,)> = ErrorUnion::new(user_err);
        let union_b = union_b.context("Normal context");
        let union_b = union_b.user_context("User context");

        let user_context = union_b
            .contexts()
            .filter(|frame| frame.is_user_facing())
            .collect::<Vec<_>>();

        assert_eq!(user_context.len(), 1);
        assert_eq!(user_context[0].to_string(), "User context");

        let user_error_message = create_user_error_message(union_b.inner());
        assert_eq!(
            user_error_message.as_deref(),
            Some("Your password must be at least 8 characters long.")
        );
    }
}
