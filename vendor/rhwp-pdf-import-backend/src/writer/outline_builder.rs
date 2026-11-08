//! Document outline (bookmarks) builder for PDF generation.
//!
//! This module provides support for creating navigable document outlines
//! per PDF spec Section 12.3.3 (Document Outline).
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{OutlineBuilder, OutlineItem};
//!
//! let mut outline = OutlineBuilder::new();
//! outline.add_item("Chapter 1", 0);
//! outline.add_child("Section 1.1", 0);
//! outline.add_child("Section 1.2", 0);
//! outline.pop(); // Back to root level
//! outline.add_item("Chapter 2", 1);
//! ```

use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Destination type for outline items.
#[derive(Debug, Clone)]
pub enum OutlineDestination {
    /// Go to a specific page (0-indexed)
    Page(usize),
    /// Go to a page with specific fit mode
    PageFit {
        /// Page index (0-indexed)
        page: usize,
        /// Fit mode
        fit: FitMode,
    },
    /// Go to a named destination
    Named(String),
    /// External URI link
    Uri(String),
}

/// Page fit mode for destinations.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum FitMode {
    /// Fit the entire page in the window (default)
    #[default]
    Fit,
    /// Fit the page width, with top at specified position
    FitH(Option<f32>),
    /// Fit the page height, with left at specified position
    FitV(Option<f32>),
    /// Fit a specific rectangle
    FitR {
        /// Left coordinate
        left: f32,
        /// Bottom coordinate
        bottom: f32,
        /// Right coordinate
        right: f32,
        /// Top coordinate
        top: f32,
    },
    /// Fit the bounding box of the page contents
    FitB,
    /// Fit bounding box width
    FitBH(Option<f32>),
    /// Fit bounding box height
    FitBV(Option<f32>),
    /// Display at specific position with zoom
    XYZ {
        /// Left coordinate (None = unchanged)
        left: Option<f32>,
        /// Top coordinate (None = unchanged)
        top: Option<f32>,
        /// Zoom factor (None = unchanged, 0 = fit)
        zoom: Option<f32>,
    },
}

/// Text style for outline items.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutlineStyle {
    /// Display in italic
    pub italic: bool,
    /// Display in bold
    pub bold: bool,
    /// Text color (RGB, 0.0-1.0)
    pub color: Option<(f32, f32, f32)>,
}

impl OutlineStyle {
    /// Create a new default style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bold style.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set italic style.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set text color.
    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Some((r, g, b));
        self
    }

    /// Get the flags value for this style (PDF spec Section 12.3.3).
    pub fn flags(&self) -> i64 {
        let mut flags = 0i64;
        if self.italic {
            flags |= 1;
        }
        if self.bold {
            flags |= 2;
        }
        flags
    }
}

/// A single outline item (bookmark).
#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// Display title
    pub title: String,
    /// Destination when clicked
    pub destination: OutlineDestination,
    /// Display style
    pub style: OutlineStyle,
    /// Whether the item is initially open (expanded)
    pub open: bool,
    /// Child items
    pub children: Vec<OutlineItem>,
}

impl OutlineItem {
    /// Create a new outline item pointing to a page.
    pub fn new(title: impl Into<String>, page: usize) -> Self {
        Self {
            title: title.into(),
            destination: OutlineDestination::Page(page),
            style: OutlineStyle::default(),
            open: true,
            children: Vec::new(),
        }
    }

    /// Create a new outline item with a custom destination.
    pub fn with_destination(title: impl Into<String>, destination: OutlineDestination) -> Self {
        Self {
            title: title.into(),
            destination,
            style: OutlineStyle::default(),
            open: true,
            children: Vec::new(),
        }
    }

    /// Set the display style.
    pub fn with_style(mut self, style: OutlineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether the item is initially open.
    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Add a child item.
    pub fn add_child(&mut self, child: OutlineItem) {
        self.children.push(child);
    }

    /// Get the total count of descendants (for PDF Count entry).
    /// Positive if open, negative if closed.
    fn descendant_count(&self) -> i64 {
        let mut count = self.children.len() as i64;
        for child in &self.children {
            count += child.visible_descendant_count();
        }
        if self.open {
            count
        } else {
            -count
        }
    }

    /// Count visible descendants (only count if parent is open).
    fn visible_descendant_count(&self) -> i64 {
        if !self.open {
            return 0;
        }
        let mut count = self.children.len() as i64;
        for child in &self.children {
            count += child.visible_descendant_count();
        }
        count
    }
}

/// Builder for document outlines (bookmarks).
///
/// Creates a hierarchical structure of outline items that can be
/// added to a PDF document for navigation.
#[derive(Debug, Default)]
pub struct OutlineBuilder {
    /// Root items
    items: Vec<OutlineItem>,
    /// Stack of indices for building hierarchy
    current_path: Vec<usize>,
}

impl OutlineBuilder {
    /// Create a new outline builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_path: Vec::new(),
        }
    }

    /// Add a top-level outline item.
    pub fn add_item(&mut self, item: OutlineItem) -> &mut Self {
        self.current_path.clear();
        let index = self.items.len();
        self.items.push(item);
        self.current_path.push(index);
        self
    }

    /// Add an item at the current level with a page destination.
    pub fn item(&mut self, title: impl Into<String>, page: usize) -> &mut Self {
        self.add_item(OutlineItem::new(title, page))
    }

    /// Add a child to the current item.
    pub fn add_child(&mut self, item: OutlineItem) -> &mut Self {
        if self.current_path.is_empty() {
            // No parent, add as root
            return self.add_item(item);
        }

        // Navigate to current parent
        let parent = self.get_current_mut();
        let child_index = parent.children.len();
        parent.children.push(item);

        // Update path to point to new child
        self.current_path.push(child_index);
        self
    }

    /// Add a child item with a page destination.
    pub fn child(&mut self, title: impl Into<String>, page: usize) -> &mut Self {
        self.add_child(OutlineItem::new(title, page))
    }

    /// Go back up one level in the hierarchy.
    pub fn pop(&mut self) -> &mut Self {
        if !self.current_path.is_empty() {
            self.current_path.pop();
        }
        self
    }

    /// Go back to the root level.
    pub fn root(&mut self) -> &mut Self {
        self.current_path.clear();
        self
    }

    /// Get the current item mutably.
    fn get_current_mut(&mut self) -> &mut OutlineItem {
        let mut current = &mut self.items[self.current_path[0]];
        for &idx in &self.current_path[1..] {
            current = &mut current.children[idx];
        }
        current
    }

    /// Check if the outline is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of top-level items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Get the root items.
    pub fn items(&self) -> &[OutlineItem] {
        &self.items
    }

    /// Build the outline objects for inclusion in a PDF.
    ///
    /// Returns:
    /// - The outline dictionary object
    /// - A map of object IDs to objects
    /// - The object IDs that need to be written
    pub fn build(&self, page_refs: &[ObjectRef], start_obj_id: u32) -> Option<OutlineBuildResult> {
        if self.items.is_empty() {
            return None;
        }

        let mut objects: HashMap<u32, Object> = HashMap::new();
        let mut next_id = start_obj_id;

        // Allocate root outline object
        let root_id = next_id;
        next_id += 1;

        // Build all items recursively, collecting object IDs
        let mut item_ids: Vec<u32> = Vec::new();
        for item in &self.items {
            let (item_id, new_next_id) =
                self.build_item(item, root_id, page_refs, next_id, &mut objects);
            item_ids.push(item_id);
            next_id = new_next_id;
        }

        // Set up sibling links (Prev/Next)
        for i in 0..item_ids.len() {
            let item_id = item_ids[i];
            if let Some(Object::Dictionary(dict)) = objects.get_mut(&item_id) {
                if i > 0 {
                    dict.insert(
                        "Prev".to_string(),
                        Object::Reference(ObjectRef::new(item_ids[i - 1], 0)),
                    );
                }
                if i < item_ids.len() - 1 {
                    dict.insert(
                        "Next".to_string(),
                        Object::Reference(ObjectRef::new(item_ids[i + 1], 0)),
                    );
                }
            }
        }

        // Calculate total count
        let total_count: i64 = self
            .items
            .iter()
            .map(|i| 1 + i.visible_descendant_count())
            .sum();

        // Build root outline dictionary
        let mut root_dict = HashMap::new();
        root_dict.insert("Type".to_string(), Object::Name("Outlines".to_string()));
        root_dict.insert("First".to_string(), Object::Reference(ObjectRef::new(item_ids[0], 0)));
        root_dict.insert(
            "Last".to_string(),
            Object::Reference(ObjectRef::new(*item_ids.last().unwrap(), 0)),
        );
        root_dict.insert("Count".to_string(), Object::Integer(total_count));

        objects.insert(root_id, Object::Dictionary(root_dict));

        Some(OutlineBuildResult {
            root_ref: ObjectRef::new(root_id, 0),
            objects,
            next_obj_id: next_id,
        })
    }

    /// Build a single outline item and its children.
    fn build_item(
        &self,
        item: &OutlineItem,
        parent_id: u32,
        page_refs: &[ObjectRef],
        start_id: u32,
        objects: &mut HashMap<u32, Object>,
    ) -> (u32, u32) {
        let item_id = start_id;
        let mut next_id = start_id + 1;

        let mut dict = HashMap::new();
        dict.insert("Title".to_string(), Object::text_string(&item.title));
        dict.insert("Parent".to_string(), Object::Reference(ObjectRef::new(parent_id, 0)));

        // Add destination
        match &item.destination {
            OutlineDestination::Page(page_idx) => {
                if let Some(page_ref) = page_refs.get(*page_idx) {
                    let dest = Object::Array(vec![
                        Object::Reference(*page_ref),
                        Object::Name("Fit".to_string()),
                    ]);
                    dict.insert("Dest".to_string(), dest);
                }
            },
            OutlineDestination::PageFit { page, fit } => {
                if let Some(page_ref) = page_refs.get(*page) {
                    let dest = self.build_destination(*page_ref, fit);
                    dict.insert("Dest".to_string(), dest);
                }
            },
            OutlineDestination::Named(name) => {
                dict.insert("Dest".to_string(), Object::text_string(name));
            },
            OutlineDestination::Uri(uri) => {
                let mut action = HashMap::new();
                action.insert("S".to_string(), Object::Name("URI".to_string()));
                action.insert("URI".to_string(), Object::text_string(uri));
                dict.insert("A".to_string(), Object::Dictionary(action));
            },
        }

        // Add style if non-default
        let flags = item.style.flags();
        if flags != 0 {
            dict.insert("F".to_string(), Object::Integer(flags));
        }
        if let Some((r, g, b)) = item.style.color {
            dict.insert(
                "C".to_string(),
                Object::Array(vec![
                    Object::Real(r as f64),
                    Object::Real(g as f64),
                    Object::Real(b as f64),
                ]),
            );
        }

        // Build children
        let mut child_ids: Vec<u32> = Vec::new();
        for child in &item.children {
            let (child_id, new_next_id) =
                self.build_item(child, item_id, page_refs, next_id, objects);
            child_ids.push(child_id);
            next_id = new_next_id;
        }

        // Add child links
        if !child_ids.is_empty() {
            dict.insert("First".to_string(), Object::Reference(ObjectRef::new(child_ids[0], 0)));
            dict.insert(
                "Last".to_string(),
                Object::Reference(ObjectRef::new(*child_ids.last().unwrap(), 0)),
            );

            // Add count
            let count = item.descendant_count();
            if count != 0 {
                dict.insert("Count".to_string(), Object::Integer(count));
            }

            // Set up sibling links for children
            for i in 0..child_ids.len() {
                let cid = child_ids[i];
                if let Some(Object::Dictionary(cdict)) = objects.get_mut(&cid) {
                    if i > 0 {
                        cdict.insert(
                            "Prev".to_string(),
                            Object::Reference(ObjectRef::new(child_ids[i - 1], 0)),
                        );
                    }
                    if i < child_ids.len() - 1 {
                        cdict.insert(
                            "Next".to_string(),
                            Object::Reference(ObjectRef::new(child_ids[i + 1], 0)),
                        );
                    }
                }
            }
        }

        objects.insert(item_id, Object::Dictionary(dict));
        (item_id, next_id)
    }

    /// Build a destination array for a fit mode.
    fn build_destination(&self, page_ref: ObjectRef, fit: &FitMode) -> Object {
        let mut arr = vec![Object::Reference(page_ref)];

        match fit {
            FitMode::Fit => {
                arr.push(Object::Name("Fit".to_string()));
            },
            FitMode::FitH(top) => {
                arr.push(Object::Name("FitH".to_string()));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitV(left) => {
                arr.push(Object::Name("FitV".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitR {
                left,
                bottom,
                right,
                top,
            } => {
                arr.push(Object::Name("FitR".to_string()));
                arr.push(Object::Real(*left as f64));
                arr.push(Object::Real(*bottom as f64));
                arr.push(Object::Real(*right as f64));
                arr.push(Object::Real(*top as f64));
            },
            FitMode::FitB => {
                arr.push(Object::Name("FitB".to_string()));
            },
            FitMode::FitBH(top) => {
                arr.push(Object::Name("FitBH".to_string()));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
            },
            FitMode::FitBV(left) => {
                arr.push(Object::Name("FitBV".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
            },
            FitMode::XYZ { left, top, zoom } => {
                arr.push(Object::Name("XYZ".to_string()));
                arr.push(left.map(|l| Object::Real(l as f64)).unwrap_or(Object::Null));
                arr.push(top.map(|t| Object::Real(t as f64)).unwrap_or(Object::Null));
                arr.push(zoom.map(|z| Object::Real(z as f64)).unwrap_or(Object::Null));
            },
        }

        Object::Array(arr)
    }
}

/// Result of building an outline.
#[derive(Debug)]
pub struct OutlineBuildResult {
    /// Reference to the root outline object
    pub root_ref: ObjectRef,
    /// All outline objects
    pub objects: HashMap<u32, Object>,
    /// Next available object ID
    pub next_obj_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outline_item_creation() {
        let item = OutlineItem::new("Chapter 1", 0);
        assert_eq!(item.title, "Chapter 1");
        assert!(matches!(item.destination, OutlineDestination::Page(0)));
        assert!(item.open);
    }

    #[test]
    fn test_outline_style_flags() {
        let style = OutlineStyle::new();
        assert_eq!(style.flags(), 0);

        let bold = OutlineStyle::new().bold();
        assert_eq!(bold.flags(), 2);

        let italic = OutlineStyle::new().italic();
        assert_eq!(italic.flags(), 1);

        let bold_italic = OutlineStyle::new().bold().italic();
        assert_eq!(bold_italic.flags(), 3);
    }

    #[test]
    fn test_outline_builder_empty() {
        let builder = OutlineBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_outline_builder_simple() {
        let mut builder = OutlineBuilder::new();
        builder.item("Chapter 1", 0);
        builder.item("Chapter 2", 5);

        assert!(!builder.is_empty());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn test_outline_builder_hierarchy() {
        let mut builder = OutlineBuilder::new();
        builder.item("Chapter 1", 0);
        builder.child("Section 1.1", 1);
        builder.child("Subsection 1.1.1", 2);
        builder.pop(); // Back to Section 1.1
        builder.pop(); // Back to Chapter 1
        builder.child("Section 1.2", 3);
        builder.root();
        builder.item("Chapter 2", 4);

        let items = builder.items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].children.len(), 2); // Section 1.1 and 1.2
        assert_eq!(items[0].children[0].children.len(), 1); // Subsection 1.1.1
    }

    #[test]
    fn test_outline_build() {
        let mut builder = OutlineBuilder::new();
        builder.item("Page 1", 0);
        builder.item("Page 2", 1);

        let page_refs = vec![ObjectRef::new(10, 0), ObjectRef::new(11, 0)];

        let result = builder.build(&page_refs, 100);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.root_ref.id, 100);
        assert!(result.objects.contains_key(&100)); // Root
        assert!(result.objects.contains_key(&101)); // Item 1
        assert!(result.objects.contains_key(&102)); // Item 2
    }

    #[test]
    fn test_outline_with_uri() {
        let item = OutlineItem::with_destination(
            "External Link",
            OutlineDestination::Uri("https://example.com".to_string()),
        );

        assert!(matches!(item.destination, OutlineDestination::Uri(_)));
    }

    #[test]
    fn test_descendant_count() {
        let mut item = OutlineItem::new("Root", 0);
        let mut child1 = OutlineItem::new("Child 1", 1);
        child1.add_child(OutlineItem::new("Grandchild 1", 2));
        child1.add_child(OutlineItem::new("Grandchild 2", 3));
        item.add_child(child1);
        item.add_child(OutlineItem::new("Child 2", 4));

        // Root has 2 children, child1 has 2 grandchildren = 4 total descendants
        assert_eq!(item.descendant_count(), 4);
    }

    #[test]
    fn test_closed_outline_negative_count() {
        let mut item = OutlineItem::new("Root", 0);
        item.open = false;
        item.add_child(OutlineItem::new("Child 1", 1));
        item.add_child(OutlineItem::new("Child 2", 2));

        // Closed item should have negative count
        assert_eq!(item.descendant_count(), -2);
    }
}
