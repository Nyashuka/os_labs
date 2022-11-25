const BUFFER_ADDRESS: u32 = 0xb8000;
const WINDOW_WIDTH: u32 = 80;
const WINDOW_HEIGHT: u32 = 25;
const DEFAULT_COLOR: u8 = 0x0;

pub struct AsciiSymbol
{
    pub char_byte: u8,
    pub foreground_color_byte: u8,
    pub background_color_byte: u8,
}

pub enum Alignment 
{
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy)]
pub enum Color
{
    Black = 0x0,
    Purple = 0x5,
    White = 0xf,
    Pink = 0xd
}

pub struct VGADriver 
{
    // public
    pub buffer: *mut u8,
    pub width: *mut u32,
    pub height: *mut u32,
    pub foreground_color: u8,
    pub background_color: u8,
    pub alignment: Alignment,

    // private
    current_column: u32,
    current_row: u32,
}


pub fn calculate_alignment(align: &Alignment) -> u32
    {
        return match align 
        {
            Alignment::Left => 0,
            Alignment::Right => WINDOW_WIDTH - 1,
            Alignment::Center => 0
        };
    }


impl VGADriver 
{
    pub fn new(fr_color: Color, bg_color: Color ,align: Alignment) -> VGADriver
    {
        return VGADriver
        {
            buffer : BUFFER_ADDRESS as *mut u8,
            foreground_color: fr_color as u8,
            background_color: bg_color as u8,
            current_column: calculate_alignment(&align),
            alignment: align,
            current_row: 0,
            width: 80 as *mut u32,
            height: 25 as *mut u32,
        };
    }

    

    pub fn write_char(&self, offset: u32, symbol: AsciiSymbol ) 
    {
        unsafe 
        {
            *self.buffer.offset(offset as isize * 2 ) = symbol.char_byte;
            *self.buffer.offset(offset as isize * 2 + 1) = ((symbol.background_color_byte) << 4) | (symbol.foreground_color_byte);
        }
    }

    pub fn write_byte_char(&self, offset: u32, char: u8) {
        unsafe
        {
            *self.buffer.offset(offset as isize * 2) = char;
            *self.buffer.offset(offset as isize * 2 + 1) = ((self.background_color) << 4) | (self.foreground_color);
        }
    }

    pub fn read_char(&self, offset: u32) -> AsciiSymbol 
    {
        unsafe 
        {
            return AsciiSymbol 
            { 
                char_byte: *self.buffer.offset(offset as isize * 2 ), 
                foreground_color_byte: *self.buffer.offset(offset as isize * 2 + 1),
                background_color_byte: (*self.buffer.offset(offset as isize * 2 + 1) << 4)
            };
        }
    }

    pub fn move_row(&self)
    {       
            for i in 1..WINDOW_HEIGHT
            {
                for j in 0..WINDOW_WIDTH
                {
                    let symbol = self.read_char(i * WINDOW_WIDTH + j);        
                    self.write_char((i-1) * WINDOW_WIDTH + j, symbol);   
                }

            }
            for i in 0..WINDOW_WIDTH 
            {
                self.write_char((WINDOW_HEIGHT - 1) * WINDOW_WIDTH + i, 
                                AsciiSymbol { char_byte: (b' '), foreground_color_byte: (DEFAULT_COLOR),
                                                    background_color_byte: (DEFAULT_COLOR) });     
            }
    }

    pub fn move_char_to_left(&self)
    {
        for i in 1..WINDOW_WIDTH
        {

            let symbol = self.read_char(self.current_row * WINDOW_WIDTH + i);  

            self.write_char(self.current_row * WINDOW_WIDTH + i - 1, symbol);
        }
        
        
    }


    pub fn print(&mut self, s: &str)
    {
        for symbol in s.bytes()
        {
            let symbol_to_print = AsciiSymbol { char_byte: (symbol), foreground_color_byte: (self.foreground_color),
                                                             background_color_byte: (self.background_color) };

            if symbol == b'\n'
            {
                if self.current_row < WINDOW_HEIGHT - 1
                {
                    self.current_row += 1;
                }
                else
                {
                    self.move_row();
                }
                self.current_column = calculate_alignment(&self.alignment);
            }
            else
            {
                match self.alignment 
                {
                    Alignment::Left =>
                    {
                        self.write_char(self.current_row * WINDOW_WIDTH + self.current_column, symbol_to_print);
                        self.current_column += 1;
                    }
                    Alignment::Right =>
                    {
                        self.move_char_to_left();
                        self.write_char(self.current_row * WINDOW_WIDTH + self.current_column, symbol_to_print);
                    }
                    Alignment::Center =>
                    {
                        if self.current_column % 2 == 1
                        {
                            self.move_char_to_left();
                        }
                        self.write_char(self.current_row * WINDOW_WIDTH + WINDOW_WIDTH / 2 + self.current_column / 2, symbol_to_print);
                        self.current_column += 1;
                    }
                }
            }
        }
        
    }

}

impl core::fmt::Write for VGADriver
{
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            self.print(s);
            Ok(())
        }
}