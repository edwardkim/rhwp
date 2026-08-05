/// The PitchAndFamily Object specifies the pitch and family properties of a
/// Font Object. Pitch refers to the width of the characters, and family refers
/// to the general appearance of a font.
#[derive(Clone, Debug)]
pub struct PitchAndFamily {
    /// Family (4 bits): A property of a font that describes its general
    /// appearance. This MUST be a value in the FamilyFont Enumeration.
    pub family: crate::wmf::parser::FamilyFont,
    /// Pitch (2 bits): A property of a font that describes the pitch, of the
    /// characters. This MUST be a value in the PitchFont Enumeration.
    pub pitch: crate::wmf::parser::PitchFont,
}

impl PitchAndFamily {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (byte, consumed_bytes) = crate::wmf::parser::read_u8_from_le_bytes(buf)?;

        let family = byte >> 4;
        let Some(family) = crate::wmf::parser::FamilyFont::from_repr(byte >> 4) else {
            return Err(crate::wmf::parser::ParseError::UnexpectedEnumValue {
                cause: format!("unexpected value as FamilyFont: {family:#04X}"),
            });
        };

        // 2비트 필드의 유효값은 0~2 뿐이라 3은 스펙 밖이지만, 실문서 WMF 가 3을
        // 담아 온다 (#4063). pitch 는 렌더 힌트라 여기서 파싱을 실패시키면 그림
        // 전체가 변환 불가로 번진다 — 미정의 값은 DEFAULT_PITCH 로 관용한다.
        let pitch = crate::wmf::parser::PitchFont::from_repr(byte & 0b00000011)
            .unwrap_or(crate::wmf::parser::PitchFont::DEFAULT_PITCH);

        Ok((Self { family, pitch }, consumed_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 실문서 WMF 가 pitch 비트에 스펙 밖 값 3을 담아 온다 (#4063). 여기서
    /// 파싱을 실패시키면 폰트 하나 때문에 그림 전체가 변환 불가로 번지므로,
    /// 미정의 pitch 는 DEFAULT_PITCH 로 관용해야 한다.
    #[test]
    fn undefined_pitch_bits_fall_back_to_default_instead_of_failing() {
        // family=FF_SWISS(0x02) 상위 니블 + pitch=3(미정의) 하위 2비트.
        let mut input: &[u8] = &[0x23];
        let (paf, consumed) =
            PitchAndFamily::parse(&mut input).expect("미정의 pitch 가 파싱을 죽이면 안 된다");
        assert_eq!(consumed, 1);
        assert_eq!(paf.pitch, crate::wmf::parser::PitchFont::DEFAULT_PITCH);
    }
}
