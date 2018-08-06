
/// A struct containing necessary info about a epd (electronic paper display). E.g:
/// 
/// - Width
/// - Height
/// ...
/// 
/// This needs to be implemented by each new Display
pub struct EPD {
    pub(crate) width: u16,
    pub(crate) height: u16
    //displayrotation?
}

impl EPD {
    pub(crate) fn new(width: u16, height: u16) -> EPD {
        EPD {width, height}
    }


}