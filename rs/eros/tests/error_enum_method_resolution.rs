use eros::{ErrorUnion, MsgError, TypeSet};
use std::fmt;

// Method lookup on &mut ErrorUnion can choose this trait before dereferencing
// to ErrorUnion's inherent is_inner method. Generated safety checks must not
// depend on traits in the caller's scope.
#[allow(dead_code)]
trait SpoofTypeCheck {
    fn is_inner<T: 'static>(&self) -> bool {
        true
    }
}

impl<E: TypeSet> SpoofTypeCheck for &mut ErrorUnion<E> {}

#[eros::error_enums(CheckedError)]
type CheckedSet = (MsgError, fmt::Error);

#[test]
fn mutable_enum_dispatch_ignores_downstream_methods() {
    let mut error: ErrorUnion<CheckedSet> = ErrorUnion::new(fmt::Error);
    assert!(matches!(
        CheckedErrorMut::from(&mut error),
        CheckedErrorMut::FmtError(_)
    ));
}

#[test]
fn erased_mutable_enum_dispatch_ignores_downstream_methods() {
    let mut error: ErrorUnion = ErrorUnion::new(fmt::Error);
    assert!(matches!(
        CheckedErrorMut::try_from(&mut error).unwrap(),
        CheckedErrorMut::FmtError(_)
    ));

    let mut unlisted: ErrorUnion = ErrorUnion::new("bad".parse::<u32>().unwrap_err());
    assert!(CheckedErrorMut::try_from(&mut unlisted).is_err());
    assert!(unlisted.is_inner::<std::num::ParseIntError>());
}

mod spoofed_rejection {
    use eros::{ErrorUnion, MsgError, TypeSet};
    use std::fmt;

    #[allow(dead_code)]
    trait SpoofTypeCheck {
        fn is_inner<T: 'static>(&self) -> bool {
            false
        }
    }

    impl<E: TypeSet> SpoofTypeCheck for &mut ErrorUnion<E> {}

    #[eros::error_enums(CheckedError)]
    type CheckedSet = (MsgError, fmt::Error);

    #[test]
    fn mutable_enum_cannot_be_forced_into_the_last_variant() {
        let mut error: ErrorUnion<CheckedSet> = ErrorUnion::new(MsgError::from("first"));
        match CheckedErrorMut::from(&mut error) {
            CheckedErrorMut::MsgError(message) => *message = MsgError::from("updated"),
            CheckedErrorMut::FmtError(_) => panic!("incorrect fallback variant"),
        }
        let mut error: ErrorUnion = error.into();
        assert!(matches!(
            CheckedErrorMut::try_from(&mut error).unwrap(),
            CheckedErrorMut::MsgError(_)
        ));
        assert_eq!(
            error.downcast_inner::<MsgError>().unwrap().as_str(),
            "updated"
        );
    }
}
