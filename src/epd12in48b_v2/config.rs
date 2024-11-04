#[derive(Copy, Clone, Debug)]
/// EPD Configuration
pub struct Config {
    /// Specifies how data1 bits are mapped to colors:
    /// - `false`: 0 => black, 1 => white
    /// - `true`: 0 => white, 1 => black
    pub inverted_kw: bool,
    /// Specifies how data2 bits are mapped to colors:
    /// - `false`: 0 => red not active, 1 => red active
    /// - `true`: 0 => red active, 1 => red not active
    ///
    /// Note that whenever the red channel is active, the black/white channel is ignored.
    pub inverted_r: bool,
    /// Lookup table to use for the screen border
    pub border_lut: BorderLUT,
    /// Whether to use the lookup tables loaded via `set_lut...` methods, or the built-in ones.
    pub external_lut: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inverted_kw: false,
            inverted_r: false,
            border_lut: BorderLUT::LUTBD,
            external_lut: false,
        }
    }
}

/// Screen border lookup table variants
#[derive(Copy, Clone, Debug)]
pub enum BorderLUT {
    /// Use LUTBD
    LUTBD,
    /// Use LUTK
    LUTK,
    /// Use LUTW
    LUTW,
    /// Use LUTR
    LUTR,
}
