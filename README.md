# eink-waveshare-rs [![Build Status](https://travis-ci.com/Caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.com/Caemor/eink-waveshare-rs)

This library contains a driver for the [4.2 Inch E-Paper Module](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module) from Waveshare.

Support for more (especially the smaller and faster ones) should follow after the library around the 4.2" EInk is stable and tested enough.

## (Supported) Devices

| Device | Colors | Partial Refresh | Supported | Tested |
| :---: | --- | :---: | :---: | :---: |
| 4.2 Inch B/W | Black, White | Not officially [1](#42-inch-e-ink-blackwhite) | ✔ | ✔ |
| 1.54 Inch B/W | Black, White | ✔ |  |  |
| 2.13 Inch B/W | Black, White | ✔ |  |  |
| 2.9 Inch B/W | Black, White | ✔ |  |  |


### 4.2 Inch E-Ink Black/White

Out of the Box the original driver from Waveshare only supports full updates. 

But behind Be careful with the partial updates!
It was only tested in a Mbed implementation, the rust one wasn't tested enough yet!!!

[1]: It's possible with this driver but might lead to ghosting / burn-in effects therefore it's hidden behind a feature.

### Interface

| Interface | Description |
| :---: |  :--- |
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




