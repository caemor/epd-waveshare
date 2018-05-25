[![Build status](https://travis-ci.org/caemor/eink-waveshare-rs.svg?branch=master)](https://travis-ci.org/caemor/eink-waveshare-rs)

# eink-waveshare-rs
IN WORK! Drivers for various EPDs from Waveshare. Currently only support for the 4.2 Black/White one

Be careful with the partial updates!
It was only tested in a mBED implementation, this one wasn't tested yet!!!

Due to a broken 

## TODO's

- [ ] add some basic buffer drawing abilities
- [ ] test Embedded Linux (rpi) example
- [ ] add f3 example
- [ ] improve the partial drawing/check the timings/timing improvements/....
- [ ] for later: add support for the smaller waveshare epds
- [ ] License: Stay with ISC (=MIT) or go to Apache+MIT Dual Version as used in many other projects?


## Drawing

### With a Buffer

Is still missing at the moment. 

### Without a Buffer

Maybe add support for Non-Buffer drawing from the https://crates.io/crates/embedded-graphics Crate later on.


## Examples

There are some examples in the examples folder.

The f3 example is broken/working on a old version

