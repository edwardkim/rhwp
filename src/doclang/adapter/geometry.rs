//! `CommonObjAttr` → [`ir::Geometry`] lowering.
//!
//! This is a straight HWPUNIT passthrough: the rhwp object-attribute fields are
//! copied verbatim (no unit conversion) so that the DocLang v2 `<location>`
//! element can later consume the raw spatial values.  v1 output ignores
//! geometry, but capturing it here means the adapter does not need rewriting
//! when v2 support lands.
//!
//! ## Field provenance (rhwp `CommonObjAttr`, `src/model/shape.rs:31-90`)
//!
//! - `width` / `height`        — object box size  (`HwpUnit` = `u32`)
//! - `horizontal_offset`       — `h_offset`       (`HwpUnit` = `u32`)
//! - `vertical_offset`         — `v_offset`       (`HwpUnit` = `u32`)
//! - `treat_as_char: bool`     — inline-anchored when `true`, floating otherwise
//!
//! The IR [`ir::Geometry`] stores these as `i32`; HWP sizes/offsets are always
//! non-negative and well within `i32::MAX`, so the `as i32` casts are lossless
//! in practice (and saturate harmlessly on the theoretical overflow).

use crate::model::shape::CommonObjAttr;

use crate::doclang::ir;

/// Lower a rhwp [`CommonObjAttr`] into the crate-IR [`ir::Geometry`].
///
/// Pure HWPUNIT passthrough — see the module docs for field mapping.
pub(crate) fn from_common_attr(attr: &CommonObjAttr) -> ir::Geometry {
    ir::Geometry {
        width: attr.width as i32,
        height: attr.height as i32,
        h_offset: attr.horizontal_offset as i32,
        v_offset: attr.vertical_offset as i32,
        treat_as_char: attr.treat_as_char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_preserves_dimensions_and_offsets() {
        let attr = CommonObjAttr {
            width: 7200,
            height: 3600,
            horizontal_offset: 100,
            vertical_offset: 250,
            treat_as_char: true,
            ..Default::default()
        };
        let geo = from_common_attr(&attr);
        assert_eq!(geo.width, 7200);
        assert_eq!(geo.height, 3600);
        assert_eq!(geo.h_offset, 100);
        assert_eq!(geo.v_offset, 250);
        assert!(geo.treat_as_char);
    }

    #[test]
    fn floating_object_reports_not_treat_as_char() {
        let attr = CommonObjAttr {
            treat_as_char: false,
            ..Default::default()
        };
        assert!(!from_common_attr(&attr).treat_as_char);
    }
}
