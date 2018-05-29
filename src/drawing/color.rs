

pub enum Color {
    Black,
    White
}

impl Color {
    pub fn get_bit_value(&self) -> u8 {
        match self {
            Color::White => 1u8,
            Color::Black => 0u8,            
        }
    }

    pub fn get_byte_value(&self) -> u8 {
        match self {
            Color::White => 0xff,
            Color::Black => 0x00,
        }
    }

    //position counted from the left (highest value) from 0 to 7
    //remember: 1 is white, 0 is black
    pub(crate) fn get_color(input: u8, pos: u8, color: &Color) -> Color {
        match Color::is_drawable_pixel(input, pos) {
            true    => Color::normal_color(color),
            false   => Color::inverse_color(color)
        }
    }

    fn inverse_color(color: &Color) -> Color {
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    
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


    pub(crate) fn convert_color(input: u8, pos: u8, foreground_color: &Color) -> Color {
        //match color: 
        //      - white for "nothing to draw"/background drawing
        //      - black for pixel to draw
        //
        //foreground color is the color you want to have in the foreground
        match Color::is_drawable_pixel(input, pos) {
            true => Color::normal_color(foreground_color),
            false => Color::inverse_color(foreground_color)
        }
    }
}