#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::ToString;
use eros::{
    AnyError, ErrorUnion, IntoAnyUnion, IntoUnion, MsgError, ReshapeUnion, SendSyncError, error,
};

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
    assert_type::<MsgError>(untitled.inner(), "MsgError present")?;
    assert_display(untitled.inner(), "something went wrong")?;

    let formatted: ErrorUnion<AnyError> = eros::error!("val = {}", 7u32);
    assert_display(formatted.inner(), "val = 7")?;
    assert_type::<MsgError>(formatted.inner(), "owned MsgError")?;

    let value = 7u32;
    let captured = eros::error!("val = {value}");
    assert_display(captured.inner(), "val = 7")?;

    static ERROR: &str = "static message";
    let named = eros::error!(ERROR);
    assert_display(named.inner(), ERROR)?;

    let named = named.downcast_inner::<Timeout>().unwrap_err();
    assert_eq_str(named.downcast_inner::<MsgError>().unwrap().as_str(), ERROR)?;

    let result: eros::Result<u8, (Timeout, MsgError)> = (|| eros::bail!(ERROR))();
    let recovered: eros::Result<u8, (Timeout,)> =
        result.recover(|error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.into_single().as_str(), ERROR);
            7
        });
    assert_eq(recovered.recover::<Timeout, _>(|_| 0).into_value(), 7)?;
    let result: eros::Result<u8, (Timeout, MsgError)> = (|| eros::bail!(ERROR))();
    let recovered: eros::Result<u8, (Timeout,)> =
        result.try_recover(|error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.into_single().as_str(), ERROR);
            Err(ErrorUnion::new(Timeout))
        });
    assert_eq(recovered.unwrap_err().into_single(), Timeout)?;
    let result: eros::Result<u8, (Timeout, MsgError)> = (|| eros::bail!(ERROR))();
    let value = result
        .recover(|error: ErrorUnion<(Timeout, MsgError)>| {
            assert!(matches!(error.as_enum(), eros::E2::B(_)));
            7
        })
        .into_value();
    assert_eq(value, 7)?;
    let result: eros::Result<u8, (Timeout, MsgError)> = (|| eros::bail!(ERROR))();
    let recovered: eros::Result<u8, (NotEnoughMemory,)> = result
        .try_recover(|_: ErrorUnion<(MsgError, Timeout)>| Err(ErrorUnion::new(NotEnoughMemory)));
    assert_eq(recovered.unwrap_err().into_single(), NotEnoughMemory)?;
    let result: eros::Result<(), (Timeout, MsgError)> = (|| {
        eros::ensure!(false, "typed {}", 7);
        Ok(())
    })();
    assert_display(result.unwrap_err().inner(), "typed 7")?;

    let r: eros::Result<()> = bailing_function();
    let union = r.expect_err("bail should error");
    assert_display(union.inner(), "boom from bail")?;
    assert_type::<MsgError>(union.inner(), "bail MsgError")?;

    let r: Result<(), ErrorUnion<(NotEnoughMemory,)>> =
        Err(NotEnoughMemory).union::<_, (NotEnoughMemory,)>();
    let union = r.unwrap_err();
    assert_type::<NotEnoughMemory>(union.inner(), "NotEnoughMemory")?;
    assert_eq(union.into_single(), NotEnoughMemory)?;

    let u: ErrorUnion<(Timeout,)> = ErrorUnion::new(Timeout);
    assert_eq(u.as_ref(), &Timeout)?;
    assert_eq(u.into_single(), Timeout)?;

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
    let capture = u.location();
    let u = u.context("allocating memory failed");
    assert_type::<NotEnoughMemory>(u.inner(), "context preserves type")?;
    let u = u.with_context(|| "while booting");
    assert_eq(u.location(), capture)?;
    assert_eq(u.contexts().len(), 2)?;
    for frame in u.contexts() {
        assert_eq(frame.location().file(), file!())?;
        assert_eq(frame.is_user_facing(), false)?;
        assert!(frame.value().as_str().is_some());
    }
    assert_type::<NotEnoughMemory>(u.inner(), "with_context preserves type")?;
    if u.latest_context_error().is_some() {
        return Err(CheckOutcome::Fail("string contexts have no context error"));
    }
    assert_type::<NotEnoughMemory>(
        u.latest_context_error().unwrap_or_else(|| u.inner()),
        "context error fallback preserves inner error",
    )?;

    let u: ErrorUnion<(InvalidPassword,)> = ErrorUnion::new(InvalidPassword);
    let u = u.user_context("Please choose a stronger password.");
    let user_ctxs: alloc::vec::Vec<alloc::string::String> = u
        .contexts()
        .filter(|frame| frame.is_user_facing())
        .map(|frame| frame.to_string())
        .collect();
    assert_eq(user_ctxs.len(), 1)?;
    assert_eq_str(&user_ctxs[0], "Please choose a stronger password.")?;

    let r: Result<(), ErrorUnion<AnyError>> = Err(NotEnoughMemory).any_union();
    let union = r.unwrap_err();
    assert_type::<NotEnoughMemory>(union.inner(), "any_union preserves type")?;

    let u: ErrorUnion<(NotEnoughMemory,)> = ErrorUnion::new(NotEnoughMemory);
    let mapped: ErrorUnion<(Timeout,)> = u.map_single(|NotEnoughMemory| Timeout);
    assert_eq(mapped.into_single(), Timeout)?;

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
    assert_type::<NotEnoughMemory>(union.inner(), "chain failure type")?;

    let u: ErrorUnion<(InvalidPassword,)> = ErrorUnion::new(InvalidPassword);
    let dyn_err = u.into_inner();
    if !(&*dyn_err as &dyn core::any::Any).is::<InvalidPassword>() {
        return Err(CheckOutcome::Fail("into_inner_dyn_error concrete type"));
    }
    assert_display(
        &*dyn_err,
        "Your password must be at least 8 characters long.",
    )?;

    let u: ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)> = ErrorUnion::new(Timeout);
    let sub: Result<ErrorUnion<(Timeout,)>, ErrorUnion<(NotEnoughMemory, InvalidPassword)>> =
        u.narrow::<(Timeout,), _>();
    if sub.is_err() {
        return Err(CheckOutcome::Fail("narrow (Timeout,) should succeed"));
    }
    let sub_union = sub.unwrap();
    assert_eq(sub_union.into_single(), Timeout)?;

    let result: eros::Result<(), (NotEnoughMemory, Timeout, InvalidPassword)> =
        Err(ErrorUnion::new(Timeout));
    let selected = result.narrow::<(InvalidPassword, Timeout), _>().unwrap();
    assert_type::<Timeout>(selected.inner(), "narrow result group")?;

    let u: ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)> = ErrorUnion::new(NotEnoughMemory);
    let remainder = u.narrow::<(Timeout,), _>().expect_err("non-member must be rejected");
    assert_type::<NotEnoughMemory>(remainder.inner(), "narrow remainder")?;

    let diagnostic = eros::error!("diagnostic root").context("diagnostic context").to_debug_json();
    assert_eq_str(diagnostic["root"].as_str().unwrap(), "diagnostic root")?;
    assert_eq_str(diagnostic["contexts"][0]["message"].as_str().unwrap(), "diagnostic context")?;

    Ok(())
}

fn bailing_function() -> eros::Result<()> {
    const BAIL_ERROR: &str = "boom from bail";
    eros::bail!(BAIL_ERROR)
}

fn chain_with_question() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> {
    let _: () =
        Err::<(), NotEnoughMemory>(NotEnoughMemory)
            .union::<_, (NotEnoughMemory, Timeout, InvalidPassword)>()?;
    Ok(())
}

fn chain_with_failure() -> Result<(), ErrorUnion<(NotEnoughMemory, Timeout, InvalidPassword)>> {
    let _: () =
        Err::<(), NotEnoughMemory>(NotEnoughMemory)
            .union::<_, (NotEnoughMemory, Timeout, InvalidPassword)>()?;
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
    use eros::{AnyError, Context, ErrorUnion, SendSyncError, MsgError};

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
        let dyn_err = u.into_inner();
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
    fn into_enum_works() {
        let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(Timeout);
        assert!(matches!(u.into_enum(), eros::E2::B(Timeout)));

        let u: ErrorUnion<(NotEnoughMemory, Timeout)> = ErrorUnion::new(NotEnoughMemory);
        assert!(matches!(u.into_enum(), eros::E2::A(NotEnoughMemory)));
    }

    #[test]
    fn context_on_plain_result() {
        let r: Result<(), NotEnoughMemory> = Err(NotEnoughMemory);
        let widened: Result<(), ErrorUnion<(NotEnoughMemory,)>> = r.context("op failed");
        let u = widened.unwrap_err();
        assert_eq!(u.into_single(), NotEnoughMemory);
    }

    #[test]
    fn context_on_option() {
        let none: Option<u8> = None;
        let r: Result<u8, ErrorUnion<(eros::AbsentValueError,)>> = none.context("expected a value");
        assert!(r.is_err());
        let u = r.unwrap_err();
        assert!(u.inner().as_any().is::<eros::AbsentValueError>());
    }

    #[test]
    fn msg_error_alloc_conversions() {
        let from_string: MsgError = String::from("hi").into();
        assert_eq!(from_string.as_str(), "hi");
        let from_static: MsgError = "x".into();
        assert_eq!(from_static.as_str(), "x");
    }
}
