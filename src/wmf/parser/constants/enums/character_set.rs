use crate::wmf::imports::*;

/// The CharacterSet Enumeration defines the possible sets of character glyphs
/// that are defined in fonts for graphics output.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum CharacterSet {
    /// Specifies the English character set.
    ANSI_CHARSET = 0x00000000,
    /// Specifies a character set based on the current system locale; for
    /// example, when the system locale is United States English, the default
    /// character set is ANSI_CHARSET.
    DEFAULT_CHARSET = 0x00000001,
    /// Specifies a character set of symbols.
    SYMBOL_CHARSET = 0x00000002,
    /// This value is not supported.
    MAC_CHARSET = 0x0000004D,
    /// Specifies the Japanese character set.
    SHIFTJIS_CHARSET = 0x00000080,
    /// Also spelled "Hangeul". Specifies the Hangul Korean character set.
    HANGUL_CHARSET = 0x00000081,
    /// Also spelled "Johap". Specifies the Johab Korean character set.
    JOHAB_CHARSET = 0x00000082,
    /// Specifies the "simplified" Chinese character set for People's Republic
    /// of China.
    GB2312_CHARSET = 0x00000086,
    /// Specifies the "traditional" Chinese character set, used mostly in
    /// Taiwan and in the Hong Kong and Macao Special Administrative Regions.
    CHINESEBIG5_CHARSET = 0x00000088,
    /// Specifies the Greek character set.
    GREEK_CHARSET = 0x000000A1,
    /// Specifies the Turkish character set.
    TURKISH_CHARSET = 0x000000A2,
    /// Specifies the Vietnamese character set.
    VIETNAMESE_CHARSET = 0x000000A3,
    /// Specifies the Hebrew character set
    HEBREW_CHARSET = 0x000000B1,
    /// Specifies the Arabic character set
    ARABIC_CHARSET = 0x000000B2,
    /// Specifies the Baltic (Northeastern European) character set
    BALTIC_CHARSET = 0x000000BA,
    /// Specifies the Russian Cyrillic character set.
    RUSSIAN_CHARSET = 0x000000CC,
    /// Specifies the Thai character set.
    THAI_CHARSET = 0x000000DE,
    /// Specifies a Eastern European character set.
    EASTEUROPE_CHARSET = 0x000000EE,
    /// Specifies a mapping to one of the OEM code pages, according to the
    /// current system locale setting.
    OEM_CHARSET = 0x000000FF,
}

crate::wmf::parser::constants::impl_parser!(CharacterSet, u8);

impl From<CharacterSet> for &'static encoding_rs::Encoding {
    fn from(v: CharacterSet) -> Self {
        let codepage_id = codepage_table().get(&v).copied().unwrap_or(1252);
        codepage::to_encoding(codepage_id).unwrap_or(encoding_rs::REPLACEMENT)
    }
}

static SYMBOL_CHARSET_TABLE_INITIALIZED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);
static mut SYMBOL_CHARSET_TABLE: BTreeMap<u8, char> = BTreeMap::new();

#[rustfmt::skip]
pub(in crate::wmf::parser) fn symbol_charset_table(
) -> &'static BTreeMap<u8, char> {
    if !SYMBOL_CHARSET_TABLE_INITIALIZED.load(core::sync::atomic::Ordering::Acquire) {
        SYMBOL_CHARSET_TABLE_INITIALIZED.store(true, core::sync::atomic::Ordering::Release);

        unsafe {
            // via: https://en.wikipedia.org/wiki/Symbol_(typeface)
            SYMBOL_CHARSET_TABLE = BTreeMap::from_iter([
                // 2x
                (0x20, ' '), (0x21, '!'), (0x22, '∀'), (0x23, '#'),
                (0x24, '∃'), (0x25, '%'), (0x26, '&'), (0x27, '∍'),
                (0x28, '('), (0x29, ')'), (0x2A, '*'), (0x2B, '+'),
                (0x2C, ','), (0x2D, '-'), (0x2E, '.'), (0x2F, '/'),
                // 3x
                (0x30, '0'), (0x31, '1'), (0x32, '2'), (0x33, '3'),
                (0x34, '4'), (0x35, '5'), (0x36, '6'), (0x37, '7'),
                (0x38, '8'), (0x39, '9'), (0x3A, ':'), (0x3B, ';'),
                (0x3C, '<'), (0x3D, '='), (0x3E, '>'), (0x3F, '?'),
                // 4x
                (0x40, '≅'), (0x41, 'Α'), (0x42, 'Β'), (0x43, 'Χ'),
                (0x44, 'Δ'), (0x45, 'Ε'), (0x46, 'Φ'), (0x47, 'Γ'),
                (0x48, 'Η'), (0x49, 'Ι'), (0x4A, 'ϑ'), (0x4B, 'Κ'),
                (0x4C, 'Λ'), (0x4D, 'Μ'), (0x4E, 'Ν'), (0x4F, 'Ο'),
                // 5x
                (0x50, 'Π'), (0x51, 'Θ'), (0x52, 'Ρ'), (0x53, 'Σ'),
                (0x54, 'Τ'), (0x55, 'Υ'), (0x56, 'ς'), (0x57, 'Ω'),
                (0x58, 'Ξ'), (0x59, 'Ψ'), (0x5A, 'Ζ'), (0x5B, '['),
                (0x5C, '∴'), (0x5D, ']'), (0x5E, '⊥'), (0x5F, '_'),
                // 6x
                (0x60, '‾'), (0x61, 'α'), (0x62, 'β'), (0x63, 'χ'),
                (0x64, 'δ'), (0x65, 'ε'), (0x66, 'φ'), (0x67, 'γ'),
                (0x68, 'η'), (0x69, 'ι'), (0x6A, 'ϕ'), (0x6B, 'κ'),
                (0x6C, 'λ'), (0x6D, 'μ'), (0x6E, 'ν'), (0x6F, 'ο'),
                // 7x
                (0x70, 'π'), (0x71, 'θ'), (0x72, 'ρ'), (0x73, 'σ'),
                (0x74, 'τ'), (0x75, 'υ'), (0x76, 'ϖ'), (0x77, 'ω'),
                (0x78, 'ξ'), (0x79, 'ψ'), (0x7A, 'ζ'), (0x7B, '{'),
                (0x7C, '|'), (0x7D, '}'), (0x7E, '~'),
                // Ax
                (0xA0, '€'), (0xA1, 'ϒ'), (0xA2, '′'), (0xA3, '≤'),
                (0xA4, '⁄'), (0xA5, '∞'), (0xA6, 'ƒ'), (0xA7, '♣'),
                (0xA8, '♦'), (0xA9, '♥'), (0xAA, '♠'), (0xAB, '↔'),
                (0xAC, '←'), (0xAD, '↑'), (0xAE, '→'), (0xAF, '↓'),
                // Bx
                (0xB0, '°'), (0xB1, '±'), (0xB2, '″'), (0xB3, '≥'),
                (0xB4, '×'), (0xB5, '∝'), (0xB6, '∂'), (0xB7, '•'),
                (0xB8, '÷'), (0xB9, '≠'), (0xBA, '≡'), (0xBB, '≈'),
                (0xBC, '…'), (0xBD, '⏐'), (0xBE, '⎯'), (0xBF, '↵'),
                // Cx
                (0xC0, 'ℵ'), (0xC1, 'ℑ'), (0xC2, 'ℜ'), (0xC3, '℘'),
                (0xC4, '⊗'), (0xC5, '⊕'), (0xC6, '∅'), (0xC7, '∩'),
                (0xC8, '∪'), (0xC9, '⊃'), (0xCA, '⊇'), (0xCB, '⊄'),
                (0xCC, '⊂'), (0xCD, '⊆'), (0xCE, '∈'), (0xCF, '∉'),
                //Dx
                (0xD0, '∠'), (0xD1, '∇'), (0xD2, '®'), (0xD3, '©'),
                (0xD4, '™'), (0xD5, '∏'), (0xD6, '√'), (0xD7, '⋅'),
                (0xD8, '¬'), (0xD9, '∧'), (0xDA, '∨'), (0xDB, '⇔'),
                (0xDC, '⇐'), (0xDD, '⇑'), (0xDE, '⇒'), (0xDF, '⇓'),
                // Ex
                (0xE0, '◊'), (0xE1, '⟨'), (0xE2, '®'), (0xE3, '©'),
                (0xE4, '™'), (0xE5, '∑'), (0xE6, '⎛'), (0xE7, '⎜'),
                (0xE8, '⎝'), (0xE9, '⎡'), (0xEA, '⎢'), (0xEB, '⎣'),
                (0xEC, '⎧'), (0xED, '⎨'), (0xEE, '⎩'), (0xEF, '⎪'),
                // Fx
                (0xF1, '⟩'), (0xF2, '∫'), (0xF3, '⌠'), (0xF4, '⎮'),
                (0xF5, '⌡'), (0xF6, '⎞'), (0xF7, '⎟'), (0xF8, '⎠'),
                (0xF9, '⎤'), (0xFA, '⎥'), (0xFB, '⎦'), (0xFC, '⎫'),
                (0xFD, '⎬'), (0xFE, '⎭'),
            ]);
        }
    }

    // SAFETY: Mutable access is not possible after state has been
    // initialized.
    #[allow(clippy::deref_addrof)]
    unsafe { &*&raw const SYMBOL_CHARSET_TABLE }
}

static CODEPAGE_TABLE_INITIALIZED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);
static mut CODEPAGE_TABLE: BTreeMap<CharacterSet, u16> = BTreeMap::new();

fn codepage_table() -> &'static BTreeMap<CharacterSet, u16> {
    if !CODEPAGE_TABLE_INITIALIZED.load(core::sync::atomic::Ordering::Acquire) {
        CODEPAGE_TABLE_INITIALIZED.store(true, core::sync::atomic::Ordering::Release);

        unsafe {
            // via: https://en.wikipedia.org/wiki/Code_page
            CODEPAGE_TABLE = BTreeMap::from_iter([
                (CharacterSet::ANSI_CHARSET, 1252),
                (CharacterSet::SYMBOL_CHARSET, 42),
                (CharacterSet::SHIFTJIS_CHARSET, 932),
                (CharacterSet::HANGUL_CHARSET, 949),
                (CharacterSet::JOHAB_CHARSET, 1361),
                (CharacterSet::GB2312_CHARSET, 936),
                (CharacterSet::CHINESEBIG5_CHARSET, 950),
                (CharacterSet::GREEK_CHARSET, 1253),
                (CharacterSet::TURKISH_CHARSET, 1254),
                (CharacterSet::VIETNAMESE_CHARSET, 1258),
                (CharacterSet::HEBREW_CHARSET, 1255),
                (CharacterSet::ARABIC_CHARSET, 1256),
                (CharacterSet::BALTIC_CHARSET, 1257),
                (CharacterSet::RUSSIAN_CHARSET, 1251),
                (CharacterSet::THAI_CHARSET, 874),
                (CharacterSet::EASTEUROPE_CHARSET, 1250),
                (CharacterSet::OEM_CHARSET, 437),
            ]);
        }
    }

    // SAFETY: Mutable access is not possible after state has been
    // initialized.
    #[allow(clippy::deref_addrof)]
    unsafe {
        &*&raw const CODEPAGE_TABLE
    }
}
