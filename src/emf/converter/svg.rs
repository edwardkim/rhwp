//! SVG 빌더 + COLORREF → rgb(R,G,B) 유틸.

use std::fmt::Write;

/// SVG 조각 문자열 빌더. 각 노드는 한 줄로 추가.
#[derive(Debug, Default)]
pub struct SvgBuilder {
    buf: String,
}

impl SvgBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, node: &str) {
        self.buf.push_str(node);
    }

    /// `<g transform="matrix(a b c d e f)">` 래퍼 열기.
    pub fn open_group_matrix(&mut self, m: [f32; 6]) {
        let _ = write!(
            self.buf,
            "<g transform=\"matrix({:.6} {:.6} {:.6} {:.6} {:.6} {:.6})\">",
            m[0], m[1], m[2], m[3], m[4], m[5],
        );
    }

    pub fn close_group(&mut self) {
        self.buf.push_str("</g>");
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.buf
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.buf.as_str()
    }
}

/// COLORREF(0x00BBGGRR) → SVG `rgb(R,G,B)` 문자열.
#[must_use]
pub fn colorref_to_rgb(c: u32) -> String {
    let r = (c & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = ((c >> 16) & 0xFF) as u8;
    format!("rgb({r},{g},{b})")
}

/// XML 특수문자 이스케이프.
#[must_use]
pub fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // XML 1.0 허용 문자: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            // 그 외(제어문자, U+FFFE, U+FFFF 등)는 제거 — 방출된 SVG 가 불법 XML 이 되지 않도록 (#3382)
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(ch),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(ch)
            }
            _ => {} // XML 무효 문자 제거
        }
    }
    out
}
