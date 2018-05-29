
pub mod font;
use self::font::Font;

pub mod color;
use self::color::Color;


#[derive(Clone, Copy)]
pub enum Displayorientation {
    /// No rotation
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

//WARNING: Adapt for bigger sized displays!
// pub struct DisplayDescription {
//     width: u16,
//     height: u16,
//     buffer_size: u16
// }

// impl Display_Description {
//     pub fn new(width: u16, height: u16, buffer_size: u16) -> Display_Description {

//     }
// }

pub enum Display {
    Eink42BlackWhite,
}

impl Display {
    /// Gets the Dimensions of a dipslay in the following order:
    /// - Width
    /// - Height
    /// - Neccessary Buffersize
    pub fn get_dimensions(&self) -> (u16, u16, u16) {
        match self {
            Display::Eink42BlackWhite => (400, 300, 15000)
        }
    }
}



#[allow(dead_code)]
pub struct Graphics {
    width: u16,
    height: u16,
    rotation: Displayorientation,
    //buffer: Box<u8>//[u8; 15000],   
}

impl Graphics {
    /// width needs to be a multiple of 8!
    pub fn new(width: u16, height: u16) -> Graphics{
        Graphics {width, height, rotation: Displayorientation::Rotate0}
    }

    /// Clears/Fills the full buffer with `color`
    pub fn clear(&self, buffer: &mut[u8], color: &Color) {
        for elem in buffer.iter_mut() {
            *elem = color.get_byte_value();
        }
    }

    /// Draw a single Pixel with `color`
    /// 
    /// limited to i16::max images (buffer_size) at the moment
    pub fn draw_pixel(&self, buffer: &mut[u8], x: u16, y: u16, color: &Color) {
        let (idx, bit) = match self.rotation {
            Displayorientation::Rotate0 | Displayorientation::Rotate180 
                => ((x as usize / 8 + (self.width as usize / 8) * y as usize) ,
                    0x80 >> (x % 8)),
            Displayorientation::Rotate90 | Displayorientation::Rotate270
                => (y as usize / 8 * self.width as usize + x as usize,
                    0x80 >> (y % 8)),
        };

        if idx >= buffer.len() {
            return;
        }

        match color {
            Color::Black => {
                buffer[idx] &= !bit; 
            },
            Color::White => {
                buffer[idx] |= bit;
            }
        }
    }

    /// Draw a single Pixel with `color`
    /// 
    /// limited to i16::max images (buffer_size) at the moment
    #[allow(dead_code)]
    fn draw_byte(&self, buffer: &mut[u8], x: u16, y: u16, filling: u8, color: &Color) {
        let idx = match self.rotation {
            Displayorientation::Rotate0 | Displayorientation::Rotate180 
                => x as usize / 8 + (self.width as usize / 8) * y as usize,
            Displayorientation::Rotate90 | Displayorientation::Rotate270
                => y as usize / 8 + (self.width as usize / 8) * x as usize,
        };

        if idx >= buffer.len() {
            return;
        }

        match color {
            Color::Black => {
                buffer[idx] = !filling; 
            },
            Color::White => {
                buffer[idx] = filling;
            }
        }
    }

    ///TODO: test!
    pub fn draw_char(&self, buffer: &mut[u8], x0: u16, y0: u16, input: char, font: &Font, color: &Color) {
        self.draw_char_helper(buffer, x0, y0, input, font, color);
    }

    ///TODO: test!
    /// no autobreak line yet
    pub fn draw_string(&self, buffer: &mut[u8], x0: u16, y0: u16, input: &str, font: &Font, color: &Color) {
        let mut counter = 0;
        for input_char in input.chars() {
            self.draw_char(buffer, x0 + counter, y0, input_char, font, color);
            counter += font.get_char_width(input_char) as u16;
        }
    }

    
    //TODO: add support for font_height = 0
    //TODO: add support for char offset in y direction to reduce font file size
    fn draw_char_helper(&self, buffer: &mut[u8], x0: u16, y0: u16, input: char, font: &Font, color: &Color) {
        //width: u8, height: u8, charbuffer: &[u8]
        //TODO: font.get_char(input) -> FontChar {width, height, [u8]}
        //TODO: font.get_char_offset(input) -> u16

        let buff = font.get_char(input);
        let char_width = font.get_char_width(input);

        
        let mut row_counter = 0;
        let mut width_counter = 0u8;
        for &elem in buff.iter() {
            for _ in 0..8 {

                self.draw_pixel(
                    buffer, 
                    x0 + width_counter as u16, 
                    y0 + row_counter, 
                    &Color::get_color(elem, width_counter % 8, color));

                //Widthcounter shows how far we are in x direction 
                width_counter += 1;
                // if we have reached
                if width_counter >= char_width {
                    width_counter = 0;
                    row_counter += 1;
                    break;
                }
            }
        }
    }

    /// Draws a single 8x8 Char somewhere (1 pixel padding included)
    pub fn draw_char_8x8(&self, buffer: &mut[u8], x0: u16, y0: u16, input: char, color: &Color) {
        let mut counter = 0;
        // includes special draw_char instructions as this one is ordered columnwise and not rowwise (first byte == first 8 pixel columnwise)
        for &elem in font::bitmap_8x8(input).iter() {
            for i in 0..8u8 {
                self.draw_pixel(buffer, x0 + counter, y0 + 7 - i as u16, &Color::convert_color(elem, i, color))
            }
            counter += 1;
        }
    }

    /// Draws Strings with 8x8 Chars (1 pixel padding included)
    /// 
    /// Is quite small for the 400x300 E-Ink
    /// 
    /// no autobreak line yet
    pub fn draw_string_8x8(&self, buffer: &mut[u8], x0: u16, y0: u16, input: &str, color: &Color) {
        let mut counter = 0;
        for input_char in input.chars() {
            self.draw_char_8x8(buffer, x0 + counter*8, y0, input_char, color);
            counter += 1;
        }
    }

//     void plotLine(int x0, int y0, int x1, int y1)
// {
//    int dx =  abs(x1-x0), sx = x0<x1 ? 1 : -1;
//    int dy = -abs(y1-y0), sy = y0<y1 ? 1 : -1; 
//    int err = dx+dy, e2; /* error value e_xy */
 
//    for(;;){  /* loop */
//       setPixel(x0,y0);
//       if (x0==x1 && y0==y1) break;
//       e2 = 2*err;
//       if (e2 >= dy) { err += dy; x0 += sx; } /* e_xy+e_x > 0 */
//       if (e2 <= dx) { err += dx; y0 += sy; } /* e_xy+e_y < 0 */
//    }
// }
    //bresenham algorithm for lines
    /// draw line 
    pub fn draw_line(&self, buffer: &mut[u8], x0: u16, y0: u16, x1: u16, y1: u16, color: &Color) {
        let mut x0 = x0 as i16;
        let x1 = x1 as i16;
        let mut y0 = y0 as i16;
        let y1 = y1 as i16;

        let dx = i16::abs(x1 - x0);
        let sx = if x0 < x1 { 1 } else { -1 };

        let dy = - i16::abs(y1 - y0);
        let sy = if y0 < y1 { 1 } else { -1 };
        
        let mut err = dx + dy;

        loop {
            self.draw_pixel(buffer, x0 as u16, y0 as u16, color);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2*err;

            if e2 >= dy {
                err += dy;
                x0 += sx;
            }

            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a horizontal line 
    /// TODO: maybe optimize by grouping up the bytes? But is it worth the longer and more complicated function? is it even faster?
    pub fn draw_horizontal_line(&self, buffer: &mut[u8], x: u16, y: u16, length: u16, color: &Color) {
        for i in 0..length {
            self.draw_pixel(buffer, x + i, y, color);
        }
    }

    /// Draws a vertical line
    pub fn draw_vertical_line(&self, buffer: &mut[u8], x: u16, y: u16, length: u16, color: &Color) {
        for i in 0..length {
            self.draw_pixel(buffer, x, y + i, color);
        }
    }

    /// Draws a rectangle. (x0,y0) is top left corner, (x1,y1) bottom right
    pub fn draw_rectangle(&self, buffer: &mut[u8], x0: u16, y0: u16, x1: u16, y1: u16, color: &Color) {
        let (min_x, max_x) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let (min_y, max_y) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        let x_len = max_x - min_x;
        let y_len = max_y - min_y;
        self.draw_horizontal_line(buffer, min_x, min_y, x_len, color);
        self.draw_horizontal_line(buffer, min_x, max_y, x_len, color);
        self.draw_vertical_line(buffer, min_x, min_y, y_len, color);
        self.draw_vertical_line(buffer, max_x, min_y, y_len, color);
    }

    /// Draws a filled rectangle. For more info see draw_rectangle
    pub fn draw_filled_rectangle(&self, buffer: &mut[u8], x0: u16, y0: u16, x1: u16, y1: u16, color: &Color) {
        let (min_x, max_x) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let (min_y, max_y) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        let x_len = max_x - min_x;
        let y_len = max_y - min_y;
        for i in 0..y_len {
            self.draw_horizontal_line(buffer, min_x, min_y + i, x_len, color);
        }
    }

    fn draw_pixel_helper(&self, buffer: &mut[u8], x: i16, y: i16, color: &Color) {
        if x >= 0 && y >= 0 {
            self.draw_pixel(buffer, x as u16, y as u16, color);
        }
    }


fn draw_circle_helper(&self, buffer: &mut[u8], x0: u16, y0: u16, radius: u16, filled: bool, color: &Color) {
    let mut x = radius - 1;
    let mut y = 0;
    let mut dx = 1;
    let mut dy = 1;
    let mut err: i16 = dx - 2 * radius as i16;

    while x >= y {
        if filled {
            self.circle_helper_filled_putpixel(buffer, x0, y0, x, y, color);
        } else {
            self.circle_helper_putpixel(buffer, x0, y0, x, y, color);
        }

        if err <= 0 {
            y += 1;
            err += dy;
            dy += 2;
        }

        if err > 0 {
            x -= 1;
            dx += 2;
            err += dx - 2 * radius as i16;
        }
    }

}

fn circle_helper_putpixel(&self, buffer: &mut[u8], x0: u16, y0: u16,  x: u16, y: u16, color: &Color) {
    self.draw_horizontal_line(buffer, x0 - x, y0 + y, 2*x, color);
    // self.draw_pixel(buffer, x0 + x, y0 + y, color);
    // self.draw_pixel(buffer, x0 - x, y0 + y, color);

    self.draw_horizontal_line(buffer, x0 - y, y0 + x, 2*y, color);
    // self.draw_pixel(buffer, x0 + y, y0 + x, color);
    // self.draw_pixel(buffer, x0 - y, y0 + x, color);
    
    self.draw_horizontal_line(buffer, x0 - x, y0 - y, 2*x, color);
    // self.draw_pixel(buffer, x0 - x, y0 - y, color);
    // self.draw_pixel(buffer, x0 + x, y0 - y, color);

    self.draw_horizontal_line(buffer, x0 - y, y0 - y, 2*y, color);
    // self.draw_pixel(buffer, x0 - y, y0 - x, color);
    // self.draw_pixel(buffer, x0 + y, y0 - x, color);
    
}

//TODO: Test
fn circle_helper_filled_putpixel(&self, buffer: &mut[u8], x0: u16, y0: u16,  x: u16, y: u16, color: &Color) {
    self.draw_pixel(buffer, x0 + x, y0 + y, color);
    self.draw_pixel(buffer, x0 + y, y0 + x, color);
    self.draw_pixel(buffer, x0 - y, y0 + x, color);
    self.draw_pixel(buffer, x0 - x, y0 + y, color);
    self.draw_pixel(buffer, x0 - x, y0 - y, color);
    self.draw_pixel(buffer, x0 - y, y0 - x, color);
    self.draw_pixel(buffer, x0 + y, y0 - x, color);
    self.draw_pixel(buffer, x0 + x, y0 - y, color);
}




    ///TODO: test if circle looks good
    /// Draws a circle
    pub fn draw_circle(&self, buffer: &mut[u8], x0: u16, y0: u16, radius: u16, color: &Color) {
        self.draw_circle_helper(buffer, x0, y0, radius, false, color);
    }

    ///TODO: test if circle looks good
    /// Draws a circle
    pub fn draw_circle2(&self, buffer: &mut[u8], x: u16, y: u16, radius: u16, color: &Color) {
        let radius = radius as i16;
        let x_mid = x as i16;
        let y_mid = y as i16;
        let mut x_pos: i16 = 0 - radius; 
        let mut y_pos = 0;
        let mut err: i16 = 2 - 2 * radius;

        loop {
            self.draw_pixel_helper(buffer, x_mid - x_pos, y_mid + y_pos, color);
            self.draw_pixel_helper(buffer, x_mid - y_pos, y_mid - x_pos, color);
            self.draw_pixel_helper(buffer, x_mid + x_pos, y_mid - y_pos, color);
            self.draw_pixel_helper(buffer, x_mid + y_pos, y_mid + x_pos, color);

            let radius = err;

            if radius <= y_pos {
                y_pos += 1;
                err += y_pos*2 + 1;
            }

            if radius > x_pos || err > y_pos {
                x_pos += 1;
                err += x_pos*2 + 1;
            }

            if x_pos >= 0 {
                break;
            }
        }
    }


    ///TODO: test!
    pub fn draw_filled_circle(&self, buffer: &mut[u8], x0: u16, y0: u16, radius: u16, color: &Color) {
        self.draw_circle_helper(buffer, x0, y0, radius, true, color);
    }

    
}


/*

############   ############  ############  ############
    ##         ##            #                 ##      
    ##         ##            #                 ##      
    ##         ######         #####            ##      
    ##         ######              #####       ##      
    ##         ##                       #      ##      
    ##         ##                       #      ##      
    ##         ############  ############      ##      

*/



#[cfg(test)]
mod graphics {
    use super::*;

    #[test]
    fn test_filled_rectangle() {
        let mut buffer = [Color::White.get_byte_value(); 150];
        let graphics = Graphics::new(40, 30);
        graphics.draw_filled_rectangle(&mut buffer, 0, 0, 40, 30, &Color::Black);
        
        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &elem in buffer.iter() {
            
            assert_eq!(elem, Color::Black.get_byte_value());
        }

        
    }

    /// draw a 4x4 in the top left corner
    #[test]
    fn test_filled_rectangle2() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_filled_rectangle(&mut buffer, 0, 0, 4, 4, &Color::Black);
        
        assert_eq!(buffer[0], 0x0f);

        let mut counter = 0;
        for &elem in buffer.iter() {
            counter += 1;           

            if counter <= 4 {
                assert_eq!(elem, 0x0f);
            } else {
                assert_eq!(elem, Color::White.get_byte_value());
            }
        }

        
    }

    #[test]
    fn test_horizontal_line() {
        let mut buffer = [Color::White.get_byte_value(); 4];
        let graphics = Graphics::new(16, 2);
        graphics.draw_horizontal_line(&mut buffer, 1, 0, 14, &Color::Black);
        
        assert_eq!(buffer[0], 0x80);
        assert_eq!(buffer[1], 0x01);
        assert_eq!(buffer[2], Color::White.get_byte_value());
        assert_eq!(buffer[3], Color::White.get_byte_value());
    }

    #[test]
    fn test_vertical_line() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_vertical_line(&mut buffer, 0, 0, 8, &Color::Black);

        graphics.draw_vertical_line(&mut buffer, 5, 0, 8, &Color::Black);
        
        
        assert_eq!(buffer[0], 0x7b);

        for &elem in buffer.iter() {
            
            assert_eq!(elem, 0x7bu8);
        }
    }

    //test draw_line for compatibility with draw_vertical_line
    #[test]
    fn draw_line_1() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);

        graphics.draw_vertical_line(&mut buffer, 5, 0, 8, &Color::Black);

        let mut buffer2 = [Color::White.get_byte_value(); 8];
        let graphics2 = Graphics::new(8, 8);

        graphics2.draw_line(&mut buffer2, 5, 0, 5, 8, &Color::Black);       

        for i in 0..buffer.len() {            
            assert_eq!(buffer[i], buffer2[i]);
        }
    }

    //test draw_line for compatibility with draw_horizontal_line
    #[test]
    fn draw_line_2() {
        let mut buffer = [Color::White.get_byte_value(); 4];
        let graphics = Graphics::new(16, 2);
        graphics.draw_horizontal_line(&mut buffer, 1, 0, 14, &Color::Black);

        let mut buffer2 = [Color::White.get_byte_value(); 4];
        let graphics2 = Graphics::new(16, 2);
        graphics2.draw_line(&mut buffer2, 1, 0, 14, 0, &Color::Black);       

        for i in 0..buffer.len() {            
            assert_eq!(buffer[i], buffer2[i]);
        }
    }

    //test draw_line for diago
    #[test]
    fn draw_line_3() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);

        graphics.draw_line(&mut buffer, 0, 0, 16, 16, &Color::Black);       

        for i in 0..buffer.len() {            
            assert_eq!(buffer[i], !(0x80 >> i % 8));
        }
    }



    #[test]
    fn test_pixel() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_pixel(&mut buffer, 1, 0, &Color::Black);

        assert_eq!(buffer[0], !0x40);


        let mut buffer = [Color::White.get_byte_value(); 16];
        let graphics = Graphics::new(16, 8);
        graphics.draw_pixel(&mut buffer, 9, 0, &Color::Black);
        assert_eq!(buffer[0], Color::White.get_byte_value());
        assert_eq!(buffer[1], !0x40);
    }

    #[test]
    fn test_byte() {
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_byte(&mut buffer, 0, 0, 0xff, &Color::Black);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for i in 1..buffer.len() {
            assert_eq!(buffer[i], Color::White.get_byte_value());
        } 

        graphics.draw_byte(&mut buffer, 0, 0, 0x5A, &Color::Black)  ;
        assert_eq!(buffer[0], !0x5A);
    }

    #[test]
    fn test_char_with_8x8_font() {

        // Test !
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_char_8x8(&mut buffer, 0, 0, '!', &Color::Black);

        for i in 0..5 {
            assert_eq!(buffer[i], !0x20);
        }
        assert_eq!(buffer[5], Color::White.get_byte_value());
        assert_eq!(buffer[6], !0x20);
        assert_eq!(buffer[7], Color::White.get_byte_value());  


        // Test H
        let mut buffer = [Color::White.get_byte_value(); 8];
        let graphics = Graphics::new(8, 8);
        graphics.draw_char_8x8(&mut buffer, 0, 0, 'H', &Color::Black);

        for i in 0..3 {
            assert_eq!(buffer[i], !0x88);
        }
        assert_eq!(buffer[3], !0xF8);        
        for i in 4..7 {
            assert_eq!(buffer[i], !0x88);
        }
        assert_eq!(buffer[7], Color::White.get_byte_value());  
    }

    #[test]
    fn test_string_with_8x8_font() {

        // Test !H
        let mut buffer = [Color::White.get_byte_value(); 16];
        let graphics = Graphics::new(16, 8);
        graphics.draw_string_8x8(&mut buffer, 0, 0, "!H", &Color::Black);

        for i in 0..5 {
            assert_eq!(buffer[i*2], !0x20);
        }
        assert_eq!(buffer[5*2], Color::White.get_byte_value());
        assert_eq!(buffer[6*2], !0x20);
        assert_eq!(buffer[7*2], Color::White.get_byte_value());  


        for i in 0..3 {
            assert_eq!(buffer[i*2 + 1], !0x88);
        }
        assert_eq!(buffer[3*2 + 1], !0xF8);        
        for i in 4..7 {
            assert_eq!(buffer[i*2 + 1], !0x88);
        }
        assert_eq!(buffer[7*2 + 1], Color::White.get_byte_value());
    }
}