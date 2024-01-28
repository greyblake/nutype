#![no_main]
#![no_std]

use core::panic::PanicInfo;
use nutype::nutype;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[nutype(validate(greater_or_equal = 1, less_or_equal = 6), derive(Debug))]
struct GermanTaxClass(i64);
