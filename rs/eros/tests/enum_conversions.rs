//! Exercise every variant of every supported arity. The conversions use
//! unchecked downcasts, so exercise every named variant at each tuple arity.
use eros::ErrorUnion;
use std::fmt;

#[derive(Debug, PartialEq)]
struct Payload<const N: u8>(String);

impl<const N: u8> fmt::Display for Payload<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error {N}: {}", self.0)
    }
}

impl<const N: u8> std::error::Error for Payload<N> {}

macro_rules! check_arity {
    ($test:ident, $($variant:ident: $n:literal),+ $(,)?) => {
        #[test]
        fn $test() {
            $(type $variant = Payload<$n>;)+
            #[eros::error_enum(Owned)]
            #[eros::error_enum_ref(OwnedRef)]
            #[eros::error_enum_mut(OwnedMut)]
            #[eros::error_enum_kind(Kind)]
            type Set = ($($variant,)+);
            type Shared<'a> = OwnedRef<'a>;
            type Mutable<'a> = OwnedMut<'a>;
            $(
                let original = format!("payload {}", $n);
                let mut error: ErrorUnion<Set> = ErrorUnion::new(Payload::<$n>(original.clone()));
                assert!(matches!(Kind::from(&error), Kind::$variant));
                assert!(matches!(Kind::from(&mut error), Kind::$variant));
                assert_eq!(error.to_string(), format!("error {}: {original}", $n));
                assert_eq!(format!("{error:#?}"), format!("error {}: {original}", $n));
                match OwnedRef::from(&error) {
                    OwnedRef::$variant(payload) => assert_eq!(payload.0, original),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong borrowed variant for {}", $n),
                }
                match OwnedMut::from(&mut error) {
                    OwnedMut::$variant(payload) => payload.0.push_str(" updated"),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong mutable variant for {}", $n),
                }
                assert_eq!(error.downcast_inner_ref::<Payload<$n>>().unwrap().0, format!("{original} updated"));
                match Owned::from(error) {
                    Owned::$variant(payload) => assert_eq!(payload.0, format!("{original} updated")),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong owned variant for {}", $n),
                }

                // Each tuple arity has its own From implementation for erasure.
                let error: ErrorUnion<Set> = ErrorUnion::new(Payload::<$n>(original.clone()));
                let error = error.context("retained metadata");
                let report = format!("{error:?}");
                let adapter = Box::new(error.into_std_error());
                assert!(std::error::Error::source(adapter.as_ref()).is_none());
                let error = ErrorUnion::<Set>::try_from_dyn_error(adapter).unwrap();
                let mut erased: ErrorUnion = error.into();
                assert!(matches!(Kind::try_from(&erased).unwrap(), Kind::$variant));
                assert!(matches!(Kind::try_from(&mut erased).unwrap(), Kind::$variant));
                assert_eq!(format!("{erased:?}"), report);
                let shared: Shared<'_> = (&erased).try_into().unwrap();
                match shared {
                    OwnedRef::$variant(payload) => assert_eq!(payload.0, original),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong fallible borrowed variant for {}", $n),
                }
                let mutable: Mutable<'_> = (&mut erased).try_into().unwrap();
                match mutable {
                    OwnedMut::$variant(payload) => payload.0.push_str(" updated"),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong fallible mutable variant for {}", $n),
                }
                let owned: Owned = erased.try_into().unwrap();
                match owned {
                    Owned::$variant(payload) => assert_eq!(payload.0, format!("{original} updated")),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong fallible owned variant for {}", $n),
                }
                let error: ErrorUnion<Set> = ErrorUnion::new(Payload::<$n>(original.clone()));
                assert!(matches!(Kind::from(error), Kind::$variant));
                let erased: ErrorUnion = ErrorUnion::new(Payload::<$n>(original.clone()));
                assert!(matches!(Kind::try_from(erased).unwrap(), Kind::$variant));
            )+
            let erased: ErrorUnion = ErrorUnion::new(Payload::<26>("unmatched".into()));
            let unmatched = Owned::try_from(erased).err().unwrap();
            assert_eq!(unmatched.downcast_inner::<Payload<26>>().unwrap().0, "unmatched");
        }
    };
}

check_arity!(arity_1, A: 0);
check_arity!(arity_2, A: 0, B: 1);
check_arity!(arity_3, A: 0, B: 1, C: 2);
check_arity!(arity_4, A: 0, B: 1, C: 2, D: 3);
check_arity!(arity_5, A: 0, B: 1, C: 2, D: 3, E: 4);
check_arity!(arity_6, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
check_arity!(arity_7, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
check_arity!(arity_8, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
check_arity!(arity_9, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
check_arity!(arity_10, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
check_arity!(arity_11, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
check_arity!(arity_12, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
check_arity!(arity_13, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);
check_arity!(arity_14, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13);
check_arity!(arity_15, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14);
check_arity!(arity_16, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15);
check_arity!(arity_17, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16);
check_arity!(arity_18, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17);
check_arity!(arity_19, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18);
check_arity!(arity_20, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19);
check_arity!(arity_21, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20);
check_arity!(arity_22, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21);
check_arity!(arity_23, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22);
check_arity!(arity_24, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23);
check_arity!(arity_25, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23, Y: 24);
check_arity!(arity_26, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23, Y: 24, Z: 25);
