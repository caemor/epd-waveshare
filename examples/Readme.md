# Examples:

All of these examples are projects of their own. 

A few notes:
 - If not stated otherwise the example is for a Raspberry Pi running Linux.
 - epdXinYY_full showcase most of what can be done with this crate. This means that they are using graphics feature and use the DisplayXinYY with its buffer. 

Special Examples:

### epd4in2_var_display_buffer

This examples used the graphics feature with VarDisplay and therefore a variable buffer(size).

### epd1in54_no_graphics (Fastest Example)

This example doesn't use the graphics feature and handles all the "drawing" by itself. It also has a speeddemonstration included.

### epd4in2_full_blue_pill

Connect epd4in2 display to blue pill board:
- BUSY -> A10
- RST -> A9
- DC -> A8
- CS -> B12
- CLK -> B13
- DIN -> B15
- GND -> G
- VCC -> 3.3

For compiling and flashing, please refer to [TeXitois blue pill quickstart](https://github.com/TeXitoi/blue-pill-quickstart/blob/master/README.md).

Basically:

```shell
curl https://sh.rustup.rs -sSf | sh
rustup target add thumbv7m-none-eabi
sudo apt-get install gdb-arm-none-eabi openocd
cd epd4in2_full_blue_pill
# connect ST-Link v2 to the blue pill and the computer
# openocd in another terminal
cargo run --release
```

Ff you can't connect to openocd you might need to adapt your udev rules or use sudo ([openOCD Problems](https://rust-embedded.github.io/discovery/03-setup/linux.html#udev-rules))


