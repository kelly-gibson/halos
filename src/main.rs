// disables the standard library
#![no_std]
// tells the compiler to not use the normal entry point chain
#![no_main]

pub mod vga_buffer;

use core::panic::PanicInfo;

// This function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// static HELLO: &[u8] = b"Welcome to focus OS";

// Creating an entry point. Also tells the compiler to use the C calling convention, rather than the rust convention.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    //vga_buffer::print_something();
    //vga_buffer::example_global_writer();
    use core::fmt::Write;
    use vga_buffer::with_writer;
    with_writer(|writer| {
        // Perform write operations using the writer
        write!(writer, "The numbers are {} and {}", 56, 1.0 / 3.0).unwrap();
    });

    loop{}
}