use super::{
    E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15, E16, E17, E18, E19, E20, E21,
    E22, E23, E24, E25, E26, ErrorUnion,
};

/* ------------------------- Enum conversions ----------------------- */

#[rustfmt::skip]
impl<A> From<ErrorUnion<(A,)>> for E1<A>
where
    A: 'static,
{
    fn from(union_of: ErrorUnion<(A,)>) -> Self {
        E1::A(unsafe { union_of.inner.downcast_error_unchecked() })
    }
}

#[rustfmt::skip]
impl<'a, A> From<&'a ErrorUnion<(A,)>> for E1<&'a A>
where
    A: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A,)>) -> Self {
        E1::A(union_of.inner.downcast_error_ref().unwrap())
    }
}

#[rustfmt::skip]
impl<'a, A> From<&'a mut ErrorUnion<(A,)>> for E1<&'a mut A>
where
    A: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A,)>) -> Self {
        E1::A(union_of.inner.downcast_error_mut().unwrap())
    }
}

#[rustfmt::skip]
impl<A, B> From<ErrorUnion<(A, B)>>
    for E2<A, B>
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: ErrorUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E2::B(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B> From<&'a ErrorUnion<(A, B)>>
    for E2<&'a A, &'a B>
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E2::B(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B> From<&'a mut ErrorUnion<(A, B)>>
    for E2<
        &'a mut A,
        &'a mut B,
    >
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E2::B(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C> From<ErrorUnion<(A, B, C)>>
    for E3<A, B, C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E3::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E3::C(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C> From<&'a ErrorUnion<(A, B, C)>>
    for E3<&'a A, &'a B, &'a C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E3::B(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E3::C(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C> From<&'a mut ErrorUnion<(A, B, C)>>
    for E3<
        &'a mut A,
        &'a mut B,
        &'a mut C,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E3::B(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E3::C(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D> From<ErrorUnion<(A, B, C, D)>>
    for E4<A, B, C, D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E4::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E4::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E4::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E4::D(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D> From<&'a ErrorUnion<(A, B, C, D)>>
    for E4<&'a A, &'a B, &'a C, &'a D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E4::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E4::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E4::C(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E4::D(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D> From<&'a mut ErrorUnion<(A, B, C, D)>>
    for E4<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E4::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E4::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E4::C(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E4::D(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E> From<ErrorUnion<(A, B, C, D, E)>>
    for E5<A, B, C, D, E>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E5::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E5::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E5::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E5::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E5::E(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E> From<&'a ErrorUnion<(A, B, C, D, E)>>
    for E5<&'a A, &'a B, &'a C, &'a D, &'a E>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E5::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E5::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E5::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E5::D(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E5::E(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E> From<&'a mut ErrorUnion<(A, B, C, D, E)>>
    for E5<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E5::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E5::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E5::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E5::D(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E5::E(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F> From<ErrorUnion<(A, B, C, D, E, F)>>
    for E6<A, B, C, D, E, F>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E6::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E6::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E6::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E6::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E6::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E6::F(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F> From<&'a ErrorUnion<(A, B, C, D, E, F)>>
    for E6<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E6::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E6::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E6::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E6::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E6::E(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E6::F(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F> From<&'a mut ErrorUnion<(A, B, C, D, E, F)>>
    for E6<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E6::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E6::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E6::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E6::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E6::E(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E6::F(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G> From<ErrorUnion<(A, B, C, D, E, F, G)>>
    for E7<A, B, C, D, E, F, G>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E7::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E7::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E7::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E7::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E7::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E7::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E7::G(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G> From<&'a ErrorUnion<(A, B, C, D, E, F, G)>>
    for E7<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E7::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E7::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E7::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E7::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E7::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E7::F(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E7::G(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G)>>
    for E7<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E7::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E7::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E7::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E7::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E7::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E7::F(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E7::G(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H> From<ErrorUnion<(A, B, C, D, E, F, G, H)>>
    for E8<A, B, C, D, E, F, G, H>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E8::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E8::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E8::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E8::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E8::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E8::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E8::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E8::H(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H)>>
    for E8<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E8::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E8::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E8::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E8::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E8::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E8::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E8::G(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E8::H(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H)>>
    for E8<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E8::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E8::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E8::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E8::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E8::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E8::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E8::G(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E8::H(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I> From<ErrorUnion<(A, B, C, D, E, F, G, H, I)>>
    for E9<A, B, C, D, E, F, G, H, I>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E9::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E9::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E9::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E9::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E9::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E9::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E9::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E9::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E9::I(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I)>>
    for E9<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E9::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E9::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E9::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E9::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E9::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E9::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E9::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E9::H(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E9::I(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I)>>
    for E9<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E9::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E9::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E9::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E9::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E9::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E9::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E9::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E9::H(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E9::I(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>>
    for E10<A, B, C, D, E, F, G, H, I, J>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E10::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E10::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E10::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E10::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E10::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E10::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E10::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E10::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E10::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E10::J(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>>
    for E10<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E10::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E10::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E10::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E10::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E10::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E10::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E10::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E10::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E10::I(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E10::J(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>>
    for E10<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E10::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E10::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E10::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E10::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E10::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E10::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E10::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E10::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E10::I(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E10::J(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>>
    for E11<A, B, C, D, E, F, G, H, I, J, K>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E11::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E11::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E11::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E11::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E11::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E11::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E11::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E11::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E11::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E11::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E11::K(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>>
    for E11<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E11::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E11::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E11::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E11::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E11::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E11::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E11::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E11::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E11::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E11::J(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E11::K(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>>
    for E11<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E11::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E11::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E11::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E11::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E11::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E11::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E11::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E11::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E11::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E11::J(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E11::K(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>>
    for E12<A, B, C, D, E, F, G, H, I, J, K, L>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E12::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E12::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E12::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E12::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E12::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E12::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E12::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E12::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E12::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E12::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E12::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E12::L(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>>
    for E12<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E12::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E12::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E12::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E12::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E12::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E12::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E12::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E12::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E12::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E12::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E12::K(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E12::L(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>>
    for E12<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E12::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E12::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E12::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E12::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E12::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E12::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E12::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E12::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E12::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E12::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E12::K(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E12::L(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>>
    for E13<A, B, C, D, E, F, G, H, I, J, K, L, M>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E13::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E13::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E13::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E13::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E13::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E13::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E13::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E13::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E13::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E13::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E13::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E13::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E13::M(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>>
    for E13<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E13::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E13::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E13::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E13::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E13::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E13::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E13::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E13::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E13::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E13::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E13::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E13::L(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E13::M(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>>
    for E13<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E13::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E13::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E13::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E13::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E13::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E13::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E13::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E13::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E13::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E13::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E13::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E13::L(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E13::M(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>>
    for E14<A, B, C, D, E, F, G, H, I, J, K, L, M, N>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E14::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E14::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E14::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E14::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E14::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E14::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E14::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E14::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E14::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E14::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E14::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E14::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E14::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E14::N(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>>
    for E14<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E14::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E14::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E14::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E14::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E14::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E14::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E14::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E14::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E14::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E14::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E14::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E14::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E14::M(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E14::N(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>>
    for E14<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E14::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E14::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E14::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E14::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E14::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E14::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E14::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E14::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E14::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E14::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E14::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E14::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E14::M(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E14::N(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>>
    for E15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E15::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E15::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E15::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E15::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E15::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E15::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E15::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E15::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E15::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E15::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E15::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E15::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E15::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E15::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E15::O(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>>
    for E15<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E15::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E15::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E15::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E15::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E15::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E15::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E15::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E15::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E15::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E15::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E15::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E15::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E15::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E15::N(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E15::O(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>>
    for E15<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E15::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E15::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E15::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E15::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E15::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E15::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E15::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E15::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E15::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E15::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E15::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E15::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E15::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E15::N(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E15::O(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>>
    for E16<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E16::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E16::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E16::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E16::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E16::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E16::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E16::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E16::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E16::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E16::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E16::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E16::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E16::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E16::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E16::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E16::P(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>>
    for E16<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E16::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E16::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E16::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E16::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E16::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E16::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E16::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E16::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E16::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E16::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E16::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E16::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E16::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E16::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E16::O(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E16::P(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>>
    for E16<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E16::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E16::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E16::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E16::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E16::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E16::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E16::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E16::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E16::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E16::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E16::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E16::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E16::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E16::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E16::O(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E16::P(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>>
    for E17<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E17::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E17::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E17::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E17::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E17::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E17::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E17::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E17::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E17::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E17::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E17::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E17::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E17::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E17::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E17::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E17::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E17::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>>
    for E17<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E17::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E17::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E17::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E17::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E17::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E17::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E17::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E17::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E17::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E17::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E17::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E17::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E17::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E17::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E17::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E17::P(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E17::Q(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>>
    for E17<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E17::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E17::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E17::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E17::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E17::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E17::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E17::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E17::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E17::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E17::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E17::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E17::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E17::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E17::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E17::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E17::P(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E17::Q(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>>
    for E18<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E18::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E18::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E18::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E18::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E18::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E18::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E18::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E18::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E18::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E18::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E18::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E18::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E18::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E18::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E18::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E18::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E18::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E18::R(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>>
    for E18<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E18::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E18::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E18::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E18::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E18::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E18::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E18::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E18::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E18::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E18::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E18::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E18::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E18::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E18::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E18::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E18::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E18::Q(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E18::R(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>>
    for E18<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E18::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E18::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E18::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E18::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E18::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E18::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E18::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E18::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E18::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E18::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E18::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E18::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E18::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E18::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E18::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E18::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E18::Q(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E18::R(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>>
    for E19<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E19::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E19::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E19::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E19::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E19::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E19::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E19::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E19::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E19::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E19::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E19::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E19::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E19::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E19::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E19::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E19::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E19::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E19::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E19::S(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>>
    for E19<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E19::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E19::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E19::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E19::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E19::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E19::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E19::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E19::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E19::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E19::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E19::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E19::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E19::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E19::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E19::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E19::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E19::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E19::R(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E19::S(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>>
    for E19<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E19::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E19::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E19::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E19::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E19::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E19::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E19::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E19::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E19::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E19::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E19::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E19::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E19::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E19::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E19::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E19::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E19::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E19::R(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E19::S(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>>
    for E20<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E20::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E20::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E20::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E20::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E20::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E20::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E20::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E20::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E20::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E20::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E20::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E20::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E20::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E20::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E20::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E20::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E20::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E20::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E20::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E20::T(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>>
    for E20<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E20::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E20::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E20::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E20::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E20::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E20::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E20::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E20::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E20::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E20::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E20::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E20::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E20::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E20::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E20::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E20::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E20::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E20::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E20::S(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E20::T(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>>
    for E20<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E20::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E20::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E20::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E20::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E20::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E20::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E20::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E20::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E20::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E20::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E20::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E20::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E20::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E20::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E20::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E20::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E20::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E20::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E20::S(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E20::T(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>>
    for E21<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E21::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E21::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E21::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E21::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E21::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E21::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E21::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E21::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E21::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E21::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E21::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E21::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E21::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E21::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E21::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E21::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E21::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E21::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E21::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E21::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E21::U(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>>
    for E21<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E21::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E21::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E21::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E21::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E21::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E21::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E21::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E21::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E21::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E21::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E21::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E21::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E21::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E21::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E21::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E21::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E21::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E21::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E21::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E21::T(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E21::U(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>>
    for E21<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E21::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E21::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E21::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E21::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E21::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E21::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E21::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E21::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E21::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E21::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E21::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E21::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E21::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E21::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E21::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E21::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E21::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E21::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E21::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E21::T(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E21::U(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>>
    for E22<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E22::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E22::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E22::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E22::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E22::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E22::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E22::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E22::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E22::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E22::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E22::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E22::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E22::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E22::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E22::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E22::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E22::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E22::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E22::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E22::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<U>() {
            E22::U(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E22::V(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>>
    for E22<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E22::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E22::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E22::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E22::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E22::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E22::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E22::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E22::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E22::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E22::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E22::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E22::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E22::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E22::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E22::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E22::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E22::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E22::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E22::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E22::T(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E22::U(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E22::V(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>>
    for E22<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
        &'a mut V,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E22::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E22::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E22::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E22::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E22::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E22::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E22::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E22::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E22::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E22::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E22::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E22::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E22::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E22::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E22::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E22::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E22::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E22::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E22::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E22::T(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E22::U(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E22::V(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>>
    for E23<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E23::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E23::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E23::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E23::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E23::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E23::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E23::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E23::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E23::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E23::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E23::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E23::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E23::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E23::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E23::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E23::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E23::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E23::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E23::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E23::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<U>() {
            E23::U(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<V>() {
            E23::V(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E23::W(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>>
    for E23<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E23::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E23::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E23::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E23::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E23::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E23::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E23::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E23::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E23::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E23::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E23::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E23::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E23::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E23::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E23::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E23::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E23::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E23::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E23::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E23::T(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E23::U(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E23::V(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E23::W(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>>
    for E23<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
        &'a mut V,
        &'a mut W,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E23::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E23::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E23::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E23::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E23::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E23::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E23::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E23::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E23::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E23::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E23::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E23::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E23::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E23::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E23::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E23::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E23::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E23::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E23::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E23::T(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E23::U(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E23::V(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E23::W(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>>
    for E24<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E24::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E24::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E24::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E24::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E24::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E24::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E24::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E24::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E24::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E24::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E24::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E24::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E24::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E24::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E24::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E24::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E24::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E24::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E24::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E24::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<U>() {
            E24::U(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<V>() {
            E24::V(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<W>() {
            E24::W(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E24::X(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>>
    for E24<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E24::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E24::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E24::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E24::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E24::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E24::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E24::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E24::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E24::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E24::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E24::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E24::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E24::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E24::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E24::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E24::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E24::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E24::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E24::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E24::T(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E24::U(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E24::V(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E24::W(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E24::X(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>>
    for E24<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
        &'a mut V,
        &'a mut W,
        &'a mut X,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E24::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E24::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E24::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E24::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E24::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E24::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E24::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E24::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E24::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E24::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E24::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E24::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E24::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E24::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E24::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E24::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E24::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E24::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E24::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E24::T(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E24::U(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E24::V(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E24::W(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E24::X(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>>
    for E25<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E25::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E25::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E25::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E25::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E25::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E25::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E25::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E25::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E25::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E25::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E25::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E25::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E25::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E25::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E25::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E25::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E25::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E25::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E25::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E25::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<U>() {
            E25::U(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<V>() {
            E25::V(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<W>() {
            E25::W(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<X>() {
            E25::X(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E25::Y(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>>
    for E25<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X, &'a Y>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E25::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E25::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E25::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E25::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E25::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E25::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E25::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E25::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E25::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E25::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E25::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E25::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E25::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E25::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E25::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E25::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E25::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E25::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E25::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E25::T(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E25::U(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E25::V(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E25::W(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<X>() {
            E25::X(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E25::Y(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>>
    for E25<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
        &'a mut V,
        &'a mut W,
        &'a mut X,
        &'a mut Y,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E25::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E25::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E25::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E25::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E25::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E25::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E25::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E25::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E25::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E25::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E25::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E25::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E25::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E25::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E25::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E25::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E25::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E25::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E25::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E25::T(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E25::U(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E25::V(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E25::W(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<X>() {
            E25::X(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E25::Y(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> From<ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>>
    for E26<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
    Z: 'static,
{
    fn from(union_of: ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E26::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E26::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<C>() {
            E26::C(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<D>() {
            E26::D(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<E>() {
            E26::E(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<F>() {
            E26::F(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<G>() {
            E26::G(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<H>() {
            E26::H(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<I>() {
            E26::I(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<J>() {
            E26::J(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<K>() {
            E26::K(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<L>() {
            E26::L(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<M>() {
            E26::M(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<N>() {
            E26::N(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<O>() {
            E26::O(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<P>() {
            E26::P(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Q>() {
            E26::Q(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<R>() {
            E26::R(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<S>() {
            E26::S(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<T>() {
            E26::T(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<U>() {
            E26::U(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<V>() {
            E26::V(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<W>() {
            E26::W(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<X>() {
            E26::X(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<Y>() {
            E26::Y(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E26::Z(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> From<&'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>>
    for E26<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X, &'a Y, &'a Z>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
    Z: 'static,
{
    fn from(union_of: &'a ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E26::A(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E26::B(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E26::C(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E26::D(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E26::E(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E26::F(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E26::G(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E26::H(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E26::I(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E26::J(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E26::K(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E26::L(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E26::M(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E26::N(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E26::O(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E26::P(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E26::Q(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E26::R(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E26::S(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E26::T(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E26::U(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E26::V(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E26::W(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<X>() {
            E26::X(union_of.inner.downcast_error_ref().unwrap())
        } else if union_of.inner.is_error::<Y>() {
            E26::Y(union_of.inner.downcast_error_ref().unwrap())
        } else {
            E26::Z(union_of.inner.downcast_error_ref().unwrap())
        }
    }
}

#[rustfmt::skip]
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> From<&'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>>
    for E26<
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        &'a mut E,
        &'a mut F,
        &'a mut G,
        &'a mut H,
        &'a mut I,
        &'a mut J,
        &'a mut K,
        &'a mut L,
        &'a mut M,
        &'a mut N,
        &'a mut O,
        &'a mut P,
        &'a mut Q,
        &'a mut R,
        &'a mut S,
        &'a mut T,
        &'a mut U,
        &'a mut V,
        &'a mut W,
        &'a mut X,
        &'a mut Y,
        &'a mut Z,
    >
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
    H: 'static,
    I: 'static,
    J: 'static,
    K: 'static,
    L: 'static,
    M: 'static,
    N: 'static,
    O: 'static,
    P: 'static,
    Q: 'static,
    R: 'static,
    S: 'static,
    T: 'static,
    U: 'static,
    V: 'static,
    W: 'static,
    X: 'static,
    Y: 'static,
    Z: 'static,
{
    fn from(union_of: &'a mut ErrorUnion<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E26::A(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<B>() {
            E26::B(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<C>() {
            E26::C(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<D>() {
            E26::D(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<E>() {
            E26::E(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<F>() {
            E26::F(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<G>() {
            E26::G(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<H>() {
            E26::H(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<I>() {
            E26::I(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<J>() {
            E26::J(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<K>() {
            E26::K(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<L>() {
            E26::L(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<M>() {
            E26::M(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<N>() {
            E26::N(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<O>() {
            E26::O(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<P>() {
            E26::P(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Q>() {
            E26::Q(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<R>() {
            E26::R(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<S>() {
            E26::S(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<T>() {
            E26::T(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<U>() {
            E26::U(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<V>() {
            E26::V(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<W>() {
            E26::W(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<X>() {
            E26::X(union_of.inner.downcast_error_mut().unwrap())
        } else if union_of.inner.is_error::<Y>() {
            E26::Y(union_of.inner.downcast_error_mut().unwrap())
        } else {
            E26::Z(union_of.inner.downcast_error_mut().unwrap())
        }
    }
}
