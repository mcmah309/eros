//! Exercise every variant of every supported arity. The conversions contain
//! unchecked downcasts, so testing only E2 or the last E26 variant is insufficient.
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
    ($test:ident, $enum:ident, $($variant:ident: $n:literal),+ $(,)?) => {
        #[test]
        fn $test() {
            type Set = ($(Payload<$n>,)+);
            $(
                let original = format!("payload {}", $n);
                let mut error: ErrorUnion<Set> = ErrorUnion::new(Payload::<$n>(original.clone()));
                assert_eq!(error.to_string(), format!("error {}: {original}", $n));
                assert_eq!(format!("{error:#?}"), format!("error {}: {original}", $n));
                match error.as_enum() {
                    eros::$enum::$variant(payload) => assert_eq!(payload.0, original),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong borrowed variant for {}", $n),
                }
                match error.as_mut_enum() {
                    eros::$enum::$variant(payload) => payload.0.push_str(" updated"),
                    #[allow(unreachable_patterns)]
                    _ => panic!("wrong mutable variant for {}", $n),
                }
                assert_eq!(error.downcast_inner_ref::<Payload<$n>>().unwrap().0, format!("{original} updated"));
                match error.into_enum() {
                    eros::$enum::$variant(payload) => assert_eq!(payload.0, format!("{original} updated")),
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
                let erased: ErrorUnion = error.into();
                assert_eq!(format!("{erased:?}"), report);
                assert_eq!(erased.downcast_inner::<Payload<$n>>().unwrap().0, original);
            )+
        }
    };
}

#[test]
fn e1_can_be_constructed_directly_from_its_payload() {
    let eros::E1::A(error) = eros::E1::from(Payload::<0>("direct".into()));
    assert_eq!(error.0, "direct");
}

check_arity!(arity_1, E1, A: 0);
check_arity!(arity_2, E2, A: 0, B: 1);
check_arity!(arity_3, E3, A: 0, B: 1, C: 2);
check_arity!(arity_4, E4, A: 0, B: 1, C: 2, D: 3);
check_arity!(arity_5, E5, A: 0, B: 1, C: 2, D: 3, E: 4);
check_arity!(arity_6, E6, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
check_arity!(arity_7, E7, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
check_arity!(arity_8, E8, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
check_arity!(arity_9, E9, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
check_arity!(arity_10, E10, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
check_arity!(arity_11, E11, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
check_arity!(arity_12, E12, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
check_arity!(arity_13, E13, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);
check_arity!(arity_14, E14, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13);
check_arity!(arity_15, E15, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14);
check_arity!(arity_16, E16, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15);
check_arity!(arity_17, E17, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16);
check_arity!(arity_18, E18, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17);
check_arity!(arity_19, E19, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18);
check_arity!(arity_20, E20, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19);
check_arity!(arity_21, E21, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20);
check_arity!(arity_22, E22, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21);
check_arity!(arity_23, E23, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22);
check_arity!(arity_24, E24, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23);
check_arity!(arity_25, E25, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23, Y: 24);
check_arity!(arity_26, E26, A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23, Y: 24, Z: 25);
