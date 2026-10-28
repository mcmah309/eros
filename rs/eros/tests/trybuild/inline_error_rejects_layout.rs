#[eros::inline_error("too large")]
struct Large([usize; 2]);

#[eros::inline_error("wide reference")]
struct Wide(&'static str);

#[eros::inline_error("aligned")]
#[repr(align(64))]
struct Aligned(u8);

#[eros::inline_error("large enum")]
#[repr(C)]
enum LargeEnum { Empty, Value(usize) }

#[eros::inline_error]
struct ManualLarge([usize; 2]);

impl core::fmt::Display for ManualLarge {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("large")
    }
}

fn main() {}
