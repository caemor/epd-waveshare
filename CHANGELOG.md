# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

<!-- ## [v0.3.1] - 2019-04-04 -->

## [v0.3.0] - 2019-04-04

### Added
- added eink to keywords
- added reference to previous crate-name
- improved readme/docs e.g. added reference to a few great arduino display libs for these epds
- Added is_busy to Waveshare_Interface
- Added IS_BUSY_LOW const for all supported epds
- Added is_busy to DisplayInterface
- Added VarDisplay (a variable buffersize display/graphic driver)
- Updated and added more examples
- add a feature gated alternative full lut for type_a displays

### Removed
- Removed all Buffers (Buffer1in54,...) and instead made specialised Displays (Display1in54,...) with included Buffers

### Changed
- Switch to 2018 edition
- "cargo fix --edition" for the library
- Use cargo fix edition-idioms and remove the internal renaming from embedded_hal to hal
- moved width, height and default_background_color directly to epd4in2 module
- remove pub from set_lut_helper function
- fix behaviour of set_lut for epd2in9. it always sets the LUT now!

## v0.2.0 - 2018-10-30

Initial release with Changelog

### Added
- Uses embedded-graphics now
- Tested and fixed 1.54 inch, 2.9 inch and 4.2 inch display

### Removed
- Old included Graphics Library

### Changed
- Lots of internal changes
- Renamed to `epd-waveshare`


[Unreleased]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.2.0...v0.3.0
