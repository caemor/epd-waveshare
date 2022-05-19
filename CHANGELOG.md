# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- Added support for positive and negatives modes of rendering in TriColor display in #92 (thanks to @akashihi)
- Added Epd 5in83 V2 (B) support in #92 (thanks to @akashihi)

### Changed

- Made Examples and Linux embedded hal optional (linux only) and therefore allowed building on other hosts (#101, #94)

### Fixed

## [v0.5.0] - 2021-11-28

### Added

- Added QuickRefresh Trait and implemented it for EPD4in2 in #62 (thanks to @David-OConnor)
- Added Epd 2in7 (B) support in #60 (thanks to @pjsier)
- Added Epd 7in5 HD support (thanks to @whiite)
- Added Epd 2in9 V2 support in #73 & #74 (thanks to @andber1)
- Added Epd 2in13 (BC) support in #75 (thanks to @Irbalt)
- Added Color conversion methods in #87 & #88 (thanks to @crzysdrs)
- Provide full QuickRefresh interface for 4.2 inch display in #81 (thanks to @sirhcel)


### Changed

- Updated embedded-graphics to 0.7 and switch to e-g-core #78 (@Irbalt) & #85 (@jamwaffles)
- Use specific ParseColorError instead of ()
- Epd4in2: Don't set the resolution (and some more) over and over again (#48)
- Removed `#[allow(non_camel_case_types)]` to fix various issues around it
- Added Delay to QuickRefresh Trait due to #74 (thanks to @andber1)
- Write data over SPI 1 byte at a time due to #82 (thanks to @belak)
- Enable drawing in three colors for epd2in13bc in #76 (thanks to @Irbalt)


## [v0.4.0] - 2020-04-06

### Added

- New supported epds: epd7in5 (thanks to @str4d), epd7in5 v2 (thanks to @asaaki), epd1in54b (thanks to @jkristell)
- Added update_and_display_frame to WaveshareDisplay trait (fixes #38)
- also improve position of busy_wait (#30) once more
- More Documentation

### Changed

- Update embedded-graphics to 0.6 (changes Display Trait) (and to 0.5 before thanks to @dbr)
- Remove useless feature gates (Doesn't change size)
- Update and integrate a few important examples and remove the others
- Use Embedded_hal:digital::v2

### Fixed

- Doc Tests

## [v0.3.2] - 2019-06-17

### Fixed

- Added some more missing wait_until_idle calls

## [v0.3.1] - 2019-04-06

### Added

- Example for epd4in2 and BluePill-Board

### Changed

- Improved CI

### Fixed

- Timing issues in display_frame function: epd1in54 and epd2in9 were both missing a necessary wait_until_idle call at
  the end of their display_frame function which sometimes caused invalid/ignored commands/inputs afterwards
- Some CI Targets were not tested correctly before

## [v0.3.0] - 2019-04-04

### Added

- added eink to keywords
- added reference to previous crate-name
- improved readme/docs e.g. added reference to a few great arduino display libs for these epds
- Added is_busy to Waveshare_Interface
- Added IS_BUSY_LOW const for all supported epds
- Added is_busy to DisplayInterface
- Added VarDisplay (a variable buffer-size display/graphic driver)
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

[Unreleased]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.4.0...HEAD

[v0.4.0]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.3.2...v0.4.0

[v0.3.2]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.3.1...v0.3.2

[v0.3.1]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.3.0...v0.3.1

[v0.3.0]: https://github.com/Caemor/eink-waveshare-rs/compare/v0.2.0...v0.3.0
