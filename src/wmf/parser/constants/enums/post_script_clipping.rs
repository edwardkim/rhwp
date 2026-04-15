/// The PostScriptClipping Enumeration defines functions that can be applied to
/// the clipping path used for PostScript output.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum PostScriptClipping {
    /// Saves the current PostScript clipping path.
    CLIP_SAVE = 0x0000,
    /// Restores the PostScript clipping path to the last clipping path that
    /// was saved by a previous CLIP_SAVE function applied by a CLIP_TO_PATH
    /// Record.
    CLIP_RESTORE = 0x0001,
    /// Intersects the current PostScript clipping path with the current
    /// clipping path and saves the result as the new PostScript clipping path.
    CLIP_INCLUSIVE = 0x0002,
}

crate::wmf::parser::constants::impl_parser!(PostScriptClipping, u16);
