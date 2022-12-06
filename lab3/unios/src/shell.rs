use crate::vga_buf::SCREEN;
use crate::{print, println};
use core::ptr::null_mut;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, KeyCode};

lazy_static! {
    static ref SH: spin::Mutex<Shell> = spin::Mutex::new({
        let mut sh = Shell::new();
        sh
    });
}

pub fn handle_keyboard_interrupt(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(c) => SH.lock().on_key_pressed(c as u8),
        DecodedKey::RawKey(rk) => {}
    }
}

// REGION of MY METHODS

pub fn mu_split(arr: [u8; 80], buf_len: usize) -> ([u8; 10], [u8; 70]) {
    let mut cmd: [u8; 10] = [b'\0'; 10];
    let mut argument: [u8; 70] = [b'\0'; 70];

    let mut i = 0;

    while i < 9 || arr[i] != b' ' {
        cmd[i] = arr[i];
        i += 1;
    }
    i += 1;
    let mut j = 0;
    while i < buf_len {
        argument[j] = arr[i];
        i += 1;
        j += 1;
    }

    return (cmd, argument);
}

pub fn compare_str_with_arr(str_for_compare: &str, arr: [u8; 10]) -> bool {
    let mut are_the_same = true;

    let mut i = 0;
    for symbol in str_for_compare.bytes() {
        if symbol != arr[i] {
            are_the_same = false;
        }
        i += 1;
    }
    return are_the_same;
}

pub fn execute_command(argv: ([u8; 10], [u8; 70])) {
    if compare_str_with_arr("echo", argv.0) {
        echo_command(argv.1);
    }
}

fn echo_command(argv: [u8; 70]) {
    println!();
    for symbol in argv {
        print!("{}", symbol);
    }
}

// END REGION of MY METHODS

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            buf: [0; 80],
            buf_len: 0,
        }
    }

    pub fn on_key_pressed(&mut self, key: u8) {
        match key {
            b'\n' => {
                let argv = mu_split(self.buf, self.buf_len);

                execute_command(argv);
            }
            8 => {
                SCREEN.lock().delete_last_symbol();
            }
            32 =>
            // key code of space button
            {
                self.buf[self.buf_len] = b' ';
                self.buf_len += 1;
                print!("{}", key as char);
            }
            _ => {
                self.buf[self.buf_len] = key;
                self.buf_len += 1;
                print!("{}", key as char);
            }
        }
    }
}
