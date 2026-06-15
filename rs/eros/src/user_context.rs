use std::marker::PhantomData;

use crate::{ErrorUnion, SendSyncError, StrError, type_set::TypeSet};

impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    pub fn get_user_context(&self) -> Vec<StrError> {
        let mut user_context = Vec::new();
        for context in self.inner.context.iter().filter(|e| e.is_user_facing) {
            user_context.push(context.context.clone());
        }
        user_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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


    fn get_user_error_message(error: &dyn SendSyncError) -> Option<String> {
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

        let user_context = union_a.get_user_context();

        assert_eq!(user_context.len(), 1);
        assert_eq!(user_context[0].to_string(), "User context");

        let user_error_message = get_user_error_message(union_a.inner_ref());
        assert_eq!(user_error_message, None);

        let user_err = InvalidPasswordError;
        let union_b: ErrorUnion<(InvalidPasswordError,)> = ErrorUnion::new(user_err);
        let union_b = union_b.context("Normal context");
        let union_b = union_b.user_context("User context");

        let user_context = union_b.get_user_context();

        assert_eq!(user_context.len(), 1);
        assert_eq!(user_context[0].to_string(), "User context");

        let user_error_message = get_user_error_message(union_b.inner_ref());
        assert_eq!(
            user_error_message.as_deref(),
            Some("Your password must be at least 8 characters long.")
        );
    }
}
