[![Build Status](https://travis-ci.com/Caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.com/Caemor/eink-waveshare-rs)

This library contains a driver for E-Paper Modules  from Waveshare.

It uses the [embedded graphics](https://crates.io/crates/embedded-graphics) library for the optional graphics support.

## (Supported) Devices

| Device (with Link) | Colors | Flexible Display | Partial Refresh | Supported | Tested |
| :---: | --- | :---: | :---: | :---: | :---: |
| [4.2 Inch B/W (A)](https://www.waveshare.com/product/4.2inch-e-paper-module.htm) | Black, White | ✕ | Not officially [[1](#42-inch-e-ink-blackwhite)] | ✔ | ✔ |
| [1.54 Inch B/W (A)](https://www.waveshare.com/1.54inch-e-Paper-Module.htm) | Black, White | ✕ | ✔ | ✔ | ✔ |
| [2.13 Inch B/W (A)](https://www.waveshare.com/product/2.13inch-e-paper-hat.htm) | Black, White | ✕ | ✔ |  |  |
| [2.9 Inch B/W (A)](https://www.waveshare.com/product/2.9inch-e-paper-module.htm) | Black, White | ✕ | ✔ | ✔ | ✔ [[2](#2-29-inch-e-ink-blackwhite---tests)] |


### [1]: 4.2 Inch E-Ink Black/White - Partial Refresh

Out of the Box the original driver from Waveshare only supports full updates. 

That means: Be careful with the quick refresh updates: <br>
It's possible with this driver but might lead to ghosting / burn-in effects therefore it's hidden behind a feature.

### [2]: 2.9 Inch E-Ink Black/White - Tests

Since my 2.9 Inch Display has some blurring issues I am not absolutly sure if everything was working correctly as it should :-)

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

### Display Configs

There are two types of Display Configurations used in Wavedshare EPDs, which also needs to be set on the "new" E-Paper Driver HAT.
They are also called A and B, but you shouldn't get confused and mix it with the Type A,B,C and D of the various Displays, which just describe different types (colored variants) or new versions. In the Display Config the seperation is most likely due to included fast partial refresh of the displays. In a Tabular form: 

| Type A | Tybe B |
| :---: |  :---: |
| 1.54in (A) | 1.54in (B) |
| 2.13in (A) | 1.54in (C) |
| 2.13in (D) | 2.13in (B) |
| 2.9in (A)  | 2.13in (C) |
|            | 2.7in  (A) |
|            | 2.7in  (B) |
|            | 2.9in  (B) |
|            | 2.9in  (C) |
|            | 4.2in  (A) |
|            | 4.2in  (B) |
|            | 4.2in  (C) |
|            | 7.5in  (A) |
|            | 7.5in  (B) |
|            | 7.5in  (C) |

## TODO's

- [ ] improve the partial drawing/check the timings/timing improvements/....

## Examples

There are multiple examples in the examples folder. For more infos see the seperate Readme [there](/examples/Readme.md):




