//! 3D annotations for PDF generation.
//!
//! This module provides support for 3D annotations per PDF spec Section 12.5.6.24.
//! 3D annotations embed 3D models in U3D or PRC format that can be interactively
//! manipulated by viewers.
//!
//! # Supported Formats
//!
//! - **U3D** (Universal 3D): ECMA-363 format, widely supported
//! - **PRC** (Product Representation Compact): ISO 14739-1 format
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{ThreeDAnnotation, ThreeDStream, ThreeDView};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a 3D annotation with a U3D model
//! let model = ThreeDStream::u3d(model_data);
//! let view = ThreeDView::default_view()
//!     .with_camera_distance(100.0);
//!
//! let annot = ThreeDAnnotation::new(
//!     Rect::new(72.0, 400.0, 400.0, 300.0),
//!     model,
//! ).with_view(view);
//! ```

// Allow uppercase acronyms for industry-standard format names (U3D, PRC, CAD)
#![allow(clippy::upper_case_acronyms)]

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// 3D model format type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDFormat {
    /// Universal 3D format (ECMA-363)
    #[default]
    U3D,
    /// Product Representation Compact (ISO 14739-1)
    PRC,
}

impl ThreeDFormat {
    /// Get the PDF subtype name.
    pub fn subtype(&self) -> &'static str {
        match self {
            ThreeDFormat::U3D => "U3D",
            ThreeDFormat::PRC => "PRC",
        }
    }
}

/// 3D lighting scheme.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDLighting {
    /// Artwork-defined lighting
    Artwork,
    /// No lighting
    None,
    /// White lights
    #[default]
    White,
    /// Day lighting
    Day,
    /// Night lighting
    Night,
    /// Hard lighting
    Hard,
    /// Primary lighting
    Primary,
    /// Blue lighting
    Blue,
    /// Red lighting
    Red,
    /// Cube lighting
    Cube,
    /// CAD-optimized lighting
    CAD,
    /// Headlamp lighting
    Headlamp,
}

impl ThreeDLighting {
    /// Get the PDF name for this lighting scheme.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ThreeDLighting::Artwork => "Artwork",
            ThreeDLighting::None => "None",
            ThreeDLighting::White => "White",
            ThreeDLighting::Day => "Day",
            ThreeDLighting::Night => "Night",
            ThreeDLighting::Hard => "Hard",
            ThreeDLighting::Primary => "Primary",
            ThreeDLighting::Blue => "Blue",
            ThreeDLighting::Red => "Red",
            ThreeDLighting::Cube => "Cube",
            ThreeDLighting::CAD => "CAD",
            ThreeDLighting::Headlamp => "Headlamp",
        }
    }
}

/// 3D render mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDRenderMode {
    /// Solid rendering
    #[default]
    Solid,
    /// Solid wireframe
    SolidWireframe,
    /// Transparent
    Transparent,
    /// Transparent wireframe
    TransparentWireframe,
    /// Bounding box
    BoundingBox,
    /// Transparent bounding box
    TransparentBoundingBox,
    /// Transparent bounding box outline
    TransparentBoundingBoxOutline,
    /// Wireframe
    Wireframe,
    /// Shaded wireframe
    ShadedWireframe,
    /// Hidden wireframe
    HiddenWireframe,
    /// Vertices
    Vertices,
    /// Shaded vertices
    ShadedVertices,
    /// Illustration
    Illustration,
    /// Solid outline
    SolidOutline,
    /// Shaded illustration
    ShadedIllustration,
}

impl ThreeDRenderMode {
    /// Get the PDF name for this render mode.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ThreeDRenderMode::Solid => "Solid",
            ThreeDRenderMode::SolidWireframe => "SolidWireframe",
            ThreeDRenderMode::Transparent => "Transparent",
            ThreeDRenderMode::TransparentWireframe => "TransparentWireframe",
            ThreeDRenderMode::BoundingBox => "BoundingBox",
            ThreeDRenderMode::TransparentBoundingBox => "TransparentBoundingBox",
            ThreeDRenderMode::TransparentBoundingBoxOutline => "TransparentBoundingBoxOutline",
            ThreeDRenderMode::Wireframe => "Wireframe",
            ThreeDRenderMode::ShadedWireframe => "ShadedWireframe",
            ThreeDRenderMode::HiddenWireframe => "HiddenWireframe",
            ThreeDRenderMode::Vertices => "Vertices",
            ThreeDRenderMode::ShadedVertices => "ShadedVertices",
            ThreeDRenderMode::Illustration => "Illustration",
            ThreeDRenderMode::SolidOutline => "SolidOutline",
            ThreeDRenderMode::ShadedIllustration => "ShadedIllustration",
        }
    }
}

/// 3D projection type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDProjection {
    /// Orthographic projection
    Orthographic,
    /// Perspective projection (default)
    #[default]
    Perspective,
}

impl ThreeDProjection {
    /// Get the PDF name for this projection type.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ThreeDProjection::Orthographic => "Orthographic",
            ThreeDProjection::Perspective => "Perspective",
        }
    }
}

/// 3D activation mode - when to activate the 3D content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDActivation {
    /// Activate when page is visible
    #[default]
    PageVisible,
    /// Activate when page is opened
    PageOpen,
    /// Activate explicitly (user click)
    Explicit,
}

impl ThreeDActivation {
    /// Get the PDF name for this activation mode.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ThreeDActivation::PageVisible => "PV",
            ThreeDActivation::PageOpen => "PO",
            ThreeDActivation::Explicit => "XA",
        }
    }
}

/// 3D deactivation mode - when to deactivate the 3D content.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThreeDDeactivation {
    /// Deactivate when page is not visible
    #[default]
    PageNotVisible,
    /// Deactivate when page is closed
    PageClose,
    /// Deactivate explicitly
    Explicit,
}

impl ThreeDDeactivation {
    /// Get the PDF name for this deactivation mode.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            ThreeDDeactivation::PageNotVisible => "PI",
            ThreeDDeactivation::PageClose => "PC",
            ThreeDDeactivation::Explicit => "XD",
        }
    }
}

/// 3D background settings.
#[derive(Debug, Clone)]
pub struct ThreeDBackground {
    /// Background color RGB (0.0-1.0 each)
    pub color: (f32, f32, f32),
}

impl Default for ThreeDBackground {
    fn default() -> Self {
        Self {
            color: (1.0, 1.0, 1.0), // White background
        }
    }
}

impl ThreeDBackground {
    /// Create a new background with the specified color.
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)),
        }
    }

    /// Create a white background.
    pub fn white() -> Self {
        Self::default()
    }

    /// Create a black background.
    pub fn black() -> Self {
        Self {
            color: (0.0, 0.0, 0.0),
        }
    }

    /// Create a gray background.
    pub fn gray(level: f32) -> Self {
        let l = level.clamp(0.0, 1.0);
        Self { color: (l, l, l) }
    }

    /// Build the background dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("3DBG".to_string()));

        // CS - color space (DeviceRGB)
        dict.insert("CS".to_string(), Object::Name("DeviceRGB".to_string()));

        // C - color
        dict.insert(
            "C".to_string(),
            Object::Array(vec![
                Object::Real(self.color.0 as f64),
                Object::Real(self.color.1 as f64),
                Object::Real(self.color.2 as f64),
            ]),
        );

        dict
    }
}

/// Camera position for 3D view.
#[derive(Debug, Clone)]
pub struct ThreeDCamera {
    /// Camera-to-origin distance (orbit radius)
    pub distance: f32,
    /// Camera orbit angle (azimuth, degrees)
    pub azimuth: f32,
    /// Camera elevation angle (pitch, degrees)
    pub elevation: f32,
    /// Center of orbit (x, y, z)
    pub center: (f32, f32, f32),
    /// Field of view angle (degrees, for perspective)
    pub fov: f32,
    /// Roll angle (degrees)
    pub roll: f32,
}

impl Default for ThreeDCamera {
    fn default() -> Self {
        Self {
            distance: 100.0,
            azimuth: 45.0,
            elevation: 30.0,
            center: (0.0, 0.0, 0.0),
            fov: 45.0,
            roll: 0.0,
        }
    }
}

impl ThreeDCamera {
    /// Create a new camera with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set camera distance from the origin.
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = distance;
        self
    }

    /// Set camera azimuth (horizontal orbit angle).
    pub fn with_azimuth(mut self, degrees: f32) -> Self {
        self.azimuth = degrees;
        self
    }

    /// Set camera elevation (vertical angle).
    pub fn with_elevation(mut self, degrees: f32) -> Self {
        self.elevation = degrees;
        self
    }

    /// Set orbit center point.
    pub fn with_center(mut self, x: f32, y: f32, z: f32) -> Self {
        self.center = (x, y, z);
        self
    }

    /// Set field of view.
    pub fn with_fov(mut self, degrees: f32) -> Self {
        self.fov = degrees;
        self
    }

    /// Set roll angle.
    pub fn with_roll(mut self, degrees: f32) -> Self {
        self.roll = degrees;
        self
    }

    /// Calculate the camera-to-world matrix.
    /// Returns the matrix as 12 values (3x4 matrix without perspective row).
    pub fn calculate_matrix(&self) -> Vec<f64> {
        // Convert angles to radians
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        let rl = self.roll.to_radians();

        // Calculate camera position on sphere
        let cos_az = az.cos();
        let sin_az = az.sin();
        let cos_el = el.cos();
        let sin_el = el.sin();
        let cos_rl = rl.cos();
        let sin_rl = rl.sin();

        // Camera position
        let cam_x = self.distance * cos_el * sin_az + self.center.0;
        let cam_y = self.distance * cos_el * cos_az + self.center.1;
        let cam_z = self.distance * sin_el + self.center.2;

        // Look direction (pointing at center)
        let look_x = self.center.0 - cam_x;
        let look_y = self.center.1 - cam_y;
        let look_z = self.center.2 - cam_z;

        // Normalize
        let look_len = (look_x * look_x + look_y * look_y + look_z * look_z).sqrt();
        let zx = -look_x / look_len;
        let zy = -look_y / look_len;
        let zz = -look_z / look_len;

        // Right vector (cross product with up)
        let up_x = 0.0;
        let up_y = 0.0;
        let up_z = 1.0;

        let rx = up_y * zz - up_z * zy;
        let ry = up_z * zx - up_x * zz;
        let rz = up_x * zy - up_y * zx;

        let right_len = (rx * rx + ry * ry + rz * rz).sqrt();
        let xx = rx / right_len;
        let xy = ry / right_len;
        let xz = rz / right_len;

        // Up vector (cross product)
        let ux = zy * xz - zz * xy;
        let uy = zz * xx - zx * xz;
        let uz = zx * xy - zy * xx;

        // Apply roll rotation
        let yx = ux * cos_rl - xx * sin_rl;
        let yy = uy * cos_rl - xy * sin_rl;
        let yz = uz * cos_rl - xz * sin_rl;

        let xx_r = xx * cos_rl + ux * sin_rl;
        let xy_r = xy * cos_rl + uy * sin_rl;
        let xz_r = xz * cos_rl + uz * sin_rl;

        // Camera-to-world matrix (3x4)
        vec![
            xx_r as f64,
            xy_r as f64,
            xz_r as f64,
            yx as f64,
            yy as f64,
            yz as f64,
            zx as f64,
            zy as f64,
            zz as f64,
            cam_x as f64,
            cam_y as f64,
            cam_z as f64,
        ]
    }
}

/// 3D view definition.
///
/// Per PDF spec Section 13.6.4, defines a specific view of the 3D artwork.
#[derive(Debug, Clone, Default)]
pub struct ThreeDView {
    /// View name
    pub name: Option<String>,
    /// Camera position and orientation
    pub camera: ThreeDCamera,
    /// Projection type
    pub projection: ThreeDProjection,
    /// Render mode
    pub render_mode: ThreeDRenderMode,
    /// Lighting scheme
    pub lighting: ThreeDLighting,
    /// Background settings
    pub background: Option<ThreeDBackground>,
}

impl ThreeDView {
    /// Create a new view with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a default isometric view.
    pub fn default_view() -> Self {
        Self {
            name: Some("Default".to_string()),
            camera: ThreeDCamera::default()
                .with_azimuth(45.0)
                .with_elevation(30.0),
            projection: ThreeDProjection::Perspective,
            render_mode: ThreeDRenderMode::Solid,
            lighting: ThreeDLighting::White,
            background: None,
        }
    }

    /// Create a front view.
    pub fn front() -> Self {
        Self {
            name: Some("Front".to_string()),
            camera: ThreeDCamera::default()
                .with_azimuth(0.0)
                .with_elevation(0.0),
            ..Default::default()
        }
    }

    /// Create a top view.
    pub fn top() -> Self {
        Self {
            name: Some("Top".to_string()),
            camera: ThreeDCamera::default()
                .with_azimuth(0.0)
                .with_elevation(90.0),
            ..Default::default()
        }
    }

    /// Create a right side view.
    pub fn right() -> Self {
        Self {
            name: Some("Right".to_string()),
            camera: ThreeDCamera::default()
                .with_azimuth(90.0)
                .with_elevation(0.0),
            ..Default::default()
        }
    }

    /// Set the view name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set camera distance.
    pub fn with_camera_distance(mut self, distance: f32) -> Self {
        self.camera.distance = distance;
        self
    }

    /// Set camera position.
    pub fn with_camera(mut self, camera: ThreeDCamera) -> Self {
        self.camera = camera;
        self
    }

    /// Set projection type.
    pub fn with_projection(mut self, projection: ThreeDProjection) -> Self {
        self.projection = projection;
        self
    }

    /// Set render mode.
    pub fn with_render_mode(mut self, mode: ThreeDRenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Set lighting scheme.
    pub fn with_lighting(mut self, lighting: ThreeDLighting) -> Self {
        self.lighting = lighting;
        self
    }

    /// Set background.
    pub fn with_background(mut self, background: ThreeDBackground) -> Self {
        self.background = Some(background);
        self
    }

    /// Build the 3DView dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("3DView".to_string()));

        // XN - external name (required)
        if let Some(ref name) = self.name {
            dict.insert("XN".to_string(), Object::text_string(name));
        } else {
            dict.insert("XN".to_string(), Object::text_string("Default"));
        }

        // IN - internal name (same as external)
        if let Some(ref name) = self.name {
            dict.insert("IN".to_string(), Object::text_string(name));
        }

        // C2W - camera-to-world transformation matrix
        let matrix = self.camera.calculate_matrix();
        dict.insert(
            "C2W".to_string(),
            Object::Array(matrix.into_iter().map(Object::Real).collect()),
        );

        // CO - camera orbit radius
        dict.insert("CO".to_string(), Object::Real(self.camera.distance as f64));

        // P - projection dictionary
        let mut proj = HashMap::new();
        proj.insert("Subtype".to_string(), Object::Name(self.projection.pdf_name().to_string()));
        if matches!(self.projection, ThreeDProjection::Perspective) {
            proj.insert("FOV".to_string(), Object::Real(self.camera.fov as f64));
        }
        dict.insert("P".to_string(), Object::Dictionary(proj));

        // RM - render mode dictionary
        let mut rm = HashMap::new();
        rm.insert("Type".to_string(), Object::Name("3DRenderMode".to_string()));
        rm.insert("Subtype".to_string(), Object::Name(self.render_mode.pdf_name().to_string()));
        dict.insert("RM".to_string(), Object::Dictionary(rm));

        // LS - lighting scheme dictionary
        let mut ls = HashMap::new();
        ls.insert("Type".to_string(), Object::Name("3DLightingScheme".to_string()));
        ls.insert("Subtype".to_string(), Object::Name(self.lighting.pdf_name().to_string()));
        dict.insert("LS".to_string(), Object::Dictionary(ls));

        // BG - background
        if let Some(ref bg) = self.background {
            dict.insert("BG".to_string(), Object::Dictionary(bg.build()));
        }

        dict
    }
}

/// 3D stream containing the model data.
///
/// Per PDF spec Section 13.6.3, this is the stream containing the 3D artwork.
#[derive(Debug, Clone)]
pub struct ThreeDStream {
    /// Raw 3D model data
    pub data: Vec<u8>,
    /// Model format
    pub format: ThreeDFormat,
    /// Optional animation style name
    pub animation_style: Option<String>,
}

impl ThreeDStream {
    /// Create a new 3D stream with U3D format.
    pub fn u3d(data: Vec<u8>) -> Self {
        Self {
            data,
            format: ThreeDFormat::U3D,
            animation_style: None,
        }
    }

    /// Create a new 3D stream with PRC format.
    pub fn prc(data: Vec<u8>) -> Self {
        Self {
            data,
            format: ThreeDFormat::PRC,
            animation_style: None,
        }
    }

    /// Create a new 3D stream with specified format.
    pub fn new(data: Vec<u8>, format: ThreeDFormat) -> Self {
        Self {
            data,
            format,
            animation_style: None,
        }
    }

    /// Set animation style.
    pub fn with_animation_style(mut self, style: impl Into<String>) -> Self {
        self.animation_style = Some(style.into());
        self
    }

    /// Build the 3D stream dictionary.
    ///
    /// The data should be written as a stream with these dictionary entries.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("3D".to_string()));
        dict.insert("Subtype".to_string(), Object::Name(self.format.subtype().to_string()));

        if let Some(ref anim) = self.animation_style {
            let mut anim_dict = HashMap::new();
            anim_dict.insert("Subtype".to_string(), Object::Name(anim.clone()));
            dict.insert("AN".to_string(), Object::Dictionary(anim_dict));
        }

        dict
    }

    /// Get the raw data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// 3D annotation for embedding 3D models.
///
/// Per PDF spec Section 12.5.6.24.
#[derive(Debug, Clone)]
pub struct ThreeDAnnotation {
    /// Bounding rectangle for the 3D display area
    pub rect: Rect,
    /// 3D model stream
    pub stream: ThreeDStream,
    /// Default view
    pub default_view: ThreeDView,
    /// Additional views
    pub views: Vec<ThreeDView>,
    /// Activation settings
    pub activation: ThreeDActivation,
    /// Deactivation settings
    pub deactivation: ThreeDDeactivation,
    /// Annotation title
    pub title: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Contents/description
    pub contents: Option<String>,
    /// Interactive mode enabled
    pub interactive: bool,
    /// Show toolbar
    pub toolbar: bool,
    /// Show navigation panel
    pub nav_panel: bool,
}

impl ThreeDAnnotation {
    /// Create a new 3D annotation.
    pub fn new(rect: Rect, stream: ThreeDStream) -> Self {
        Self {
            rect,
            stream,
            default_view: ThreeDView::default_view(),
            views: Vec::new(),
            activation: ThreeDActivation::default(),
            deactivation: ThreeDDeactivation::default(),
            title: None,
            flags: AnnotationFlags::printable(),
            contents: None,
            interactive: true,
            toolbar: true,
            nav_panel: false,
        }
    }

    /// Create a 3D annotation with U3D model.
    pub fn u3d(rect: Rect, data: Vec<u8>) -> Self {
        Self::new(rect, ThreeDStream::u3d(data))
    }

    /// Create a 3D annotation with PRC model.
    pub fn prc(rect: Rect, data: Vec<u8>) -> Self {
        Self::new(rect, ThreeDStream::prc(data))
    }

    /// Set the default view.
    pub fn with_view(mut self, view: ThreeDView) -> Self {
        self.default_view = view;
        self
    }

    /// Add an additional view.
    pub fn add_view(mut self, view: ThreeDView) -> Self {
        self.views.push(view);
        self
    }

    /// Set camera distance.
    pub fn with_camera_distance(mut self, distance: f32) -> Self {
        self.default_view.camera.distance = distance;
        self
    }

    /// Set render mode.
    pub fn with_render_mode(mut self, mode: ThreeDRenderMode) -> Self {
        self.default_view.render_mode = mode;
        self
    }

    /// Set lighting scheme.
    pub fn with_lighting(mut self, lighting: ThreeDLighting) -> Self {
        self.default_view.lighting = lighting;
        self
    }

    /// Set background.
    pub fn with_background(mut self, background: ThreeDBackground) -> Self {
        self.default_view.background = Some(background);
        self
    }

    /// Set activation mode.
    pub fn with_activation(mut self, activation: ThreeDActivation) -> Self {
        self.activation = activation;
        self
    }

    /// Set deactivation mode.
    pub fn with_deactivation(mut self, deactivation: ThreeDDeactivation) -> Self {
        self.deactivation = deactivation;
        self
    }

    /// Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set description/contents.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set interactive mode.
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set toolbar visibility.
    pub fn with_toolbar(mut self, toolbar: bool) -> Self {
        self.toolbar = toolbar;
        self
    }

    /// Set navigation panel visibility.
    pub fn with_nav_panel(mut self, nav_panel: bool) -> Self {
        self.nav_panel = nav_panel;
        self
    }

    /// Build the annotation dictionary.
    ///
    /// Note: The 3D stream must be written separately and referenced.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("3D".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Title
        if let Some(ref title) = self.title {
            dict.insert("T".to_string(), Object::text_string(title));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // 3DA - 3D Activation dictionary
        let mut activation = HashMap::new();

        // A - activation mode
        activation.insert("A".to_string(), Object::Name(self.activation.pdf_name().to_string()));

        // D - deactivation mode
        activation.insert("D".to_string(), Object::Name(self.deactivation.pdf_name().to_string()));

        // AIS - activation state (I = Interactive, L = Live)
        if self.interactive {
            activation.insert("AIS".to_string(), Object::Name("I".to_string()));
        } else {
            activation.insert("AIS".to_string(), Object::Name("L".to_string()));
        }

        // DIS - deactivation state
        activation.insert("DIS".to_string(), Object::Name("U".to_string())); // Uninstantiated

        // TB - toolbar
        activation.insert("TB".to_string(), Object::Boolean(self.toolbar));

        // NP - navigation panel
        activation.insert("NP".to_string(), Object::Boolean(self.nav_panel));

        dict.insert("3DA".to_string(), Object::Dictionary(activation));

        // 3DV - default view (will be set when view is written)
        // Note: This should be set to a reference to the view object
        // For now, include the view inline
        dict.insert("3DV".to_string(), Object::Dictionary(self.default_view.build()));

        // Note: 3DD (3D stream reference) must be added by the caller

        dict
    }

    /// Build the 3D stream dictionary and data.
    pub fn build_stream(&self) -> (HashMap<String, Object>, Vec<u8>) {
        (self.stream.build(), self.stream.data.clone())
    }

    /// Get the 3D stream reference.
    pub fn stream(&self) -> &ThreeDStream {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threed_format() {
        assert_eq!(ThreeDFormat::U3D.subtype(), "U3D");
        assert_eq!(ThreeDFormat::PRC.subtype(), "PRC");
    }

    #[test]
    fn test_threed_lighting() {
        assert_eq!(ThreeDLighting::White.pdf_name(), "White");
        assert_eq!(ThreeDLighting::CAD.pdf_name(), "CAD");
        assert_eq!(ThreeDLighting::Headlamp.pdf_name(), "Headlamp");
    }

    #[test]
    fn test_threed_render_mode() {
        assert_eq!(ThreeDRenderMode::Solid.pdf_name(), "Solid");
        assert_eq!(ThreeDRenderMode::Wireframe.pdf_name(), "Wireframe");
        assert_eq!(ThreeDRenderMode::Illustration.pdf_name(), "Illustration");
    }

    #[test]
    fn test_threed_projection() {
        assert_eq!(ThreeDProjection::Perspective.pdf_name(), "Perspective");
        assert_eq!(ThreeDProjection::Orthographic.pdf_name(), "Orthographic");
    }

    #[test]
    fn test_threed_activation() {
        assert_eq!(ThreeDActivation::PageVisible.pdf_name(), "PV");
        assert_eq!(ThreeDActivation::PageOpen.pdf_name(), "PO");
        assert_eq!(ThreeDActivation::Explicit.pdf_name(), "XA");
    }

    #[test]
    fn test_threed_deactivation() {
        assert_eq!(ThreeDDeactivation::PageNotVisible.pdf_name(), "PI");
        assert_eq!(ThreeDDeactivation::PageClose.pdf_name(), "PC");
        assert_eq!(ThreeDDeactivation::Explicit.pdf_name(), "XD");
    }

    #[test]
    fn test_threed_background() {
        let bg = ThreeDBackground::new(0.5, 0.5, 0.5);
        assert_eq!(bg.color, (0.5, 0.5, 0.5));

        let dict = bg.build();
        assert_eq!(dict.get("Type"), Some(&Object::Name("3DBG".to_string())));
    }

    #[test]
    fn test_threed_camera_default() {
        let camera = ThreeDCamera::default();
        assert_eq!(camera.distance, 100.0);
        assert_eq!(camera.azimuth, 45.0);
        assert_eq!(camera.elevation, 30.0);
    }

    #[test]
    fn test_threed_camera_builder() {
        let camera = ThreeDCamera::new()
            .with_distance(200.0)
            .with_azimuth(90.0)
            .with_elevation(45.0)
            .with_center(10.0, 20.0, 30.0);

        assert_eq!(camera.distance, 200.0);
        assert_eq!(camera.azimuth, 90.0);
        assert_eq!(camera.elevation, 45.0);
        assert_eq!(camera.center, (10.0, 20.0, 30.0));
    }

    #[test]
    fn test_threed_camera_matrix() {
        let camera = ThreeDCamera::default();
        let matrix = camera.calculate_matrix();
        assert_eq!(matrix.len(), 12);
    }

    #[test]
    fn test_threed_view_default() {
        let view = ThreeDView::default_view();
        assert_eq!(view.name, Some("Default".to_string()));
        assert!(matches!(view.projection, ThreeDProjection::Perspective));
        assert!(matches!(view.render_mode, ThreeDRenderMode::Solid));
    }

    #[test]
    fn test_threed_view_presets() {
        let front = ThreeDView::front();
        assert_eq!(front.name, Some("Front".to_string()));
        assert_eq!(front.camera.azimuth, 0.0);

        let top = ThreeDView::top();
        assert_eq!(top.name, Some("Top".to_string()));
        assert_eq!(top.camera.elevation, 90.0);

        let right = ThreeDView::right();
        assert_eq!(right.name, Some("Right".to_string()));
        assert_eq!(right.camera.azimuth, 90.0);
    }

    #[test]
    fn test_threed_view_builder() {
        let view = ThreeDView::new()
            .with_name("Custom View")
            .with_camera_distance(150.0)
            .with_render_mode(ThreeDRenderMode::Wireframe)
            .with_lighting(ThreeDLighting::CAD)
            .with_background(ThreeDBackground::gray(0.8));

        assert_eq!(view.name, Some("Custom View".to_string()));
        assert_eq!(view.camera.distance, 150.0);
        assert!(matches!(view.render_mode, ThreeDRenderMode::Wireframe));
        assert!(matches!(view.lighting, ThreeDLighting::CAD));
        assert!(view.background.is_some());
    }

    #[test]
    fn test_threed_view_build() {
        let view = ThreeDView::default_view();
        let dict = view.build();

        assert_eq!(dict.get("Type"), Some(&Object::Name("3DView".to_string())));
        assert!(dict.contains_key("XN"));
        assert!(dict.contains_key("C2W"));
        assert!(dict.contains_key("P"));
        assert!(dict.contains_key("RM"));
        assert!(dict.contains_key("LS"));
    }

    #[test]
    fn test_threed_stream_u3d() {
        let data = vec![0u8; 1000];
        let stream = ThreeDStream::u3d(data.clone());

        assert_eq!(stream.data, data);
        assert!(matches!(stream.format, ThreeDFormat::U3D));
    }

    #[test]
    fn test_threed_stream_prc() {
        let data = vec![0u8; 500];
        let stream = ThreeDStream::prc(data.clone());

        assert_eq!(stream.data, data);
        assert!(matches!(stream.format, ThreeDFormat::PRC));
    }

    #[test]
    fn test_threed_stream_build() {
        let stream = ThreeDStream::u3d(vec![]);
        let dict = stream.build();

        assert_eq!(dict.get("Type"), Some(&Object::Name("3D".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("U3D".to_string())));
    }

    #[test]
    fn test_threed_annotation_new() {
        let rect = Rect::new(72.0, 400.0, 400.0, 300.0);
        let annot = ThreeDAnnotation::u3d(rect, vec![0u8; 100]);

        assert_eq!(annot.rect.x, 72.0);
        assert_eq!(annot.rect.width, 400.0);
        assert!(annot.interactive);
        assert!(annot.toolbar);
    }

    #[test]
    fn test_threed_annotation_builder() {
        let rect = Rect::new(72.0, 400.0, 400.0, 300.0);
        let annot = ThreeDAnnotation::u3d(rect, vec![])
            .with_title("3D Model")
            .with_camera_distance(200.0)
            .with_render_mode(ThreeDRenderMode::SolidWireframe)
            .with_lighting(ThreeDLighting::CAD)
            .with_background(ThreeDBackground::gray(0.9))
            .with_activation(ThreeDActivation::Explicit)
            .with_toolbar(true)
            .with_nav_panel(true);

        assert_eq!(annot.title, Some("3D Model".to_string()));
        assert_eq!(annot.default_view.camera.distance, 200.0);
        assert!(matches!(annot.default_view.render_mode, ThreeDRenderMode::SolidWireframe));
        assert!(matches!(annot.activation, ThreeDActivation::Explicit));
        assert!(annot.toolbar);
        assert!(annot.nav_panel);
    }

    #[test]
    fn test_threed_annotation_add_views() {
        let rect = Rect::new(72.0, 400.0, 400.0, 300.0);
        let annot = ThreeDAnnotation::u3d(rect, vec![])
            .add_view(ThreeDView::front())
            .add_view(ThreeDView::top())
            .add_view(ThreeDView::right());

        assert_eq!(annot.views.len(), 3);
    }

    #[test]
    fn test_threed_annotation_build() {
        let rect = Rect::new(72.0, 400.0, 400.0, 300.0);
        let annot = ThreeDAnnotation::u3d(rect, vec![1, 2, 3])
            .with_title("Test Model")
            .with_contents("A 3D model");

        let dict = annot.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("3D".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("T")); // Title
        assert!(dict.contains_key("3DA")); // Activation
        assert!(dict.contains_key("3DV")); // Default view
    }
}
