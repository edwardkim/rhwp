/// The MetafileType Enumeration specifies where the metafile is stored.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum MetafileType {
    /// Metafile is stored in memory.
    MEMORYMETAFILE = 0x0001,
    /// Metafile is stored on disk.
    DISKMETAFILE = 0x0002,
}

crate::wmf::parser::constants::impl_parser!(MetafileType, u16);
