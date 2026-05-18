use crate::wmf::{imports::*, parser::*};

#[derive(Clone)]
pub struct Bitmap(Vec<u8>);

impl core::fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Bitmap")
            .field(&format!("[u8; {}]", self.0.len()))
            .finish()
    }
}

impl Bitmap {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<DeviceIndependentBitmap> for Bitmap {
    fn from(dib: DeviceIndependentBitmap) -> Self {
        let dib = dib.expand_color_palette();

        let mut info_header = vec![];
        let mut file_size: u32 = 0;

        // write info header
        match dib.dib_header_info {
            BitmapInfoHeader::Core(BitmapInfoHeaderCore {
                header_size,
                width,
                height,
                planes,
                bit_count,
            }) => {
                file_size += header_size;
                info_header.extend(header_size.to_le_bytes());
                info_header.extend(width.to_le_bytes());
                info_header.extend(height.to_le_bytes());
                info_header.extend(planes.to_le_bytes());
                info_header.extend((bit_count as u16).to_le_bytes());
            }
            BitmapInfoHeader::Info(BitmapInfoHeaderInfo {
                header_size,
                width,
                height,
                planes,
                bit_count,
                compression,
                image_size,
                x_pels_per_meter,
                y_pels_per_meter,
                color_used,
                color_important,
            }) => {
                file_size += header_size;
                info_header.extend(header_size.to_le_bytes());
                info_header.extend(width.to_le_bytes());
                info_header.extend(height.to_le_bytes());
                info_header.extend(planes.to_le_bytes());
                info_header.extend((bit_count as u16).to_le_bytes());
                info_header.extend((compression as u32).to_le_bytes());
                info_header.extend(image_size.to_le_bytes());
                info_header.extend(x_pels_per_meter.to_le_bytes());
                info_header.extend(y_pels_per_meter.to_le_bytes());
                info_header.extend(color_used.to_le_bytes());
                info_header.extend(color_important.to_le_bytes());
            }
            BitmapInfoHeader::V4(BitmapInfoHeaderV4 {
                header_size,
                width,
                height,
                planes,
                bit_count,
                compression,
                image_size,
                x_pels_per_meter,
                y_pels_per_meter,
                color_used,
                color_important,
                red_mask,
                green_mask,
                blue_mask,
                alpha_mask,
                color_space_type,
                endpoints,
                gamma_red,
                gamma_green,
                gamma_blue,
            }) => {
                file_size += header_size;
                info_header.extend(header_size.to_le_bytes());
                info_header.extend(width.to_le_bytes());
                info_header.extend(height.to_le_bytes());
                info_header.extend(planes.to_le_bytes());
                info_header.extend((bit_count as u16).to_le_bytes());
                info_header.extend((compression as u32).to_le_bytes());
                info_header.extend(image_size.to_le_bytes());
                info_header.extend(x_pels_per_meter.to_le_bytes());
                info_header.extend(y_pels_per_meter.to_le_bytes());
                info_header.extend(color_used.to_le_bytes());
                info_header.extend(color_important.to_le_bytes());
                info_header.extend(red_mask.to_le_bytes());
                info_header.extend(green_mask.to_le_bytes());
                info_header.extend(blue_mask.to_le_bytes());
                info_header.extend(alpha_mask.to_le_bytes());
                info_header.extend((color_space_type as u32).to_le_bytes());
                info_header.extend(endpoints.red.x.to_le_bytes());
                info_header.extend(endpoints.red.y.to_le_bytes());
                info_header.extend(endpoints.red.z.to_le_bytes());
                info_header.extend(endpoints.green.x.to_le_bytes());
                info_header.extend(endpoints.green.y.to_le_bytes());
                info_header.extend(endpoints.green.z.to_le_bytes());
                info_header.extend(endpoints.blue.x.to_le_bytes());
                info_header.extend(endpoints.blue.y.to_le_bytes());
                info_header.extend(endpoints.blue.z.to_le_bytes());
                info_header.extend(gamma_red.to_le_bytes());
                info_header.extend(gamma_green.to_le_bytes());
                info_header.extend(gamma_blue.to_le_bytes());
            }
            BitmapInfoHeader::V5(BitmapInfoHeaderV5 {
                header_size,
                width,
                height,
                planes,
                bit_count,
                compression,
                image_size,
                x_pels_per_meter,
                y_pels_per_meter,
                color_used,
                color_important,
                red_mask,
                green_mask,
                blue_mask,
                alpha_mask,
                color_space_type,
                endpoints,
                gamma_red,
                gamma_green,
                gamma_blue,
                intent,
                profile_data,
                profile_size,
                reserved,
            }) => {
                file_size += header_size;
                info_header.extend(header_size.to_le_bytes());
                info_header.extend(width.to_le_bytes());
                info_header.extend(height.to_le_bytes());
                info_header.extend(planes.to_le_bytes());
                info_header.extend((bit_count as u16).to_le_bytes());
                info_header.extend((compression as u32).to_le_bytes());
                info_header.extend(image_size.to_le_bytes());
                info_header.extend(x_pels_per_meter.to_le_bytes());
                info_header.extend(y_pels_per_meter.to_le_bytes());
                info_header.extend(color_used.to_le_bytes());
                info_header.extend(color_important.to_le_bytes());
                info_header.extend(red_mask.to_le_bytes());
                info_header.extend(green_mask.to_le_bytes());
                info_header.extend(blue_mask.to_le_bytes());
                info_header.extend(alpha_mask.to_le_bytes());
                info_header.extend((color_space_type as u32).to_le_bytes());
                info_header.extend(endpoints.red.x.to_le_bytes());
                info_header.extend(endpoints.red.y.to_le_bytes());
                info_header.extend(endpoints.red.z.to_le_bytes());
                info_header.extend(endpoints.green.x.to_le_bytes());
                info_header.extend(endpoints.green.y.to_le_bytes());
                info_header.extend(endpoints.green.z.to_le_bytes());
                info_header.extend(endpoints.blue.x.to_le_bytes());
                info_header.extend(endpoints.blue.y.to_le_bytes());
                info_header.extend(endpoints.blue.z.to_le_bytes());
                info_header.extend(gamma_red.to_le_bytes());
                info_header.extend(gamma_green.to_le_bytes());
                info_header.extend(gamma_blue.to_le_bytes());
                info_header.extend((intent as u32).to_le_bytes());
                info_header.extend(profile_data.to_le_bytes());
                info_header.extend(profile_size.to_le_bytes());
                info_header.extend(reserved.to_le_bytes());
            }
        }

        // write pixel data
        let data = dib.bitmap_buffer.a_data;
        let data_len = u32::try_from(data.len()).expect("should be as u32");
        file_size += data_len;

        // write file headers
        let mut file_header = vec![];
        file_size += 14;
        file_header.extend(b"BM");
        file_header.extend(file_size.to_le_bytes());
        file_header.extend(0u32.to_le_bytes());
        file_header.extend((file_size - data_len).to_le_bytes());

        let data = {
            file_header.extend(info_header);
            file_header.extend(data);
            file_header
        };

        Self(data)
    }
}

impl From<(ColorRef, HatchStyle)> for Bitmap {
    fn from((color_ref, brush_hatch): (ColorRef, HatchStyle)) -> Self {
        let mut a_data = Vec::with_capacity(100);

        match brush_hatch {
            HatchStyle::HS_HORIZONTAL => {
                for i in 0..10 {
                    if i == 0 {
                        a_data.extend([1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
                    } else {
                        a_data.extend([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                    }
                }
            }
            HatchStyle::HS_VERTICAL => {
                a_data.extend(
                    vec![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; 10]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                );
            }
            HatchStyle::HS_FDIAGONAL => {
                for i in 0..10 {
                    for j in 0..10 {
                        if i + j == 9 {
                            a_data.push(1);
                        } else {
                            a_data.push(0);
                        }
                    }
                }
            }
            HatchStyle::HS_BDIAGONAL => {
                for i in 0..10 {
                    for j in 0..10 {
                        if i == j {
                            a_data.push(1);
                        } else {
                            a_data.push(0);
                        }
                    }
                }
            }
            HatchStyle::HS_CROSS => {
                for i in 0..10 {
                    if i == 0 {
                        a_data.extend([1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
                    } else {
                        a_data.extend([1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                    }
                }
            }
            HatchStyle::HS_DIAGCROSS => {
                for i in 0..10 {
                    for j in 0..10 {
                        if i == j || i + j == 9 {
                            a_data.push(1);
                        } else {
                            a_data.push(0);
                        }
                    }
                }
            }
        }

        DeviceIndependentBitmap {
            dib_header_info: BitmapInfoHeader::Info(BitmapInfoHeaderInfo {
                header_size: 40,
                width: 10,
                height: 10,
                planes: 1,
                bit_count: BitCount::BI_BITCOUNT_5,
                compression: Compression::BI_RGB,
                image_size: 0,
                x_pels_per_meter: 0,
                y_pels_per_meter: 0,
                color_used: 0,
                color_important: 0,
            }),
            colors: Colors::RGBTriple(vec![
                RGBTriple {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
                RGBTriple {
                    red: color_ref.red,
                    green: color_ref.green,
                    blue: color_ref.blue,
                },
            ]),
            bitmap_buffer: BitmapBuffer {
                undefined_space: vec![],
                a_data,
            },
        }
        .into()
    }
}

impl DeviceIndependentBitmap {
    fn expand_color_palette(self) -> Self {
        // nothing to do.
        if matches!(
            self.colors,
            crate::wmf::parser::Colors::Null | crate::wmf::parser::Colors::PaletteIndices(_)
        ) {
            return self;
        }

        let Self {
            dib_header_info,
            colors,
            bitmap_buffer,
        } = self;
        let bit_count = dib_header_info.bit_count();
        let palette: Vec<_> = match colors {
            Colors::RGBTriple(values) => values
                .into_iter()
                .map(|v| vec![v.red, v.green, v.blue])
                .collect(),
            Colors::RGBQuad(values) => values
                .into_iter()
                .map(|v| vec![v.red, v.green, v.blue])
                .collect(),
            _ => unreachable!(),
        };

        let new_bit_count = crate::wmf::parser::BitCount::BI_BITCOUNT_5;
        let new_line_bits = dib_header_info.width() * (new_bit_count as usize);
        let new_line_bytes = ((new_line_bits + 31) / 32) * 4;
        let new_line_padding =
            new_line_bytes - dib_header_info.width() * (new_bit_count as usize / 8);

        let line_bits = dib_header_info.width() * (bit_count as usize);
        let line_bytes = ((line_bits + 31) / 32) * 4;
        let mut position = 0;
        let mut new_data = vec![];

        for _ in 0..dib_header_info.height() {
            let mut reader =
                BitReader::new(&bitmap_buffer.a_data[position..(position + line_bytes)]);

            for _ in 0..dib_header_info.width() {
                let Some(idx) = reader.read_bits(bit_count as u8) else {
                    break;
                };

                let rgb = palette
                    .get(idx as usize)
                    .cloned()
                    .unwrap_or_else(|| vec![0xFF, 0xFF, 0xFF]);

                new_data.extend(rgb);
            }

            new_data.extend(vec![0; new_line_padding]);
            position += line_bytes;
        }

        Self {
            dib_header_info: match dib_header_info {
                crate::wmf::parser::BitmapInfoHeader::Core(v) => {
                    crate::wmf::parser::BitmapInfoHeader::Core(v)
                }
                crate::wmf::parser::BitmapInfoHeader::Info(v) => {
                    crate::wmf::parser::BitmapInfoHeader::Info(
                        crate::wmf::parser::BitmapInfoHeaderInfo {
                            bit_count: new_bit_count,
                            ..v
                        },
                    )
                }
                crate::wmf::parser::BitmapInfoHeader::V4(v) => {
                    crate::wmf::parser::BitmapInfoHeader::V4(
                        crate::wmf::parser::BitmapInfoHeaderV4 {
                            bit_count: new_bit_count,
                            ..v
                        },
                    )
                }
                crate::wmf::parser::BitmapInfoHeader::V5(v) => {
                    crate::wmf::parser::BitmapInfoHeader::V5(
                        crate::wmf::parser::BitmapInfoHeaderV5 {
                            bit_count: new_bit_count,
                            ..v
                        },
                    )
                }
            },
            colors: crate::wmf::parser::Colors::Null,
            bitmap_buffer: crate::wmf::parser::BitmapBuffer {
                undefined_space: vec![],
                a_data: new_data,
            },
        }
    }
}

struct BitReader<'a> {
    data: &'a [u8],
    byte_index: usize,
    bit_index: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            byte_index: 0,
            bit_index: 0,
        }
    }

    fn read_bits(&mut self, num_bits: u8) -> Option<u32> {
        if num_bits == 0 || num_bits > 32 {
            return None;
        }

        let mut value = 0u32;
        for _ in 0..num_bits {
            if self.byte_index >= self.data.len() {
                return None;
            }

            let bit = (self.data[self.byte_index] >> (7 - self.bit_index)) & 1;
            value = (value << 1) | u32::from(bit);

            self.bit_index += 1;
            if self.bit_index >= 8 {
                self.bit_index = 0;
                self.byte_index += 1;
            }
        }

        Some(value)
    }
}
