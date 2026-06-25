use core::any::Any;
use core::error::Error;
use core::fmt;
#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;

#[cfg(feature = "context")]
use crate::context::ErosContext;
use crate::{AnyError, SendSyncError};

/* ------------------------- Helpers ----------------------- */

/// The final element of a type-level Cons list.
#[doc(hidden)]
#[derive(Debug)]
pub enum End {}

impl core::error::Error for End {}

/// A compile-time list of types, similar to other basic functional list structures.
#[doc(hidden)]
#[derive(Debug)]
pub struct Cons<Head, Tail>(core::marker::PhantomData<Head>, Tail);

#[doc(hidden)]
#[derive(Debug)]
pub struct Recurse<Tail>(Tail);

/* ------------------------- std::error::Error support ----------------------- */

pub trait ErrorFold {
    fn source_fold(any: &dyn Any) -> Option<&(dyn Error + 'static)>;
}

impl ErrorFold for End {
    fn source_fold(_: &dyn Any) -> Option<&(dyn Error + 'static)> {
        unreachable!("source_fold called on End");
    }
}

impl<Head, Tail> Error for Cons<Head, Tail>
where
    Head: Error,
    Tail: Error,
{
}

impl<Head, Tail> ErrorFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: Error,
    Head: 'static + Error,
    Tail: ErrorFold,
{
    fn source_fold(any: &dyn Any) -> Option<&(dyn Error + 'static)> {
        if let Some(head_ref) = any.downcast_ref::<Head>() {
            head_ref.source()
        } else {
            Tail::source_fold(any)
        }
    }
}

/* ------------------------- Display support ----------------------- */

impl<Head, Tail> fmt::Display for Cons<Head, Tail>
where
    Head: fmt::Display,
    Tail: fmt::Display,
{
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("Display called for Cons which is not constructable")
    }
}

impl fmt::Display for End {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("Display::fmt called for an End, which is not constructible.")
    }
}

pub trait DisplayFold {
    fn display_fold(any: &dyn Any, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl DisplayFold for End {
    fn display_fold(_: &dyn Any, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("display_fold called on End");
    }
}

pub(crate) fn write_display<T: fmt::Display + ?Sized>(
    t: &T,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    t.fmt(formatter)
}

impl<Head, Tail> DisplayFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: fmt::Display,
    Head: 'static + fmt::Display,
    Tail: DisplayFold,
{
    fn display_fold(any: &dyn Any, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(head_ref) = any.downcast_ref::<Head>() {
            write_display(head_ref, formatter)
        } else {
            Tail::display_fold(any, formatter)
        }
    }
}

/* ------------------------- Debug support ----------------------- */

pub trait DebugFold {
    fn debug_fold(
        any: &dyn SendSyncError,
        formatter: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] context: &[ErosContext],
        #[cfg(feature = "backtrace")] backtrace: &Backtrace,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
    ) -> fmt::Result;
}

impl DebugFold for End {
    fn debug_fold(
        _: &dyn SendSyncError,
        _: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] _context: &[ErosContext],
        #[cfg(feature = "backtrace")] _backtrace: &Backtrace,
        #[cfg(feature = "location")] _location: &'static core::panic::Location<'static>,
    ) -> fmt::Result {
        unreachable!("debug_fold called on End");
    }
}

pub(crate) fn write_debug<T: SendSyncError + ?Sized>(
    t: &T,
    formatter: &mut fmt::Formatter<'_>,
    #[cfg(feature = "context")] context: &[ErosContext],
    #[cfg(feature = "backtrace")] backtrace: &Backtrace,
    #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
) -> fmt::Result {
    #[cfg(feature = "context")]
    fn write_eros_context(
        context: &ErosContext,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        #[cfg(feature = "location")]
        {
            writeln!(
                formatter,
                "{}:{}:{}\n\t- {}",
                context.location.file(),
                context.location.line(),
                context.location.column(),
                context.context
            )
        }
        #[cfg(not(feature = "location"))]
        {
            writeln!(formatter, "\t- {}", context.context)
        }
    }
    #[cfg(feature = "location")]
    {
        writeln!(
            formatter,
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )?;
    }
    #[cfg(feature = "anyhow")]
    {
        use crate::error_union::{AnyhowError, AnyhowErrorArc};
        let anyhow_error: Option<&anyhow::Error> =
            if let Some(err) = t.as_any().downcast_ref::<AnyhowError>() {
                Some(&err.0)
            } else if let Some(err) = t.as_any().downcast_ref::<AnyhowErrorArc>() {
                Some(&*err.0)
            } else {
                None
            };
        if let Some(anyhow_error) = anyhow_error {
            let mut chain = anyhow_error.chain().rev().peekable();
            let root = chain.next().unwrap();
            writeln!(formatter, "{root}")?;
            #[cfg(feature = "context")]
            {
                let has_context = chain.peek().is_some() || !context.is_empty();
                if has_context {
                    writeln!(formatter, "\nContext:")?;
                }
                for context_item in chain {
                    writeln!(formatter, "\t- {}", context_item)?;
                }
                for context_item in context {
                    write_eros_context(context_item, formatter)?;
                }
                if has_context {
                    writeln!(formatter, "\n---")?;
                }
            }
            #[cfg(feature = "backtrace")]
            {
                use std::backtrace::BacktraceStatus;

                let anyhow_backtrace = anyhow_error.backtrace();
                if matches!(anyhow_backtrace.status(), BacktraceStatus::Captured) {
                    #[cfg(feature = "better_backtrace")]
                    write_better_backtrace(anyhow_backtrace, formatter)?;
                    #[cfg(not(feature = "better_backtrace"))]
                    write_backtrace(anyhow_backtrace, formatter)?;
                } else if matches!(backtrace.status(), BacktraceStatus::Captured) {
                    #[cfg(feature = "better_backtrace")]
                    write_better_backtrace(backtrace, formatter)?;
                    #[cfg(not(feature = "better_backtrace"))]
                    write_backtrace(backtrace, formatter)?;
                }
            }
            return Ok(());
        }
    }
    fmt::Debug::fmt(&t, formatter)?;
    writeln!(formatter, "\n---")?;
    #[cfg(feature = "context")]
    {
        if !context.is_empty() {
            writeln!(formatter, "\nContext:")?;
            for context_item in context.iter() {
                write_eros_context(context_item, formatter)?;
            }
            writeln!(formatter, "\n---")?;
        }
    }
    #[cfg(feature = "backtrace")]
    {
        use std::backtrace::BacktraceStatus;

        if matches!(backtrace.status(), BacktraceStatus::Captured) {
            #[cfg(feature = "better_backtrace")]
            write_better_backtrace(backtrace, formatter)?;
            #[cfg(not(feature = "better_backtrace"))]
            write_backtrace(backtrace, formatter)?;
        }
    }
    Ok(())
}

#[cfg(feature = "backtrace")]
fn write_backtrace(
    backtrace: &std::backtrace::Backtrace,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    use std::backtrace::BacktraceStatus;

    if matches!(backtrace.status(), BacktraceStatus::Captured) {
        writeln!(formatter, "\nBacktrace:")?;
        fmt::Display::fmt(&backtrace, formatter)?;
    }
    Ok(())
}

#[cfg(feature = "better_backtrace")]
fn write_better_backtrace(
    backtrace: &std::backtrace::Backtrace,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let printer = color_backtrace::BacktracePrinter::new().add_frame_filter(Box::new(
        |frames: &mut Vec<&color_backtrace::Frame>| {
            frames.retain(|frame| {
                !(frame.is_dependency_code()
                    || frame.is_post_panic_code()
                    || frame.is_runtime_init_code())
            });
        },
    ));
    let Ok(btparse_backtrace) = btparse::deserialize(backtrace) else {
        write_backtrace(backtrace, formatter)?;
        return Ok(());
    };
    let Ok(backtrace_string) = printer.format_trace_to_string(&btparse_backtrace) else {
        write_backtrace(backtrace, formatter)?;
        return Ok(());
    };
    fmt::Display::fmt(&backtrace_string, formatter)?;
    Ok(())
}

impl<Head, Tail> DebugFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: fmt::Debug,
    Head: SendSyncError,
    Tail: DebugFold,
{
    fn debug_fold(
        any: &dyn SendSyncError,
        formatter: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] context: &[ErosContext],
        #[cfg(feature = "backtrace")] backtrace: &Backtrace,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
    ) -> fmt::Result {
        if let Some(head_ref) = (any as &dyn Any).downcast_ref::<Head>() {
            write_debug(
                head_ref,
                formatter,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "backtrace")]
                backtrace,
                #[cfg(feature = "location")]
                location,
            )
        } else {
            Tail::debug_fold(
                any,
                formatter,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "backtrace")]
                backtrace,
                #[cfg(feature = "location")]
                location,
            )
        }
    }
}

/* ------------------------- Any::is support ----------------------- */

pub trait IsFold {
    fn is_fold(any: &dyn Any) -> bool;
}

impl IsFold for End {
    fn is_fold(_: &dyn Any) -> bool {
        false
    }
}

impl<Head, Tail> IsFold for Cons<Head, Tail>
where
    Head: 'static,
    Tail: IsFold,
{
    fn is_fold(any: &dyn Any) -> bool {
        if any.is::<Head>() {
            true
        } else {
            Tail::is_fold(any)
        }
    }
}

/* ------------------------- TypeSet implemented for tuples ----------------------- */

#[rustfmt::skip]
pub trait TypeSet {
    type Variants: TupleForm;
    type Enum;
    type RefEnum<'a> where Self: 'a;
    type MutEnum<'a> where Self: 'a;
}

impl TupleForm for AnyError {
    type Tuple = AnyError;
}

#[rustfmt::skip]
impl TypeSet for AnyError {
    type Variants = AnyError;
    type Enum = AnyError;
    type RefEnum<'a> = &'a AnyError where Self: 'a;
    type MutEnum<'a> = &'a mut AnyError where Self: 'a;
}

#[rustfmt::skip]
impl TypeSet for () {
    type Variants = End;
    type Enum = E0;
    type RefEnum<'a> = E0 where Self: 'a;
    type MutEnum<'a> = E0 where Self: 'a;
}

#[rustfmt::skip]
impl<A> TypeSet for (A,) {
    type Variants = Cons<A, End>;
    type Enum = E1<A>;
    type RefEnum<'a> = E1<&'a A> where Self: 'a;
    type MutEnum<'a> = E1<&'a mut A> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B> TypeSet for (A, B) {
    type Variants = Cons<A, Cons<B, End>>;
    type Enum = E2<A, B>;
    type RefEnum<'a> = E2<&'a A, &'a B> where Self: 'a;
    type MutEnum<'a> = E2<&'a mut A, &'a mut B> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C> TypeSet for (A, B, C) {
    type Variants = Cons<A, Cons<B, Cons<C, End>>>;
    type Enum = E3<A, B, C>;
    type RefEnum<'a> = E3<&'a A, &'a B, &'a C> where Self: 'a;
    type MutEnum<'a> = E3<&'a mut A, &'a mut B, &'a mut C> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D> TypeSet for (A, B, C, D) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, End>>>>;
    type Enum = E4<A, B, C, D>;
    type RefEnum<'a> = E4<&'a A, &'a B, &'a C, &'a D> where Self: 'a;
    type MutEnum<'a> = E4<&'a mut A, &'a mut B, &'a mut C, &'a mut D> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E> TypeSet for (A, B, C, D, E) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>>;
    type Enum = E5<A, B, C, D, E>;
    type RefEnum<'a> = E5<&'a A, &'a B, &'a C, &'a D, &'a E> where Self: 'a;
    type MutEnum<'a> = E5<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F> TypeSet for (A, B, C, D, E, F) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>>;
    type Enum = E6<A, B, C, D, E, F>;
    type RefEnum<'a> = E6<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F> where Self: 'a;
    type MutEnum<'a> = E6<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G> TypeSet for (A, B, C, D, E, F, G) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>;
    type Enum = E7<A, B, C, D, E, F, G>;
    type RefEnum<'a> = E7<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G> where Self: 'a;
    type MutEnum<'a> = E7<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H> TypeSet for (A, B, C, D, E, F, G, H) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>;
    type Enum = E8<A, B, C, D, E, F, G, H>;
    type RefEnum<'a> = E8<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H> where Self: 'a;
    type MutEnum<'a> = E8<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I> TypeSet for (A, B, C, D, E, F, G, H, I) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>;
    type Enum = E9<A, B, C, D, E, F, G, H, I>;
    type RefEnum<'a> = E9<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I> where Self: 'a;
    type MutEnum<'a> = E9<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J> TypeSet for (A, B, C, D, E, F, G, H, I, J) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, End>>>>>>>>>>;
    type Enum = E10<A, B, C, D, E, F, G, H, I, J>;
    type RefEnum<'a> = E10<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J> where Self: 'a;
    type MutEnum<'a> = E10<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K> TypeSet for (A, B, C, D, E, F, G, H, I, J, K) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, End>>>>>>>>>>>;
    type Enum = E11<A, B, C, D, E, F, G, H, I, J, K>;
    type RefEnum<'a> = E11<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K> where Self: 'a;
    type MutEnum<'a> = E11<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, End>>>>>>>>>>>>;
    type Enum = E12<A, B, C, D, E, F, G, H, I, J, K, L>;
    type RefEnum<'a> = E12<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L> where Self: 'a;
    type MutEnum<'a> = E12<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, End>>>>>>>>>>>>>;
    type Enum = E13<A, B, C, D, E, F, G, H, I, J, K, L, M>;
    type RefEnum<'a> = E13<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M> where Self: 'a;
    type MutEnum<'a> = E13<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, End>>>>>>>>>>>>>>;
    type Enum = E14<A, B, C, D, E, F, G, H, I, J, K, L, M, N>;
    type RefEnum<'a> = E14<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N> where Self: 'a;
    type MutEnum<'a> = E14<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, End>>>>>>>>>>>>>>>;
    type Enum = E15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O>;
    type RefEnum<'a> = E15<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O> where Self: 'a;
    type MutEnum<'a> = E15<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, End>>>>>>>>>>>>>>>>;
    type Enum = E16<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P>;
    type RefEnum<'a> = E16<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P> where Self: 'a;
    type MutEnum<'a> = E16<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, End>>>>>>>>>>>>>>>>>;
    type Enum = E17<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q>;
    type RefEnum<'a> = E17<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q> where Self: 'a;
    type MutEnum<'a> = E17<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, End>>>>>>>>>>>>>>>>>>;
    type Enum = E18<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R>;
    type RefEnum<'a> = E18<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R> where Self: 'a;
    type MutEnum<'a> = E18<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, End>>>>>>>>>>>>>>>>>>>;
    type Enum = E19<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S>;
    type RefEnum<'a> = E19<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S> where Self: 'a;
    type MutEnum<'a> = E19<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, End>>>>>>>>>>>>>>>>>>>>;
    type Enum = E20<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>;
    type RefEnum<'a> = E20<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T> where Self: 'a;
    type MutEnum<'a> = E20<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, End>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E21<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U>;
    type RefEnum<'a> = E21<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U> where Self: 'a;
    type MutEnum<'a> = E21<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, End>>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E22<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V>;
    type RefEnum<'a> = E22<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V> where Self: 'a;
    type MutEnum<'a> = E22<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U, &'a mut V> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, End>>>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E23<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W>;
    type RefEnum<'a> = E23<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W> where Self: 'a;
    type MutEnum<'a> = E23<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U, &'a mut V, &'a mut W> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, End>>>>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E24<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X>;
    type RefEnum<'a> = E24<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X> where Self: 'a;
    type MutEnum<'a> = E24<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U, &'a mut V, &'a mut W, &'a mut X> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, End>>>>>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E25<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y>;
    type RefEnum<'a> = E25<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X, &'a Y> where Self: 'a;
    type MutEnum<'a> = E25<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U, &'a mut V, &'a mut W, &'a mut X, &'a mut Y> where Self: 'a;
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, Cons<Z, End>>>>>>>>>>>>>>>>>>>>>>>>>>;
    type Enum = E26<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z>;
    type RefEnum<'a> = E26<&'a A, &'a B, &'a C, &'a D, &'a E, &'a F, &'a G, &'a H, &'a I, &'a J, &'a K, &'a L, &'a M, &'a N, &'a O, &'a P, &'a Q, &'a R, &'a S, &'a T, &'a U, &'a V, &'a W, &'a X, &'a Y, &'a Z> where Self: 'a;
    type MutEnum<'a> = E26<&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E, &'a mut F, &'a mut G, &'a mut H, &'a mut I, &'a mut J, &'a mut K, &'a mut L, &'a mut M, &'a mut N, &'a mut O, &'a mut P, &'a mut Q, &'a mut R, &'a mut S, &'a mut T, &'a mut U, &'a mut V, &'a mut W, &'a mut X, &'a mut Y, &'a mut Z> where Self: 'a;
}

/* ------------------------- TupleForm implemented for TypeSet ----------------------- */

pub trait TupleForm {
    type Tuple: TypeSet;
}

impl TupleForm for End {
    type Tuple = ();
}

#[rustfmt::skip]
impl<A> TupleForm
    for Cons<A, End>
{
    type Tuple = (A,);
}

#[rustfmt::skip]
impl<A, B> TupleForm
    for Cons<A, Cons<B, End>>
{
    type Tuple = (A, B);
}

#[rustfmt::skip]
impl<A, B, C> TupleForm
    for Cons<A, Cons<B, Cons<C, End>>>
{
    type Tuple = (A, B, C);
}

#[rustfmt::skip]
impl<A, B, C, D> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, End>>>>
{
    type Tuple = (A, B, C, D);
}

#[rustfmt::skip]
impl<A, B, C, D, E> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>>
{
    type Tuple = (A, B, C, D, E);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>>
{
    type Tuple = (A, B, C, D, E, F);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, End>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, End>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, End>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, End>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, End>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, End>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, End>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, End>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, End>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, End>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, End>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, End>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, End>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, End>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, End>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, End>>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
}

#[rustfmt::skip]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, Cons<Z, End>>>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
}

/* ------------------------- Lifted ----------------------- */

impl<A> From<A> for E1<A> {
    fn from(a: A) -> E1<A> {
        E1::A(a)
    }
}

pub enum E0 {}

#[rustfmt::skip]
pub enum E1<A> { A(A) }

#[rustfmt::skip]
pub enum E2<A, B> { A(A), B(B) }

#[rustfmt::skip]
pub enum E3<A, B, C> { A(A), B(B), C(C) }

#[rustfmt::skip]
pub enum E4<A, B, C, D> { A(A), B(B), C(C), D(D) }

#[rustfmt::skip]
pub enum E5<A, B, C, D, E> { A(A), B(B), C(C), D(D), E(E) }

#[rustfmt::skip]
pub enum E6<A, B, C, D, E, F> { A(A), B(B), C(C), D(D), E(E), F(F) }

#[rustfmt::skip]
pub enum E7<A, B, C, D, E, F, G> { A(A), B(B), C(C), D(D), E(E), F(F), G(G) }

#[rustfmt::skip]
pub enum E8<A, B, C, D, E, F, G, H> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H) }

#[rustfmt::skip]
pub enum E9<A, B, C, D, E, F, G, H, I> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I) }

#[rustfmt::skip]
pub enum E10<A, B, C, D, E, F, G, H, I, J> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J) }

#[rustfmt::skip]
pub enum E11<A, B, C, D, E, F, G, H, I, J, K> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K) }

#[rustfmt::skip]
pub enum E12<A, B, C, D, E, F, G, H, I, J, K, L> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L) }

#[rustfmt::skip]
pub enum E13<A, B, C, D, E, F, G, H, I, J, K, L, M> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M) }

#[rustfmt::skip]
pub enum E14<A, B, C, D, E, F, G, H, I, J, K, L, M, N> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N) }

#[rustfmt::skip]
pub enum E15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O) }

#[rustfmt::skip]
pub enum E16<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P) }

#[rustfmt::skip]
pub enum E17<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q) }

#[rustfmt::skip]
pub enum E18<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R) }

#[rustfmt::skip]
pub enum E19<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S) }

#[rustfmt::skip]
pub enum E20<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T) }

#[rustfmt::skip]
pub enum E21<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U) }

#[rustfmt::skip]
pub enum E22<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U), V(V) }

#[rustfmt::skip]
pub enum E23<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U), V(V), W(W) }

#[rustfmt::skip]
pub enum E24<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U), V(V), W(W), X(X) }

#[rustfmt::skip]
pub enum E25<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U), V(V), W(W), X(X), Y(Y) }

#[rustfmt::skip]
pub enum E26<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> { A(A), B(B), C(C), D(D), E(E), F(F), G(G), H(H), I(I), J(J), K(K), L(L), M(M), N(N), O(O), P(P), Q(Q), R(R), S(S), T(T), U(U), V(V), W(W), X(X), Y(Y), Z(Z) }

/* ------------------------- Contains ----------------------- */

/// A trait that assists with compile-time type set inclusion testing.
/// The `Index` parameter is either `End` or `Cons<...>` depending on
/// whether the trait implementation is a base case or the recursive
/// case.
pub trait Contains<T, Index> {}

/// Base case implementation for when the Cons Head is T.
impl<T, Tail> Contains<T, End> for Cons<T, Tail> {}

/// Recursive case for when the Cons Tail contains T.
impl<T, Index, Head, Tail> Contains<T, Cons<Index, ()>> for Cons<Head, Tail> where
    Tail: Contains<T, Index>
{
}

impl<T> Contains<T, End> for AnyError {}

/* ------------------------- Narrow ----------------------- */

/// A trait for pulling a specific type out of a Variants at compile-time
/// and having access to the other types as the Remainder.
pub trait Narrow<Target, Index>: TupleForm {
    type Remainder: TupleForm;
}

/// Base case where the search Target is in the Head of the Variants.
impl<Target, Tail> Narrow<Target, End> for Cons<Target, Tail>
where
    Tail: TupleForm,
    Cons<Target, Tail>: TupleForm,
{
    type Remainder = Tail;
}

/// Recursive case where the search Target is in the Tail of the Variants.
impl<Head, Tail, Target, Index> Narrow<Target, Recurse<Index>> for Cons<Head, Tail>
where
    Tail: Narrow<Target, Index>,
    Tail: TupleForm,
    Cons<Head, Tail>: TupleForm,
    Cons<Head, <Tail as Narrow<Target, Index>>::Remainder>: TupleForm,
{
    type Remainder = Cons<Head, <Tail as Narrow<Target, Index>>::Remainder>;
}

fn _narrow_test() {
    use alloc::string::String;
    fn can_narrow<Types, Target, Remainder, Index>()
    where
        Types: Narrow<Target, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(u32, String) as TypeSet>::Variants;

    can_narrow::<T0, u32, _, _>();
    can_narrow::<T0, String, Cons<u32, End>, _>();
}

/* ------------------------- SupersetOf ----------------------- */

/// When all types in a Variants are present in a second Variants
pub trait SupersetOf<Other, Index> {
    type Remainder: TupleForm;
}

/// Base case
impl<T: TupleForm> SupersetOf<End, End> for T {
    type Remainder = T;
}

/// Recursive case - more complex because we have to reason about the Index itself as a
/// heterogenous list.
impl<SubHead, SubTail, SuperHead, SuperTail, HeadIndex, TailIndex>
    SupersetOf<Cons<SubHead, SubTail>, Cons<HeadIndex, TailIndex>> for Cons<SuperHead, SuperTail>
where
    Cons<SuperHead, SuperTail>: Narrow<SubHead, HeadIndex>,
    <Cons<SuperHead, SuperTail> as Narrow<SubHead, HeadIndex>>::Remainder:
        SupersetOf<SubTail, TailIndex>,
{
    type Remainder =
        <<Cons<SuperHead, SuperTail> as Narrow<SubHead, HeadIndex>>::Remainder as SupersetOf<
            SubTail,
            TailIndex,
        >>::Remainder;
}

impl SupersetOf<AnyError, End> for AnyError {
    type Remainder = AnyError;
}

fn _superset_test() {
    use alloc::{string::String, vec::Vec};
    fn is_superset<S1, S2, Remainder, Index>()
    where
        S1: SupersetOf<S2, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(u32,) as TypeSet>::Variants;
    type T1A = <(u32, String) as TypeSet>::Variants;
    type T1B = <(String, u32) as TypeSet>::Variants;
    type T2 = <(String, i32, u32) as TypeSet>::Variants;
    type T3 = <(Vec<u8>, Vec<i8>, u32, f32, String, f64, i32) as TypeSet>::Variants;

    is_superset::<T0, T0, _, _>();
    is_superset::<T1A, T1A, _, _>();
    is_superset::<T1A, T1B, _, _>();
    is_superset::<T1B, T1A, _, _>();
    is_superset::<T2, T2, _, _>();
    is_superset::<T1A, T0, _, _>();
    is_superset::<T1B, T0, _, _>();
    is_superset::<T2, T0, <(String, i32) as TypeSet>::Variants, _>();
    is_superset::<T2, T1A, <(i32,) as TypeSet>::Variants, _>();
    is_superset::<T2, T1B, <(i32,) as TypeSet>::Variants, _>();
    is_superset::<T3, T1A, <(Vec<u8>, Vec<i8>, f32, f64, i32) as TypeSet>::Variants, _>();
    is_superset::<T3, T1B, _, _>();
    is_superset::<T3, T0, _, _>();
    is_superset::<T3, T2, _, _>();

    type T5sup = <(u8, u16, u32, u64, u128) as TypeSet>::Variants;
    type T5sub = <(u8, u128) as TypeSet>::Variants;
    type T5rem = <(u16, u32, u64) as TypeSet>::Variants;

    is_superset::<T5sup, T5sub, T5rem, _>();
}
