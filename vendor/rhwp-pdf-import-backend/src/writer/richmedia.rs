//! RichMedia annotations for PDF generation.
//!
//! This module provides support for RichMedia annotations per PDF spec
//! Adobe Supplement to ISO 32000-1 (Extension Level 3).
//!
//! RichMedia annotations embed interactive content like Flash (SWF) and
//! video players with advanced controls. Note that Flash support is
//! deprecated in modern PDF viewers.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{RichMediaAnnotation, RichMediaContent, RichMediaAsset};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a RichMedia annotation with video
//! let video_asset = RichMediaAsset::new("video.mp4", video_data)
//!     .with_mime_type("video/mp4");
//!
//! let content = RichMediaContent::new()
//!     .add_asset(video_asset);
//!
//! let annot = RichMediaAnnotation::new(
//!     Rect::new(72.0, 400.0, 400.0, 300.0),
//!     content,
//! );
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// RichMedia activation condition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RichMediaActivation {
    /// Activate explicitly (user interaction)
    #[default]
    Explicit,
    /// Activate when page is visible
    PageVisible,
    /// Activate when page is opened
    PageOpen,
}

impl RichMediaActivation {
    /// Get the PDF name for this activation.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            RichMediaActivation::Explicit => "XA",
            RichMediaActivation::PageVisible => "PV",
            RichMediaActivation::PageOpen => "PO",
        }
    }
}

/// RichMedia deactivation condition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RichMediaDeactivation {
    /// Deactivate explicitly
    Explicit,
    /// Deactivate when page is not visible
    #[default]
    PageNotVisible,
    /// Deactivate when page is closed
    PageClose,
}

impl RichMediaDeactivation {
    /// Get the PDF name for this deactivation.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            RichMediaDeactivation::Explicit => "XD",
            RichMediaDeactivation::PageNotVisible => "PI",
            RichMediaDeactivation::PageClose => "PC",
        }
    }
}

/// RichMedia window type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RichMediaWindow {
    /// Embedded in annotation rectangle
    #[default]
    Embedded,
    /// Floating window
    Floating,
    /// Full screen
    FullScreen,
}

impl RichMediaWindow {
    /// Get the PDF name for this window type.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            RichMediaWindow::Embedded => "Embedded",
            RichMediaWindow::Floating => "Windowed",
            RichMediaWindow::FullScreen => "Fullscreen",
        }
    }
}

/// An asset in RichMedia content.
#[derive(Debug, Clone)]
pub struct RichMediaAsset {
    /// Asset name (filename)
    pub name: String,
    /// Raw asset data
    pub data: Vec<u8>,
    /// MIME type
    pub mime_type: Option<String>,
}

impl RichMediaAsset {
    /// Create a new asset.
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            mime_type: None,
        }
    }

    /// Create a video asset.
    pub fn video(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            mime_type: Some("video/mp4".to_string()),
        }
    }

    /// Create an SWF (Flash) asset.
    pub fn swf(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            mime_type: Some("application/x-shockwave-flash".to_string()),
        }
    }

    /// Set the MIME type.
    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    /// Build the asset name tree entry.
    pub fn build(&self, file_spec_ref: Option<ObjectRef>) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Name
        dict.insert("Name".to_string(), Object::text_string(&self.name));

        // EF - embedded file reference (if provided)
        if let Some(ref_obj) = file_spec_ref {
            let mut ef = HashMap::new();
            ef.insert("F".to_string(), Object::Reference(ref_obj));
            dict.insert("EF".to_string(), Object::Dictionary(ef));
        }

        // Mime type
        if let Some(ref mime) = self.mime_type {
            dict.insert("Subtype".to_string(), Object::text_string(mime));
        }

        dict
    }
}

/// RichMedia content containing assets and configurations.
#[derive(Debug, Clone)]
pub struct RichMediaContent {
    /// Assets (embedded files)
    pub assets: Vec<RichMediaAsset>,
    /// Configuration name
    pub configuration_name: Option<String>,
}

impl Default for RichMediaContent {
    fn default() -> Self {
        Self::new()
    }
}

impl RichMediaContent {
    /// Create new empty content.
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            configuration_name: None,
        }
    }

    /// Add an asset.
    pub fn add_asset(mut self, asset: RichMediaAsset) -> Self {
        self.assets.push(asset);
        self
    }

    /// Set configuration name.
    pub fn with_configuration(mut self, name: impl Into<String>) -> Self {
        self.configuration_name = Some(name.into());
        self
    }

    /// Build the RichMediaContent dictionary.
    ///
    /// Note: Assets must be written separately and referenced.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("RichMediaContent".to_string()));

        // Assets - name tree (will need asset references added by caller)
        // This is a placeholder structure
        let mut assets = HashMap::new();
        let names: Vec<Object> = self
            .assets
            .iter()
            .flat_map(|a| {
                vec![
                    Object::text_string(&a.name),
                    Object::Dictionary(a.build(None)),
                ]
            })
            .collect();
        if !names.is_empty() {
            assets.insert("Names".to_string(), Object::Array(names));
        }
        dict.insert("Assets".to_string(), Object::Dictionary(assets));

        // Configurations - array of configuration dictionaries
        if self.configuration_name.is_some() {
            let mut config = HashMap::new();
            config.insert("Type".to_string(), Object::Name("RichMediaConfiguration".to_string()));
            config.insert("Subtype".to_string(), Object::Name("Video".to_string()));
            if let Some(ref name) = self.configuration_name {
                config.insert("Name".to_string(), Object::text_string(name));
            }
            dict.insert(
                "Configurations".to_string(),
                Object::Array(vec![Object::Dictionary(config)]),
            );
        }

        dict
    }
}

/// RichMedia settings for playback.
#[derive(Debug, Clone)]
pub struct RichMediaSettings {
    /// Activation condition
    pub activation: RichMediaActivation,
    /// Deactivation condition
    pub deactivation: RichMediaDeactivation,
    /// Window type
    pub window: RichMediaWindow,
    /// Show toolbar
    pub toolbar: bool,
    /// Transparent background
    pub transparent: bool,
}

impl Default for RichMediaSettings {
    fn default() -> Self {
        Self {
            activation: RichMediaActivation::default(),
            deactivation: RichMediaDeactivation::default(),
            window: RichMediaWindow::default(),
            toolbar: true,
            transparent: false,
        }
    }
}

impl RichMediaSettings {
    /// Create new settings with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set activation condition.
    pub fn with_activation(mut self, activation: RichMediaActivation) -> Self {
        self.activation = activation;
        self
    }

    /// Set deactivation condition.
    pub fn with_deactivation(mut self, deactivation: RichMediaDeactivation) -> Self {
        self.deactivation = deactivation;
        self
    }

    /// Set window type.
    pub fn with_window(mut self, window: RichMediaWindow) -> Self {
        self.window = window;
        self
    }

    /// Set toolbar visibility.
    pub fn with_toolbar(mut self, toolbar: bool) -> Self {
        self.toolbar = toolbar;
        self
    }

    /// Set transparent background.
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Build the RichMediaSettings dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("RichMediaSettings".to_string()));

        // Activation dictionary
        let mut activation = HashMap::new();
        activation.insert("Type".to_string(), Object::Name("RichMediaActivation".to_string()));
        activation
            .insert("Condition".to_string(), Object::Name(self.activation.pdf_name().to_string()));

        // Presentation dictionary
        let mut presentation = HashMap::new();
        presentation.insert("Type".to_string(), Object::Name("RichMediaPresentation".to_string()));
        presentation.insert("Style".to_string(), Object::Name(self.window.pdf_name().to_string()));
        presentation.insert("Toolbar".to_string(), Object::Boolean(self.toolbar));
        presentation.insert("Transparent".to_string(), Object::Boolean(self.transparent));

        // Window (for floating windows)
        if matches!(self.window, RichMediaWindow::Floating) {
            let mut window = HashMap::new();
            window.insert("Type".to_string(), Object::Name("RichMediaWindow".to_string()));
            // Default size
            let mut width = HashMap::new();
            width.insert("Default".to_string(), Object::Real(400.0));
            width.insert("Min".to_string(), Object::Real(200.0));
            width.insert("Max".to_string(), Object::Real(800.0));
            window.insert("Width".to_string(), Object::Dictionary(width));

            let mut height = HashMap::new();
            height.insert("Default".to_string(), Object::Real(300.0));
            height.insert("Min".to_string(), Object::Real(150.0));
            height.insert("Max".to_string(), Object::Real(600.0));
            window.insert("Height".to_string(), Object::Dictionary(height));

            presentation.insert("Window".to_string(), Object::Dictionary(window));
        }

        activation.insert("Presentation".to_string(), Object::Dictionary(presentation));
        dict.insert("Activation".to_string(), Object::Dictionary(activation));

        // Deactivation dictionary
        let mut deactivation = HashMap::new();
        deactivation.insert("Type".to_string(), Object::Name("RichMediaDeactivation".to_string()));
        deactivation.insert(
            "Condition".to_string(),
            Object::Name(self.deactivation.pdf_name().to_string()),
        );
        dict.insert("Deactivation".to_string(), Object::Dictionary(deactivation));

        dict
    }
}

/// A RichMedia annotation for interactive content.
///
/// Per Adobe Supplement to ISO 32000-1 (Extension Level 3).
/// Note: Flash support is deprecated in modern viewers.
#[derive(Debug, Clone)]
pub struct RichMediaAnnotation {
    /// Bounding rectangle
    pub rect: Rect,
    /// Content (assets and configurations)
    pub content: RichMediaContent,
    /// Settings (activation, presentation)
    pub settings: RichMediaSettings,
    /// Annotation title
    pub title: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Contents/description
    pub contents: Option<String>,
}

impl RichMediaAnnotation {
    /// Create a new RichMedia annotation.
    pub fn new(rect: Rect, content: RichMediaContent) -> Self {
        Self {
            rect,
            content,
            settings: RichMediaSettings::default(),
            title: None,
            flags: AnnotationFlags::printable(),
            contents: None,
        }
    }

    /// Create a RichMedia annotation with a single video asset.
    pub fn video(rect: Rect, name: impl Into<String>, data: Vec<u8>) -> Self {
        let content = RichMediaContent::new().add_asset(RichMediaAsset::video(name, data));
        Self::new(rect, content)
    }

    /// Set the settings.
    pub fn with_settings(mut self, settings: RichMediaSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Set activation condition.
    pub fn with_activation(mut self, activation: RichMediaActivation) -> Self {
        self.settings.activation = activation;
        self
    }

    /// Set deactivation condition.
    pub fn with_deactivation(mut self, deactivation: RichMediaDeactivation) -> Self {
        self.settings.deactivation = deactivation;
        self
    }

    /// Set window type.
    pub fn with_window(mut self, window: RichMediaWindow) -> Self {
        self.settings.window = window;
        self
    }

    /// Set toolbar visibility.
    pub fn with_toolbar(mut self, toolbar: bool) -> Self {
        self.settings.toolbar = toolbar;
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

    /// Add an asset to the content.
    pub fn add_asset(mut self, asset: RichMediaAsset) -> Self {
        self.content.assets.push(asset);
        self
    }

    /// Build the annotation dictionary.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("RichMedia".to_string()));

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

        // NM - unique name
        dict.insert(
            "NM".to_string(),
            Object::String(
                format!("RichMedia_{}", self.rect.x as i32)
                    .as_bytes()
                    .to_vec(),
            ),
        );

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // RichMediaContent - embedded content dictionary
        dict.insert("RichMediaContent".to_string(), Object::Dictionary(self.content.build()));

        // RichMediaSettings - settings dictionary
        dict.insert("RichMediaSettings".to_string(), Object::Dictionary(self.settings.build()));

        dict
    }

    /// Get the content reference.
    pub fn content(&self) -> &RichMediaContent {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_richmedia_activation() {
        assert_eq!(RichMediaActivation::Explicit.pdf_name(), "XA");
        assert_eq!(RichMediaActivation::PageVisible.pdf_name(), "PV");
        assert_eq!(RichMediaActivation::PageOpen.pdf_name(), "PO");
    }

    #[test]
    fn test_richmedia_deactivation() {
        assert_eq!(RichMediaDeactivation::Explicit.pdf_name(), "XD");
        assert_eq!(RichMediaDeactivation::PageNotVisible.pdf_name(), "PI");
        assert_eq!(RichMediaDeactivation::PageClose.pdf_name(), "PC");
    }

    #[test]
    fn test_richmedia_window() {
        assert_eq!(RichMediaWindow::Embedded.pdf_name(), "Embedded");
        assert_eq!(RichMediaWindow::Floating.pdf_name(), "Windowed");
        assert_eq!(RichMediaWindow::FullScreen.pdf_name(), "Fullscreen");
    }

    #[test]
    fn test_richmedia_asset_new() {
        let asset = RichMediaAsset::new("test.mp4", vec![1, 2, 3]);
        assert_eq!(asset.name, "test.mp4");
        assert_eq!(asset.data, vec![1, 2, 3]);
        assert!(asset.mime_type.is_none());
    }

    #[test]
    fn test_richmedia_asset_video() {
        let asset = RichMediaAsset::video("demo.mp4", vec![]);
        assert_eq!(asset.name, "demo.mp4");
        assert_eq!(asset.mime_type, Some("video/mp4".to_string()));
    }

    #[test]
    fn test_richmedia_asset_swf() {
        let asset = RichMediaAsset::swf("player.swf", vec![]);
        assert_eq!(asset.name, "player.swf");
        assert_eq!(asset.mime_type, Some("application/x-shockwave-flash".to_string()));
    }

    #[test]
    fn test_richmedia_content_new() {
        let content = RichMediaContent::new();
        assert!(content.assets.is_empty());
        assert!(content.configuration_name.is_none());
    }

    #[test]
    fn test_richmedia_content_builder() {
        let content = RichMediaContent::new()
            .add_asset(RichMediaAsset::video("video1.mp4", vec![]))
            .add_asset(RichMediaAsset::video("video2.mp4", vec![]))
            .with_configuration("VideoPlayer");

        assert_eq!(content.assets.len(), 2);
        assert_eq!(content.configuration_name, Some("VideoPlayer".to_string()));
    }

    #[test]
    fn test_richmedia_content_build() {
        let content = RichMediaContent::new()
            .add_asset(RichMediaAsset::video("test.mp4", vec![]))
            .with_configuration("Player");

        let dict = content.build();

        assert_eq!(dict.get("Type"), Some(&Object::Name("RichMediaContent".to_string())));
        assert!(dict.contains_key("Assets"));
        assert!(dict.contains_key("Configurations"));
    }

    #[test]
    fn test_richmedia_settings_default() {
        let settings = RichMediaSettings::default();

        assert!(matches!(settings.activation, RichMediaActivation::Explicit));
        assert!(matches!(settings.deactivation, RichMediaDeactivation::PageNotVisible));
        assert!(matches!(settings.window, RichMediaWindow::Embedded));
        assert!(settings.toolbar);
        assert!(!settings.transparent);
    }

    #[test]
    fn test_richmedia_settings_builder() {
        let settings = RichMediaSettings::new()
            .with_activation(RichMediaActivation::PageVisible)
            .with_deactivation(RichMediaDeactivation::PageClose)
            .with_window(RichMediaWindow::Floating)
            .with_toolbar(false)
            .with_transparent(true);

        assert!(matches!(settings.activation, RichMediaActivation::PageVisible));
        assert!(matches!(settings.deactivation, RichMediaDeactivation::PageClose));
        assert!(matches!(settings.window, RichMediaWindow::Floating));
        assert!(!settings.toolbar);
        assert!(settings.transparent);
    }

    #[test]
    fn test_richmedia_settings_build() {
        let settings = RichMediaSettings::new();
        let dict = settings.build();

        assert_eq!(dict.get("Type"), Some(&Object::Name("RichMediaSettings".to_string())));
        assert!(dict.contains_key("Activation"));
        assert!(dict.contains_key("Deactivation"));
    }

    #[test]
    fn test_richmedia_annotation_new() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let content = RichMediaContent::new();
        let annot = RichMediaAnnotation::new(rect, content);

        assert_eq!(annot.rect.x, 72.0);
        assert!(annot.content.assets.is_empty());
    }

    #[test]
    fn test_richmedia_annotation_video() {
        let rect = Rect::new(100.0, 300.0, 400.0, 300.0);
        let annot = RichMediaAnnotation::video(rect, "demo.mp4", vec![0u8; 100]);

        assert_eq!(annot.content.assets.len(), 1);
        assert_eq!(annot.content.assets[0].name, "demo.mp4");
    }

    #[test]
    fn test_richmedia_annotation_builder() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let annot = RichMediaAnnotation::video(rect, "video.mp4", vec![])
            .with_title("Video Player")
            .with_activation(RichMediaActivation::PageVisible)
            .with_window(RichMediaWindow::Embedded)
            .with_toolbar(true)
            .with_contents("A demonstration video");

        assert_eq!(annot.title, Some("Video Player".to_string()));
        assert!(matches!(annot.settings.activation, RichMediaActivation::PageVisible));
        assert!(matches!(annot.settings.window, RichMediaWindow::Embedded));
        assert!(annot.settings.toolbar);
        assert_eq!(annot.contents, Some("A demonstration video".to_string()));
    }

    #[test]
    fn test_richmedia_annotation_add_asset() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let annot = RichMediaAnnotation::new(rect, RichMediaContent::new())
            .add_asset(RichMediaAsset::video("video1.mp4", vec![]))
            .add_asset(RichMediaAsset::video("video2.mp4", vec![]));

        assert_eq!(annot.content.assets.len(), 2);
    }

    #[test]
    fn test_richmedia_annotation_build() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let annot =
            RichMediaAnnotation::video(rect, "test.mp4", vec![1, 2, 3]).with_title("Test Video");

        let dict = annot.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("RichMedia".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("T")); // Title
        assert!(dict.contains_key("RichMediaContent"));
        assert!(dict.contains_key("RichMediaSettings"));
    }
}
