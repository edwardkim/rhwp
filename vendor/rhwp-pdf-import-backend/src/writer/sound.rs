//! Sound annotations for PDF generation.
//!
//! This module provides support for sound annotations per PDF spec Section 12.5.6.15.
//! Sound annotations play audio when activated.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::SoundAnnotation;
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a sound annotation
//! let sound = SoundAnnotation::new(
//!     Rect::new(72.0, 720.0, 24.0, 24.0),
//!     audio_data,
//!     44100, // 44.1 kHz sample rate
//! ).with_channels(2)  // Stereo
//!  .with_bits(16)     // 16-bit audio
//!  .with_encoding(SoundEncoding::Signed);
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Sound encoding format per PDF spec Section 13.3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SoundEncoding {
    /// Raw unsigned sample values
    #[default]
    Raw,
    /// Signed twos-complement values
    Signed,
    /// mu-law encoded samples
    MuLaw,
    /// A-law encoded samples
    ALaw,
}

impl SoundEncoding {
    /// Get the PDF name for this encoding.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            SoundEncoding::Raw => "Raw",
            SoundEncoding::Signed => "Signed",
            SoundEncoding::MuLaw => "muLaw",
            SoundEncoding::ALaw => "ALaw",
        }
    }
}

/// Icon name for sound annotations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SoundIcon {
    /// Speaker icon (default)
    #[default]
    Speaker,
    /// Microphone icon
    Mic,
}

impl SoundIcon {
    /// Get the PDF name for this icon.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            SoundIcon::Speaker => "Speaker",
            SoundIcon::Mic => "Mic",
        }
    }
}

/// Sound data for embedding in PDF.
#[derive(Debug, Clone)]
pub struct SoundData {
    /// Raw audio sample data
    pub data: Vec<u8>,
    /// Sampling rate in Hz (required)
    pub sample_rate: u32,
    /// Number of audio channels (default: 1 = mono)
    pub channels: u8,
    /// Bits per sample (default: 8)
    pub bits_per_sample: u8,
    /// Encoding format
    pub encoding: SoundEncoding,
}

impl SoundData {
    /// Create new sound data with required parameters.
    pub fn new(data: Vec<u8>, sample_rate: u32) -> Self {
        Self {
            data,
            sample_rate,
            channels: 1,
            bits_per_sample: 8,
            encoding: SoundEncoding::default(),
        }
    }

    /// Create stereo sound data.
    pub fn stereo(data: Vec<u8>, sample_rate: u32) -> Self {
        Self {
            data,
            sample_rate,
            channels: 2,
            bits_per_sample: 16,
            encoding: SoundEncoding::Signed,
        }
    }

    /// Set the number of channels (1 = mono, 2 = stereo).
    pub fn with_channels(mut self, channels: u8) -> Self {
        self.channels = channels;
        self
    }

    /// Set bits per sample (8 or 16).
    pub fn with_bits(mut self, bits: u8) -> Self {
        self.bits_per_sample = bits;
        self
    }

    /// Set encoding format.
    pub fn with_encoding(mut self, encoding: SoundEncoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Build the Sound stream dictionary.
    pub fn build_sound_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Type (optional but recommended)
        dict.insert("Type".to_string(), Object::Name("Sound".to_string()));

        // R - sampling rate (required)
        dict.insert("R".to_string(), Object::Integer(self.sample_rate as i64));

        // C - channels (optional, default 1)
        if self.channels != 1 {
            dict.insert("C".to_string(), Object::Integer(self.channels as i64));
        }

        // B - bits per sample (optional, default 8)
        if self.bits_per_sample != 8 {
            dict.insert("B".to_string(), Object::Integer(self.bits_per_sample as i64));
        }

        // E - encoding (optional, default Raw)
        if self.encoding != SoundEncoding::Raw {
            dict.insert("E".to_string(), Object::Name(self.encoding.pdf_name().to_string()));
        }

        dict
    }
}

/// A sound annotation.
///
/// Per PDF spec Section 12.5.6.15, a sound annotation represents audio content
/// that can be played when the annotation is activated.
#[derive(Debug, Clone)]
pub struct SoundAnnotation {
    /// Bounding rectangle for the icon
    pub rect: Rect,
    /// Sound data
    pub sound: SoundData,
    /// Icon to display
    pub icon: SoundIcon,
    /// Annotation contents (description)
    pub contents: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Author/creator of the annotation
    pub author: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
}

impl SoundAnnotation {
    /// Create a new sound annotation.
    ///
    /// # Arguments
    ///
    /// * `rect` - Bounding rectangle for the icon
    /// * `data` - Raw audio sample data
    /// * `sample_rate` - Sampling rate in Hz (e.g., 44100 for CD quality)
    pub fn new(rect: Rect, data: Vec<u8>, sample_rate: u32) -> Self {
        Self {
            rect,
            sound: SoundData::new(data, sample_rate),
            icon: SoundIcon::default(),
            contents: None,
            flags: AnnotationFlags::printable(),
            author: None,
            modification_date: None,
        }
    }

    /// Create a sound annotation from SoundData.
    pub fn from_sound_data(rect: Rect, sound: SoundData) -> Self {
        Self {
            rect,
            sound,
            icon: SoundIcon::default(),
            contents: None,
            flags: AnnotationFlags::printable(),
            author: None,
            modification_date: None,
        }
    }

    /// Set the number of channels (1 = mono, 2 = stereo).
    pub fn with_channels(mut self, channels: u8) -> Self {
        self.sound.channels = channels;
        self
    }

    /// Set bits per sample.
    pub fn with_bits(mut self, bits: u8) -> Self {
        self.sound.bits_per_sample = bits;
        self
    }

    /// Set encoding format.
    pub fn with_encoding(mut self, encoding: SoundEncoding) -> Self {
        self.sound.encoding = encoding;
        self
    }

    /// Set the icon.
    pub fn with_icon(mut self, icon: SoundIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Set the description/contents.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Build the annotation dictionary for PDF output.
    ///
    /// Note: The Sound stream must be written separately and referenced via ObjectRef.
    /// This method returns the annotation dict; the sound stream is returned via `build_sound_stream()`.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Sound".to_string()));

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

        // Icon name
        dict.insert("Name".to_string(), Object::Name(self.icon.pdf_name().to_string()));

        // Contents (description)
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Author
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        // Modification date
        if let Some(ref date) = self.modification_date {
            dict.insert("M".to_string(), Object::text_string(date));
        }

        // Note: The Sound entry (stream reference) must be added by the caller
        // after the sound stream is written to the PDF

        dict
    }

    /// Build the sound stream dictionary.
    ///
    /// This returns the dictionary entries for the Sound stream object.
    /// The data should be written as a stream with these entries.
    pub fn build_sound_stream(&self) -> (HashMap<String, Object>, Vec<u8>) {
        (self.sound.build_sound_dict(), self.sound.data.clone())
    }

    /// Get the sound data reference.
    pub fn sound_data(&self) -> &SoundData {
        &self.sound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_data_new() {
        let data = vec![0u8; 1000];
        let sound = SoundData::new(data.clone(), 44100);

        assert_eq!(sound.data, data);
        assert_eq!(sound.sample_rate, 44100);
        assert_eq!(sound.channels, 1);
        assert_eq!(sound.bits_per_sample, 8);
        assert!(matches!(sound.encoding, SoundEncoding::Raw));
    }

    #[test]
    fn test_sound_data_stereo() {
        let data = vec![0u8; 2000];
        let sound = SoundData::stereo(data, 48000);

        assert_eq!(sound.sample_rate, 48000);
        assert_eq!(sound.channels, 2);
        assert_eq!(sound.bits_per_sample, 16);
        assert!(matches!(sound.encoding, SoundEncoding::Signed));
    }

    #[test]
    fn test_sound_data_builder() {
        let sound = SoundData::new(vec![], 22050)
            .with_channels(2)
            .with_bits(16)
            .with_encoding(SoundEncoding::MuLaw);

        assert_eq!(sound.channels, 2);
        assert_eq!(sound.bits_per_sample, 16);
        assert!(matches!(sound.encoding, SoundEncoding::MuLaw));
    }

    #[test]
    fn test_sound_data_build_dict() {
        let sound = SoundData::new(vec![], 44100)
            .with_channels(2)
            .with_bits(16)
            .with_encoding(SoundEncoding::Signed);

        let dict = sound.build_sound_dict();

        assert_eq!(dict.get("Type"), Some(&Object::Name("Sound".to_string())));
        assert_eq!(dict.get("R"), Some(&Object::Integer(44100)));
        assert_eq!(dict.get("C"), Some(&Object::Integer(2)));
        assert_eq!(dict.get("B"), Some(&Object::Integer(16)));
        assert_eq!(dict.get("E"), Some(&Object::Name("Signed".to_string())));
    }

    #[test]
    fn test_sound_annotation_new() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);
        let data = vec![0u8; 500];
        let annot = SoundAnnotation::new(rect, data, 44100);

        assert_eq!(annot.rect.x, 72.0);
        assert_eq!(annot.sound.sample_rate, 44100);
        assert!(matches!(annot.icon, SoundIcon::Speaker));
    }

    #[test]
    fn test_sound_annotation_builder() {
        let rect = Rect::new(100.0, 500.0, 32.0, 32.0);
        let annot = SoundAnnotation::new(rect, vec![], 22050)
            .with_channels(2)
            .with_bits(16)
            .with_encoding(SoundEncoding::Signed)
            .with_icon(SoundIcon::Mic)
            .with_contents("Audio recording")
            .with_author("User");

        assert_eq!(annot.sound.channels, 2);
        assert_eq!(annot.sound.bits_per_sample, 16);
        assert!(matches!(annot.icon, SoundIcon::Mic));
        assert_eq!(annot.contents, Some("Audio recording".to_string()));
        assert_eq!(annot.author, Some("User".to_string()));
    }

    #[test]
    fn test_sound_annotation_build() {
        let rect = Rect::new(72.0, 720.0, 24.0, 24.0);
        let annot = SoundAnnotation::new(rect, vec![1, 2, 3], 44100)
            .with_contents("Click to play")
            .with_icon(SoundIcon::Speaker);

        let dict = annot.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Sound".to_string())));
        assert_eq!(dict.get("Name"), Some(&Object::Name("Speaker".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("Contents"));
    }

    #[test]
    fn test_sound_encoding_names() {
        assert_eq!(SoundEncoding::Raw.pdf_name(), "Raw");
        assert_eq!(SoundEncoding::Signed.pdf_name(), "Signed");
        assert_eq!(SoundEncoding::MuLaw.pdf_name(), "muLaw");
        assert_eq!(SoundEncoding::ALaw.pdf_name(), "ALaw");
    }

    #[test]
    fn test_sound_icon_names() {
        assert_eq!(SoundIcon::Speaker.pdf_name(), "Speaker");
        assert_eq!(SoundIcon::Mic.pdf_name(), "Mic");
    }
}
