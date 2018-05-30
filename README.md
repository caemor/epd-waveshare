# eink-waveshare-rs [![Build Status](https://travis-ci.com/Caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.com/Caemor/eink-waveshare-rs)

This library contains a driver for the [4.2 Inch E-Paper Moduel](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module) from Waveshare.

Support for more (especially the smaller and faster ones) should follow after the library around the 4.2" EInk is stable and tested enough.


## 4.2 Inch E-Ink

Out of the Box the original driver from Waveshare supported only full updates. 

Currently only support for the 4.2 Black/White one

Be careful with the partial updates!
It was only tested in a mBED implementation, the rust one wasn't tested enough yet!!!

### Interface

| Interface | Description |
| :---: |  :---: |
| VCC 	|   3.3V |
| GND   | 	GND |
| DIN   | 	SPI MOSI |
| CLK   | 	SPI SCK |
| CS    | 	SPI chip select (Low active) |
| DC    | 	Data/Command control pin (High for data, and low for command) |
| RST   | 	External reset pin (Low for reset) |
| BUSY  | 	Busy state output pin (Low for busy)  |

## TODO's

- [ ] add more examples (e.g. for f3)
- [ ] improve the partial drawing/check the timings/timing improvements/....
- [ ] for later: add support for the smaller waveshare epds
- [ ] License: Stay with ISC (=MIT) or go to the Apache+MIT Dual License as used in many other projects?

## Graphics/Drawing

Supports:
- Lines
- Squares
- Circles
- Pixels
- Chars
- Strings

Chars and Strings work with a 8x8-Font.

Support for bigger sized/independent Fonts is in work.

## Examples

There is an example for Raspberry Pi in the example folder.

