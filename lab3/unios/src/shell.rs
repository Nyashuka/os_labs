use crate::vga_buf::SCREEN;
use crate::{print, println};
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;

const FORMATING_STRING: &str = " $ ";
const FORMATING_STRING_LENGTH: u32 = 3;
const MAX_COUNT_CHILDREN_DIRECTORIES: usize = 20;
const MAX_COUNT_DIRECTORIES: usize = 100;
const MAX_COUNT_FILES_IN_FOLDER: usize = 5;
const MAX_COUNT_FILES: usize = 10;
const DELETED_INDEX_DIRECTORY: usize = MAX_COUNT_DIRECTORIES + 1;
const DELETED_INDEX_FILE: usize = MAX_COUNT_FILES + 1;
const MAX_SIZE_DIRECTORY_NAME: usize = 10;
const COMMAND_SIZE: usize = 10;
const ARGV_SIZE: usize = 70;

const BUF_HEIGHT: u32 = 25;
const BUF_WIDTH: u32 = 80;
const BUF_SIZE: usize = (BUF_HEIGHT * BUF_WIDTH) as usize;

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

#[derive(Debug, Clone, Copy)]
struct File {
    index: usize,
    name: [u8; MAX_SIZE_DIRECTORY_NAME],
    count_lines: usize,
    folder_index: usize,
    content: [u8; BUF_SIZE],
}

#[derive(Debug, Clone, Copy)]
struct Directory {
    index: usize,
    name: [u8; MAX_SIZE_DIRECTORY_NAME],
    parent_index: usize,
    child_count: usize,
    child_indexes: [usize; MAX_COUNT_CHILDREN_DIRECTORIES],
    files_indexes: [usize; MAX_COUNT_FILES_IN_FOLDER],
}

struct DirectoryList {
    directories: [Directory; MAX_COUNT_DIRECTORIES],
}

struct FileList {
    files: [File; MAX_COUNT_FILES],
}

struct Shell {
    buf: [u8; 80],
    buf_len: usize,
    directory_list: DirectoryList,
    files_list: FileList,
    current_directory: usize,
    is_editing_file: bool,
    current_editing_file: usize,
}

pub fn parameters_splitter(arr: [u8; 80], buf_len: usize) -> ([u8; COMMAND_SIZE], [u8; ARGV_SIZE]) {
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

impl Shell {
    fn execute_command(&mut self, argv: ([u8; COMMAND_SIZE], [u8; ARGV_SIZE])) {
        if compare_str_with_arr("echo", argv.0) {
            self.echo_command(argv.1);
        } else if compare_str_with_arr("curdir", argv.0) {
            self.current_directory_command(self.directory_list.directories[self.current_directory]);
        } else if compare_str_with_arr("mkdir", argv.0) {
            self.create_folder_command(argv.1);
        } else if compare_str_with_arr("clear", argv.0) {
            self.clear_command();
        } else if compare_str_with_arr("cd", argv.0) {
            self.change_directory_command(argv.1);
        } else if compare_str_with_arr("dirtree", argv.0) {
            self.directory_tree_command(self.directory_list.directories[self.current_directory], 0);
        } else if compare_str_with_arr("deldir", argv.0) {
            self.delete_directory_command(argv.1);
        } else if compare_str_with_arr("ls", argv.0) {
            self.list_files_command();
        } else if compare_str_with_arr("mkfile", argv.0) {
            self.create_file_command(argv.1);
        } else if compare_str_with_arr("delfile", argv.0) {
            self.delete_file_command(argv.1);
        } else if compare_str_with_arr("readfile", argv.0) {
            self.print_file_content_command(argv.1);
        } else if compare_str_with_arr("editfile", argv.0) {
            self.edit_file_command(argv.1);
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

    fn list_files_command(&mut self) {
        println!();
        for i in 0..MAX_COUNT_FILES_IN_FOLDER {
            if self.directory_list.directories[self.current_directory].files_indexes[i]
                != DELETED_INDEX_FILE
            {
                print!(
                    "{}    ",
                    core::str::from_utf8(
                        &self.files_list.files[self.directory_list.directories
                            [self.current_directory]
                            .files_indexes[i]]
                            .name
                            .clone()
                    )
                    .unwrap()
                    .trim_matches('\0')
                );
            }
        }
    }

    fn get_file_index_by_name(&mut self, argv: [u8; ARGV_SIZE]) -> usize {
        let mut current_file_index = DELETED_INDEX_FILE;
        let mut is_same = true;

        for i in 0..MAX_COUNT_FILES_IN_FOLDER {
            current_file_index =
                self.directory_list.directories[self.current_directory].files_indexes[i];

            if current_file_index == DELETED_INDEX_FILE {
                continue;
            }

            is_same = true;
            for j in 0..MAX_SIZE_DIRECTORY_NAME {
                if argv[j] == b'\0' {
                    break;
                }

                if argv[j] != self.files_list.files[current_file_index].name[j] {
                    is_same = false;
                    break;
                }
            }

            if is_same {
                return current_file_index;
            }
        }

        return DELETED_INDEX_FILE;
    }

    fn get_folder_index_by_name(&mut self, argv: [u8; ARGV_SIZE]) -> usize {

        for i in 0..MAX_COUNT_CHILDREN_DIRECTORIES {
            let mut is_same = true;
            let dir_index = self.directory_list.directories[self.current_directory].child_indexes[i];

            if dir_index == DELETED_INDEX_DIRECTORY
            {
                continue;
            }

            for i in 0..ARGV_SIZE {
                if argv[i] == b'\0' {
                    break;
                }

                if self.directory_list.directories[dir_index].name[i] != argv[i] {
                    is_same = false;
                    break;
                }
            }

            if is_same {
                // return index of child_indeexes array of current folder
                return i;
            } 
        }

        return DELETED_INDEX_DIRECTORY;
    }

    fn print_file_content_command(&mut self, argv: [u8; ARGV_SIZE]) {
        self.clear_command();

        let current_file_index = self.get_file_index_by_name(argv);

        if current_file_index == DELETED_INDEX_FILE {
            print!(
                "\n[Error] File \"{}\" does not exist!",
                core::str::from_utf8(&argv.clone())
                    .unwrap()
                    .trim_matches('\0')
            );
            return;
        }

        for j in 0..(BUF_WIDTH * (self.files_list.files[current_file_index].count_lines) as u32) {
            print!(
                "{}",
                self.files_list.files[current_file_index].content[j as usize] as char
            );
        }
    }

    fn edit_file_command(&mut self, argv: [u8; ARGV_SIZE]) {
        let current_file_index = self.get_file_index_by_name(argv);

        if current_file_index == DELETED_INDEX_FILE {
            print!(
                "\n[Error] File \"{}\" does not exist!",
                core::str::from_utf8(&argv.clone())
                    .unwrap()
                    .trim_matches('\0')
            );
            return;
        }

        self.is_editing_file = true;
        self.current_editing_file = current_file_index;
        self.files_list.files[self.current_editing_file].count_lines = 0;

        self.clear_command();
    }

    fn delete_file_command(&mut self, argv: [u8; ARGV_SIZE]) {
        self.clear_command();

        let current_file_index = self.get_file_index_by_name(argv);

        if current_file_index == DELETED_INDEX_FILE {
            print!(
                "\n[Error] File \"{}\" does not exist!",
                core::str::from_utf8(&argv.clone())
                    .unwrap()
                    .trim_matches('\0')
            );
            return;
        }

        self.files_list.files[current_file_index] = File {
            index: DELETED_INDEX_FILE,
            name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
            count_lines: 0,
            folder_index: DELETED_INDEX_DIRECTORY,
            content: [b' '; BUF_SIZE],
        };

        for i in 0..MAX_COUNT_FILES_IN_FOLDER {
            if self.directory_list.directories[self.current_directory].child_indexes[i]
                == current_file_index
            {
                self.directory_list.directories[self.current_directory].child_indexes[i] =
                    DELETED_INDEX_FILE;
            }
        }
    }

    fn create_file_command(&mut self, argv: [u8; ARGV_SIZE]) {
        let mut name_size = 0;
        let mut name = [b'\0'; MAX_SIZE_DIRECTORY_NAME];

        for i in 0..ARGV_SIZE {
            if argv[i] == b'\0' {
                break;
            }
            name[i] = argv[i];
            name_size += 1;
        }

        if name_size > MAX_SIZE_DIRECTORY_NAME {
            print!("\n[Error] The maximum size of the file name is 10 characters");
            return;
        }

        let mut file_index = DELETED_INDEX_FILE;

        for i in 0..MAX_COUNT_FILES {
            if self.files_list.files[i].index == DELETED_INDEX_FILE {
                file_index = i;
                break;
            }
        }

        if file_index == DELETED_INDEX_FILE {
            print!("\n[Error] There is not a free space!");
            return;
        }

        for i in 0..MAX_COUNT_FILES_IN_FOLDER {
            if self.files_list.files[i].index == DELETED_INDEX_FILE {
                file_index = i;
                break;
            }
        }

        let file = File {
            index: file_index,
            name: name,
            count_lines: 0,
            folder_index: self.current_directory,
            content: [b' '; BUF_SIZE],
        };

        self.is_editing_file = true;
        self.current_editing_file = file_index;

        SCREEN.lock().clear();

        self.files_list.files[file_index] = file;

        let mut index_for_folder = DELETED_INDEX_FILE;
        for i in 0..MAX_COUNT_FILES_IN_FOLDER {
            if self.directory_list.directories[self.current_directory].files_indexes[i]
                == DELETED_INDEX_FILE
            {
                index_for_folder = i;
            }
        }

        self.directory_list.directories[self.current_directory].files_indexes[index_for_folder] =
            file_index;
    }

    fn check_correct_dir_name(&mut self, dir_name: [u8; ARGV_SIZE]) -> bool {
        for i in 0..MAX_SIZE_DIRECTORY_NAME {
            if dir_name[i] != b'\0' {
                return true;
            }
        }

        return false;
    }

    fn delete_directory_command(&mut self, dir_name: [u8; ARGV_SIZE]) {
        let mut name_size = 0;

        for i in 0..ARGV_SIZE {
            if dir_name[i] == b'\0' {
                break;
            }
            name_size += 1;
        }

        if name_size == 0
        {
            print!("\n[Error] Specify a folder name!");
            return;
        }

        if name_size > MAX_SIZE_DIRECTORY_NAME {
            print!("\n[Error] The maximum size of the directory name is 10 characters");
            return;
        }

        let cur_dir = self.directory_list.directories[self.current_directory];

        let index_dir_to_delete = self.get_folder_index_by_name(dir_name);

        if index_dir_to_delete == DELETED_INDEX_DIRECTORY {
            print!(
                "\n[Error] Directory \"{}\" not found!",
                core::str::from_utf8(&dir_name).unwrap().trim_matches('\0')
            );
            return;
        }

        let dir_to_delete = self.directory_list.directories[self.current_directory].child_indexes[index_dir_to_delete];

        if self.directory_list.directories[dir_to_delete].child_count > 0 {
            print!("[Error] Count of childrens must be 0");
            return;
        }

        self.directory_list.directories[self.current_directory].child_count -= 1;

        self.directory_list.directories[dir_to_delete] = Directory {
            index: DELETED_INDEX_DIRECTORY,
            name: [b' '; MAX_SIZE_DIRECTORY_NAME],
            parent_index: DELETED_INDEX_DIRECTORY,
            child_count: DELETED_INDEX_DIRECTORY,
            child_indexes: [DELETED_INDEX_DIRECTORY; MAX_COUNT_CHILDREN_DIRECTORIES],
            files_indexes: [DELETED_INDEX_FILE; MAX_COUNT_FILES_IN_FOLDER],
        };

        self.directory_list.directories[cur_dir.index].child_indexes[index_dir_to_delete] = DELETED_INDEX_DIRECTORY;

        print!(
            "\n[Ok] Directory \"{}\" deleted succsessfully!",
            core::str::from_utf8(&dir_name.clone())
                .unwrap()
                .trim_matches('\0')
        );
    }

    fn change_directory_command(&mut self, argv: [u8; ARGV_SIZE]) {
        if argv[0] == b'.' {
            self.current_directory =
                self.directory_list.directories[self.current_directory].parent_index;
            return;
        }

        let cur_dir = self.directory_list.directories[self.current_directory];

        for dir_index in cur_dir.child_indexes {
            let mut is_same = true;

            if dir_index == DELETED_INDEX_DIRECTORY
            {
                continue;
            }

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
                self.current_directory = self.directory_list.directories[dir_index].index;
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
        for i in 0..MAX_COUNT_CHILDREN_DIRECTORIES {
            if current_directory.child_indexes[i] == DELETED_INDEX_DIRECTORY
            {
                continue;
            }

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

    fn search_free_index_child_indexes(&mut self) -> usize {
        for i in 0..MAX_COUNT_CHILDREN_DIRECTORIES {
            if self.directory_list.directories[self.current_directory].child_indexes[i]
                == DELETED_INDEX_DIRECTORY
            {
                return i;
            }
        }

        return DELETED_INDEX_DIRECTORY;
    }

    fn create_folder_command(&mut self, argv: [u8; ARGV_SIZE]) {
        let mut name_size = 0;
        for i in 0..ARGV_SIZE {
            if argv[i] == b'\0' {
                break;
            }
            name_size += 1;
        }

        if name_size == 0
        {
            print!("\n[Error] Specify a folder name!");
            return;
        }

        if name_size > MAX_SIZE_DIRECTORY_NAME {
            print!("\n[Error] The maximum size of the directory name is 10 characters");
            return;
        }

        if self.get_folder_index_by_name(argv) != DELETED_INDEX_DIRECTORY
        {
            print!("\n[Error] Directory \"{}\" already exist", core::str::from_utf8(&argv.clone())
            .unwrap()
            .trim_matches('\0'));
            return;
        }

        let mut dir_index = DELETED_INDEX_DIRECTORY;

        for i in 0..MAX_COUNT_DIRECTORIES {
            if self.directory_list.directories[i].index == DELETED_INDEX_DIRECTORY {
                dir_index = i;
                break;
            }
        }

        if dir_index == DELETED_INDEX_DIRECTORY {
            print!("\n[Error] There is not a free space!");
            return;
        }

        let free_index = self.search_free_index_child_indexes();

        if free_index == DELETED_INDEX_DIRECTORY {
            print!("\n[Error] There is not a free space!");
            return;
        }

        let mut directory: Directory = Directory {
            index: dir_index,
            name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
            parent_index: self.current_directory,
            child_count: 0,
            child_indexes: [DELETED_INDEX_DIRECTORY; MAX_COUNT_CHILDREN_DIRECTORIES],
            files_indexes: [DELETED_INDEX_FILE; MAX_COUNT_FILES_IN_FOLDER],
        };

        for i in 0..MAX_SIZE_DIRECTORY_NAME {
            directory.name[i] = argv[i];
        }

        self.directory_list.directories[dir_index] = directory;
        self.directory_list.directories[self.current_directory].child_indexes[free_index] =
            dir_index;

        self.directory_list.directories[self.current_directory].child_count += 1;

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
                    index: DELETED_INDEX_DIRECTORY,
                    name: [b' '; MAX_SIZE_DIRECTORY_NAME],
                    parent_index: 0,
                    child_count: 0,
                    child_indexes: [DELETED_INDEX_DIRECTORY; MAX_COUNT_CHILDREN_DIRECTORIES],
                    files_indexes: [DELETED_INDEX_FILE; MAX_COUNT_FILES_IN_FOLDER],
                }; MAX_COUNT_DIRECTORIES]),
            },
            current_directory: 0,
            files_list: FileList {
                files: [File {
                    index: DELETED_INDEX_FILE,
                    name: [b'\0'; MAX_SIZE_DIRECTORY_NAME],
                    count_lines: 0,
                    folder_index: DELETED_INDEX_DIRECTORY,
                    content: [b' '; BUF_SIZE],
                }; MAX_COUNT_FILES],
            },
            is_editing_file: false,
            current_editing_file: DELETED_INDEX_FILE,
        };

        let root_directory = Directory {
            index: 0,
            name: [
                b'r', b'o', b'o', b't', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0',
            ],
            parent_index: 0,
            child_count: 0,
            child_indexes: [DELETED_INDEX_DIRECTORY; MAX_COUNT_CHILDREN_DIRECTORIES],
            files_indexes: [DELETED_INDEX_FILE; MAX_COUNT_FILES_IN_FOLDER],
        };

        shell.directory_list.directories[0] = root_directory;

        return shell;
    }

    pub fn on_key_pressed(&mut self, key: u8) {
        match key {
            b'\n' => {
                if self.is_editing_file {
                    self.files_list.files[self.current_editing_file].count_lines += 1;
                    println!();
                    return;
                }

                let argv = parameters_splitter(self.buf, self.buf_len);

                self.execute_command(argv);
                self.buf_len = 0;

                if self.is_editing_file {
                    return;
                }
                println!();
                good_formatting();
            }
            37 =>
                // key code arrow left
                {}
            38 =>
                // key code arrow up
                {}
            39 =>
                // key code arrow right
                {}
            40 =>
                // key code arrow down
                {}
            8 =>
            // key code of backspace
            {
                if self.is_editing_file {
                    SCREEN.lock().delete_last_symbol(0);
                    return;
                }

                SCREEN.lock().delete_last_symbol(FORMATING_STRING_LENGTH);

                if self.buf_len > 0 {
                    self.buf_len -= 1;
                }

                self.buf[self.buf_len] = 0;
            }
            32 =>
            // key code of space button
            {
                print!("{}", key as char);

                if self.is_editing_file {
                    return;
                }

                self.buf[self.buf_len] = b' ';
                self.buf_len += 1;
            }
            9 =>
            // tab key
            {
                if self.is_editing_file {
                    self.is_editing_file = false;
                    self.files_list.files[self.current_editing_file].count_lines += 1;

                    self.files_list.files[self.current_editing_file].content =
                        SCREEN.lock().get_buffer();

                    self.clear_command();
                    print!(
                        "\n[Ok] File \"{}\" saved succsessfully!\n",
                        core::str::from_utf8(
                            &self.files_list.files[self.current_editing_file]
                                .name
                                .clone()
                        )
                        .unwrap()
                        .trim_matches('\0')
                    );
                    good_formatting();
                }
            }
            _ => {
                if self.is_editing_file {
                    print!("{}", key as char);
                    return;
                }

                self.buf[self.buf_len] = key;
                self.buf_len += 1;
                print!("{}", key as char);
            }
        }
    }
}
