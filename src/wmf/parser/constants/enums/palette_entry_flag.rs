/// The PaletteEntryFlag Enumeration specifies how the palette entry is used.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum PaletteEntryFlag {
    /// Specifies that the logical palette entry be used for palette animation.
    /// This value prevents other windows from matching colors to the palette
    /// entry because the color frequently changes. If an unused system-palette
    /// entry is available, the color is placed in that entry. Otherwise, the
    /// color is not available for animation.
    PC_RESERVED = 0x01,
    /// Specifies that the low-order word of the logical palette entry
    /// designates a hardware palette index. This value allows the application
    /// to show the contents of the display device palette.
    PC_EXPLICIT = 0x02,
    /// Specifies that the color be placed in an unused entry in the system
    /// palette instead of being matched to an existing color in the system
    /// palette. If there are no unused entries in the system palette, the
    /// color is matched normally. Once this color is in the system palette,
    /// colors in other logical palettes can be matched to this color.
    /// PC_RESERVED = 0x01,
    PC_NOCOLLAPSE = 0x04,
}

crate::wmf::parser::constants::impl_parser!(PaletteEntryFlag, u8);
