//! Movie annotations for PDF generation (legacy).
//!
//! This module provides support for movie annotations per PDF spec Section 12.5.6.17.
//! Movie annotations embed video content in PDFs.
//!
//! **Note:** This is a legacy format. For modern video embedding, prefer Screen annotations
//! with renditions (Section 12.5.6.18).
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::MovieAnnotation;
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a movie annotation
//! let movie = MovieAnnotation::new(
//!     Rect::new(72.0, 500.0, 320.0, 240.0),
//!     "video.mov",
//!     video_data,
//! ).with_aspect(640, 480)
//!  .with_title("Demo Video");
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Movie play mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MoviePlayMode {
    /// Play once and stop (default)
    #[default]
    Once,
    /// Play once then close the movie
    Open,
    /// Play in a loop
    Repeat,
    /// Play forward and backward in a loop (palindrome)
    Palindrome,
}

impl MoviePlayMode {
    /// Get the PDF name for this play mode.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            MoviePlayMode::Once => "Once",
            MoviePlayMode::Open => "Open",
            MoviePlayMode::Repeat => "Repeat",
            MoviePlayMode::Palindrome => "Palindrome",
        }
    }
}

/// Movie activation parameters.
#[derive(Debug, Clone)]
pub struct MovieActivation {
    /// Start time offset (in movie time units)
    pub start: Option<i64>,
    /// Duration to play (in movie time units)
    pub duration: Option<i64>,
    /// Playback rate (1.0 = normal)
    pub rate: f32,
    /// Volume (0.0 = mute, 1.0 = full)
    pub volume: f32,
    /// Show player controls
    pub show_controls: bool,
    /// Play mode
    pub mode: MoviePlayMode,
    /// Synchronous playback (block interaction while playing)
    pub synchronous: bool,
    /// Scale factor for floating window (if not embedded)
    pub fws_scale: Option<(f32, f32)>,
    /// Position for floating window
    pub fws_position: Option<(f32, f32)>,
}

impl Default for MovieActivation {
    fn default() -> Self {
        Self {
            start: None,
            duration: None,
            rate: 1.0,
            volume: 1.0,
            show_controls: true,
            mode: MoviePlayMode::default(),
            synchronous: false,
            fws_scale: None,
            fws_position: None,
        }
    }
}

impl MovieActivation {
    /// Create default activation parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set start offset.
    pub fn with_start(mut self, start: i64) -> Self {
        self.start = Some(start);
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, duration: i64) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set playback rate.
    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    /// Set volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set whether to show controls.
    pub fn with_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Set play mode.
    pub fn with_mode(mut self, mode: MoviePlayMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set synchronous playback.
    pub fn with_synchronous(mut self, sync: bool) -> Self {
        self.synchronous = sync;
        self
    }

    /// Build the activation dictionary.
    pub fn build(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Start time
        if let Some(start) = self.start {
            dict.insert("Start".to_string(), Object::Integer(start));
        }

        // Duration
        if let Some(duration) = self.duration {
            dict.insert("Duration".to_string(), Object::Integer(duration));
        }

        // Rate (default 1.0)
        if (self.rate - 1.0).abs() > 0.001 {
            dict.insert("Rate".to_string(), Object::Real(self.rate as f64));
        }

        // Volume (default 1.0)
        if (self.volume - 1.0).abs() > 0.001 {
            dict.insert("Volume".to_string(), Object::Real(self.volume as f64));
        }

        // ShowControls
        if !self.show_controls {
            dict.insert("ShowControls".to_string(), Object::Boolean(false));
        }

        // Mode
        if self.mode != MoviePlayMode::Once {
            dict.insert("Mode".to_string(), Object::Name(self.mode.pdf_name().to_string()));
        }

        // Synchronous
        if self.synchronous {
            dict.insert("Synchronous".to_string(), Object::Boolean(true));
        }

        // Floating window scale
        if let Some((x_scale, y_scale)) = self.fws_scale {
            dict.insert(
                "FWScale".to_string(),
                Object::Array(vec![Object::Real(x_scale as f64), Object::Real(y_scale as f64)]),
            );
        }

        // Floating window position
        if let Some((x, y)) = self.fws_position {
            dict.insert(
                "FWPosition".to_string(),
                Object::Array(vec![Object::Real(x as f64), Object::Real(y as f64)]),
            );
        }

        dict
    }
}

/// Movie data for embedding in PDF.
#[derive(Debug, Clone)]
pub struct MovieData {
    /// Movie filename (for the file specification)
    pub filename: String,
    /// Raw movie data
    pub data: Vec<u8>,
    /// Native width in pixels
    pub width: Option<u32>,
    /// Native height in pixels
    pub height: Option<u32>,
    /// Rotation angle (0, 90, 180, 270)
    pub rotation: u16,
    /// Poster image (first frame snapshot)
    pub poster: Option<Vec<u8>>,
}

impl MovieData {
    /// Create new movie data.
    pub fn new(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            data,
            width: None,
            height: None,
            rotation: 0,
            poster: None,
        }
    }

    /// Set the aspect ratio (native dimensions).
    pub fn with_aspect(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set rotation angle.
    pub fn with_rotation(mut self, rotation: u16) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set poster frame data.
    pub fn with_poster(mut self, poster: Vec<u8>) -> Self {
        self.poster = Some(poster);
        self
    }

    /// Build the Movie dictionary.
    pub fn build_movie_dict(&self, file_spec_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // F - file specification (required)
        dict.insert("F".to_string(), Object::Reference(file_spec_ref));

        // Aspect - native dimensions
        if let (Some(w), Some(h)) = (self.width, self.height) {
            dict.insert(
                "Aspect".to_string(),
                Object::Array(vec![Object::Integer(w as i64), Object::Integer(h as i64)]),
            );
        }

        // Rotate
        if self.rotation != 0 {
            dict.insert("Rotate".to_string(), Object::Integer(self.rotation as i64));
        }

        // Poster - use true to indicate poster should be retrieved from movie
        if self.poster.is_some() {
            // If we have poster data, it needs to be written as a stream reference
            // For now, use true to indicate the viewer should extract from movie
            dict.insert("Poster".to_string(), Object::Boolean(true));
        }

        dict
    }
}

/// A movie annotation (legacy video embedding).
///
/// Per PDF spec Section 12.5.6.17, a movie annotation represents video content.
/// This is a legacy format; prefer Screen annotations for modern PDFs.
#[derive(Debug, Clone)]
pub struct MovieAnnotation {
    /// Bounding rectangle for the video display area
    pub rect: Rect,
    /// Movie data
    pub movie: MovieData,
    /// Annotation title
    pub title: Option<String>,
    /// Activation parameters
    pub activation: MovieActivation,
    /// Whether to show border when not playing
    pub border: bool,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Contents/description
    pub contents: Option<String>,
}

impl MovieAnnotation {
    /// Create a new movie annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - Display area for the video
    /// * `filename` - Name for the embedded file
    /// * `data` - Raw video data
    pub fn new(rect: Rect, filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            rect,
            movie: MovieData::new(filename, data),
            title: None,
            activation: MovieActivation::default(),
            border: true,
            flags: AnnotationFlags::printable(),
            contents: None,
        }
    }

    /// Create a movie annotation from MovieData.
    pub fn from_movie_data(rect: Rect, movie: MovieData) -> Self {
        Self {
            rect,
            movie,
            title: None,
            activation: MovieActivation::default(),
            border: true,
            flags: AnnotationFlags::printable(),
            contents: None,
        }
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set native aspect ratio.
    pub fn with_aspect(mut self, width: u32, height: u32) -> Self {
        self.movie = self.movie.with_aspect(width, height);
        self
    }

    /// Set rotation angle.
    pub fn with_rotation(mut self, rotation: u16) -> Self {
        self.movie = self.movie.with_rotation(rotation);
        self
    }

    /// Set activation parameters.
    pub fn with_activation(mut self, activation: MovieActivation) -> Self {
        self.activation = activation;
        self
    }

    /// Set playback volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.activation.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set play mode.
    pub fn with_mode(mut self, mode: MoviePlayMode) -> Self {
        self.activation.mode = mode;
        self
    }

    /// Set whether to show border.
    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
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

    /// Build the annotation dictionary for PDF output.
    ///
    /// Note: The Movie dictionary and file specification must be written separately.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Movie".to_string()));

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

        // Border
        if !self.border {
            dict.insert(
                "Border".to_string(),
                Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(0)]),
            );
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // A - Activation parameters (if not default)
        let activation_dict = self.activation.build();
        if !activation_dict.is_empty() {
            dict.insert("A".to_string(), Object::Dictionary(activation_dict));
        }

        // Note: The Movie entry must be added by the caller after writing the Movie dict

        dict
    }

    /// Get the movie data reference.
    pub fn movie_data(&self) -> &MovieData {
        &self.movie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movie_data_new() {
        let data = vec![0u8; 1000];
        let movie = MovieData::new("video.mov", data.clone());

        assert_eq!(movie.filename, "video.mov");
        assert_eq!(movie.data, data);
        assert!(movie.width.is_none());
        assert!(movie.height.is_none());
        assert_eq!(movie.rotation, 0);
    }

    #[test]
    fn test_movie_data_builder() {
        let movie = MovieData::new("test.mp4", vec![])
            .with_aspect(1920, 1080)
            .with_rotation(90);

        assert_eq!(movie.width, Some(1920));
        assert_eq!(movie.height, Some(1080));
        assert_eq!(movie.rotation, 90);
    }

    #[test]
    fn test_movie_activation_default() {
        let activation = MovieActivation::default();

        assert_eq!(activation.rate, 1.0);
        assert_eq!(activation.volume, 1.0);
        assert!(activation.show_controls);
        assert!(matches!(activation.mode, MoviePlayMode::Once));
        assert!(!activation.synchronous);
    }

    #[test]
    fn test_movie_activation_builder() {
        let activation = MovieActivation::new()
            .with_start(100)
            .with_duration(5000)
            .with_rate(1.5)
            .with_volume(0.7)
            .with_controls(false)
            .with_mode(MoviePlayMode::Repeat);

        assert_eq!(activation.start, Some(100));
        assert_eq!(activation.duration, Some(5000));
        assert_eq!(activation.rate, 1.5);
        assert_eq!(activation.volume, 0.7);
        assert!(!activation.show_controls);
        assert!(matches!(activation.mode, MoviePlayMode::Repeat));
    }

    #[test]
    fn test_movie_activation_build() {
        let activation = MovieActivation::new()
            .with_rate(2.0)
            .with_mode(MoviePlayMode::Palindrome);

        let dict = activation.build();

        assert!(dict.contains_key("Rate"));
        assert_eq!(dict.get("Mode"), Some(&Object::Name("Palindrome".to_string())));
    }

    #[test]
    fn test_movie_annotation_new() {
        let rect = Rect::new(72.0, 500.0, 320.0, 240.0);
        let annot = MovieAnnotation::new(rect, "video.mov", vec![0u8; 100]);

        assert_eq!(annot.rect.x, 72.0);
        assert_eq!(annot.rect.width, 320.0);
        assert_eq!(annot.movie.filename, "video.mov");
        assert!(annot.border);
    }

    #[test]
    fn test_movie_annotation_builder() {
        let rect = Rect::new(100.0, 400.0, 640.0, 480.0);
        let annot = MovieAnnotation::new(rect, "demo.avi", vec![])
            .with_title("Demo Video")
            .with_aspect(640, 480)
            .with_volume(0.5)
            .with_mode(MoviePlayMode::Repeat)
            .with_border(false)
            .with_contents("A demonstration video");

        assert_eq!(annot.title, Some("Demo Video".to_string()));
        assert_eq!(annot.movie.width, Some(640));
        assert_eq!(annot.movie.height, Some(480));
        assert_eq!(annot.activation.volume, 0.5);
        assert!(matches!(annot.activation.mode, MoviePlayMode::Repeat));
        assert!(!annot.border);
        assert_eq!(annot.contents, Some("A demonstration video".to_string()));
    }

    #[test]
    fn test_movie_annotation_build() {
        let rect = Rect::new(72.0, 500.0, 320.0, 240.0);
        let annot = MovieAnnotation::new(rect, "video.mov", vec![])
            .with_title("Test Movie")
            .with_border(false);

        let dict = annot.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Movie".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("T")); // Title
        assert!(dict.contains_key("Border")); // Border disabled
    }

    #[test]
    fn test_play_mode_names() {
        assert_eq!(MoviePlayMode::Once.pdf_name(), "Once");
        assert_eq!(MoviePlayMode::Open.pdf_name(), "Open");
        assert_eq!(MoviePlayMode::Repeat.pdf_name(), "Repeat");
        assert_eq!(MoviePlayMode::Palindrome.pdf_name(), "Palindrome");
    }
}
