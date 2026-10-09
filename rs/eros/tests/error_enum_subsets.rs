#![cfg(feature = "alloc")]

use eros::{ErrorUnion, TypeSet};
use std::fmt;

#[derive(Debug)]
struct Payload<const N: u8>(String);

impl<const N: u8> fmt::Display for Payload<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<const N: u8> std::error::Error for Payload<N> {}

type X = Payload<0>;
type Y = Payload<1>;
type Z = Payload<2>;

#[eros::error_enum(AppError)]
#[eros::error_enum_ref(AppErrorRef)]
#[eros::error_enum_mut(AppErrorMut)]
type App = (X, Y, Z);

fn inspect<const N: u8>(payload: &Payload<N>, expected: u8, original: *const ()) -> *const u8 {
    assert_eq!(N, expected);
    assert_eq!(payload.0, "original");
    assert_eq!(payload as *const Payload<N> as *const (), original);
    payload.0.as_ptr()
}

fn update<const N: u8>(payload: &mut Payload<N>, expected: u8, original: *const ()) {
    assert_eq!(N, expected);
    assert_eq!(payload as *mut Payload<N> as *const (), original);
    payload.0.push_str(" updated");
}

fn inspect_owned<const N: u8>(payload: Payload<N>, expected: u8, storage: *const u8) {
    assert_eq!(N, expected);
    assert_eq!(payload.0, "original updated");
    assert_eq!(payload.0.as_ptr(), storage);
}

fn check_subset<S>(mut union: ErrorUnion<S>, expected: u8)
where
    S: TypeSet,
    AppError: From<ErrorUnion<S>>,
    for<'a> AppErrorRef<'a>: From<&'a ErrorUnion<S>>,
    for<'a> AppErrorMut<'a>: From<&'a mut ErrorUnion<S>>,
{
    union = union.context("retained metadata");
    let original = union.inner() as *const dyn eros::SendSyncError as *const ();
    let report = format!("{union:?}");
    let shared: AppErrorRef<'_> = (&union).into();
    let storage = match shared {
        AppErrorRef::X(payload) => inspect(payload, expected, original),
        AppErrorRef::Y(payload) => inspect(payload, expected, original),
        AppErrorRef::Z(payload) => inspect(payload, expected, original),
    };
    assert_eq!(format!("{union:?}"), report);

    let mutable: AppErrorMut<'_> = (&mut union).into();
    match mutable {
        AppErrorMut::X(payload) => update(payload, expected, original),
        AppErrorMut::Y(payload) => update(payload, expected, original),
        AppErrorMut::Z(payload) => update(payload, expected, original),
    }
    let owned: AppError = union.into();
    match owned {
        AppError::X(payload) => inspect_owned(payload, expected, storage),
        AppError::Y(payload) => inspect_owned(payload, expected, storage),
        AppError::Z(payload) => inspect_owned(payload, expected, storage),
    }
}

macro_rules! subset_case {
    ($name:ident, [$($ty:ty),+], [$($n:literal),+]) => {
        subset_case!($name, ($($ty,)+), [$($n),+]);
    };
    ($name:ident, $source:ty, [$($n:literal),+]) => {
        #[test]
        fn $name() {
            type Source = $source;
            $(
                let mut message = String::with_capacity(32);
                message.push_str("original");
                let union: ErrorUnion<Source> = ErrorUnion::new(Payload::<$n>(message));
                check_subset(union, $n);
            )+
        }
    };
}

subset_case!(x, [X], [0]);
subset_case!(y, [Y], [1]);
subset_case!(z, [Z], [2]);
subset_case!(xy, [X, Y], [0, 1]);
subset_case!(yx, [Y, X], [0, 1]);
subset_case!(xz, [X, Z], [0, 2]);
subset_case!(zx, [Z, X], [0, 2]);
subset_case!(yz, [Y, Z], [1, 2]);
subset_case!(zy, [Z, Y], [1, 2]);
subset_case!(xyz, App, [0, 1, 2]);
subset_case!(xzy, [X, Z, Y], [0, 1, 2]);
subset_case!(yxz, [Y, X, Z], [0, 1, 2]);
subset_case!(yzx, [Y, Z, X], [0, 1, 2]);
subset_case!(zxy, [Z, X, Y], [0, 1, 2]);
subset_case!(zyx, [Z, Y, X], [0, 1, 2]);

#[test]
fn question_mark_converts_subset_errors() {
    fn operation() -> Result<(), AppError> {
        let result: eros::Result<(), (Y, X)> =
            Err(ErrorUnion::new(Payload::<1>("subset failure".into())));
        result?;
        Ok(())
    }
    match operation().unwrap_err() {
        AppError::Y(payload) => assert_eq!(payload.0, "subset failure"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn empty_union_has_all_conversions() {
    fn convert(mut union: ErrorUnion<()>) {
        let _: AppErrorRef<'_> = (&union).into();
        let _: AppErrorMut<'_> = (&mut union).into();
        let _: AppError = union.into();
    }
    // An empty union cannot be constructed, but it is a subset of every enum.
    let _: fn(ErrorUnion<()>) = convert;
}

mod maximum_arity {
    use super::*;

    macro_rules! check_maximum {
        ($($ty:ident: $n:literal),+) => {
            $(type $ty = Payload<$n>;)+

            #[eros::error_enum(LargeError)]

            #[eros::error_enum_ref(LargeErrorRef)]

            #[eros::error_enum_mut(LargeErrorMut)]
            type Large = ($($ty,)+);

            #[test]
            fn every_variant_accepts_singleton_and_full_sources() {
                $(
                    let mut union: ErrorUnion<($ty,)> =
                        ErrorUnion::new(Payload::<$n>("singleton".into()));
                    assert!(matches!(LargeErrorRef::from(&union), LargeErrorRef::$ty(_)));
                    match LargeErrorMut::from(&mut union) {
                        LargeErrorMut::$ty(payload) => payload.0.push_str(" updated"),
                        _ => panic!("wrong mutable variant"),
                    }
                    match LargeError::from(union) {
                        LargeError::$ty(payload) => assert_eq!(payload.0, "singleton updated"),
                        _ => panic!("wrong owned variant"),
                    }

                    let mut union: ErrorUnion<Large> =
                        ErrorUnion::new(Payload::<$n>("full".into()));
                    assert!(matches!(LargeErrorRef::from(&union), LargeErrorRef::$ty(_)));
                    assert!(matches!(LargeErrorMut::from(&mut union), LargeErrorMut::$ty(_)));
                    assert!(matches!(LargeError::from(union), LargeError::$ty(_)));
                )+
            }
        };
    }

    check_maximum!(
        P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8,
        P9: 9, P10: 10, P11: 11, P12: 12, P13: 13, P14: 14, P15: 15, P16: 16,
        P17: 17, P18: 18, P19: 19, P20: 20, P21: 21, P22: 22, P23: 23, P24: 24, P25: 25
    );
}
