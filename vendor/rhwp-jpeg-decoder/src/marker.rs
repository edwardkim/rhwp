// Table B.1
#[derive(Clone, Copy, Debug, PartialEq)]
// Note: Established names.
#[allow(clippy::upper_case_acronyms)]
pub enum Marker {
    /// Start Of Frame markers
    ///
    /// - SOF(0):  Baseline DCT (Huffman coding)
    /// - SOF(1):  Extended sequential DCT (Huffman coding)
    /// - SOF(2):  Progressive DCT (Huffman coding)
    /// - SOF(3):  Lossless (sequential) (Huffman coding)
    /// - SOF(5):  Differential sequential DCT (Huffman coding)
    /// - SOF(6):  Differential progressive DCT (Huffman coding)
    /// - SOF(7):  Differential lossless (sequential) (Huffman coding)
    /// - SOF(9):  Extended sequential DCT (arithmetic coding)
    /// - SOF(10): Progressive DCT (arithmetic coding)
    /// - SOF(11): Lossless (sequential) (arithmetic coding)
    /// - SOF(13): Differential sequential DCT (arithmetic coding)
    /// - SOF(14): Differential progressive DCT (arithmetic coding)
    /// - SOF(15): Differential lossless (sequential) (arithmetic coding)
    SOF(u8),
    /// Reserved for JPEG extensions
    JPG,
    /// Define Huffman table(s)
    DHT,
    /// Define arithmetic coding conditioning(s)
    DAC,
    /// Restart with modulo 8 count `m`
    RST(u8),
    /// Start of image
    SOI,
    /// End of image
    EOI,
    /// Start of scan
    SOS,
    /// Define quantization table(s)
    DQT,
    /// Define number of lines
    DNL,
    /// Define restart interval
    DRI,
    /// Define hierarchical progression
    DHP,
    /// Expand reference component(s)
    EXP,
    /// Reserved for application segments
    APP(u8),
    /// Reserved for JPEG extensions
    JPGn(u8),
    /// Comment
    COM,
    /// For temporary private use in arithmetic coding
    TEM,
    /// Reserved
    RES,
}

impl Marker {
    pub fn has_length(self) -> bool {
        use self::Marker::*;
        !matches!(self, RST(..) | SOI | EOI | TEM)
    }

    pub fn from_u8(n: u8) -> Option<Marker> {
        use self::Marker::*;
        match n {
            0x00 => None, // Byte stuffing
            0x01 => Some(TEM),
            0x02..=0xBF => Some(RES),
            0xC0 => Some(SOF(0)),
            0xC1 => Some(SOF(1)),
            0xC2 => Some(SOF(2)),
            0xC3 => Some(SOF(3)),
            0xC4 => Some(DHT),
            0xC5 => Some(SOF(5)),
            0xC6 => Some(SOF(6)),
            0xC7 => Some(SOF(7)),
            0xC8 => Some(JPG),
            0xC9 => Some(SOF(9)),
            0xCA => Some(SOF(10)),
            0xCB => Some(SOF(11)),
            0xCC => Some(DAC),
            0xCD => Some(SOF(13)),
            0xCE => Some(SOF(14)),
            0xCF => Some(SOF(15)),
            0xD0 => Some(RST(0)),
            0xD1 => Some(RST(1)),
            0xD2 => Some(RST(2)),
            0xD3 => Some(RST(3)),
            0xD4 => Some(RST(4)),
            0xD5 => Some(RST(5)),
            0xD6 => Some(RST(6)),
            0xD7 => Some(RST(7)),
            0xD8 => Some(SOI),
            0xD9 => Some(EOI),
            0xDA => Some(SOS),
            0xDB => Some(DQT),
            0xDC => Some(DNL),
            0xDD => Some(DRI),
            0xDE => Some(DHP),
            0xDF => Some(EXP),
            0xE0 => Some(APP(0)),
            0xE1 => Some(APP(1)),
            0xE2 => Some(APP(2)),
            0xE3 => Some(APP(3)),
            0xE4 => Some(APP(4)),
            0xE5 => Some(APP(5)),
            0xE6 => Some(APP(6)),
            0xE7 => Some(APP(7)),
            0xE8 => Some(APP(8)),
            0xE9 => Some(APP(9)),
            0xEA => Some(APP(10)),
            0xEB => Some(APP(11)),
            0xEC => Some(APP(12)),
            0xED => Some(APP(13)),
            0xEE => Some(APP(14)),
            0xEF => Some(APP(15)),
            0xF0 => Some(JPGn(0)),
            0xF1 => Some(JPGn(1)),
            0xF2 => Some(JPGn(2)),
            0xF3 => Some(JPGn(3)),
            0xF4 => Some(JPGn(4)),
            0xF5 => Some(JPGn(5)),
            0xF6 => Some(JPGn(6)),
            0xF7 => Some(JPGn(7)),
            0xF8 => Some(JPGn(8)),
            0xF9 => Some(JPGn(9)),
            0xFA => Some(JPGn(10)),
            0xFB => Some(JPGn(11)),
            0xFC => Some(JPGn(12)),
            0xFD => Some(JPGn(13)),
            0xFE => Some(COM),
            0xFF => None, // Fill byte
        }
    }
}
