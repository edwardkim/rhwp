use crate::wmf::imports::*;

/// The DeviceIndependentBitmap (DIB) Object defines an image in
/// device-independent bitmap (DIB) format.
#[derive(Clone, Debug)]
pub struct DeviceIndependentBitmap {
    /// DIBHeaderInfo (variable): Either a BitmapCoreHeader Object or a
    /// BitmapInfoHeader Object that specifies information about the image.
    ///
    /// The first 32 bits of this field is the HeaderSize value. If it is
    /// 0x0000000C, then this is a BitmapCoreHeader; otherwise, this is a
    /// BitmapInfoHeader.
    pub dib_header_info: crate::wmf::parser::BitmapInfoHeader,
    /// Colors (variable): An optional array of either RGBQuad Objects or
    /// 16-bit unsigned integers that define a color table.
    ///
    /// The size and contents of this field SHOULD be determined from the
    /// metafile record or object that contains this DeviceIndependentBitmap
    /// Object and from information in the DIBHeaderInfo field. See ColorUsage
    /// Enumeration and BitCount Enumeration for additional details.
    pub colors: Colors,
    /// BitmapBuffer (variable): A buffer containing the image, which is not
    /// required to be contiguous with the DIB header, unless this is a packed
    /// bitmap.
    pub bitmap_buffer: BitmapBuffer,
}

impl DeviceIndependentBitmap {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub(crate) fn parse_with_color_usage<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        record_bytes: Option<usize>,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (dib_header_info, mut consumed_bytes) =
            crate::wmf::parser::BitmapInfoHeader::parse(buf)?;
        // [MS-WMF] 2.2.2.9: Colors 의 크기는 이 DIB 를 담은 레코드에서 정한다. 레코드에 남은
        // 바이트에서 헤더와 픽셀(aData)을 뺀 만큼만 색상표가 있을 수 있다. 헤더가 말하는 항목
        // 수를 그대로 읽으면 색상표 없이 저장된 DIB 에서 다음 레코드를 침범한다.
        let color_table_room = record_bytes.map(|bytes| {
            bytes.saturating_sub(consumed_bytes.saturating_add(dib_header_info.size()))
        });
        let (colors, c) =
            Colors::parse_within(buf, color_usage, &dib_header_info, color_table_room)?;
        consumed_bytes += c;

        //  TODO: Not written in [MS-WMF] how to parse this field.
        let undefined_space = vec![];
        let (a_data, c) = crate::wmf::parser::read_variable(buf, dib_header_info.size())?;
        consumed_bytes += c;

        Ok((
            Self {
                dib_header_info,
                colors,
                bitmap_buffer: BitmapBuffer {
                    undefined_space,
                    a_data,
                },
            },
            consumed_bytes,
        ))
    }
}

#[derive(Clone, Debug)]
pub enum Colors {
    PaletteIndices(Vec<u16>),
    RGBQuad(Vec<crate::wmf::parser::RGBQuad>),
    RGBTriple(Vec<crate::wmf::parser::RGBTriple>),
    Null,
}

impl Colors {
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        dib_header_info: &crate::wmf::parser::BitmapInfoHeader,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        Self::parse_within(buf, color_usage, dib_header_info, None)
    }

    /// `table_room` 은 담는 레코드에서 색상표가 차지할 수 있는 바이트 수다(`None` 은 제한 없음).
    fn parse_within<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        dib_header_info: &crate::wmf::parser::BitmapInfoHeader,
        table_room: Option<usize>,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        if matches!(color_usage, crate::wmf::parser::ColorUsage::DIB_PAL_INDICES)
            || matches!(
                dib_header_info.bit_count(),
                crate::wmf::parser::BitCount::BI_BITCOUNT_0
            )
        {
            return Ok((Colors::Null, 0));
        }

        if matches!(
            dib_header_info,
            crate::wmf::parser::BitmapInfoHeader::Core { .. }
        ) {
            return Self::parse_with_core_header(buf, color_usage, dib_header_info, table_room);
        }

        Self::parse_with_info_header(buf, color_usage, dib_header_info, table_room)
    }

    fn parse_with_core_header<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        dib_header_info: &crate::wmf::parser::BitmapInfoHeader,
        table_room: Option<usize>,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        assert!(matches!(
            dib_header_info,
            crate::wmf::parser::BitmapInfoHeader::Core { .. }
        ));

        // core header does not have color_used field.
        if matches!(
            dib_header_info.bit_count(),
            crate::wmf::parser::BitCount::BI_BITCOUNT_5
        ) {
            return Ok((Colors::Null, 0));
        }

        Self::parse_from_color_usage(
            buf,
            color_usage,
            dib_header_info.color_used() as usize,
            true,
            table_room,
        )
    }

    fn parse_with_info_header<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        dib_header_info: &crate::wmf::parser::BitmapInfoHeader,
        table_room: Option<usize>,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        assert!(matches!(
            dib_header_info,
            crate::wmf::parser::BitmapInfoHeader::Info { .. }
                | crate::wmf::parser::BitmapInfoHeader::V4 { .. }
                | crate::wmf::parser::BitmapInfoHeader::V5 { .. }
        ));

        match dib_header_info.bit_count() {
            crate::wmf::parser::BitCount::BI_BITCOUNT_0 => unreachable!(),
            crate::wmf::parser::BitCount::BI_BITCOUNT_1
            | crate::wmf::parser::BitCount::BI_BITCOUNT_2
            | crate::wmf::parser::BitCount::BI_BITCOUNT_3 => Self::parse_from_color_usage(
                buf,
                color_usage,
                dib_header_info.color_used() as usize,
                false,
                table_room,
            ),
            crate::wmf::parser::BitCount::BI_BITCOUNT_5 => {
                // ignore result
                let (_, bytes) = Self::parse_from_color_usage(
                    buf,
                    color_usage,
                    dib_header_info.color_used() as usize,
                    false,
                    table_room,
                )?;

                Ok((Colors::Null, bytes))
            }

            crate::wmf::parser::BitCount::BI_BITCOUNT_4
            | crate::wmf::parser::BitCount::BI_BITCOUNT_6 => {
                match &dib_header_info {
                    crate::wmf::parser::BitmapInfoHeader::Core(_) => unreachable!(),
                    crate::wmf::parser::BitmapInfoHeader::Info(
                        crate::wmf::parser::BitmapInfoHeaderInfo { compression, .. },
                    )
                    | crate::wmf::parser::BitmapInfoHeader::V4(
                        crate::wmf::parser::BitmapInfoHeaderV4 { compression, .. },
                    )
                    | crate::wmf::parser::BitmapInfoHeader::V5(
                        crate::wmf::parser::BitmapInfoHeaderV5 { compression, .. },
                    ) => match compression {
                        crate::wmf::parser::Compression::BI_RGB => {
                            // ignore result
                            let (_, bytes) = Self::parse_from_color_usage(
                                buf,
                                color_usage,
                                dib_header_info.color_used() as usize,
                                false,
                                table_room,
                            )?;

                            Ok((Colors::Null, bytes))
                        }
                        crate::wmf::parser::Compression::BI_BITFIELDS => {
                            Self::parse_from_color_usage(
                                buf,
                                color_usage,
                                dib_header_info.color_used() as usize,
                                false,
                                table_room,
                            )
                        }
                        _ => Ok((Colors::Null, 0)),
                    },
                }
            }
        }
    }

    fn parse_from_color_usage<R: crate::wmf::Read>(
        buf: &mut R,
        color_usage: crate::wmf::parser::ColorUsage,
        colors_length: usize,
        core: bool,
        table_room: Option<usize>,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        // colors_length는 BITMAPINFOHEADER의 colors_used(검증되지 않은 u32)에서
        // 온다. 색상표가 존재하는 비트 심도(<=8bpp)에서 사양상 최대 항목 수는
        // 2^8=256이므로 이를 상한으로 둔다. 상한이 없으면 Vec::with_capacity가
        // 거대한 값을 그대로 예약해 OOM abort로 이어진다 (doc_info.rs tab_count,
        // emf parse_points16 #2992와 동일한 클래스).
        let colors_length = colors_length.min(256);
        let entry_bytes = match color_usage {
            crate::wmf::parser::ColorUsage::DIB_RGB_COLORS if core => 3,
            crate::wmf::parser::ColorUsage::DIB_RGB_COLORS => 4,
            _ => 2,
        };
        let colors_length =
            table_room.map_or(colors_length, |room| colors_length.min(room / entry_bytes));
        let mut consumed_bytes: usize = 0;

        match color_usage {
            crate::wmf::parser::ColorUsage::DIB_RGB_COLORS if core => {
                let mut table = Vec::with_capacity(colors_length);

                for _ in 0..colors_length {
                    let (v, c) = crate::wmf::parser::RGBTriple::parse(buf)?;

                    consumed_bytes += c;
                    table.push(v);
                }

                Ok((Colors::RGBTriple(table), consumed_bytes))
            }
            crate::wmf::parser::ColorUsage::DIB_RGB_COLORS => {
                let mut table = Vec::with_capacity(colors_length);

                for _ in 0..colors_length {
                    let (v, c) = crate::wmf::parser::RGBQuad::parse(buf)?;

                    consumed_bytes += c;
                    table.push(v);
                }

                Ok((Colors::RGBQuad(table), consumed_bytes))
            }
            crate::wmf::parser::ColorUsage::DIB_PAL_COLORS => {
                let mut table = Vec::with_capacity(colors_length);

                for _ in 0..colors_length {
                    let (v, c) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;

                    consumed_bytes += c;
                    table.push(v);
                }

                Ok((Colors::PaletteIndices(table), consumed_bytes))
            }
            crate::wmf::parser::ColorUsage::DIB_PAL_INDICES => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct BitmapBuffer {
    /// UndefinedSpace (variable): An optional field that MUST be ignored. If
    /// this DIB is a packed bitmap, this field MUST NOT be present.
    pub undefined_space: Vec<u8>,
    /// aData (variable): An array of bytes that define the image.
    ///
    /// The size and format of this data is determined by information in the
    /// DIBHeaderInfo field. If it is a BitmapCoreHeader, the size in bytes
    /// MUST be calculated as follows:
    ///
    /// ```text
    /// (((Width * Planes * BitCount + 31) & ~31) / 8) * abs(Height)
    /// ```
    ///
    /// This formula SHOULD also be used to calculate the size of aData when
    /// DIBHeaderInfo is a BitmapInfoHeader Object, using values from that
    /// object, but only if its Compression value is BI_RGB, BI_BITFIELDS, or
    /// BI_CMYK.
    ///
    /// Otherwise, the size of aData MUST be the BitmapInfoHeader Object value
    /// ImageSize.
    pub a_data: Vec<u8>,
}

impl core::fmt::Debug for BitmapBuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BitmapBuffer")
            .field(
                "undefined_space",
                &format!("[u8; {}]", self.undefined_space.len()),
            )
            .field("a_data", &format!("[u8; {}]", self.a_data.len()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// colors_length가 BITMAPINFOHEADER의 검증되지 않은 colors_used(u32)에서
    /// 그대로 오는 경우, 거대한 값(u32::MAX)을 넘겨도 Vec::with_capacity가
    /// 그 값을 그대로 예약하지 않고(256 상한) 데이터 부족으로 정상적으로
    /// 에러를 반환해야 한다 (OOM abort 없이 graceful degradation).
    #[test]
    fn parse_from_color_usage_bounds_huge_colors_length() {
        let mut data: &[u8] = &[0u8; 4]; // RGBQuad 1개 분량도 안 되는 소량 데이터
        let result = Colors::parse_from_color_usage(
            &mut data,
            crate::wmf::parser::ColorUsage::DIB_RGB_COLORS,
            u32::MAX as usize,
            false,
            None,
        );

        assert!(result.is_err());
    }
}
