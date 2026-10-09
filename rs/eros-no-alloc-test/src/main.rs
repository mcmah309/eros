#![no_std]
#![no_main]

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    // Linux enters without a return address; establish the function-call ABI.
    core::arch::naked_asm!("and rsp, -16", "call {run}", run = sym run);
}

#[cfg(not(target_arch = "x86_64"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    run()
}

fn run() -> ! {
    eros_no_alloc_test::run_checks();
    exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    exit(1)
}

fn exit(status: usize) -> ! {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, source: *const u8, count: usize) -> *mut u8 {
    for index in 0..count {
        unsafe {
            dest.add(index)
                .write_volatile(source.add(index).read_volatile());
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, value: i32, count: usize) -> *mut u8 {
    for index in 0..count {
        unsafe {
            dest.add(index).write_volatile(value as u8);
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(left: *const u8, right: *const u8, count: usize) -> i32 {
    for index in 0..count {
        let a = unsafe { left.add(index).read_volatile() };
        let b = unsafe { right.add(index).read_volatile() };
        if a != b {
            return i32::from(a) - i32::from(b);
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(left: *const u8, right: *const u8, count: usize) -> i32 {
    unsafe { memcmp(left, right, count) }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
