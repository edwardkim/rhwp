//! Debug visualizer for rendering PDF pages with element annotations.

use crate::api::Pdf;
use crate::editor::{PdfElement, PdfPage};
use crate::error::{Error, Result};
use crate::geometry::Rect;

#[cfg(feature = "rendering")]
use crate::rendering::{RenderOptions, RenderedImage};

#[cfg(feature = "rendering")]
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Colors for different element types.
#[derive(Debug, Clone)]
pub struct ElementColors {
    /// Color for text elements (RGBA)
    pub text: [f32; 4],
    /// Color for image elements (RGBA)
    pub image: [f32; 4],
    /// Color for path elements (RGBA)
    pub path: [f32; 4],
    /// Color for table elements (RGBA)
    pub table: [f32; 4],
    /// Color for structure elements (RGBA)
    pub structure: [f32; 4],
}

impl Default for ElementColors {
    fn default() -> Self {
        Self {
            text: [1.0, 0.0, 0.0, 0.5],      // Red with 50% opacity
            image: [0.0, 1.0, 0.0, 0.5],     // Green with 50% opacity
            path: [0.0, 0.0, 1.0, 0.5],      // Blue with 50% opacity
            table: [1.0, 1.0, 0.0, 0.5],     // Yellow with 50% opacity
            structure: [1.0, 0.0, 1.0, 0.5], // Magenta with 50% opacity
        }
    }
}

/// Options for debug visualization.
#[derive(Debug, Clone)]
pub struct DebugOptions {
    /// Whether to show text element bounding boxes
    pub show_text_bounds: bool,
    /// Whether to show image element bounding boxes
    pub show_image_bounds: bool,
    /// Whether to show path element bounding boxes
    pub show_path_bounds: bool,
    /// Whether to show table element bounding boxes
    pub show_table_bounds: bool,
    /// Whether to show structure element bounding boxes
    pub show_structure_bounds: bool,
    /// Whether to label elements with their type
    pub label_elements: bool,
    /// Line width for bounding boxes
    pub line_width: f32,
    /// Colors for different element types
    pub colors: ElementColors,
    /// DPI for rendering
    pub dpi: u32,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            show_text_bounds: true,
            show_image_bounds: true,
            show_path_bounds: true,
            show_table_bounds: true,
            show_structure_bounds: false,
            label_elements: false,
            line_width: 1.0,
            colors: ElementColors::default(),
            dpi: 150,
        }
    }
}

impl DebugOptions {
    /// Show only text bounds.
    pub fn text_only() -> Self {
        Self {
            show_text_bounds: true,
            show_image_bounds: false,
            show_path_bounds: false,
            show_table_bounds: false,
            show_structure_bounds: false,
            ..Default::default()
        }
    }

    /// Show all element types.
    pub fn all() -> Self {
        Self {
            show_text_bounds: true,
            show_image_bounds: true,
            show_path_bounds: true,
            show_table_bounds: true,
            show_structure_bounds: true,
            ..Default::default()
        }
    }
}

/// Debug visualizer for PDF pages.
pub struct DebugVisualizer {
    options: DebugOptions,
}

impl DebugVisualizer {
    /// Create a new debug visualizer with the given options.
    pub fn new(options: DebugOptions) -> Self {
        Self { options }
    }

    /// Render a debug visualization of a page.
    ///
    /// This renders the page with bounding boxes overlaid for each element type.
    ///
    /// Requires the "rendering" feature.
    #[cfg(feature = "rendering")]
    pub fn render_debug_page(&self, pdf: &mut Pdf, page: usize) -> Result<RenderedImage> {
        // First render the page normally
        let render_options = RenderOptions::with_dpi(self.options.dpi);
        let base_image = pdf.render_page_with_options(page, &render_options)?;

        // Get the page for element analysis
        let page_obj = pdf.page(page)?;

        // Create pixmap from rendered image
        let mut pixmap = self.load_pixmap_from_png(&base_image.data)?;

        // Calculate scale factor
        let scale = self.options.dpi as f32 / 72.0;

        // Overlay bounding boxes
        self.draw_element_bounds(&mut pixmap, &page_obj, scale)?;

        // Encode back to PNG
        let data = pixmap
            .encode_png()
            .map_err(|e| Error::InvalidPdf(format!("PNG encoding failed: {}", e)))?;

        Ok(RenderedImage {
            data,
            width: base_image.width,
            height: base_image.height,
            format: crate::rendering::ImageFormat::Png,
        })
    }

    /// Render a debug visualization and save to file.
    ///
    /// Requires the "rendering" feature.
    #[cfg(feature = "rendering")]
    pub fn render_debug_page_to_file(
        &self,
        pdf: &mut Pdf,
        page: usize,
        path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let image = self.render_debug_page(pdf, page)?;
        image.save(path)
    }

    /// Load a pixmap from PNG data.
    #[cfg(feature = "rendering")]
    fn load_pixmap_from_png(&self, png_data: &[u8]) -> Result<Pixmap> {
        // Decode PNG using the image crate
        let img = image::load_from_memory(png_data)
            .map_err(|e| Error::InvalidPdf(format!("Failed to decode PNG: {}", e)))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        Pixmap::from_vec(rgba.into_raw(), tiny_skia::IntSize::from_wh(width, height).unwrap())
            .ok_or_else(|| Error::InvalidPdf("Failed to create pixmap".to_string()))
    }

    /// Draw element bounding boxes on the pixmap.
    #[cfg(feature = "rendering")]
    fn draw_element_bounds(&self, pixmap: &mut Pixmap, page: &PdfPage, scale: f32) -> Result<()> {
        let height = pixmap.height() as f32;

        // Transform to flip Y axis (PDF origin is bottom-left)
        let transform = Transform::from_scale(scale, -scale).post_translate(0.0, height);

        // Draw bounds for all child elements
        for element in page.children() {
            self.draw_element_bounds_recursive(pixmap, &element, transform)?;
        }

        Ok(())
    }

    /// Recursively draw bounds for an element and its children.
    #[cfg(feature = "rendering")]
    fn draw_element_bounds_recursive(
        &self,
        pixmap: &mut Pixmap,
        element: &PdfElement,
        transform: Transform,
    ) -> Result<()> {
        match element {
            PdfElement::Text(text) => {
                if self.options.show_text_bounds {
                    self.draw_rect(pixmap, &text.bbox(), &self.options.colors.text, transform);
                }
            },
            PdfElement::Image(image) => {
                if self.options.show_image_bounds {
                    self.draw_rect(pixmap, &image.bbox(), &self.options.colors.image, transform);
                }
            },
            PdfElement::Path(path) => {
                if self.options.show_path_bounds {
                    self.draw_rect(pixmap, &path.bbox(), &self.options.colors.path, transform);
                }
            },
            PdfElement::Table(table) => {
                if self.options.show_table_bounds {
                    self.draw_rect(pixmap, &table.bbox(), &self.options.colors.table, transform);
                }
            },
            PdfElement::Structure(structure) => {
                if self.options.show_structure_bounds {
                    self.draw_rect(
                        pixmap,
                        &structure.bbox(),
                        &self.options.colors.structure,
                        transform,
                    );
                }
                // Note: Structure children are not recursed as PdfStructure doesn't expose children
                // The structure bbox already encompasses all child elements
            },
        }

        Ok(())
    }

    /// Draw a rectangle outline on the pixmap.
    #[cfg(feature = "rendering")]
    fn draw_rect(&self, pixmap: &mut Pixmap, rect: &Rect, color: &[f32; 4], transform: Transform) {
        let mut paint = Paint::default();
        paint.set_color(
            Color::from_rgba(color[0], color[1], color[2], color[3]).unwrap_or(Color::BLACK),
        );
        paint.anti_alias = true;

        let stroke = Stroke {
            width: self.options.line_width,
            ..Stroke::default()
        };

        let mut path = PathBuilder::new();
        path.push_rect(
            tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
                .unwrap_or(tiny_skia::Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap()),
        );

        if let Some(path) = path.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
        }
    }

    /// Export page elements to JSON format.
    ///
    /// This produces a structured representation of all elements on the page.
    pub fn export_elements_json(&self, page: &PdfPage) -> Result<String> {
        let mut elements = Vec::new();

        for element in page.children() {
            elements.push(self.element_to_json(&element));
        }

        serde_json::to_string_pretty(&elements)
            .map_err(|e| Error::InvalidPdf(format!("JSON serialization failed: {}", e)))
    }

    /// Convert an element to a JSON-serializable structure.
    fn element_to_json(&self, element: &PdfElement) -> serde_json::Value {
        match element {
            PdfElement::Text(text) => {
                serde_json::json!({
                    "type": "text",
                    "content": text.text(),
                    "bbox": self.rect_to_json(&text.bbox()),
                })
            },
            PdfElement::Image(image) => {
                let (width, height) = image.dimensions();
                serde_json::json!({
                    "type": "image",
                    "width": width,
                    "height": height,
                    "bbox": self.rect_to_json(&image.bbox()),
                })
            },
            PdfElement::Path(path) => {
                serde_json::json!({
                    "type": "path",
                    "bbox": self.rect_to_json(&path.bbox()),
                })
            },
            PdfElement::Table(table) => {
                serde_json::json!({
                    "type": "table",
                    "bbox": self.rect_to_json(&table.bbox()),
                })
            },
            PdfElement::Structure(structure) => {
                serde_json::json!({
                    "type": "structure",
                    "structure_type": structure.structure_type(),
                    "bbox": self.rect_to_json(&structure.bbox()),
                })
            },
        }
    }

    /// Convert a Rect to JSON format.
    fn rect_to_json(&self, rect: &Rect) -> serde_json::Value {
        serde_json::json!({
            "x": rect.x,
            "y": rect.y,
            "width": rect.width,
            "height": rect.height,
        })
    }

    /// Export page elements to SVG format.
    ///
    /// This produces an SVG with element bounding boxes overlaid.
    pub fn export_elements_svg(
        &self,
        page: &PdfPage,
        page_width: f32,
        page_height: f32,
    ) -> Result<String> {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            page_width, page_height, page_width, page_height
        );

        // Add element rectangles
        for element in page.children() {
            self.element_to_svg(&mut svg, &element, page_height);
        }

        svg.push_str("</svg>");
        Ok(svg)
    }

    /// Add an element's SVG representation.
    fn element_to_svg(&self, svg: &mut String, element: &PdfElement, page_height: f32) {
        let (rect, color_name, stroke_color) = match element {
            PdfElement::Text(text) => {
                if !self.options.show_text_bounds {
                    return;
                }
                (text.bbox(), "text", self.color_to_svg(&self.options.colors.text))
            },
            PdfElement::Image(image) => {
                if !self.options.show_image_bounds {
                    return;
                }
                (image.bbox(), "image", self.color_to_svg(&self.options.colors.image))
            },
            PdfElement::Path(path) => {
                if !self.options.show_path_bounds {
                    return;
                }
                (path.bbox(), "path", self.color_to_svg(&self.options.colors.path))
            },
            PdfElement::Table(table) => {
                if !self.options.show_table_bounds {
                    return;
                }
                (table.bbox(), "table", self.color_to_svg(&self.options.colors.table))
            },
            PdfElement::Structure(structure) => {
                if self.options.show_structure_bounds {
                    let rect = structure.bbox();
                    let stroke = self.color_to_svg(&self.options.colors.structure);
                    // Flip Y for SVG (SVG origin is top-left)
                    let y = page_height - rect.y - rect.height;
                    svg.push_str(&format!(
                        r#"<rect class="structure" x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
                        rect.x, y, rect.width, rect.height, stroke, self.options.line_width
                    ));
                }
                // Note: Structure children are not recursed as PdfStructure doesn't expose children
                return;
            },
        };

        // Flip Y for SVG (SVG origin is top-left)
        let y = page_height - rect.y - rect.height;

        svg.push_str(&format!(
            r#"<rect class="{}" x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
            color_name, rect.x, y, rect.width, rect.height, stroke_color, self.options.line_width
        ));
    }

    /// Convert RGBA color to SVG rgba() format.
    fn color_to_svg(&self, color: &[f32; 4]) -> String {
        format!(
            "rgba({},{},{},{})",
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            color[3]
        )
    }
}

impl Default for DebugVisualizer {
    fn default() -> Self {
        Self::new(DebugOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_options_default() {
        let opts = DebugOptions::default();
        assert!(opts.show_text_bounds);
        assert!(opts.show_image_bounds);
        assert!(opts.show_path_bounds);
        assert!(opts.show_table_bounds);
        assert!(!opts.show_structure_bounds);
        assert!(!opts.label_elements);
        assert_eq!(opts.dpi, 150);
    }

    #[test]
    fn test_debug_options_text_only() {
        let opts = DebugOptions::text_only();
        assert!(opts.show_text_bounds);
        assert!(!opts.show_image_bounds);
        assert!(!opts.show_path_bounds);
        assert!(!opts.show_table_bounds);
    }

    #[test]
    fn test_debug_options_all() {
        let opts = DebugOptions::all();
        assert!(opts.show_text_bounds);
        assert!(opts.show_image_bounds);
        assert!(opts.show_path_bounds);
        assert!(opts.show_table_bounds);
        assert!(opts.show_structure_bounds);
    }

    #[test]
    fn test_element_colors_default() {
        let colors = ElementColors::default();
        assert_eq!(colors.text[0], 1.0); // Red
        assert_eq!(colors.image[1], 1.0); // Green
        assert_eq!(colors.path[2], 1.0); // Blue
    }

    #[test]
    fn test_color_to_svg() {
        let visualizer = DebugVisualizer::default();
        let color = [1.0, 0.0, 0.0, 0.5];
        let svg_color = visualizer.color_to_svg(&color);
        assert_eq!(svg_color, "rgba(255,0,0,0.5)");
    }
}
