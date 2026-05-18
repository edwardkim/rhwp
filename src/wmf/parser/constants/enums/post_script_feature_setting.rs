/// The PostScriptFeatureSetting Enumeration defines values that are used to
/// retrieve information about specific features in a PostScript printer driver.
/// (Windows NT 3.1, Windows NT 3.5, Windows NT 3.51, Windows 95, Windows 98,
/// and Windows Millennium Edition: This functionality is not supported.)
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u32)]
pub enum PostScriptFeatureSetting {
    /// Specifies the n-up printing (page layout) setting.
    FEATURESETTING_NUP = 0x00000000,
    /// Specifies PostScript driver output options.
    FEATURESETTING_OUTPUT = 0x00000001,
    /// Specifies the language level.
    FEATURESETTING_PSLEVEL = 0x00000002,
    /// Specifies custom paper parameters.
    FEATURESETTING_CUSTPAPER = 0x00000003,
    /// Specifies the mirrored output setting.
    FEATURESETTING_MIRROR = 0x00000004,
    /// Specifies the negative output setting.
    FEATURESETTING_NEGATIVE = 0x00000005,
    /// Specifies the output protocol setting.
    FEATURESETTING_PROTOCOL = 0x00000006,
    /// Specifies the start of a range of values that a driver can use for
    /// retrieving data concerning proprietary features. (Windows NT 4.0 and
    /// Windows 2000: This functionality is not supported.)
    FEATURESETTING_PRIVATE_BEGIN = 0x00001000,
    /// Specifies the end of a range of values that a driver can use for
    /// retrieving data concerning proprietary features. (Windows NT 4.0 and
    /// Windows 2000: This functionality is not supported.)
    FEATURESETTING_PRIVATE_END = 0x00001FFF,
}

crate::wmf::parser::constants::impl_parser!(PostScriptFeatureSetting, u32);
