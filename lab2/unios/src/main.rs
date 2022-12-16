#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

mod vga_buf;
mod game_of_life;


use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::write;

use self::vga_buf::*;
use self::game_of_life::*;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

//static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {

    let mut driver = VGADriver::new(Color::Pink, Color::White, Alignment::Center);

    for i in 0..100
    {
        write!(driver,"dasdsad {}\n", i);
    }

    //game_of_life(&mut driver);
    
    loop {}
}
