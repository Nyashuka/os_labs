#![no_main]

use crate::vga_buf::*;

pub const HEIGHT: usize = 25;
pub const WIDTH: usize = 80;

const MAP: [&str; 25] = [
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                    x                                           ",
    "                                  x x                                           ",
    "                        xx      xx            xx                                ",
    "                       x   x    xx            xx                                ",
    "            xx        x     x   xx                                              ",
    "            xx        x   x xx    x x                                           ",
    "                      x     x       x                                           ",
    "                       x   x                                                    ",
    "                        xx                                                      ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                "
];

pub fn sleep()
{
    let mut number = 2.24632767;
    for i in 0..50000{
        number *= number;
    }
}

pub fn render(game_field: &[[u8; 80]; 25], vga_driver: &mut VGADriver)
{
    for i in 0..game_field.len()
    {
        for j in 0..game_field[0].len()
        {
            vga_driver.write_byte_char((i * 80 + j) as u32, game_field[i][j]);
        }
    }
}

pub fn get_count_nearest_cells(game_field: &[[u8; 80]; 25], i: usize, j: usize) -> u32
{
    let mut count: u32 = 0;

    if i + 1 < HEIGHT && j + 1 < WIDTH && game_field[i + 1][j + 1] == b'x'
    {
        count += 1;
    }
    if i + 1 < HEIGHT && j > 0 && game_field[i + 1][j - 1] == b'x'
    {
        count += 1;
    }
    if i > 0 && j > 0 && game_field[i - 1][j - 1] == b'x'
    {
        count += 1;
    }
    if i > 0 && j + 1 < WIDTH && game_field[i - 1][j + 1] == b'x'
    {
        count += 1;
    }
    if i > 0 && game_field[i - 1][j] == b'x'
    {
        count += 1;
    }
    if i + 1 < HEIGHT && game_field[i + 1][j] == b'x'
    {
        count += 1;
    }
    if j + 1 < WIDTH && game_field[i][j + 1] == b'x'
    {
        count += 1;
    }
    if j > 0 && game_field[i][j - 1] == b'x'
    {
        count += 1;
    }
    return count;
}

pub fn game_of_life(vga_driver: &mut VGADriver)
{
    let mut current_gen: [[u8; 80]; 25] = [[0; 80]; 25];
    for i in 0..MAP.len()
    {
        for (j, byte) in MAP[i].bytes().enumerate()
        {
            current_gen[i][j] = byte;
        }
    }
    render(&current_gen, vga_driver);

    // TODO: implement game of life
    let mut count: u32 = 0;
    loop {
        sleep();
        let mut evolution: [[u8; 80]; 25] = [[0; 80]; 25];

        for i in 0..current_gen.len()
        {
            for j in 0..current_gen[0].len()
            {
                let count_nearest_cells: u32 = get_count_nearest_cells(&current_gen, i, j);

                if current_gen[i][j] == b'x' && (count_nearest_cells == 3 || count_nearest_cells == 2)
                {
                    evolution[i][j] = b'x';
                } else if current_gen[i][j] == b' ' && count_nearest_cells == 3
                {
                    evolution[i][j] = b'x';
                } else {
                    evolution[i][j] = b' ';
                }
            }
        }
        //evolution[0][0] = b'u';
        current_gen = evolution;
        render(&current_gen, vga_driver);
    }
}

