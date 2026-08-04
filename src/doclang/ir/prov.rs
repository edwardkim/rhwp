//! Block provenance and resolved geometry for the v2 `<location>` feature.
//!
//! This module is **rhwp-agnostic**: a [`Prov`] is a plain triple of model
//! indices `(section, paragraph, control)` that the parser adapter stamps onto
//! the location-eligible blocks while it still knows where each block came
//! from. The geometry pass (in `parser_adapter`, the only rhwp-aware layer)
//! later builds a `Prov -> Location` map from the render tree, and the writer
//! consumes the [`Location`] without ever touching rhwp types.
//!
//! Keeping `Prov` as bare indices (rather than carrying rhwp node handles)
//! means the IR, writer, and loss modules stay free of any rhwp dependency.

/// Model provenance of a block: where it originated in the parsed HWP document.
///
/// `section` / `para` index into `Document.sections[section].paragraphs[para]`;
/// `ctrl`, when present, indexes that paragraph's `controls[]` (tables,
/// pictures, equations, …). Text blocks (paragraph / heading / list item)
/// carry `ctrl = None`; object blocks carry the owning control index.
///
/// These indices are the join key against the render-tree provenance fields
/// (`section_index`, `para_index`, `control_index`) that rhwp embeds on
/// `TextLine` / `Table` / `Image` / `Equation` nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Prov {
    /// Zero-based section index.
    pub section: usize,
    /// Zero-based paragraph index within the section.
    pub para: usize,
    /// Zero-based control index within the paragraph, for object blocks.
    pub ctrl: Option<usize>,
}

impl Prov {
    /// A text-block provenance (no owning control).
    pub fn text(section: usize, para: usize) -> Self {
        Prov {
            section,
            para,
            ctrl: None,
        }
    }

    /// An object-block provenance referencing `controls[ctrl]`.
    pub fn object(section: usize, para: usize, ctrl: usize) -> Self {
        Prov {
            section,
            para,
            ctrl: Some(ctrl),
        }
    }
}

/// A resolved bounding box, normalised to the DocLang 0..=512 location grid.
///
/// Values are page-relative: `x_*` are normalised against the page width and
/// `y_*` against the page height, then `round(coord / page_dim * 512)` clamped
/// to `0..=512`. The writer emits them as four `<location>` elements in the
/// order `x_min, y_min, x_max, y_max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// Zero-based page index that owns this box.
    ///
    /// Multi-page blocks use the first page on which the block appears.
    pub page: u32,
    /// Left edge (normalised 0..=512).
    pub x_min: u16,
    /// Top edge (normalised 0..=512).
    pub y_min: u16,
    /// Right edge (normalised 0..=512).
    pub x_max: u16,
    /// Bottom edge (normalised 0..=512).
    pub y_max: u16,
}

/// The DocLang location resolution: coordinates are expressed on a 0..=512 grid.
pub const LOCATION_RESOLUTION: u16 = 512;

/// Map from block provenance to its resolved, normalised bounding box.
///
/// Produced by the geometry pass (rhwp-aware) and consumed by the writer
/// (rhwp-agnostic): the writer looks up each block's [`Prov`] and, on a hit,
/// emits the four `<location>` head elements. Empty when location is disabled.
pub type LocationMap = std::collections::HashMap<Prov, Location>;

/// Normalise one pixel coordinate against a page dimension onto the 0..=512
/// grid: `clamp(round(coord / page_dim * 512), 0, 512)`.
///
/// A non-positive or non-finite `page_dim` yields `0` (degenerate page; nothing
/// meaningful to normalise against).
pub fn normalize(coord: f64, page_dim: f64) -> u16 {
    if !page_dim.is_finite() || page_dim <= 0.0 || !coord.is_finite() {
        return 0;
    }
    let scaled = (coord / page_dim) * LOCATION_RESOLUTION as f64;
    let rounded = scaled.round();
    if rounded <= 0.0 {
        0
    } else if rounded >= LOCATION_RESOLUTION as f64 {
        LOCATION_RESOLUTION
    } else {
        rounded as u16
    }
}

impl Location {
    /// Build a normalised [`Location`] from a pixel-space box `(x, y, w, h)` and
    /// the page dimensions in pixels. `x` normalises against `page_w`, `y`
    /// against `page_h`.
    pub fn from_px_box(x: f64, y: f64, w: f64, h: f64, page_w: f64, page_h: f64) -> Self {
        Self::from_px_box_on_page(0, x, y, w, h, page_w, page_h)
    }

    /// Build a normalised [`Location`] from a pixel-space box on `page`.
    pub fn from_px_box_on_page(
        page: u32,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        page_w: f64,
        page_h: f64,
    ) -> Self {
        Location {
            page,
            x_min: normalize(x, page_w),
            y_min: normalize(y, page_h),
            x_max: normalize(x + w, page_w),
            y_max: normalize(y + h, page_h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic_rounding() {
        // half of the page → 256
        assert_eq!(normalize(50.0, 100.0), 256);
        // exact top
        assert_eq!(normalize(0.0, 100.0), 0);
        // exact bottom maps to the grid max
        assert_eq!(normalize(100.0, 100.0), 512);
    }

    #[test]
    fn normalize_clamps_out_of_range() {
        // beyond the page → clamped to 512
        assert_eq!(normalize(150.0, 100.0), 512);
        // negative → clamped to 0
        assert_eq!(normalize(-10.0, 100.0), 0);
    }

    #[test]
    fn normalize_rounds_to_nearest() {
        // 1123 px page, coord 76 → 76/1123*512 = 34.65 → 35
        assert_eq!(normalize(76.0, 1123.0), 35);
        // coord 57 → 57/1123*512 = 25.98 → 26
        assert_eq!(normalize(57.0, 1123.0), 26);
    }

    #[test]
    fn normalize_degenerate_page_is_zero() {
        assert_eq!(normalize(10.0, 0.0), 0);
        assert_eq!(normalize(10.0, -5.0), 0);
        assert_eq!(normalize(f64::NAN, 100.0), 0);
    }

    #[test]
    fn from_px_box_normalises_each_axis_independently() {
        // 800x1000 page, box at (400,500) size 200x100.
        let loc = Location::from_px_box(400.0, 500.0, 200.0, 100.0, 800.0, 1000.0);
        assert_eq!(loc.page, 0);
        assert_eq!(loc.x_min, 256); // 400/800*512
        assert_eq!(loc.y_min, 256); // 500/1000*512
        assert_eq!(loc.x_max, 384); // 600/800*512
        assert_eq!(loc.y_max, 307); // 600/1000*512 = 307.2 → 307
    }

    #[test]
    fn from_px_box_on_page_keeps_page_index() {
        let loc = Location::from_px_box_on_page(3, 0.0, 0.0, 10.0, 10.0, 100.0, 100.0);
        assert_eq!(loc.page, 3);
    }

    #[test]
    fn prov_constructors() {
        assert_eq!(
            Prov::text(0, 3),
            Prov {
                section: 0,
                para: 3,
                ctrl: None
            }
        );
        assert_eq!(
            Prov::object(1, 2, 4),
            Prov {
                section: 1,
                para: 2,
                ctrl: Some(4)
            }
        );
    }
}
