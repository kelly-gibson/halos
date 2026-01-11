#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// An enum representing VGA colors
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
// A transparent wrapper around a u8 used to combine foreground and background colors into a single ColorCode
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Text buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
// Represents a character on the screen with and ASCII Character (code page 41) and a color code
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;
#[repr(transparent)]
// A wrapper around a 2 dimensional array of Volatile<ScreenChar> representing the VGA text buffer
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Represents a writer for the VGA buffer with a current column position, color code, and mutable reference to a buffer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // Write byte method
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'/' => self.new_line(),
            byte => {
                if self.column_position>= BUFFER_WIDTH {
                    self.new_line();
                }

                let row =  BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    // New line method
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT{
            for col in 0..BUFFER_WIDTH{
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    // This method clears a row by overwriting all of its characters with a space character
    fn clear_row(&mut self, row: usize) { 
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
    // Write String method
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Attempt to make a global Writer that can be used as an interface in other modules
use lazy_static::lazy_static;
use spin::Once;
use x86_64::instructions::interrupts;

lazy_static! {
    static ref WRITER_INITIALIZED: Once = Once::new();
    pub static ref WRITER: Writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightRed, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
}

// Function to perform a critical section with the WRITER
// takes a closure f, which operates on a Writer.
pub fn with_writer<F, R>(f: F) -> R

    where
        F: FnOnce(&mut Writer) -> R,
    {
        // Disable interrupts to create a critical section
        interrupts::without_interrupts(|| {

        // Ensure that WRITER has been initialized
        WRITER_INITIALIZED.call_once(|| {});

        // Create a mutable Writer instance within the critical section
        let mut writer_instance = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };

        // Obtain a mutable reference to the Writer instance
        let writer_ref = &mut writer_instance;

        // Invoke the closure with the mutable reference
        f(writer_ref)
    })
}

// Perfroms write operations on the global WRITER
pub fn example_global_writer() {
    use core::fmt::Write;
    with_writer(|writer| {
        // Perform write operations using the writer
        write!(writer, "The numbers are {} and {}", 54, 1.0 / 3.0).unwrap();
    });
}
