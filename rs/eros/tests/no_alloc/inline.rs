use core::{
    error::Error,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use eros::{ErrorUnion, IntoAnyUnion, IntoUnion, ReshapeUnion};

#[derive(Debug, PartialEq, Eq)]
struct Code(u32);
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code {}", self.0)
    }
}
impl Error for Code {}

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
struct Padded {
    tag: u8,
    value: u16,
}
impl fmt::Display for Padded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.tag, self.value)
    }
}
impl Error for Padded {}

#[eros::error_enum(Owned)]
#[eros::error_enum_ref(Shared)]
#[eros::error_enum_mut(Mutable)]
type Errors = (Code, Padded, fmt::Error);

#[cfg_attr(test, test)]
pub fn inline_values_survive_mutation_moving_reshaping_and_enum_extraction() {
    let mut code: ErrorUnion<Errors> = Err::<(), _>(Code(17)).union().unwrap_err();
    assert!(matches!(Shared::from(&code), Shared::Code(Code(17))));
    match Mutable::from(&mut code) {
        Mutable::Code(code) => code.0 += 1,
        _ => panic!("wrong variant"),
    }
    let erased = Err::<(), _>(code).any_union().any_union().unwrap_err();
    let erased = erased.downcast_inner::<u64>().unwrap_err();
    let selected = erased.narrow::<(Padded, Code), _>().unwrap();
    assert_eq!(selected.downcast_inner_ref::<Code>().unwrap().0, 18);
    let widened: ErrorUnion<Errors> = selected.widen();
    assert!(matches!(Owned::from(widened), Owned::Code(Code(18))));

    let error: ErrorUnion<Errors> = ErrorUnion::new(Padded { tag: 7, value: 513 });
    let error = error.narrow::<Code, _>().unwrap_err();
    let error = error.narrow::<(Padded,), _>().unwrap();
    let mut error = error.into_std_error().into_union();
    error.as_single_mut().value += 1;
    let mapped = error.map_single(|padded| Code(u32::from(padded.value)));
    assert_eq!(mapped.into_single(), Code(514));
}

#[cfg_attr(test, test)]
pub fn full_word_payload_survives_multiple_moves_and_shared_interior_mutation() {
    #[derive(Debug)]
    struct Counter(AtomicUsize);
    impl fmt::Display for Counter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.load(Ordering::SeqCst))
        }
    }
    impl Error for Counter {}
    assert_eq!(size_of::<Counter>(), size_of::<usize>());
    let error: ErrorUnion<(Counter,)> = ErrorUnion::new(Counter(AtomicUsize::new(usize::MAX - 10)));
    // Both shared borrows permit atomic mutation. The opaque storage must not
    // freeze memory that T deliberately exposes through UnsafeCell.
    let first = error.as_single();
    let second = error.downcast_inner_ref::<Counter>().unwrap();
    first.0.fetch_add(1, Ordering::SeqCst);
    assert_eq!(second.0.load(Ordering::SeqCst), usize::MAX - 9);
    let erased: ErrorUnion = error.into();
    let error = erased.narrow::<(Counter,), _>().unwrap();
    let error: ErrorUnion<(fmt::Error, Counter)> = error.widen();
    let error = error.narrow::<(Counter,), _>().unwrap().into_std_error();
    assert_eq!(
        error.into_union().into_single().0.into_inner(),
        usize::MAX - 9
    );
}

#[cfg_attr(test, test)]
pub fn thin_static_reference_preserves_pointer_provenance_and_source() {
    #[derive(Debug)]
    struct Details {
        message: &'static str,
        code: u32,
    }
    impl fmt::Display for Details {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.message)
        }
    }
    impl Error for Details {}
    static DETAILS: Details = Details {
        message: "static message",
        code: 91,
    };
    #[derive(Debug)]
    struct StaticError(&'static Details);
    impl fmt::Display for StaticError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }
    impl Error for StaticError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(self.0)
        }
    }
    let error: ErrorUnion<(StaticError,)> = StaticError(&DETAILS).into();
    let erased: ErrorUnion = error.into_std_error().into_union().into();
    let error = erased.narrow::<(StaticError,), _>().unwrap();
    assert!(core::ptr::eq(error.as_single().0, &DETAILS));
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<Details>()
            .unwrap()
            .code,
        91
    );
    let error = error.into_single();
    assert_eq!(error.0.message, "static message");
    assert!(core::ptr::eq(error.0, &DETAILS));
}

#[cfg_attr(test, test)]
pub fn inline_non_copy_owners_drop_once_after_recovery_mapping_and_extraction() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct Tracked(&'static AtomicUsize);
    impl fmt::Display for Tracked {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("tracked")
        }
    }
    impl Error for Tracked {}
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let before = DROPS.load(Ordering::SeqCst);
    let error: ErrorUnion<(Tracked, Code)> = ErrorUnion::new(Tracked(&DROPS));
    let error = error.narrow::<Code, _>().unwrap_err();
    let error: ErrorUnion = error.into();
    let error = error.downcast_inner::<Code>().unwrap_err();
    let owner = error.downcast_inner::<Tracked>().unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), before);
    drop(owner);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);

    let input: eros::Result<u32, (Tracked, Code)> = Err(Tracked(&DROPS)).union();
    let output: eros::Result<u32, (Code,)> = input.try_recover(|error: ErrorUnion<(Tracked,)>| {
        Err(error.map_single(|owner| {
            assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
            drop(owner);
            Code(19)
        }))
    });
    let value = output
        .recover::<Code, _>(|error| error.into_single().0)
        .into_value();
    assert_eq!(value, 19);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);
    let error: ErrorUnion<(Tracked,)> = ErrorUnion::new(Tracked(&DROPS));
    drop(error.into_std_error());
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 3);
}

#[cfg_attr(test, allow(dead_code))]
pub fn run_checks() {
    inline_values_survive_mutation_moving_reshaping_and_enum_extraction();
    full_word_payload_survives_multiple_moves_and_shared_interior_mutation();
    thin_static_reference_preserves_pointer_provenance_and_source();
    inline_non_copy_owners_drop_once_after_recovery_mapping_and_extraction();
}
