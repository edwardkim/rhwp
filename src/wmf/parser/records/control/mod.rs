//! Implementation of the definitions in Section 2.3.2 of the WMF
//! specifications.

mod eof;
mod header;
mod placeable;

pub use self::{eof::*, header::*, placeable::*};

#[derive(Clone, Debug)]
pub enum MetafileHeader {
    StartsWithPlaceable(
        crate::wmf::parser::META_PLACEABLE,
        crate::wmf::parser::META_HEADER,
    ),
    StartsWithHeader(crate::wmf::parser::META_HEADER),
}

impl MetafileHeader {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (mut key, mut consumed_bytes) = crate::wmf::parser::read_u32_from_le_bytes(buf)?;

        let placeable = if key == 0x9AC6CDD7 {
            let (v, c) = crate::wmf::parser::META_PLACEABLE::parse(buf, key)?;
            let (k, key_bytes) = crate::wmf::parser::read_u32_from_le_bytes(buf)?;

            (key, consumed_bytes) = (k, c + key_bytes);

            Some(v)
        } else {
            None
        };

        let (header, c) = crate::wmf::parser::META_HEADER::parse(buf, key)?;
        consumed_bytes += c;

        Ok(if let Some(placeable) = placeable {
            (Self::StartsWithPlaceable(placeable, header), consumed_bytes)
        } else {
            (Self::StartsWithHeader(header), consumed_bytes)
        })
    }
}
