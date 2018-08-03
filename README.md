# eink-waveshare-rs [![Build Status](https://travis-ci.com/Caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.com/Caemor/eink-waveshare-rs)

This library contains a driver for E-Paper Modules  from Waveshare.

Support for more than the 4.2" EPD (especially the smaller and faster ones) is in the work.

## (Supported) Devices

| Device (with Link) | Colors | Flexible Display | Partial Refresh | Supported | Tested |
| :---: | --- | :---: | :---: | :---: | :---: |
| [4.2 Inch B/W (A)](https://www.waveshare.com/product/4.2inch-e-paper-module.htm) | Black, White | ✕ | Not officially [[1](#42-inch-e-ink-blackwhite)] | ✔ | ✔ |
| [1.54 Inch B/W (A)](https://www.waveshare.com/1.54inch-e-Paper-Module.htm) | Black, White | ✕ | ✔ |  |  |
| [2.13 Inch B/W (A)](https://www.waveshare.com/product/2.13inch-e-paper-hat.htm) | Black, White | ✕ | ✔ |  |  |
| [2.9 Inch B/W (A)](https://www.waveshare.com/product/2.9inch-e-paper-module.htm) | Black, White | ✕ | ✔ | ✔ |  |


### 4.2 Inch E-Ink Black/White

Out of the Box the original driver from Waveshare only supports full updates. 

- [1]: Be careful with the quick refresh updates: <br>
It's possible with this driver but might lead to ghosting / burn-in effects therefore it's hidden behind a feature.

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




