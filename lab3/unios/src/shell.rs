use core::ptr::null_mut;
use crate::{print, println};
use crate::vga_buf::SCREEN;
use pc_keyboard::{DecodedKey,KeyCode};
use lazy_static::lazy_static;

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

pub fn mu_split(arr: &[u8;80]) -> [String; 2]
{
    let mut argv: [String; 2] = ["".to_string(), "".to_string()];

    let mut current_str: String = String::from("");
    let mut counter = 0;

    for i in 0..arr.len() {
        if arr[i] != b' '{
            current_str.push((arr[i] as char));
        }
        if (arr[i] == b' ' || i == arr.len() - 1) && current_str != ""
        {
            argv[counter] = current_str;
            current_str = String::from("");
            counter += 1;
        }
    }

    return argv;
}

pub fn read_command_from_console(key: DecodedKey)
{
    match key {
        //DecodedKey::RawKey(KeyCode::Backspace) => ,
        DecodedKey::Unicode(k) => print!("{}", k),
        _ => print!("_")
    }
}

// END REGION of MY METHODS

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
}


pub fn execute_command(buf: &[u8; 80])
{
    let cmd = mu_split(&buf);


}

pub fn echo_command(res_str: &String)
{
    print!("{}", res_str);
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
                print!("\nImplement command execution: ");
                for i in 0..self.buf_len {
                    print!("{}", self.buf[i] as char)
                }
                println!()
            }
            b' ' => {

            }
            _ => {
                self.buf[self.buf_len] = key;
                self.buf_len += 1;
                print!("{}", key as char);
            }
        }
    }
}
