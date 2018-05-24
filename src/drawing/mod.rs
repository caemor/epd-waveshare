
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
/*
//WARNING: Adapt for bigger sized displays!
pub struct Display_Description {
    width: u16,
    height: u16,
    buffer_size: u16,
}

impl Display_Description {
    pub fn new(width: u16, height: u16, buffer_size: u16) -> Display_Description {

    }
}

pub enum Display {
    Eink_42_BW,
}

impl Display {
    /// Gets the Dimensions of a dipslay in the following order:
    /// - Width
    /// - Height
    /// - Neccessary Buffersize
    pub fn get_dimensions(&self) -> (u16, u16, u16) {
        match self {
            Display::Eink_42_BW => (400, 300, 15000)
        }
    }
}

pub struct Graphics {
    width: u16,
    height: u16,
    rotate: Displayorientation,
    buffer: [u8; 15000],   
}



impl Graphics {
    /// width needs to be a multiple of 8!
    pub fn new(width: u16, height: u16) -> Graphics{
        Graphics {width, height, rotate: Displayorientation::Rotate0}
    }

    pub fn clear(&mut self) {
        self.buffer = &mut [0u8; 1000]
    }
}*/