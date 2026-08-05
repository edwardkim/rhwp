use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// PDF export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfOptions {
    /// Enable color output
    #[serde(default = "default_true")]
    pub color: bool,

    /// Compression level (0-9, 9 = maximum)
    #[serde(default = "default_compression")]
    pub compression: u8,

    /// Include metadata
    #[serde(default = "default_true")]
    pub include_metadata: bool,

    /// Enable bookmarks
    #[serde(default = "default_true")]
    pub include_bookmarks: bool,
}

/// PNG export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngOptions {
    /// DPI (dots per inch)
    #[serde(default = "default_dpi")]
    pub dpi: u32,

    /// Quality (1-100)
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// Background color (hex, e.g., "ffffff")
    #[serde(default = "default_background")]
    pub background: String,

    /// Export all pages or just first page
    #[serde(default = "default_true")]
    pub export_all_pages: bool,
}

/// SVG export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgOptions {
    /// Preserve viewBox
    #[serde(default = "default_true")]
    pub preserve_viewbox: bool,

    /// Embed fonts
    #[serde(default = "default_false")]
    pub embed_fonts: bool,

    /// Convert text to paths
    #[serde(default = "default_false")]
    pub text_to_paths: bool,

    /// Separate layers
    #[serde(default = "default_false")]
    pub separate_layers: bool,
}

/// Text export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOptions {
    /// Include formatting (bold, italic, etc.)
    #[serde(default = "default_true")]
    pub include_formatting: bool,

    /// Include table structure
    #[serde(default = "default_true")]
    pub include_tables: bool,

    /// Include headers and footers
    #[serde(default = "default_true")]
    pub include_headers_footers: bool,

    /// Preserve paragraphs
    #[serde(default = "default_true")]
    pub preserve_paragraphs: bool,

    /// Line ending style (unix, windows, mac)
    #[serde(default = "default_line_ending")]
    pub line_ending: String,
}

/// Main conversion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    /// Enabled output formats
    pub formats: FormatsConfig,

    /// PDF options
    #[serde(default)]
    pub pdf: PdfOptions,

    /// PNG options
    #[serde(default)]
    pub png: PngOptions,

    /// SVG options
    #[serde(default)]
    pub svg: SvgOptions,

    /// Text options
    #[serde(default)]
    pub text: TextOptions,

    /// Behavior options
    #[serde(default)]
    pub behavior: BehaviorOptions,
}

/// Format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatsConfig {
    pub pdf: bool,
    pub png: bool,
    pub svg: bool,
    pub text: bool,
}

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorOptions {
    /// Overwrite existing files
    #[serde(default = "default_true")]
    pub overwrite: bool,

    /// Create subdirectories per format
    #[serde(default = "default_true")]
    pub create_format_dirs: bool,

    /// Copy failed files to separate directory
    #[serde(default = "default_false")]
    pub collect_failed: bool,

    /// Stop on first error
    #[serde(default = "default_false")]
    pub fail_fast: bool,

    /// Retry failed conversions
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Skip already converted files
    #[serde(default = "default_false")]
    pub skip_existing: bool,
}

impl ConversionConfig {
    /// Load configuration from JSON file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        let config: ConversionConfig =
            serde_json::from_str(&content).context("Failed to parse configuration JSON")?;
        Ok(config)
    }

    /// Get default configuration
    pub fn default() -> Self {
        ConversionConfig {
            formats: FormatsConfig {
                pdf: true,
                png: false,
                svg: false,
                text: true,
            },
            pdf: PdfOptions::default(),
            png: PngOptions::default(),
            svg: SvgOptions::default(),
            text: TextOptions::default(),
            behavior: BehaviorOptions::default(),
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_compression() -> u8 {
    6
}

fn default_dpi() -> u32 {
    300
}

fn default_quality() -> u8 {
    90
}

fn default_background() -> String {
    "ffffff".to_string()
}

fn default_line_ending() -> String {
    "unix".to_string()
}

fn default_retries() -> u32 {
    3
}

impl Default for PdfOptions {
    fn default() -> Self {
        PdfOptions {
            color: true,
            compression: 6,
            include_metadata: true,
            include_bookmarks: true,
        }
    }
}

impl Default for PngOptions {
    fn default() -> Self {
        PngOptions {
            dpi: 300,
            quality: 90,
            background: "ffffff".to_string(),
            export_all_pages: true,
        }
    }
}

impl Default for SvgOptions {
    fn default() -> Self {
        SvgOptions {
            preserve_viewbox: true,
            embed_fonts: false,
            text_to_paths: false,
            separate_layers: false,
        }
    }
}

impl Default for TextOptions {
    fn default() -> Self {
        TextOptions {
            include_formatting: true,
            include_tables: true,
            include_headers_footers: true,
            preserve_paragraphs: true,
            line_ending: "unix".to_string(),
        }
    }
}

impl Default for BehaviorOptions {
    fn default() -> Self {
        BehaviorOptions {
            overwrite: true,
            create_format_dirs: true,
            collect_failed: false,
            fail_fast: false,
            max_retries: 3,
            skip_existing: false,
        }
    }
}
