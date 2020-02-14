[![Build Status](https://travis-ci.com/caemor/epd-waveshare.svg?branch=master)](https://travis-ci.com/caemor/epd-waveshare)

This library contains a driver for E-Paper Modules from Waveshare (which are basically the same as the Dalian Good Display ones).

It uses the [embedded graphics](https://crates.io/crates/embedded-graphics) library for the optional graphics support.

A 2018-edition compatible version (Rust 1.31+) is needed.

Other similiar libraries with support for much more displays are [u8g2](https://github.com/olikraus/u8g2) and [GxEPD](https://github.com/ZinggJM/GxEPD) for arduino.

## Examples

There are multiple examples in the examples folder. For more infos about the examples see the seperate Readme [there](/examples/Readme.md). These examples are all rust projects of their own, so you need to go inside the project to execute it (cargo run --example doesn't work).

```Rust
// Setup the epd
let mut epd = EPD4in2::new(&mut spi, cs, busy, dc, rst, &mut delay)?;

// Setup the graphics
let mut buffer = Buffer4in2::default();
let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);

// Draw some text
display.draw(
    Font12x16::render_str("Hello Rust!")
        .stroke(Some(Color::Black))
        .fill(Some(Color::White))
        .translate(Coord::new(5, 50))
        .into_iter(),
);

// Transfer the frame data to the epd
epd.update_frame(&mut spi, &display.buffer())?;

// Display the frame on the epd
epd.display_frame(&mut spi)?;
```

## (Supported) Devices

| Device (with Link) | Colors | Flexible Display | Partial Refresh | Supported | Tested |
| :---: | --- | :---: | :---: | :---: | :---: |
| [7.5 Inch B/W V2 (A)](https://www.waveshare.com/product/7.5inch-e-paper-hat.htm) [[1](#1-75-inch-bw-v2-a)] | Black, White | ✕ | ✕ | ✔ | ✔ |
| [7.5 Inch B/W (A)](https://www.waveshare.com/product/7.5inch-e-paper-hat.htm) | Black, White | ✕ | ✕ | ✔ | ✔ |
| [4.2 Inch B/W (A)](https://www.waveshare.com/product/4.2inch-e-paper-module.htm) | Black, White | ✕ | Not officially [[2](#2-42-inch-e-ink-blackwhite---partial-refresh)] | ✔ | ✔ |
| [1.54 Inch B/W (A)](https://www.waveshare.com/1.54inch-e-Paper-Module.htm) | Black, White | ✕ | ✔ | ✔ | ✔ |
| [2.13 Inch B/W (A)](https://www.waveshare.com/product/2.13inch-e-paper-hat.htm) | Black, White | ✕ | ✔ |  |  |
| [2.9 Inch B/W (A)](https://www.waveshare.com/product/2.9inch-e-paper-module.htm) | Black, White | ✕ | ✔ | ✔ | ✔ [[3](#3-29-inch-e-ink-blackwhite---tests)] |
| [1.54 Inch B/W/R (B)](https://www.waveshare.com/product/modules/oleds-lcds/e-paper/1.54inch-e-paper-module-b.htm) | Black, White, Red | ✕ | ✕ | ✔ | ✔ |

### [1]: 7.5 Inch B/W V2 (A)

Since November 2019 Waveshare sells their updated version of these displays.
They should have a "V2" marking sticker on the backside of the panel.

Use `epd7in5_v2` instead of `epd7in5`, because the protocol changed.

### [2]: 4.2 Inch E-Ink Black/White - Partial Refresh

Out of the Box the original driver from Waveshare only supports full updates.

That means: Be careful with the quick refresh updates: <br>
It's possible with this driver but might lead to ghosting / burn-in effects therefore it's hidden behind a feature.

### [3]: 2.9 Inch E-Ink Black/White - Tests

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
