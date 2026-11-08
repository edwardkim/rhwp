//! Screen annotations for PDF generation.
//!
//! This module provides support for screen annotations per PDF spec Section 12.5.6.18.
//! Screen annotations provide the modern approach for embedding multimedia content
//! (video, audio) in PDFs, replacing the legacy Movie annotation.
//!
//! # Architecture
//!
//! Screen annotations use a Rendition system:
//! - ScreenAnnotation contains a Rendition Action
//! - Rendition Action references a Media Rendition
//! - Media Rendition references a Media Clip
//! - Media Clip contains or references the actual media data
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::{ScreenAnnotation, MediaClip, MediaRendition};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a screen annotation with video
//! let clip = MediaClip::new("demo.mp4", video_data)
//!     .with_mime_type("video/mp4");
//!
//! let rendition = MediaRendition::new(clip)
//!     .with_name("Demo Video");
//!
//! let screen = ScreenAnnotation::new(
//!     Rect::new(72.0, 400.0, 320.0, 240.0),
//!     rendition,
//! ).with_title("Video Player");
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Media clip temporal access type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TemporalAccess {
    /// Media can be accessed randomly
    #[default]
    Random,
    /// Media must be accessed sequentially
    Sequential,
}

impl TemporalAccess {
    /// Get the PDF name for this access type.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            TemporalAccess::Random => "Random",
            TemporalAccess::Sequential => "Sequential",
        }
    }
}

/// Media permissions for the clip.
#[derive(Debug, Clone, Default)]
pub struct MediaPermissions {
    /// Allow temp file creation (default: true)
    pub temp_file_access: bool,
}

impl MediaPermissions {
    /// Create default permissions.
    pub fn new() -> Self {
        Self {
            temp_file_access: true,
        }
    }

    /// Build the permissions dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("MediaPermissions".to_string()));

        // TF - temp file permission
        dict.insert(
            "TF".to_string(),
            Object::String(if self.temp_file_access {
                "TEMPACCESS".as_bytes().to_vec()
            } else {
                "TEMPNEVER".as_bytes().to_vec()
            }),
        );

        dict
    }
}

/// Media clip data for Screen annotations.
///
/// Per PDF spec Section 13.2.4, a media clip dictionary specifies
/// the media data for playback.
#[derive(Debug, Clone)]
pub struct MediaClip {
    /// Name for the embedded file
    pub filename: String,
    /// Raw media data
    pub data: Vec<u8>,
    /// MIME type (Content-Type)
    pub content_type: Option<String>,
    /// Alternate text description
    pub alt_text: Option<String>,
    /// Temporal access type
    pub temporal_access: TemporalAccess,
    /// Media permissions
    pub permissions: MediaPermissions,
}

impl MediaClip {
    /// Create a new media clip.
    pub fn new(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            data,
            content_type: None,
            alt_text: None,
            temporal_access: TemporalAccess::default(),
            permissions: MediaPermissions::default(),
        }
    }

    /// Set the MIME content type.
    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.content_type = Some(mime.into());
        self
    }

    /// Set alternate text description.
    pub fn with_alt_text(mut self, text: impl Into<String>) -> Self {
        self.alt_text = Some(text.into());
        self
    }

    /// Set temporal access type.
    pub fn with_temporal_access(mut self, access: TemporalAccess) -> Self {
        self.temporal_access = access;
        self
    }

    /// Build the MediaClip dictionary.
    ///
    /// Note: The file data needs to be embedded separately and referenced.
    pub fn build(&self, file_spec_ref: Option<ObjectRef>) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("MediaClip".to_string()));
        dict.insert("S".to_string(), Object::Name("MCD".to_string())); // Media Clip Data

        // N - name/filename
        dict.insert("N".to_string(), Object::text_string(&self.filename));

        // D - file specification (if provided)
        if let Some(ref_obj) = file_spec_ref {
            dict.insert("D".to_string(), Object::Reference(ref_obj));
        }

        // CT - content type (MIME)
        if let Some(ref ct) = self.content_type {
            dict.insert("CT".to_string(), Object::text_string(ct));
        }

        // Alt - alternate text (array of language/text pairs)
        if let Some(ref alt) = self.alt_text {
            dict.insert(
                "Alt".to_string(),
                Object::Array(vec![
                    Object::String(Vec::new()), // empty string = default language
                    Object::text_string(alt),
                ]),
            );
        }

        // P - media permissions
        dict.insert("P".to_string(), Object::Dictionary(self.permissions.build()));

        dict
    }
}

/// Media play parameters.
///
/// Per PDF spec Section 13.2.5, these control playback behavior.
#[derive(Debug, Clone)]
pub struct MediaPlayParams {
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Auto-play when page is displayed
    pub auto_play: bool,
    /// Repeat count (0 = infinite loop)
    pub repeat_count: u32,
    /// Show player controls
    pub show_controls: bool,
    /// Background color RGB (0-255 each)
    pub bg_color: Option<(u8, u8, u8)>,
    /// Background opacity (0.0 to 1.0)
    pub bg_opacity: Option<f32>,
    /// Window type for floating player
    pub window_type: WindowType,
}

/// Window type for media playback.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WindowType {
    /// Play in annotation rectangle (default)
    #[default]
    Annotation,
    /// Play in floating window
    Floating,
    /// Play full screen
    FullScreen,
    /// Play hidden (audio only)
    Hidden,
}

impl WindowType {
    /// Get the PDF integer value.
    pub fn pdf_value(&self) -> i64 {
        match self {
            WindowType::Annotation => 3, // Window = 3 (annotation rectangle)
            WindowType::Floating => 0,   // Window = 0 (floating window)
            WindowType::FullScreen => 1, // Window = 1 (full screen)
            WindowType::Hidden => 2,     // Window = 2 (hidden)
        }
    }
}

impl Default for MediaPlayParams {
    fn default() -> Self {
        Self {
            volume: 1.0,
            auto_play: true,
            repeat_count: 1,
            show_controls: true,
            bg_color: None,
            bg_opacity: None,
            window_type: WindowType::default(),
        }
    }
}

impl MediaPlayParams {
    /// Create default play parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set volume (0.0 to 1.0).
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set auto-play on page display.
    pub fn with_auto_play(mut self, auto: bool) -> Self {
        self.auto_play = auto;
        self
    }

    /// Set repeat count (0 = infinite loop).
    pub fn with_repeat(mut self, count: u32) -> Self {
        self.repeat_count = count;
        self
    }

    /// Set whether to show controls.
    pub fn with_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Set background color.
    pub fn with_background(mut self, r: u8, g: u8, b: u8) -> Self {
        self.bg_color = Some((r, g, b));
        self
    }

    /// Set window type.
    pub fn with_window_type(mut self, wtype: WindowType) -> Self {
        self.window_type = wtype;
        self
    }

    /// Build the MediaPlayParameters dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("MediaPlayParams".to_string()));

        // V - volume (default 100 = 100%)
        if (self.volume - 1.0).abs() > 0.001 {
            dict.insert("V".to_string(), Object::Integer((self.volume * 100.0) as i64));
        }

        // A - auto play
        if !self.auto_play {
            dict.insert("A".to_string(), Object::Boolean(false));
        }

        // RC - repeat count (0 = infinite)
        if self.repeat_count != 1 {
            dict.insert("RC".to_string(), Object::Real(self.repeat_count as f64));
        }

        // BE - best effort parameters including controls visibility
        let mut be = HashMap::new();
        if !self.show_controls {
            be.insert("C".to_string(), Object::Boolean(false));
        }
        if !be.is_empty() {
            dict.insert("BE".to_string(), Object::Dictionary(be));
        }

        // BG - background color
        if let Some((r, g, b)) = self.bg_color {
            let mut bg = HashMap::new();
            bg.insert(
                "C".to_string(),
                Object::Array(vec![
                    Object::Real(r as f64 / 255.0),
                    Object::Real(g as f64 / 255.0),
                    Object::Real(b as f64 / 255.0),
                ]),
            );
            if let Some(opacity) = self.bg_opacity {
                bg.insert("O".to_string(), Object::Real(opacity as f64));
            }
            dict.insert("BG".to_string(), Object::Dictionary(bg));
        }

        // F - floating window parameters
        if self.window_type != WindowType::Annotation {
            let mut fw = HashMap::new();
            // W - window type (0=floating, 1=fullscreen, 2=hidden, 3=annotation)
            fw.insert("W".to_string(), Object::Integer(self.window_type.pdf_value()));
            dict.insert("F".to_string(), Object::Dictionary(fw));
        }

        dict
    }
}

/// Media rendition for Screen annotations.
///
/// Per PDF spec Section 13.2.3, a rendition describes how to present media.
#[derive(Debug, Clone)]
pub struct MediaRendition {
    /// Media clip
    pub clip: MediaClip,
    /// Play parameters
    pub play_params: MediaPlayParams,
    /// Rendition name
    pub name: Option<String>,
}

impl MediaRendition {
    /// Create a new media rendition.
    pub fn new(clip: MediaClip) -> Self {
        Self {
            clip,
            play_params: MediaPlayParams::default(),
            name: None,
        }
    }

    /// Set rendition name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set play parameters.
    pub fn with_play_params(mut self, params: MediaPlayParams) -> Self {
        self.play_params = params;
        self
    }

    /// Set volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.play_params.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set repeat count.
    pub fn with_repeat(mut self, count: u32) -> Self {
        self.play_params.repeat_count = count;
        self
    }

    /// Build the Rendition dictionary.
    ///
    /// Note: The MediaClip must be written separately and referenced.
    pub fn build(&self, clip_ref: Option<ObjectRef>) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Rendition".to_string()));
        dict.insert("S".to_string(), Object::Name("MR".to_string())); // Media Rendition

        // N - name
        if let Some(ref name) = self.name {
            dict.insert("N".to_string(), Object::text_string(name));
        }

        // C - media clip reference
        if let Some(ref_obj) = clip_ref {
            dict.insert("C".to_string(), Object::Reference(ref_obj));
        }

        // P - play parameters
        let play_dict = self.play_params.build();
        if !play_dict.is_empty() {
            dict.insert("P".to_string(), Object::Dictionary(play_dict));
        }

        dict
    }

    /// Get the media clip reference.
    pub fn clip(&self) -> &MediaClip {
        &self.clip
    }
}

/// Rendition action operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RenditionOperation {
    /// Play the rendition (operation 0)
    #[default]
    Play,
    /// Stop the rendition (operation 1)
    Stop,
    /// Pause the rendition (operation 2)
    Pause,
    /// Resume the rendition (operation 3)
    Resume,
    /// Play and associate (operation 4)
    PlayAssociate,
}

impl RenditionOperation {
    /// Get the PDF integer value.
    pub fn pdf_value(&self) -> i64 {
        match self {
            RenditionOperation::Play => 0,
            RenditionOperation::Stop => 1,
            RenditionOperation::Pause => 2,
            RenditionOperation::Resume => 3,
            RenditionOperation::PlayAssociate => 4,
        }
    }
}

/// A screen annotation for modern multimedia.
///
/// Per PDF spec Section 12.5.6.18, a screen annotation represents
/// a region on a page where media can be played.
#[derive(Debug, Clone)]
pub struct ScreenAnnotation {
    /// Bounding rectangle for the media display area
    pub rect: Rect,
    /// Media rendition
    pub rendition: MediaRendition,
    /// Annotation title
    pub title: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Contents/description
    pub contents: Option<String>,
    /// Background appearance stream
    pub appearance: Option<Vec<u8>>,
    /// Rendition operation
    pub operation: RenditionOperation,
}

impl ScreenAnnotation {
    /// Create a new screen annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - Display area for the media
    /// * `rendition` - Media rendition to play
    pub fn new(rect: Rect, rendition: MediaRendition) -> Self {
        Self {
            rect,
            rendition,
            title: None,
            flags: AnnotationFlags::printable(),
            contents: None,
            appearance: None,
            operation: RenditionOperation::default(),
        }
    }

    /// Create a screen annotation from a media clip.
    pub fn from_clip(rect: Rect, clip: MediaClip) -> Self {
        Self::new(rect, MediaRendition::new(clip))
    }

    /// Create a video annotation.
    pub fn video(rect: Rect, filename: impl Into<String>, data: Vec<u8>) -> Self {
        let clip = MediaClip::new(filename, data).with_mime_type("video/mp4");
        Self::from_clip(rect, clip)
    }

    /// Create an audio annotation.
    pub fn audio(rect: Rect, filename: impl Into<String>, data: Vec<u8>) -> Self {
        let clip = MediaClip::new(filename, data).with_mime_type("audio/mpeg");
        Self::from_clip(rect, clip)
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set rendition name.
    pub fn with_rendition_name(mut self, name: impl Into<String>) -> Self {
        self.rendition = self.rendition.with_name(name);
        self
    }

    /// Set volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.rendition.play_params.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set repeat count (0 = infinite loop).
    pub fn with_repeat(mut self, count: u32) -> Self {
        self.rendition.play_params.repeat_count = count;
        self
    }

    /// Set auto-play.
    pub fn with_auto_play(mut self, auto: bool) -> Self {
        self.rendition.play_params.auto_play = auto;
        self
    }

    /// Set whether to show controls.
    pub fn with_controls(mut self, show: bool) -> Self {
        self.rendition.play_params.show_controls = show;
        self
    }

    /// Set window type.
    pub fn with_window_type(mut self, wtype: WindowType) -> Self {
        self.rendition.play_params.window_type = wtype;
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

    /// Set rendition operation.
    pub fn with_operation(mut self, op: RenditionOperation) -> Self {
        self.operation = op;
        self
    }

    /// Build the annotation dictionary for PDF output.
    ///
    /// Note: The Rendition and MediaClip must be written separately.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Screen".to_string()));

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

        // MK - appearance characteristics (border)
        let mut mk = HashMap::new();
        mk.insert(
            "BG".to_string(),
            Object::Array(vec![
                Object::Real(0.9), // Light gray background
                Object::Real(0.9),
                Object::Real(0.9),
            ]),
        );
        dict.insert("MK".to_string(), Object::Dictionary(mk));

        // A - Rendition action
        // Note: The actual rendition reference (R) must be added by the caller
        let mut action = HashMap::new();
        action.insert("S".to_string(), Object::Name("Rendition".to_string()));
        action.insert("OP".to_string(), Object::Integer(self.operation.pdf_value()));
        // AN - annotation reference (this annotation) - must be added by caller
        dict.insert("A".to_string(), Object::Dictionary(action));

        dict
    }

    /// Get the rendition reference.
    pub fn rendition(&self) -> &MediaRendition {
        &self.rendition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_clip_new() {
        let data = vec![0u8; 1000];
        let clip = MediaClip::new("video.mp4", data.clone());

        assert_eq!(clip.filename, "video.mp4");
        assert_eq!(clip.data, data);
        assert!(clip.content_type.is_none());
    }

    #[test]
    fn test_media_clip_builder() {
        let clip = MediaClip::new("audio.mp3", vec![])
            .with_mime_type("audio/mpeg")
            .with_alt_text("Background music")
            .with_temporal_access(TemporalAccess::Sequential);

        assert_eq!(clip.content_type, Some("audio/mpeg".to_string()));
        assert_eq!(clip.alt_text, Some("Background music".to_string()));
        assert!(matches!(clip.temporal_access, TemporalAccess::Sequential));
    }

    #[test]
    fn test_media_clip_build() {
        let clip = MediaClip::new("test.mp4", vec![1, 2, 3]).with_mime_type("video/mp4");

        let dict = clip.build(None);

        assert_eq!(dict.get("Type"), Some(&Object::Name("MediaClip".to_string())));
        assert_eq!(dict.get("S"), Some(&Object::Name("MCD".to_string())));
        assert!(dict.contains_key("N"));
        assert!(dict.contains_key("CT"));
    }

    #[test]
    fn test_media_play_params_default() {
        let params = MediaPlayParams::default();

        assert_eq!(params.volume, 1.0);
        assert!(params.auto_play);
        assert_eq!(params.repeat_count, 1);
        assert!(params.show_controls);
        assert!(matches!(params.window_type, WindowType::Annotation));
    }

    #[test]
    fn test_media_play_params_builder() {
        let params = MediaPlayParams::new()
            .with_volume(0.5)
            .with_auto_play(false)
            .with_repeat(3)
            .with_controls(false)
            .with_background(255, 0, 0)
            .with_window_type(WindowType::FullScreen);

        assert_eq!(params.volume, 0.5);
        assert!(!params.auto_play);
        assert_eq!(params.repeat_count, 3);
        assert!(!params.show_controls);
        assert_eq!(params.bg_color, Some((255, 0, 0)));
        assert!(matches!(params.window_type, WindowType::FullScreen));
    }

    #[test]
    fn test_media_play_params_build() {
        let params = MediaPlayParams::new().with_volume(0.7).with_repeat(0); // infinite

        let dict = params.build();

        assert!(dict.contains_key("V")); // volume
        assert!(dict.contains_key("RC")); // repeat count
    }

    #[test]
    fn test_media_rendition_new() {
        let clip = MediaClip::new("video.mp4", vec![]);
        let rendition = MediaRendition::new(clip);

        assert!(rendition.name.is_none());
        assert_eq!(rendition.play_params.volume, 1.0);
    }

    #[test]
    fn test_media_rendition_builder() {
        let clip = MediaClip::new("video.mp4", vec![]);
        let rendition = MediaRendition::new(clip)
            .with_name("Demo Video")
            .with_volume(0.8)
            .with_repeat(2);

        assert_eq!(rendition.name, Some("Demo Video".to_string()));
        assert_eq!(rendition.play_params.volume, 0.8);
        assert_eq!(rendition.play_params.repeat_count, 2);
    }

    #[test]
    fn test_media_rendition_build() {
        let clip = MediaClip::new("test.mp4", vec![]);
        let rendition = MediaRendition::new(clip).with_name("Test");

        let dict = rendition.build(None);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Rendition".to_string())));
        assert_eq!(dict.get("S"), Some(&Object::Name("MR".to_string())));
        assert!(dict.contains_key("N"));
    }

    #[test]
    fn test_screen_annotation_new() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let clip = MediaClip::new("video.mp4", vec![]);
        let rendition = MediaRendition::new(clip);
        let screen = ScreenAnnotation::new(rect, rendition);

        assert_eq!(screen.rect.x, 72.0);
        assert_eq!(screen.rect.width, 320.0);
        assert!(screen.title.is_none());
    }

    #[test]
    fn test_screen_annotation_video() {
        let rect = Rect::new(100.0, 300.0, 640.0, 480.0);
        let screen = ScreenAnnotation::video(rect, "demo.mp4", vec![0u8; 100]);

        assert_eq!(screen.rendition.clip.filename, "demo.mp4");
        assert_eq!(screen.rendition.clip.content_type, Some("video/mp4".to_string()));
    }

    #[test]
    fn test_screen_annotation_audio() {
        let rect = Rect::new(100.0, 700.0, 200.0, 50.0);
        let screen = ScreenAnnotation::audio(rect, "music.mp3", vec![]);

        assert_eq!(screen.rendition.clip.content_type, Some("audio/mpeg".to_string()));
    }

    #[test]
    fn test_screen_annotation_builder() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let screen = ScreenAnnotation::video(rect, "video.mp4", vec![])
            .with_title("Video Player")
            .with_rendition_name("Main Video")
            .with_volume(0.5)
            .with_repeat(0)
            .with_auto_play(false)
            .with_controls(true)
            .with_contents("Demo video content");

        assert_eq!(screen.title, Some("Video Player".to_string()));
        assert_eq!(screen.rendition.name, Some("Main Video".to_string()));
        assert_eq!(screen.rendition.play_params.volume, 0.5);
        assert_eq!(screen.rendition.play_params.repeat_count, 0);
        assert!(!screen.rendition.play_params.auto_play);
        assert!(screen.rendition.play_params.show_controls);
        assert_eq!(screen.contents, Some("Demo video content".to_string()));
    }

    #[test]
    fn test_screen_annotation_build() {
        let rect = Rect::new(72.0, 400.0, 320.0, 240.0);
        let screen = ScreenAnnotation::video(rect, "test.mp4", vec![]).with_title("Test Video");

        let dict = screen.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Screen".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("T")); // Title
        assert!(dict.contains_key("A")); // Action
        assert!(dict.contains_key("MK")); // Appearance
    }

    #[test]
    fn test_window_type_values() {
        assert_eq!(WindowType::Floating.pdf_value(), 0);
        assert_eq!(WindowType::FullScreen.pdf_value(), 1);
        assert_eq!(WindowType::Hidden.pdf_value(), 2);
        assert_eq!(WindowType::Annotation.pdf_value(), 3);
    }

    #[test]
    fn test_rendition_operation_values() {
        assert_eq!(RenditionOperation::Play.pdf_value(), 0);
        assert_eq!(RenditionOperation::Stop.pdf_value(), 1);
        assert_eq!(RenditionOperation::Pause.pdf_value(), 2);
        assert_eq!(RenditionOperation::Resume.pdf_value(), 3);
        assert_eq!(RenditionOperation::PlayAssociate.pdf_value(), 4);
    }

    #[test]
    fn test_temporal_access_names() {
        assert_eq!(TemporalAccess::Random.pdf_name(), "Random");
        assert_eq!(TemporalAccess::Sequential.pdf_name(), "Sequential");
    }
}
