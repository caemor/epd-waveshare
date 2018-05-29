[![Build Status](https://travis-ci.com/Caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.com/Caemor/eink-waveshare-rs)

# eink-waveshare-rs

IN WORK! Drivers for various EPDs from Waveshare. 

Currently only support for the 4.2 Black/White one

Be careful with the partial updates!
It was only tested in a mBED implementation, the rust one wasn't tested enough yet!!!

## TODO's

- [ ] add more example (e.g. for f3)
- [ ] improve the partial drawing/check the timings/timing improvements/....
- [ ] for later: add support for the smaller waveshare epds
- [ ] License: Stay with ISC (=MIT) or go to Apache+MIT Dual Version as used in many other projects?


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


### With a Buffer

- Chars, Strings and filled circles are still missing
- maybe work with traits here for line_drawing and so on?

### Without a Buffer

Maybe add support for Non-Buffer drawing from the https://crates.io/crates/embedded-graphics Crate later on.


## Examples

There is an example for Raspberry Pi in the example folder.

