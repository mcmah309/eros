#![no_std]
#![no_main]

use core::any::TypeId;

#[eros::inline_error("operation timed out")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Timeout;

fn application_failure(inner: eros::InnerError) -> Timeout {
    use eros_no_alloc_test::inline_error::DomainFailure;
    assert_eq!(inner.downcast::<DomainFailure>().unwrap(), DomainFailure);
    Timeout
}

fn application_pipeline(mode: u8) -> eros::Result<u8, (Timeout,)> {
    use eros::{Erase, ReshapeUnion};
    use eros_no_alloc_test::inline_error as library;

    let input: eros::Result<
        u8,
        (
            Timeout,
            library::Timeout,
            library::DeviceUnavailable,
            library::OtherError,
        ),
    > = library::pipeline(mode).widen();
    let input = match library::select_result(input.erase()) {
        Ok(selected) => Err(library::erase_error(library::widen_selected(selected))),
        Err(remaining) => remaining,
    };
    library::classify_result(input).classify(application_failure)
}

fn application_payload_pipeline(
    mode: u8,
) -> eros::Result<
    u8,
    (
        Timeout,
        eros_no_alloc_test::inline_error::DeviceFailure,
        eros_no_alloc_test::inline_error::DomainFailure,
    ),
> {
    use eros::ReshapeUnion;
    use eros_no_alloc_test::inline_error as library;
    let input = match library::restore_payloads(library::erase_payload_pipeline(mode)) {
        Ok(selected) => Err(library::widen_payloads(selected).erase()),
        Err(remaining) => remaining,
    };
    library::classify_payload(input).widen()
}

#[cfg(target_arch = "x86_64")]
// SAFETY: this binary supplies its sole process entry symbol; startup code is
// excluded at link time. The naked body establishes stack alignment before
// calling Rust code and never returns to the loader.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // Linux enters without a return address; establish the function-call ABI.
    core::arch::naked_asm!("and rsp, -16", "call {run}", run = sym run);
}

#[cfg(not(target_arch = "x86_64"))]
// SAFETY: this freestanding binary supplies its sole entry symbol and never
// returns; the target's startup environment must provide a valid stack.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    run()
}

fn run() -> ! {
    use eros_no_alloc_test::inline_error as library;
    assert_ne!(TypeId::of::<Timeout>(), TypeId::of::<library::Timeout>());
    let error = library::erased_timeout()
        .downcast_inner::<Timeout>()
        .unwrap_err();
    assert_eq!(
        error.downcast_inner::<library::Timeout>().unwrap(),
        library::Timeout
    );
    // Run the complete chain across this executable's crate boundary, with
    // classification back into its own distinct Timeout type.
    for mode in 0..=3 {
        let result = application_pipeline(mode);
        if mode == 3 {
            assert_eq!(result.unwrap(), 12);
        } else {
            let error = result.unwrap_err();
            assert!(!error.is_inner::<library::Timeout>());
            let error = library::select_error(error.erase()).unwrap_err();
            assert_eq!(error.downcast_inner::<Timeout>().unwrap(), Timeout);
        }
    }
    for mode in 0..=7 {
        let result = application_payload_pipeline(mode);
        if mode == 5 {
            assert_eq!(result.unwrap(), 43);
        } else {
            let error = result.unwrap_err().erase();
            let expected = [42, 2, 513, 7, 42, 0, 0, 43][usize::from(mode)];
            assert_eq!(
                error
                    .downcast_inner::<library::DeviceFailure>()
                    .unwrap()
                    .code,
                expected
            );
        }
    }
    let error = library::erase_payload_pipeline(3).unwrap_err();
    assert!(core::ptr::eq(
        error.downcast_inner::<library::StaticFailure>().unwrap().0,
        &library::DETAILS
    ));
    eros_no_alloc_test::run_checks();
    exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    exit(1)
}

fn exit(status: usize) -> ! {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    // SAFETY: Linux x86_64 syscall 60 exits the calling thread without accessing
    // user memory. Its arguments use the required registers and it never returns.
    unsafe {
        core::arch::asm!("syscall", in("rax") 60usize, in("rdi") status, options(noreturn));
    }
    #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
    {
        let _ = status;
        loop {
            core::hint::spin_loop();
        }
    }
}

// Runtime routines normally supplied by the platform. Volatile accesses keep
// LLVM from rewriting these implementations into calls to themselves.
/// # Safety
/// For nonzero count, source and dest must be valid for count bytes of reads and
/// writes respectively, with no overlap. Source bytes must be initialized, and
/// accesses must not conflict with other references or concurrent accesses.
// SAFETY: this freestanding binary supplies the sole memcpy symbol with the
// platform's C signature and memory-copy behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, source: *const u8, count: usize) -> *mut u8 {
    for index in 0..count {
        // SAFETY: the caller provides initialized readable source and writable
        // dest ranges; index remains within both and u8 requires alignment 1.
        unsafe {
            dest.add(index)
                .write_volatile(source.add(index).read_volatile());
        }
    }
    dest
}

/// # Safety
/// For nonzero count, dest must be valid for count bytes of writes, with no
/// conflicting references or concurrent accesses to those bytes.
// SAFETY: this freestanding binary supplies the sole memset symbol with the
// platform's C signature and byte-fill behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, count: usize) -> *mut u8 {
    for index in 0..count {
        // SAFETY: the caller provides a writable range containing index;
        // u8 requires alignment 1, and no conflicting access is permitted.
        unsafe {
            dest.add(index).write_volatile(value as u8);
        }
    }
    dest
}

/// # Safety
/// For nonzero count, left and right must each be valid for count bytes of reads
/// from initialized memory, without conflicting concurrent writes.
// SAFETY: this freestanding binary supplies the sole memcmp symbol with the
// platform's C signature and byte-comparison behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(left: *const u8, right: *const u8, count: usize) -> i32 {
    for index in 0..count {
        // SAFETY: the caller provides initialized readable ranges, index is
        // within both, and u8 requires alignment 1.
        let a = unsafe { left.add(index).read_volatile() };
        // SAFETY: the same caller guarantees apply to right's indexed byte.
        let b = unsafe { right.add(index).read_volatile() };
        if a != b {
            return i32::from(a) - i32::from(b);
        }
    }
    0
}

/// # Safety
/// Both input ranges must satisfy memcmp's read and initialization requirements.
// SAFETY: this freestanding binary supplies the sole bcmp symbol. memcmp has
// the same C signature and satisfies bcmp's zero/nonzero result convention.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(left: *const u8, right: *const u8, count: usize) -> i32 {
    // SAFETY: the caller supplies exactly the ranges required by memcmp.
    unsafe { memcmp(left, right, count) }
}

// SAFETY: this freestanding binary supplies the sole personality symbol and is
// built with panic=abort, so no unwinder calls this placeholder.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
