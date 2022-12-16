use crate::vga_buf::SCREEN;
use crate::{print, println};
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;

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

pub fn init_shell() {
    good_formatting();
}

// REGION of MY METHODS

#[derive(Debug, Clone, Copy)]
struct Directory {
    index: usize,
    name: [u8; MAX_SIZE_DIRECTORY_NAME],
    parent_index: usize,
    child_count: usize,
    child_indexes: [usize; MAX_COUNT_CHILDREN_DIRECTORIES],
}

struct DirectoryList {
    directories: [Directory; 100],
    directory_count: usize,
}

pub fn mu_split(arr: [u8; 80], buf_len: usize) -> ([u8; COMMAND_SIZE], [u8; ARGV_SIZE]) {
    let mut cmd: [u8; COMMAND_SIZE] = [b'\0'; COMMAND_SIZE];
    let mut argument: [u8; ARGV_SIZE] = [b'\0'; ARGV_SIZE];

    let mut i = 0;

    while arr[i] != b' ' && i < COMMAND_SIZE {
        cmd[i] = arr[i];
        i += 1;
    }

    if i == buf_len {
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
// END REGION of MY METHODS

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
    directory_list: DirectoryList,
    current_directory: Directory,
}

impl Shell {
    fn execute_command(&mut self, argv: ([u8; COMMAND_SIZE], [u8; ARGV_SIZE])) {
        if compare_str_with_arr("echo", argv.0) {
            self.echo_command(argv.1);
        } else if compare_str_with_arr("curdir", argv.0) {
            self.current_directory_command(self.current_directory);
        } else if compare_str_with_arr("mkdir", argv.0) {
            self.create_folder_command(argv.1);
        } else if compare_str_with_arr("clear", argv.0) {
            self.clear_command();
        } else if compare_str_with_arr("cd", argv.0) {
            self.change_directory_command(argv.1);
        } else if compare_str_with_arr("dirtree", argv.0) {
            self.directory_tree_command(self.directory_list.directories[self.current_directory.index], 0);
        } else if compare_str_with_arr("nocmd", argv.0) {
        } else {
            print_error_command_not_found(argv.0);
        }
    }

    fn echo_command(&mut self, argv: [u8; ARGV_SIZE]) {
        println!();
        for symbol in argv {
            print!("{}", symbol as char);
        }
    }

    fn change_directory_command(&mut self, argv: [u8; ARGV_SIZE]) {
        if argv[0] == b'.' {
            self.current_directory =
                self.directory_list.directories[self.current_directory.parent_index];
            return;
        }

        let cur_dir = self.directory_list.directories[self.current_directory.index];

        for dir_index in cur_dir.child_indexes {
            let mut is_same = true;
            for i in 0..ARGV_SIZE {
                if argv[i] == b'\0' {
                    break;
                }

                if i == MAX_SIZE_DIRECTORY_NAME {
                    print!("[Error] The maximum size of the directory name is 10 characters");
                    return;
                }

                if self.directory_list.directories[dir_index].name[i] != argv[i] {
                    is_same = false;
                    break;
                }
            }

            if is_same {
                self.current_directory = self.directory_list.directories[dir_index];
                return;
            }
        }

        print!(
            "\nFolder \"{}\" is not exist!",
            core::str::from_utf8(&argv.clone())
                .unwrap()
                .trim_matches('\0')
        )
    }

    fn clear_command(&mut self) {
        SCREEN.lock().clear();
    }

    fn directory_tree_command(&mut self, current_directory: Directory, tab_count: usize) {
        println!();
        for i in 0..current_directory.child_count {
            let child_directory =
                self.directory_list.directories[current_directory.child_indexes[i]];

            for tc in 0..tab_count {
                for ts in 0..4 {
                    print!(" ");
                }
            }
            print!(
                "/{}",
                core::str::from_utf8(&child_directory.name)
                    .unwrap()
                    .trim_matches('\0')
            );

            self.directory_tree_command(child_directory, tab_count + 1);
        }
    }

    fn print_directory_name(&mut self, dir_name: [u8; MAX_SIZE_DIRECTORY_NAME]) {
        print!(
            "{}",
            core::str::from_utf8(&dir_name).unwrap().trim_matches('\0')
        );
        SCREEN.lock().push_row_to_right(0);
        SCREEN.lock().move_print_to(0);

        print!("/");
    }

    fn create_folder_command(&mut self, argv: [u8; ARGV_SIZE]) {
        let mut name_size = 0;
        for i in 0..ARGV_SIZE {
            if argv[i] == b'\0' {
                break;
            }
            name_size += 1;
        }

        if name_size > MAX_SIZE_DIRECTORY_NAME {
            print!("\n[Error] The maximum size of the directory name is 10 characters");
            return;
        }

        let mut directory: Directory = Directory {
            index: self.directory_list.directory_count,
            name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
            parent_index: self.current_directory.index,
            child_count: 0,
            child_indexes: [0; MAX_COUNT_CHILDREN_DIRECTORIES],
        };

        for i in 0..MAX_SIZE_DIRECTORY_NAME {
            directory.name[i] = argv[i];
        }
        let current_directory = self.directory_list.directories[self.current_directory.index];

        self.directory_list.directories[self.directory_list.directory_count] = directory;
        self.directory_list.directories[self.current_directory.index].child_indexes[current_directory.child_count] = self.directory_list.directory_count;

        self.directory_list.directory_count += 1;
        self.directory_list.directories[self.current_directory.index].child_count += 1;

        print!(
            "\n[Ok] Directory \"{}\" created succsessfully!",
            core::str::from_utf8(&directory.name.clone())
                .unwrap()
                .trim_matches('\0')
        );
    }

    fn current_directory_command(&mut self, current_directory: Directory) -> usize {
        let parent_directory = self.directory_list.directories[current_directory.parent_index];
        let mut nesting = 0;

        if current_directory.index > 0 {
            nesting = self.current_directory_command(parent_directory);
        } else {
            println!();
        }

        print!(
            "/{}",
            core::str::from_utf8(&current_directory.name.clone())
                .unwrap()
                .trim_matches('\0')
        );

        return nesting;
    }

    pub fn new() -> Shell {
        let mut shell: Shell = Shell {
            buf: [0; 80],
            buf_len: 0,
            directory_list: DirectoryList {
                directories: ([Directory {
                    index: 0,
                    name: [b' '; MAX_SIZE_DIRECTORY_NAME],
                    parent_index: 0,
                    child_count: 0,
                    child_indexes: [0; MAX_COUNT_CHILDREN_DIRECTORIES],
                }; 100]),
                directory_count: 1,
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

        shell.directory_list.directories[0] = shell.current_directory;

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
                if self.buf_len > 0 {
                    self.buf_len -= 1;
                }
                self.buf[self.buf_len] = 0;
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
