#[rustfmt::skip]

#[cfg(feature = "epd2in13_v2")]
// Original Waveforms from Waveshare
pub(crate) const LUT_FULL_UPDATE: [u8; 70] =[
    0x80,0x60,0x40,0x00,0x00,0x00,0x00,             // LUT0: BB:     VS 0 ~7
    0x10,0x60,0x20,0x00,0x00,0x00,0x00,             // LUT1: BW:     VS 0 ~7
    0x80,0x60,0x40,0x00,0x00,0x00,0x00,             // LUT2: WB:     VS 0 ~7
    0x10,0x60,0x20,0x00,0x00,0x00,0x00,             // LUT3: WW:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT4: VCOM:   VS 0 ~7

    0x03,0x03,0x00,0x00,0x02,                       //  TP0 A~D RP0
    0x09,0x09,0x00,0x00,0x02,                       //  TP1 A~D RP1
    0x03,0x03,0x00,0x00,0x02,                       //  TP2 A~D RP2
    0x00,0x00,0x00,0x00,0x00,                       //  TP3 A~D RP3
    0x00,0x00,0x00,0x00,0x00,                       //  TP4 A~D RP4
    0x00,0x00,0x00,0x00,0x00,                       //  TP5 A~D RP5
    0x00,0x00,0x00,0x00,0x00,                       //  TP6 A~D RP6
];

#[cfg(feature = "epd2in13_v2")]
#[rustfmt::skip]
pub(crate) const LUT_PARTIAL_UPDATE: [u8; 70] =[
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT0: BB:     VS 0 ~7
    0x80,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT1: BW:     VS 0 ~7
    0x40,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT2: WB:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT3: WW:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT4: VCOM:   VS 0 ~7

    0x0A,0x00,0x00,0x00,0x00,                       //  TP0 A~D RP0
    0x00,0x00,0x00,0x00,0x00,                       //  TP1 A~D RP1
    0x00,0x00,0x00,0x00,0x00,                       //  TP2 A~D RP2
    0x00,0x00,0x00,0x00,0x00,                       //  TP3 A~D RP3
    0x00,0x00,0x00,0x00,0x00,                       //  TP4 A~D RP4
    0x00,0x00,0x00,0x00,0x00,                       //  TP5 A~D RP5
    0x00,0x00,0x00,0x00,0x00,                       //  TP6 A~D RP6
];

#[cfg(feature = "epd2in13_v3")]
#[rustfmt::skip]
// Original Waveforms from Waveshare
pub(crate) const LUT_PARTIAL_UPDATE: [u8; 159] =[
	0x0,0x40,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x80,0x80,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x40,0x40,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x80,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x14,0x0,0x0,0x0,0x0,0x0,0x0,  
	0x1,0x0,0x0,0x0,0x0,0x0,0x0,
	0x1,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x22,0x22,0x22,0x22,0x22,0x22,0x0,0x0,0x0,
	0x22,0x17,0x41,0x00,0x32,0x36,
];

#[cfg(feature = "epd2in13_v3")]
#[rustfmt::skip]
pub(crate) const LUT_FULL_UPDATE: [u8; 159] =[
	0x80,0x4A,0x40,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x40,0x4A,0x80,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x80,0x4A,0x40,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x40,0x4A,0x80,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0xF,0x0,0x0,0x0,0x0,0x0,0x0,
	0xF,0x0,0x0,0xF,0x0,0x0,0x2,
	0xF,0x0,0x0,0x0,0x0,0x0,0x0,
	0x1,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x0,0x0,0x0,0x0,0x0,0x0,0x0,
	0x22,0x22,0x22,0x22,0x22,0x22,0x0,0x0,0x0,		
	0x22,0x17,0x41,0x0,0x32,0x36
];
