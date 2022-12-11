use crate::vga_buf::SCREEN;
use crate::{print, println};
use core::ptr::null_mut;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, KeyCode};

const FORMATING_STRING: &str = " $ ";
const FORMATING_STRING_LENGTH: u32 = 3;
const MAX_COUNT_CHILDREN_DIRECTORIES: usize = 20;
const MAX_SIZE_DIRECTORY_NAME: usize = 10;
const COMMAND_SIZE: usize = 10;
const ARGV_SIZE: usize = 70;

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

pub fn init_shell()
{
    good_formatting();
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

pub fn mu_split(arr: [u8; 80], buf_len: usize) -> ([u8; COMMAND_SIZE], [u8; ARGV_SIZE]) {
    let mut cmd: [u8; COMMAND_SIZE] = [b'\0'; COMMAND_SIZE];
    let mut argument: [u8; ARGV_SIZE] = [b'\0'; ARGV_SIZE];

    let mut i = 0;

    while arr[i] != b' ' && i < COMMAND_SIZE {
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

pub fn compare_str_with_arr(str_for_compare: &str, arr: [u8; COMMAND_SIZE]) -> bool {
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

fn print_error_command_not_found(cmd: [u8; COMMAND_SIZE]) {
    println!();
    print!(
        "Command \"{}\" not found!",
        core::str::from_utf8(&cmd).unwrap().trim_matches('\0')
    );
}

// pub fn concat_u8_arr(first_arr: [u8;10], second_arr: [u8;10]) -> &'static str
// {
//     let mut n = 0;
//     for i in first_arr
//     {
//         if i == b'\0' {break};
//         n += 1;
//     }
//     for i in second_arr
//     {
//         if i == b'\0' {break};
//         n += 1;
//     }
//     const count:usize = n;

//     let mut new_arr = [b' '; n];
//     n = 0;
//     for i in first_arr
//     {
//         if i == b'\0' {break};
//         new_arr[n] = i;
//         n += 1;
//     }
//     for i in second_arr
//     {
       
//         if i == b'\0' {break};
//         new_arr[n] = i;
//         n += 1;
//     }

//     let concated_str = core::str::from_utf8(&new_arr).unwrap();
    
//     return concated_str;
// }

// END REGION of MY METHODS

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
    directories: DirectoryList,
    current_directory: Directory,
}

impl Shell {
    fn execute_command(&mut self, argv: ([u8; COMMAND_SIZE], [u8; ARGV_SIZE])) {
        if compare_str_with_arr("echo", argv.0) {
            self.echo_command(argv.1);
        } else if compare_str_with_arr("curdir", argv.0) {
            self.current_directory_command();
        } 
        else if compare_str_with_arr("clear", argv.0) {
            self.clear_command();
        } 
        else {
            print_error_command_not_found(argv.0);
        }
    }

    fn echo_command(&mut self, argv: [u8; ARGV_SIZE]) {
        println!();
        for symbol in argv {
            print!("{}", symbol as char);
        }
    }

    fn clear_command(&mut self)
    {
        SCREEN.lock().clear();
    }

    fn print_directory_name(&mut self, dir_name:[u8; MAX_SIZE_DIRECTORY_NAME])
    {
        print!(
            "{}",
            core::str::from_utf8(&dir_name).unwrap().trim_matches('\0')
        );
        SCREEN.lock().push_row_to_right(0);
        SCREEN.lock().move_print_to(0);
        
        print!("/");
    }

    fn current_directory_command(&mut self) {


        let mut dir = self.current_directory.clone();
        println!();
        while dir.parent_index != 0 {
            for symbol in dir.name {
                if (symbol == b'\0') {
                    break;
                }

                print!("{}", symbol as char);
                SCREEN.lock().push_row_to_right(0);
                SCREEN.lock().move_print_to(0);
            }
            print!("/");
            SCREEN.lock().push_row_to_right(0);
            SCREEN.lock().move_print_to(0);
        }
        // root
        self.print_directory_name(self.directories.directories[0].clone().name);
        
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
                name: [
                    b'r', b'o', b'o', b't', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0',
                ],
                parent_index: 0,
                child_count: 0,
                child_indexes: [0; MAX_COUNT_CHILDREN_DIRECTORIES],
            },
        };

        shell.directories.directories[0] = shell.current_directory.clone();

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
