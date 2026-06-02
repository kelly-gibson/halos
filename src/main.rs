// disables the standard library
#![no_std]
// tells the compiler to not use the normal entry point chain
#![no_main]
// feature attributes to enable tests (clippy stop yelling at me >.<)
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;
pub mod vga_buffer;
mod serial;

// This function is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Creating an entry point. Also tells the compiler to use the C calling convention, rather than the rust convention.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("test harness running");

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    serial_println!("Done, exiting QEMU");
    exit_qemu(QemuExitCode::Success);
}
#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
