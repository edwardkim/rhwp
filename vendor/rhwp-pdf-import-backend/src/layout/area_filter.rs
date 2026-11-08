//! Spatial filtering for document elements.
//!
//! This module provides utilities for filtering text and geometric elements
//! based on their rectangular regions.

use crate::elements::PathContent;
use crate::geometry::Rect;
use crate::layout::{TextChar, TextLine, TextSpan, Word};

/// Filter mode for bounded extraction.
///
/// Determines how elements are selected based on their bounding box
/// relationship with a target region.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum RectFilterMode {
    /// Include elements that have any overlap with the target rectangle.
    #[default]
    #[serde(rename = "intersects")]
    Intersects,
    /// Include only elements that are fully contained within the target rectangle.
    #[serde(rename = "contained")]
    FullyContained,
    /// Include elements where at least the specified fraction (0.0-1.0) overlaps
    /// with the target rectangle.
    #[serde(rename = "overlap")]
    MinOverlap(f32),
}

/// Common trait for spatial filtering of layout objects.
pub trait LayoutObjectSpatial {
    /// Get the bounding box of the object.
    fn bbox(&self) -> Rect;

    /// Check if this object intersects with the given rectangle.
    fn intersects_rect(&self, rect: &Rect) -> bool {
        self.bbox().intersects(rect)
    }

    /// Check if this object is fully contained within the given rectangle.
    fn contained_in_rect(&self, rect: &Rect) -> bool {
        rect.contains_rect(&self.bbox())
    }

    /// Calculate the overlap fraction (0.0-1.0) with the given rectangle.
    ///
    /// Returns the ratio of the intersection area to **this object's** area —
    /// i.e. "what fraction of me lies inside the rect?". This is asymmetric:
    /// a small element mostly inside a large rect scores high; a large element
    /// with only a corner inside scores low. Used by `MinOverlap(t)` filtering.
    fn overlap_with_rect(&self, rect: &Rect) -> f32 {
        let bbox = self.bbox();
        let intersection = bbox.intersection(rect);
        match intersection {
            Some(inter) => {
                let area = bbox.width * bbox.height;
                if area > 0.0 {
                    (inter.width * inter.height) / area
                } else {
                    // Zero-area objects (like lines) use a binary 1.0 or 0.0
                    1.0
                }
            },
            None => 0.0,
        }
    }

    /// Check if this object matches the given filter mode for a rectangle.
    fn matches_filter(&self, rect: &Rect, mode: RectFilterMode) -> bool {
        match mode {
            RectFilterMode::Intersects => self.intersects_rect(rect),
            RectFilterMode::FullyContained => self.contained_in_rect(rect),
            RectFilterMode::MinOverlap(threshold) => self.overlap_with_rect(rect) >= threshold,
        }
    }
}

// Implement for core types
impl LayoutObjectSpatial for TextChar {
    fn bbox(&self) -> Rect {
        self.bbox
    }
}

impl LayoutObjectSpatial for TextSpan {
    fn bbox(&self) -> Rect {
        self.bbox
    }
}

impl LayoutObjectSpatial for Word {
    fn bbox(&self) -> Rect {
        self.bbox
    }
}

impl LayoutObjectSpatial for TextLine {
    fn bbox(&self) -> Rect {
        self.bbox
    }
}

impl LayoutObjectSpatial for PathContent {
    fn bbox(&self) -> Rect {
        self.bbox
    }
}

/// Extension trait for filtering collections of layout objects.
pub trait SpatialCollectionFiltering<T: LayoutObjectSpatial> {
    /// Filter objects by their spatial relationship with a rectangle.
    fn filter_by_rect(&self, rect: &Rect, mode: RectFilterMode) -> Vec<T>;

    /// Exclude objects that match any of the given rectangles under `mode`.
    ///
    /// The inverse of [`Self::filter_by_rect`]: keeps every element that does NOT
    /// match ANY of the excluded regions. Useful for stripping figure-internal
    /// or header/footer text from a page extraction result.
    fn exclude_rects(&self, rects: &[Rect], mode: RectFilterMode) -> Vec<T>;
}

impl<T: LayoutObjectSpatial + Clone> SpatialCollectionFiltering<T> for [T] {
    fn filter_by_rect(&self, rect: &Rect, mode: RectFilterMode) -> Vec<T> {
        self.iter()
            .filter(|obj| obj.matches_filter(rect, mode))
            .cloned()
            .collect()
    }

    fn exclude_rects(&self, rects: &[Rect], mode: RectFilterMode) -> Vec<T> {
        self.iter()
            .filter(|obj| !rects.iter().any(|r| obj.matches_filter(r, mode)))
            .cloned()
            .collect()
    }
}

impl<T: LayoutObjectSpatial + Clone> SpatialCollectionFiltering<T> for Vec<T> {
    fn filter_by_rect(&self, rect: &Rect, mode: RectFilterMode) -> Vec<T> {
        self.as_slice().filter_by_rect(rect, mode)
    }

    fn exclude_rects(&self, rects: &[Rect], mode: RectFilterMode) -> Vec<T> {
        self.as_slice().exclude_rects(rects, mode)
    }
}
