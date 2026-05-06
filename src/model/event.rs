//! 문서 변경 이벤트 정의
//!
//! 각 Command 메서드가 발행하는 이벤트로, 변경 이력 추적과
//! Batch 모드에서의 이벤트 수집에 사용된다.

/// 문서 변경 이벤트
#[derive(Debug, Clone)]
pub enum DocumentEvent {
    // ── 텍스트 편집 ──
    TextInserted { section: usize, para: usize, offset: usize, len: usize },
    TextDeleted { section: usize, para: usize, offset: usize, count: usize },
    ParagraphSplit { section: usize, para: usize, offset: usize },
    ParagraphMerged { section: usize, para: usize },
    ParagraphDeleted { section: usize, para: usize },
    ParagraphInserted { section: usize, para: usize },

    // ── 서식 변경 ──
    CharFormatChanged { section: usize, para: usize, start: usize, end: usize },
    ParaFormatChanged { section: usize, para: usize },

    // ── 표 구조 ──
    TableRowInserted { section: usize, para: usize, ctrl: usize },
    TableRowDeleted { section: usize, para: usize, ctrl: usize },
    TableColumnInserted { section: usize, para: usize, ctrl: usize },
    TableColumnDeleted { section: usize, para: usize, ctrl: usize },
    CellsMerged { section: usize, para: usize, ctrl: usize },
    CellSplit { section: usize, para: usize, ctrl: usize },
    CellTextChanged { section: usize, para: usize, ctrl: usize, cell: usize },

    // ── 개체 ──
    PictureInserted { section: usize, para: usize },
    PictureDeleted { section: usize, para: usize, ctrl: usize },
    PictureMoved { section: usize, para: usize, ctrl: usize },
    PictureResized { section: usize, para: usize, ctrl: usize },
    FootnoteDeleted { section: usize, para: usize, ctrl: usize },

    // ── 클립보드/HTML ──
    ContentPasted { section: usize, para: usize },
    HtmlImported { section: usize, para: usize },
}

impl DocumentEvent {
    /// 이벤트를 JSON 객체 문자열로 직렬화한다.
    pub fn to_json(&self) -> String {
        match self {
            // 텍스트 편집
            DocumentEvent::TextInserted { section, para, offset, len } =>
                format!(r#"{{"type":"TextInserted","section":{},"para":{},"offset":{},"len":{}}}"#, section, para, offset, len),
            DocumentEvent::TextDeleted { section, para, offset, count } =>
                format!(r#"{{"type":"TextDeleted","section":{},"para":{},"offset":{},"count":{}}}"#, section, para, offset, count),
            DocumentEvent::ParagraphSplit { section, para, offset } =>
                format!(r#"{{"type":"ParagraphSplit","section":{},"para":{},"offset":{}}}"#, section, para, offset),
            DocumentEvent::ParagraphMerged { section, para } =>
                format!(r#"{{"type":"ParagraphMerged","section":{},"para":{}}}"#, section, para),
            DocumentEvent::ParagraphDeleted { section, para } =>
                format!(r#"{{"type":"ParagraphDeleted","section":{},"para":{}}}"#, section, para),
            DocumentEvent::ParagraphInserted { section, para } =>
                format!(r#"{{"type":"ParagraphInserted","section":{},"para":{}}}"#, section, para),

            // 서식 변경
            DocumentEvent::CharFormatChanged { section, para, start, end } =>
                format!(r#"{{"type":"CharFormatChanged","section":{},"para":{},"start":{},"end":{}}}"#, section, para, start, end),
            DocumentEvent::ParaFormatChanged { section, para } =>
                format!(r#"{{"type":"ParaFormatChanged","section":{},"para":{}}}"#, section, para),

            // 표 구조
            DocumentEvent::TableRowInserted { section, para, ctrl } =>
                format!(r#"{{"type":"TableRowInserted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::TableRowDeleted { section, para, ctrl } =>
                format!(r#"{{"type":"TableRowDeleted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::TableColumnInserted { section, para, ctrl } =>
                format!(r#"{{"type":"TableColumnInserted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::TableColumnDeleted { section, para, ctrl } =>
                format!(r#"{{"type":"TableColumnDeleted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::CellsMerged { section, para, ctrl } =>
                format!(r#"{{"type":"CellsMerged","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::CellSplit { section, para, ctrl } =>
                format!(r#"{{"type":"CellSplit","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::CellTextChanged { section, para, ctrl, cell } =>
                format!(r#"{{"type":"CellTextChanged","section":{},"para":{},"ctrl":{},"cell":{}}}"#, section, para, ctrl, cell),

            // 개체
            DocumentEvent::PictureInserted { section, para } =>
                format!(r#"{{"type":"PictureInserted","section":{},"para":{}}}"#, section, para),
            DocumentEvent::PictureDeleted { section, para, ctrl } =>
                format!(r#"{{"type":"PictureDeleted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::PictureMoved { section, para, ctrl } =>
                format!(r#"{{"type":"PictureMoved","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::PictureResized { section, para, ctrl } =>
                format!(r#"{{"type":"PictureResized","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),
            DocumentEvent::FootnoteDeleted { section, para, ctrl } =>
                format!(r#"{{"type":"FootnoteDeleted","section":{},"para":{},"ctrl":{}}}"#, section, para, ctrl),

            // 클립보드/HTML
            DocumentEvent::ContentPasted { section, para } =>
                format!(r#"{{"type":"ContentPasted","section":{},"para":{}}}"#, section, para),
            DocumentEvent::HtmlImported { section, para } =>
                format!(r#"{{"type":"HtmlImported","section":{},"para":{}}}"#, section, para),
        }
    }
}

/// 이벤트 로그를 JSON 배열로 직렬화한다.
pub fn serialize_event_log(events: &[DocumentEvent]) -> String {
    if events.is_empty() {
        return r#"{"ok":true,"events":[]}"#.to_string();
    }
    let items: Vec<String> = events.iter().map(|e| e.to_json()).collect();
    format!(r#"{{"ok":true,"events":[{}]}}"#, items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_inserted_to_json() {
        let event = DocumentEvent::TextInserted { section: 0, para: 1, offset: 5, len: 3 };
        let json = event.to_json();
        assert!(json.contains(r#""type":"TextInserted""#));
        assert!(json.contains(r#""section":0"#));
        assert!(json.contains(r#""para":1"#));
        assert!(json.contains(r#""offset":5"#));
        assert!(json.contains(r#""len":3"#));
    }

    #[test]
    fn test_serialize_event_log_empty() {
        let result = serialize_event_log(&[]);
        assert_eq!(result, r#"{"ok":true,"events":[]}"#);
    }

    #[test]
    fn test_serialize_event_log_multiple() {
        let events = vec![
            DocumentEvent::TextInserted { section: 0, para: 0, offset: 0, len: 5 },
            DocumentEvent::TextDeleted { section: 0, para: 0, offset: 3, count: 2 },
        ];
        let result = serialize_event_log(&events);
        assert!(result.contains(r#""events":["#));
        assert!(result.contains(r#""type":"TextInserted""#));
        assert!(result.contains(r#""type":"TextDeleted""#));
    }
}
