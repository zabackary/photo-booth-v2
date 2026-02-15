pub mod capture_flash;
pub mod capture_preview;
pub mod countdown_circle;
pub mod ready;

#[cfg(feature = "fast_animations")]
const LENGTH_DIVISOR: u32 = 10;
#[cfg(not(feature = "fast_animations"))]
const LENGTH_DIVISOR: u64 = 1;
