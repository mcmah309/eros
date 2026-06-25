#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::ToString;
use eros::{AnyError, ErrorUnion, IntoDynUnion, IntoUnion, SendSyncError, StrError, error};

#[derive(Debug, PartialEq, Eq)]
pub struct NotEnoughMemory;

impl core::fmt::Display for NotEnoughMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Not enough memory")
    }
}
impl core::error::Error for NotEnoughMemory {}

#[derive(Debug, PartialEq, Eq)]
pub struct Timeout;

impl core::fmt::Display for Timeout {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Timeout")
    }
}
impl core::error::Error for Timeout {}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPassword;

impl core::fmt::Display for InvalidPassword {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Your password must be at least 8 characters long.")
    }
}
impl core::error::Error for InvalidPassword {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    Ok,
    Fail(&'static str),
}

pub fn run_no_std_checks() -> Result<(), CheckOutcome> {
    let untitled: ErrorUnion<AnyError> = error!("something went wrong");
    assert_type::<StrError>(untitled.inner_ref(), "StrError present")?;
    assert_display(untitled.inner_ref(), "something went wrong")?;

    let formatted: ErrorUnion<AnyError> = eros::error!("val = {}", 7u32);
    assert_display(formatted.inner_ref(), "val = 7")?;
    assert_type::<StrError>(formatted.inner_ref(), "owned StrError")?;

    let r: eros::Result<()> = bailing_function();
    let union = r.expect_err("bail should error");
    assert_display(union.inner_ref(), "boom from bail")?;
    assert_type::<StrError>(union.inner_ref(), "bail StrError")?;

    let r: Result<(), ErrorUnion<(NotEnoughMemory,)>> =
        Err(NotEnoughMemory).into_union::<_, (NotEnoughMemory,)>();
    let union = r.unwrap_err();
    assert_type::<NotEnoughMemory>(union.inner_ref(), "NotEnoughMemory")?;
    assert_eq(union.into_inner(), NotEnoughMemory)?;

    let u: ErrorUnion<(Timeout,)> = ErrorUnion::new(Timeout);
    assert_eq(u.inner(), &Timeout)?;
    assert_eq(u.into_inner(), Timeout)?;

    let u: ErrorUnion<(NotEnoughMemory,)> = ErrorUnion::new(NotEnoughMemory);
    let widened: ErrorUnion<(NotEnoughMemory, Timeout)> = u.widen();
    match widened.narrow::<NotEnoughMemory, _>() {
        Ok(NotEnoughMemory) => {}
        Err(_) => return Err(CheckOutcome::Fail("narrow NotEnoughMemory")),
    }

    let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
    match u.narrow::<Timeout, _>() {
        Ok(Timeout) => {}
        Err(_) => return Err(CheckOutcome::Fail("narrow Timeout")),
    }
    let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
    let remainder: Result<Timeout, ErrorUnion<(NotEnoughMemory,)>> = u.narrow::<Timeout, _>();
    if remainder.is_err() {
        return Err(CheckOutcome::Fail("narrow Timeout remainder"));
    }
    assert_eq(remainder.unwrap(), Timeout)?;

    let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
    assert_eq(u.narrow::<Timeout, _>().unwrap(), Timeout)?;
    let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);
    assert_eq(u.narrow::<NotEnoughMemory, _>().unwrap(), NotEnoughMemory)?;

    let u: ErrorUnion<(NotEnoughMemory,)> = ErrorUnion::new(NotEnoughMemory);
    let u = u.context("allocating memory failed");
    assert_type::<NotEnoughMemory>(u.inner_ref(), "context preserves type")?;
    let u = u.with_context(|| "while booting");
    assert_type::<NotEnoughMemory>(u.inner_ref(), "with_context preserves type")?;
    assert_type::<NotEnoughMemory>(u.latest_error(), "latest_error is original")?;

    let u: ErrorUnion<(InvalidPassword,)> = ErrorUnion::new(InvalidPassword);
    let u = u.user_context("Please choose a stronger password.");
    let user_ctxs: alloc::vec::Vec<alloc::string::String> =
        u.user_contexts().map(|c| c.to_string()).collect();
    assert_eq(user_ctxs.len(), 1)?;
    assert_eq_str(&user_ctxs[0], "Please choose a stronger password.")?;

    let r: Result<(), ErrorUnion<AnyError>> = Err(NotEnoughMemory).into_dyn_union();
    let union = r.unwrap_err();
    assert_type::<NotEnoughMemory>(union.inner_ref(), "into_dyn_union preserves type")?;

    let u: ErrorUnion<(NotEnoughMemory,)> = ErrorUnion::new(NotEnoughMemory);
    let mapped: ErrorUnion<(Timeout,)> = u.map(|NotEnoughMemory| Timeout);
    assert_eq(mapped.into_inner(), Timeout)?;

    let outcome: Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> =
        chain_with_question();
    if outcome.is_ok() {
        return Err(CheckOutcome::Fail(
            "chain_with_question should have propogated the error",
        ));
    }

    let outcome: Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> =
        chain_with_failure();
    let union = outcome.unwrap_err();
    assert_type::<NotEnoughMemory>(union.inner_ref(), "chain failure type")?;

    let u: ErrorUnion<(InvalidPassword,)> = ErrorUnion::new(InvalidPassword);
    let dyn_err = u.into_inner_dyn_error();
    if !(&*dyn_err as &dyn core::any::Any).is::<InvalidPassword>() {
        return Err(CheckOutcome::Fail("into_inner_dyn_error concrete type"));
    }
    assert_display(
        &*dyn_err,
        "Your password must be at least 8 characters long.",
    )?;

    let u: ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)> = ErrorUnion::new(Timeout);
    let sub: Result<ErrorUnion<(Timeout,)>, ErrorUnion<(NotEnoughMemory, InvalidPassword)>> =
        u.subset::<(Timeout,), _>();
    if sub.is_err() {
        return Err(CheckOutcome::Fail("subset (Timeout,) should succeed"));
    }
    let sub_union = sub.unwrap();
    assert_eq(sub_union.into_inner(), Timeout)?;

    Ok(())
}

fn bailing_function() -> eros::Result<()> {
    eros::bail!("boom from bail")
}

fn chain_with_question() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> {
    let _: () =
        Err::<(), NotEnoughMemory>(NotEnoughMemory)
            .into_union::<_, (NotEnoughMemory, Timeout, InvalidPassword)>()?;
    Ok(())
}

fn chain_with_failure() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> {
    let _: () =
        Err::<(), NotEnoughMemory>(NotEnoughMemory)
            .into_union::<_, (NotEnoughMemory, Timeout, InvalidPassword)>()?;
    Ok(())
}

fn assert_eq<T: PartialEq + core::fmt::Debug>(got: T, want: T) -> Result<(), CheckOutcome> {
    if got == want {
        Ok(())
    } else {
        Err(CheckOutcome::Fail("assert_eq mismatch"))
    }
}

fn assert_eq_str(got: &str, want: &'static str) -> Result<(), CheckOutcome> {
    if got == want {
        Ok(())
    } else {
        Err(CheckOutcome::Fail("assert_eq_str mismatch"))
    }
}

fn assert_type<T: SendSyncError>(
    error: &dyn SendSyncError,
    label: &'static str,
) -> Result<(), CheckOutcome> {
    if error.as_any().is::<T>() {
        Ok(())
    } else {
        let _ = label;
        Err(CheckOutcome::Fail("assert_type"))
    }
}

fn assert_display(error: &dyn SendSyncError, want: &'static str) -> Result<(), CheckOutcome> {
    let got = error.to_string();
    if got == want {
        Ok(())
    } else {
        let _ = got;
        Err(CheckOutcome::Fail("assert_display"))
    }
}

#[cfg(all(not(test), not(feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::{NotEnoughMemory, Timeout, run_no_std_checks};
    use eros::{AnyError, Context, ErrorUnion, SendSyncError, StrError};

    #[test]
    fn all_no_std_checks_pass() {
        assert_eq!(
            run_no_std_checks(),
            Ok(()),
            "at least one eros no-std check failed"
        );
    }

    #[test]
    fn error_union_is_send_sync_without_std() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        let u: ErrorUnion<(NotEnoughMemory,)> = ErrorUnion::new(NotEnoughMemory);
        assert_send::<ErrorUnion<(NotEnoughMemory,)>>();
        assert_sync::<ErrorUnion<(NotEnoughMemory,)>>();
        let dyn_err = u.into_inner_dyn_error();
        assert_send::<Box<dyn SendSyncError>>();
        assert_sync::<Box<dyn SendSyncError>>();
        assert!((&*dyn_err as &dyn core::any::Any).is::<NotEnoughMemory>());
    }

    #[test]
    fn macros_format_through_alloc() {
        let e: ErrorUnion<AnyError> = eros::error!("val = {}", 7u32);
        assert_eq!(e.to_string(), "val = 7");
    }

    #[test]
    fn to_enum_works() {
        let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
        assert!(matches!(u.to_enum(), eros::E2::B(Timeout)));

        let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);
        assert!(matches!(u.to_enum(), eros::E2::A(NotEnoughMemory)));
    }

    #[test]
    fn context_on_plain_result() {
        let r: Result<(), NotEnoughMemory> = Err(NotEnoughMemory);
        let widened: Result<(), ErrorUnion<(NotEnoughMemory,)>> = r.context("op failed");
        let u = widened.unwrap_err();
        assert_eq!(u.into_inner(), NotEnoughMemory);
    }

    #[test]
    fn context_on_option() {
        let none: Option<u8> = None;
        let r: Result<u8, ErrorUnion<(eros::AbsentValueError,)>> = none.context("expected a value");
        assert!(r.is_err());
        let u = r.unwrap_err();
        assert!(u.inner_ref().as_any().is::<eros::AbsentValueError>());
    }

    #[test]
    fn str_error_alloc_conversions() {
        let from_string: StrError = String::from("hi").into();
        assert_eq!(from_string.as_str(), "hi");
        let from_static: StrError = "x".into();
        assert_eq!(from_static.as_str(), "x");
    }
}
