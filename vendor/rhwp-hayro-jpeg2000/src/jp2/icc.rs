#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub(crate) enum ICCColorSpace {
    Xyz,
    Lab,
    Luv,
    Ycbr,
    Yxy,
    Lms,
    Rgb,
    Gray,
    Hsv,
    Hls,
    Cmyk,
    Cmy,
    OneClr,
    ThreeClr,
    FourClr,
    // There are more, but those should be the most important
    // ones.
}

impl ICCColorSpace {
    pub(crate) fn num_components(&self) -> u8 {
        match self {
            Self::Xyz => 3,
            Self::Lab => 3,
            Self::Luv => 3,
            Self::Ycbr => 3,
            Self::Yxy => 3,
            Self::Lms => 3,
            Self::Rgb => 3,
            Self::Gray => 1,
            Self::Hsv => 3,
            Self::Hls => 3,
            Self::Cmyk => 4,
            Self::Cmy => 3,
            Self::OneClr => 1,
            Self::ThreeClr => 3,
            Self::FourClr => 4,
        }
    }
}

impl TryFrom<u32> for ICCColorSpace {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        rhwp_codec_resource::work(1).map_err(|_| ())?;
        match value {
            0x58595A20 => Ok(Self::Xyz),
            0x4C616220 => Ok(Self::Lab),
            0x4C757620 => Ok(Self::Luv),
            0x59436272 => Ok(Self::Ycbr),
            0x59787920 => Ok(Self::Yxy),
            0x4C4D5320 => Ok(Self::Lms),
            0x52474220 => Ok(Self::Rgb),
            0x47524159 => Ok(Self::Gray),
            0x48535620 => Ok(Self::Hsv),
            0x484C5320 => Ok(Self::Hls),
            0x434D594B => Ok(Self::Cmyk),
            0x434D5920 => Ok(Self::Cmy),
            0x31434C52 => Ok(Self::OneClr),
            0x33434C52 => Ok(Self::ThreeClr),
            0x34434C52 => Ok(Self::FourClr),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub(crate) struct ICCMetadata {
    pub(crate) color_space: ICCColorSpace,
}

impl ICCMetadata {
    pub(crate) fn from_data(data: &[u8]) -> Option<Self> {
        rhwp_codec_resource::work(1).ok()?;
        let color_space = {
            let marker = u32::from_be_bytes(data.get(16..20)?.try_into().ok()?);
            ICCColorSpace::try_from(marker).ok()?
        };

        Some(Self { color_space })
    }
}
