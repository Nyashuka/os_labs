use crate::vga_buf::SCREEN;
use crate::{print, println};
use core::ptr::null_mut;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, KeyCode};

const FORMATING_STRING: &str = " $ ";
const FORMATING_STRING_LENGTH: u32 = 3;
const MAX_COUNT_CHILDREN_DIRECTORIES: usize = 20;
const MAX_SIZE_DIRECTORY_NAME: usize = 10;

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

#[derive(Debug, Clone, Copy)]
struct Directory {
    index: usize,
    name: [u8; MAX_SIZE_DIRECTORY_NAME],
    parent_index: usize,
    child_count: u8,
    child_indexes: [usize; MAX_COUNT_CHILDREN_DIRECTORIES],
}

struct DirectoryList {
    directories: [Directory; 100],
    next_directory: u8,
}

pub fn mu_split(arr: [u8; 80], buf_len: usize) -> ([u8; 10], [u8; 70]) {
    let mut cmd: [u8; 10] = [b'\0'; 10];
    let mut argument: [u8; 70] = [b'\0'; 70];

    let mut i = 0;

    while arr[i] != b' ' && i < 10 {
        cmd[i] = arr[i];
        i += 1;
    }

    if i == buf_len - 1 {
        return (cmd, argument); 
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

fn good_formatting() {
    print!("{}", FORMATING_STRING);
}

// END REGION of MY METHODS

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
    directories: DirectoryList,
    current_directory: Directory,
}

impl Shell {

    fn execute_command(&mut self, argv: ([u8; 10], [u8; 70])) {
        if compare_str_with_arr("echo", argv.0) {
            self.echo_command(argv.1);
        } else if compare_str_with_arr("curdir", argv.0) {
            self.current_directory_command();
        }
    }
    
    fn echo_command(&mut self, argv: [u8; 70]) {
        println!();
        for symbol in argv {
            print!("{}", symbol as char);
        }
    }
    
    fn current_directory_command(&mut self) {
        let mut dir = self.current_directory.clone();
        println!();
        while dir.parent_index != 0
        {
            for symbol in dir.name {
                print!("{}", symbol as char);
            }
            print!("/");
        }

        for symbol in self.directories.directories[0].clone().name {
            print!("{}", symbol as char);
        }
        print!("/");
    }   

    pub fn new() -> Shell {
        let mut shell: Shell = Shell {
            buf: [0; 80],
            buf_len: 0,
            directories: DirectoryList {
                directories: ([Directory {
                    index: 0,
                    name: [b' '; MAX_SIZE_DIRECTORY_NAME],
                    parent_index: 0,
                    child_count: 0,
                    child_indexes: [0; MAX_COUNT_CHILDREN_DIRECTORIES],
                }; 100]),
                next_directory: (1),
            },
            current_directory: Directory {
                index: 0,
                name: [b'r', b'o', b'o', b't', b' ', b' ', b' ', b' ', b' ', b' '],
                parent_index: 0,
                child_count: 0,
                child_indexes: [0; MAX_COUNT_CHILDREN_DIRECTORIES],
            },
        };

        shell.directories.directories[0] = shell.current_directory.clone();

        good_formatting();

        return shell;
    }

    pub fn on_key_pressed(&mut self, key: u8) {
        match key {
            b'\n' => {
                let argv = mu_split(self.buf, self.buf_len);

                self.execute_command(argv);
                self.buf_len = 0;
                println!();
                good_formatting()
            }
            8 =>
            // key code of backspace
            {
                SCREEN.lock().delete_last_symbol(FORMATING_STRING_LENGTH);
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
