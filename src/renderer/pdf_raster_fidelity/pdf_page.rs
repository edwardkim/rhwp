//! Minimal PDF page-tree builder/parser for isolated raster fixtures.

use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfBuildPage {
    pub width_pt: u32,
    pub height_pt: u32,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsolatedPdfPage {
    pub page_index: usize,
    pub width_pt: u32,
    pub height_pt: u32,
    pub contents: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfPageTree {
    pub page_count: usize,
    pub pages: Vec<IsolatedPdfPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfParseError {
    NotPdf,
    MissingCount,
    MissingPage { index: usize },
    BadMediaBox { index: usize },
}

impl core::fmt::Display for PdfParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPdf => write!(f, "not a PDF"),
            Self::MissingCount => write!(f, "PDF page tree has no /Count"),
            Self::MissingPage { index } => write!(f, "PDF missing page {}", index + 1),
            Self::BadMediaBox { index } => write!(f, "PDF page {} has a bad MediaBox", index + 1),
        }
    }
}

impl std::error::Error for PdfParseError {}

pub fn build_multipage_pdf(pages: &[PdfBuildPage]) -> Result<Vec<u8>, String> {
    if pages.is_empty() {
        return Err("PDF builder requires at least one page".to_string());
    }
    let mut body = String::from("%PDF-1.4\n%RHWP\n");
    let mut offsets = Vec::new();
    let push_obj = |body: &mut String, offsets: &mut Vec<usize>, obj: String| {
        offsets.push(body.len());
        body.push_str(&obj);
        if !body.ends_with('\n') {
            body.push('\n');
        }
    };

    let catalog_id = 1u32;
    let pages_id = 2u32;
    let first_page_id = 3u32;
    let page_ids: Vec<u32> = (0..pages.len()).map(|i| first_page_id + i as u32).collect();
    let content_ids: Vec<u32> = (0..pages.len())
        .map(|i| first_page_id + pages.len() as u32 + i as u32)
        .collect();

    push_obj(
        &mut body,
        &mut offsets,
        format!("1 0 obj\n<< /Type /Catalog /Pages {pages_id} 0 R >>\nendobj\n"),
    );
    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    push_obj(
        &mut body,
        &mut offsets,
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [{kids}] /Count {} >>\nendobj\n",
            pages.len()
        ),
    );
    for (index, page) in pages.iter().enumerate() {
        push_obj(
            &mut body,
            &mut offsets,
            format!(
                "{} 0 obj\n<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources << >> >>\nendobj\n",
                page_ids[index], page.width_pt, page.height_pt, content_ids[index]
            ),
        );
    }
    for (index, page) in pages.iter().enumerate() {
        let stream = page.contents.as_bytes();
        push_obj(
            &mut body,
            &mut offsets,
            format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                content_ids[index],
                stream.len(),
                page.contents
            ),
        );
    }

    let xref_at = body.len();
    let _ = write!(body, "xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
    for offset in &offsets {
        let _ = writeln!(body, "{offset:010} 00000 n ");
    }
    let _ = write!(
        body,
        "trailer\n<< /Size {} /Root {catalog_id} 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
        offsets.len() + 1
    );
    Ok(body.into_bytes())
}

pub fn parse_pdf_page_tree(pdf: &[u8]) -> Result<PdfPageTree, PdfParseError> {
    if !pdf.starts_with(b"%PDF-") {
        return Err(PdfParseError::NotPdf);
    }
    let text = String::from_utf8_lossy(pdf);
    let page_count = parse_count(&text).ok_or(PdfParseError::MissingCount)?;
    let mut pages = Vec::with_capacity(page_count);
    let boxes = parse_media_boxes(&text);
    let contents = parse_content_streams(&text);
    if boxes.len() < page_count || contents.len() < page_count {
        return Err(PdfParseError::MissingPage {
            index: boxes.len().min(contents.len()),
        });
    }
    for index in 0..page_count {
        let (width_pt, height_pt) = boxes[index];
        if width_pt == 0 || height_pt == 0 {
            return Err(PdfParseError::BadMediaBox { index });
        }
        let page_bytes = extract_isolated_page(pdf, index)?;
        pages.push(IsolatedPdfPage {
            page_index: index,
            width_pt,
            height_pt,
            contents: contents[index].clone(),
            bytes: page_bytes,
        });
    }
    Ok(PdfPageTree { page_count, pages })
}

pub fn extract_isolated_page(pdf: &[u8], page_index: usize) -> Result<Vec<u8>, PdfParseError> {
    let tree = parse_page_fields(pdf)?;
    if page_index >= tree.0 {
        return Err(PdfParseError::MissingPage { index: page_index });
    }
    let (width, height) = tree.1[page_index];
    let contents = tree.2[page_index].clone();
    build_multipage_pdf(&[PdfBuildPage {
        width_pt: width,
        height_pt: height,
        contents,
    }])
    .map_err(|_| PdfParseError::MissingPage { index: page_index })
}

fn parse_page_fields(pdf: &[u8]) -> Result<(usize, Vec<(u32, u32)>, Vec<String>), PdfParseError> {
    if !pdf.starts_with(b"%PDF-") {
        return Err(PdfParseError::NotPdf);
    }
    let text = String::from_utf8_lossy(pdf);
    let count = parse_count(&text).ok_or(PdfParseError::MissingCount)?;
    let boxes = parse_media_boxes(&text);
    let contents = parse_content_streams(&text);
    if boxes.len() < count || contents.len() < count {
        return Err(PdfParseError::MissingPage {
            index: boxes.len().min(contents.len()),
        });
    }
    Ok((count, boxes, contents))
}

fn parse_count(text: &str) -> Option<usize> {
    text.match_indices("/Count ")
        .filter_map(|(idx, _)| {
            text[idx + "/Count ".len()..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max()
}

fn parse_media_boxes(text: &str) -> Vec<(u32, u32)> {
    let mut boxes = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find("/MediaBox [") {
        let after = &rest[idx + "/MediaBox [".len()..];
        let nums: Vec<f32> = after
            .split(|c: char| c == ']' || c == '\n')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .filter_map(|n| n.parse::<f32>().ok())
            .collect();
        if nums.len() >= 4 {
            let width = (nums[2] - nums[0]).max(0.0) as u32;
            let height = (nums[3] - nums[1]).max(0.0) as u32;
            boxes.push((width, height));
        }
        rest = &after[1.min(after.len())..];
    }
    boxes
}

fn parse_content_streams(text: &str) -> Vec<String> {
    let mut streams = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find("stream\n") {
        let after = &rest[idx + "stream\n".len()..];
        if let Some(end) = after.find("\nendstream") {
            streams.push(after[..end].to_string());
            rest = &after[end + "\nendstream".len()..];
        } else {
            break;
        }
    }
    streams
}

/// Draw a rectangle into a content stream. Used by isolation fixtures.
pub fn rect_content(x: u32, y: u32, w: u32, h: u32) -> String {
    format!("{x} {y} {w} {h} re f")
}
