//! PPTX to PDF conversion.
//!
//! Parses Microsoft PowerPoint presentations (.pptx) and converts them to PDF.
//!
//! PPTX files are ZIP archives containing XML files in Open XML format.
//! Slides are in `ppt/slides/slideN.xml`.

use super::OfficeConfig;
use crate::error::{Error, Result};
use crate::writer::{DocumentBuilder, DocumentMetadata, PageSize};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// PPTX to PDF converter.
pub struct PptxConverter {
    config: OfficeConfig,
}

impl PptxConverter {
    /// Create a new PPTX converter.
    pub fn new(config: OfficeConfig) -> Self {
        Self { config }
    }

    /// Convert PPTX bytes to PDF bytes.
    pub fn convert(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| Error::InvalidPdf(format!("Failed to open PPTX archive: {}", e)))?;

        // Get slide count and parse slides
        let slide_count = self.get_slide_count(&mut archive)?;
        let mut slides: Vec<SlideContent> = Vec::new();

        for i in 1..=slide_count {
            if let Ok(slide) = self.parse_slide(&mut archive, i) {
                slides.push(slide);
            }
        }

        if slides.is_empty() {
            return Err(Error::InvalidPdf("No slides found in presentation".to_string()));
        }

        // Get presentation title from metadata
        let title = self.parse_metadata(&mut archive).unwrap_or_default();

        self.build_pdf(&title, &slides)
    }

    /// Get the number of slides in the presentation.
    fn get_slide_count<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Result<usize> {
        // Count slide files in ppt/slides/
        let mut count = 0;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name();
                if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Parse a single slide.
    fn parse_slide<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        slide_num: usize,
    ) -> Result<SlideContent> {
        let slide_path = format!("ppt/slides/slide{}.xml", slide_num);

        let xml_content = match archive.by_name(&slide_path) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| Error::InvalidPdf(format!("Failed to read slide: {}", e)))?;
                content
            },
            Err(e) => return Err(Error::InvalidPdf(format!("Slide not found: {}", e))),
        };

        let mut slide = SlideContent {
            number: slide_num,
            title: None,
            text_boxes: Vec::new(),
        };

        // Parse XML
        let mut reader = Reader::from_str(&xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_shape = false;
        let mut in_text_body = false;
        let mut _in_paragraph = false;
        let mut in_text = false;
        let mut current_text = String::new();
        let mut current_paragraph = String::new();
        let mut is_title = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"sp" => {
                            in_shape = true;
                            is_title = false;
                        },
                        b"txBody" => {
                            in_text_body = true;
                        },
                        b"p" => {
                            _in_paragraph = true;
                            current_paragraph.clear();
                        },
                        b"t" => {
                            in_text = true;
                        },
                        b"ph" => {
                            // Placeholder type - check if title
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"type" {
                                    let val = String::from_utf8_lossy(&attr.value);
                                    if val == "title" || val == "ctrTitle" {
                                        is_title = true;
                                    }
                                }
                            }
                        },
                        _ => {},
                    }
                },
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"sp" => {
                            in_shape = false;
                            is_title = false;
                        },
                        b"txBody" => {
                            in_text_body = false;
                            if !current_text.is_empty() {
                                if is_title && slide.title.is_none() {
                                    slide.title = Some(current_text.trim().to_string());
                                } else {
                                    slide.text_boxes.push(TextBox {
                                        text: current_text.trim().to_string(),
                                    });
                                }
                                current_text.clear();
                            }
                        },
                        b"p" => {
                            _in_paragraph = false;
                            if !current_paragraph.is_empty() {
                                if !current_text.is_empty() {
                                    current_text.push('\n');
                                }
                                current_text.push_str(&current_paragraph);
                            }
                        },
                        b"t" => {
                            in_text = false;
                        },
                        _ => {},
                    }
                },
                Ok(Event::Text(e)) => {
                    if in_text && in_text_body && in_shape {
                        let text = e.xml11_content().unwrap_or_default();
                        current_paragraph.push_str(&text);
                    }
                },
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {},
            }
            buf.clear();
        }

        Ok(slide)
    }

    /// Parse presentation metadata.
    fn parse_metadata<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
    ) -> Option<String> {
        let xml_content = match archive.by_name("docProps/core.xml") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).ok()?;
                content
            },
            Err(_) => return None,
        };

        let mut reader = Reader::from_str(&xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_title = false;
        let mut title = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"title" {
                        in_title = true;
                    }
                },
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"title" {
                        in_title = false;
                    }
                },
                Ok(Event::Text(e)) => {
                    if in_title {
                        title = Some(e.xml11_content().unwrap_or_default().to_string());
                    }
                },
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {},
            }
            buf.clear();
        }

        title
    }

    /// Build PDF from parsed slides.
    fn build_pdf(&self, title: &str, slides: &[SlideContent]) -> Result<Vec<u8>> {
        let metadata = DocumentMetadata::new()
            .title(if title.is_empty() {
                "Presentation"
            } else {
                title
            })
            .creator("pdf_oxide");

        let mut builder = DocumentBuilder::new().metadata(metadata);

        // Use landscape orientation for slides (swap width/height from config)
        let (config_width, config_height) = self.config.page_size.dimensions();
        let slide_size =
            PageSize::Custom(config_height.max(config_width), config_height.min(config_width));
        let (page_width, page_height) = slide_size.dimensions();
        let margin = self.config.margins.left.min(self.config.margins.top); // Use smaller margin for slides

        // Font configuration from config
        let title_size = self.config.default_font_size * 2.5; // Title is 2.5x body text
        let text_size = self.config.default_font_size;
        let slide_number_size = self.config.default_font_size * 0.9;

        // Pre-process into render operations
        #[derive(Clone)]
        enum RenderOp {
            SlideNumber {
                num: usize,
                total: usize,
                x: f32,
                y: f32,
            },
            Title {
                text: String,
                y: f32,
            },
            Text {
                text: String,
                x: f32,
                y: f32,
            },
        }

        let mut all_ops: Vec<Vec<RenderOp>> = Vec::new();
        let total_slides = slides.len();

        for slide in slides {
            let mut ops: Vec<RenderOp> = Vec::new();
            let mut current_y = page_height - margin;

            // Slide number at bottom right
            ops.push(RenderOp::SlideNumber {
                num: slide.number,
                total: total_slides,
                x: page_width - margin - 50.0,
                y: self.config.margins.bottom,
            });

            // Slide title
            if let Some(ref title) = slide.title {
                ops.push(RenderOp::Title {
                    text: title.clone(),
                    y: current_y,
                });
                current_y -= title_size * 1.7; // Title height + spacing
            }

            // Text boxes
            let line_height = text_size * self.config.line_height;

            for text_box in &slide.text_boxes {
                if text_box.text.is_empty() {
                    continue;
                }

                // Split into lines
                for line in text_box.text.lines() {
                    if line.trim().is_empty() {
                        current_y -= line_height * 0.5;
                        continue;
                    }

                    // Simple word wrap
                    let max_width = page_width - margin * 2.0;
                    let avg_char_width = text_size * 0.5;
                    let max_chars = (max_width / avg_char_width) as usize;

                    if line.len() <= max_chars {
                        ops.push(RenderOp::Text {
                            text: line.to_string(),
                            x: margin,
                            y: current_y,
                        });
                        current_y -= line_height;
                    } else {
                        // Word wrap long lines
                        let words: Vec<&str> = line.split_whitespace().collect();
                        let mut current_line = String::new();

                        for word in words {
                            let test_line = if current_line.is_empty() {
                                word.to_string()
                            } else {
                                format!("{} {}", current_line, word)
                            };

                            if test_line.len() <= max_chars {
                                current_line = test_line;
                            } else {
                                if !current_line.is_empty() {
                                    ops.push(RenderOp::Text {
                                        text: current_line,
                                        x: margin,
                                        y: current_y,
                                    });
                                    current_y -= line_height;
                                }
                                current_line = word.to_string();
                            }
                        }

                        if !current_line.is_empty() {
                            ops.push(RenderOp::Text {
                                text: current_line,
                                x: margin,
                                y: current_y,
                            });
                            current_y -= line_height;
                        }
                    }
                }

                // Add spacing between text boxes
                current_y -= line_height * 0.5;
            }

            all_ops.push(ops);
        }

        // Render all operations
        let font_name = &self.config.default_font;
        let bold_font = format!("{}-Bold", font_name);

        for ops in &all_ops {
            let mut page_builder = builder.page(slide_size);

            for op in ops {
                match op {
                    RenderOp::SlideNumber { num, total, x, y } => {
                        page_builder = page_builder
                            .at(*x, *y)
                            .font(font_name, slide_number_size)
                            .text(&format!("{}/{}", num, total));
                    },
                    RenderOp::Title { text, y } => {
                        page_builder = page_builder
                            .at(margin, *y)
                            .font(&bold_font, title_size)
                            .text(text);
                    },
                    RenderOp::Text { text, x, y } => {
                        page_builder = page_builder
                            .at(*x, *y)
                            .font(font_name, text_size)
                            .text(text);
                    },
                }
            }

            page_builder.done();
        }

        builder.build()
    }
}

/// Parsed content from a slide.
#[derive(Debug)]
struct SlideContent {
    number: usize,
    title: Option<String>,
    text_boxes: Vec<TextBox>,
}

/// A text box on a slide.
#[derive(Debug)]
struct TextBox {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pptx_converter_new() {
        let config = OfficeConfig::default();
        let converter = PptxConverter::new(config);
        assert_eq!(converter.config.default_font, "Helvetica");
    }

    #[test]
    fn test_slide_content_default() {
        let slide = SlideContent {
            number: 1,
            title: Some("Test Title".to_string()),
            text_boxes: vec![],
        };
        assert_eq!(slide.number, 1);
        assert_eq!(slide.title, Some("Test Title".to_string()));
    }
}
