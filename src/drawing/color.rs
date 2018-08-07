/// Only for the B/W Displays atm
pub enum Color {
    Black,
    White,
}

impl Color {
    /// Get the color encoding of the color for one bit
    pub fn get_bit_value(&self) -> u8 {
        match self {
            Color::White => 1u8,
            Color::Black => 0u8,
        }
    }

    /// Gets a full byte of black or white pixels
    pub fn get_byte_value(&self) -> u8 {
        match self {
            Color::White => 0xff,
            Color::Black => 0x00,
        }
    }

    /// Get the color encoding of a specific bit in a byte
    ///
    /// input is the byte where one bit is gonna be selected
    /// pos is counted from the left (highest value) from 0 to 7
    /// remember: 1 is white, 0 is black
    /// Color is the color you want to draw with in the foreground
    pub(crate) fn get_color(input: u8, pos: u8, color: &Color) -> Color {
        if Color::is_drawable_pixel(input, pos) {
            Color::normal_color(color)
        } else {
            Color::inverse_color(color)
        }
    }

    // Inverses the given color from Black to White or from White to Black
    fn inverse_color(color: &Color) -> Color {
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    // Gives you a new owned copy of the color
    //TODO: just use clone?
    fn normal_color(color: &Color) -> Color {
        match color {
            Color::White => Color::White,
            Color::Black => Color::Black,
        }
    }

    //position counted from the left (highest value) from 0 to 7
    //remember: 1 is white, 0 is black
    pub(crate) fn is_drawable_pixel(input: u8, pos: u8) -> bool {
        ((input >> (7 - pos)) & 1u8) > 0u8
    }

    //TODO: does basically the same as get_color, so remove one of them?
    pub(crate) fn convert_color(input: u8, pos: u8, foreground_color: &Color) -> Color {
        //match color:
        //      - white for "nothing to draw"/background drawing
        //      - black for pixel to draw
        //
        //foreground color is the color you want to have in the foreground
        if Color::is_drawable_pixel(input, pos) {
            Color::normal_color(foreground_color)
        } else {
            Color::inverse_color(foreground_color)
        }
    }
}
