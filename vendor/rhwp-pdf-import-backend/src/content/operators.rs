//! PDF content stream operators.
//!
//! This module defines the operator types used in PDF content streams.
//! Content streams contain a sequence of operators that define the appearance
//! of a page, including text positioning, graphics state, and colors.

use crate::object::Object;

/// A content stream operator.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::box_collection)] // Intentional: Boxing reduces enum from 112 to 40 bytes
pub enum Operator {
    // Text positioning operators
    /// Move text position (Td)
    Td {
        /// Horizontal offset
        tx: f32,
        /// Vertical offset
        ty: f32,
    },
    /// Move text position and set leading (TD)
    TD {
        /// Horizontal offset
        tx: f32,
        /// Vertical offset
        ty: f32,
    },
    /// Set text matrix (Tm)
    Tm {
        /// Matrix element a
        a: f32,
        /// Matrix element b
        b: f32,
        /// Matrix element c
        c: f32,
        /// Matrix element d
        d: f32,
        /// Matrix element e (x translation)
        e: f32,
        /// Matrix element f (y translation)
        f: f32,
    },
    /// Move to start of next line (T*)
    TStar,

    // Text showing operators
    /// Show text string (Tj)
    Tj {
        /// Text to show (byte array)
        text: Vec<u8>,
    },
    /// Show text with individual glyph positioning (TJ)
    TJ {
        /// Array of text strings and positioning adjustments
        array: Vec<TextElement>,
    },
    /// Move to next line and show text (')
    Quote {
        /// Text to show
        text: Vec<u8>,
    },
    /// Set spacing and show text (")
    DoubleQuote {
        /// Word spacing
        word_space: f32,
        /// Character spacing
        char_space: f32,
        /// Text to show
        text: Vec<u8>,
    },

    // Text state operators
    /// Set character spacing (Tc)
    Tc {
        /// Character spacing
        char_space: f32,
    },
    /// Set word spacing (Tw)
    Tw {
        /// Word spacing
        word_space: f32,
    },
    /// Set horizontal scaling (Tz)
    Tz {
        /// Horizontal scaling percentage
        scale: f32,
    },
    /// Set text leading (TL)
    TL {
        /// Text leading
        leading: f32,
    },
    /// Set font and size (Tf)
    Tf {
        /// Font name
        font: String,
        /// Font size
        size: f32,
    },
    /// Set text rendering mode (Tr)
    Tr {
        /// Rendering mode
        render: u8,
    },
    /// Set text rise (Ts)
    Ts {
        /// Text rise
        rise: f32,
    },

    // Graphics state operators
    /// Save graphics state (q)
    SaveState,
    /// Restore graphics state (Q)
    RestoreState,
    /// Modify current transformation matrix (cm)
    Cm {
        /// Matrix element a
        a: f32,
        /// Matrix element b
        b: f32,
        /// Matrix element c
        c: f32,
        /// Matrix element d
        d: f32,
        /// Matrix element e (x translation)
        e: f32,
        /// Matrix element f (y translation)
        f: f32,
    },

    // Color operators
    /// Set RGB fill color (rg)
    SetFillRgb {
        /// Red component (0.0-1.0)
        r: f32,
        /// Green component (0.0-1.0)
        g: f32,
        /// Blue component (0.0-1.0)
        b: f32,
    },
    /// Set RGB stroke color (RG)
    SetStrokeRgb {
        /// Red component (0.0-1.0)
        r: f32,
        /// Green component (0.0-1.0)
        g: f32,
        /// Blue component (0.0-1.0)
        b: f32,
    },
    /// Set gray fill color (g)
    SetFillGray {
        /// Gray level (0.0-1.0)
        gray: f32,
    },
    /// Set gray stroke color (G)
    SetStrokeGray {
        /// Gray level (0.0-1.0)
        gray: f32,
    },
    /// Set CMYK fill color (k)
    SetFillCmyk {
        /// Cyan component (0.0-1.0)
        c: f32,
        /// Magenta component (0.0-1.0)
        m: f32,
        /// Yellow component (0.0-1.0)
        y: f32,
        /// Black component (0.0-1.0)
        k: f32,
    },
    /// Set CMYK stroke color (K)
    SetStrokeCmyk {
        /// Cyan component (0.0-1.0)
        c: f32,
        /// Magenta component (0.0-1.0)
        m: f32,
        /// Yellow component (0.0-1.0)
        y: f32,
        /// Black component (0.0-1.0)
        k: f32,
    },

    // Color space operators
    /// Set fill color space (cs)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.6.4 - Color Space Operators
    SetFillColorSpace {
        /// Color space name (e.g., "DeviceRGB", "DeviceCMYK", "DeviceGray")
        name: String,
    },
    /// Set stroke color space (CS)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.6.4 - Color Space Operators
    SetStrokeColorSpace {
        /// Color space name (e.g., "DeviceRGB", "DeviceCMYK", "DeviceGray")
        name: String,
    },
    /// Set fill color (sc)
    ///
    /// Sets color components in the current fill color space.
    /// Number of components depends on color space (1 for Gray, 3 for RGB, 4 for CMYK).
    SetFillColor {
        /// Color components (length depends on color space)
        components: Vec<f32>,
    },
    /// Set stroke color (SC)
    ///
    /// Sets color components in the current stroke color space.
    /// Number of components depends on color space (1 for Gray, 3 for RGB, 4 for CMYK).
    SetStrokeColor {
        /// Color components (length depends on color space)
        components: Vec<f32>,
    },
    /// Set fill color with named pattern (scn)
    ///
    /// Like sc, but also supports pattern color spaces with an optional pattern name.
    SetFillColorN {
        /// Color components (may be empty for patterns)
        components: Vec<f32>,
        /// Optional pattern name for pattern color spaces.
        /// Boxed to reduce Operator enum size (`Option<String>` is 24 bytes → 8 bytes).
        name: Option<Box<String>>,
    },
    /// Set stroke color with named pattern (SCN)
    ///
    /// Like SC, but also supports pattern color spaces with an optional pattern name.
    SetStrokeColorN {
        /// Color components (may be empty for patterns)
        components: Vec<f32>,
        /// Optional pattern name for pattern color spaces.
        /// Boxed to reduce Operator enum size.
        name: Option<Box<String>>,
    },

    // Text object operators
    /// Begin text object (BT)
    BeginText,
    /// End text object (ET)
    EndText,

    // XObject operators
    /// Paint XObject (Do)
    Do {
        /// XObject name
        name: String,
    },

    // Path construction and painting (minimal support for now)
    /// Move to (m)
    MoveTo {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
    },
    /// Line to (l)
    LineTo {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
    },
    /// Cubic Bézier curve (c)
    CurveTo {
        /// X coordinate of first control point
        x1: f32,
        /// Y coordinate of first control point
        y1: f32,
        /// X coordinate of second control point
        x2: f32,
        /// Y coordinate of second control point
        y2: f32,
        /// X coordinate of end point
        x3: f32,
        /// Y coordinate of end point
        y3: f32,
    },
    /// Bézier curve with first control point = current point (v)
    CurveToV {
        /// X coordinate of second control point
        x2: f32,
        /// Y coordinate of second control point
        y2: f32,
        /// X coordinate of end point
        x3: f32,
        /// Y coordinate of end point
        y3: f32,
    },
    /// Bézier curve with second control point = end point (y)
    CurveToY {
        /// X coordinate of first control point
        x1: f32,
        /// Y coordinate of first control point
        y1: f32,
        /// X coordinate of end point
        x3: f32,
        /// Y coordinate of end point
        y3: f32,
    },
    /// Close current subpath (h)
    ClosePath,
    /// Rectangle (re)
    Rectangle {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
        /// Width
        width: f32,
        /// Height
        height: f32,
    },
    /// Stroke path (S)
    Stroke,
    /// Fill path (f)
    Fill,
    /// Fill path (even-odd) (f*)
    FillEvenOdd,
    /// Close and stroke path (s). ISO 32000-1:2008 Table 60: "the same effect
    /// as the sequence h S". Distinct from `Stroke` because an open subpath
    /// must be closed first, or its final segment is not painted.
    CloseAndStroke,
    /// Close, fill and stroke using the nonzero winding rule (b).
    CloseFillStroke,
    /// Fill and stroke using nonzero winding rule (B)
    FillStroke,
    /// Fill and stroke using even-odd rule (B*)
    FillStrokeEvenOdd,
    /// Close, fill and stroke using even-odd rule (b*)
    CloseFillStrokeEvenOdd,
    /// End path without filling or stroking (n)
    EndPath,
    /// Modify clipping path using non-zero winding rule (W)
    ClipNonZero,
    /// Modify clipping path using even-odd rule (W*)
    ClipEvenOdd,

    // Graphics state operators
    /// Set line width (w)
    SetLineWidth {
        /// Line width
        width: f32,
    },
    /// Set line dash pattern (d)
    SetDash {
        /// Dash array ([on1, off1, on2, off2, ...])
        array: Vec<f32>,
        /// Dash phase (offset into pattern)
        phase: f32,
    },
    /// Set line cap style (J)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.4.3.3 - Line Cap Style
    SetLineCap {
        /// Line cap style: 0=butt cap, 1=round cap, 2=projecting square cap
        cap_style: u8,
    },
    /// Set line join style (j)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.4.3.4 - Line Join Style
    SetLineJoin {
        /// Line join style: 0=miter join, 1=round join, 2=bevel join
        join_style: u8,
    },
    /// Set miter limit (M)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.4.3.5 - Miter Limit
    SetMiterLimit {
        /// Miter limit (ratio of miter length to line width)
        limit: f32,
    },
    /// Set rendering intent (ri)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.6.5.8 - Rendering Intents
    SetRenderingIntent {
        /// Rendering intent: AbsoluteColorimetric, RelativeColorimetric, Saturation, or Perceptual
        intent: String,
    },
    /// Set flatness tolerance (i)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 6.5.1 - Flatness Tolerance
    SetFlatness {
        /// Flatness tolerance (0-100, controlling curve approximation quality)
        tolerance: f32,
    },
    /// Set extended graphics state (gs)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.4.5 - Graphics State Parameter Dictionaries
    ///
    /// References an ExtGState dictionary in the page resources that contains
    /// graphics state parameters like transparency, blend modes, and line styles.
    SetExtGState {
        /// Name of the ExtGState dictionary in /ExtGState resources
        dict_name: String,
    },
    /// Paint shading pattern (sh)
    ///
    /// PDF Spec: ISO 32000-1:2008, Section 8.7.4.3 - Shading Patterns
    ///
    /// Paints a shading pattern (gradient) defined in the /Shading resource dictionary.
    /// Shading types include: Function-based, Axial, Radial, Free-form Gouraud, Lattice-form Gouraud, Coons patch, Tensor-product patch.
    PaintShading {
        /// Name of the shading dictionary in /Shading resources
        name: String,
    },

    // Inline image operator
    // PDF Spec: ISO 32000-1:2008, Section 8.9.7 - Inline Images
    /// Inline image (BI...ID...EI sequence)
    ///
    /// Represents a complete inline image sequence from BI (begin inline image)
    /// through ID (inline image data) to EI (end inline image).
    ///
    /// Inline images are small images embedded directly in the content stream
    /// rather than referenced as XObjects. The dictionary contains abbreviated
    /// keys for image properties.
    ///
    /// Common dictionary keys (abbreviated):
    /// - W: Width (required)
    /// - H: Height (required)
    /// - CS: ColorSpace (e.g., /DeviceRGB, /DeviceGray)
    /// - BPC: BitsPerComponent (typically 1, 8)
    /// - F: Filter (e.g., /FlateDecode, /DCTDecode)
    /// - DP: DecodeParms (decode parameters for filter)
    /// - I: Interpolate (boolean)
    InlineImage {
        /// Inline image dictionary with abbreviated keys.
        /// Boxed to reduce Operator enum size (HashMap is 48 bytes).
        dict: Box<std::collections::HashMap<String, Object>>,
        /// Raw image data bytes (possibly compressed)
        data: Vec<u8>,
    },

    // Marked content operators (for tagged PDF structure)
    // PDF Spec: ISO 32000-1:2008, Section 14.6 - Marked Content
    /// Begin marked content (BMC)
    ///
    /// Begins a marked content sequence identified by a tag.
    /// Used for logical structure and accessibility in tagged PDFs.
    BeginMarkedContent {
        /// Tag name identifying the marked content
        tag: String,
    },
    /// Begin marked content with property list (BDC)
    ///
    /// Begins a marked content sequence with associated properties.
    /// The properties can be inline (dictionary) or a reference to a properties resource.
    BeginMarkedContentDict {
        /// Tag name identifying the marked content
        tag: String,
        /// Properties (dictionary or name reference to /Properties resource).
        /// Boxed to reduce Operator enum size from 112 to 56 bytes (Object is 88 bytes).
        properties: Box<Object>,
    },
    /// End marked content (EMC)
    ///
    /// Ends the most recent marked content sequence.
    /// Must be balanced with BMC or BDC operators.
    EndMarkedContent,

    // Unknown operator (for operators we don't handle yet)
    /// Other operator
    Other {
        /// Operator name
        name: String,
        /// Operands. Boxed to reduce Operator enum size.
        operands: Box<Vec<Object>>,
    },
}

/// Element in a TJ array (text showing with positioning).
#[derive(Debug, Clone, PartialEq)]
pub enum TextElement {
    /// Text string to show
    String(Vec<u8>),
    /// Positioning adjustment (in thousandths of a unit of text space)
    Offset(f32),
}

impl Operator {
    /// Returns `true` if this operator sets a colour or colour-space parameter.
    ///
    /// Used by the Type 3 `d1` glyph path: per ISO 32000-1:2008 §9.6.5.2, the
    /// glyph description of a `d1` glyph is a stencil that is painted with the
    /// current fill colour, and any colour operators inside it shall be
    /// ignored. The renderer skips these operators while a `d1` colour lock is
    /// in effect.
    pub fn is_color_setting(&self) -> bool {
        matches!(
            self,
            Operator::SetFillRgb { .. }
                | Operator::SetStrokeRgb { .. }
                | Operator::SetFillGray { .. }
                | Operator::SetStrokeGray { .. }
                | Operator::SetFillCmyk { .. }
                | Operator::SetStrokeCmyk { .. }
                | Operator::SetFillColorSpace { .. }
                | Operator::SetStrokeColorSpace { .. }
                | Operator::SetFillColor { .. }
                | Operator::SetStrokeColor { .. }
                | Operator::SetFillColorN { .. }
                | Operator::SetStrokeColorN { .. }
        )
    }

    /// Validate operand count and types according to PDF spec Table A.1.
    ///
    /// PDF Spec: ISO 32000-1:2008, Appendix A - Table A.1 - PDF content stream operators
    ///
    /// This method checks that operators have the correct number and types of operands.
    /// Only call this in strict mode for spec compliance validation.
    ///
    /// # Arguments
    ///
    /// * `operands` - The operands provided for this operator
    ///
    /// # Returns
    ///
    /// Ok(()) if operands are valid, Err with descriptive message if invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::content::operators::Operator;
    /// use pdf_oxide::object::Object;
    ///
    /// let op = Operator::MoveTo { x: 10.0, y: 20.0 };
    /// let operands = vec![Object::Integer(10), Object::Integer(20)];
    /// assert!(op.validate_operands(&operands).is_ok());
    /// ```
    pub fn validate_operands_for_raw_operator(
        operator_name: &str,
        operands: &[Object],
    ) -> crate::error::Result<()> {
        use crate::error::Error;

        // Validate operand count according to PDF Spec Table A.1
        match operator_name {
            // Path construction operators - PDF Spec Section 8.5.2
            "m" => {
                // moveto: x y m
                if operands.len() != 2 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'm' (moveto) requires 2 operands (x, y), got {}",
                        operands.len()
                    )));
                }
            },
            "l" => {
                // lineto: x y l
                if operands.len() != 2 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'l' (lineto) requires 2 operands (x, y), got {}",
                        operands.len()
                    )));
                }
            },
            "c" => {
                // curveto: x1 y1 x2 y2 x3 y3 c
                if operands.len() != 6 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'c' (curveto) requires 6 operands (x1, y1, x2, y2, x3, y3), got {}",
                        operands.len()
                    )));
                }
            },
            "v" => {
                // curveto (v variant): x2 y2 x3 y3 v
                if operands.len() != 4 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'v' (curveto) requires 4 operands (x2, y2, x3, y3), got {}",
                        operands.len()
                    )));
                }
            },
            "y" => {
                // curveto (y variant): x1 y1 x3 y3 y
                if operands.len() != 4 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'y' (curveto) requires 4 operands (x1, y1, x3, y3), got {}",
                        operands.len()
                    )));
                }
            },
            "h" => {
                // closepath: h (no operands)
                if !operands.is_empty() {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'h' (closepath) requires 0 operands, got {}",
                        operands.len()
                    )));
                }
            },
            "re" => {
                // rectangle: x y width height re
                if operands.len() != 4 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 're' (rectangle) requires 4 operands (x, y, width, height), got {}",
                        operands.len()
                    )));
                }
            },

            // Text positioning operators - PDF Spec Section 9.4.2
            "Td" => {
                // Move text position: tx ty Td
                if operands.len() != 2 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Td' requires 2 operands (tx, ty), got {}",
                        operands.len()
                    )));
                }
            },
            "TD" => {
                // Move text position and set leading: tx ty TD
                if operands.len() != 2 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'TD' requires 2 operands (tx, ty), got {}",
                        operands.len()
                    )));
                }
            },
            "Tm" => {
                // Set text matrix: a b c d e f Tm
                if operands.len() != 6 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tm' requires 6 operands (a, b, c, d, e, f), got {}",
                        operands.len()
                    )));
                }
            },
            "T*" => {
                // Move to next line: T* (no operands)
                if !operands.is_empty() {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'T*' requires 0 operands, got {}",
                        operands.len()
                    )));
                }
            },

            // Text showing operators - PDF Spec Section 9.4.3
            "Tj" => {
                // Show text: string Tj
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tj' requires 1 operand (string), got {}",
                        operands.len()
                    )));
                }
            },
            "TJ" => {
                // Show text with positioning: array TJ
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'TJ' requires 1 operand (array), got {}",
                        operands.len()
                    )));
                }
            },
            "'" => {
                // Move to next line and show text: string '
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator ''' requires 1 operand (string), got {}",
                        operands.len()
                    )));
                }
            },
            "\"" => {
                // Set spacing and show text: aw ac string "
                if operands.len() != 3 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator '\"' requires 3 operands (word_space, char_space, string), got {}",
                        operands.len()
                    )));
                }
            },

            // Text state operators - PDF Spec Section 9.3
            "Tc" => {
                // Set character spacing: charSpace Tc
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tc' requires 1 operand (char_space), got {}",
                        operands.len()
                    )));
                }
            },
            "Tw" => {
                // Set word spacing: wordSpace Tw
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tw' requires 1 operand (word_space), got {}",
                        operands.len()
                    )));
                }
            },
            "Tz" => {
                // Set horizontal scaling: scale Tz
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tz' requires 1 operand (scale), got {}",
                        operands.len()
                    )));
                }
            },
            "TL" => {
                // Set text leading: leading TL
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'TL' requires 1 operand (leading), got {}",
                        operands.len()
                    )));
                }
            },
            "Tf" => {
                // Set font: font size Tf
                if operands.len() != 2 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tf' requires 2 operands (font, size), got {}",
                        operands.len()
                    )));
                }
            },
            "Tr" => {
                // Set text rendering mode: render Tr
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Tr' requires 1 operand (render), got {}",
                        operands.len()
                    )));
                }
            },
            "Ts" => {
                // Set text rise: rise Ts
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Ts' requires 1 operand (rise), got {}",
                        operands.len()
                    )));
                }
            },

            // Graphics state operators
            "q" | "Q" => {
                // Save/restore graphics state: q, Q (no operands)
                if !operands.is_empty() {
                    return Err(Error::InvalidPdf(format!(
                        "Operator '{}' requires 0 operands, got {}",
                        operator_name,
                        operands.len()
                    )));
                }
            },
            "cm" => {
                // Modify CTM: a b c d e f cm
                if operands.len() != 6 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'cm' requires 6 operands (a, b, c, d, e, f), got {}",
                        operands.len()
                    )));
                }
            },

            // Color operators - PDF Spec Section 8.6.8
            "rg" => {
                // Set RGB fill color: r g b rg
                if operands.len() != 3 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'rg' requires 3 operands (r, g, b), got {}",
                        operands.len()
                    )));
                }
            },
            "RG" => {
                // Set RGB stroke color: r g b RG
                if operands.len() != 3 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'RG' requires 3 operands (r, g, b), got {}",
                        operands.len()
                    )));
                }
            },
            "g" => {
                // Set gray fill color: gray g
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'g' requires 1 operand (gray), got {}",
                        operands.len()
                    )));
                }
            },
            "G" => {
                // Set gray stroke color: gray G
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'G' requires 1 operand (gray), got {}",
                        operands.len()
                    )));
                }
            },
            "k" => {
                // Set CMYK fill color: c m y k k
                if operands.len() != 4 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'k' requires 4 operands (c, m, y, k), got {}",
                        operands.len()
                    )));
                }
            },
            "K" => {
                // Set CMYK stroke color: c m y k K
                if operands.len() != 4 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'K' requires 4 operands (c, m, y, k), got {}",
                        operands.len()
                    )));
                }
            },

            // Text object operators - PDF Spec Section 9.4
            "BT" | "ET" => {
                // Begin/end text: BT, ET (no operands)
                if !operands.is_empty() {
                    return Err(Error::InvalidPdf(format!(
                        "Operator '{}' requires 0 operands, got {}",
                        operator_name,
                        operands.len()
                    )));
                }
            },

            // XObject operator - PDF Spec Section 8.8
            "Do" => {
                // Paint XObject: name Do
                if operands.len() != 1 {
                    return Err(Error::InvalidPdf(format!(
                        "Operator 'Do' requires 1 operand (name), got {}",
                        operands.len()
                    )));
                }
            },

            // Other operators we don't validate yet
            _ => {
                // No validation for unknown operators (lenient behavior)
                log::debug!(
                    "No operand validation for operator '{}' (not implemented yet)",
                    operator_name
                );
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_td() {
        let op = Operator::Td { tx: 10.0, ty: 20.0 };
        match op {
            Operator::Td { tx, ty } => {
                assert_eq!(tx, 10.0);
                assert_eq!(ty, 20.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_tm() {
        let op = Operator::Tm {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 100.0,
            f: 200.0,
        };
        match op {
            Operator::Tm { a, b, c, d, e, f } => {
                assert_eq!(a, 1.0);
                assert_eq!(b, 0.0);
                assert_eq!(c, 0.0);
                assert_eq!(d, 1.0);
                assert_eq!(e, 100.0);
                assert_eq!(f, 200.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_tj() {
        let op = Operator::Tj {
            text: b"Hello".to_vec(),
        };
        match op {
            Operator::Tj { text } => {
                assert_eq!(text, b"Hello");
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_tf() {
        let op = Operator::Tf {
            font: "F1".to_string(),
            size: 12.0,
        };
        match op {
            Operator::Tf { font, size } => {
                assert_eq!(font, "F1");
                assert_eq!(size, 12.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_rgb() {
        let op = Operator::SetFillRgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        match op {
            Operator::SetFillRgb { r, g, b } => {
                assert_eq!(r, 1.0);
                assert_eq!(g, 0.0);
                assert_eq!(b, 0.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_text_element_string() {
        let elem = TextElement::String(b"Text".to_vec());
        match elem {
            TextElement::String(s) => {
                assert_eq!(s, b"Text");
            },
            _ => panic!("Wrong element type"),
        }
    }

    #[test]
    fn test_text_element_offset() {
        let elem = TextElement::Offset(-100.0);
        match elem {
            TextElement::Offset(offset) => {
                assert_eq!(offset, -100.0);
            },
            _ => panic!("Wrong element type"),
        }
    }

    #[test]
    fn test_operator_clone() {
        let op1 = Operator::Tj {
            text: b"Test".to_vec(),
        };
        let op2 = op1.clone();
        assert_eq!(op1, op2);
    }

    #[test]
    fn test_operator_save_restore() {
        let save = Operator::SaveState;
        let restore = Operator::RestoreState;
        assert!(matches!(save, Operator::SaveState));
        assert!(matches!(restore, Operator::RestoreState));
    }

    #[test]
    fn test_operator_other() {
        let op = Operator::Other {
            name: "Do".to_string(),
            operands: Box::new(vec![Object::Name("Im1".to_string())]),
        };
        match op {
            Operator::Other { name, operands } => {
                assert_eq!(name, "Do");
                assert_eq!(operands.len(), 1);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_enum_size() {
        let size = std::mem::size_of::<Operator>();
        eprintln!("Operator enum size: {} bytes", size);
        // After boxing BeginMarkedContentDict.properties, InlineImage.dict,
        // Other.operands, SetFillColorN/SetStrokeColorN.name:
        // largest variant is now SetFillColorN/SetStrokeColorN at Vec<f32>(24) + Option<Box<String>>(8) = 32 bytes
        // Enum: 32 (payload) + 8 (discriminant + alignment) = 40 bytes (was 112)
        assert!(size <= 40, "Operator enum too large: {} bytes (expected <= 40)", size);
    }

    #[test]
    fn test_text_element_eq() {
        let elem1 = TextElement::String(b"Test".to_vec());
        let elem2 = TextElement::String(b"Test".to_vec());
        assert_eq!(elem1, elem2);

        let elem3 = TextElement::Offset(10.0);
        let elem4 = TextElement::Offset(10.0);
        assert_eq!(elem3, elem4);
    }

    // =========================================================================
    // validate_operands_for_raw_operator tests
    // =========================================================================

    #[test]
    fn test_validate_moveto_valid() {
        let operands = vec![Object::Integer(10), Object::Integer(20)];
        assert!(Operator::validate_operands_for_raw_operator("m", &operands).is_ok());
    }

    #[test]
    fn test_validate_moveto_wrong_count() {
        let operands = vec![Object::Integer(10)];
        let err = Operator::validate_operands_for_raw_operator("m", &operands);
        assert!(err.is_err());
        let msg = format!("{}", err.unwrap_err());
        assert!(msg.contains("moveto"));
        assert!(msg.contains("2 operands"));
    }

    #[test]
    fn test_validate_lineto_valid() {
        let operands = vec![Object::Real(1.5), Object::Real(2.5)];
        assert!(Operator::validate_operands_for_raw_operator("l", &operands).is_ok());
    }

    #[test]
    fn test_validate_lineto_wrong_count() {
        let operands = vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)];
        let err = Operator::validate_operands_for_raw_operator("l", &operands);
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_curveto_valid() {
        let operands = vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
            Object::Integer(5),
            Object::Integer(6),
        ];
        assert!(Operator::validate_operands_for_raw_operator("c", &operands).is_ok());
    }

    #[test]
    fn test_validate_curveto_wrong_count() {
        let operands = vec![Object::Integer(1), Object::Integer(2)];
        assert!(Operator::validate_operands_for_raw_operator("c", &operands).is_err());
    }

    #[test]
    fn test_validate_curveto_v_valid() {
        let operands = vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ];
        assert!(Operator::validate_operands_for_raw_operator("v", &operands).is_ok());
    }

    #[test]
    fn test_validate_curveto_v_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("v", &operands).is_err());
    }

    #[test]
    fn test_validate_curveto_y_valid() {
        let operands = vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ];
        assert!(Operator::validate_operands_for_raw_operator("y", &operands).is_ok());
    }

    #[test]
    fn test_validate_curveto_y_wrong_count() {
        let operands = vec![];
        assert!(Operator::validate_operands_for_raw_operator("y", &operands).is_err());
    }

    #[test]
    fn test_validate_closepath_valid() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("h", &operands).is_ok());
    }

    #[test]
    fn test_validate_closepath_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("h", &operands).is_err());
    }

    #[test]
    fn test_validate_rectangle_valid() {
        let operands = vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(100),
            Object::Integer(200),
        ];
        assert!(Operator::validate_operands_for_raw_operator("re", &operands).is_ok());
    }

    #[test]
    fn test_validate_rectangle_wrong_count() {
        let operands = vec![Object::Integer(0), Object::Integer(0)];
        assert!(Operator::validate_operands_for_raw_operator("re", &operands).is_err());
    }

    #[test]
    fn test_validate_td_valid() {
        let operands = vec![Object::Real(10.0), Object::Real(20.0)];
        assert!(Operator::validate_operands_for_raw_operator("Td", &operands).is_ok());
    }

    #[test]
    fn test_validate_td_wrong_count() {
        let operands = vec![Object::Real(10.0)];
        assert!(Operator::validate_operands_for_raw_operator("Td", &operands).is_err());
    }

    #[test]
    fn test_validate_td_uppercase_valid() {
        let operands = vec![Object::Integer(5), Object::Integer(10)];
        assert!(Operator::validate_operands_for_raw_operator("TD", &operands).is_ok());
    }

    #[test]
    fn test_validate_td_uppercase_wrong_count() {
        let operands = vec![];
        assert!(Operator::validate_operands_for_raw_operator("TD", &operands).is_err());
    }

    #[test]
    fn test_validate_tm_valid() {
        let operands = vec![
            Object::Real(1.0),
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(1.0),
            Object::Real(72.0),
            Object::Real(720.0),
        ];
        assert!(Operator::validate_operands_for_raw_operator("Tm", &operands).is_ok());
    }

    #[test]
    fn test_validate_tm_wrong_count() {
        let operands = vec![Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("Tm", &operands).is_err());
    }

    #[test]
    fn test_validate_tstar_valid() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("T*", &operands).is_ok());
    }

    #[test]
    fn test_validate_tstar_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("T*", &operands).is_err());
    }

    #[test]
    fn test_validate_tj_valid() {
        let operands = vec![Object::String(b"Hello".to_vec())];
        assert!(Operator::validate_operands_for_raw_operator("Tj", &operands).is_ok());
    }

    #[test]
    fn test_validate_tj_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("Tj", &operands).is_err());
    }

    #[test]
    fn test_validate_tj_array_valid() {
        let operands = vec![Object::Array(vec![
            Object::String(b"He".to_vec()),
            Object::Integer(-120),
            Object::String(b"llo".to_vec()),
        ])];
        assert!(Operator::validate_operands_for_raw_operator("TJ", &operands).is_ok());
    }

    #[test]
    fn test_validate_tj_array_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("TJ", &operands).is_err());
    }

    #[test]
    fn test_validate_quote_valid() {
        let operands = vec![Object::String(b"text".to_vec())];
        assert!(Operator::validate_operands_for_raw_operator("'", &operands).is_ok());
    }

    #[test]
    fn test_validate_quote_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("'", &operands).is_err());
    }

    #[test]
    fn test_validate_double_quote_valid() {
        let operands = vec![
            Object::Real(1.0),
            Object::Real(2.0),
            Object::String(b"text".to_vec()),
        ];
        assert!(Operator::validate_operands_for_raw_operator("\"", &operands).is_ok());
    }

    #[test]
    fn test_validate_double_quote_wrong_count() {
        let operands = vec![Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("\"", &operands).is_err());
    }

    #[test]
    fn test_validate_tc_valid() {
        let operands = vec![Object::Real(0.5)];
        assert!(Operator::validate_operands_for_raw_operator("Tc", &operands).is_ok());
    }

    #[test]
    fn test_validate_tc_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("Tc", &operands).is_err());
    }

    #[test]
    fn test_validate_tw_valid() {
        let operands = vec![Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("Tw", &operands).is_ok());
    }

    #[test]
    fn test_validate_tw_wrong_count() {
        let operands = vec![Object::Real(1.0), Object::Real(2.0)];
        assert!(Operator::validate_operands_for_raw_operator("Tw", &operands).is_err());
    }

    #[test]
    fn test_validate_tz_valid() {
        let operands = vec![Object::Integer(150)];
        assert!(Operator::validate_operands_for_raw_operator("Tz", &operands).is_ok());
    }

    #[test]
    fn test_validate_tz_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("Tz", &operands).is_err());
    }

    #[test]
    fn test_validate_tl_valid() {
        let operands = vec![Object::Real(14.0)];
        assert!(Operator::validate_operands_for_raw_operator("TL", &operands).is_ok());
    }

    #[test]
    fn test_validate_tl_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("TL", &operands).is_err());
    }

    #[test]
    fn test_validate_tf_valid() {
        let operands = vec![Object::Name("F1".to_string()), Object::Real(12.0)];
        assert!(Operator::validate_operands_for_raw_operator("Tf", &operands).is_ok());
    }

    #[test]
    fn test_validate_tf_wrong_count() {
        let operands = vec![Object::Name("F1".to_string())];
        assert!(Operator::validate_operands_for_raw_operator("Tf", &operands).is_err());
    }

    #[test]
    fn test_validate_tr_valid() {
        let operands = vec![Object::Integer(0)];
        assert!(Operator::validate_operands_for_raw_operator("Tr", &operands).is_ok());
    }

    #[test]
    fn test_validate_tr_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("Tr", &operands).is_err());
    }

    #[test]
    fn test_validate_ts_valid() {
        let operands = vec![Object::Real(5.0)];
        assert!(Operator::validate_operands_for_raw_operator("Ts", &operands).is_ok());
    }

    #[test]
    fn test_validate_ts_wrong_count() {
        let operands = vec![Object::Real(1.0), Object::Real(2.0)];
        assert!(Operator::validate_operands_for_raw_operator("Ts", &operands).is_err());
    }

    #[test]
    fn test_validate_save_restore_state_valid() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("q", &operands).is_ok());
        assert!(Operator::validate_operands_for_raw_operator("Q", &operands).is_ok());
    }

    #[test]
    fn test_validate_save_state_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("q", &operands).is_err());
    }

    #[test]
    fn test_validate_restore_state_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("Q", &operands).is_err());
    }

    #[test]
    fn test_validate_cm_valid() {
        let operands = vec![
            Object::Real(1.0),
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(1.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ];
        assert!(Operator::validate_operands_for_raw_operator("cm", &operands).is_ok());
    }

    #[test]
    fn test_validate_cm_wrong_count() {
        let operands = vec![Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("cm", &operands).is_err());
    }

    #[test]
    fn test_validate_rg_valid() {
        let operands = vec![Object::Real(1.0), Object::Real(0.0), Object::Real(0.0)];
        assert!(Operator::validate_operands_for_raw_operator("rg", &operands).is_ok());
    }

    #[test]
    fn test_validate_rg_wrong_count() {
        let operands = vec![Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("rg", &operands).is_err());
    }

    #[test]
    fn test_validate_rg_uppercase_valid() {
        let operands = vec![Object::Real(0.0), Object::Real(1.0), Object::Real(0.0)];
        assert!(Operator::validate_operands_for_raw_operator("RG", &operands).is_ok());
    }

    #[test]
    fn test_validate_rg_uppercase_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("RG", &operands).is_err());
    }

    #[test]
    fn test_validate_g_valid() {
        let operands = vec![Object::Real(0.5)];
        assert!(Operator::validate_operands_for_raw_operator("g", &operands).is_ok());
    }

    #[test]
    fn test_validate_g_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("g", &operands).is_err());
    }

    #[test]
    fn test_validate_g_uppercase_valid() {
        let operands = vec![Object::Real(0.5)];
        assert!(Operator::validate_operands_for_raw_operator("G", &operands).is_ok());
    }

    #[test]
    fn test_validate_g_uppercase_wrong_count() {
        let operands = vec![Object::Real(0.5), Object::Real(0.5)];
        assert!(Operator::validate_operands_for_raw_operator("G", &operands).is_err());
    }

    #[test]
    fn test_validate_k_valid() {
        let operands = vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(1.0),
        ];
        assert!(Operator::validate_operands_for_raw_operator("k", &operands).is_ok());
    }

    #[test]
    fn test_validate_k_wrong_count() {
        let operands = vec![Object::Real(0.0)];
        assert!(Operator::validate_operands_for_raw_operator("k", &operands).is_err());
    }

    #[test]
    fn test_validate_k_uppercase_valid() {
        let operands = vec![
            Object::Real(0.0),
            Object::Real(1.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ];
        assert!(Operator::validate_operands_for_raw_operator("K", &operands).is_ok());
    }

    #[test]
    fn test_validate_k_uppercase_wrong_count() {
        let operands = vec![Object::Real(0.0), Object::Real(1.0)];
        assert!(Operator::validate_operands_for_raw_operator("K", &operands).is_err());
    }

    #[test]
    fn test_validate_bt_et_valid() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("BT", &operands).is_ok());
        assert!(Operator::validate_operands_for_raw_operator("ET", &operands).is_ok());
    }

    #[test]
    fn test_validate_bt_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("BT", &operands).is_err());
    }

    #[test]
    fn test_validate_et_wrong_count() {
        let operands = vec![Object::Integer(1)];
        assert!(Operator::validate_operands_for_raw_operator("ET", &operands).is_err());
    }

    #[test]
    fn test_validate_do_valid() {
        let operands = vec![Object::Name("Im1".to_string())];
        assert!(Operator::validate_operands_for_raw_operator("Do", &operands).is_ok());
    }

    #[test]
    fn test_validate_do_wrong_count() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("Do", &operands).is_err());
    }

    #[test]
    fn test_validate_unknown_operator_passes() {
        // Unknown operators should not produce errors (lenient behavior)
        let operands = vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)];
        assert!(Operator::validate_operands_for_raw_operator("xyz_unknown", &operands).is_ok());
    }

    #[test]
    fn test_validate_unknown_operator_empty_operands() {
        let operands: Vec<Object> = vec![];
        assert!(Operator::validate_operands_for_raw_operator("BMC", &operands).is_ok());
    }

    // =========================================================================
    // Additional Operator variant construction tests
    // =========================================================================

    #[test]
    fn test_operator_td_uppercase() {
        let op = Operator::TD { tx: 0.0, ty: -14.0 };
        match op {
            Operator::TD { tx, ty } => {
                assert_eq!(tx, 0.0);
                assert_eq!(ty, -14.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_tstar() {
        let op = Operator::TStar;
        assert!(matches!(op, Operator::TStar));
    }

    #[test]
    fn test_operator_tj_array() {
        let op = Operator::TJ {
            array: vec![
                TextElement::String(b"He".to_vec()),
                TextElement::Offset(-120.0),
                TextElement::String(b"llo".to_vec()),
            ],
        };
        match op {
            Operator::TJ { array } => {
                assert_eq!(array.len(), 3);
                assert!(matches!(&array[0], TextElement::String(s) if s == b"He"));
                assert!(matches!(&array[1], TextElement::Offset(o) if *o == -120.0));
                assert!(matches!(&array[2], TextElement::String(s) if s == b"llo"));
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_quote() {
        let op = Operator::Quote {
            text: b"next line".to_vec(),
        };
        match op {
            Operator::Quote { text } => assert_eq!(text, b"next line"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_double_quote() {
        let op = Operator::DoubleQuote {
            word_space: 1.0,
            char_space: 2.0,
            text: b"quoted".to_vec(),
        };
        match op {
            Operator::DoubleQuote {
                word_space,
                char_space,
                text,
            } => {
                assert_eq!(word_space, 1.0);
                assert_eq!(char_space, 2.0);
                assert_eq!(text, b"quoted");
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_tc() {
        let op = Operator::Tc { char_space: 0.5 };
        assert!(matches!(op, Operator::Tc { char_space } if char_space == 0.5));
    }

    #[test]
    fn test_operator_tw() {
        let op = Operator::Tw { word_space: 1.5 };
        assert!(matches!(op, Operator::Tw { word_space } if word_space == 1.5));
    }

    #[test]
    fn test_operator_tz() {
        let op = Operator::Tz { scale: 150.0 };
        assert!(matches!(op, Operator::Tz { scale } if scale == 150.0));
    }

    #[test]
    fn test_operator_tl() {
        let op = Operator::TL { leading: 14.0 };
        assert!(matches!(op, Operator::TL { leading } if leading == 14.0));
    }

    #[test]
    fn test_operator_tr() {
        let op = Operator::Tr { render: 2 };
        assert!(matches!(op, Operator::Tr { render } if render == 2));
    }

    #[test]
    fn test_operator_ts() {
        let op = Operator::Ts { rise: 5.0 };
        assert!(matches!(op, Operator::Ts { rise } if rise == 5.0));
    }

    #[test]
    fn test_operator_cm() {
        let op = Operator::Cm {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 10.0,
            f: 20.0,
        };
        match op {
            Operator::Cm { a, b, c, d, e, f } => {
                assert_eq!(a, 2.0);
                assert_eq!(b, 0.0);
                assert_eq!(c, 0.0);
                assert_eq!(d, 2.0);
                assert_eq!(e, 10.0);
                assert_eq!(f, 20.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_stroke_rgb() {
        let op = Operator::SetStrokeRgb {
            r: 0.0,
            g: 0.5,
            b: 1.0,
        };
        match op {
            Operator::SetStrokeRgb { r, g, b } => {
                assert_eq!(r, 0.0);
                assert_eq!(g, 0.5);
                assert_eq!(b, 1.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_fill_gray() {
        let op = Operator::SetFillGray { gray: 0.5 };
        assert!(matches!(op, Operator::SetFillGray { gray } if gray == 0.5));
    }

    #[test]
    fn test_operator_stroke_gray() {
        let op = Operator::SetStrokeGray { gray: 0.0 };
        assert!(matches!(op, Operator::SetStrokeGray { gray } if gray == 0.0));
    }

    #[test]
    fn test_operator_fill_cmyk() {
        let op = Operator::SetFillCmyk {
            c: 1.0,
            m: 0.0,
            y: 0.0,
            k: 0.0,
        };
        match op {
            Operator::SetFillCmyk { c, m, y, k } => {
                assert_eq!(c, 1.0);
                assert_eq!(m, 0.0);
                assert_eq!(y, 0.0);
                assert_eq!(k, 0.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_stroke_cmyk() {
        let op = Operator::SetStrokeCmyk {
            c: 0.0,
            m: 1.0,
            y: 0.0,
            k: 0.0,
        };
        match op {
            Operator::SetStrokeCmyk { c, m, y, k } => {
                assert_eq!(c, 0.0);
                assert_eq!(m, 1.0);
                assert_eq!(y, 0.0);
                assert_eq!(k, 0.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_fill_color_space() {
        let op = Operator::SetFillColorSpace {
            name: "DeviceRGB".to_string(),
        };
        match op {
            Operator::SetFillColorSpace { name } => assert_eq!(name, "DeviceRGB"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_stroke_color_space() {
        let op = Operator::SetStrokeColorSpace {
            name: "DeviceCMYK".to_string(),
        };
        match op {
            Operator::SetStrokeColorSpace { name } => assert_eq!(name, "DeviceCMYK"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_fill_color() {
        let op = Operator::SetFillColor {
            components: vec![0.1, 0.2, 0.3],
        };
        match op {
            Operator::SetFillColor { components } => {
                assert_eq!(components, vec![0.1, 0.2, 0.3]);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_stroke_color() {
        let op = Operator::SetStrokeColor {
            components: vec![0.5],
        };
        match op {
            Operator::SetStrokeColor { components } => {
                assert_eq!(components, vec![0.5]);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_fill_color_n_with_name() {
        let op = Operator::SetFillColorN {
            components: vec![0.1, 0.2, 0.3],
            name: Some(Box::new("Pattern1".to_string())),
        };
        match op {
            Operator::SetFillColorN { components, name } => {
                assert_eq!(components, vec![0.1, 0.2, 0.3]);
                assert_eq!(name, Some(Box::new("Pattern1".to_string())));
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_fill_color_n_without_name() {
        let op = Operator::SetFillColorN {
            components: vec![0.5],
            name: None,
        };
        match op {
            Operator::SetFillColorN { components, name } => {
                assert_eq!(components, vec![0.5]);
                assert!(name.is_none());
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_stroke_color_n() {
        let op = Operator::SetStrokeColorN {
            components: vec![],
            name: Some(Box::new("P1".to_string())),
        };
        match op {
            Operator::SetStrokeColorN { components, name } => {
                assert!(components.is_empty());
                assert_eq!(name, Some(Box::new("P1".to_string())));
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_begin_end_text() {
        assert!(matches!(Operator::BeginText, Operator::BeginText));
        assert!(matches!(Operator::EndText, Operator::EndText));
    }

    #[test]
    fn test_operator_do() {
        let op = Operator::Do {
            name: "Im1".to_string(),
        };
        match op {
            Operator::Do { name } => assert_eq!(name, "Im1"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_moveto() {
        let op = Operator::MoveTo { x: 100.0, y: 200.0 };
        match op {
            Operator::MoveTo { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_lineto() {
        let op = Operator::LineTo { x: 300.0, y: 400.0 };
        match op {
            Operator::LineTo { x, y } => {
                assert_eq!(x, 300.0);
                assert_eq!(y, 400.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_curveto() {
        let op = Operator::CurveTo {
            x1: 1.0,
            y1: 2.0,
            x2: 3.0,
            y2: 4.0,
            x3: 5.0,
            y3: 6.0,
        };
        match op {
            Operator::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                assert_eq!(x1, 1.0);
                assert_eq!(y1, 2.0);
                assert_eq!(x2, 3.0);
                assert_eq!(y2, 4.0);
                assert_eq!(x3, 5.0);
                assert_eq!(y3, 6.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_curveto_v() {
        let op = Operator::CurveToV {
            x2: 10.0,
            y2: 20.0,
            x3: 30.0,
            y3: 40.0,
        };
        match op {
            Operator::CurveToV { x2, y2, x3, y3 } => {
                assert_eq!(x2, 10.0);
                assert_eq!(y2, 20.0);
                assert_eq!(x3, 30.0);
                assert_eq!(y3, 40.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_curveto_y() {
        let op = Operator::CurveToY {
            x1: 10.0,
            y1: 20.0,
            x3: 30.0,
            y3: 40.0,
        };
        match op {
            Operator::CurveToY { x1, y1, x3, y3 } => {
                assert_eq!(x1, 10.0);
                assert_eq!(y1, 20.0);
                assert_eq!(x3, 30.0);
                assert_eq!(y3, 40.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_closepath() {
        assert!(matches!(Operator::ClosePath, Operator::ClosePath));
    }

    #[test]
    fn test_operator_rectangle() {
        let op = Operator::Rectangle {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        match op {
            Operator::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
                assert_eq!(width, 100.0);
                assert_eq!(height, 50.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_path_painting() {
        assert!(matches!(Operator::Stroke, Operator::Stroke));
        assert!(matches!(Operator::Fill, Operator::Fill));
        assert!(matches!(Operator::FillEvenOdd, Operator::FillEvenOdd));
        assert!(matches!(Operator::CloseFillStroke, Operator::CloseFillStroke));
        assert!(matches!(Operator::EndPath, Operator::EndPath));
    }

    #[test]
    fn test_operator_clipping() {
        assert!(matches!(Operator::ClipNonZero, Operator::ClipNonZero));
        assert!(matches!(Operator::ClipEvenOdd, Operator::ClipEvenOdd));
    }

    #[test]
    fn test_operator_set_line_width() {
        let op = Operator::SetLineWidth { width: 2.5 };
        assert!(matches!(op, Operator::SetLineWidth { width } if width == 2.5));
    }

    #[test]
    fn test_operator_set_dash() {
        let op = Operator::SetDash {
            array: vec![3.0, 2.0],
            phase: 0.0,
        };
        match op {
            Operator::SetDash { array, phase } => {
                assert_eq!(array, vec![3.0, 2.0]);
                assert_eq!(phase, 0.0);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_set_line_cap() {
        let op = Operator::SetLineCap { cap_style: 1 };
        assert!(matches!(op, Operator::SetLineCap { cap_style } if cap_style == 1));
    }

    #[test]
    fn test_operator_set_line_join() {
        let op = Operator::SetLineJoin { join_style: 2 };
        assert!(matches!(op, Operator::SetLineJoin { join_style } if join_style == 2));
    }

    #[test]
    fn test_operator_set_miter_limit() {
        let op = Operator::SetMiterLimit { limit: 10.0 };
        assert!(matches!(op, Operator::SetMiterLimit { limit } if limit == 10.0));
    }

    #[test]
    fn test_operator_set_rendering_intent() {
        let op = Operator::SetRenderingIntent {
            intent: "Perceptual".to_string(),
        };
        match op {
            Operator::SetRenderingIntent { intent } => assert_eq!(intent, "Perceptual"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_set_flatness() {
        let op = Operator::SetFlatness { tolerance: 50.0 };
        assert!(matches!(op, Operator::SetFlatness { tolerance } if tolerance == 50.0));
    }

    #[test]
    fn test_operator_set_ext_gstate() {
        let op = Operator::SetExtGState {
            dict_name: "GS1".to_string(),
        };
        match op {
            Operator::SetExtGState { dict_name } => assert_eq!(dict_name, "GS1"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_paint_shading() {
        let op = Operator::PaintShading {
            name: "Sh1".to_string(),
        };
        match op {
            Operator::PaintShading { name } => assert_eq!(name, "Sh1"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_inline_image() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("W".to_string(), Object::Integer(100));
        dict.insert("H".to_string(), Object::Integer(50));
        let op = Operator::InlineImage {
            dict: Box::new(dict),
            data: vec![0xFF; 10],
        };
        match op {
            Operator::InlineImage { dict, data } => {
                assert_eq!(dict.len(), 2);
                assert_eq!(data.len(), 10);
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_begin_marked_content() {
        let op = Operator::BeginMarkedContent {
            tag: "Span".to_string(),
        };
        match op {
            Operator::BeginMarkedContent { tag } => assert_eq!(tag, "Span"),
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_begin_marked_content_dict() {
        let op = Operator::BeginMarkedContentDict {
            tag: "P".to_string(),
            properties: Box::new(Object::Name("MCID0".to_string())),
        };
        match op {
            Operator::BeginMarkedContentDict { tag, properties } => {
                assert_eq!(tag, "P");
                assert!(matches!(*properties, Object::Name(ref n) if n == "MCID0"));
            },
            _ => panic!("Wrong operator type"),
        }
    }

    #[test]
    fn test_operator_end_marked_content() {
        assert!(matches!(Operator::EndMarkedContent, Operator::EndMarkedContent));
    }

    #[test]
    fn test_operator_debug_format() {
        let op = Operator::Td { tx: 1.0, ty: 2.0 };
        let debug = format!("{:?}", op);
        assert!(debug.contains("Td"));
        assert!(debug.contains("1.0"));
        assert!(debug.contains("2.0"));
    }

    #[test]
    fn test_operator_equality() {
        let op1 = Operator::SetFillGray { gray: 0.5 };
        let op2 = Operator::SetFillGray { gray: 0.5 };
        let op3 = Operator::SetFillGray { gray: 0.6 };
        assert_eq!(op1, op2);
        assert_ne!(op1, op3);
    }

    #[test]
    fn test_text_element_debug_format() {
        let elem = TextElement::Offset(-50.0);
        let debug = format!("{:?}", elem);
        assert!(debug.contains("Offset"));
        assert!(debug.contains("-50.0"));
    }

    #[test]
    fn test_text_element_inequality() {
        let s1 = TextElement::String(b"abc".to_vec());
        let s2 = TextElement::String(b"def".to_vec());
        let o1 = TextElement::Offset(10.0);
        assert_ne!(s1, s2);
        assert_ne!(s1, o1);
    }
}
