use super::{TracedUnion, E1, E2, E3, E4, E5, E6, E7, E8, E9};

/* ------------------------- Enum conversions ----------------------- */

impl<A> From<TracedUnion<(A,)>> for E1<A>
where
    A: 'static,
{
    fn from(union_of: TracedUnion<(A,)>) -> Self {
        E1::A(unsafe { union_of.inner.downcast_error_unchecked() })
    }
}

impl<'a, A> From<&'a TracedUnion<(A,)>> for E1<&'a A>
where
    A: 'static,
{
    fn from(union_of: &'a TracedUnion<(A,)>) -> Self {
        E1::A(union_of.inner.downcast_error_ref())
    }
}

impl<'a, A> From<&'a mut TracedUnion<(A,)>> for E1<&'a mut A>
where
    A: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A,)>) -> Self {
        E1::A(union_of.inner.downcast_error_mut())
    }
}

impl<A, B> From<TracedUnion<(A, B)>> for E2<A, B>
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: TracedUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E2::B(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

impl<'a, A, B> From<&'a TracedUnion<(A, B)>> for E2<&'a A, &'a B>
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: &'a TracedUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(union_of.inner.downcast_error_ref())
        } else {
            E2::B(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B> From<&'a mut TracedUnion<(A, B)>> for E2<&'a mut A, &'a mut B>
where
    A: 'static,
    B: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E2::A(union_of.inner.downcast_error_mut())
        } else {
            E2::B(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C> From<TracedUnion<(A, B, C)>> for E3<A, B, C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: TracedUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(unsafe { union_of.inner.downcast_error_unchecked() })
        } else if union_of.inner.is_error::<B>() {
            E3::B(unsafe { union_of.inner.downcast_error_unchecked() })
        } else {
            E3::C(unsafe { union_of.inner.downcast_error_unchecked() })
        }
    }
}

impl<'a, A, B, C> From<&'a TracedUnion<(A, B, C)>> for E3<&'a A, &'a B, &'a C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: &'a TracedUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E3::B(union_of.inner.downcast_error_ref())
        } else {
            E3::C(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C> From<&'a mut TracedUnion<(A, B, C)>> for E3<&'a mut A, &'a mut B, &'a mut C>
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B, C)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E3::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E3::B(union_of.inner.downcast_error_mut())
        } else {
            E3::C(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C, D> From<TracedUnion<(A, B, C, D)>> for E4<A, B, C, D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: TracedUnion<(A, B, C, D)>) -> Self {
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

impl<'a, A, B, C, D> From<&'a TracedUnion<(A, B, C, D)>> for E4<&'a A, &'a B, &'a C, &'a D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: &'a TracedUnion<(A, B, C, D)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E4::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E4::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E4::C(union_of.inner.downcast_error_ref())
        } else {
            E4::D(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C, D> From<&'a mut TracedUnion<(A, B, C, D)>>
    for E4<&'a mut A, &'a mut B, &'a mut C, &'a mut D>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E4::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E4::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E4::C(union_of.inner.downcast_error_mut())
        } else {
            E4::D(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C, D, E> From<TracedUnion<(A, B, C, D, E)>> for E5<A, B, C, D, E>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: TracedUnion<(A, B, C, D, E)>) -> Self {
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

impl<'a, A, B, C, D, E> From<&'a TracedUnion<(A, B, C, D, E)>>
    for E5<&'a A, &'a B, &'a C, &'a D, &'a E>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: &'a TracedUnion<(A, B, C, D, E)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E5::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E5::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E5::C(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<D>() {
            E5::D(union_of.inner.downcast_error_ref())
        } else {
            E5::E(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C, D, E> From<&'a mut TracedUnion<(A, B, C, D, E)>>
    for E5<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D, E)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E5::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E5::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E5::C(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<D>() {
            E5::D(union_of.inner.downcast_error_mut())
        } else {
            E5::E(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C, D, E, F> From<TracedUnion<(A, B, C, D, E, F)>> for E6<A, B, C, D, E, F>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: TracedUnion<(A, B, C, D, E, F)>) -> Self {
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

impl<'a, A, B, C, D, E, F> From<&'a TracedUnion<(A, B, C, D, E, F)>>
    for E6<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: &'a TracedUnion<(A, B, C, D, E, F)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E6::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E6::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E6::C(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<D>() {
            E6::D(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<E>() {
            E6::E(union_of.inner.downcast_error_ref())
        } else {
            E6::F(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C, D, E, F> From<&'a mut TracedUnion<(A, B, C, D, E, F)>>
    for E6<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D, E, F)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E6::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E6::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E6::C(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<D>() {
            E6::D(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<E>() {
            E6::E(union_of.inner.downcast_error_mut())
        } else {
            E6::F(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C, D, E, F, G> From<TracedUnion<(A, B, C, D, E, F, G)>> for E7<A, B, C, D, E, F, G>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
{
    fn from(union_of: TracedUnion<(A, B, C, D, E, F, G)>) -> Self {
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

impl<'a, A, B, C, D, E, F, G> From<&'a mut TracedUnion<(A, B, C, D, E, F, G)>>
    for E7<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G>
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
    E: 'static,
    F: 'static,
    G: 'static,
{
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D, E, F, G)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E7::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E7::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E7::C(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<D>() {
            E7::D(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<E>() {
            E7::E(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<F>() {
            E7::F(union_of.inner.downcast_error_mut())
        } else {
            E7::G(union_of.inner.downcast_error_mut())
        }
    }
}

impl<'a, A, B, C, D, E, F, G> From<&'a TracedUnion<(A, B, C, D, E, F, G)>>
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
    fn from(union_of: &'a TracedUnion<(A, B, C, D, E, F, G)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E7::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E7::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E7::C(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<D>() {
            E7::D(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<E>() {
            E7::E(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<F>() {
            E7::F(union_of.inner.downcast_error_ref())
        } else {
            E7::G(union_of.inner.downcast_error_ref())
        }
    }
}

impl<A, B, C, D, E, F, G, H> From<TracedUnion<(A, B, C, D, E, F, G, H)>>
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
    fn from(union_of: TracedUnion<(A, B, C, D, E, F, G, H)>) -> Self {
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

impl<'a, A, B, C, D, E, F, G, H> From<&'a TracedUnion<(A, B, C, D, E, F, G, H)>>
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
    fn from(union_of: &'a TracedUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E8::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E8::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E8::C(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<D>() {
            E8::D(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<E>() {
            E8::E(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<F>() {
            E8::F(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<G>() {
            E8::G(union_of.inner.downcast_error_ref())
        } else {
            E8::H(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C, D, E, F, G, H> From<&'a mut TracedUnion<(A, B, C, D, E, F, G, H)>>
    for E8<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H>
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
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E8::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E8::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E8::C(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<D>() {
            E8::D(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<E>() {
            E8::E(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<F>() {
            E8::F(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<G>() {
            E8::G(union_of.inner.downcast_error_mut())
        } else {
            E8::H(union_of.inner.downcast_error_mut())
        }
    }
}

impl<A, B, C, D, E, F, G, H, I> From<TracedUnion<(A, B, C, D, E, F, G, H, I)>>
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
    fn from(union_of: TracedUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
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

impl<'a, A, B, C, D, E, F, G, H, I> From<&'a TracedUnion<(A, B, C, D, E, F, G, H, I)>>
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
    fn from(union_of: &'a TracedUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E9::A(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<B>() {
            E9::B(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<C>() {
            E9::C(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<D>() {
            E9::D(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<E>() {
            E9::E(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<F>() {
            E9::F(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<G>() {
            E9::G(union_of.inner.downcast_error_ref())
        } else if union_of.inner.is_error::<H>() {
            E9::H(union_of.inner.downcast_error_ref())
        } else {
            E9::I(union_of.inner.downcast_error_ref())
        }
    }
}

impl<'a, A, B, C, D, E, F, G, H, I> From<&'a mut TracedUnion<(A, B, C, D, E, F, G, H, I)>>
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
    fn from(union_of: &'a mut TracedUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        if union_of.inner.is_error::<A>() {
            E9::A(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<B>() {
            E9::B(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<C>() {
            E9::C(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<D>() {
            E9::D(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<E>() {
            E9::E(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<F>() {
            E9::F(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<G>() {
            E9::G(union_of.inner.downcast_error_mut())
        } else if union_of.inner.is_error::<H>() {
            E9::H(union_of.inner.downcast_error_mut())
        } else {
            E9::I(union_of.inner.downcast_error_mut())
        }
    }
}
