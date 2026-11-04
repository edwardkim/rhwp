//! Path extraction from PDF content streams.
//!
//! This module provides the `PathExtractor` type for extracting vector graphics
//! (paths) from PDF content streams.
//!
//! # PDF Path Operations
//!
//! PDF paths are constructed using a sequence of operators:
//! - `m` (MoveTo): Begin a new subpath
//! - `l` (LineTo): Add a line segment
//! - `c`, `v`, `y` (CurveTo variants): Add Bezier curve segments
//! - `re` (Rectangle): Add a rectangle as a complete subpath
//! - `h` (ClosePath): Close the current subpath
//!
//! Paths are then painted using:
//! - `S` (Stroke): Stroke the path
//! - `f`, `F`, `f*` (Fill): Fill the path
//! - `B`, `B*`, `b`, `b*`: Fill and stroke
//! - `n` (EndPath): End path without painting (used with clipping)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::extractors::paths::PathExtractor;
//!
//! let mut extractor = PathExtractor::new();
//!
//! // Process path construction operators
//! extractor.move_to(100.0, 100.0);
//! extractor.line_to(200.0, 100.0);
//! extractor.line_to(200.0, 200.0);
//! extractor.close_path();
//!
//! // Finalize with a painting operator
//! extractor.stroke();
//!
//! // Get extracted paths
//! let paths = extractor.finish();
//! ```

use crate::content::graphics_state::{GraphicsState, Matrix};
use crate::elements::{LineCap, LineJoin, PathContent, PathOperation};
use crate::geometry::{Point, Rect};
use crate::layout::Color;

/// Lightweight graphics state: scalar styles and a shared immutable clip chain.
/// q/Q does not deep-copy path geometry or allocate a full GraphicsState.
#[derive(Debug, Clone)]
pub(crate) struct PathGraphicsState {
    pub ctm: Matrix,
    pub stroke_color_space: crate::path_color::PathColorSpace,
    pub fill_color_space: crate::path_color::PathColorSpace,
    pub stroke_color_rgb: (f32, f32, f32),
    pub fill_color_rgb: (f32, f32, f32),
    pub line_width: f32,
    pub text_render_mode: u8,
    pub miter_limit: f32,
    pub line_cap: u8,
    pub line_join: u8,
    pub fill_alpha: f32,
    pub stroke_alpha: f32,
    pub clip_path: Option<std::sync::Arc<crate::elements::PathClip>>,
    pub dash_pattern: Option<(Vec<f32>, f32)>,
}

impl PathGraphicsState {
    pub fn new() -> Self {
        Self {
            ctm: Matrix::identity(),
            stroke_color_space: crate::path_color::PathColorSpace::GRAY,
            fill_color_space: crate::path_color::PathColorSpace::GRAY,
            stroke_color_rgb: (0.0, 0.0, 0.0),
            fill_color_rgb: (0.0, 0.0, 0.0),
            line_width: 1.0,
            text_render_mode: 0,
            miter_limit: 10.0,
            line_cap: 0,
            line_join: 0,
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            clip_path: None,
            dash_pattern: None,
        }
    }
}

/// Graphics state stack using the lightweight [`PathGraphicsState`].
pub(crate) struct PathGraphicsStateStack {
    stack: Vec<PathGraphicsState>,
}

impl PathGraphicsStateStack {
    pub fn new() -> Self {
        Self {
            stack: vec![PathGraphicsState::new()],
        }
    }

    pub fn current(&self) -> &PathGraphicsState {
        self.stack.last().expect("Stack should never be empty")
    }

    pub fn current_mut(&mut self) -> &mut PathGraphicsState {
        self.stack.last_mut().expect("Stack should never be empty")
    }

    pub fn save(&mut self) {
        let state = self.current().clone();
        self.stack.push(state);
    }

    pub fn restore(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }
}

/// Fill rule for path filling operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding number rule (f, F operators)
    NonZero,
    /// Even-odd rule (f* operator)
    EvenOdd,
}

/// Path extractor for accumulating path operations from content streams.
///
/// This struct maintains state during content stream processing and accumulates
/// path operations. When a painting operator is encountered, the current path
/// is finalized and added to the extracted paths list.
///
/// # XObject Support (Issue #40)
///
/// The extractor can recursively process Form XObjects by maintaining a reference
/// to page resources and the document. When encountering a `Do` operator with a
/// Form XObject name, it will:
/// 1. Resolve the XObject from resources
/// 2. Check if it's a Form (not Image)
/// 3. Apply coordinate transformations
/// 4. Recursively extract paths from the XObject stream
///
/// XObjects are tracked to prevent infinite loops from circular references.
#[derive(Debug)]
pub struct PathExtractor {
    /// Accumulated complete paths
    paths: Vec<PathContent>,
    clip_observations: Option<crate::elements::PathClipObservations>,
    pub(crate) paint: crate::elements::PathPaint,
    /// Current path being constructed
    current_operations: Vec<PathOperation>,
    /// Current point (set by MoveTo, LineTo, CurveTo, etc.)
    current_point: Option<Point>,
    pending_clip: Option<bool>,
    /// Start point of current subpath (for ClosePath)
    subpath_start: Option<Point>,
    /// Current graphics state snapshot (for colors, line style)
    current_stroke_color: Option<Color>,
    current_fill_color: Option<Color>,
    current_line_width: f32,
    current_dash: Option<(Vec<f32>, f32)>,
    current_line_cap: LineCap,
    current_line_join: LineJoin,
    /// Current transformation matrix for coordinate transformation
    ctm: Matrix,
    /// Page resources for XObject resolution (Issue #40)
    resources: Option<crate::object::Object>,
    /// Stack of XObjects being processed to detect cycles (Issue #40)
    xobject_processing_stack: Vec<crate::object::ObjectRef>,
    /// Set of XObjects already fully processed. Prevents combinatorial
    /// explosion when multiple parent XObjects reference the same children
    /// (e.g., chart/plot pages where 9 top-level Form XObjects each contain
    /// 25-42 nested `Do` operators pointing to the same shared XObjects).
    processed_xobjects: std::collections::HashSet<(crate::object::ObjectRef, [i32; 6])>,
    /// Maximum XObject nesting depth (prevent stack overflow)
    max_xobject_depth: usize,
    /// Cached XObject name → ObjectRef mapping, built on first lookup.
    cached_xobject_dict: Option<std::collections::HashMap<String, crate::object::ObjectRef>>,
    /// Stack of active Optional Content Group (PDF "layer") names. Each
    /// `BDC`/`BMC` operator (any tag) pushes one entry and each `EMC` pops
    /// one, so the depth tracks the marked-content nesting precisely
    /// (ISO 32000-1:2008 §8.11, §14.6). Entries are stored *pre-resolved*
    /// to the effective layer at that level: an `/OC` tag whose property
    /// dict resolves to a named OCG stores that name, while non-`/OC`
    /// marked content (and `/OC` regions whose name did not resolve)
    /// inherits the layer already in effect. The top entry is therefore
    /// always the active layer, making `current_layer` O(1).
    oc_layer_stack: Vec<Option<String>>,
}

impl PathExtractor {
    /// Create a new path extractor.
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            clip_observations: None,
            paint: Default::default(),
            current_operations: Vec::new(),
            current_point: None,
            pending_clip: None,
            subpath_start: None,
            current_stroke_color: Some(Color::black()),
            current_fill_color: None,
            current_line_width: 1.0,
            current_dash: None,
            current_line_cap: LineCap::Butt,
            current_line_join: LineJoin::Miter,
            ctm: Matrix::identity(),
            resources: None,
            xobject_processing_stack: Vec::new(),
            processed_xobjects: std::collections::HashSet::new(),
            max_xobject_depth: 100,
            cached_xobject_dict: None,
            oc_layer_stack: Vec::new(),
        }
    }

    /// Push an entry onto the marked-content layer stack.
    ///
    /// Pass `Some(name)` for `BDC /OC <<...>>` (or `BDC /OC /MC0`) where
    /// the property dict resolves to a named Optional Content Group. Pass
    /// `None` for any other `BDC`/`BMC` (e.g. `BDC /Span <<...>>`,
    /// `BDC /Artifact`) so the stack stays balanced and outer OCG context
    /// continues to apply to nested non-OC content.
    pub fn push_oc_layer(&mut self, layer: Option<String>) {
        // Store the *effective* layer for this nesting level: an `/OC` tag
        // with a resolved OCG name takes effect; every other entry (non-OC
        // `BDC`/`BMC`, or an `/OC` whose name did not resolve) inherits the
        // layer already in effect. Pre-resolving here keeps `current_layer`
        // O(1) instead of an O(depth) scan on every finalized path.
        let effective = layer.or_else(|| self.oc_layer_stack.last().cloned().flatten());
        self.oc_layer_stack.push(effective);
    }

    /// Pop the most recently pushed marked-content entry (called on `EMC`).
    /// No-op if the stack is empty — keeps extraction robust against PDFs
    /// with unbalanced markers.
    pub fn pop_oc_layer(&mut self) {
        self.oc_layer_stack.pop();
    }

    /// Current marked-content nesting depth. Paired with [`truncate_oc_layers`]
    /// to isolate a Form XObject's marked-content scope from its caller.
    pub(crate) fn oc_layer_depth(&self) -> usize {
        self.oc_layer_stack.len()
    }

    /// Drop any marked-content entries pushed above `depth`. Called when a
    /// Form XObject content stream returns so an unbalanced `BDC` left open
    /// inside the XObject cannot leak its layer onto the caller's subsequent
    /// paths. The spec requires per-stream balance (§14.6), but real-world
    /// PDFs occasionally violate it.
    pub(crate) fn truncate_oc_layers(&mut self, depth: usize) {
        self.oc_layer_stack.truncate(depth);
    }

    /// Return the active Optional Content Group ("layer") name, or `None`
    /// if no `/OC` region is currently in effect. O(1): each stack entry is
    /// pre-resolved to its effective layer by [`push_oc_layer`].
    fn current_layer(&self) -> Option<String> {
        self.oc_layer_stack.last().cloned().flatten()
    }

    /// Set the page resources for XObject resolution (Issue #40).
    pub fn set_resources(&mut self, resources: crate::object::Object) {
        self.resources = Some(resources);
    }

    /// The resource dictionary currently in scope — the page resources, or
    /// the active Form XObject's own `/Resources` after a [`swap_resources`].
    /// Used to resolve `BDC /OC /Name` property references against the
    /// current `/Properties` subdictionary (ISO 32000-1:2008 §14.6.2).
    pub(crate) fn current_resources(&self) -> Option<&crate::object::Object> {
        self.resources.as_ref()
    }

    /// Swap in a new resource scope for `Do` name lookups and return the
    /// previous (resources, cached_dict) pair so the caller can restore it
    /// after descending into a nested Form XObject. Clears the name→ref
    /// cache so the next `resolve_xobject_ref` call rebuilds it against the
    /// new scope.
    ///
    /// Needed because Form XObjects that carry their own /Resources define a
    /// fresh XObject name scope — without swapping, nested `/Name Do`
    /// operators resolve against the parent scope and can trigger pathological
    /// cross-recursion between sibling forms whose local resource names happen
    /// to collide with parent form names.
    pub(crate) fn swap_resources(
        &mut self,
        new_resources: Option<crate::object::Object>,
    ) -> (
        Option<crate::object::Object>,
        Option<std::collections::HashMap<String, crate::object::ObjectRef>>,
    ) {
        let prev_resources = std::mem::replace(&mut self.resources, new_resources);
        let prev_cache = self.cached_xobject_dict.take();
        (prev_resources, prev_cache)
    }

    /// Restore a (resources, cached_dict) pair previously returned by
    /// [`swap_resources`].
    pub(crate) fn restore_resources(
        &mut self,
        saved: (
            Option<crate::object::Object>,
            Option<std::collections::HashMap<String, crate::object::ObjectRef>>,
        ),
    ) {
        self.resources = saved.0;
        self.cached_xobject_dict = saved.1;
    }

    /// Resolve an XObject name to its ObjectRef, caching the XObject dict on first call.
    pub(crate) fn resolve_xobject_ref<F>(
        &mut self,
        name: &str,
        mut load_object: F,
    ) -> Option<crate::object::ObjectRef>
    where
        F: FnMut(crate::object::ObjectRef) -> crate::error::Result<crate::object::Object>,
    {
        // Build cache on first call
        if self.cached_xobject_dict.is_none() {
            let mut map = std::collections::HashMap::new();

            let resources = self.resources.as_ref()?;
            let resolved_resources = if let Some(ref_obj) = resources.as_reference() {
                load_object(ref_obj).ok()?
            } else {
                resources.clone()
            };
            let resources_dict = resolved_resources.as_dict()?;
            let xobject_obj = resources_dict.get("XObject")?;
            let resolved_xobject_obj = if let Some(ref_obj) = xobject_obj.as_reference() {
                load_object(ref_obj).ok()?
            } else {
                xobject_obj.clone()
            };
            if let Some(xobject_dict) = resolved_xobject_obj.as_dict() {
                for (key, val) in xobject_dict.iter() {
                    if let Some(obj_ref) = val.as_reference() {
                        map.insert(key.clone(), obj_ref);
                    }
                }
            }
            self.cached_xobject_dict = Some(map);
        }

        self.cached_xobject_dict.as_ref()?.get(name).copied()
    }

    /// Compute a rounded fingerprint of the current CTM for dedup purposes.
    /// Translation components (e, f) are rounded to 0.1 while scale/rotation
    /// components (a-d) are rounded to 0.01, balancing dedup accuracy with
    /// floating-point tolerance. Uses banker's-style `f32::round` so negative
    /// values round symmetrically rather than truncating toward zero.
    fn ctm_fingerprint(ctm: &Matrix) -> [i32; 6] {
        [
            (ctm.a * 100.0).round() as i32,
            (ctm.b * 100.0).round() as i32,
            (ctm.c * 100.0).round() as i32,
            (ctm.d * 100.0).round() as i32,
            (ctm.e * 10.0).round() as i32,
            (ctm.f * 10.0).round() as i32,
        ]
    }

    pub(crate) fn can_process_xobject(&self, xobject_ref: crate::object::ObjectRef) -> bool {
        let key = (xobject_ref, Self::ctm_fingerprint(&self.ctm));
        if self.processed_xobjects.contains(&key) {
            return false;
        }
        if self.xobject_processing_stack.contains(&xobject_ref) {
            return false;
        }
        // Check if we've exceeded maximum nesting depth
        if self.xobject_processing_stack.len() >= self.max_xobject_depth {
            return false;
        }
        true
    }

    /// Push an XObject onto the processing stack (called before processing).
    pub(crate) fn push_xobject(&mut self, xobject_ref: crate::object::ObjectRef) {
        self.xobject_processing_stack.push(xobject_ref);
    }

    /// Pop an XObject from the processing stack after successful processing.
    /// Marks it as permanently processed to prevent re-processing from
    /// other parent XObjects at the same CTM.
    pub(crate) fn pop_xobject(&mut self) {
        if let Some(ref_obj) = self.xobject_processing_stack.pop() {
            let key = (ref_obj, Self::ctm_fingerprint(&self.ctm));
            self.processed_xobjects.insert(key);
        }
    }

    /// Pop an XObject from the processing stack after a failure.
    /// Does NOT mark it as permanently processed, allowing retry.
    pub(crate) fn pop_xobject_failed(&mut self) {
        self.xobject_processing_stack.pop();
    }

    /// Update the current transformation matrix.
    pub fn set_ctm(&mut self, ctm: Matrix) {
        self.ctm = ctm;
    }

    /// Update graphics state from a GraphicsState snapshot.
    pub fn update_from_state(&mut self, state: &GraphicsState) {
        self.ctm = state.ctm;
        self.current_line_width = state.line_width;
        self.current_dash = (!state.dash_pattern.0.is_empty()).then(|| state.dash_pattern.clone());

        // Convert line cap (0=butt, 1=round, 2=square)
        self.current_line_cap = match state.line_cap {
            1 => LineCap::Round,
            2 => LineCap::Square,
            _ => LineCap::Butt,
        };

        // Convert line join (0=miter, 1=round, 2=bevel)
        self.current_line_join = match state.line_join {
            1 => LineJoin::Round,
            2 => LineJoin::Bevel,
            _ => LineJoin::Miter,
        };

        // Convert stroke color from RGB
        let (r, g, b) = state.stroke_color_rgb;
        self.current_stroke_color = Some(Color::new(r, g, b));

        // Convert fill color from RGB
        let (r, g, b) = state.fill_color_rgb;
        self.current_fill_color = Some(Color::new(r, g, b));
    }

    /// Update extractor state from a lightweight path graphics state.
    pub(crate) fn update_from_path_state(&mut self, state: &PathGraphicsState) {
        self.ctm = state.ctm;
        self.paint.fill_alpha = state.fill_alpha;
        self.paint.stroke_alpha = state.stroke_alpha;
        self.paint.clip_path = state.clip_path.clone();
        self.current_line_width = state.line_width;
        self.current_dash = state.dash_pattern.clone();
        self.current_line_cap = match state.line_cap {
            1 => LineCap::Round,
            2 => LineCap::Square,
            _ => LineCap::Butt,
        };
        self.current_line_join = match state.line_join {
            1 => LineJoin::Round,
            2 => LineJoin::Bevel,
            _ => LineJoin::Miter,
        };
        let (r, g, b) = state.stroke_color_rgb;
        self.current_stroke_color =
            (!matches!(state.stroke_color_space, crate::path_color::PathColorSpace::Pattern(_)))
                .then_some(Color::new(r, g, b));
        let (r, g, b) = state.fill_color_rgb;
        self.current_fill_color =
            (!matches!(state.fill_color_space, crate::path_color::PathColorSpace::Pattern(_)))
                .then_some(Color::new(r, g, b));
    }

    /// Set stroke color.
    pub fn set_stroke_color(&mut self, color: Color) {
        self.current_stroke_color = Some(color);
    }

    /// Set fill color.
    pub fn set_fill_color(&mut self, color: Color) {
        self.current_fill_color = Some(color);
    }

    /// Set line width.
    pub fn set_line_width(&mut self, width: f32) {
        self.current_line_width = width;
    }

    /// Set line cap style.
    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.current_line_cap = cap;
    }

    /// Set line join style.
    pub fn set_line_join(&mut self, join: LineJoin) {
        self.current_line_join = join;
    }

    // === Path Construction Operators ===

    /// Move to a point (m operator).
    ///
    /// Begins a new subpath at the specified point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        let point = self.transform_point(x, y);
        // Consecutive m operators replace the pending, segment-free subpath
        // start (PDF 32000-1, table 59). Preserve actual preceding subpaths.
        if let Some(PathOperation::MoveTo(a, b)) = self.current_operations.last_mut() {
            (*a, *b) = (point.x, point.y);
        } else {
            self.current_operations
                .push(PathOperation::MoveTo(point.x, point.y));
        }
        self.current_point = Some(point);
        self.subpath_start = Some(point);
    }

    /// Line to a point (l operator).
    ///
    /// Adds a line segment from the current point to the specified point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        let point = self.transform_point(x, y);
        self.current_operations
            .push(PathOperation::LineTo(point.x, point.y));
        self.current_point = Some(point);
    }

    /// Cubic Bezier curve (c operator).
    ///
    /// Adds a cubic Bezier curve from the current point to (x3, y3),
    /// using (x1, y1) and (x2, y2) as control points.
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = self.transform_point(x1, y1);
        let p2 = self.transform_point(x2, y2);
        let p3 = self.transform_point(x3, y3);
        self.current_operations
            .push(PathOperation::CurveTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y));
        self.current_point = Some(p3);
    }

    /// Cubic Bezier curve with first control point = current point (v operator).
    ///
    /// Adds a cubic Bezier curve from the current point to (x3, y3),
    /// using the current point as the first control point and (x2, y2) as the second.
    pub fn curve_to_v(&mut self, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = self.current_point.unwrap_or(Point::new(0.0, 0.0));
        let p2 = self.transform_point(x2, y2);
        let p3 = self.transform_point(x3, y3);
        self.current_operations
            .push(PathOperation::CurveTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y));
        self.current_point = Some(p3);
    }

    /// Cubic Bezier curve with second control point = end point (y operator).
    ///
    /// Adds a cubic Bezier curve from the current point to (x3, y3),
    /// using (x1, y1) as the first control point and (x3, y3) as the second.
    pub fn curve_to_y(&mut self, x1: f32, y1: f32, x3: f32, y3: f32) {
        let p1 = self.transform_point(x1, y1);
        let p3 = self.transform_point(x3, y3);
        // Second control point equals end point
        self.current_operations
            .push(PathOperation::CurveTo(p1.x, p1.y, p3.x, p3.y, p3.x, p3.y));
        self.current_point = Some(p3);
    }

    /// Rectangle (re operator).
    ///
    /// Adds a complete rectangle subpath.
    pub fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // re begins with an implicit m, which also replaces a pending m.
        if matches!(self.current_operations.last(), Some(PathOperation::MoveTo(..))) {
            self.current_operations.pop();
        }
        // Rectangle needs to transform all corners properly
        let p1 = self.transform_point(x, y);
        let p2 = self.transform_point(x + width, y + height);

        // Calculate transformed width and height
        let transformed_width = p2.x - p1.x;
        let transformed_height = p2.y - p1.y;

        self.current_operations.push(PathOperation::Rectangle(
            p1.x,
            p1.y,
            transformed_width,
            transformed_height,
        ));

        // Rectangle implicitly closes the subpath
        self.current_point = Some(p1);
        self.subpath_start = Some(p1);
    }

    /// Close the current subpath (h operator).
    ///
    /// Adds a line segment from the current point to the start of the subpath.
    pub fn close_path(&mut self) {
        self.current_operations.push(PathOperation::ClosePath);
        // Reset current point to subpath start
        if let Some(start) = self.subpath_start {
            self.current_point = Some(start);
        }
    }

    // === Path Painting Operators ===

    /// Stroke the path (S operator).
    pub fn stroke(&mut self) {
        self.finalize_path(true, false, FillRule::NonZero);
    }

    /// Close and stroke the path (s operator).
    pub fn close_and_stroke(&mut self) {
        self.close_path();
        self.stroke();
    }

    /// Fill the path (f or F operator).
    pub fn fill(&mut self, rule: FillRule) {
        self.finalize_path(false, true, rule);
    }

    /// Fill and stroke the path (B operator).
    pub fn fill_and_stroke(&mut self, rule: FillRule) {
        self.finalize_path(true, true, rule);
    }

    /// Close, fill and stroke the path (b operator).
    pub fn close_fill_and_stroke(&mut self, rule: FillRule) {
        self.close_path();
        self.fill_and_stroke(rule);
    }

    /// End path without painting (n operator).
    ///
    /// Used primarily with clipping paths.
    pub fn end_path(&mut self) {
        if let Some(even_odd) = self.pending_clip.take() {
            self.paint.clip_path = Some(std::sync::Arc::new(crate::elements::PathClip {
                operations: self.current_operations.clone(),
                even_odd,
                parent: self.paint.clip_path.clone(),
            }));
            self.record_clip();
        }
        // Clear current path without creating a PathContent
        self.current_operations.clear();
        self.current_point = None;
        self.subpath_start = None;
    }

    // === Clipping Operators ===

    /// Set clipping path using non-zero winding rule (W operator).
    pub fn clip_non_zero(&mut self) {
        self.pending_clip = Some(false);
    }

    /// Set clipping path using even-odd rule (W* operator).
    pub fn clip_even_odd(&mut self) {
        self.pending_clip = Some(true);
    }

    // === Helper Methods ===

    /// Transform a point using the current CTM.
    fn transform_point(&self, x: f32, y: f32) -> Point {
        self.ctm.transform_point(x, y)
    }

    /// Finalize the current path and add it to the extracted paths.
    fn finalize_path(&mut self, stroke: bool, fill: bool, rule: FillRule) {
        if self.current_operations.is_empty() {
            return;
        }

        // Compute bounding box
        let bbox = Self::compute_bbox(&self.current_operations);

        // Create PathContent
        let mut path = PathContent::new(bbox);
        path.operations = std::mem::take(&mut self.current_operations);
        path.paint = self.paint.clone();
        path.paint.even_odd = rule == FillRule::EvenOdd;

        // Set stroke properties
        if stroke && self.paint.stroke_alpha > 0.0 {
            path.stroke_color = self.current_stroke_color;
            // The `w` operand is in user space while the path coordinates
            // above were CTM-transformed (§8.4.3.2: the line width is
            // transformed like all other geometry). Store the effective
            // rendered width — `sqrt(|det|)` is the standard uniform-scale
            // approximation renderers use — so `stroke_width` and `bbox`
            // share one coordinate space.
            path.stroke_width = self.current_line_width * self.ctm.stroke_scale();
            path.line_cap = self.current_line_cap;
            path.line_join = self.current_line_join;
            path.dash_pattern = self.current_dash.as_ref().map(|(dashes, phase)| {
                let scale = self.ctm.stroke_scale();
                (dashes.iter().map(|d| d * scale).collect(), phase * scale)
            });
        } else {
            path.stroke_color = None;
        }

        // Set fill properties
        if fill && self.paint.fill_alpha > 0.0 {
            path.fill_color = self.current_fill_color;
        } else {
            path.fill_color = None;
        }

        // Attach the active Optional Content Group (PDF "layer") name, if
        // the path was emitted inside a `BDC /OC … EMC` region.
        path.layer = self.current_layer();

        if let Some(even_odd) = self.pending_clip.take() {
            self.paint.clip_path = Some(std::sync::Arc::new(crate::elements::PathClip {
                operations: path.operations.clone(),
                even_odd,
                parent: self.paint.clip_path.clone(),
            }));
            self.record_clip();
        }
        self.paths.push(path);

        // Reset state
        self.current_point = None;
        self.subpath_start = None;
    }

    /// Compute bounding box from path operations.
    fn compute_bbox(operations: &[PathOperation]) -> Rect {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for op in operations {
            match op {
                PathOperation::MoveTo(x, y) | PathOperation::LineTo(x, y) => {
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x);
                    max_y = max_y.max(*y);
                },
                PathOperation::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    for (x, y) in [(*x1, *y1), (*x2, *y2), (*x3, *y3)] {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                },
                PathOperation::Rectangle(x, y, w, h) => {
                    // ISO 32000-1 §8.5.2: re is an oriented closed subpath;
                    // either dimension may be negative. Preserve its winding
                    // in the operations, but include both ends in the raster
                    // bounds. Assuming positive sizes crops compound fills.
                    min_x = min_x.min(*x).min(*x + *w);
                    min_y = min_y.min(*y).min(*y + *h);
                    max_x = max_x.max(*x).max(*x + *w);
                    max_y = max_y.max(*y).max(*y + *h);
                },
                PathOperation::ClosePath => {},
            }
        }

        if min_x == f32::MAX {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
        }
    }

    /// Finish extraction and return all extracted paths.
    pub fn finish(self) -> Vec<PathContent> {
        self.paths
    }

    /// Opt in without changing ordinary path extraction or inventing painted paths.
    pub fn observe_clips(&mut self) {
        self.clip_observations = Some(Default::default());
    }

    /// Record the live graphics-state clip, never the nearest clip in stream order.
    pub(crate) fn record_text_clip(
        &mut self,
        state: &PathGraphicsState,
        budget: Option<&crate::import_limits::ImportBudget>,
    ) -> crate::error::Result<()> {
        if let Some(observations) = &mut self.clip_observations {
            if observations.text_uses.len() == 32_768 {
                observations.text_uses_omitted += 1;
                return Ok(());
            }
            if let Some(budget) = budget {
                budget.charge_entries(1)?;
                budget.charge_cache_copy(
                    self.paint.source_order.len() * std::mem::size_of::<usize>(),
                )?;
                observations
                    .text_uses
                    .try_reserve(1)
                    .map_err(|_| budget.reject("text clip observation allocation"))?;
            }
            if matches!(state.text_render_mode, 0 | 2 | 4 | 6)
                && matches!(state.fill_color_space, crate::path_color::PathColorSpace::Pattern(_))
            {
                return Err(crate::error::Error::UnsupportedTextPaint(
                    "unsupported text pattern fill".into(),
                ));
            }
            if matches!(state.text_render_mode, 1 | 2 | 5 | 6) {
                let m = state.ctm;
                let scale2 = m.a * m.a + m.b * m.b;
                // Line width lives in user space, not text space (9.3.6).
                // An anisotropic CTM needs an affine stroke, not an averaged width.
                // The editable consumer stores widths in 1/1000 pt. Orthogonal
                // axes that quantize to the very same width are representable
                // without adding a document-specific scale tolerance (printer
                // CTMs can carry independently rounded X/Y device resolutions).
                let width_x = state.line_width * scale2.sqrt();
                let width_y = state.line_width * (m.c * m.c + m.d * m.d).sqrt();
                if !scale2.is_finite()
                    || scale2 <= 0.0
                    || m.c * m.c + m.d * m.d <= 0.0
                    || (m.a * m.c + m.b * m.d).abs() > scale2 * 1e-6
                    || !width_y.is_finite()
                    || (f64::from(width_x) * 1000.0).round()
                        != (f64::from(width_y) * 1000.0).round()
                    || !state.line_width.is_finite()
                    || state.line_width < 0.0
                    || !state.miter_limit.is_finite()
                    || state.miter_limit < 1.0
                    || state.line_join > 2
                    || state.dash_pattern.is_some()
                    || matches!(
                        state.stroke_color_space,
                        crate::path_color::PathColorSpace::Unsupported
                            | crate::path_color::PathColorSpace::Pattern(_)
                    )
                    || state.text_render_mode >= 4
                {
                    return Err(crate::error::Error::UnsupportedTextPaint(
                        "unsupported text stroke transform, clipping or line style".into(),
                    ));
                }
                if let Some(budget) = budget {
                    budget.charge_entries(1)?;
                    budget.charge_cache_copy(
                        self.paint.source_order.len() * std::mem::size_of::<usize>(),
                    )?;
                    observations
                        .text_paints
                        .try_reserve(1)
                        .map_err(|_| budget.reject("text paint observation allocation"))?;
                }
                observations.text_paints.push((
                    self.paint.source_order.clone(),
                    crate::elements::TextStrokePaint {
                        fill: state.text_render_mode == 2,
                        width: width_x,
                        color: state.stroke_color_rgb,
                        fill_alpha: state.fill_alpha,
                        stroke_alpha: state.stroke_alpha,
                        line_join: state.line_join,
                        miter_limit: state.miter_limit,
                    },
                ));
            }
            observations
                .text_uses
                .push((self.paint.source_order.clone(), self.paint.clip_path.clone()));
        }
        Ok(())
    }

    /// Whether the caller requested the clip/use observation ledger.
    pub fn observing_clips(&self) -> bool {
        self.clip_observations.is_some()
    }

    fn record_clip(&mut self) {
        if let (Some(observations), Some(clip)) =
            (&mut self.clip_observations, &self.paint.clip_path)
        {
            // A diagnostic ledger must not retain arbitrarily many otherwise dead clips.
            if observations.clips.len() == 16_384 {
                observations.omitted += 1;
                return;
            }
            observations
                .clips
                .push(crate::elements::PathClipObservation {
                    source_order: self.paint.source_order.clone(),
                    transform: [
                        self.ctm.a, self.ctm.b, self.ctm.c, self.ctm.d, self.ctm.e, self.ctm.f,
                    ],
                    clip: clip.clone(),
                });
        }
    }

    /// Return original paint and the separately requested clipping ledger.
    pub fn finish_observed(self) -> (Vec<PathContent>, crate::elements::PathClipObservations) {
        (self.paths, self.clip_observations.unwrap_or_default())
    }

    /// Get the number of paths extracted so far.
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }

    /// Check if there's a path currently being constructed.
    pub fn has_current_path(&self) -> bool {
        !self.current_operations.is_empty()
    }
}

impl Default for PathExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_extractor_new() {
        let extractor = PathExtractor::new();
        assert_eq!(extractor.path_count(), 0);
        assert!(!extractor.has_current_path());
    }

    #[test]
    fn test_simple_line_stroke() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(10.0, 10.0);
        extractor.line_to(100.0, 10.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].operations.len(), 2);
        assert!(paths[0].has_stroke());
        assert!(!paths[0].has_fill());
    }

    #[test]
    fn test_rectangle_fill() {
        let mut extractor = PathExtractor::new();
        extractor.set_fill_color(Color::new(1.0, 0.0, 0.0));

        extractor.rectangle(50.0, 50.0, 100.0, 80.0);
        extractor.fill(FillRule::NonZero);

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].operations.len(), 1);
        assert!(!paths[0].has_stroke());
        assert!(paths[0].has_fill());

        // Check bbox
        assert_eq!(paths[0].bbox.x, 50.0);
        assert_eq!(paths[0].bbox.y, 50.0);
        assert_eq!(paths[0].bbox.width, 100.0);
        assert_eq!(paths[0].bbox.height, 80.0);
    }

    #[test]
    fn test_closed_path() {
        let mut extractor = PathExtractor::new();
        // Set fill color for fill_and_stroke to work
        extractor.set_fill_color(Color::new(0.5, 0.5, 0.5));

        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 0.0);
        extractor.line_to(100.0, 100.0);
        extractor.line_to(0.0, 100.0);
        extractor.close_path();
        extractor.fill_and_stroke(FillRule::NonZero);

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].operations.len(), 5); // MoveTo, 3x LineTo, ClosePath
        assert!(paths[0].has_stroke());
        assert!(paths[0].has_fill());
    }

    #[test]
    fn test_bezier_curve() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(0.0, 0.0);
        extractor.curve_to(25.0, 100.0, 75.0, 100.0, 100.0, 0.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].operations.len(), 2);
        assert!(matches!(paths[0].operations[1], PathOperation::CurveTo(_, _, _, _, _, _)));
    }

    #[test]
    fn test_multiple_paths() {
        let mut extractor = PathExtractor::new();

        // First path: horizontal line
        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 0.0);
        extractor.stroke();

        // Second path: vertical line
        extractor.move_to(50.0, 0.0);
        extractor.line_to(50.0, 100.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_end_path_clears_operations() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 100.0);
        extractor.end_path(); // Should discard the path

        let paths = extractor.finish();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_line_style_properties() {
        let mut extractor = PathExtractor::new();
        extractor.set_line_width(3.0);
        extractor.set_line_cap(LineCap::Round);
        extractor.set_line_join(LineJoin::Bevel);
        extractor.set_stroke_color(Color::new(0.0, 0.0, 1.0));

        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 100.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].stroke_width, 3.0);
        assert_eq!(paths[0].line_cap, LineCap::Round);
        assert_eq!(paths[0].line_join, LineJoin::Bevel);
    }

    #[test]
    fn test_ctm_transformation() {
        let mut extractor = PathExtractor::new();

        // Set a translation matrix (move everything by 50, 50)
        extractor.set_ctm(Matrix::translation(50.0, 50.0));

        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 0.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);

        // Check that the first operation was transformed
        if let PathOperation::MoveTo(x, y) = paths[0].operations[0] {
            assert_eq!(x, 50.0);
            assert_eq!(y, 50.0);
        } else {
            panic!("Expected MoveTo operation");
        }
    }

    #[test]
    fn test_bbox_calculation() {
        let mut extractor = PathExtractor::new();

        // Create a path with known bounds
        extractor.move_to(10.0, 20.0);
        extractor.line_to(110.0, 20.0);
        extractor.line_to(110.0, 120.0);
        extractor.line_to(10.0, 120.0);
        extractor.close_path();
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);

        let bbox = &paths[0].bbox;
        assert_eq!(bbox.x, 10.0);
        assert_eq!(bbox.y, 20.0);
        assert_eq!(bbox.width, 100.0);
        assert_eq!(bbox.height, 100.0);
    }

    #[test]
    fn test_curve_to_v() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(0.0, 0.0);
        extractor.curve_to_v(50.0, 100.0, 100.0, 0.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);

        // The v operator uses current point as first control point
        if let PathOperation::CurveTo(x1, y1, _, _, _, _) = paths[0].operations[1] {
            assert_eq!(x1, 0.0);
            assert_eq!(y1, 0.0);
        }
    }

    #[test]
    fn test_curve_to_y() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(0.0, 0.0);
        extractor.curve_to_y(50.0, 100.0, 100.0, 0.0);
        extractor.stroke();

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);

        // The y operator uses end point as second control point
        if let PathOperation::CurveTo(_, _, x2, y2, x3, y3) = paths[0].operations[1] {
            assert_eq!(x2, x3);
            assert_eq!(y2, y3);
        }
    }

    #[test]
    fn test_fill_even_odd() {
        let mut extractor = PathExtractor::new();
        extractor.set_fill_color(Color::new(0.0, 1.0, 0.0));

        extractor.rectangle(0.0, 0.0, 100.0, 100.0);
        extractor.fill(FillRule::EvenOdd);

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].has_fill());
    }

    #[test]
    fn test_close_and_stroke() {
        let mut extractor = PathExtractor::new();

        extractor.move_to(0.0, 0.0);
        extractor.line_to(100.0, 0.0);
        extractor.line_to(50.0, 100.0);
        extractor.close_and_stroke(); // Should add ClosePath and stroke

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        // Operations: MoveTo, LineTo, LineTo, ClosePath
        assert_eq!(paths[0].operations.len(), 4);
        assert!(matches!(paths[0].operations[3], PathOperation::ClosePath));
    }

    #[test]
    fn test_update_from_state() {
        let mut extractor = PathExtractor::new();

        let mut state = GraphicsState::new();
        state.line_width = 5.0;
        state.line_cap = 1; // Round
        state.line_join = 2; // Bevel
        state.stroke_color_rgb = (1.0, 0.0, 0.0);
        state.fill_color_rgb = (0.0, 1.0, 0.0);

        extractor.update_from_state(&state);

        extractor.rectangle(0.0, 0.0, 100.0, 100.0);
        extractor.fill_and_stroke(FillRule::NonZero);

        let paths = extractor.finish();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].stroke_width, 5.0);
        assert_eq!(paths[0].line_cap, LineCap::Round);
        assert_eq!(paths[0].line_join, LineJoin::Bevel);
    }

    #[test]
    fn test_pop_xobject_marks_as_processed() {
        let mut ext = PathExtractor::new();
        let r = crate::object::ObjectRef::new(42, 0);

        assert!(ext.can_process_xobject(r));
        ext.push_xobject(r);
        ext.pop_xobject(); // success path
        assert!(
            !ext.can_process_xobject(r),
            "Successfully processed XObject should be permanently skipped"
        );
    }

    #[test]
    fn test_pop_xobject_failed_allows_retry() {
        let mut ext = PathExtractor::new();
        let r = crate::object::ObjectRef::new(42, 0);

        assert!(ext.can_process_xobject(r));
        ext.push_xobject(r);
        ext.pop_xobject_failed(); // failure path
        assert!(ext.can_process_xobject(r), "Failed XObject should be retryable");
    }

    #[test]
    fn test_oc_layer_truncate_isolates_unbalanced_xobject() {
        // A page-level `/OC` region is active; we then descend into a Form
        // XObject that opens a `/OC` marker but (malformed) never closes it.
        // `truncate_oc_layers` back to the entry depth must drop the leaked
        // entry so the caller's next path keeps the page-level layer rather
        // than inheriting the XObject's.
        let mut ext = PathExtractor::new();
        ext.set_stroke_color(Color::black());

        ext.push_oc_layer(Some("A-GRID".to_string())); // page-level BDC /OC
        let base = ext.oc_layer_depth();

        // Inside the XObject: opens its own region but leaves it dangling.
        ext.push_oc_layer(Some("S-COLS".to_string()));
        ext.move_to(0.0, 0.0);
        ext.line_to(10.0, 0.0);
        ext.stroke(); // belongs to the XObject's S-COLS
                      // (no matching EMC — simulates a malformed XObject stream)
        ext.truncate_oc_layers(base); // XObject returns

        ext.move_to(0.0, 20.0);
        ext.line_to(10.0, 20.0);
        ext.stroke(); // back at page scope → A-GRID, not S-COLS

        let paths = ext.finish();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].layer.as_deref(), Some("S-COLS"));
        assert_eq!(paths[1].layer.as_deref(), Some("A-GRID"));
    }

    #[test]
    fn test_oc_layer_effective_is_o1_top_of_stack() {
        // push_oc_layer pre-resolves the effective layer, so a non-OC `BMC`
        // nested inside an `/OC` region inherits the OCG name and the top of
        // the stack is always the active layer.
        let mut ext = PathExtractor::new();
        ext.push_oc_layer(Some("A-WALL".to_string())); // BDC /OC
        ext.push_oc_layer(None); // BMC /Span (non-OC) inherits A-WALL
        assert_eq!(ext.current_layer().as_deref(), Some("A-WALL"));
        ext.pop_oc_layer();
        ext.pop_oc_layer();
        assert_eq!(ext.current_layer(), None);
    }
}
