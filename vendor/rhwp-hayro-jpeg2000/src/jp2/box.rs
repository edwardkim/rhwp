//! Parsing a JP2 box, as specified in I.4.

#![allow(
    dead_code,
    reason = "JP2 box constants exist for completeness but not all are referenced yet"
)]

use alloc::string::{String, ToString};

use crate::reader::BitReader;

/// JP2 signature box - 'jP\040\040'.
pub(crate) const JP2_SIGNATURE: u32 = 0x6A502020;
/// File Type box - 'ftyp'.
pub(crate) const FILE_TYPE: u32 = 0x66747970;
/// JP2 Header box - 'jp2h'.
pub(crate) const JP2_HEADER: u32 = 0x6A703268;
/// Image Header box - 'ihdr'.
pub(crate) const IMAGE_HEADER: u32 = 0x69686472;
/// Bits Per Component box - 'bpcc'.
pub(crate) const BITS_PER_COMPONENT: u32 = 0x62706363;
/// Colour Specification box - 'colr'.
pub(crate) const COLOUR_SPECIFICATION: u32 = 0x636F6C72;
/// Palette box - 'pclr'.
pub(crate) const PALETTE: u32 = 0x70636C72;
/// Component Mapping box - 'cmap'.
pub(crate) const COMPONENT_MAPPING: u32 = 0x636D6170;
/// Channel Definition box - 'cdef'.
pub(crate) const CHANNEL_DEFINITION: u32 = 0x63646566;
/// Resolution box - 'res\x20'.
pub(crate) const RESOLUTION: u32 = 0x72657320;
/// Capture Resolution box - 'resc'.
pub(crate) const CAPTURE_RESOLUTION: u32 = 0x72657363;
/// Default Display Resolution box - 'resd'.
pub(crate) const DISPLAY_RESOLUTION: u32 = 0x72657364;
/// Contiguous Codestream box - 'jp2c'.
pub(crate) const CONTIGUOUS_CODESTREAM: u32 = 0x6A703263;
/// Intellectual Property box - 'jp2i'.
pub(crate) const INTELLECTUAL_PROPERTY: u32 = 0x6A703269;
/// XML box - 'xml\x20'.
pub(crate) const XML: u32 = 0x786D6C20;
/// UUID box - 'uuid'.
pub(crate) const UUID: u32 = 0x75756964;
/// UUID Info box - 'uinf'.
pub(crate) const UUID_INFO: u32 = 0x75696E66;
/// UUID List box - 'ulst'.
pub(crate) const UUID_LIST: u32 = 0x756C7374;
/// URL box - 'url\x20'.
pub(crate) const URL: u32 = 0x75726C20;

pub(crate) struct Jp2Box<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) box_type: u32,
}

/// Converts a box tag to its string representation.
///
/// Box tags are stored as 4-byte ASCII codes in big-endian format.
pub(crate) fn tag_to_string(tag: u32) -> String {
    let bytes = [
        ((tag >> 24) & 0xFF) as u8,
        ((tag >> 16) & 0xFF) as u8,
        ((tag >> 8) & 0xFF) as u8,
        (tag & 0xFF) as u8,
    ];
    String::from_utf8_lossy(&bytes).to_string()
}

pub(crate) fn read<'a>(reader: &mut BitReader<'a>) -> Option<Jp2Box<'a>> {
    rhwp_codec_resource::work(1).ok()?;
    let l_box = reader.read_u32()?;
    let t_box = reader.read_u32()?;

    let data = match l_box {
        // If the value of this field is 0, then the length of the box
        // was not known when the LBox field was written. In this case, this box contains
        // all bytes up to the end of the file.
        0 => {
            let data = reader.tail()?;
            reader.jump_to_end();
            data
        }
        // If the value of this field is 1, then the XLBox field shall exist and the value of
        // that field shall be the actual length of the box.
        // The value includes all of the fields of the box, including the LBox, TBox and XLBox
        // fields.
        1 => {
            let xl_box = reader.read_u64()?.checked_sub(16)?;
            reader.read_bytes(xl_box as usize)?
        }
        // This field specifies the length of the box, stored as a 4-byte big-endian unsigned integer.
        // This value includes all of the fields of the box, including the length and type.
        _ => {
            let length = l_box.checked_sub(8)?;
            reader.read_bytes(length as usize)?
        }
    };

    Some(Jp2Box {
        data,
        box_type: t_box,
    })
}
