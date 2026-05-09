    use super::*;
    use crate::model::document::{Document, Section};
    use crate::model::paragraph::{Paragraph, LineSeg};
    use crate::paint::LAYER_TREE_SCHEMA;

    #[test]
    fn test_create_empty_document() {
        let doc = HwpDocument::create_empty();
        assert_eq!(doc.page_count(), 1);
    }

    #[test]
    fn test_empty_document_info() {
        let doc = HwpDocument::create_empty();
        let info = doc.get_document_info();
        assert!(info.contains("\"pageCount\":1"));
        assert!(info.contains("\"encrypted\":false"));
    }

    #[test]
    fn test_render_empty_page_svg() {
        let doc = HwpDocument::create_empty();
        let svg = doc.render_page_svg_native(0);
        assert!(svg.is_ok());
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_empty_page_html() {
        let doc = HwpDocument::create_empty();
        let html = doc.render_page_html_native(0);
        assert!(html.is_ok());
        let html = html.unwrap();
        assert!(html.contains("hwp-page"));
    }

    #[test]
    fn test_page_out_of_range() {
        let doc = HwpDocument::create_empty();
        let result = doc.render_page_svg_native(999);
        assert!(result.is_err());
        match result.unwrap_err() {
            HwpError::PageOutOfRange(n) => assert_eq!(n, 999),
            _ => panic!("Expected PageOutOfRange error"),
        }
    }

    #[test]
    fn test_page_layer_tree_export_uses_schema_contract() {
        let doc = HwpDocument::create_empty();
        let json = doc
            .get_page_layer_tree_native(0)
            .expect("empty document layer tree should export");

        assert!(json.contains(&format!(
            "\"schemaVersion\":{}",
            LAYER_TREE_SCHEMA.schema_version
        )));
        assert!(json.contains(&format!(
            "\"resourceTableVersion\":{}",
            LAYER_TREE_SCHEMA.resource_table_version
        )));
        assert!(json.contains(&format!("\"unit\":\"{}\"", LAYER_TREE_SCHEMA.unit)));
        assert!(json.contains(&format!(
            "\"coordinateSystem\":\"{}\"",
            LAYER_TREE_SCHEMA.coordinate_system
        )));
        assert!(json.contains("\"profile\":\"screen\""));
        assert!(json.contains("\"outputOptions\":{"));
    }

    #[test]
    fn test_page_layer_tree_export_preserves_output_options() {
        let mut doc = HwpDocument::create_empty();
        doc.set_show_paragraph_marks(true);
        doc.set_show_control_codes(true);
        doc.set_show_transparent_borders(true);
        doc.set_clip_enabled(false);
        doc.set_debug_overlay(true);

        let json = doc
            .get_page_layer_tree_native(0)
            .expect("layer tree should export output options");

        assert!(json.contains("\"showParagraphMarks\":true"));
        assert!(json.contains("\"showControlCodes\":true"));
        assert!(json.contains("\"showTransparentBorders\":true"));
        assert!(json.contains("\"clipEnabled\":false"));
        assert!(json.contains("\"debugOverlay\":true"));
    }

    #[test]
    fn test_normalize_canvas_scale_rejects_invalid_page_dimensions() {
        for (width, height) in [
            (0.0, 100.0),
            (100.0, 0.0),
            (-1.0, 100.0),
            (100.0, -1.0),
            (f64::NAN, 100.0),
            (100.0, f64::NAN),
            (f64::INFINITY, 100.0),
            (100.0, f64::INFINITY),
        ] {
            assert!(
                normalize_canvas_scale(width, height, 1.0).is_err(),
                "invalid page dimensions must fail: {width} x {height}"
            );
        }
    }

    #[test]
    fn test_normalize_canvas_scale_clamps_request_and_canvas_extent() {
        assert_eq!(normalize_canvas_scale(100.0, 100.0, 0.0), Ok(1.0));
        assert_eq!(normalize_canvas_scale(100.0, 100.0, f64::NAN), Ok(1.0));
        assert_eq!(normalize_canvas_scale(100.0, 100.0, f64::INFINITY), Ok(1.0));
        assert_eq!(normalize_canvas_scale(100.0, 100.0, 0.1), Ok(0.25));
        assert_eq!(normalize_canvas_scale(100.0, 100.0, 20.0), Ok(12.0));

        let scale = normalize_canvas_scale(20_000.0, 10_000.0, 1.0)
            .expect("large finite page should be scaled down");
        assert!((scale - (16_384.0 / 20_000.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_document_with_paragraphs() {
        use crate::model::page::PageDef;
        use crate::model::document::SectionDef;

        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();

        // A4 크기 페이지 정의 (단위: HwpUnit, 1pt = 100)
        let page_def = PageDef {
            width: 59528,  // A4 가로 (약 210mm)
            height: 84188, // A4 세로 (약 297mm)
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        };

        document.sections.push(Section {
            section_def: SectionDef {
                page_def,
                ..Default::default()
            },
            paragraphs: vec![
                Paragraph {
                    text: "첫 번째 문단".to_string(),
                    line_segs: vec![LineSeg {
                        line_height: 400,
                        baseline_distance: 320,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Paragraph {
                    text: "두 번째 문단".to_string(),
                    line_segs: vec![LineSeg {
                        line_height: 400,
                        baseline_distance: 320,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            raw_stream: None,
        });
        doc.set_document(document);

        assert_eq!(doc.page_count(), 1);
        let svg = doc.render_page_svg_native(0).unwrap();
        // 문자별 개별 렌더링이므로 개별 문자 존재 확인
        assert!(svg.contains(">첫</text>"));
        assert!(svg.contains(">문</text>"));
        assert!(svg.contains(">단</text>"));
    }

    #[test]
    fn test_set_dpi() {
        let mut doc = HwpDocument::create_empty();
        doc.set_dpi(72.0);
        assert!((doc.get_dpi() - 72.0).abs() < 0.01);
    }

    #[test]
    fn test_fallback_font() {
        let mut doc = HwpDocument::create_empty();
        assert_eq!(doc.get_fallback_font(), DEFAULT_FALLBACK_FONT);
        doc.set_fallback_font("/custom/font.ttf");
        assert_eq!(doc.get_fallback_font(), "/custom/font.ttf");
    }

    #[test]
    fn test_viewer_creation() {
        let doc = HwpDocument::create_empty();
        let viewer = HwpViewer::new(doc);
        assert_eq!(viewer.page_count(), 1);
        assert_eq!(viewer.pending_task_count(), 0);
    }

    #[test]
    fn test_viewer_viewport_update() {
        let doc = HwpDocument::create_empty();
        let mut viewer = HwpViewer::new(doc);
        viewer.update_viewport(0.0, 0.0, 800.0, 600.0);
        let visible = viewer.visible_pages();
        assert!(!visible.is_empty());
    }

    #[test]
    fn test_export_hwp_empty() {
        let doc = HwpDocument::create_empty();
        let bytes = doc.export_hwp_native();
        assert!(bytes.is_ok());
        let bytes = bytes.unwrap();
        // CFB 시그니처 확인
        assert!(bytes.len() > 512);
        assert_eq!(&bytes[0..4], &[0xD0, 0xCF, 0x11, 0xE0]);
    }

    #[test]
    fn test_hwp_error_display() {
        let err = HwpError::InvalidFile("테스트".to_string());
        assert!(err.to_string().contains("테스트"));
        let err = HwpError::PageOutOfRange(5);
        assert!(err.to_string().contains("5"));
    }

    /// 텍스트의 UTF-16 char_offsets를 생성한다.
    fn make_char_offsets(text: &str) -> Vec<u32> {
        let mut offsets = Vec::new();
        let mut pos: u32 = 0;
        for c in text.chars() {
            offsets.push(pos);
            pos += if (c as u32) > 0xFFFF { 2 } else { 1 };
        }
        offsets
    }

    /// 표 셀이 포함된 테스트 문서를 생성한다.
    fn create_doc_with_table() -> HwpDocument {
        use crate::model::page::PageDef;
        use crate::model::document::SectionDef;
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        use crate::model::Padding;

        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();

        let page_def = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        };

        let table = Table {
            row_count: 2,
            col_count: 2,
            padding: Padding { left: 100, right: 100, top: 100, bottom: 100 },
            cells: vec![
                Cell {
                    col: 0, row: 0, col_span: 1, row_span: 1,
                    width: 21000, height: 3000,
                    paragraphs: vec![Paragraph {
                        text: "셀A".to_string(),
                        char_count: 2,
                        char_offsets: make_char_offsets("셀A"),
                        line_segs: vec![LineSeg {
                            line_height: 400,
                            baseline_distance: 320,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Cell {
                    col: 1, row: 0, col_span: 1, row_span: 1,
                    width: 21000, height: 3000,
                    paragraphs: vec![Paragraph {
                        text: "셀B".to_string(),
                        char_count: 2,
                        char_offsets: make_char_offsets("셀B"),
                        line_segs: vec![LineSeg {
                            line_height: 400,
                            baseline_distance: 320,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Cell {
                    col: 0, row: 1, col_span: 1, row_span: 1,
                    width: 21000, height: 3000,
                    paragraphs: vec![Paragraph {
                        text: "셀C".to_string(),
                        char_count: 2,
                        char_offsets: make_char_offsets("셀C"),
                        line_segs: vec![LineSeg {
                            line_height: 400,
                            baseline_distance: 320,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Cell {
                    col: 1, row: 1, col_span: 1, row_span: 1,
                    width: 21000, height: 3000,
                    paragraphs: vec![Paragraph {
                        text: "셀D".to_string(),
                        char_count: 2,
                        char_offsets: make_char_offsets("셀D"),
                        line_segs: vec![LineSeg {
                            line_height: 400,
                            baseline_distance: 320,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let parent_para = Paragraph {
            text: String::new(),
            controls: vec![Control::Table(Box::new(table))],
            line_segs: vec![LineSeg {
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            }],
            ..Default::default()
        };

        document.sections.push(Section {
            section_def: SectionDef {
                page_def,
                ..Default::default()
            },
            paragraphs: vec![parent_para],
            raw_stream: None,
        });
        doc.set_document(document);
        doc
    }

    #[test]
    fn test_insert_text_in_cell() {
        let mut doc = create_doc_with_table();
        let result = doc.insert_text_in_cell_native(0, 0, 0, 0, 0, 1, "추가");
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"charOffset\":3"));

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells[0].paragraphs[0].text, "셀추가A");
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_delete_text_in_cell() {
        let mut doc = create_doc_with_table();
        let result = doc.delete_text_in_cell_native(0, 0, 0, 1, 0, 0, 1);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells[1].paragraphs[0].text, "B");
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_cell_text_edit_invalid_indices() {
        let mut doc = create_doc_with_table();

        let result = doc.insert_text_in_cell_native(0, 0, 0, 99, 0, 0, "X");
        assert!(result.is_err());

        let result = doc.insert_text_in_cell_native(0, 0, 5, 0, 0, 0, "X");
        assert!(result.is_err());

        let result = doc.insert_text_in_cell_native(99, 0, 0, 0, 0, 0, "X");
        assert!(result.is_err());
    }

    #[test]
    fn test_cell_text_layout_contains_cell_info() {
        let doc = create_doc_with_table();
        let layout = doc.get_page_text_layout_native(0);
        assert!(layout.is_ok());
        let json = layout.unwrap();

        assert!(json.contains("\"parentParaIdx\":"));
        assert!(json.contains("\"controlIdx\":"));
        assert!(json.contains("\"cellIdx\":"));
        assert!(json.contains("\"cellParaIdx\":"));
    }

    #[test]
    fn test_insert_and_delete_roundtrip_in_cell() {
        let mut doc = create_doc_with_table();

        let result = doc.insert_text_in_cell_native(0, 0, 0, 2, 0, 2, "테스트");
        assert!(result.is_ok());

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells[2].paragraphs[0].text, "셀C테스트");
        }

        let result = doc.delete_text_in_cell_native(0, 0, 0, 2, 0, 2, 3);
        assert!(result.is_ok());

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells[2].paragraphs[0].text, "셀C");
        }
    }

    #[test]
    fn test_svg_render_with_table_after_cell_edit() {
        let mut doc = create_doc_with_table();

        doc.insert_text_in_cell_native(0, 0, 0, 3, 0, 2, "수정됨").unwrap();
        // 삽입 후 셀 텍스트 확인
        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells[3].paragraphs[0].text, "셀D수정됨");
        }
        let svg = doc.render_page_svg_native(0);
        assert!(svg.is_ok());
        let svg = svg.unwrap();
        // 언어별 폰트 분기로 "셀", "D", "수정됨"이 별도 text run으로 분리될 수 있으므로
        // 각 부분이 SVG에 포함되는지 확인
        // 문자별 개별 렌더링이므로 개별 문자 존재 확인
        assert!(svg.contains(">수</text>"), "SVG에 '수' 없음");
        assert!(svg.contains(">정</text>"), "SVG에 '정' 없음");
        assert!(svg.contains(">됨</text>"), "SVG에 '됨' 없음");
    }


    #[test]
    fn test_get_page_control_layout_with_table() {
        let doc = create_doc_with_table();
        let result = doc.get_page_control_layout_native(0);
        assert!(result.is_ok());
        let json = result.unwrap();

        // 표 컨트롤이 포함되어야 함
        assert!(json.contains("\"type\":\"table\""));
        assert!(json.contains("\"rowCount\":"));
        assert!(json.contains("\"colCount\":"));
        // 문서 좌표 포함
        assert!(json.contains("\"secIdx\":"));
        assert!(json.contains("\"paraIdx\":"));
        assert!(json.contains("\"controlIdx\":"));
        // 셀 정보 포함
        assert!(json.contains("\"cells\":["));
        assert!(json.contains("\"cellIdx\":"));
        assert!(json.contains("\"row\":"));
        assert!(json.contains("\"col\":"));
    }

    #[test]
    fn test_control_layout_cell_bounding_boxes() {
        let doc = create_doc_with_table();
        let result = doc.get_page_control_layout_native(0);
        assert!(result.is_ok());
        let json = result.unwrap();

        // JSON 파싱 검증: 표 바운딩 박스가 유효한 크기를 가짐
        assert!(json.contains("\"w\":"));
        assert!(json.contains("\"h\":"));

        // 셀이 4개 (2x2 표)
        let cell_count = json.matches("\"cellIdx\":").count();
        assert_eq!(cell_count, 4, "2x2 표에는 4개의 셀이 있어야 합니다");
    }

    // === 표 구조 편집 테스트 ===

    #[test]
    fn test_insert_table_row_below() {
        let mut doc = create_doc_with_table();
        let result = doc.insert_table_row_native(0, 0, 0, 0, true);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"rowCount\":3"));
        assert!(json.contains("\"colCount\":2"));

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.row_count, 3);
            assert_eq!(table.cells.len(), 6);
            // 원래 첫 행의 셀A는 여전히 행 0
            assert_eq!(table.cells[0].row, 0);
            assert_eq!(table.cells[0].paragraphs[0].text, "셀A");
            // 새 행은 행 1 (빈 문단)
            assert_eq!(table.cells[2].row, 1);
            assert!(table.cells[2].paragraphs[0].text.is_empty());
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_insert_table_column_right() {
        let mut doc = create_doc_with_table();
        let result = doc.insert_table_column_native(0, 0, 0, 0, true);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"rowCount\":2"));
        assert!(json.contains("\"colCount\":3"));

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.col_count, 3);
            assert_eq!(table.cells.len(), 6);
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_merge_table_cells() {
        let mut doc = create_doc_with_table();
        // 첫 행의 2개 셀 병합
        let result = doc.merge_table_cells_native(0, 0, 0, 0, 0, 0, 1);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"cellCount\":3")); // 비주 셀 1개 제거

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells.len(), 3); // 비주 셀 제거됨
            let merged = &table.cells[0];
            assert_eq!(merged.col_span, 2);
            assert_eq!(merged.row_span, 1);
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_split_table_cell() {
        let mut doc = create_doc_with_table();
        // 먼저 병합
        doc.merge_table_cells_native(0, 0, 0, 0, 0, 0, 1).unwrap();
        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells.len(), 3);
        }

        // 나누기
        let result = doc.split_table_cell_native(0, 0, 0, 0, 0);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"cellCount\":4"));

        if let Some(Control::Table(table)) = doc.document.sections[0].paragraphs[0].controls.get(0) {
            assert_eq!(table.cells.len(), 4);
            let cell = &table.cells[0];
            assert_eq!(cell.col_span, 1);
            assert_eq!(cell.row_span, 1);
        } else {
            panic!("표 컨트롤을 찾을 수 없음");
        }
    }

    #[test]
    fn test_merge_then_control_layout_has_colSpan() {
        let mut doc = create_doc_with_table();
        // 병합 전: colSpan=1
        let layout_before = doc.get_page_control_layout_native(0).unwrap();
        assert!(!layout_before.contains("\"colSpan\":2"), "병합 전에는 colSpan:2가 없어야 합니다");

        // 병합: 첫 행의 2개 셀
        doc.merge_table_cells_native(0, 0, 0, 0, 0, 0, 1).unwrap();

        // 병합 후: colSpan=2가 레이아웃에 반영되어야 함
        let layout_after = doc.get_page_control_layout_native(0).unwrap();
        assert!(layout_after.contains("\"colSpan\":2"),
            "병합 후 colSpan:2가 있어야 합니다. 레이아웃: {}", layout_after);
    }

    #[test]
    fn test_insert_table_row_invalid_index() {
        let mut doc = create_doc_with_table();
        let result = doc.insert_table_row_native(0, 0, 0, 99, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_structure_edit_roundtrip() {
        let mut doc = create_doc_with_table();
        // 행 삽입
        doc.insert_table_row_native(0, 0, 0, 0, true).unwrap();
        // 열 삽입
        doc.insert_table_column_native(0, 0, 0, 0, true).unwrap();

        // 직렬화 → 재파싱
        let bytes = doc.export_hwp_native();
        assert!(bytes.is_ok(), "행/열 삽입 후 직렬화 실패");
        let bytes = bytes.unwrap();
        assert!(!bytes.is_empty());

        // 재파싱 가능 여부 확인
        let reparsed = crate::parser::parse_hwp(&bytes);
        assert!(reparsed.is_ok(), "재파싱 실패: {:?}", reparsed.err());
    }

    #[test]
    fn test_real_hwp_table_insert_row_roundtrip() {
        use std::path::Path;
        use crate::parser::record::Record;

        let path = Path::new("samples/hwp_table_test.hwp");
        if !path.exists() {
            eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = crate::parser::parse_hwp(&data).unwrap();

        // 원본 BodyText 레코드
        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&data).unwrap();
        let orig_bt = cfb.read_body_text_section(0, doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        // 원본 표 부분 레코드 추출 (CTRL_HEADER tbl ~ 다음 레벨0 레코드)
        let mut table_start = 0;
        let mut table_end = 0;
        for (i, rec) in orig_recs.iter().enumerate() {
            if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                if ctrl_id == crate::parser::tags::CTRL_TABLE {
                    table_start = i;
                    // 표 끝 찾기
                    table_end = orig_recs.len();
                    for j in (i+1)..orig_recs.len() {
                        if orig_recs[j].level <= rec.level {
                            table_end = j;
                            break;
                        }
                    }
                    break;
                }
            }
        }

        eprintln!("=== 원본 표 레코드: [{}..{}] ({} records) ===", table_start, table_end, table_end - table_start);
        for i in table_start..table_end {
            let r = &orig_recs[i];
            let tag = crate::parser::tags::tag_name(r.tag_id);
            eprintln!("  [{}] {} L{} {}B {:02X?}", i, tag, r.level, r.data.len(),
                &r.data[..r.data.len().min(16)]);
        }

        // 행 삽입 후 내보내기
        let mut hwp_doc = HwpDocument::create_empty();
        hwp_doc.set_document(crate::parser::parse_hwp(&data).unwrap());
        hwp_doc.insert_table_row_native(0, 3, 0, 0, true).unwrap();

        let exported = hwp_doc.export_hwp_native().unwrap();
        let mut cfb2 = crate::parser::cfb_reader::CfbReader::open(&exported).unwrap();
        let new_doc = crate::parser::parse_hwp(&exported).unwrap();
        let new_bt = cfb2.read_body_text_section(0, new_doc.header.compressed, false).unwrap();
        let new_recs = Record::read_all(&new_bt).unwrap();

        // 수정 후 표 레코드
        let mut new_table_start = 0;
        let mut new_table_end = 0;
        for (i, rec) in new_recs.iter().enumerate() {
            if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                if ctrl_id == crate::parser::tags::CTRL_TABLE {
                    new_table_start = i;
                    new_table_end = new_recs.len();
                    for j in (i+1)..new_recs.len() {
                        if new_recs[j].level <= rec.level {
                            new_table_end = j;
                            break;
                        }
                    }
                    break;
                }
            }
        }

        eprintln!("\n=== 수정 후 표 레코드: [{}..{}] ({} records) ===", new_table_start, new_table_end, new_table_end - new_table_start);
        for i in new_table_start..new_table_end {
            let r = &new_recs[i];
            let tag = crate::parser::tags::tag_name(r.tag_id);
            eprintln!("  [{}] {} L{} {}B {:02X?}", i, tag, r.level, r.data.len(),
                &r.data[..r.data.len().min(16)]);
        }

        // 원본 빈 셀(1,0)과 새 셀 LIST_HEADER 바이트 비교
        let orig_table_recs = &orig_recs[table_start..table_end];
        let new_table_recs = &new_recs[new_table_start..new_table_end];

        // LIST_HEADER 레코드 모두 추출
        eprintln!("\n=== 원본 LIST_HEADER 바이트 ===");
        for (i, r) in orig_table_recs.iter().enumerate() {
            if r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER {
                eprintln!("  [{}] {}B: {:02X?}", table_start + i, r.data.len(), &r.data);
            }
        }
        eprintln!("\n=== 수정 후 LIST_HEADER 바이트 ===");
        for (i, r) in new_table_recs.iter().enumerate() {
            if r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER {
                eprintln!("  [{}] {}B: {:02X?}", new_table_start + i, r.data.len(), &r.data);
            }
        }

        // PARA_HEADER 바이트 비교
        eprintln!("\n=== 원본 PARA_HEADER (표 내부) ===");
        for (i, r) in orig_table_recs.iter().enumerate() {
            if r.tag_id == crate::parser::tags::HWPTAG_PARA_HEADER {
                eprintln!("  [{}] {}B: {:02X?}", table_start + i, r.data.len(), &r.data);
            }
        }
        eprintln!("\n=== 수정 후 PARA_HEADER (표 내부) ===");
        for (i, r) in new_table_recs.iter().enumerate() {
            if r.tag_id == crate::parser::tags::HWPTAG_PARA_HEADER {
                eprintln!("  [{}] {}B: {:02X?}", new_table_start + i, r.data.len(), &r.data);
            }
        }

        // TABLE 레코드 비교
        eprintln!("\n=== TABLE 레코드 비교 ===");
        for r in orig_table_recs.iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_TABLE {
                eprintln!("  원본: {}B: {:02X?}", r.data.len(), &r.data);
            }
        }
        for r in new_table_recs.iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_TABLE {
                eprintln!("  수정: {}B: {:02X?}", r.data.len(), &r.data);
            }
        }
    }

    #[test]
    /// 실제 HWP 파일에서 셀 병합 후 포괄적 바이너리 비교
    fn test_merge_cells_roundtrip_real_hwp() {
        use std::path::Path;
        use crate::parser::record::Record;

        let orig_path = Path::new("samples/hwp_table_test.hwp");
        if !orig_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();

        // 1) 원본 → 수정 없이 라운드트립 (기준선)
        let mut baseline_doc = HwpDocument::from_bytes(&orig_data).unwrap();
        let baseline_exported = baseline_doc.export_hwp_native().unwrap();

        // 2) 원본 → 병합 후 내보내기
        let mut merged_doc = HwpDocument::from_bytes(&orig_data).unwrap();
        merged_doc.merge_table_cells_native(0, 3, 0, 2, 0, 2, 1).unwrap();
        let merged_exported = merged_doc.export_hwp_native().unwrap();

        // 검증용 파일 저장
        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/merge_test_baseline.hwp", &baseline_exported).unwrap();
        std::fs::write("output/merge_test_programmatic.hwp", &merged_exported).unwrap();
        eprintln!("검증 파일 저장: output/merge_test_baseline.hwp, output/merge_test_programmatic.hwp");

        // 기준선 BodyText
        let baseline_parsed = crate::parser::parse_hwp(&baseline_exported).unwrap();
        let mut baseline_cfb = crate::parser::cfb_reader::CfbReader::open(&baseline_exported).unwrap();
        let baseline_bt = baseline_cfb.read_body_text_section(0, baseline_parsed.header.compressed, false).unwrap();
        let baseline_recs = Record::read_all(&baseline_bt).unwrap();

        // 병합 BodyText
        let merged_parsed = crate::parser::parse_hwp(&merged_exported).unwrap();
        let mut merged_cfb = crate::parser::cfb_reader::CfbReader::open(&merged_exported).unwrap();
        let merged_bt = merged_cfb.read_body_text_section(0, merged_parsed.header.compressed, false).unwrap();
        let merged_recs = Record::read_all(&merged_bt).unwrap();

        eprintln!("기준선 레코드: {}, 병합 레코드: {}", baseline_recs.len(), merged_recs.len());

        // 표 범위 찾기
        let find_table = |recs: &[Record]| -> (usize, usize) {
            for (i, rec) in recs.iter().enumerate() {
                if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                    if ctrl_id == crate::parser::tags::CTRL_TABLE {
                        let mut end = recs.len();
                        for j in (i+1)..recs.len() {
                            if recs[j].level <= rec.level { end = j; break; }
                        }
                        return (i, end);
                    }
                }
            }
            (0, 0)
        };

        let (bt_start, bt_end) = find_table(&baseline_recs);
        let (mt_start, mt_end) = find_table(&merged_recs);
        eprintln!("기준선 표: [{}..{}] ({} recs), 병합 표: [{}..{}] ({} recs)",
            bt_start, bt_end, bt_end - bt_start, mt_start, mt_end, mt_end - mt_start);

        // 표 앞쪽 레코드 비교 (동일해야 함)
        let pre_count = bt_start.min(mt_start);
        for i in 0..pre_count {
            if baseline_recs[i].tag_id != merged_recs[i].tag_id || baseline_recs[i].data != merged_recs[i].data {
                let tag = crate::parser::tags::tag_name(baseline_recs[i].tag_id);
                eprintln!("!! 표 앞 [{}] {} 차이:", i, tag);
                eprintln!("  기준: {:02X?}", &baseline_recs[i].data[..baseline_recs[i].data.len().min(40)]);
                eprintln!("  병합: {:02X?}", &merged_recs[i].data[..merged_recs[i].data.len().min(40)]);
            }
        }

        // 표 뒤쪽 레코드 비교
        let bt_after = &baseline_recs[bt_end..];
        let mt_after = &merged_recs[mt_end..];
        if bt_after.len() != mt_after.len() {
            eprintln!("!! 표 뒤 레코드 수 차이: {} vs {}", bt_after.len(), mt_after.len());
        }
        for i in 0..bt_after.len().min(mt_after.len()) {
            if bt_after[i].tag_id != mt_after[i].tag_id || bt_after[i].data != mt_after[i].data {
                let tag = crate::parser::tags::tag_name(bt_after[i].tag_id);
                eprintln!("!! 표 뒤 [{}] {} 차이:", i, tag);
                eprintln!("  기준: {:02X?}", &bt_after[i].data[..bt_after[i].data.len().min(40)]);
                eprintln!("  병합: {:02X?}", &mt_after[i].data[..mt_after[i].data.len().min(40)]);
            }
        }

        // 표 내부 레코드 전체 출력
        eprintln!("\n=== 기준선 표 레코드 ===");
        for i in bt_start..bt_end {
            let r = &baseline_recs[i];
            let tag = crate::parser::tags::tag_name(r.tag_id);
            eprintln!("  [{}] {} L{} {}B {:02X?}", i, tag, r.level, r.data.len(),
                &r.data[..r.data.len().min(50)]);
        }
        eprintln!("\n=== 병합 표 레코드 ===");
        for i in mt_start..mt_end {
            let r = &merged_recs[i];
            let tag = crate::parser::tags::tag_name(r.tag_id);
            eprintln!("  [{}] {} L{} {}B {:02X?}", i, tag, r.level, r.data.len(),
                &r.data[..r.data.len().min(50)]);
        }

        // DocInfo 스트림 비교
        let mut baseline_cfb2 = crate::parser::cfb_reader::CfbReader::open(&baseline_exported).unwrap();
        let mut merged_cfb2 = crate::parser::cfb_reader::CfbReader::open(&merged_exported).unwrap();
        let baseline_di = baseline_cfb2.read_doc_info(baseline_parsed.header.compressed).unwrap();
        let merged_di = merged_cfb2.read_doc_info(merged_parsed.header.compressed).unwrap();
        if baseline_di == merged_di {
            eprintln!("\nDocInfo: 동일 ({}B)", baseline_di.len());
        } else {
            eprintln!("\n!! DocInfo 차이: {}B vs {}B", baseline_di.len(), merged_di.len());
            for i in 0..baseline_di.len().min(merged_di.len()) {
                if baseline_di[i] != merged_di[i] {
                    eprintln!("  offset {}: {:02X} vs {:02X}", i, baseline_di[i], merged_di[i]);
                    if i > 5 { eprintln!("  ... (더 있을 수 있음)"); break; }
                }
            }
        }

        // FileHeader 비교
        let baseline_hdr = &baseline_exported[0..256.min(baseline_exported.len())];
        let merged_hdr = &merged_exported[0..256.min(merged_exported.len())];
        if baseline_hdr != merged_hdr {
            eprintln!("\n!! FileHeader 차이 (첫 256바이트)");
        }

        eprintln!("\n파일 크기: 기준선={}B, 병합={}B", baseline_exported.len(), merged_exported.len());
    }

    /// 사용자 저장 파일 vs 프로그래밍적 병합 파일 비교
    #[test]
    fn test_compare_user_saved_vs_programmatic() {
        use std::path::Path;
        use crate::parser::record::Record;

        let orig_path = Path::new("samples/hwp_table_test.hwp");
        let saved_path = Path::new("samples/hwp_table_test_saved.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();

        // 프로그래밍적 병합 내보내기
        let mut merged_doc = HwpDocument::from_bytes(&orig_data).unwrap();
        merged_doc.merge_table_cells_native(0, 3, 0, 2, 0, 2, 1).unwrap();
        let prog_data = merged_doc.export_hwp_native().unwrap();

        // 사용자 저장 파일 BodyText
        let saved_parsed = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_parsed.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        // 프로그래밍적 병합 BodyText
        let prog_parsed = crate::parser::parse_hwp(&prog_data).unwrap();
        let mut prog_cfb = crate::parser::cfb_reader::CfbReader::open(&prog_data).unwrap();
        let prog_bt = prog_cfb.read_body_text_section(0, prog_parsed.header.compressed, false).unwrap();
        let prog_recs = Record::read_all(&prog_bt).unwrap();

        eprintln!("사용자 저장: {} recs, 프로그래밍: {} recs", saved_recs.len(), prog_recs.len());
        eprintln!("사용자 저장 파일: {}B, 프로그래밍 파일: {}B", saved_data.len(), prog_data.len());

        // 모든 레코드 비교
        let max_recs = saved_recs.len().max(prog_recs.len());
        let mut diffs = 0;
        for i in 0..max_recs {
            if i >= saved_recs.len() {
                eprintln!("!! [{}] 사용자에 없음, 프로그래밍: {} L{}", i,
                    crate::parser::tags::tag_name(prog_recs[i].tag_id), prog_recs[i].level);
                diffs += 1;
                continue;
            }
            if i >= prog_recs.len() {
                eprintln!("!! [{}] 프로그래밍에 없음, 사용자: {} L{}", i,
                    crate::parser::tags::tag_name(saved_recs[i].tag_id), saved_recs[i].level);
                diffs += 1;
                continue;
            }
            let s = &saved_recs[i];
            let p = &prog_recs[i];
            if s.tag_id != p.tag_id || s.level != p.level || s.data != p.data {
                let stag = crate::parser::tags::tag_name(s.tag_id);
                let ptag = crate::parser::tags::tag_name(p.tag_id);
                eprintln!("!! [{}] 차이:", i);
                eprintln!("  사용자: {} L{} {}B {:02X?}", stag, s.level, s.data.len(), &s.data[..s.data.len().min(50)]);
                eprintln!("  프로그: {} L{} {}B {:02X?}", ptag, p.level, p.data.len(), &p.data[..p.data.len().min(50)]);
                diffs += 1;
            }
        }
        eprintln!("총 차이: {} 레코드", diffs);

        // DocInfo 비교
        let mut saved_cfb2 = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_di = saved_cfb2.read_doc_info(saved_parsed.header.compressed).unwrap();
        let mut prog_cfb2 = crate::parser::cfb_reader::CfbReader::open(&prog_data).unwrap();
        let prog_di = prog_cfb2.read_doc_info(prog_parsed.header.compressed).unwrap();
        if saved_di == prog_di {
            eprintln!("\nDocInfo: 동일 ({}B)", saved_di.len());
        } else {
            eprintln!("\n!! DocInfo 차이: 사용자={}B, 프로그래밍={}B", saved_di.len(), prog_di.len());
        }

        // BodyText raw bytes 비교
        if saved_bt == prog_bt {
            eprintln!("BodyText raw: 동일 ({}B)", saved_bt.len());
        } else {
            eprintln!("!! BodyText raw 차이: 사용자={}B, 프로그래밍={}B", saved_bt.len(), prog_bt.len());
            let mut byte_diffs = 0;
            for i in 0..saved_bt.len().min(prog_bt.len()) {
                if saved_bt[i] != prog_bt[i] {
                    if byte_diffs < 10 {
                        eprintln!("  offset {}: {:02X} vs {:02X}", i, saved_bt[i], prog_bt[i]);
                    }
                    byte_diffs += 1;
                }
            }
            eprintln!("  총 바이트 차이: {} (+ 길이 차이: {})", byte_diffs,
                (saved_bt.len() as i64 - prog_bt.len() as i64).abs());
        }

        // 전체 CFB 파일 비교 (원본 라운드트립 vs 사용자 저장)
        let mut baseline_doc = HwpDocument::from_bytes(&orig_data).unwrap();
        let baseline_data = baseline_doc.export_hwp_native().unwrap();
        eprintln!("\n원본 라운드트립: {}B, 사용자 저장: {}B", baseline_data.len(), saved_data.len());

        // 프로그래밍적 병합 파일 디스크에 저장 (수동 확인용)
        let out_dir = Path::new("output");
        if out_dir.exists() {
            std::fs::write(out_dir.join("merge_test_programmatic.hwp"), &prog_data).unwrap();
            std::fs::write(out_dir.join("merge_test_baseline.hwp"), &baseline_data).unwrap();
            eprintln!("\n저장 완료:");
            eprintln!("  output/merge_test_baseline.hwp  (수정 없이 라운드트립)");
            eprintln!("  output/merge_test_programmatic.hwp  (프로그래밍적 병합)");
        }
    }

    /// 한컴 오피스 참조 파일 분석: 병합된 표의 셀 구조 확인
    #[test]
    fn test_analyze_hancom_merged_file() {
        use std::path::Path;
        use crate::parser::record::Record;

        let orig_path = Path::new("samples/hwp_table_test.hwp");
        let hancom_path = Path::new("samples/hwp_table_test-m.hwp");
        if !orig_path.exists() || !hancom_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let hancom_data = std::fs::read(hancom_path).unwrap();

        // 원본 BodyText
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        // 한컴 병합 BodyText
        let hancom_doc = crate::parser::parse_hwp(&hancom_data).unwrap();
        let mut hancom_cfb = crate::parser::cfb_reader::CfbReader::open(&hancom_data).unwrap();
        let hancom_bt = hancom_cfb.read_body_text_section(0, hancom_doc.header.compressed, false).unwrap();
        let hancom_recs = Record::read_all(&hancom_bt).unwrap();

        eprintln!("원본: {} recs, 한컴 병합: {} recs", orig_recs.len(), hancom_recs.len());

        // 표 범위 찾기
        let find_table = |recs: &[Record]| -> (usize, usize) {
            for (i, rec) in recs.iter().enumerate() {
                if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                    if ctrl_id == crate::parser::tags::CTRL_TABLE {
                        let mut end = recs.len();
                        for j in (i+1)..recs.len() {
                            if recs[j].level <= rec.level { end = j; break; }
                        }
                        return (i, end);
                    }
                }
            }
            (0, 0)
        };

        let (ot_start, ot_end) = find_table(&orig_recs);
        let (ht_start, ht_end) = find_table(&hancom_recs);
        eprintln!("원본 표: [{}..{}] ({} recs)", ot_start, ot_end, ot_end - ot_start);
        eprintln!("한컴 표: [{}..{}] ({} recs)", ht_start, ht_end, ht_end - ht_start);

        // 한컴 표 레코드 전체 출력
        eprintln!("\n=== 한컴 병합 표 레코드 ===");
        for i in ht_start..ht_end {
            let r = &hancom_recs[i];
            let tag = crate::parser::tags::tag_name(r.tag_id);
            eprintln!("  [{}] {} L{} {}B {:02X?}", i, tag, r.level, r.data.len(),
                &r.data[..r.data.len().min(50)]);
        }

        // TABLE 레코드 비교
        eprintln!("\n=== TABLE 레코드 비교 ===");
        for r in orig_recs[ot_start..ot_end].iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_TABLE {
                eprintln!("  원본: {:02X?}", &r.data);
            }
        }
        for r in hancom_recs[ht_start..ht_end].iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_TABLE {
                eprintln!("  한컴: {:02X?}", &r.data);
            }
        }

        // LIST_HEADER 비교 (셀 구조)
        eprintln!("\n=== 원본 LIST_HEADER (row=2 셀들) ===");
        let mut cell_idx = 0;
        for r in orig_recs[ot_start..ot_end].iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER {
                let col = u16::from_le_bytes(r.data[8..10].try_into().unwrap());
                let row = u16::from_le_bytes(r.data[10..12].try_into().unwrap());
                if row == 2 {
                    eprintln!("  cell[{}] col={} row={}: {:02X?}", cell_idx, col, row, &r.data);
                }
                cell_idx += 1;
            }
        }

        eprintln!("\n=== 한컴 LIST_HEADER (row=2 셀들) ===");
        cell_idx = 0;
        for r in hancom_recs[ht_start..ht_end].iter() {
            if r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER {
                let col = u16::from_le_bytes(r.data[8..10].try_into().unwrap());
                let row = u16::from_le_bytes(r.data[10..12].try_into().unwrap());
                let col_span = u16::from_le_bytes(r.data[12..14].try_into().unwrap());
                let row_span = u16::from_le_bytes(r.data[14..16].try_into().unwrap());
                let width = u32::from_le_bytes(r.data[16..20].try_into().unwrap());
                let height = u32::from_le_bytes(r.data[20..24].try_into().unwrap());
                eprintln!("  cell[{}] col={} row={} span={}x{} w={} h={}: {:02X?}",
                    cell_idx, col, row, col_span, row_span, width, height, &r.data);
                cell_idx += 1;
            }
        }

        // 셀 개수 비교
        let orig_cells = orig_recs[ot_start..ot_end].iter()
            .filter(|r| r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER).count();
        let hancom_cells = hancom_recs[ht_start..ht_end].iter()
            .filter(|r| r.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER).count();
        eprintln!("\n셀 개수: 원본={}, 한컴 병합={}", orig_cells, hancom_cells);
    }

    #[test]
    fn test_merge_cells_then_render() {
        let mut doc = create_doc_with_table();
        // 전체 병합
        doc.merge_table_cells_native(0, 0, 0, 0, 0, 1, 1).unwrap();

        // SVG 렌더링 성공 확인
        let svg = doc.render_page_svg_native(0);
        assert!(svg.is_ok());
        assert!(svg.unwrap().contains("<svg"));
    }

    #[test]
    fn test_distribution_raw_stream_preserved() {
        let path = "samples/20250130-hongbo-no.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }
        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        // raw_stream 확인
        let has_raw_before = doc.document().sections[0].raw_stream.is_some();
        eprintln!("raw_stream before convert: {}", has_raw_before);
        assert!(has_raw_before, "파싱 후 raw_stream 있어야 함");

        // 헤더 플래그 확인
        eprintln!("header.flags: 0x{:08X}", doc.document().header.flags);
        eprintln!("header.distribution: {}", doc.document().header.distribution);

        // convert
        let result = doc.convert_to_editable_native().unwrap();
        eprintln!("convert result: {}", result);

        let has_raw_after = doc.document().sections[0].raw_stream.is_some();
        eprintln!("raw_stream after convert: {}", has_raw_after);
        assert!(has_raw_after, "convert 후에도 raw_stream 보존되어야 함");

        // export
        let bytes = doc.export_hwp_native().unwrap();
        eprintln!("export size: {} bytes", bytes.len());

        // 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&bytes).unwrap();
        assert_eq!(doc2.document().sections[0].paragraphs.len(),
                   doc.document().sections[0].paragraphs.len());
        eprintln!("재파싱 문단 수 일치: {}", doc2.document().sections[0].paragraphs.len());
    }

    /// 배포용 문서를 변환 후, raw_stream 없이 재직렬화하는 경로 테스트 (편집 시나리오)
    #[test]
    fn test_distribution_reserialization_without_raw_stream() {
        let path = "samples/20250130-hongbo-no.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }
        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        // 변환
        doc.convert_to_editable_native().unwrap();

        // 편집 시나리오 시뮬레이션: raw_stream 제거
        let orig_para_count = doc.document().sections[0].paragraphs.len();
        doc.document.sections[0].raw_stream = None;
        eprintln!("raw_stream 제거 후 재직렬화 테스트");

        // raw_stream 보존 경로 (기준)
        let data_with_raw = {
            let mut doc2 = HwpDocument::from_bytes(&data).unwrap();
            doc2.convert_to_editable_native().unwrap();
            doc2.export_hwp_native().unwrap()
        };

        // raw_stream 없는 경로 (편집 후)
        let data_without_raw = doc.export_hwp_native().unwrap();

        eprintln!("raw_stream 보존: {} bytes", data_with_raw.len());
        eprintln!("raw_stream 없음: {} bytes", data_without_raw.len());

        // 재직렬화된 파일 파싱 가능 여부
        let doc3 = HwpDocument::from_bytes(&data_without_raw).unwrap();
        let reserialized_para_count = doc3.document().sections[0].paragraphs.len();
        eprintln!("원본 문단: {}, 재직렬화 문단: {}", orig_para_count, reserialized_para_count);

        // BodyText 레코드 수 비교
        use crate::parser::record::Record;
        let mut cfb_with = crate::parser::cfb_reader::CfbReader::open(&data_with_raw).unwrap();
        let bt_with = cfb_with.read_body_text_section(0, true, false).unwrap();
        let recs_with = Record::read_all(&bt_with).unwrap();

        let mut cfb_without = crate::parser::cfb_reader::CfbReader::open(&data_without_raw).unwrap();
        let bt_without = cfb_without.read_body_text_section(0, true, false).unwrap();
        let recs_without = Record::read_all(&bt_without).unwrap();

        eprintln!("raw_stream 보존 레코드: {}, 재직렬화 레코드: {}",
            recs_with.len(), recs_without.len());

        // 재직렬화 결과 파일을 디스크에 저장
        let out_dir = std::path::Path::new("output");
        if out_dir.exists() {
            std::fs::write(out_dir.join("hongbo_with_raw.hwp"), &data_with_raw).unwrap();
            std::fs::write(out_dir.join("hongbo_without_raw.hwp"), &data_without_raw).unwrap();
            eprintln!("저장: output/hongbo_with_raw.hwp, output/hongbo_without_raw.hwp");
        }

        // 레코드 유형별 차이 분석
        use std::collections::HashMap;
        let count_tags = |recs: &[Record]| -> HashMap<u16, usize> {
            let mut map = HashMap::new();
            for r in recs { *map.entry(r.tag_id).or_insert(0) += 1; }
            map
        };
        let tags_with = count_tags(&recs_with);
        let tags_without = count_tags(&recs_without);

        let mut all_tags: Vec<u16> = tags_with.keys().chain(tags_without.keys()).copied().collect();
        all_tags.sort();
        all_tags.dedup();
        for tag in &all_tags {
            let c1 = tags_with.get(tag).unwrap_or(&0);
            let c2 = tags_without.get(tag).unwrap_or(&0);
            if c1 != c2 {
                eprintln!("  태그 차이: {} (0x{:04X}): raw={}, reserialized={}",
                    crate::parser::tags::tag_name(*tag), tag, c1, c2);
            }
        }

        // CTRL_DATA 위치 분석
        for (idx, rec) in recs_with.iter().enumerate() {
            if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_DATA {
                // 부모 CTRL_HEADER 찾기
                let mut parent_info = "?".to_string();
                for prev_idx in (0..idx).rev() {
                    if recs_with[prev_idx].tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER
                       && recs_with[prev_idx].level < rec.level {
                        let data = &recs_with[prev_idx].data;
                        if data.len() >= 4 {
                            let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                            parent_info = format!("{} (0x{:08X})",
                                crate::parser::tags::ctrl_name(ctrl_id), ctrl_id);
                        }
                        break;
                    }
                }
                eprintln!("  CTRL_DATA[{}]: level={}, size={}, parent={}",
                    idx, rec.level, rec.data.len(), parent_info);
            }
        }

        // 인덱스 225~245 주변 레코드 트리 덤프 (level 6 구조 분석)
        eprintln!("\n--- 레코드 트리 (225~250) ---");
        for idx in 225..250.min(recs_with.len()) {
            let rec = &recs_with[idx];
            let indent = "  ".repeat(rec.level as usize);
            let mut extra = String::new();
            if rec.tag_id == crate::parser::tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let cid = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                extra = format!(" ctrl={}", crate::parser::tags::ctrl_name(cid));
            }
            eprintln!("  [{}] {}{}(lv={}, {}B){}",
                idx, indent,
                crate::parser::tags::tag_name(rec.tag_id),
                rec.level, rec.data.len(), extra);
        }

        // 문단 수가 같아야 함
        assert_eq!(reserialized_para_count, orig_para_count,
            "재직렬화 후 문단 수 불일치!");
    }

    /// 재직렬화 vs 원본: BodyText 레코드 상세 비교 (편집 시나리오)
    #[test]
    fn test_web_saved_vs_original_detailed() {
        use crate::parser::record::Record;
        use std::path::Path;

        let orig_path = Path::new("samples/20250130-hongbo.hwp");
        if !orig_path.exists() {
            eprintln!("SKIP: 파일 없음");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();

        // 현재 코드로 재직렬화 (raw_stream 제거 = 편집 시나리오)
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();
        doc.document.sections[0].raw_stream = None;
        let saved_data = doc.export_hwp_native().unwrap();
        eprintln!("원본: {} bytes, 재직렬화: {} bytes", orig_data.len(), saved_data.len());

        // CFB 스트림 목록 비교
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();

        let orig_streams = orig_cfb.list_streams();
        let saved_streams = saved_cfb.list_streams();
        eprintln!("\n=== CFB 스트림 ===");
        eprintln!("원본: {:?}", orig_streams);
        eprintln!("저장: {:?}", saved_streams);

        // FileHeader 비교
        let orig_hdr = orig_cfb.read_file_header().unwrap();
        let saved_hdr = saved_cfb.read_file_header().unwrap();
        if orig_hdr != saved_hdr {
            eprintln!("\n=== FileHeader 차이 ===");
            for i in 0..orig_hdr.len().min(saved_hdr.len()) {
                if orig_hdr[i] != saved_hdr[i] {
                    eprintln!("  offset {}: {:02X} → {:02X}", i, orig_hdr[i], saved_hdr[i]);
                }
            }
        } else {
            eprintln!("\nFileHeader: 동일");
        }

        // DocInfo 비교
        let orig_di = orig_cfb.read_doc_info(true).unwrap();
        let saved_di = saved_cfb.read_doc_info(true).unwrap();
        let orig_di_recs = Record::read_all(&orig_di).unwrap();
        let saved_di_recs = Record::read_all(&saved_di).unwrap();
        eprintln!("\n=== DocInfo ===");
        eprintln!("원본: {} recs ({}B), 저장: {} recs ({}B)",
            orig_di_recs.len(), orig_di.len(), saved_di_recs.len(), saved_di.len());

        // DocInfo 레코드별 비교
        let max_di = orig_di_recs.len().max(saved_di_recs.len());
        let mut di_diffs = 0;
        for i in 0..max_di {
            let o = orig_di_recs.get(i);
            let s = saved_di_recs.get(i);
            match (o, s) {
                (Some(or), Some(sr)) => {
                    if or.tag_id != sr.tag_id || or.data != sr.data {
                        if di_diffs < 20 {
                            eprintln!("  DocInfo[{}] 차이: {} ({}B) vs {} ({}B)",
                                i,
                                crate::parser::tags::tag_name(or.tag_id), or.data.len(),
                                crate::parser::tags::tag_name(sr.tag_id), sr.data.len());
                        }
                        di_diffs += 1;
                    }
                }
                (Some(or), None) => {
                    eprintln!("  DocInfo[{}] 원본만: {} ({}B)", i,
                        crate::parser::tags::tag_name(or.tag_id), or.data.len());
                    di_diffs += 1;
                }
                (None, Some(sr)) => {
                    eprintln!("  DocInfo[{}] 저장만: {} ({}B)", i,
                        crate::parser::tags::tag_name(sr.tag_id), sr.data.len());
                    di_diffs += 1;
                }
                _ => {}
            }
        }
        eprintln!("  DocInfo 차이 레코드 수: {}", di_diffs);

        // BodyText Section0 비교
        let orig_bt = orig_cfb.read_body_text_section(0, true, false).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, true, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();
        eprintln!("\n=== BodyText Section0 ===");
        eprintln!("원본: {} recs ({}B), 저장: {} recs ({}B)",
            orig_recs.len(), orig_bt.len(), saved_recs.len(), saved_bt.len());

        // 레코드별 비교
        let max_bt = orig_recs.len().max(saved_recs.len());
        let mut bt_diffs = 0;
        for i in 0..max_bt {
            let o = orig_recs.get(i);
            let s = saved_recs.get(i);
            match (o, s) {
                (Some(or), Some(sr)) => {
                    if or.tag_id != sr.tag_id || or.level != sr.level || or.data != sr.data {
                        if bt_diffs < 30 {
                            let tag_same = or.tag_id == sr.tag_id;
                            let data_len_diff = or.data.len() as i64 - sr.data.len() as i64;
                            eprintln!("  BT[{}] 차이: {} L{} ({}B) vs {} L{} ({}B) tag_same={} data_diff={}",
                                i,
                                crate::parser::tags::tag_name(or.tag_id), or.level, or.data.len(),
                                crate::parser::tags::tag_name(sr.tag_id), sr.level, sr.data.len(),
                                tag_same, data_len_diff);
                            // 같은 태그인데 데이터만 다른 경우 바이트 비교
                            if tag_same && or.data.len() == sr.data.len() && or.data.len() <= 100 {
                                for j in 0..or.data.len() {
                                    if or.data[j] != sr.data[j] {
                                        eprintln!("    byte[{}]: {:02X} → {:02X}", j, or.data[j], sr.data[j]);
                                    }
                                }
                            }
                        }
                        bt_diffs += 1;
                    }
                }
                (Some(or), None) => {
                    if bt_diffs < 30 {
                        eprintln!("  BT[{}] 원본만: {} L{} ({}B)", i,
                            crate::parser::tags::tag_name(or.tag_id), or.level, or.data.len());
                    }
                    bt_diffs += 1;
                }
                (None, Some(sr)) => {
                    if bt_diffs < 30 {
                        eprintln!("  BT[{}] 저장만: {} L{} ({}B)", i,
                            crate::parser::tags::tag_name(sr.tag_id), sr.level, sr.data.len());
                    }
                    bt_diffs += 1;
                }
                _ => {}
            }
        }
        eprintln!("  BodyText 차이 레코드 수: {}", bt_diffs);

        // BinData 스트림 비교
        eprintln!("\n=== BinData 스트림 비교 ===");
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
        eprintln!("원본 bin_data_content: {} 항목", orig_doc.bin_data_content.len());
        eprintln!("저장 bin_data_content: {} 항목", saved_doc.bin_data_content.len());
        for bc in &orig_doc.bin_data_content {
            let saved_bc = saved_doc.bin_data_content.iter().find(|c| c.id == bc.id);
            match saved_bc {
                Some(sbc) => {
                    if bc.data.len() == sbc.data.len() && bc.data == sbc.data {
                        eprintln!("  ID {}: 동일 ({}B, ext={})", bc.id, bc.data.len(), bc.extension);
                    } else {
                        eprintln!("  ID {}: 크기 차이! 원본={}B, 저장={}B", bc.id, bc.data.len(), sbc.data.len());
                    }
                }
                None => {
                    eprintln!("  ID {}: 저장본에 없음!", bc.id);
                }
            }
        }
    }

    // =====================================================================
    // 클립보드 테스트
    // =====================================================================

    #[test]
    fn test_clipboard_copy_paste_single_paragraph() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        let mut para = Paragraph::default();
        para.text = "Hello World 안녕하세요".to_string();
        para.char_count = para.text.chars().count() as u32 + 1;
        para.char_offsets = para.text.chars().enumerate().map(|(i, _)| i as u32).collect();
        para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        para.line_segs = vec![crate::model::paragraph::LineSeg {
            text_start: 0,
            line_height: 400,
            text_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }];
        para.has_para_text = true;
        document.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.set_document(document);

        // "World" 복사 (offset 6~11)
        let result = doc.copy_selection_native(0, 0, 6, 0, 11);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("World"));

        // 내부 클립보드 확인
        assert!(doc.has_internal_clipboard_native());
        assert_eq!(doc.get_clipboard_text_native(), "World");

        // 문단 끝에 붙여넣기
        let text_len = doc.document.sections[0].paragraphs[0].text.chars().count();
        let result = doc.paste_internal_native(0, 0, text_len);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // 텍스트 확인
        let text = &doc.document.sections[0].paragraphs[0].text;
        assert!(text.contains("Hello World 안녕하세요World"));
    }

    #[test]
    fn test_clipboard_copy_paste_multi_paragraph() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();

        let make_para = |text: &str| {
            let mut p = Paragraph::default();
            p.text = text.to_string();
            p.char_count = text.chars().count() as u32 + 1;
            p.char_offsets = text.chars().enumerate().map(|(i, _)| i as u32).collect();
            p.char_shapes = vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }];
            p.line_segs = vec![crate::model::paragraph::LineSeg {
                text_start: 0,
                line_height: 400,
                text_height: 400,
                baseline_distance: 320,
                ..Default::default()
            }];
            p.has_para_text = true;
            p
        };

        document.sections.push(Section {
            paragraphs: vec![
                make_para("첫 번째 문단"),
                make_para("두 번째 문단"),
                make_para("세 번째 문단"),
            ],
            ..Default::default()
        });
        doc.set_document(document);

        // 첫 번째 문단 3번째 글자부터 두 번째 문단 3번째 글자까지 복사
        let result = doc.copy_selection_native(0, 0, 3, 1, 3);
        assert!(result.is_ok());

        // 클립보드에 2개 문단이 있어야 함
        assert!(doc.has_internal_clipboard_native());
        let clip = doc.clipboard.as_ref().unwrap();
        assert_eq!(clip.paragraphs.len(), 2);

        // 세 번째 문단 끝에 붙여넣기
        let text_len = doc.document.sections[0].paragraphs[2].text.chars().count();
        let result = doc.paste_internal_native(0, 2, text_len);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // 문단 수 증가 확인 (3 → 4: 분할 + 삽입)
        assert_eq!(doc.document.sections[0].paragraphs.len(), 4);
    }

    #[test]
    fn test_clipboard_copy_control() {
        let mut doc = create_doc_with_table();

        // 표 컨트롤 복사
        let result = doc.copy_control_native(0, 0, 0);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("[표]"));

        // 클립보드 확인
        assert!(doc.has_internal_clipboard_native());
        let clip = doc.clipboard.as_ref().unwrap();
        assert_eq!(clip.paragraphs.len(), 1);
        assert_eq!(clip.paragraphs[0].controls.len(), 1);
        assert!(matches!(&clip.paragraphs[0].controls[0], Control::Table(_)));
    }

    #[test]
    fn test_clipboard_clear() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        let mut para = Paragraph::default();
        para.text = "테스트".to_string();
        para.char_count = 4;
        para.char_offsets = vec![0, 1, 2];
        para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
            start_pos: 0, char_shape_id: 0,
        }];
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.set_document(document);

        // 복사
        doc.copy_selection_native(0, 0, 0, 0, 3).unwrap();
        assert!(doc.has_internal_clipboard_native());

        // 초기화
        doc.clear_clipboard_native();
        assert!(!doc.has_internal_clipboard_native());
        assert_eq!(doc.get_clipboard_text_native(), "");
    }

    #[test]
    fn test_clipboard_paste_empty() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        let mut para = Paragraph::default();
        para.text = "테스트".to_string();
        para.char_count = 4;
        para.char_offsets = vec![0, 1, 2];
        para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
            start_pos: 0, char_shape_id: 0,
        }];
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.set_document(document);

        // 클립보드 비어있는 상태에서 붙여넣기
        let result = doc.paste_internal_native(0, 0, 0);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":false"));
    }

    #[test]
    fn test_export_selection_html_basic() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();

        // CharShape 추가 (bold)
        let mut cs = crate::model::style::CharShape::default();
        cs.base_size = 1200; // 12pt
        cs.bold = true;
        document.doc_info.char_shapes.push(cs);

        // ParaShape 추가 (center align)
        let mut ps = crate::model::style::ParaShape::default();
        ps.alignment = crate::model::style::Alignment::Center;
        document.doc_info.para_shapes.push(ps);

        let mut para = Paragraph::default();
        para.text = "Hello World".to_string();
        para.char_count = 12;
        para.char_offsets = (0..11).collect();
        para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        para.para_shape_id = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;

        document.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.set_document(document);

        // HTML 내보내기
        let result = doc.export_selection_html_native(0, 0, 0, 0, 11);
        assert!(result.is_ok());
        let html = result.unwrap();

        // 기본 구조 확인
        assert!(html.contains("<!--StartFragment-->"));
        assert!(html.contains("<!--EndFragment-->"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("<p "));
        assert!(html.contains("<span "));
        assert!(html.contains("text-align:center"));
    }

    #[test]
    fn test_export_selection_html_partial() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();

        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());

        let mut para = Paragraph::default();
        para.text = "ABCDE".to_string();
        para.char_count = 6;
        para.char_offsets = (0..5).collect();
        para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;

        document.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.set_document(document);

        // 부분 선택 (B, C, D)
        let result = doc.export_selection_html_native(0, 0, 1, 0, 4);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(html.contains("BCD"));
        // "ABCDE" 전체 문자열이 포함되지 않아야 함
        assert!(!html.contains("ABCDE"));
        // 정확히 BCD만 span 안에 있는지 확인
        assert!(html.contains(">BCD<"));
    }

    #[test]
    fn test_export_control_html_table() {
        let mut doc = create_doc_with_table();

        let result = doc.export_control_html_native(0, 0, 0);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(html.contains("<table"));
        assert!(html.contains("</table>"));
        assert!(html.contains("<td"));
        assert!(html.contains("<tr>"));
    }

    // === HTML 붙여넣기 테스트 ===

    #[test]
    fn test_paste_html_plain_text() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        let mut para = Paragraph::default();
        para.text = "가나다".to_string();
        para.char_count = para.text.encode_utf16().count() as u32;
        para.char_offsets = para.text.chars()
            .scan(0u32, |acc, c| { let off = *acc; *acc += c.len_utf16() as u32; Some(off) })
            .collect();
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // 플레인 텍스트 HTML 붙여넣기
        let html = "<html><body><!--StartFragment--><p>안녕하세요</p><!--EndFragment--></body></html>";
        let result = doc.paste_html_native(0, 0, 3, html);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // 삽입 후 텍스트 확인
        let text = &doc.document.sections[0].paragraphs[0].text;
        assert!(text.contains("안녕하세요"));
        assert!(text.contains("가나다"));
    }

    #[test]
    fn test_paste_html_styled_text() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        let mut para = Paragraph::default();
        para.text = "테스트".to_string();
        para.char_count = para.text.encode_utf16().count() as u32;
        para.char_offsets = para.text.chars()
            .scan(0u32, |acc, c| { let off = *acc; *acc += c.len_utf16() as u32; Some(off) })
            .collect();
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // 볼드+색상 스타일 HTML
        let html = r#"<html><body><!--StartFragment-->
            <p style="text-align:center;">
                <span style="font-weight:bold;color:#ff0000;">볼드 빨강</span>
            </p>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 0, html);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // CharShape가 추가되었는지 확인 (bold + red color)
        let char_shapes_count = doc.document.doc_info.char_shapes.len();
        assert!(char_shapes_count > 1, "새 CharShape가 생성되어야 함");

        // 볼드 속성 확인
        let bold_shape = doc.document.doc_info.char_shapes.iter()
            .find(|cs| cs.bold);
        assert!(bold_shape.is_some(), "볼드 CharShape가 존재해야 함");
    }

    #[test]
    fn test_paste_html_multi_paragraph() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        let mut para = Paragraph::default();
        para.text = "원본".to_string();
        para.char_count = para.text.encode_utf16().count() as u32;
        para.char_offsets = para.text.chars()
            .scan(0u32, |acc, c| { let off = *acc; *acc += c.len_utf16() as u32; Some(off) })
            .collect();
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // 다중 문단 HTML
        let html = r#"<html><body><!--StartFragment-->
            <p>첫째 문단</p>
            <p>둘째 문단</p>
            <p>셋째 문단</p>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 2, html);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // 문단 수 확인 (원본 1 + 삽입 3 = 최소 3)
        let para_count = doc.document.sections[0].paragraphs.len();
        assert!(para_count >= 3, "최소 3개 문단이어야 함, 실제: {}", para_count);
    }

    #[test]
    fn test_paste_html_table_as_control() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        document.doc_info.border_fills.push(crate::model::style::BorderFill::default());
        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // 2×2 표 HTML
        let html = r#"<html><body><!--StartFragment-->
            <table><tr><td>셀1</td><td>셀2</td></tr><tr><td>셀3</td><td>셀4</td></tr></table>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 0, html);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));

        // Table Control이 삽입되었는지 확인
        let paras = &doc.document.sections[0].paragraphs;
        let table_para = paras.iter().find(|p| !p.controls.is_empty());
        assert!(table_para.is_some(), "Table Control을 포함하는 문단이 있어야 함");

        let table_para = table_para.unwrap();
        assert!(table_para.text.is_empty(), "컨트롤 문단의 text는 비어있어야 함");
        assert_eq!(table_para.controls.len(), 1);

        if let Control::Table(ref tbl) = table_para.controls[0] {
            assert_eq!(tbl.row_count, 2, "행 수 2");
            assert_eq!(tbl.col_count, 2, "열 수 2");
            assert_eq!(tbl.cells.len(), 4, "셀 4개");

            // 셀 내용 확인
            let cell_texts: Vec<String> = tbl.cells.iter()
                .map(|c| c.paragraphs.iter().map(|p| p.text.clone()).collect::<Vec<_>>().join(""))
                .collect();
            assert!(cell_texts.iter().any(|t| t.contains("셀1")), "셀1 포함");
            assert!(cell_texts.iter().any(|t| t.contains("셀2")), "셀2 포함");
            assert!(cell_texts.iter().any(|t| t.contains("셀3")), "셀3 포함");
            assert!(cell_texts.iter().any(|t| t.contains("셀4")), "셀4 포함");

            // 정상 파일 패턴과 일치하는 속성값 검증
            assert_eq!(tbl.attr, 0x082A2311, "table.attr = 0x082A2311");
            assert_eq!(tbl.raw_table_record_attr, 0x04000006, "raw_table_record_attr (DIFF-5: 셀분리금지 항상 설정)");
            assert_eq!(tbl.padding.left, 510, "table padding left");
            assert_eq!(tbl.padding.right, 510, "table padding right");
            assert_eq!(tbl.padding.top, 141, "table padding top");
            assert_eq!(tbl.padding.bottom, 141, "table padding bottom");

            // 셀 속성 검증
            for cell in &tbl.cells {
                assert_eq!(cell.vertical_align, crate::model::table::VerticalAlign::Center,
                    "Cell({},{}) v_align=Center", cell.row, cell.col);
                assert!(cell.raw_list_extra.len() >= 2, "raw_list_extra >= 2 bytes");
            }

            // table_para 속성 검증
            assert_eq!(table_para.char_count, 9, "table para char_count=9");
            assert_eq!(table_para.control_mask, 0x00000800, "control_mask=0x800");
            assert!(table_para.raw_header_extra.len() >= 10, "raw_header_extra >= 10");
            let inst = u32::from_le_bytes([
                table_para.raw_header_extra[6], table_para.raw_header_extra[7],
                table_para.raw_header_extra[8], table_para.raw_header_extra[9],
            ]);
            assert_eq!(inst, 0x80000000, "table para instance_id=0x80000000");

            // DIFF-7: CTRL_HEADER instance_id (raw_ctrl_data[28..32]) 가 0이 아닌지 검증
            assert!(tbl.raw_ctrl_data.len() >= 32, "raw_ctrl_data >= 32 bytes");
            let ctrl_instance_id = u32::from_le_bytes([
                tbl.raw_ctrl_data[28], tbl.raw_ctrl_data[29],
                tbl.raw_ctrl_data[30], tbl.raw_ctrl_data[31],
            ]);
            assert_ne!(ctrl_instance_id, 0, "DIFF-7: CTRL_HEADER instance_id != 0 (got 0x{:08X})", ctrl_instance_id);
        } else {
            panic!("첫 번째 컨트롤이 Table이어야 함");
        }
    }

    /// DIFF-1 검증: &nbsp; 만 있는 빈 셀이 char_count=1, has_para_text=false 인지 확인
    #[test]
    fn test_diff1_empty_cell_nbsp() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        document.doc_info.border_fills.push(crate::model::style::BorderFill::default());
        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        document.sections.push(crate::model::document::Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        doc.document = document;

        // &nbsp; 만 포함된 셀이 있는 2×2 표 (셀2, 셀4는 빈 셀)
        let html = r#"<table><tr><td>내용1</td><td>&nbsp;</td></tr><tr><td>내용2</td><td>&nbsp;&nbsp;&nbsp;</td></tr></table>"#;
        let mut paragraphs = Vec::new();
        doc.parse_table_html(&mut paragraphs, html);

        assert_eq!(paragraphs.len(), 1, "표 문단 1개");
        if let crate::model::control::Control::Table(ref tbl) = paragraphs[0].controls[0] {
            assert_eq!(tbl.cells.len(), 4, "4 셀");
            // 셀[0]: "내용1" → 텍스트 있음
            assert!(!tbl.cells[0].paragraphs[0].text.is_empty(), "셀[0] 텍스트 있음");
            // 셀[1]: &nbsp; → 빈 셀
            let empty1 = &tbl.cells[1].paragraphs[0];
            assert_eq!(empty1.char_count, 1, "DIFF-1: &nbsp; 셀은 char_count=1");
            assert!(empty1.text.is_empty(), "DIFF-1: &nbsp; 셀은 text 비어있음");
            assert!(!empty1.has_para_text, "DIFF-1: &nbsp; 셀은 has_para_text=false");
            // 셀[3]: &nbsp;&nbsp;&nbsp; → 빈 셀
            let empty2 = &tbl.cells[3].paragraphs[0];
            assert_eq!(empty2.char_count, 1, "DIFF-1: 다중 &nbsp; 셀은 char_count=1");
            assert!(empty2.text.is_empty(), "DIFF-1: 다중 &nbsp; 셀은 text 비어있음");
            assert!(!empty2.has_para_text, "DIFF-1: 다중 &nbsp; 셀은 has_para_text=false");
        } else {
            panic!("Table 컨트롤이어야 함");
        }
    }

    #[test]
    fn test_paste_html_table_with_colspan_rowspan() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        document.doc_info.border_fills.push(crate::model::style::BorderFill::default());
        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // colspan=2, rowspan=2 포함 표
        let html = r#"<html><body><!--StartFragment-->
            <table>
                <tr><td colspan="2">병합열</td><td>C</td></tr>
                <tr><td rowspan="2">병합행</td><td>B2</td><td>C2</td></tr>
                <tr><td>B3</td><td>C3</td></tr>
            </table>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 0, html);
        assert!(result.is_ok());

        let paras = &doc.document.sections[0].paragraphs;
        let table_para = paras.iter().find(|p| !p.controls.is_empty());
        assert!(table_para.is_some(), "Table Control 문단이 있어야 함");

        if let Control::Table(ref tbl) = table_para.unwrap().controls[0] {
            assert_eq!(tbl.row_count, 3, "행 수 3");
            assert_eq!(tbl.col_count, 3, "열 수 3");

            // colspan=2인 셀 확인
            let merged_col = tbl.cells.iter().find(|c| c.col_span == 2);
            assert!(merged_col.is_some(), "colspan=2 셀이 있어야 함");
            assert_eq!(merged_col.unwrap().row, 0);

            // rowspan=2인 셀 확인
            let merged_row = tbl.cells.iter().find(|c| c.row_span == 2);
            assert!(merged_row.is_some(), "rowspan=2 셀이 있어야 함");
            assert_eq!(merged_row.unwrap().col, 0);
            assert_eq!(merged_row.unwrap().row, 1);
        } else {
            panic!("Table Control이어야 함");
        }
    }

    #[test]
    fn test_paste_html_table_with_css_styles() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        document.doc_info.border_fills.push(crate::model::style::BorderFill::default());
        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // CSS 스타일 포함 표
        let html = r#"<html><body><!--StartFragment-->
            <table style="border-collapse:collapse;">
                <tr>
                    <td style="width:38.50pt;height:21.31pt;border-top:solid #000000 0.28pt;border-bottom:solid #000000 0.28pt;border-left:solid #000000 0.28pt;border-right:solid #000000 0.28pt;padding:1.41pt 5.10pt;">데이터1</td>
                    <td style="width:50pt;height:21.31pt;background-color:#FFFF00;">데이터2</td>
                </tr>
            </table>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 0, html);
        assert!(result.is_ok());

        let paras = &doc.document.sections[0].paragraphs;
        let table_para = paras.iter().find(|p| !p.controls.is_empty());
        assert!(table_para.is_some());

        if let Control::Table(ref tbl) = table_para.unwrap().controls[0] {
            assert_eq!(tbl.row_count, 1);
            assert_eq!(tbl.col_count, 2);
            assert_eq!(tbl.cells.len(), 2);

            // 첫 번째 셀: width=38.50pt → 3850 HWPUNIT
            let cell0 = &tbl.cells[0];
            assert!(cell0.width > 3800 && cell0.width < 3900,
                "셀 폭 ~3850, 실제: {}", cell0.width);

            // 두 번째 셀: background-color → BorderFill에 등록
            let cell1 = &tbl.cells[1];
            assert!(cell1.border_fill_id > 0, "border_fill_id가 설정되어야 함");

            // 패딩 확인 (1.41pt ≈ 141, 5.10pt ≈ 510)
            assert!(cell0.padding.top > 130 && cell0.padding.top < 150,
                "상단 패딩 ~141, 실제: {}", cell0.padding.top);
            assert!(cell0.padding.left > 500 && cell0.padding.left < 520,
                "좌측 패딩 ~510, 실제: {}", cell0.padding.left);
        } else {
            panic!("Table Control이어야 함");
        }
    }

    #[test]
    fn test_paste_html_table_with_th_header() {
        let mut doc = HwpDocument::create_empty();
        let mut document = Document::default();
        document.doc_info.char_shapes.push(crate::model::style::CharShape::default());
        document.doc_info.para_shapes.push(crate::model::style::ParaShape::default());
        document.doc_info.border_fills.push(crate::model::style::BorderFill::default());
        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 0;
        para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        para.has_para_text = true;
        document.sections.push(Section { paragraphs: vec![para], ..Default::default() });
        doc.set_document(document);

        // <th> 헤더 포함 표
        let html = r#"<html><body><!--StartFragment-->
            <table>
                <tr><th>이름</th><th>나이</th></tr>
                <tr><td>홍길동</td><td>30</td></tr>
            </table>
        <!--EndFragment--></body></html>"#;

        let result = doc.paste_html_native(0, 0, 0, html);
        assert!(result.is_ok());

        let paras = &doc.document.sections[0].paragraphs;
        let table_para = paras.iter().find(|p| !p.controls.is_empty());
        assert!(table_para.is_some());

        if let Control::Table(ref tbl) = table_para.unwrap().controls[0] {
            assert_eq!(tbl.row_count, 2);
            assert_eq!(tbl.col_count, 2);
            assert!(tbl.repeat_header, "헤더 반복 활성화");

            // 첫 행 셀이 is_header=true
            let header_cells: Vec<_> = tbl.cells.iter().filter(|c| c.is_header).collect();
            assert_eq!(header_cells.len(), 2, "헤더 셀 2개");
        } else {
            panic!("Table Control이어야 함");
        }
    }

    #[test]
    fn test_table_utility_functions() {
        // parse_css_dimension_pt
        assert!((super::parse_css_dimension_pt("width:38.50pt", "width") - 38.5).abs() < 0.01);
        assert!((super::parse_css_dimension_pt("width:100px", "width") - 75.0).abs() < 0.01);
        assert!((super::parse_css_dimension_pt("height:1cm", "height") - 28.3465).abs() < 0.1);
        assert_eq!(super::parse_css_dimension_pt("width:auto", "width"), 0.0);

        // parse_css_padding_pt
        let p = super::parse_css_padding_pt("padding:1.41pt 5.10pt");
        assert!((p[0] - 5.10).abs() < 0.01, "left = 5.10"); // left
        assert!((p[1] - 5.10).abs() < 0.01, "right = 5.10"); // right
        assert!((p[2] - 1.41).abs() < 0.01, "top = 1.41"); // top
        assert!((p[3] - 1.41).abs() < 0.01, "bottom = 1.41"); // bottom

        // parse_css_border_shorthand
        let (w, c, s) = super::parse_css_border_shorthand("solid #000000 0.28pt");
        assert!((w - 0.28).abs() < 0.01, "border width 0.28pt");
        assert_eq!(c, 0x000000, "border color black");
        assert_eq!(s, 1, "border style solid");

        let (w2, _, s2) = super::parse_css_border_shorthand("none");
        assert_eq!(w2, 0.0);
        assert_eq!(s2, 0);

        // css_border_width_to_hwp
        assert_eq!(super::css_border_width_to_hwp(0.28), 0); // 0.28pt ≈ 0.1mm → index 0
        assert!(super::css_border_width_to_hwp(1.0) >= 5); // 1.0pt ≈ 0.35mm → index 5+

        // parse_html_attr_u16
        assert_eq!(super::parse_html_attr_u16(r#"<td colspan="3">"#, "colspan"), Some(3));
        assert_eq!(super::parse_html_attr_u16(r#"<td>"#, "colspan"), None);
    }

    #[test]
    fn test_html_utility_functions() {
        // decode_html_entities
        assert_eq!(super::decode_html_entities("&amp;&lt;&gt;"), "&<>");
        assert_eq!(super::decode_html_entities("&nbsp;"), " ");

        // html_strip_tags
        assert_eq!(super::html_strip_tags("<b>bold</b>"), "bold");
        assert_eq!(super::html_strip_tags("<p>text<br/>more</p>"), "textmore");

        // html_to_plain_text
        assert_eq!(super::html_to_plain_text("<p>hello &amp; world</p>"), "hello & world");

        // parse_inline_style
        assert_eq!(
            super::parse_inline_style(r#"<p style="text-align:center;font-size:12pt;">"#),
            "text-align:center;font-size:12pt;"
        );

        // parse_css_value
        assert_eq!(
            super::parse_css_value("text-align:center;font-size:12pt;", "text-align"),
            Some("center".to_string())
        );
        assert_eq!(
            super::parse_css_value("font-size:12pt;", "font-size"),
            Some("12pt".to_string())
        );

        // parse_pt_value
        assert_eq!(super::parse_pt_value("10.0pt"), Some(10.0));
        assert_eq!(super::parse_pt_value("12px"), Some(9.0)); // 12 * 0.75

        // css_color_to_hwp_bgr
        assert_eq!(super::css_color_to_hwp_bgr("#ff0000"), Some(0x0000FF)); // red → BGR
        assert_eq!(super::css_color_to_hwp_bgr("#00ff00"), Some(0x00FF00)); // green
        assert_eq!(super::css_color_to_hwp_bgr("rgb(255, 0, 0)"), Some(0x0000FF));
    }


    /// 우리 편집기 저장 파일의 직렬화→재파싱 라운드트립 검증
    #[test]
    fn test_roundtrip_saved_file() {
        use crate::model::control::Control;
        use crate::serializer::body_text::serialize_section;
        use crate::parser::body_text::parse_body_text_section;

        let path = "/app/pasts/20250130-hongbo_saved-past-005.hwp";
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => { eprintln!("File not found: {}", path); return; }
        };
        let doc = HwpDocument::from_bytes(&data).unwrap();

        for (si, section) in doc.document.sections.iter().enumerate() {
            eprintln!("\n=== Section {} ===", si);
            eprintln!("  Total paragraphs: {}", section.paragraphs.len());

            // 각 문단의 기본 정보 출력
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let ctrl_types: Vec<String> = para.controls.iter().map(|c| match c {
                    Control::Table(t) => format!("Table({}x{})", t.row_count, t.col_count),
                    Control::Picture(_) => "Picture".to_string(),
                    Control::Shape(_) => "Shape".to_string(),
                    Control::SectionDef(_) => "SectionDef".to_string(),
                    Control::ColumnDef(_) => "ColumnDef".to_string(),
                    _ => "Other".to_string(),
                }).collect();
                if !para.controls.is_empty() || para.text.is_empty() {
                    eprintln!("  para[{}]: text={:?} chars={} ctrl_mask=0x{:08X} controls={:?} char_count={} msb={}",
                        pi, &para.text.chars().take(40).collect::<String>(),
                        para.text.len(), para.control_mask, ctrl_types,
                        para.char_count, para.char_count_msb);
                }
            }

            // 직렬화 → 재파싱
            let serialized = serialize_section(section);
            eprintln!("\n  Serialized section {} = {} bytes", si, serialized.len());

            match parse_body_text_section(&serialized) {
                Ok(reparsed) => {
                    eprintln!("  Re-parsed: {} paragraphs", reparsed.paragraphs.len());

                    if reparsed.paragraphs.len() != section.paragraphs.len() {
                        eprintln!("  *** MISMATCH: original {} vs reparsed {} paragraphs ***",
                            section.paragraphs.len(), reparsed.paragraphs.len());
                    }

                    // 각 문단 비교
                    for pi in 0..section.paragraphs.len().min(reparsed.paragraphs.len()) {
                        let orig = &section.paragraphs[pi];
                        let repr = &reparsed.paragraphs[pi];

                        let mut diffs = Vec::new();
                        if orig.char_count != repr.char_count {
                            diffs.push(format!("char_count: {}→{}", orig.char_count, repr.char_count));
                        }
                        if orig.control_mask != repr.control_mask {
                            diffs.push(format!("control_mask: 0x{:08X}→0x{:08X}", orig.control_mask, repr.control_mask));
                        }
                        if orig.controls.len() != repr.controls.len() {
                            diffs.push(format!("controls.len: {}→{}", orig.controls.len(), repr.controls.len()));
                        }
                        if orig.text != repr.text {
                            diffs.push(format!("text differs"));
                        }

                        if !diffs.is_empty() {
                            eprintln!("  *** para[{}] DIFFS: {} ***", pi, diffs.join(", "));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  *** RE-PARSE FAILED: {} ***", e);
                }
            }
        }

        // DocInfo 라운드트립 검증
        eprintln!("\n=== DocInfo Check ===");
        eprintln!("  raw_stream present: {}", doc.document.doc_info.raw_stream.is_some());
        eprintln!("  char_shapes count: {}", doc.document.doc_info.char_shapes.len());
        eprintln!("  para_shapes count: {}", doc.document.doc_info.para_shapes.len());
        eprintln!("  border_fills count: {}", doc.document.doc_info.border_fills.len());

        // 모든 셀 문단의 para_shape_id/char_shape_id 범위 검증
        let max_ps = doc.document.doc_info.para_shapes.len();
        let max_cs = doc.document.doc_info.char_shapes.len();
        let max_bf = doc.document.doc_info.border_fills.len();
        for (si, section) in doc.document.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                if para.para_shape_id as usize >= max_ps {
                    eprintln!("  *** INVALID para[{}] para_shape_id={} >= max {} ***", pi, para.para_shape_id, max_ps);
                }
                for cs in &para.char_shapes {
                    if cs.char_shape_id as usize >= max_cs {
                        eprintln!("  *** INVALID para[{}] char_shape_id={} >= max {} ***", pi, cs.char_shape_id, max_cs);
                    }
                }
                // 셀 문단도 검사
                for ctrl in &para.controls {
                    if let Control::Table(tbl) = ctrl {
                        for (ci, cell) in tbl.cells.iter().enumerate() {
                            if cell.border_fill_id as usize > max_bf {
                                eprintln!("  *** INVALID table para[{}] cell[{}] border_fill_id={} > max {} ***",
                                    pi, ci, cell.border_fill_id, max_bf);
                            }
                            for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                if cp.para_shape_id as usize >= max_ps {
                                    eprintln!("  *** INVALID table para[{}] cell[{}] cp[{}] para_shape_id={} >= max {} ***",
                                        pi, ci, cpi, cp.para_shape_id, max_ps);
                                }
                                for cs in &cp.char_shapes {
                                    if cs.char_shape_id as usize >= max_cs {
                                        eprintln!("  *** INVALID table para[{}] cell[{}] cp[{}] char_shape_id={} >= max {} ***",
                                            pi, ci, cpi, cs.char_shape_id, max_cs);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // DocInfo 직렬화→재파싱 라운드트립
        let serialized_di = crate::serializer::doc_info::serialize_doc_info(
            &doc.document.doc_info, &doc.document.doc_properties);
        eprintln!("  Serialized DocInfo = {} bytes", serialized_di.len());
        match crate::parser::doc_info::parse_doc_info(&serialized_di) {
            Ok((reparsed_di, _)) => {
                eprintln!("  Re-parsed DocInfo: char_shapes={} para_shapes={} border_fills={}",
                    reparsed_di.char_shapes.len(), reparsed_di.para_shapes.len(), reparsed_di.border_fills.len());
                if reparsed_di.char_shapes.len() != doc.document.doc_info.char_shapes.len() {
                    eprintln!("  *** CHAR_SHAPES MISMATCH: {} vs {} ***",
                        doc.document.doc_info.char_shapes.len(), reparsed_di.char_shapes.len());
                }
                if reparsed_di.para_shapes.len() != doc.document.doc_info.para_shapes.len() {
                    eprintln!("  *** PARA_SHAPES MISMATCH: {} vs {} ***",
                        doc.document.doc_info.para_shapes.len(), reparsed_di.para_shapes.len());
                }
                if reparsed_di.border_fills.len() != doc.document.doc_info.border_fills.len() {
                    eprintln!("  *** BORDER_FILLS MISMATCH: {} vs {} ***",
                        doc.document.doc_info.border_fills.len(), reparsed_di.border_fills.len());
                }
            }
            Err(e) => {
                eprintln!("  *** DocInfo RE-PARSE FAILED: {} ***", e);
            }
        }
    }

    /// 정상 파일 vs 손상 파일: 바이너리 레코드 레벨 비교
    #[test]
    fn test_binary_record_comparison() {
        use crate::parser::record::Record;
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::tags;

        let files = [
            ("/app/pasts/20250130-hongbo-p2.hwp", "CORRECT"),
            ("/app/pasts/20250130-hongbo_saved-past-006.hwp", "OURS-006"),
        ];

        for (path, label) in &files {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => { eprintln!("File not found: {}", path); continue; }
            };

            eprintln!("\n{}", "=".repeat(100));
            eprintln!("=== {} : {} ===", label, path);
            eprintln!("{}", "=".repeat(100));

            let mut cfb = CfbReader::open(&data).unwrap();
            let section_data = cfb.read_body_text_section(0, true, false).unwrap();
            let records = Record::read_all(&section_data).unwrap();
            eprintln!("Total records: {}", records.len());

            let ctrl_table_id = tags::CTRL_TABLE;

            for (ri, rec) in records.iter().enumerate() {
                if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    if ctrl_id == ctrl_table_id {
                        eprintln!("\n--- TABLE CTRL_HEADER record #{} (level={}, size={}) ---", ri, rec.level, rec.data.len());
                        for cs in (0..rec.data.len()).step_by(16) {
                            let ce = (cs + 16).min(rec.data.len());
                            let hex: Vec<String> = rec.data[cs..ce].iter().map(|b| format!("{:02X}", b)).collect();
                            eprintln!("    [{:04X}] {}", cs, hex.join(" "));
                        }
                        if rec.data.len() >= 8 {
                            let table_attr = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                            eprintln!("  table.attr = 0x{:08X}", table_attr);
                        }
                        let rcd = &rec.data[8..];
                        if rcd.len() >= 36 {
                            let coa_attr = u32::from_le_bytes([rcd[0], rcd[1], rcd[2], rcd[3]]);
                            let width = u32::from_le_bytes([rcd[12], rcd[13], rcd[14], rcd[15]]);
                            let height = u32::from_le_bytes([rcd[16], rcd[17], rcd[18], rcd[19]]);
                            let z_order = i32::from_le_bytes([rcd[20], rcd[21], rcd[22], rcd[23]]);
                            let margin_l = i16::from_le_bytes([rcd[24], rcd[25]]);
                            let margin_r = i16::from_le_bytes([rcd[26], rcd[27]]);
                            let margin_t = i16::from_le_bytes([rcd[28], rcd[29]]);
                            let margin_b = i16::from_le_bytes([rcd[30], rcd[31]]);
                            let instance_id = u32::from_le_bytes([rcd[32], rcd[33], rcd[34], rcd[35]]);
                            eprintln!("  CommonObjAttr: attr=0x{:08X} w={} h={} z={}", coa_attr, width, height, z_order);
                            eprintln!("    margins=L:{} R:{} T:{} B:{}", margin_l, margin_r, margin_t, margin_b);
                            eprintln!("    instance_id={} (0x{:08X})", instance_id, instance_id);
                            if rcd.len() > 36 {
                                let desc_len = u16::from_le_bytes([rcd[36], rcd[37]]);
                                eprintln!("    desc_len={}, remaining={} bytes", desc_len, rcd.len().saturating_sub(38));
                            }
                        }

                        let tbl_level = rec.level;
                        let mut nr = ri + 1;
                        let mut table_rec_shown = false;
                        let mut cell_count = 0;
                        while nr < records.len() && records[nr].level > tbl_level {
                            let sub = &records[nr];
                            if sub.tag_id == tags::HWPTAG_TABLE && !table_rec_shown {
                                eprintln!("\n  HWPTAG_TABLE record #{} (level={}, size={}):", nr, sub.level, sub.data.len());
                                for cs in (0..sub.data.len()).step_by(16) {
                                    let ce = (cs + 16).min(sub.data.len());
                                    let hex: Vec<String> = sub.data[cs..ce].iter().map(|b| format!("{:02X}", b)).collect();
                                    eprintln!("    [{:04X}] {}", cs, hex.join(" "));
                                }
                                if sub.data.len() >= 18 {
                                    let tbl_attr = u32::from_le_bytes([sub.data[0], sub.data[1], sub.data[2], sub.data[3]]);
                                    let row_cnt = u16::from_le_bytes([sub.data[4], sub.data[5]]);
                                    let col_cnt = u16::from_le_bytes([sub.data[6], sub.data[7]]);
                                    let pad_l = i16::from_le_bytes([sub.data[10], sub.data[11]]);
                                    let pad_r = i16::from_le_bytes([sub.data[12], sub.data[13]]);
                                    let pad_t = i16::from_le_bytes([sub.data[14], sub.data[15]]);
                                    let pad_b = i16::from_le_bytes([sub.data[16], sub.data[17]]);
                                    eprintln!("    attr=0x{:08X} rows={} cols={}", tbl_attr, row_cnt, col_cnt);
                                    eprintln!("    padding=L:{} R:{} T:{} B:{}", pad_l, pad_r, pad_t, pad_b);
                                    let mut off = 18usize;
                                    for _ in 0..row_cnt { off += 2; }
                                    if off + 2 <= sub.data.len() {
                                        let bf_id = u16::from_le_bytes([sub.data[off], sub.data[off+1]]);
                                        eprintln!("    border_fill_id={}", bf_id);
                                        off += 2;
                                    }
                                    if off < sub.data.len() {
                                        let extra: Vec<String> = sub.data[off..].iter().map(|b| format!("{:02X}", b)).collect();
                                        eprintln!("    extra={}", extra.join(" "));
                                    }
                                }
                                table_rec_shown = true;
                            }
                            if sub.tag_id == tags::HWPTAG_LIST_HEADER && cell_count < 2 {
                                cell_count += 1;
                                eprintln!("\n  LIST_HEADER cell #{} record #{} (level={}, size={}):", cell_count, nr, sub.level, sub.data.len());
                                for cs in (0..sub.data.len()).step_by(16) {
                                    let ce = (cs + 16).min(sub.data.len());
                                    let hex: Vec<String> = sub.data[cs..ce].iter().map(|b| format!("{:02X}", b)).collect();
                                    eprintln!("    [{:04X}] {}", cs, hex.join(" "));
                                }
                                if sub.data.len() >= 34 {
                                    let n_p = u16::from_le_bytes([sub.data[0], sub.data[1]]);
                                    let la = u32::from_le_bytes([sub.data[2], sub.data[3], sub.data[4], sub.data[5]]);
                                    let wr = u16::from_le_bytes([sub.data[6], sub.data[7]]);
                                    let col = u16::from_le_bytes([sub.data[8], sub.data[9]]);
                                    let row = u16::from_le_bytes([sub.data[10], sub.data[11]]);
                                    let w = u32::from_le_bytes([sub.data[16], sub.data[17], sub.data[18], sub.data[19]]);
                                    let h = u32::from_le_bytes([sub.data[20], sub.data[21], sub.data[22], sub.data[23]]);
                                    eprintln!("    n_paras={} list_attr=0x{:08X} width_ref={}", n_p, la, wr);
                                    eprintln!("    col={} row={} w={} h={}", col, row, w, h);
                                    if sub.data.len() > 34 {
                                        let extra: Vec<String> = sub.data[34..].iter().map(|b| format!("{:02X}", b)).collect();
                                        eprintln!("    raw_list_extra ({} bytes) = {}", sub.data.len() - 34, extra.join(" "));
                                    }
                                }
                            } else if sub.tag_id == tags::HWPTAG_LIST_HEADER {
                                cell_count += 1;
                            }
                            if sub.tag_id == tags::HWPTAG_PARA_HEADER && cell_count <= 2 && cell_count > 0 {
                                eprintln!("\n  Cell #{} PARA_HEADER record #{} (size={}):", cell_count, nr, sub.data.len());
                                if sub.data.len() >= 22 {
                                    let ccr = u32::from_le_bytes([sub.data[0], sub.data[1], sub.data[2], sub.data[3]]);
                                    let cc = ccr & 0x7FFFFFFF;
                                    let msb = (ccr & 0x80000000) != 0;
                                    let cm = u32::from_le_bytes([sub.data[4], sub.data[5], sub.data[6], sub.data[7]]);
                                    let ps = u16::from_le_bytes([sub.data[8], sub.data[9]]);
                                    let inst = u32::from_le_bytes([sub.data[18], sub.data[19], sub.data[20], sub.data[21]]);
                                    eprintln!("    cc={} msb={} cm=0x{:08X} ps={} inst={}", cc, msb, cm, ps, inst);
                                }
                                for cs in (0..sub.data.len()).step_by(16) {
                                    let ce = (cs + 16).min(sub.data.len());
                                    let hex: Vec<String> = sub.data[cs..ce].iter().map(|b| format!("{:02X}", b)).collect();
                                    eprintln!("    [{:04X}] {}", cs, hex.join(" "));
                                }
                            }
                            nr += 1;
                        }
                        eprintln!("  Total cells: {}", cell_count);
                    }
                }

                // 테이블 포함 문단의 PARA_HEADER (level 0)
                if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.level == 0 {
                    let mut has_table = false;
                    let mut nk = ri + 1;
                    while nk < records.len() && records[nk].level > rec.level {
                        if records[nk].tag_id == tags::HWPTAG_CTRL_HEADER && records[nk].data.len() >= 4 {
                            let cid = u32::from_le_bytes([records[nk].data[0], records[nk].data[1], records[nk].data[2], records[nk].data[3]]);
                            if cid == ctrl_table_id { has_table = true; break; }
                        }
                        nk += 1;
                    }
                    if has_table {
                        eprintln!("\n--- TABLE's PARA_HEADER record #{} (level={}, size={}) ---", ri, rec.level, rec.data.len());
                        for cs in (0..rec.data.len()).step_by(16) {
                            let ce = (cs + 16).min(rec.data.len());
                            let hex: Vec<String> = rec.data[cs..ce].iter().map(|b| format!("{:02X}", b)).collect();
                            eprintln!("    [{:04X}] {}", cs, hex.join(" "));
                        }
                        if rec.data.len() >= 22 {
                            let ccr = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                            let cc = ccr & 0x7FFFFFFF;
                            let msb = (ccr & 0x80000000) != 0;
                            let cm = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                            let ps = u16::from_le_bytes([rec.data[8], rec.data[9]]);
                            let inst = u32::from_le_bytes([rec.data[18], rec.data[19], rec.data[20], rec.data[21]]);
                            eprintln!("  cc={} msb={} cm=0x{:08X} ps_id={} inst={}", cc, msb, cm, ps, inst);
                        }
                    }
                }
            }

            // 레코드 시퀀스 요약 (level 0,1)
            eprintln!("\n--- RECORD SEQUENCE (level 0-1) ---");
            for (ri, rec) in records.iter().enumerate() {
                if rec.level <= 1 {
                    let extra = if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                        let cid = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                        format!(" ctrl_id=0x{:08X}({})", cid, tags::ctrl_name(cid))
                    } else { String::new() };
                    eprintln!("  #{:4}: L{} {} size={}{}", ri, rec.level, rec.tag_name(), rec.data.len(), extra);
                }
            }
        }

        eprintln!("\n=== BINARY RECORD COMPARISON COMPLETE ===");
    }

    /// p2 (표 1개 붙여넣기) vs p3 (표 2개 붙여넣기) DocInfo 비교
    #[test]
    fn test_docinfo_comparison_p2_p3() {
        use crate::parser::record::Record;
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::tags;

        let files = [
            ("/app/pasts/20250130-hongbo-p2.hwp", "P2 (1 table pasted)"),
            ("/app/pasts/20250130-hongbo-p3.hwp", "P3 (2 tables pasted)"),
            ("/app/pasts/20250130-hongbo_saved-past-006.hwp", "OURS-006"),
        ];

        // 각 파일의 DocInfo 레코드를 비교
        let mut all_info: Vec<(String, Vec<(u16, u16, u32, Vec<u8>)>)> = Vec::new();

        for (path, label) in &files {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => { eprintln!("File not found: {}", path); continue; }
            };

            eprintln!("\n{}", "=".repeat(100));
            eprintln!("=== {} : {} ===", label, path);

            let doc = HwpDocument::from_bytes(&data).unwrap();
            let di = &doc.document.doc_info;

            eprintln!("  char_shapes:   {}", di.char_shapes.len());
            eprintln!("  para_shapes:   {}", di.para_shapes.len());
            eprintln!("  border_fills:  {}", di.border_fills.len());
            eprintln!("  bin_data_list: {}", di.bin_data_list.len());
            eprintln!("  styles:        {}", di.styles.len());
            eprintln!("  tab_defs:      {}", di.tab_defs.len());
            eprintln!("  numberings:    {}", di.numberings.len());
            eprintln!("  font_faces:    {} groups", di.font_faces.len());
            for (fi, ff) in di.font_faces.iter().enumerate() {
                if !ff.is_empty() {
                    eprintln!("    font_faces[{}]: {} fonts", fi, ff.len());
                }
            }

            // ID_MAPPINGS: DocInfo 레코드 레벨에서 직접 비교
            let mut cfb = CfbReader::open(&data).unwrap();
            let di_data = cfb.read_doc_info(true).unwrap();
            let records = Record::read_all(&di_data).unwrap();

            eprintln!("\n  DocInfo records: {}", records.len());

            // ID_MAPPINGS 레코드 찾기 (HWPTAG_ID_MAPPINGS = HWPTAG_BEGIN + 2 = 18)
            let id_mappings_tag = 16 + 2; // HWPTAG_BEGIN(16) + 2
            for rec in &records {
                if rec.tag_id == id_mappings_tag {
                    eprintln!("\n  ID_MAPPINGS record (size={}):", rec.data.len());
                    let count = rec.data.len() / 4;
                    let labels = [
                        "BinData", "KorFont", "EnFont", "CnFont", "JpFont",
                        "OtherFont", "SymFont", "UsrFont", "BorderFill", "CharShape",
                        "TabDef", "Numbering", "Bullet", "ParaShape", "Style",
                        "MemoShape", "TrackChange", "TrackChangeUser"
                    ];
                    for i in 0..count.min(18) {
                        let off = i * 4;
                        if off + 4 <= rec.data.len() {
                            let val = u32::from_le_bytes([rec.data[off], rec.data[off+1], rec.data[off+2], rec.data[off+3]]);
                            let name = if i < labels.len() { labels[i] } else { "???" };
                            eprintln!("    [{:2}] {:16} = {}", i, name, val);
                        }
                    }
                }
            }

            // DocInfo 레코드 시퀀스 요약
            let mut rec_summary: std::collections::HashMap<u16, (usize, usize)> = std::collections::HashMap::new();
            for rec in &records {
                let entry = rec_summary.entry(rec.tag_id).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += rec.data.len();
            }
            let mut sorted: Vec<_> = rec_summary.iter().collect();
            sorted.sort_by_key(|(tid, _)| **tid);
            eprintln!("\n  DocInfo record types:");
            for (tid, (cnt, total_size)) in &sorted {
                eprintln!("    tag={:3} ({:20}) count={:4} total_bytes={}", tid, tags::tag_name(**tid), cnt, total_size);
            }

            // 레코드 리스트 저장
            let rec_list: Vec<_> = records.iter().map(|r| (r.tag_id, r.level, r.size, r.data.clone())).collect();
            all_info.push((label.to_string(), rec_list));
        }

        // P2 vs P3 DocInfo 레코드 차이 출력
        if all_info.len() >= 2 {
            let (lbl_a, recs_a) = &all_info[0];
            let (lbl_b, recs_b) = &all_info[1];
            eprintln!("\n{}", "=".repeat(100));
            eprintln!("=== DIFF: {} vs {} ===", lbl_a, lbl_b);
            eprintln!("  {} has {} records, {} has {} records",
                lbl_a, recs_a.len(), lbl_b, recs_b.len());

            let max_len = recs_a.len().max(recs_b.len());
            for i in 0..max_len {
                let a = recs_a.get(i);
                let b = recs_b.get(i);
                match (a, b) {
                    (Some(a), Some(b)) => {
                        if a.0 != b.0 || a.2 != b.2 || a.3 != b.3 {
                            eprintln!("  DIFF rec #{}: {} tag={}/size={} vs {} tag={}/size={}",
                                i,
                                tags::tag_name(a.0), a.0, a.3.len(),
                                tags::tag_name(b.0), b.0, b.3.len());
                            if a.0 == b.0 && a.3.len() == b.3.len() && a.3.len() <= 256 {
                                // 동일 크기면 바이트 단위 차이 출력
                                for j in 0..a.3.len() {
                                    if a.3[j] != b.3[j] {
                                        eprintln!("    byte[{}]: {:02X} vs {:02X}", j, a.3[j], b.3[j]);
                                    }
                                }
                            }
                            if a.0 != b.0 || a.3.len() != b.3.len() {
                                // 완전히 다른 레코드면 hex dump
                                if a.3.len() <= 64 {
                                    let hex_a: Vec<String> = a.3.iter().map(|b| format!("{:02X}", b)).collect();
                                    eprintln!("    A: {}", hex_a.join(" "));
                                }
                                if b.3.len() <= 64 {
                                    let hex_b: Vec<String> = b.3.iter().map(|b| format!("{:02X}", b)).collect();
                                    eprintln!("    B: {}", hex_b.join(" "));
                                }
                            }
                        }
                    },
                    (Some(a), None) => {
                        eprintln!("  ONLY-IN-{}: rec #{} tag={} size={}", lbl_a, i, tags::tag_name(a.0), a.3.len());
                    },
                    (None, Some(b)) => {
                        eprintln!("  ONLY-IN-{}: rec #{} tag={} size={}", lbl_b, i, tags::tag_name(b.0), b.3.len());
                    },
                    _ => {}
                }
            }
        }

        eprintln!("\n=== DOCINFO COMPARISON COMPLETE ===");
    }

    /// DocInfo 라운드트립 테스트: raw_stream 제거 후 직렬화→재파싱 시 데이터 보존 검증
    #[test]
    fn test_docinfo_roundtrip_charshape_preservation() {
        use crate::parser::record::Record;
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::tags;

        // 먼저 모든 관련 파일의 char_shapes 수 출력
        let check_files = [
            "/app/pasts/20250130-hongbo_saved-past.hwp",
            "/app/pasts/20250130-hongbo_saved-past-002.hwp",
            "/app/pasts/20250130-hongbo_saved-past-003.hwp",
            "/app/pasts/20250130-hongbo_saved-past-004.hwp",
            "/app/pasts/20250130-hongbo_saved-past-005.hwp",
            "/app/pasts/20250130-hongbo-p2.hwp",
            "/app/pasts/20250130-hongbo-p3.hwp",
        ];
        eprintln!("\n=== ALL FILES: char_shapes count ===");
        for cf in &check_files {
            if let Ok(d) = std::fs::read(cf) {
                if let Ok(cdoc) = HwpDocument::from_bytes(&d) {
                    eprintln!("  {} → char_shapes={} para_shapes={} border_fills={} styles={}",
                        cf.split('/').last().unwrap_or(cf),
                        cdoc.document.doc_info.char_shapes.len(),
                        cdoc.document.doc_info.para_shapes.len(),
                        cdoc.document.doc_info.border_fills.len(),
                        cdoc.document.doc_info.styles.len());
                }
            }
        }

        let path = "/app/pasts/20250130-hongbo-p2.hwp";
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => { eprintln!("File not found: {}", path); return; }
        };

        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        let orig_cs = doc.document.doc_info.char_shapes.len();
        let orig_ps = doc.document.doc_info.para_shapes.len();
        let orig_bf = doc.document.doc_info.border_fills.len();
        let orig_st = doc.document.doc_info.styles.len();

        eprintln!("=== P2 DocInfo 라운드트립 테스트 ===");
        eprintln!("  Original: char_shapes={} para_shapes={} border_fills={} styles={}",
            orig_cs, orig_ps, orig_bf, orig_st);
        eprintln!("  raw_stream present: {}", doc.document.doc_info.raw_stream.is_some());

        // 1) raw_stream이 있는 경우 → 원본 그대로 반환
        let serialized_raw = crate::serializer::doc_info::serialize_doc_info(
            &doc.document.doc_info, &doc.document.doc_properties);
        let raw_records = Record::read_all(&serialized_raw).unwrap();
        let raw_cs_count = raw_records.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
        eprintln!("  With raw_stream: serialized={} bytes, CHAR_SHAPE records={}", serialized_raw.len(), raw_cs_count);

        // 2) raw_stream 제거 후 재직렬화
        doc.document.doc_info.raw_stream = None;
        let serialized_no_raw = crate::serializer::doc_info::serialize_doc_info(
            &doc.document.doc_info, &doc.document.doc_properties);
        let no_raw_records = Record::read_all(&serialized_no_raw).unwrap();
        let no_raw_cs_count = no_raw_records.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
        eprintln!("  Without raw_stream: serialized={} bytes, CHAR_SHAPE records={}", serialized_no_raw.len(), no_raw_cs_count);

        // 3) 재파싱
        match crate::parser::doc_info::parse_doc_info(&serialized_no_raw) {
            Ok((reparsed_di, reparsed_dp)) => {
                eprintln!("  Re-parsed: char_shapes={} para_shapes={} border_fills={} styles={}",
                    reparsed_di.char_shapes.len(), reparsed_di.para_shapes.len(),
                    reparsed_di.border_fills.len(), reparsed_di.styles.len());

                // 원본과 비교
                if reparsed_di.char_shapes.len() != orig_cs {
                    eprintln!("  *** CHAR_SHAPES LOSS: {} → {} (lost {}) ***",
                        orig_cs, reparsed_di.char_shapes.len(),
                        orig_cs as i64 - reparsed_di.char_shapes.len() as i64);
                }
                if reparsed_di.para_shapes.len() != orig_ps {
                    eprintln!("  *** PARA_SHAPES DIFF: {} → {} ***",
                        orig_ps, reparsed_di.para_shapes.len());
                }
                if reparsed_di.border_fills.len() != orig_bf {
                    eprintln!("  *** BORDER_FILLS DIFF: {} → {} ***",
                        orig_bf, reparsed_di.border_fills.len());
                }
                if reparsed_di.styles.len() != orig_st {
                    eprintln!("  *** STYLES DIFF: {} → {} ***",
                        orig_st, reparsed_di.styles.len());
                }

                assert_eq!(reparsed_di.char_shapes.len(), orig_cs,
                    "char_shapes 라운드트립 불일치!");
            }
            Err(e) => {
                eprintln!("  *** RE-PARSE FAILED: {} ***", e);
                panic!("DocInfo re-parse failed");
            }
        }

        // 4) 레코드 수준 비교: raw_stream vs no_raw_stream
        eprintln!("\n  Record type comparison:");
        let mut raw_by_tag: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
        let mut noraw_by_tag: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
        for r in &raw_records { *raw_by_tag.entry(r.tag_id).or_default() += 1; }
        for r in &no_raw_records { *noraw_by_tag.entry(r.tag_id).or_default() += 1; }

        let mut all_tags: Vec<u16> = raw_by_tag.keys().chain(noraw_by_tag.keys()).cloned().collect();
        all_tags.sort();
        all_tags.dedup();
        for tag in &all_tags {
            let raw_cnt = raw_by_tag.get(tag).unwrap_or(&0);
            let noraw_cnt = noraw_by_tag.get(tag).unwrap_or(&0);
            if raw_cnt != noraw_cnt {
                eprintln!("    tag={} ({}): raw={} vs rebuilt={}",
                    tag, tags::tag_name(*tag), raw_cnt, noraw_cnt);
            }
        }

        // 5) ID_MAPPINGS 상세 덤프
        eprintln!("\n  === ID_MAPPINGS detail (original) ===");
        let labels = [
            "BinData", "KorFont", "EnFont", "CnFont", "JpFont",
            "OtherFont", "SymFont", "UsrFont", "BorderFill", "CharShape",
            "TabDef", "Numbering", "Bullet", "ParaShape", "Style",
            "MemoShape", "TrackChange", "TrackChangeUser"
        ];
        for r in &raw_records {
            if r.tag_id == tags::HWPTAG_ID_MAPPINGS {
                eprintln!("    raw ID_MAPPINGS size={} ({} u32s)", r.data.len(), r.data.len() / 4);
                for i in 0..(r.data.len() / 4).min(18) {
                    let off = i * 4;
                    let val = u32::from_le_bytes([r.data[off], r.data[off+1], r.data[off+2], r.data[off+3]]);
                    let name = if i < labels.len() { labels[i] } else { "???" };
                    eprintln!("      [{:2}] {:16} = {}", i, name, val);
                }
            }
        }
        eprintln!("  === ID_MAPPINGS detail (rebuilt) ===");
        for r in &no_raw_records {
            if r.tag_id == tags::HWPTAG_ID_MAPPINGS {
                eprintln!("    rebuilt ID_MAPPINGS size={} ({} u32s)", r.data.len(), r.data.len() / 4);
                for i in 0..(r.data.len() / 4).min(18) {
                    let off = i * 4;
                    let val = u32::from_le_bytes([r.data[off], r.data[off+1], r.data[off+2], r.data[off+3]]);
                    let name = if i < labels.len() { labels[i] } else { "???" };
                    eprintln!("      [{:2}] {:16} = {}", i, name, val);
                }
            }
        }

        // 6) 원본 DocInfo 레코드별 크기 확인 (CHAR_SHAPE)
        eprintln!("\n  Original CHAR_SHAPE record sizes:");
        let mut cs_sizes: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for r in &raw_records {
            if r.tag_id == tags::HWPTAG_CHAR_SHAPE {
                *cs_sizes.entry(r.data.len()).or_default() += 1;
            }
        }
        for (sz, cnt) in &cs_sizes {
            eprintln!("    size={}: {} records", sz, cnt);
        }

        eprintln!("\n  Rebuilt CHAR_SHAPE record sizes:");
        let mut cs_sizes2: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for r in &no_raw_records {
            if r.tag_id == tags::HWPTAG_CHAR_SHAPE {
                *cs_sizes2.entry(r.data.len()).or_default() += 1;
            }
        }
        for (sz, cnt) in &cs_sizes2 {
            eprintln!("    size={}: {} records", sz, cnt);
        }

        // 7) PARA_SHAPE, BORDER_FILL, STYLE 레코드 크기 비교
        for check_tag in &[tags::HWPTAG_PARA_SHAPE, tags::HWPTAG_BORDER_FILL, tags::HWPTAG_STYLE, tags::HWPTAG_TAB_DEF] {
            let tag_name = tags::tag_name(*check_tag);
            let mut raw_sizes: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            let mut rebuilt_sizes: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            for r in &raw_records {
                if r.tag_id == *check_tag { *raw_sizes.entry(r.data.len()).or_default() += 1; }
            }
            for r in &no_raw_records {
                if r.tag_id == *check_tag { *rebuilt_sizes.entry(r.data.len()).or_default() += 1; }
            }
            if raw_sizes != rebuilt_sizes {
                eprintln!("\n  {} SIZE MISMATCH:", tag_name);
                eprintln!("    Original: {:?}", raw_sizes);
                eprintln!("    Rebuilt:  {:?}", rebuilt_sizes);
            }
        }

        // 8) 전체 레코드 수 비교
        eprintln!("\n  Total records: original={} vs rebuilt={}", raw_records.len(), no_raw_records.len());

        eprintln!("\n=== ROUNDTRIP TEST COMPLETE ===");
    }

    /// 007 파일 vs P2(정상) 파일의 DocInfo 레코드별 크기 비교
    #[test]
    fn test_docinfo_007_vs_correct() {
        use crate::parser::record::Record;
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::tags;

        let files = [
            ("/app/pasts/20250130-hongbo-p2.hwp", "CORRECT(P2)"),
            ("/app/pasts/20250130-hongbo_saved-past-007.hwp", "OURS-007"),
        ];

        for (path, label) in &files {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => { eprintln!("File not found: {}", path); continue; }
            };

            eprintln!("\n{}", "=".repeat(80));
            eprintln!("=== {} : {} ===", label, path);

            let mut cfb = CfbReader::open(&data).unwrap();
            let di_data = cfb.read_doc_info(true).unwrap();
            let records = Record::read_all(&di_data).unwrap();

            eprintln!("  Total DocInfo records: {}", records.len());

            // 각 레코드 타입별 크기 상세 출력
            let tag_order = [
                tags::HWPTAG_DOCUMENT_PROPERTIES,
                tags::HWPTAG_ID_MAPPINGS,
                tags::HWPTAG_BIN_DATA,
                tags::HWPTAG_FACE_NAME,
                tags::HWPTAG_BORDER_FILL,
                tags::HWPTAG_CHAR_SHAPE,
                tags::HWPTAG_TAB_DEF,
                tags::HWPTAG_NUMBERING,
                tags::HWPTAG_PARA_SHAPE,
                tags::HWPTAG_STYLE,
            ];

            for tag in &tag_order {
                let matching: Vec<_> = records.iter().filter(|r| r.tag_id == *tag).collect();
                if matching.is_empty() { continue; }

                let mut sizes: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
                for r in &matching {
                    *sizes.entry(r.data.len()).or_default() += 1;
                }
                let mut sorted_sizes: Vec<_> = sizes.iter().collect();
                sorted_sizes.sort_by_key(|(sz, _)| **sz);

                eprintln!("  {} (tag={}): count={}, sizes={:?}",
                    tags::tag_name(*tag), tag, matching.len(),
                    sorted_sizes.iter().map(|(s, c)| format!("{}b×{}", s, c)).collect::<Vec<_>>().join(", "));
            }

            // 미지원 태그도 출력
            let known_tags: std::collections::HashSet<u16> = tag_order.iter().cloned().collect();
            let mut extra_tags: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
            for r in &records {
                if !known_tags.contains(&r.tag_id) {
                    *extra_tags.entry(r.tag_id).or_default() += 1;
                }
            }
            if !extra_tags.is_empty() {
                let mut sorted_extra: Vec<_> = extra_tags.iter().collect();
                sorted_extra.sort_by_key(|(t, _)| **t);
                for (tag, cnt) in &sorted_extra {
                    eprintln!("  [extra] {} (tag={}): count={}", tags::tag_name(**tag), tag, cnt);
                }
            }

            // ID_MAPPINGS 상세
            let labels = [
                "BinData", "KorFont", "EnFont", "CnFont", "JpFont",
                "OtherFont", "SymFont", "UsrFont", "BorderFill", "CharShape",
                "TabDef", "Numbering", "Bullet", "ParaShape", "Style",
                "MemoShape",
            ];
            for r in &records {
                if r.tag_id == tags::HWPTAG_ID_MAPPINGS {
                    eprintln!("  ID_MAPPINGS ({} bytes, {} u32s):", r.data.len(), r.data.len() / 4);
                    for i in 0..(r.data.len() / 4).min(16) {
                        let off = i * 4;
                        let val = u32::from_le_bytes([r.data[off], r.data[off+1], r.data[off+2], r.data[off+3]]);
                        eprintln!("    [{:2}] {:16} = {}", i, labels[i.min(15)], val);
                    }
                }
            }

            // Style 레코드 상세 (para_shape_id, char_shape_id 확인)
            eprintln!("  Style records detail:");
            let mut style_idx = 0;
            for r in &records {
                if r.tag_id == tags::HWPTAG_STYLE {
                    let hex: String = r.data.iter().take(32).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    eprintln!("    style[{}] size={}: {}", style_idx, r.data.len(), hex);
                    style_idx += 1;
                }
            }
        }

        eprintln!("\n=== 007 vs CORRECT COMPARISON COMPLETE ===");
    }

    /// 원본 HWP 파일과 저장된 HWP 파일의 DocInfo/BodyText 스트림 비교
    #[test]
    fn test_compare_orig_vs_saved() {
        use std::path::Path;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        let saved_path = Path::new("pasts/20250130-hongbo_saved-rp-001.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();

        eprintln!("=== 원본 vs 저장 파일 비교 ===");
        eprintln!("원본 파일 크기: {} bytes", orig_data.len());
        eprintln!("저장 파일 크기: {} bytes", saved_data.len());

        // 1. parse_hwp로 파싱
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        // 2. CfbReader로 raw 스트림 추출
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();

        // DocInfo raw bytes 비교
        let orig_di = orig_cfb.read_doc_info(orig_doc.header.compressed).unwrap();
        let saved_di = saved_cfb.read_doc_info(saved_doc.header.compressed).unwrap();

        eprintln!("\n--- DocInfo 스트림 비교 ---");
        eprintln!("원본 DocInfo: {} bytes", orig_di.len());
        eprintln!("저장 DocInfo: {} bytes", saved_di.len());
        if orig_di == saved_di {
            eprintln!("DocInfo: 동일");
        } else {
            let min_len = orig_di.len().min(saved_di.len());
            let mut first_diff = None;
            let mut diff_count = 0;
            for i in 0..min_len {
                if orig_di[i] != saved_di[i] {
                    if first_diff.is_none() {
                        first_diff = Some(i);
                    }
                    diff_count += 1;
                }
            }
            eprintln!("DocInfo: 차이 발견! 첫 차이 offset={}, 총 바이트 차이={}, 길이 차이={}",
                first_diff.unwrap_or(min_len), diff_count,
                (orig_di.len() as i64 - saved_di.len() as i64).abs());
        }

        // BodyText/Section0 raw bytes 비교
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();

        eprintln!("\n--- BodyText/Section0 스트림 비교 ---");
        eprintln!("원본 BodyText: {} bytes", orig_bt.len());
        eprintln!("저장 BodyText: {} bytes", saved_bt.len());
        if orig_bt == saved_bt {
            eprintln!("BodyText: 동일");
        } else {
            let min_len = orig_bt.len().min(saved_bt.len());
            let mut first_diff = None;
            let mut diff_count = 0;
            for i in 0..min_len {
                if orig_bt[i] != saved_bt[i] {
                    if first_diff.is_none() {
                        first_diff = Some(i);
                    }
                    if diff_count < 10 {
                        eprintln!("  offset {}: orig={:02X} saved={:02X}", i, orig_bt[i], saved_bt[i]);
                    }
                    diff_count += 1;
                }
            }
            eprintln!("BodyText: 첫 차이 offset={}, 총 바이트 차이={}, 길이 차이={}",
                first_diff.unwrap_or(min_len), diff_count,
                (orig_bt.len() as i64 - saved_bt.len() as i64).abs());
        }

        // 3. 문단 및 컨트롤 수 비교
        eprintln!("\n--- 문단/컨트롤 수 비교 ---");
        let orig_paras = &orig_doc.sections[0].paragraphs;
        let saved_paras = &saved_doc.sections[0].paragraphs;
        eprintln!("원본 문단 수: {}", orig_paras.len());
        eprintln!("저장 문단 수: {}", saved_paras.len());

        let orig_ctrl_count: usize = orig_paras.iter().map(|p| p.controls.len()).sum();
        let saved_ctrl_count: usize = saved_paras.iter().map(|p| p.controls.len()).sum();
        eprintln!("원본 컨트롤 수: {}", orig_ctrl_count);
        eprintln!("저장 컨트롤 수: {}", saved_ctrl_count);

        // 원본 컨트롤 목록
        eprintln!("\n--- 원본 파일 컨트롤 목록 ---");
        for (pi, para) in orig_paras.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let ctrl_type = match ctrl {
                    crate::model::control::Control::SectionDef(_) => "SectionDef",
                    crate::model::control::Control::ColumnDef(_) => "ColumnDef",
                    crate::model::control::Control::Table(t) => {
                        eprintln!("  para[{}] ctrl[{}]: Table (rows={}, cols={})",
                            pi, ci, t.row_count, t.col_count);
                        continue;
                    },
                    crate::model::control::Control::Shape(_) => "Shape",
                    crate::model::control::Control::Picture(_) => "Picture",
                    crate::model::control::Control::Header(_) => "Header",
                    crate::model::control::Control::Footer(_) => "Footer",
                    crate::model::control::Control::Footnote(_) => "Footnote",
                    crate::model::control::Control::Endnote(_) => "Endnote",
                    crate::model::control::Control::AutoNumber(_) => "AutoNumber",
                    crate::model::control::Control::NewNumber(_) => "NewNumber",
                    crate::model::control::Control::PageNumberPos(_) => "PageNumberPos",
                    crate::model::control::Control::Bookmark(_) => "Bookmark",
                    crate::model::control::Control::Hyperlink(_) => "Hyperlink",
                    crate::model::control::Control::Ruby(_) => "Ruby",
                    crate::model::control::Control::CharOverlap(_) => "CharOverlap",
                    crate::model::control::Control::PageHide(_) => "PageHide",
                    crate::model::control::Control::HiddenComment(_) => "HiddenComment",
                    crate::model::control::Control::Equation(_) => "Equation",
                    crate::model::control::Control::Field(_) => "Field",
                    crate::model::control::Control::Form(_) => "Form",
                    crate::model::control::Control::Unknown(u) => {
                        eprintln!("  para[{}] ctrl[{}]: Unknown (ctrl_id=0x{:08X})", pi, ci, u.ctrl_id);
                        continue;
                    },
                };
                eprintln!("  para[{}] ctrl[{}]: {}", pi, ci, ctrl_type);
            }
        }

        // 저장 파일 컨트롤 목록
        eprintln!("\n--- 저장 파일 컨트롤 목록 ---");
        for (pi, para) in saved_paras.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let ctrl_type = match ctrl {
                    crate::model::control::Control::SectionDef(_) => "SectionDef",
                    crate::model::control::Control::ColumnDef(_) => "ColumnDef",
                    crate::model::control::Control::Table(t) => {
                        eprintln!("  para[{}] ctrl[{}]: Table (rows={}, cols={})",
                            pi, ci, t.row_count, t.col_count);
                        continue;
                    },
                    crate::model::control::Control::Shape(_) => "Shape",
                    crate::model::control::Control::Picture(_) => "Picture",
                    crate::model::control::Control::Header(_) => "Header",
                    crate::model::control::Control::Footer(_) => "Footer",
                    crate::model::control::Control::Footnote(_) => "Footnote",
                    crate::model::control::Control::Endnote(_) => "Endnote",
                    crate::model::control::Control::AutoNumber(_) => "AutoNumber",
                    crate::model::control::Control::NewNumber(_) => "NewNumber",
                    crate::model::control::Control::PageNumberPos(_) => "PageNumberPos",
                    crate::model::control::Control::Bookmark(_) => "Bookmark",
                    crate::model::control::Control::Hyperlink(_) => "Hyperlink",
                    crate::model::control::Control::Ruby(_) => "Ruby",
                    crate::model::control::Control::CharOverlap(_) => "CharOverlap",
                    crate::model::control::Control::PageHide(_) => "PageHide",
                    crate::model::control::Control::HiddenComment(_) => "HiddenComment",
                    crate::model::control::Control::Equation(_) => "Equation",
                    crate::model::control::Control::Field(_) => "Field",
                    crate::model::control::Control::Form(_) => "Form",
                    crate::model::control::Control::Unknown(u) => {
                        eprintln!("  para[{}] ctrl[{}]: Unknown (ctrl_id=0x{:08X})", pi, ci, u.ctrl_id);
                        continue;
                    },
                };
                eprintln!("  para[{}] ctrl[{}]: {}", pi, ci, ctrl_type);
            }
        }

        // 4. 저장 파일 Section0의 마지막 20개 레코드 분석
        eprintln!("\n--- 저장 파일 Section0: 마지막 20개 레코드 ---");
        let saved_recs = Record::read_all(&saved_bt).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();
        eprintln!("원본 레코드 수: {}", orig_recs.len());
        eprintln!("저장 레코드 수: {}", saved_recs.len());

        let start = if saved_recs.len() > 20 { saved_recs.len() - 20 } else { 0 };
        for i in start..saved_recs.len() {
            let r = &saved_recs[i];
            let tag = tags::tag_name(r.tag_id);
            // CTRL_HEADER인 경우 ctrl_id 표시
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                let ctrl = tags::ctrl_name(ctrl_id);
                eprintln!("  [{}] {} L{} {}B ctrl={} {:02X?}",
                    i, tag, r.level, r.data.len(), ctrl,
                    &r.data[..r.data.len().min(32)]);
            } else {
                eprintln!("  [{}] {} L{} {}B {:02X?}",
                    i, tag, r.level, r.data.len(),
                    &r.data[..r.data.len().min(32)]);
            }
        }

        // 원본의 마지막 20개 레코드도 비교
        eprintln!("\n--- 원본 파일 Section0: 마지막 20개 레코드 ---");
        let start = if orig_recs.len() > 20 { orig_recs.len() - 20 } else { 0 };
        for i in start..orig_recs.len() {
            let r = &orig_recs[i];
            let tag = tags::tag_name(r.tag_id);
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                let ctrl = tags::ctrl_name(ctrl_id);
                eprintln!("  [{}] {} L{} {}B ctrl={} {:02X?}",
                    i, tag, r.level, r.data.len(), ctrl,
                    &r.data[..r.data.len().min(32)]);
            } else {
                eprintln!("  [{}] {} L{} {}B {:02X?}",
                    i, tag, r.level, r.data.len(),
                    &r.data[..r.data.len().min(32)]);
            }
        }

        // 레코드 전체 비교: 첫 차이 위치 찾기
        eprintln!("\n--- 레코드 비교 (전체) ---");
        let max_recs = orig_recs.len().max(saved_recs.len());
        let mut first_rec_diff = None;
        let mut total_diffs = 0;
        for i in 0..max_recs {
            if i >= orig_recs.len() {
                if first_rec_diff.is_none() { first_rec_diff = Some(i); }
                total_diffs += 1;
                if total_diffs <= 15 {
                    eprintln!("  [{}] 원본에 없음, 저장: {} L{} {}B",
                        i, tags::tag_name(saved_recs[i].tag_id), saved_recs[i].level, saved_recs[i].data.len());
                }
                continue;
            }
            if i >= saved_recs.len() {
                if first_rec_diff.is_none() { first_rec_diff = Some(i); }
                total_diffs += 1;
                if total_diffs <= 15 {
                    eprintln!("  [{}] 저장에 없음, 원본: {} L{} {}B",
                        i, tags::tag_name(orig_recs[i].tag_id), orig_recs[i].level, orig_recs[i].data.len());
                }
                continue;
            }
            let o = &orig_recs[i];
            let s = &saved_recs[i];
            if o.tag_id != s.tag_id || o.level != s.level || o.data != s.data {
                if first_rec_diff.is_none() { first_rec_diff = Some(i); }
                total_diffs += 1;
                if total_diffs <= 15 {
                    let otag = tags::tag_name(o.tag_id);
                    let stag = tags::tag_name(s.tag_id);
                    eprintln!("  [{}] 차이:", i);
                    eprintln!("    원본: {} L{} {}B {:02X?}", otag, o.level, o.data.len(), &o.data[..o.data.len().min(40)]);
                    eprintln!("    저장: {} L{} {}B {:02X?}", stag, s.level, s.data.len(), &s.data[..s.data.len().min(40)]);
                }
            }
        }
        eprintln!("첫 차이 레코드 인덱스: {:?}, 총 차이 레코드: {}", first_rec_diff, total_diffs);
        if total_diffs > 15 {
            eprintln!("  (15개 이후 생략, 총 {}개 차이)", total_diffs);
        }

        eprintln!("\n=== 원본 vs 저장 파일 비교 완료 ===");
    }


    #[test]
    fn test_pasted_table_structure_analysis() {
        use std::path::Path;
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::cfb_reader::CfbReader;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        let saved_path = Path::new("pasts/20250130-hongbo_saved-rp-003.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();

        // Parse file headers to get compression flags
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== PASTED TABLE STRUCTURE ANALYSIS ===");
        eprintln!("Original: {} ({} bytes)", orig_path.display(), orig_data.len());
        eprintln!("Saved:    {} ({} bytes)", saved_path.display(), saved_data.len());
        eprintln!("{}", "=".repeat(120));

        // Read raw Section0 bytes
        let mut orig_cfb = CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb = CfbReader::open(&saved_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();

        let orig_recs = Record::read_all(&orig_bt).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\nOriginal records: {}", orig_recs.len());
        eprintln!("Saved records:    {}", saved_recs.len());

        // Helper: hex dump with printable ASCII
        fn hex_dump(data: &[u8], max_bytes: usize) -> String {
            let show = data.len().min(max_bytes);
            let mut s = String::new();
            for (i, chunk) in data[..show].chunks(16).enumerate() {
                s.push_str(&format!("    {:04X}: ", i * 16));
                for b in chunk {
                    s.push_str(&format!("{:02X} ", b));
                }
                // Pad for alignment
                for _ in 0..(16 - chunk.len()) {
                    s.push_str("   ");
                }
                s.push_str(" |");
                for b in chunk {
                    if *b >= 0x20 && *b < 0x7F {
                        s.push(*b as char);
                    } else {
                        s.push('.');
                    }
                }
                s.push_str("|\n");
            }
            if data.len() > max_bytes {
                s.push_str(&format!("    ... ({} more bytes)\n", data.len() - max_bytes));
            }
            s
        }

        // Helper: read ctrl_id from CTRL_HEADER record data
        fn get_ctrl_id(data: &[u8]) -> u32 {
            if data.len() >= 4 {
                u32::from_le_bytes([data[0], data[1], data[2], data[3]])
            } else {
                0
            }
        }

        // Helper: check if ctrl_id is table ("tbl " = 0x74626C20 big-endian)
        // In file: DWORD LE → bytes [0x20, 0x6C, 0x62, 0x74]
        // u32::from_le_bytes gives 0x74626C20
        fn is_table_ctrl(data: &[u8]) -> bool {
            if data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                ctrl_id == tags::CTRL_TABLE
            } else {
                false
            }
        }

        // Struct to hold a table's record cluster
        struct TableCluster {
            ctrl_header_idx: usize,
            ctrl_header: Record,
            table_rec: Option<(usize, Record)>,
            list_headers: Vec<(usize, Record)>,
            para_headers: Vec<(usize, Record)>,
            // All records in this table's scope
            all_records: Vec<(usize, Record)>,
        }

        // Find table clusters in a record list
        fn find_table_clusters(recs: &[Record]) -> Vec<TableCluster> {
            let mut clusters = Vec::new();
            let mut i = 0;
            while i < recs.len() {
                if recs[i].tag_id == tags::HWPTAG_CTRL_HEADER && is_table_ctrl(&recs[i].data) {
                    let ctrl_level = recs[i].level;
                    let mut cluster = TableCluster {
                        ctrl_header_idx: i,
                        ctrl_header: recs[i].clone(),
                        table_rec: None,
                        list_headers: Vec::new(),
                        para_headers: Vec::new(),
                        all_records: vec![(i, recs[i].clone())],
                    };
                    // Collect all child records (level > ctrl_level)
                    let mut j = i + 1;
                    while j < recs.len() && recs[j].level > ctrl_level {
                        cluster.all_records.push((j, recs[j].clone()));
                        if recs[j].tag_id == tags::HWPTAG_TABLE && cluster.table_rec.is_none() {
                            cluster.table_rec = Some((j, recs[j].clone()));
                        }
                        if recs[j].tag_id == tags::HWPTAG_LIST_HEADER {
                            cluster.list_headers.push((j, recs[j].clone()));
                        }
                        if recs[j].tag_id == tags::HWPTAG_PARA_HEADER {
                            cluster.para_headers.push((j, recs[j].clone()));
                        }
                        j += 1;
                    }
                    clusters.push(cluster);
                    i = j;
                } else {
                    i += 1;
                }
            }
            clusters
        }

        // Debug: list all CTRL_HEADER records and their ctrl_ids
        eprintln!("\n--- All CTRL_HEADER records in Original ---");
        for (i, r) in orig_recs.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                let ctrl_bytes = &r.data[0..4];
                let is_tbl = ctrl_id == tags::CTRL_TABLE;
                eprintln!("  [{}] CTRL_HEADER L{} {}B ctrl_id=0x{:08X} bytes=[{:02X} {:02X} {:02X} {:02X}] name={} is_table={}",
                    i, r.level, r.data.len(), ctrl_id,
                    ctrl_bytes[0], ctrl_bytes[1], ctrl_bytes[2], ctrl_bytes[3],
                    tags::ctrl_name(ctrl_id), is_tbl);
            }
        }
        eprintln!("\n--- All CTRL_HEADER records in Saved ---");
        for (i, r) in saved_recs.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                let ctrl_bytes = &r.data[0..4];
                let is_tbl = ctrl_id == tags::CTRL_TABLE;
                eprintln!("  [{}] CTRL_HEADER L{} {}B ctrl_id=0x{:08X} bytes=[{:02X} {:02X} {:02X} {:02X}] name={} is_table={}",
                    i, r.level, r.data.len(), ctrl_id,
                    ctrl_bytes[0], ctrl_bytes[1], ctrl_bytes[2], ctrl_bytes[3],
                    tags::ctrl_name(ctrl_id), is_tbl);
            }
        }

        let orig_tables = find_table_clusters(&orig_recs);
        let saved_tables = find_table_clusters(&saved_recs);

        eprintln!("\n--- Table Count ---");
        eprintln!("Original tables: {}", orig_tables.len());
        eprintln!("Saved tables:    {}", saved_tables.len());

        // Print summary of each table
        fn print_table_summary(label: &str, tables: &[TableCluster]) {
            eprintln!("\n--- {} Table Summary ---", label);
            for (ti, t) in tables.iter().enumerate() {
                let ctrl_id = get_ctrl_id(&t.ctrl_header.data);
                let ctrl_id_bytes = ctrl_id.to_le_bytes();
                let ctrl_str: String = ctrl_id_bytes.iter().rev().map(|b| {
                    if *b >= 0x20 && *b < 0x7F { *b as char } else { '?' }
                }).collect();
                eprintln!("  Table[{}] at rec[{}]: ctrl_id=0x{:08X} '{}' level={} total_children={} cells(LIST_HEADER)={} paras(PARA_HEADER)={}",
                    ti, t.ctrl_header_idx, ctrl_id, ctrl_str,
                    t.ctrl_header.level,
                    t.all_records.len() - 1,
                    t.list_headers.len(),
                    t.para_headers.len()
                );
                // Table record info
                if let Some((idx, ref tr)) = t.table_rec {
                    eprintln!("    TABLE record at [{}]: size={} bytes", idx, tr.data.len());
                    if tr.data.len() >= 8 {
                        let flags = u32::from_le_bytes(tr.data[0..4].try_into().unwrap());
                        let nrows = u16::from_le_bytes(tr.data[4..6].try_into().unwrap());
                        let ncols = u16::from_le_bytes(tr.data[6..8].try_into().unwrap());
                        eprintln!("    TABLE: flags=0x{:08X} nrows={} ncols={} (expected cells={})",
                            flags, nrows, ncols, nrows as u32 * ncols as u32);
                    }
                } else {
                    eprintln!("    TABLE record: MISSING!");
                }
            }
        }

        print_table_summary("Original", &orig_tables);
        print_table_summary("Saved", &saved_tables);

        // Detailed analysis of each table
        fn dump_table_detail(label: &str, t: &TableCluster) {
            eprintln!("\n  === {} Table at rec[{}] DETAILED ===", label, t.ctrl_header_idx);

            // 1. CTRL_HEADER full dump
            eprintln!("\n  [CTRL_HEADER] rec[{}] level={} size={} bytes:",
                t.ctrl_header_idx, t.ctrl_header.level, t.ctrl_header.data.len());
            eprintln!("{}", hex_dump(&t.ctrl_header.data, 256));

            // Parse CTRL_HEADER fields
            if t.ctrl_header.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes(t.ctrl_header.data[0..4].try_into().unwrap());
                eprintln!("    ctrl_id = 0x{:08X}", ctrl_id);
            }
            if t.ctrl_header.data.len() >= 8 {
                let obj_attr = u32::from_le_bytes(t.ctrl_header.data[4..8].try_into().unwrap());
                eprintln!("    obj_attr = 0x{:08X}", obj_attr);
                let vert_offset = (obj_attr >> 0) & 0x3;
                let horiz_offset = (obj_attr >> 2) & 0x3;
                let vert_rel = (obj_attr >> 4) & 0x3;
                let horiz_rel = (obj_attr >> 7) & 0x3;
                let flow_with_text = (obj_attr >> 10) & 0x1;
                let allow_overlap = (obj_attr >> 11) & 0x1;
                let wid_criterion = (obj_attr >> 12) & 0x7;
                let hgt_criterion = (obj_attr >> 15) & 0x3;
                let protect_size = (obj_attr >> 17) & 0x1;
                let text_flow = (obj_attr >> 21) & 0x7;
                let text_arrange = (obj_attr >> 24) & 0x3;
                eprintln!("      vert_offset={} horiz_offset={} vert_rel={} horiz_rel={}", vert_offset, horiz_offset, vert_rel, horiz_rel);
                eprintln!("      flow_with_text={} allow_overlap={} wid_criterion={} hgt_criterion={}", flow_with_text, allow_overlap, wid_criterion, hgt_criterion);
                eprintln!("      protect_size={} text_flow={} text_arrange={}", protect_size, text_flow, text_arrange);
            }
            if t.ctrl_header.data.len() >= 12 {
                let vert_pos = u32::from_le_bytes(t.ctrl_header.data[8..12].try_into().unwrap());
                eprintln!("    vert_offset_value = {} hwpunit", vert_pos);
            }
            if t.ctrl_header.data.len() >= 16 {
                let horiz_pos = u32::from_le_bytes(t.ctrl_header.data[12..16].try_into().unwrap());
                eprintln!("    horiz_offset_value = {} hwpunit", horiz_pos);
            }
            if t.ctrl_header.data.len() >= 20 {
                let width = u32::from_le_bytes(t.ctrl_header.data[16..20].try_into().unwrap());
                eprintln!("    width = {} hwpunit ({:.2} mm)", width, width as f64 / 7200.0 * 25.4);
            }
            if t.ctrl_header.data.len() >= 24 {
                let height = u32::from_le_bytes(t.ctrl_header.data[20..24].try_into().unwrap());
                eprintln!("    height = {} hwpunit ({:.2} mm)", height, height as f64 / 7200.0 * 25.4);
            }
            if t.ctrl_header.data.len() >= 28 {
                let zorder = i32::from_le_bytes(t.ctrl_header.data[24..28].try_into().unwrap());
                eprintln!("    z_order = {}", zorder);
            }

            // 2. TABLE record full dump
            if let Some((idx, ref tr)) = t.table_rec {
                eprintln!("\n  [TABLE] rec[{}] level={} size={} bytes:",
                    idx, tr.level, tr.data.len());
                eprintln!("{}", hex_dump(&tr.data, 512));

                // Parse TABLE fields
                if tr.data.len() >= 8 {
                    let flags = u32::from_le_bytes(tr.data[0..4].try_into().unwrap());
                    let nrows = u16::from_le_bytes(tr.data[4..6].try_into().unwrap());
                    let ncols = u16::from_le_bytes(tr.data[6..8].try_into().unwrap());
                    eprintln!("    flags=0x{:08X} nrows={} ncols={}", flags, nrows, ncols);

                    // Cell spacing (4 bytes)
                    if tr.data.len() >= 12 {
                        let cell_spacing = u32::from_le_bytes(tr.data[8..12].try_into().unwrap());
                        eprintln!("    cell_spacing={}", cell_spacing);
                    }

                    // Margins: left, right, top, bottom (each 2 bytes)
                    if tr.data.len() >= 20 {
                        let ml = u16::from_le_bytes(tr.data[12..14].try_into().unwrap());
                        let mr = u16::from_le_bytes(tr.data[14..16].try_into().unwrap());
                        let mt = u16::from_le_bytes(tr.data[16..18].try_into().unwrap());
                        let mb = u16::from_le_bytes(tr.data[18..20].try_into().unwrap());
                        eprintln!("    margins: left={} right={} top={} bottom={}", ml, mr, mt, mb);
                    }

                    // Row sizes (nrows * 2 bytes starting at offset 20)
                    let row_sizes_offset = 20;
                    let row_sizes_end = row_sizes_offset + (nrows as usize * 2);
                    if tr.data.len() >= row_sizes_end {
                        let mut row_sizes = Vec::new();
                        for r in 0..nrows as usize {
                            let off = row_sizes_offset + r * 2;
                            let rs = u16::from_le_bytes(tr.data[off..off+2].try_into().unwrap());
                            row_sizes.push(rs);
                        }
                        eprintln!("    row_sizes: {:?}", row_sizes);
                    }

                    // Border fill ID after row sizes
                    let bf_offset = row_sizes_end;
                    if tr.data.len() >= bf_offset + 2 {
                        let bf_id = u16::from_le_bytes(tr.data[bf_offset..bf_offset+2].try_into().unwrap());
                        eprintln!("    border_fill_id={}", bf_id);
                    }

                    // Remaining bytes after all parsed fields
                    let parsed_end = bf_offset + 2;
                    if tr.data.len() > parsed_end {
                        eprintln!("    remaining {} bytes after parsed fields:", tr.data.len() - parsed_end);
                        eprintln!("{}", hex_dump(&tr.data[parsed_end..], 128));
                    }
                }
            }

            // 3. LIST_HEADER records (cells) - first 5
            let cell_count = t.list_headers.len().min(5);
            eprintln!("\n  [LIST_HEADER / Cells] total={}, showing first {}:", t.list_headers.len(), cell_count);
            for ci in 0..cell_count {
                let (idx, ref lh) = t.list_headers[ci];
                eprintln!("\n    Cell[{}] LIST_HEADER rec[{}] level={} size={} bytes:", ci, idx, lh.level, lh.data.len());
                eprintln!("{}", hex_dump(&lh.data, 256));

                // Parse LIST_HEADER cell fields
                if lh.data.len() >= 2 {
                    let num_paras = u16::from_le_bytes(lh.data[0..2].try_into().unwrap());
                    eprintln!("      num_paras = {}", num_paras);
                }
                if lh.data.len() >= 6 {
                    let prop = u32::from_le_bytes(lh.data[2..6].try_into().unwrap());
                    eprintln!("      property = 0x{:08X}", prop);
                }
                // Cell-specific fields (after list header base)
                // The cell list header typically has: nParagraphs(2), property(4),
                // then cell-specific: col_addr(2), row_addr(2), col_span(2), row_span(2),
                // width(4), height(4), margins(4*2=8), border_fill_id(2)
                if lh.data.len() >= 8 {
                    let col_addr = u16::from_le_bytes(lh.data[6..8].try_into().unwrap());
                    eprintln!("      col_addr = {}", col_addr);
                }
                if lh.data.len() >= 10 {
                    let row_addr = u16::from_le_bytes(lh.data[8..10].try_into().unwrap());
                    eprintln!("      row_addr = {}", row_addr);
                }
                if lh.data.len() >= 12 {
                    let col_span = u16::from_le_bytes(lh.data[10..12].try_into().unwrap());
                    eprintln!("      col_span = {}", col_span);
                }
                if lh.data.len() >= 14 {
                    let row_span = u16::from_le_bytes(lh.data[12..14].try_into().unwrap());
                    eprintln!("      row_span = {}", row_span);
                }
                if lh.data.len() >= 18 {
                    let width = u32::from_le_bytes(lh.data[14..18].try_into().unwrap());
                    eprintln!("      width = {} hwpunit ({:.2} mm)", width, width as f64 / 7200.0 * 25.4);
                }
                if lh.data.len() >= 22 {
                    let height = u32::from_le_bytes(lh.data[18..22].try_into().unwrap());
                    eprintln!("      height = {} hwpunit ({:.2} mm)", height, height as f64 / 7200.0 * 25.4);
                }
                // Margins
                if lh.data.len() >= 30 {
                    let ml = u16::from_le_bytes(lh.data[22..24].try_into().unwrap());
                    let mr = u16::from_le_bytes(lh.data[24..26].try_into().unwrap());
                    let mt = u16::from_le_bytes(lh.data[26..28].try_into().unwrap());
                    let mb = u16::from_le_bytes(lh.data[28..30].try_into().unwrap());
                    eprintln!("      margins: left={} right={} top={} bottom={}", ml, mr, mt, mb);
                }
                if lh.data.len() >= 32 {
                    let bf_id = u16::from_le_bytes(lh.data[30..32].try_into().unwrap());
                    eprintln!("      border_fill_id = {}", bf_id);
                }
                if lh.data.len() > 32 {
                    eprintln!("      remaining {} bytes:", lh.data.len() - 32);
                    eprintln!("{}", hex_dump(&lh.data[32..], 64));
                }
            }

            // 4. PARA_HEADER records - first 5
            let para_count = t.para_headers.len().min(5);
            eprintln!("\n  [PARA_HEADER] total={}, showing first {}:", t.para_headers.len(), para_count);
            for pi in 0..para_count {
                let (idx, ref ph) = t.para_headers[pi];
                eprintln!("\n    Para[{}] PARA_HEADER rec[{}] level={} size={} bytes:", pi, idx, ph.level, ph.data.len());
                eprintln!("{}", hex_dump(&ph.data, 128));

                // Parse PARA_HEADER fields
                if ph.data.len() >= 4 {
                    let nchars = u32::from_le_bytes(ph.data[0..4].try_into().unwrap());
                    let n_char_shapes = if ph.data.len() >= 6 { u16::from_le_bytes(ph.data[4..6].try_into().unwrap()) } else { 0 };
                    let n_line_segs = if ph.data.len() >= 8 { u16::from_le_bytes(ph.data[6..8].try_into().unwrap()) } else { 0 };
                    let n_range_tags = if ph.data.len() >= 10 { u16::from_le_bytes(ph.data[8..10].try_into().unwrap()) } else { 0 };
                    let n_controls = if ph.data.len() >= 12 { u16::from_le_bytes(ph.data[10..12].try_into().unwrap()) } else { 0 };
                    let para_shape_id = if ph.data.len() >= 14 { u16::from_le_bytes(ph.data[12..14].try_into().unwrap()) } else { 0 };
                    let style_id = if ph.data.len() >= 15 { ph.data[14] } else { 0 };
                    eprintln!("      nchars={} n_char_shapes={} n_line_segs={} n_range_tags={} n_controls={}",
                        nchars, n_char_shapes, n_line_segs, n_range_tags, n_controls);
                    eprintln!("      para_shape_id={} style_id={}", para_shape_id, style_id);
                }
            }

            // 5. Full record type breakdown
            let mut tag_counts: std::collections::BTreeMap<u16, usize> = std::collections::BTreeMap::new();
            for (_, rec) in &t.all_records {
                *tag_counts.entry(rec.tag_id).or_insert(0) += 1;
            }
            eprintln!("\n  Record type breakdown:");
            for (tag, count) in &tag_counts {
                eprintln!("    {} (tag={}): {} records", tags::tag_name(*tag), tag, count);
            }
        }

        // Dump all original tables
        for (ti, t) in orig_tables.iter().enumerate() {
            eprintln!("\n{}", "=".repeat(100));
            eprintln!("ORIGINAL Table[{}]", ti);
            dump_table_detail("ORIG", t);
        }

        // Dump all saved tables
        for (ti, t) in saved_tables.iter().enumerate() {
            eprintln!("\n{}", "=".repeat(100));
            eprintln!("SAVED Table[{}]", ti);
            dump_table_detail("SAVED", t);
        }

        // Special comparison: last saved table (pasted) vs first original table
        if !saved_tables.is_empty() && !orig_tables.is_empty() {
            let pasted = saved_tables.last().unwrap();
            let orig_first = &orig_tables[0];

            eprintln!("\n{}", "=".repeat(120));
            eprintln!("=== COMPARISON: PASTED TABLE (last saved) vs FIRST ORIGINAL TABLE ===");
            eprintln!("{}", "=".repeat(120));

            // Compare CTRL_HEADER
            eprintln!("\n--- CTRL_HEADER comparison ---");
            eprintln!("ORIG size: {} bytes", orig_first.ctrl_header.data.len());
            eprintln!("PASTED size: {} bytes", pasted.ctrl_header.data.len());
            if orig_first.ctrl_header.data == pasted.ctrl_header.data {
                eprintln!("CTRL_HEADER: IDENTICAL");
            } else {
                let min_len = orig_first.ctrl_header.data.len().min(pasted.ctrl_header.data.len());
                let mut diffs = Vec::new();
                for i in 0..min_len {
                    if orig_first.ctrl_header.data[i] != pasted.ctrl_header.data[i] {
                        diffs.push((i, orig_first.ctrl_header.data[i], pasted.ctrl_header.data[i]));
                    }
                }
                eprintln!("CTRL_HEADER byte diffs ({}):", diffs.len());
                for (off, a, b) in &diffs {
                    eprintln!("  offset {}: orig=0x{:02X} pasted=0x{:02X}", off, a, b);
                }
                if orig_first.ctrl_header.data.len() != pasted.ctrl_header.data.len() {
                    eprintln!("  SIZE DIFFERENCE: orig={} pasted={}",
                        orig_first.ctrl_header.data.len(), pasted.ctrl_header.data.len());
                }
            }

            // Compare TABLE record
            eprintln!("\n--- TABLE record comparison ---");
            match (&orig_first.table_rec, &pasted.table_rec) {
                (Some((_, ref ot)), Some((_, ref pt))) => {
                    eprintln!("ORIG TABLE size: {} bytes", ot.data.len());
                    eprintln!("PASTED TABLE size: {} bytes", pt.data.len());
                    if ot.data == pt.data {
                        eprintln!("TABLE: IDENTICAL");
                    } else {
                        let min_len = ot.data.len().min(pt.data.len());
                        let mut diffs = Vec::new();
                        for i in 0..min_len {
                            if ot.data[i] != pt.data[i] {
                                diffs.push((i, ot.data[i], pt.data[i]));
                            }
                        }
                        eprintln!("TABLE byte diffs ({}):", diffs.len());
                        for (off, a, b) in &diffs {
                            eprintln!("  offset {}: orig=0x{:02X} pasted=0x{:02X}", off, a, b);
                        }
                        if ot.data.len() != pt.data.len() {
                            eprintln!("  SIZE DIFFERENCE: orig={} pasted={}", ot.data.len(), pt.data.len());
                        }
                    }
                }
                _ => {
                    eprintln!("One or both TABLE records MISSING!");
                }
            }

            // Compare LIST_HEADER records (cells)
            eprintln!("\n--- Cell LIST_HEADER comparison ---");
            eprintln!("ORIG cells: {}", orig_first.list_headers.len());
            eprintln!("PASTED cells: {}", pasted.list_headers.len());
            let compare_count = orig_first.list_headers.len().min(pasted.list_headers.len()).min(10);
            for ci in 0..compare_count {
                let (_, ref olh) = orig_first.list_headers[ci];
                let (_, ref plh) = pasted.list_headers[ci];
                if olh.data == plh.data {
                    eprintln!("  Cell[{}]: IDENTICAL ({} bytes)", ci, olh.data.len());
                } else {
                    let min_len = olh.data.len().min(plh.data.len());
                    let mut diffs = Vec::new();
                    for i in 0..min_len {
                        if olh.data[i] != plh.data[i] {
                            diffs.push((i, olh.data[i], plh.data[i]));
                        }
                    }
                    eprintln!("  Cell[{}]: {} byte diffs, orig_size={} pasted_size={}",
                        ci, diffs.len(), olh.data.len(), plh.data.len());
                    for (off, a, b) in diffs.iter().take(5) {
                        eprintln!("    offset {}: orig=0x{:02X} pasted=0x{:02X}", off, a, b);
                    }
                }
            }

            // Compare record sequences
            eprintln!("\n--- Record sequence comparison ---");
            eprintln!("ORIG records in table: {}", orig_first.all_records.len());
            eprintln!("PASTED records in table: {}", pasted.all_records.len());
            let seq_len = orig_first.all_records.len().min(pasted.all_records.len());
            let mut first_mismatch = None;
            for i in 0..seq_len {
                let (_, ref orec) = orig_first.all_records[i];
                let (_, ref prec) = pasted.all_records[i];
                if orec.tag_id != prec.tag_id || orec.level != prec.level {
                    if first_mismatch.is_none() {
                        first_mismatch = Some(i);
                    }
                    eprintln!("  [{}] MISMATCH: orig={}(tag={},L{},{}B) vs pasted={}(tag={},L{},{}B)",
                        i,
                        tags::tag_name(orec.tag_id), orec.tag_id, orec.level, orec.data.len(),
                        tags::tag_name(prec.tag_id), prec.tag_id, prec.level, prec.data.len());
                }
            }
            if first_mismatch.is_none() && orig_first.all_records.len() == pasted.all_records.len() {
                eprintln!("  Record sequences: IDENTICAL structure");
            }
        }

        // Check level integrity of pasted table
        if !saved_tables.is_empty() {
            let pasted = saved_tables.last().unwrap();
            eprintln!("\n--- Level Integrity Check (Pasted Table) ---");
            let mut prev_level: i32 = -1;
            let mut issues = 0;
            for (idx, rec) in &pasted.all_records {
                let curr_level = rec.level as i32;
                if prev_level >= 0 && curr_level > prev_level + 1 {
                    issues += 1;
                    if issues <= 10 {
                        eprintln!("  LEVEL JUMP at [{}]: prev={} curr={} tag={} size={}",
                            idx, prev_level, curr_level, tags::tag_name(rec.tag_id), rec.data.len());
                    }
                }
                prev_level = curr_level;
            }
            eprintln!("  Total level issues: {}", issues);
        }

        // Context: dump records around the pasted table
        if !saved_tables.is_empty() {
            let pasted = saved_tables.last().unwrap();
            let start_idx = if pasted.ctrl_header_idx > 5 { pasted.ctrl_header_idx - 5 } else { 0 };
            let end_idx = (pasted.ctrl_header_idx + pasted.all_records.len() + 5).min(saved_recs.len());

            eprintln!("\n--- Context: Records around pasted table (rec[{}..{}]) ---", start_idx, end_idx);
            for i in start_idx..end_idx {
                let r = &saved_recs[i];
                let marker = if i == pasted.ctrl_header_idx { " <<< PASTED TABLE START" }
                    else if i == pasted.ctrl_header_idx + pasted.all_records.len() - 1 { " <<< PASTED TABLE END" }
                    else { "" };
                if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                    eprintln!("  [{}] {} L{} {}B ctrl=0x{:08X} ({}){}",
                        i, tags::tag_name(r.tag_id), r.level, r.data.len(),
                        ctrl_id, tags::ctrl_name(ctrl_id), marker);
                } else {
                    eprintln!("  [{}] {} L{} {}B{}",
                        i, tags::tag_name(r.tag_id), r.level, r.data.len(), marker);
                }
            }
        }

        // Check all records after the last table in saved to see if there's corruption
        if !saved_tables.is_empty() {
            let pasted = saved_tables.last().unwrap();
            let after_idx = pasted.ctrl_header_idx + pasted.all_records.len();
            if after_idx < saved_recs.len() {
                eprintln!("\n--- Records AFTER pasted table ({} remaining) ---", saved_recs.len() - after_idx);
                for i in after_idx..(after_idx + 20).min(saved_recs.len()) {
                    let r = &saved_recs[i];
                    if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                        let ctrl_id = u32::from_le_bytes(r.data[0..4].try_into().unwrap());
                        eprintln!("  [{}] {} L{} {}B ctrl=0x{:08X} ({})",
                            i, tags::tag_name(r.tag_id), r.level, r.data.len(),
                            ctrl_id, tags::ctrl_name(ctrl_id));
                    } else {
                        eprintln!("  [{}] {} L{} {}B first16: {:02X?}",
                            i, tags::tag_name(r.tag_id), r.level, r.data.len(),
                            &r.data[..r.data.len().min(16)]);
                    }
                }
            } else {
                eprintln!("\n--- No records after pasted table (table is last content) ---");
            }
        }

        // Overall record comparison: saved vs original
        eprintln!("\n--- Overall Record Count by Type ---");
        let mut orig_counts: std::collections::BTreeMap<u16, usize> = std::collections::BTreeMap::new();
        let mut saved_counts: std::collections::BTreeMap<u16, usize> = std::collections::BTreeMap::new();
        for r in &orig_recs {
            *orig_counts.entry(r.tag_id).or_insert(0) += 1;
        }
        for r in &saved_recs {
            *saved_counts.entry(r.tag_id).or_insert(0) += 1;
        }
        let all_tags: std::collections::BTreeSet<u16> = orig_counts.keys().chain(saved_counts.keys()).copied().collect();
        eprintln!("{:<25} {:>6} {:>6} {:>6}", "Tag", "Orig", "Saved", "Diff");
        for tag in &all_tags {
            let oc = orig_counts.get(tag).copied().unwrap_or(0);
            let sc = saved_counts.get(tag).copied().unwrap_or(0);
            let diff = sc as i64 - oc as i64;
            if diff != 0 {
                eprintln!("{:<25} {:>6} {:>6} {:>+6}", tags::tag_name(*tag), oc, sc, diff);
            }
        }

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== PASTED TABLE STRUCTURE ANALYSIS COMPLETE ===");
        eprintln!("{}", "=".repeat(120));
    }

    #[test]
    fn test_table3_deep_comparison() {
        use std::path::Path;
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::cfb_reader::CfbReader;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        let saved_path = Path::new("pasts/20250130-hongbo_saved-rp-003.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        let mut orig_cfb = CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb = CfbReader::open(&saved_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();

        let orig_recs = Record::read_all(&orig_bt).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== TABLE[3] DEEP COMPARISON (21x7 table) ===");
        eprintln!("{}", "=".repeat(120));

        fn is_table_ctrl(data: &[u8]) -> bool {
            if data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                ctrl_id == tags::CTRL_TABLE
            } else {
                false
            }
        }

        // Find the 4th table (index 3) in each file
        fn find_nth_table(recs: &[Record], n: usize) -> Option<(usize, usize)> {
            let mut table_count = 0;
            let mut i = 0;
            while i < recs.len() {
                if recs[i].tag_id == tags::HWPTAG_CTRL_HEADER && is_table_ctrl(&recs[i].data) {
                    if table_count == n {
                        let ctrl_level = recs[i].level;
                        let start = i;
                        let mut j = i + 1;
                        while j < recs.len() && recs[j].level > ctrl_level {
                            j += 1;
                        }
                        return Some((start, j));
                    }
                    table_count += 1;
                }
                i += 1;
            }
            None
        }

        let (orig_start, orig_end) = find_nth_table(&orig_recs, 3).expect("Original Table[3] not found");
        let (saved_start, saved_end) = find_nth_table(&saved_recs, 3).expect("Saved Table[3] not found");

        let orig_table_recs = &orig_recs[orig_start..orig_end];
        let saved_table_recs = &saved_recs[saved_start..saved_end];

        eprintln!("Original Table[3]: recs[{}..{}] ({} records)", orig_start, orig_end, orig_table_recs.len());
        eprintln!("Saved Table[3]:    recs[{}..{}] ({} records)", saved_start, saved_end, saved_table_recs.len());

        // Compare TABLE record flags
        let orig_tbl = orig_table_recs.iter().find(|r| r.tag_id == tags::HWPTAG_TABLE).unwrap();
        let saved_tbl = saved_table_recs.iter().find(|r| r.tag_id == tags::HWPTAG_TABLE).unwrap();
        eprintln!("\nOriginal TABLE flags: 0x{:08X}", u32::from_le_bytes(orig_tbl.data[0..4].try_into().unwrap()));
        eprintln!("Saved TABLE flags:    0x{:08X}", u32::from_le_bytes(saved_tbl.data[0..4].try_into().unwrap()));
        let orig_flags = u32::from_le_bytes(orig_tbl.data[0..4].try_into().unwrap());
        let saved_flags = u32::from_le_bytes(saved_tbl.data[0..4].try_into().unwrap());
        let diff_bits = orig_flags ^ saved_flags;
        eprintln!("Flags diff bits: 0x{:08X}", diff_bits);
        // bit 1 = split page by cell, bit 2 = repeat header
        eprintln!("  bit 1 (split_page_by_cell): orig={} saved={}", (orig_flags >> 1) & 1, (saved_flags >> 1) & 1);
        eprintln!("  bit 2 (repeat_header): orig={} saved={}", (orig_flags >> 2) & 1, (saved_flags >> 2) & 1);
        eprintln!("  bit 3 (?): orig={} saved={}", (orig_flags >> 3) & 1, (saved_flags >> 3) & 1);

        // Compare record-by-record, finding first divergence
        eprintln!("\n--- Record-by-record comparison ---");
        let min_len = orig_table_recs.len().min(saved_table_recs.len());
        let mut first_diverge = None;
        let mut extra_saved_recs = Vec::new();

        // Use alignment: match tag+level sequences
        let mut oi = 0usize;
        let mut si = 0usize;
        let mut matched = 0;
        let mut mismatched = 0;

        while oi < orig_table_recs.len() && si < saved_table_recs.len() {
            let o = &orig_table_recs[oi];
            let s = &saved_table_recs[si];
            if o.tag_id == s.tag_id && o.level == s.level {
                // Same tag and level - compare data
                if o.data != s.data && mismatched < 20 {
                    eprintln!("  [O{}/S{}] {} L{}: DATA DIFFERS (orig={}B, saved={}B)",
                        orig_start + oi, saved_start + si,
                        tags::tag_name(o.tag_id), o.level,
                        o.data.len(), s.data.len());
                    // Show specific differences for important records
                    if o.tag_id == tags::HWPTAG_PARA_TEXT {
                        eprintln!("    ORIG PARA_TEXT: {:02X?}", &o.data[..o.data.len().min(40)]);
                        eprintln!("    SAVED PARA_TEXT: {:02X?}", &s.data[..s.data.len().min(40)]);
                    }
                    mismatched += 1;
                }
                matched += 1;
                oi += 1;
                si += 1;
            } else {
                // Divergence found
                if first_diverge.is_none() {
                    first_diverge = Some((oi, si));
                    eprintln!("\n  FIRST DIVERGENCE at orig[{}]/saved[{}]:", orig_start + oi, saved_start + si);
                    eprintln!("    ORIG: {} L{} {}B", tags::tag_name(o.tag_id), o.level, o.data.len());
                    eprintln!("    SAVED: {} L{} {}B", tags::tag_name(s.tag_id), s.level, s.data.len());
                }

                // Try to re-align: is the saved record an insertion?
                // Check if orig[oi] matches saved[si+1]
                if si + 1 < saved_table_recs.len()
                    && orig_table_recs[oi].tag_id == saved_table_recs[si + 1].tag_id
                    && orig_table_recs[oi].level == saved_table_recs[si + 1].level
                {
                    extra_saved_recs.push((saved_start + si, s.clone()));
                    si += 1;
                    continue;
                }
                // Check if saved[si] matches orig[oi+1]
                if oi + 1 < orig_table_recs.len()
                    && orig_table_recs[oi + 1].tag_id == saved_table_recs[si].tag_id
                    && orig_table_recs[oi + 1].level == saved_table_recs[si].level
                {
                    eprintln!("    ORIG record [{}] has no match in saved (deleted?)", orig_start + oi);
                    oi += 1;
                    continue;
                }
                // Both advance
                oi += 1;
                si += 1;
            }
        }

        // Print remaining saved records
        while si < saved_table_recs.len() {
            extra_saved_recs.push((saved_start + si, saved_table_recs[si].clone()));
            si += 1;
        }

        eprintln!("\nMatched: {}, Data diffs: {}", matched, mismatched);
        eprintln!("Extra records in saved: {}", extra_saved_recs.len());
        if !extra_saved_recs.is_empty() {
            eprintln!("\n  Extra records in saved (not in original):");
            for (idx, rec) in extra_saved_recs.iter().take(30) {
                eprintln!("    [{}] {} L{} {}B", idx, tags::tag_name(rec.tag_id), rec.level, rec.data.len());
                if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                    // Decode as UTF-16LE text
                    let text: String = rec.data.chunks(2).filter_map(|c| {
                        if c.len() == 2 {
                            let code = u16::from_le_bytes([c[0], c[1]]);
                            if code == 0x000D || code == 0x000A { return Some('\n'); }
                            if code < 0x20 { return None; }
                            char::from_u32(code as u32)
                        } else { None }
                    }).collect();
                    eprintln!("      text: '{}'", &text[..text.len().min(80)]);
                }
            }
        }

        // Now compare record types breakdown
        let mut orig_types: std::collections::BTreeMap<u16, usize> = std::collections::BTreeMap::new();
        let mut saved_types: std::collections::BTreeMap<u16, usize> = std::collections::BTreeMap::new();
        for r in orig_table_recs { *orig_types.entry(r.tag_id).or_insert(0) += 1; }
        for r in saved_table_recs { *saved_types.entry(r.tag_id).or_insert(0) += 1; }
        eprintln!("\n--- Record type breakdown for Table[3] ---");
        eprintln!("{:<25} {:>6} {:>6} {:>6}", "Tag", "Orig", "Saved", "Diff");
        let all_tags: std::collections::BTreeSet<u16> = orig_types.keys().chain(saved_types.keys()).copied().collect();
        for tag in &all_tags {
            let oc = orig_types.get(tag).copied().unwrap_or(0);
            let sc = saved_types.get(tag).copied().unwrap_or(0);
            let diff = sc as i64 - oc as i64;
            eprintln!("{:<25} {:>6} {:>6} {:>+6}", tags::tag_name(*tag), oc, sc, diff);
        }

        // Check cells with different nParagraphs
        eprintln!("\n--- Cells (LIST_HEADER) paragraph count comparison ---");
        let orig_cells: Vec<&Record> = orig_table_recs.iter()
            .filter(|r| r.tag_id == tags::HWPTAG_LIST_HEADER).collect();
        let saved_cells: Vec<&Record> = saved_table_recs.iter()
            .filter(|r| r.tag_id == tags::HWPTAG_LIST_HEADER).collect();
        eprintln!("Original cells: {}, Saved cells: {}", orig_cells.len(), saved_cells.len());

        let cell_count = orig_cells.len().min(saved_cells.len());
        let mut cells_with_diff = Vec::new();
        for ci in 0..cell_count {
            let o_nparas = if orig_cells[ci].data.len() >= 2 {
                u16::from_le_bytes(orig_cells[ci].data[0..2].try_into().unwrap())
            } else { 0 };
            let s_nparas = if saved_cells[ci].data.len() >= 2 {
                u16::from_le_bytes(saved_cells[ci].data[0..2].try_into().unwrap())
            } else { 0 };
            if o_nparas != s_nparas {
                cells_with_diff.push((ci, o_nparas, s_nparas));
            }
        }
        if cells_with_diff.is_empty() {
            eprintln!("All cells have same nParagraphs!");
        } else {
            eprintln!("Cells with different nParagraphs:");
            for (ci, o, s) in &cells_with_diff {
                eprintln!("  Cell[{}]: orig={} saved={} (diff={})", ci, o, s, *s as i32 - *o as i32);
            }
        }

        // Check if saved cells have different data
        eprintln!("\n--- Cell data comparison (first diff bytes) ---");
        for ci in 0..cell_count {
            if orig_cells[ci].data != saved_cells[ci].data {
                let min_len = orig_cells[ci].data.len().min(saved_cells[ci].data.len());
                let mut diffs = Vec::new();
                for i in 0..min_len {
                    if orig_cells[ci].data[i] != saved_cells[ci].data[i] {
                        diffs.push((i, orig_cells[ci].data[i], saved_cells[ci].data[i]));
                    }
                }
                if !diffs.is_empty() || orig_cells[ci].data.len() != saved_cells[ci].data.len() {
                    eprintln!("  Cell[{}]: {} byte diffs, size orig={} saved={}",
                        ci, diffs.len(), orig_cells[ci].data.len(), saved_cells[ci].data.len());
                    for (off, a, b) in diffs.iter().take(5) {
                        eprintln!("    offset {}: orig=0x{:02X} saved=0x{:02X}", off, a, b);
                    }
                }
            }
        }

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== TABLE[3] DEEP COMPARISON COMPLETE ===");
        eprintln!("{}", "=".repeat(120));
    }

    #[test]
    fn test_model_table3_cell_text_check() {
        use std::path::Path;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        let saved_path = Path::new("pasts/20250130-hongbo_saved-rp-003.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== MODEL-LEVEL TABLE[3] CELL PARAGRAPH CHECK ===");
        eprintln!("{}", "=".repeat(120));

        // Find Table[3] in the model (it should be the 4th table control)
        fn find_tables(paras: &[crate::model::paragraph::Paragraph]) -> Vec<(usize, usize, &crate::model::table::Table)> {
            let mut tables = Vec::new();
            for (pi, para) in paras.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let crate::model::control::Control::Table(t) = ctrl {
                        tables.push((pi, ci, t.as_ref()));
                    }
                }
            }
            tables
        }

        let orig_tables = find_tables(&orig_doc.sections[0].paragraphs);
        let saved_tables = find_tables(&saved_doc.sections[0].paragraphs);

        eprintln!("Original tables: {}", orig_tables.len());
        eprintln!("Saved tables:    {}", saved_tables.len());

        if orig_tables.len() > 3 && saved_tables.len() > 3 {
            let (_, _, orig_t3) = orig_tables[3];
            let (_, _, saved_t3) = saved_tables[3];

            eprintln!("\nOriginal Table[3]: rows={} cols={} cells={}", orig_t3.row_count, orig_t3.col_count, orig_t3.cells.len());
            eprintln!("Saved Table[3]:    rows={} cols={} cells={}", saved_t3.row_count, saved_t3.col_count, saved_t3.cells.len());

            // Check paragraphs in cells
            let mut orig_para_with_text = 0;
            let mut orig_para_without_text = 0;
            let mut orig_para_with_flag = 0;
            let mut saved_para_with_text = 0;
            let mut saved_para_without_text = 0;
            let mut saved_para_with_flag = 0;

            eprintln!("\n--- Original Table[3] cell paragraphs ---");
            for (ci, cell) in orig_t3.cells.iter().enumerate() {
                for (pi, para) in cell.paragraphs.iter().enumerate() {
                    let has_text = !para.text.is_empty();
                    if has_text { orig_para_with_text += 1; } else { orig_para_without_text += 1; }
                    if para.has_para_text { orig_para_with_flag += 1; }
                    if !has_text && !para.has_para_text {
                        // Only show the first few empty paragraphs
                        if ci < 3 {
                            eprintln!("  cell[{}] para[{}]: text='{}' has_para_text={} char_count={} controls={}",
                                ci, pi, &para.text[..para.text.len().min(20)],
                                para.has_para_text, para.char_count, para.controls.len());
                        }
                    }
                }
            }

            eprintln!("\n--- Saved Table[3] cell paragraphs ---");
            for (ci, cell) in saved_t3.cells.iter().enumerate() {
                for (pi, para) in cell.paragraphs.iter().enumerate() {
                    let has_text = !para.text.is_empty();
                    if has_text { saved_para_with_text += 1; } else { saved_para_without_text += 1; }
                    if para.has_para_text { saved_para_with_flag += 1; }
                    if ci < 3 {
                        eprintln!("  cell[{}] para[{}]: text='{}' has_para_text={} char_count={} controls={}",
                            ci, pi, &para.text[..para.text.len().min(40)],
                            para.has_para_text, para.char_count, para.controls.len());
                    }
                }
            }

            eprintln!("\n--- Summary ---");
            eprintln!("Original: {} with text, {} without text, {} with has_para_text flag",
                orig_para_with_text, orig_para_without_text, orig_para_with_flag);
            eprintln!("Saved:    {} with text, {} without text, {} with has_para_text flag",
                saved_para_with_text, saved_para_without_text, saved_para_with_flag);

            // Find cells where text content differs
            let cell_count = orig_t3.cells.len().min(saved_t3.cells.len());
            let mut text_diff_cells = Vec::new();
            for ci in 0..cell_count {
                let opara_count = orig_t3.cells[ci].paragraphs.len();
                let spara_count = saved_t3.cells[ci].paragraphs.len();
                if opara_count != spara_count {
                    text_diff_cells.push((ci, format!("para count differs: {} vs {}", opara_count, spara_count)));
                    continue;
                }
                for pi in 0..opara_count {
                    let op = &orig_t3.cells[ci].paragraphs[pi];
                    let sp = &saved_t3.cells[ci].paragraphs[pi];
                    if op.text != sp.text || op.has_para_text != sp.has_para_text {
                        text_diff_cells.push((ci, format!(
                            "para[{}]: orig_text='{}' orig_flag={} saved_text='{}' saved_flag={}",
                            pi, &op.text[..op.text.len().min(30)], op.has_para_text,
                            &sp.text[..sp.text.len().min(30)], sp.has_para_text)));
                    }
                }
            }
            eprintln!("\nCells with text/flag differences: {}", text_diff_cells.len());
            for (ci, desc) in text_diff_cells.iter().take(20) {
                eprintln!("  cell[{}]: {}", ci, desc);
            }
        }

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== MODEL-LEVEL CHECK COMPLETE ===");
        eprintln!("{}", "=".repeat(120));
    }

    #[test]
    fn test_roundtrip_empty_cell_corruption() {
        use std::path::Path;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        if !orig_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== ROUND-TRIP EMPTY CELL CORRUPTION TEST ===");
        eprintln!("{}", "=".repeat(120));

        // Find Table[3] in the original model
        fn find_tables(paras: &[crate::model::paragraph::Paragraph]) -> Vec<(usize, usize, &crate::model::table::Table)> {
            let mut tables = Vec::new();
            for (pi, para) in paras.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let crate::model::control::Control::Table(t) = ctrl {
                        tables.push((pi, ci, t.as_ref()));
                    }
                }
            }
            tables
        }

        let orig_tables = find_tables(&orig_doc.sections[0].paragraphs);
        assert!(orig_tables.len() > 3, "Need at least 4 tables");
        let (_, _, orig_t3) = orig_tables[3];

        // Check original model: char_count values for empty cells
        eprintln!("\n--- Original Table[3] empty cell char_count/msb/text analysis ---");
        let mut empty_cell_count = 0;
        let mut empty_cell_char_counts = std::collections::HashMap::new();
        for (ci, cell) in orig_t3.cells.iter().enumerate() {
            for (pi, para) in cell.paragraphs.iter().enumerate() {
                if para.text.is_empty() && !para.has_para_text {
                    empty_cell_count += 1;
                    *empty_cell_char_counts.entry(para.char_count).or_insert(0) += 1;
                    if ci < 5 {
                        eprintln!("  cell[{}] para[{}]: text='{}' has_para_text={} char_count={} char_count_msb={} controls={} raw_header_extra_len={}",
                            ci, pi, para.text, para.has_para_text, para.char_count, para.char_count_msb,
                            para.controls.len(), para.raw_header_extra.len());
                        if para.raw_header_extra.len() >= 10 {
                            eprintln!("    raw_header_extra: {:02x?}", &para.raw_header_extra);
                        }
                    }
                }
            }
        }
        eprintln!("\nEmpty cells: {}", empty_cell_count);
        for (cc, count) in &empty_cell_char_counts {
            eprintln!("  char_count={}: {} cells", cc, count);
        }

        // Now do a round-trip: serialize from model (bypassing raw_stream)
        eprintln!("\n--- Round-trip: serialize section from model ---");
        // Build records manually from the model paragraphs (bypassing raw_stream check)
        let mut records_from_model = Vec::new();
        for para in &orig_doc.sections[0].paragraphs {
            crate::serializer::body_text::serialize_paragraph_list(
                std::slice::from_ref(para), 0, &mut records_from_model
            );
        }
        let serialized = crate::serializer::record_writer::write_records(&records_from_model);
        eprintln!("Serialized section size: {} bytes", serialized.len());

        // Parse the serialized records
        let records = Record::read_all(&serialized).unwrap();
        eprintln!("Total records: {}", records.len());

        // Count PARA_TEXT records
        let para_text_count = records.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_TEXT).count();
        let para_header_count = records.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER).count();
        eprintln!("PARA_HEADER: {}", para_header_count);
        eprintln!("PARA_TEXT: {}", para_text_count);

        // Also count from original raw_stream for comparison
        let orig_raw = orig_doc.sections[0].raw_stream.as_ref().unwrap();
        let orig_records = Record::read_all(orig_raw).unwrap();
        let orig_pt_count = orig_records.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_TEXT).count();
        let orig_ph_count = orig_records.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER).count();
        eprintln!("\nOriginal raw:");
        eprintln!("  PARA_HEADER: {}", orig_ph_count);
        eprintln!("  PARA_TEXT: {}", orig_pt_count);

        eprintln!("\nDelta:");
        eprintln!("  PARA_HEADER: {} (expected 0)", para_header_count as i32 - orig_ph_count as i32);
        eprintln!("  PARA_TEXT: {} (expected 0)", para_text_count as i32 - orig_pt_count as i32);

        // Check specific records inside Table[3] area
        // Find all CTRL_HEADER records with Table ctrl_id
        let mut table_idx = 0;
        for (ri, rec) in records.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                if ctrl_id == tags::CTRL_TABLE {
                    if table_idx == 3 {
                        // Found Table[3] in serialized output - count children
                        let table_level = rec.level;
                        let mut child_pt_count = 0;
                        let mut child_ph_count = 0;
                        for child_rec in &records[ri+1..] {
                            if child_rec.level <= table_level { break; }
                            if child_rec.tag_id == tags::HWPTAG_PARA_TEXT { child_pt_count += 1; }
                            if child_rec.tag_id == tags::HWPTAG_PARA_HEADER { child_ph_count += 1; }
                        }
                        eprintln!("\nSerialized Table[3] children:");
                        eprintln!("  PARA_HEADER: {}", child_ph_count);
                        eprintln!("  PARA_TEXT: {}", child_pt_count);

                        // Do the same for original
                        let mut orig_table_idx2 = 0;
                        for (ori, orec) in orig_records.iter().enumerate() {
                            if orec.tag_id == tags::HWPTAG_CTRL_HEADER && orec.data.len() >= 4 {
                                let cid = u32::from_le_bytes([orec.data[0], orec.data[1], orec.data[2], orec.data[3]]);
                                if cid == tags::CTRL_TABLE {
                                    if orig_table_idx2 == 3 {
                                        let olevel = orec.level;
                                        let mut o_pt = 0;
                                        let mut o_ph = 0;
                                        for child_rec in &orig_records[ori+1..] {
                                            if child_rec.level <= olevel { break; }
                                            if child_rec.tag_id == tags::HWPTAG_PARA_TEXT { o_pt += 1; }
                                            if child_rec.tag_id == tags::HWPTAG_PARA_HEADER { o_ph += 1; }
                                        }
                                        eprintln!("\nOriginal Table[3] children:");
                                        eprintln!("  PARA_HEADER: {}", o_ph);
                                        eprintln!("  PARA_TEXT: {}", o_pt);
                                        break;
                                    }
                                    orig_table_idx2 += 1;
                                }
                            }
                        }

                        // Check each PARA_TEXT in serialized Table[3] - look for 5-space entries
                        eprintln!("\nSerialized Table[3] PARA_TEXT analysis:");
                        let mut five_space_count = 0;
                        for child_rec in &records[ri+1..] {
                            if child_rec.level <= table_level { break; }
                            if child_rec.tag_id == tags::HWPTAG_PARA_TEXT {
                                let data = &child_rec.data;
                                // 5 spaces + terminator = [0x20,0x00, 0x20,0x00, 0x20,0x00, 0x20,0x00, 0x20,0x00, 0x0D,0x00]
                                if data.len() == 12 {
                                    let is_five_spaces = data == &[0x20,0x00, 0x20,0x00, 0x20,0x00, 0x20,0x00, 0x20,0x00, 0x0D,0x00];
                                    if is_five_spaces {
                                        five_space_count += 1;
                                    } else {
                                        eprintln!("  12-byte PARA_TEXT (not 5-spaces): {:02x?}", data);
                                    }
                                }
                            }
                        }
                        eprintln!("  5-space PARA_TEXT entries: {}", five_space_count);

                        break;
                    }
                    table_idx += 1;
                }
            }
        }

        // Check model-level data that was used for serialization
        eprintln!("\n--- Model data used for serialization ---");
        let tables_check = find_tables(&orig_doc.sections[0].paragraphs);
        if tables_check.len() > 3 {
            let (_, _, t3) = tables_check[3];
            let mut model_empty = 0;
            let mut model_with_text = 0;
            for cell in &t3.cells {
                for para in &cell.paragraphs {
                    if para.text.is_empty() && !para.has_para_text {
                        model_empty += 1;
                    } else {
                        model_with_text += 1;
                    }
                }
            }
            eprintln!("Model Table[3] paragraphs (from original parse):");
            eprintln!("  Empty paragraphs (text='' && has_para_text=false): {}", model_empty);
            eprintln!("  Paragraphs with text or has_para_text: {}", model_with_text);
        }

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== ROUND-TRIP TEST COMPLETE ===");
        eprintln!("{}", "=".repeat(120));
    }

    #[test]
    fn test_saved_file_table_flags_and_origin() {
        use std::path::Path;

        let orig_path = Path::new("pasts/20250130-hongbo-p2.hwp");
        let saved_path = Path::new("pasts/20250130-hongbo_saved-rp-003.hwp");
        if !orig_path.exists() || !saved_path.exists() {
            eprintln!("파일 없음 — 건너뜀");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== SAVED FILE TABLE FLAGS AND ORIGIN ANALYSIS ===");
        eprintln!("{}", "=".repeat(120));

        fn find_tables(paras: &[crate::model::paragraph::Paragraph]) -> Vec<(usize, usize, &crate::model::table::Table)> {
            let mut tables = Vec::new();
            for (pi, para) in paras.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let crate::model::control::Control::Table(t) = ctrl {
                        tables.push((pi, ci, t.as_ref()));
                    }
                }
            }
            tables
        }

        let orig_tables = find_tables(&orig_doc.sections[0].paragraphs);
        let saved_tables = find_tables(&saved_doc.sections[0].paragraphs);

        eprintln!("\nOriginal tables: {} | Saved tables: {}", orig_tables.len(), saved_tables.len());

        // Compare all tables: rows, cols, cells, flags, para positions
        for i in 0..orig_tables.len().max(saved_tables.len()) {
            eprintln!("\n--- Table[{}] ---", i);
            if i < orig_tables.len() {
                let (pi, ci, t) = orig_tables[i];
                let total_paras: usize = t.cells.iter().map(|c| c.paragraphs.len()).sum();
                let text_paras: usize = t.cells.iter().map(|c| c.paragraphs.iter().filter(|p| !p.text.is_empty()).count()).sum();
                eprintln!("  Original: para_pos={} ctrl_pos={} rows={} cols={} cells={} total_paras={} text_paras={} flags=0x{:08x} page_break={:?}",
                    pi, ci, t.row_count, t.col_count, t.cells.len(), total_paras, text_paras, t.attr, t.page_break);
            } else {
                eprintln!("  Original: MISSING");
            }
            if i < saved_tables.len() {
                let (pi, ci, t) = saved_tables[i];
                let total_paras: usize = t.cells.iter().map(|c| c.paragraphs.len()).sum();
                let text_paras: usize = t.cells.iter().map(|c| c.paragraphs.iter().filter(|p| !p.text.is_empty()).count()).sum();
                eprintln!("  Saved:    para_pos={} ctrl_pos={} rows={} cols={} cells={} total_paras={} text_paras={} flags=0x{:08x} page_break={:?}",
                    pi, ci, t.row_count, t.col_count, t.cells.len(), total_paras, text_paras, t.attr, t.page_break);
            } else {
                eprintln!("  Saved: MISSING");
            }
        }

        // Detailed check of Table[3]: compare cell-by-cell
        if orig_tables.len() > 3 && saved_tables.len() > 3 {
            let (_, _, ot) = orig_tables[3];
            let (_, _, st) = saved_tables[3];

            eprintln!("\n--- Table[3] cell-by-cell comparison ---");
            let cell_count = ot.cells.len().min(st.cells.len());
            let mut diffs = Vec::new();
            for ci in 0..cell_count {
                let oc = &ot.cells[ci];
                let sc = &st.cells[ci];
                let mut cell_diffs = Vec::new();

                // Compare cell structure
                if oc.col != sc.col || oc.row != sc.row || oc.col_span != sc.col_span || oc.row_span != sc.row_span {
                    cell_diffs.push(format!("position: ({},{}) cs={}x{} vs ({},{}) cs={}x{}",
                        oc.col, oc.row, oc.col_span, oc.row_span,
                        sc.col, sc.row, sc.col_span, sc.row_span));
                }
                if oc.width != sc.width || oc.height != sc.height {
                    cell_diffs.push(format!("size: {}x{} vs {}x{}", oc.width, oc.height, sc.width, sc.height));
                }
                if oc.paragraphs.len() != sc.paragraphs.len() {
                    cell_diffs.push(format!("para_count: {} vs {}", oc.paragraphs.len(), sc.paragraphs.len()));
                }
                // Compare paragraph text
                for pi in 0..oc.paragraphs.len().min(sc.paragraphs.len()) {
                    let op = &oc.paragraphs[pi];
                    let sp = &sc.paragraphs[pi];
                    if op.text != sp.text {
                        cell_diffs.push(format!("para[{}] text: '{}' vs '{}'", pi,
                            &op.text[..op.text.len().min(30)],
                            &sp.text[..sp.text.len().min(30)]));
                    }
                    if op.char_count != sp.char_count {
                        cell_diffs.push(format!("para[{}] char_count: {} vs {}", pi, op.char_count, sp.char_count));
                    }
                    if op.has_para_text != sp.has_para_text {
                        cell_diffs.push(format!("para[{}] has_para_text: {} vs {}", pi, op.has_para_text, sp.has_para_text));
                    }
                }

                if !cell_diffs.is_empty() {
                    diffs.push((ci, cell_diffs));
                }
            }

            eprintln!("Cells with differences: {} out of {}", diffs.len(), cell_count);
            for (ci, cell_diffs) in &diffs {
                eprintln!("  cell[{}] (row={}, col={}):", ci, ot.cells[*ci].row, ot.cells[*ci].col);
                for d in cell_diffs {
                    eprintln!("    {}", d);
                }
            }

            // Show the specific text content of first 5 differing cells
            eprintln!("\n--- First 5 differing cells detail ---");
            for (ci, _) in diffs.iter().take(5) {
                let oc = &ot.cells[*ci];
                let sc = &st.cells[*ci];
                eprintln!("  cell[{}] (row={}, col={}):", ci, oc.row, oc.col);
                for pi in 0..oc.paragraphs.len().min(sc.paragraphs.len()) {
                    let op = &oc.paragraphs[pi];
                    let sp = &sc.paragraphs[pi];
                    eprintln!("    orig para[{}]: text={:?} char_count={} msb={} has_pt={} char_offsets={:?} char_shapes_len={}",
                        pi, &op.text, op.char_count, op.char_count_msb, op.has_para_text, &op.char_offsets, op.char_shapes.len());
                    eprintln!("    saved para[{}]: text={:?} char_count={} msb={} has_pt={} char_offsets={:?} char_shapes_len={}",
                        pi, &sp.text, sp.char_count, sp.char_count_msb, sp.has_para_text, &sp.char_offsets, sp.char_shapes.len());
                }
            }
        }

        // Also check: which paragraph does Table[3] belong to, and what else changed in the document?
        eprintln!("\n--- Document-level comparison ---");
        let orig_para_count = orig_doc.sections[0].paragraphs.len();
        let saved_para_count = saved_doc.sections[0].paragraphs.len();
        eprintln!("Section[0] paragraph count: orig={} saved={}", orig_para_count, saved_para_count);

        // Check non-table paragraphs for text differences
        let min_paras = orig_para_count.min(saved_para_count);
        let mut non_table_diffs = 0;
        for pi in 0..min_paras {
            let op = &orig_doc.sections[0].paragraphs[pi];
            let sp = &saved_doc.sections[0].paragraphs[pi];
            if op.text != sp.text || op.controls.len() != sp.controls.len() {
                non_table_diffs += 1;
                if non_table_diffs <= 5 {
                    eprintln!("  para[{}] differs: orig_text_len={} saved_text_len={} orig_ctrls={} saved_ctrls={}",
                        pi, op.text.len(), sp.text.len(), op.controls.len(), sp.controls.len());
                }
            }
        }
        eprintln!("Non-table paragraph differences: {}", non_table_diffs);
        if saved_para_count > orig_para_count {
            eprintln!("Extra paragraphs in saved: {}", saved_para_count - orig_para_count);
            for pi in orig_para_count..saved_para_count {
                let sp = &saved_doc.sections[0].paragraphs[pi];
                eprintln!("  para[{}]: text_len={} controls={}", pi, sp.text.len(), sp.controls.len());
            }
        }

        // Detailed check of para[8] and para[10]
        eprintln!("\n--- Detailed check of para[8] and para[10] ---");
        for pi in [8, 9, 10, 11] {
            if pi < orig_doc.sections[0].paragraphs.len() {
                let p = &orig_doc.sections[0].paragraphs[pi];
                eprintln!("  ORIG para[{}]: text_len={} text={:?} ctrls={} ctrl_types={:?}",
                    pi, p.text.len(), &p.text.chars().take(30).collect::<String>(),
                    p.controls.len(),
                    p.controls.iter().map(|c| match c {
                        crate::model::control::Control::Table(_) => "Table",
                        crate::model::control::Control::Shape(_) => "Shape",
                        crate::model::control::Control::Footnote(_) => "Footnote",
                        crate::model::control::Control::Endnote(_) => "Endnote",
                        crate::model::control::Control::Header(_) => "Header",
                        crate::model::control::Control::Footer(_) => "Footer",
                        crate::model::control::Control::SectionDef(_) => "SectionDef",
                        crate::model::control::Control::ColumnDef(_) => "ColumnDef",
                        crate::model::control::Control::Picture(_) => "Picture",
                        _ => "Other",
                    }).collect::<Vec<_>>());
            }
            if pi < saved_doc.sections[0].paragraphs.len() {
                let p = &saved_doc.sections[0].paragraphs[pi];
                eprintln!("  SAVED para[{}]: text_len={} text={:?} ctrls={} ctrl_types={:?}",
                    pi, p.text.len(), &p.text.chars().take(30).collect::<String>(),
                    p.controls.len(),
                    p.controls.iter().map(|c| match c {
                        crate::model::control::Control::Table(_) => "Table",
                        crate::model::control::Control::Shape(_) => "Shape",
                        crate::model::control::Control::Footnote(_) => "Footnote",
                        crate::model::control::Control::Endnote(_) => "Endnote",
                        crate::model::control::Control::Header(_) => "Header",
                        crate::model::control::Control::Footer(_) => "Footer",
                        crate::model::control::Control::SectionDef(_) => "SectionDef",
                        crate::model::control::Control::ColumnDef(_) => "ColumnDef",
                        crate::model::control::Control::Picture(_) => "Picture",
                        _ => "Other",
                    }).collect::<Vec<_>>());
            }
        }

        // Check if Table[3] in saved is the same table (same col/row structure) as original
        // Or if it's a newly created table from paste
        eprintln!("\n--- Table[3] structural identity check ---");
        if orig_tables.len() > 3 && saved_tables.len() > 3 {
            let (_, _, ot) = orig_tables[3];
            let (_, _, st) = saved_tables[3];
            eprintln!("  Same row_count: {} ({}=={})", ot.row_count == st.row_count, ot.row_count, st.row_count);
            eprintln!("  Same col_count: {} ({}=={})", ot.col_count == st.col_count, ot.col_count, st.col_count);
            eprintln!("  Same cell count: {} ({}=={})", ot.cells.len() == st.cells.len(), ot.cells.len(), st.cells.len());
            eprintln!("  Same attr: {} (0x{:08x}==0x{:08x})", ot.attr == st.attr, ot.attr, st.attr);
            eprintln!("  Same border_fill_id: {} ({}=={})", ot.border_fill_id == st.border_fill_id, ot.border_fill_id, st.border_fill_id);

            // Compare cells with text - are the actual text values the same?
            let mut text_match_count = 0;
            let mut text_mismatch_count = 0;
            for ci in 0..ot.cells.len().min(st.cells.len()) {
                for pi in 0..ot.cells[ci].paragraphs.len().min(st.cells[ci].paragraphs.len()) {
                    let op = &ot.cells[ci].paragraphs[pi];
                    let sp = &st.cells[ci].paragraphs[pi];
                    if !op.text.is_empty() && op.text == sp.text {
                        text_match_count += 1;
                    } else if !op.text.is_empty() && op.text != sp.text {
                        text_mismatch_count += 1;
                    }
                }
            }
            eprintln!("  Cells with original text preserved: {}", text_match_count);
            eprintln!("  Cells with original text changed: {}", text_mismatch_count);
        }

        eprintln!("\n{}", "=".repeat(120));
    }

    /// 재직렬화 격리 테스트: paste 없이 raw_stream 제거만으로 레코드 수 비교
    #[test]
    fn test_roundtrip_isolation_no_paste() {
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 파일 없음");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // Step 1: Re-serialize WITHOUT paste (just clear raw_stream)
        doc.document.sections[0].raw_stream = None;
        let saved_data = doc.export_hwp_native().unwrap();
        eprintln!("원본: {} bytes, 재직렬화(no paste): {} bytes", orig_data.len(), saved_data.len());

        // Step 2: Re-parse the saved file
        let doc2 = HwpDocument::from_bytes(&saved_data);
        match &doc2 {
            Ok(d) => eprintln!("재파싱 성공: {} sections, {} paragraphs",
                d.document().sections.len(),
                d.document().sections[0].paragraphs.len()),
            Err(e) => eprintln!("재파싱 실패: {:?}", e),
        }
        assert!(doc2.is_ok(), "재직렬화 파일 파싱 실패");

        // Step 3: Compare record counts
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\n=== Record count comparison (no paste) ===");
        eprintln!("Original records: {}", orig_recs.len());
        eprintln!("Saved records: {}", saved_recs.len());

        let count_tag = |recs: &[Record], tag: u16| recs.iter().filter(|r| r.tag_id == tag).count();

        let tags_to_check: [(u16, &str); 7] = [
            (tags::HWPTAG_PARA_HEADER, "PARA_HEADER"),
            (tags::HWPTAG_PARA_TEXT, "PARA_TEXT"),
            (tags::HWPTAG_PARA_CHAR_SHAPE, "PARA_CHAR_SHAPE"),
            (tags::HWPTAG_PARA_LINE_SEG, "PARA_LINE_SEG"),
            (tags::HWPTAG_CTRL_HEADER, "CTRL_HEADER"),
            (tags::HWPTAG_LIST_HEADER, "LIST_HEADER"),
            (tags::HWPTAG_TABLE, "TABLE"),
        ];

        let mut any_diff = false;
        for (tag, name) in &tags_to_check {
            let orig_cnt = count_tag(&orig_recs, *tag);
            let saved_cnt = count_tag(&saved_recs, *tag);
            let diff = saved_cnt as i64 - orig_cnt as i64;
            if diff != 0 {
                eprintln!("  {}: {} → {} ({}{}) ← DIFF", name, orig_cnt, saved_cnt,
                    if diff > 0 { "+" } else { "" }, diff);
                any_diff = true;
            } else {
                eprintln!("  {}: {} (동일)", name, orig_cnt);
            }
        }

        if !any_diff {
            eprintln!("\n모든 레코드 타입 동일 ✓");
        }

        // Step 4: Check that PARA_HEADER char_count matches PARA_TEXT existence
        eprintln!("\n=== PARA_HEADER/PARA_TEXT consistency check ===");
        let mut inconsistencies = 0;
        let mut i = 0;
        while i < saved_recs.len() {
            if saved_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph_data = &saved_recs[i].data;
                let ph_level = saved_recs[i].level;
                let nchars = if ph_data.len() >= 4 {
                    u32::from_le_bytes([ph_data[0], ph_data[1], ph_data[2], ph_data[3]]) & 0x7FFFFFFF
                } else { 0 };
                // Check next record
                let has_text = i + 1 < saved_recs.len()
                    && saved_recs[i+1].tag_id == tags::HWPTAG_PARA_TEXT
                    && saved_recs[i+1].level == ph_level + 1;
                if nchars > 1 && !has_text {
                    eprintln!("  rec[{}] PARA_HEADER nchars={} but NO PARA_TEXT follows!", i, nchars);
                    inconsistencies += 1;
                }
                if nchars <= 1 && has_text {
                    let pt_size = saved_recs[i+1].data.len();
                    eprintln!("  rec[{}] PARA_HEADER nchars={} but HAS PARA_TEXT ({}B) — might be OK (terminator only)", i, nchars, pt_size);
                }
            }
            i += 1;
        }
        eprintln!("  Total inconsistencies: {}", inconsistencies);
    }

    /// 테이블 paste 후 재직렬화 유효성 검증
    #[test]
    fn test_paste_table_then_export_validation() {
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 파일 없음");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // 원본 레코드 수 저장
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();
        let orig_para_text_count = orig_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_TEXT).count();
        let orig_para_count = orig_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER).count();
        eprintln!("원본: {} records, {} PARA_HEADER, {} PARA_TEXT", orig_recs.len(), orig_para_count, orig_para_text_count);

        // 간단한 HTML 테이블 paste
        let simple_table_html = r#"<table><tr><td>Cell A</td><td>Cell B</td></tr><tr><td>Cell C</td><td>Cell D</td></tr></table>"#;
        let last_para = doc.document.sections[0].paragraphs.len() - 1;
        let result = doc.paste_html_native(0, last_para, 0, simple_table_html);
        match &result {
            Ok(r) => eprintln!("Paste result: {}", r),
            Err(e) => { eprintln!("Paste failed: {:?}", e); return; },
        }

        // Export
        let saved_data = doc.export_hwp_native().unwrap();
        eprintln!("재직렬화(with paste): {} bytes", saved_data.len());

        // Re-parse
        let doc2 = HwpDocument::from_bytes(&saved_data);
        match &doc2 {
            Ok(d) => eprintln!("재파싱 성공: {} sections, {} paragraphs",
                d.document().sections.len(),
                d.document().sections[0].paragraphs.len()),
            Err(e) => {
                eprintln!("재파싱 실패: {:?}", e);
                // 실패시에도 record-level 분석 진행
            }
        }

        // Record level 분석
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\n=== Record count comparison (with paste) ===");
        let count_tag = |recs: &[Record], tag: u16| recs.iter().filter(|r| r.tag_id == tag).count();
        let tags_to_check: [(u16, &str); 7] = [
            (tags::HWPTAG_PARA_HEADER, "PARA_HEADER"),
            (tags::HWPTAG_PARA_TEXT, "PARA_TEXT"),
            (tags::HWPTAG_PARA_CHAR_SHAPE, "PARA_CHAR_SHAPE"),
            (tags::HWPTAG_PARA_LINE_SEG, "PARA_LINE_SEG"),
            (tags::HWPTAG_CTRL_HEADER, "CTRL_HEADER"),
            (tags::HWPTAG_LIST_HEADER, "LIST_HEADER"),
            (tags::HWPTAG_TABLE, "TABLE"),
        ];
        for (tag, name) in &tags_to_check {
            let orig_cnt = count_tag(&orig_recs, *tag);
            let saved_cnt = count_tag(&saved_recs, *tag);
            let diff = saved_cnt as i64 - orig_cnt as i64;
            eprintln!("  {}: {} → {} ({}{}){}",
                name, orig_cnt, saved_cnt,
                if diff > 0 { "+" } else { "" }, diff,
                if diff != 0 { " ← DIFF" } else { "" });
        }

        // PARA_HEADER/PARA_TEXT consistency
        eprintln!("\n=== Consistency check ===");
        let mut issues = 0;
        let mut i = 0;
        while i < saved_recs.len() {
            if saved_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph_data = &saved_recs[i].data;
                let ph_level = saved_recs[i].level;
                let nchars = if ph_data.len() >= 4 {
                    u32::from_le_bytes([ph_data[0], ph_data[1], ph_data[2], ph_data[3]]) & 0x7FFFFFFF
                } else { 0 };

                // numCharShapes from para_header
                let n_cs = if ph_data.len() >= 14 {
                    u16::from_le_bytes([ph_data[12], ph_data[13]])
                } else { 0 };

                // Count actual PARA_CHAR_SHAPE entries
                let mut actual_cs = 0u32;
                let mut j = i + 1;
                while j < saved_recs.len() && saved_recs[j].level > ph_level {
                    if saved_recs[j].tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && saved_recs[j].level == ph_level + 1 {
                        actual_cs = (saved_recs[j].data.len() / 8) as u32;
                    }
                    j += 1;
                }

                if n_cs as u32 != actual_cs && actual_cs > 0 {
                    eprintln!("  rec[{}] PARA_HEADER: numCharShapes={} but actual PARA_CHAR_SHAPE entries={}", i, n_cs, actual_cs);
                    issues += 1;
                }

                // Check if nchars > 1 but no PARA_TEXT
                let has_text = i + 1 < saved_recs.len()
                    && saved_recs[i+1].tag_id == tags::HWPTAG_PARA_TEXT
                    && saved_recs[i+1].level == ph_level + 1;
                if nchars > 1 && !has_text {
                    eprintln!("  rec[{}] PARA_HEADER nchars={} but NO PARA_TEXT!", i, nchars);
                    issues += 1;
                }
            }
            i += 1;
        }
        eprintln!("  Total issues: {}", issues);

        // Dump pasted table records
        eprintln!("\n=== Pasted table structure ===");
        let tables: Vec<usize> = saved_recs.iter().enumerate()
            .filter(|(_, r)| r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 && {
                let ctrl_id = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                ctrl_id == tags::CTRL_TABLE
            })
            .map(|(i, _)| i)
            .collect();
        eprintln!("Total tables: {}", tables.len());
        if let Some(&last_tbl_idx) = tables.last() {
            let tbl_level = saved_recs[last_tbl_idx].level;
            let mut end = last_tbl_idx + 1;
            while end < saved_recs.len() && saved_recs[end].level > tbl_level {
                end += 1;
            }
            eprintln!("Last (pasted) table: rec[{}..{}] ({} records)", last_tbl_idx, end, end - last_tbl_idx);
            for ri in last_tbl_idx..end.min(last_tbl_idx + 40) {
                let r = &saved_recs[ri];
                let tag_name = tags::tag_name(r.tag_id);
                eprintln!("  [{}] {} L{} {}B", ri, tag_name, r.level, r.data.len());
            }
        }

        // Check TABLE record extra bytes in original tables
        eprintln!("\n=== Original TABLE record sizes ===");
        for (ri, r) in orig_recs.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_TABLE {
                let data = &r.data;
                if data.len() >= 8 {
                    let nrows = u16::from_le_bytes([data[4], data[5]]);
                    let ncols = u16::from_le_bytes([data[6], data[7]]);
                    let expected_min = 4 + 2 + 2 + 2 + 8 + (nrows as usize) * 2 + 2;
                    let extra = data.len().saturating_sub(expected_min);
                    let extra_bytes: Vec<u8> = data[expected_min..].to_vec();
                    eprintln!("  rec[{}] TABLE {}B (nrows={} ncols={} expected_min={} extra={} extra_bytes={:02X?})",
                        ri, data.len(), nrows, ncols, expected_min, extra, extra_bytes);
                }
            }
        }

        // 저장 (수동 확인용)
        let out_dir = std::path::Path::new("output");
        if out_dir.exists() {
            std::fs::write(out_dir.join("hongbo_paste_test.hwp"), &saved_data).unwrap();
            eprintln!("\n저장: output/hongbo_paste_test.hwp");
        }
    }

    /// DocInfo CharShape 수 추적: 파싱 → convertToEditable → paste → export
    #[test]
    fn test_trace_charshape_loss() {
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 파일 없음");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // Helper: count CHAR_SHAPE records in raw_stream
        fn count_cs_in_raw(raw: &Option<Vec<u8>>) -> usize {
            match raw {
                Some(data) => {
                    let records = Record::read_all(data).unwrap_or_default();
                    records.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count()
                }
                None => 0,
            }
        }

        // Step 1: After parsing
        let model_cs_1 = doc.document().doc_info.char_shapes.len();
        let raw_cs_1 = count_cs_in_raw(&doc.document().doc_info.raw_stream);
        let is_dist = doc.document().header.distribution;
        eprintln!("Step 1 (after parse): model={} raw={} distribution={}", model_cs_1, raw_cs_1, is_dist);

        // Step 2: After convert_to_editable
        let converted = doc.convert_to_editable_native().unwrap();
        let model_cs_2 = doc.document().doc_info.char_shapes.len();
        let raw_cs_2 = count_cs_in_raw(&doc.document().doc_info.raw_stream);
        eprintln!("Step 2 (after convert): model={} raw={} result={}", model_cs_2, raw_cs_2, converted);

        // Step 3: Export without paste
        let saved_no_paste = doc.export_hwp_native().unwrap();
        let doc_np = crate::parser::parse_hwp(&saved_no_paste).unwrap();
        eprintln!("Step 3 (export no paste): model_cs={}", doc_np.doc_info.char_shapes.len());

        // Step 4: Paste simple table
        let last_para = doc.document.sections[0].paragraphs.len() - 1;
        let _ = doc.paste_html_native(0, last_para, 0,
            r#"<table><tr><td>A</td><td>B</td></tr></table>"#);
        let model_cs_4 = doc.document().doc_info.char_shapes.len();
        let raw_cs_4 = count_cs_in_raw(&doc.document().doc_info.raw_stream);
        eprintln!("Step 4 (after paste): model={} raw={}", model_cs_4, raw_cs_4);

        // Step 5: Export after paste
        let saved_with_paste = doc.export_hwp_native().unwrap();
        let doc_wp = crate::parser::parse_hwp(&saved_with_paste).unwrap();
        eprintln!("Step 5 (export with paste): model_cs={}", doc_wp.doc_info.char_shapes.len());

        // Assertions
        assert_eq!(model_cs_1, raw_cs_1, "Model vs raw after parse should match");
    }

    /// rp-006 BodyText 레코드 분석: dangling CharShape/ParaShape 참조 검출
    #[test]
    fn test_rp006_dangling_references() {
        use crate::parser::cfb_reader::{CfbReader, decompress_stream};
        use crate::parser::record::Record;
        use crate::parser::tags;

        let saved_path = "pasts/20250130-hongbo_saved-rp-006.hwp";
        if !std::path::Path::new(saved_path).exists() {
            eprintln!("SKIP: rp-006 파일 없음");
            return;
        }

        let saved_bytes = std::fs::read(saved_path).unwrap();
        let mut cfb = CfbReader::open(&saved_bytes).expect("CFB 열기 실패");

        // DocInfo: CharShape/ParaShape 총 수
        let doc_info_data = cfb.read_doc_info(true).expect("DocInfo 읽기 실패");
        let doc_recs = Record::read_all(&doc_info_data).unwrap();

        let cs_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
        let ps_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_SHAPE).count();
        let bf_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_BORDER_FILL).count();
        eprintln!("\n=== rp-006 DocInfo: CharShape={}, ParaShape={}, BorderFill={} ===", cs_count, ps_count, bf_count);

        // BodyText Section0
        let body_data = cfb.read_body_text_section(0, true, false).expect("BodyText 읽기 실패");
        let body_recs = Record::read_all(&body_data).unwrap();
        eprintln!("BodyText 레코드 총 수: {}", body_recs.len());

        // 모든 PARA_HEADER에서 para_shape_id 추출
        let mut dangling_ps = Vec::new();
        let mut dangling_cs = Vec::new();
        let mut dangling_bf = Vec::new();
        let mut para_idx = 0;

        for (ri, rec) in body_recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.data.len() >= 10 {
                let ps_id = u16::from_le_bytes([rec.data[8], rec.data[9]]) as usize;
                if ps_id >= ps_count {
                    dangling_ps.push((para_idx, ri, ps_id));
                }
                para_idx += 1;
            }
            // PARA_CHAR_SHAPE: 각 4바이트 쌍 (start_pos u32 + char_shape_id u32)
            if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                let mut pos = 0;
                while pos + 8 <= rec.data.len() {
                    let cs_id = u32::from_le_bytes([
                        rec.data[pos + 4], rec.data[pos + 5],
                        rec.data[pos + 6], rec.data[pos + 7],
                    ]) as usize;
                    if cs_id >= cs_count {
                        dangling_cs.push((ri, pos / 8, cs_id));
                    }
                    pos += 8;
                }
            }
            // LIST_HEADER의 border_fill_id (셀) 및 TABLE의 border_fill_id
            if rec.tag_id == tags::HWPTAG_LIST_HEADER && rec.data.len() >= 34 {
                let bf_id = u16::from_le_bytes([rec.data[32], rec.data[33]]) as usize;
                if bf_id > 0 && bf_id - 1 >= bf_count {
                    dangling_bf.push((ri, "LIST_HEADER", bf_id));
                }
            }
        }

        eprintln!("\n--- Dangling ParaShape References ---");
        if dangling_ps.is_empty() {
            eprintln!("  None found (OK)");
        } else {
            for (pi, ri, ps_id) in &dangling_ps {
                eprintln!("  para[{}] rec[{}]: para_shape_id={} >= max {}", pi, ri, ps_id, ps_count);
            }
        }

        eprintln!("\n--- Dangling CharShape References ---");
        if dangling_cs.is_empty() {
            eprintln!("  None found (OK)");
        } else {
            for (ri, entry, cs_id) in &dangling_cs {
                eprintln!("  rec[{}] entry[{}]: char_shape_id={} >= max {}", ri, entry, cs_id, cs_count);
            }
        }

        eprintln!("\n--- Dangling BorderFill References ---");
        if dangling_bf.is_empty() {
            eprintln!("  None found (OK)");
        } else {
            for (ri, source, bf_id) in &dangling_bf {
                eprintln!("  rec[{}] {}: border_fill_id={} >= max {}", ri, source, bf_id, bf_count);
            }
        }

        // 마지막 TABLE + 셀들의 레코드 덤프 (붙여넣기된 표 추정)
        eprintln!("\n--- Last 100 records (pasted table area) ---");
        let start = if body_recs.len() > 100 { body_recs.len() - 100 } else { 0 };
        for (i, rec) in body_recs[start..].iter().enumerate() {
            let indent = "  ".repeat(rec.level as usize);
            let tag_name = tags::tag_name(rec.tag_id);
            let extra = if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.data.len() >= 12 {
                let cc = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                let cm = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                let ps_id = u16::from_le_bytes([rec.data[8], rec.data[9]]);
                let style_id = rec.data[10];
                let char_count = cc & 0x7FFFFFFF;
                let msb = cc >> 31;
                format!(" char_count={} msb={} ctrl_mask=0x{:08X} ps_id={} style_id={}",
                    char_count, msb, cm, ps_id, style_id)
            } else if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                let rev_bytes: Vec<u8> = rec.data[0..4].iter().rev().cloned().collect();
                let ctrl_str = String::from_utf8_lossy(&rev_bytes);
                format!(" ctrl_id=0x{:08X}('{}')", ctrl_id, ctrl_str)
            } else if rec.tag_id == tags::HWPTAG_TABLE && rec.data.len() >= 8 {
                let nrows = u16::from_le_bytes([rec.data[4], rec.data[5]]);
                let ncols = u16::from_le_bytes([rec.data[6], rec.data[7]]);
                format!(" rows={} cols={}", nrows, ncols)
            } else if rec.tag_id == tags::HWPTAG_LIST_HEADER && rec.data.len() >= 8 {
                let nparas = u16::from_le_bytes([rec.data[0], rec.data[1]]);
                format!(" n_paras={}", nparas)
            } else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                let n_entries = rec.data.len() / 8;
                let mut ids: Vec<u32> = Vec::new();
                let mut pos = 0;
                while pos + 8 <= rec.data.len() {
                    ids.push(u32::from_le_bytes([
                        rec.data[pos + 4], rec.data[pos + 5],
                        rec.data[pos + 6], rec.data[pos + 7],
                    ]));
                    pos += 8;
                }
                format!(" entries={} cs_ids={:?}", n_entries, ids)
            } else {
                String::new()
            };
            eprintln!("  rec[{}] {}L{} {} ({}B){}", start + i, indent, rec.level, tag_name, rec.data.len(), extra);
        }

        // Summary assertion
        let total_dangling = dangling_cs.len() + dangling_ps.len();
        if total_dangling > 0 {
            eprintln!("\n*** FOUND {} DANGLING REFERENCES ***", total_dangling);
        }
    }

    /// template 파일 비교: step1 (원본 2x2표) vs step1-p (HWP 붙여넣기) vs step1_saved (우리 뷰어 붙여넣기)
    #[test]
    fn test_template_comparison() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let files = [
            ("step1_saved (뷰어 저장/손상)", "template/empty-step1_saved.hwp"),
            ("step1_saved-a (HWP 다른이름저장/정상)", "template/empty-step1_saved-a.hwp"),
        ];

        for (label, path) in &files {
            if !std::path::Path::new(path).exists() {
                eprintln!("SKIP: {} 파일 없음", path);
                continue;
            }

            let bytes = std::fs::read(path).unwrap();
            let mut cfb = CfbReader::open(&bytes).expect(&format!("{} CFB 열기 실패", label));

            // DocInfo 분석
            let doc_info_data = cfb.read_doc_info(true).expect("DocInfo 읽기 실패");
            let doc_recs = Record::read_all(&doc_info_data).unwrap();

            let cs_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
            let ps_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_SHAPE).count();
            let bf_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_BORDER_FILL).count();
            let style_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_STYLE).count();

            eprintln!("\n{}", "=".repeat(80));
            eprintln!("  {} ({} bytes)", label, bytes.len());
            eprintln!("  DocInfo: CS={} PS={} BF={} Style={}", cs_count, ps_count, bf_count, style_count);

            // BodyText Section0 분석
            let body_data = cfb.read_body_text_section(0, true, false).expect("BodyText 읽기 실패");
            let body_recs = Record::read_all(&body_data).unwrap();

            eprintln!("  BodyText: {} records, {} bytes", body_recs.len(), body_data.len());

            // 전체 레코드 덤프
            eprintln!("\n  --- ALL RECORDS ---");
            for (i, rec) in body_recs.iter().enumerate() {
                let indent = "  ".repeat(rec.level as usize);
                let tag_name = tags::tag_name(rec.tag_id);
                let extra = if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.data.len() >= 12 {
                    let cc = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    let cm = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                    let ps_id = u16::from_le_bytes([rec.data[8], rec.data[9]]);
                    let style = rec.data[10];
                    let char_count = cc & 0x7FFFFFFF;
                    let msb = cc >> 31;
                    format!(" cc={} msb={} cm=0x{:08X} ps={} st={}", char_count, msb, cm, ps_id, style)
                } else if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    let rev: Vec<u8> = rec.data[0..4].iter().rev().cloned().collect();
                    let ctrl_str = String::from_utf8_lossy(&rev);
                    if rec.data.len() >= 8 {
                        let attr = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                        format!(" '{}' attr=0x{:08X}", ctrl_str, attr)
                    } else {
                        format!(" '{}'", ctrl_str)
                    }
                } else if rec.tag_id == tags::HWPTAG_TABLE && rec.data.len() >= 8 {
                    let attr = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    let nrows = u16::from_le_bytes([rec.data[4], rec.data[5]]);
                    let ncols = u16::from_le_bytes([rec.data[6], rec.data[7]]);
                    format!(" attr=0x{:08X} {}x{}", attr, nrows, ncols)
                } else if rec.tag_id == tags::HWPTAG_LIST_HEADER && rec.data.len() >= 2 {
                    let nparas = u16::from_le_bytes([rec.data[0], rec.data[1]]);
                    format!(" nparas={}", nparas)
                } else if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                    // 첫 20바이트 hex 덤프
                    let hex: String = rec.data.iter().take(20)
                        .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    format!(" [{}{}]", hex, if rec.data.len() > 20 { "..." } else { "" })
                } else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                    let mut ids = Vec::new();
                    let mut pos = 0;
                    while pos + 8 <= rec.data.len() {
                        let cs_id = u32::from_le_bytes([
                            rec.data[pos + 4], rec.data[pos + 5],
                            rec.data[pos + 6], rec.data[pos + 7],
                        ]);
                        ids.push(cs_id);
                        pos += 8;
                    }
                    format!(" cs_ids={:?}", ids)
                } else {
                    String::new()
                };
                eprintln!("  rec[{:3}] {}L{} {} ({}B){}", i, indent, rec.level, tag_name, rec.data.len(), extra);
            }

            // CTRL_HEADER 바이트 덤프 (tbl 컨트롤만)
            eprintln!("\n  --- TABLE CTRL_HEADER hex dump ---");
            for (i, rec) in body_recs.iter().enumerate() {
                if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    if ctrl_id == 0x6C626174 { // 'tbl '
                        let hex: String = rec.data.iter()
                            .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        eprintln!("  rec[{:3}] CTRL_HEADER(tbl) {}B: {}", i, rec.data.len(), hex);
                    }
                }
                if rec.tag_id == tags::HWPTAG_TABLE {
                    let hex: String = rec.data.iter()
                        .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    eprintln!("  rec[{:3}] TABLE {}B: {}", i, rec.data.len(), hex);
                }
                if rec.tag_id == tags::HWPTAG_LIST_HEADER {
                    let hex: String = rec.data.iter()
                        .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    eprintln!("  rec[{:3}] LIST_HEADER {}B: {}", i, rec.data.len(), hex);
                }
            }

            eprintln!("\n{}", "=".repeat(80));
        }
    }

    /// 손상 HWP vs 정상 HWP 종합 비교 (DocInfo ID_MAPPINGS + BodyText Section 0 전체 레코드)
    #[test]
    fn test_complex_comparison() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let damaged_path = "template/20250130-hongbo_saved_err.hwp";
        let fixed_path = "template/111111.hwp";

        if !std::path::Path::new(damaged_path).exists() {
            eprintln!("SKIP: {} not found", damaged_path);
            return;
        }
        if !std::path::Path::new(fixed_path).exists() {
            eprintln!("SKIP: {} not found", fixed_path);
            return;
        }

        let damaged_bytes = std::fs::read(damaged_path).unwrap();
        let fixed_bytes = std::fs::read(fixed_path).unwrap();

        let mut damaged_cfb = CfbReader::open(&damaged_bytes).expect("damaged CFB open failed");
        let mut fixed_cfb = CfbReader::open(&fixed_bytes).expect("fixed CFB open failed");

        eprintln!("\n{}", "=".repeat(90));
        eprintln!("  COMPLEX COMPARISON: Damaged vs Fixed HWP");
        eprintln!("  Damaged: {} ({} bytes)", damaged_path, damaged_bytes.len());
        eprintln!("  Fixed:   {} ({} bytes)", fixed_path, fixed_bytes.len());
        eprintln!("{}", "=".repeat(90));

        // =====================================================================
        // Part 1: DocInfo - ID_MAPPINGS counts comparison
        // =====================================================================
        eprintln!("\n{}", "=".repeat(90));
        eprintln!("  PART 1: DocInfo ID_MAPPINGS Comparison");
        eprintln!("{}", "=".repeat(90));

        let damaged_di = damaged_cfb.read_doc_info(true).expect("damaged DocInfo read failed");
        let fixed_di = fixed_cfb.read_doc_info(true).expect("fixed DocInfo read failed");

        let damaged_di_recs = Record::read_all(&damaged_di).unwrap();
        let fixed_di_recs = Record::read_all(&fixed_di).unwrap();

        eprintln!("  Damaged DocInfo: {} records, {} bytes", damaged_di_recs.len(), damaged_di.len());
        eprintln!("  Fixed   DocInfo: {} records, {} bytes", fixed_di_recs.len(), fixed_di.len());

        // Count records by tag type
        let tag_types_of_interest: Vec<(u16, &str)> = vec![
            (tags::HWPTAG_BIN_DATA, "BinData"),
            (tags::HWPTAG_FACE_NAME, "FaceName"),
            (tags::HWPTAG_BORDER_FILL, "BorderFill"),
            (tags::HWPTAG_CHAR_SHAPE, "CharShape"),
            (tags::HWPTAG_TAB_DEF, "TabDef"),
            (tags::HWPTAG_NUMBERING, "Numbering"),
            (tags::HWPTAG_BULLET, "Bullet"),
            (tags::HWPTAG_PARA_SHAPE, "ParaShape"),
            (tags::HWPTAG_STYLE, "Style"),
        ];

        eprintln!("\n  {:<20} {:>10} {:>10} {:>10}", "Record Type", "Damaged", "Fixed", "Diff");
        eprintln!("  {}", "-".repeat(55));
        let mut docinfo_diff_count = 0;
        for (tag_id, name) in &tag_types_of_interest {
            let d_cnt = damaged_di_recs.iter().filter(|r| r.tag_id == *tag_id).count();
            let f_cnt = fixed_di_recs.iter().filter(|r| r.tag_id == *tag_id).count();
            let diff = f_cnt as i64 - d_cnt as i64;
            let marker = if diff != 0 { " <== DIFF" } else { "" };
            if diff != 0 { docinfo_diff_count += 1; }
            eprintln!("  {:<20} {:>10} {:>10} {:>+10}{}", name, d_cnt, f_cnt, diff, marker);
        }

        // ID_MAPPINGS record comparison
        let id_mappings_field_names = [
            "BinData", "Font_Korean", "Font_English", "Font_Hanja",
            "Font_Japanese", "Font_Other", "Font_Symbol", "Font_User",
            "BorderFill", "CharShape", "TabDef", "Numbering",
            "Bullet", "ParaShape", "Style", "MemoShape",
            "Field16", "Field17", "Field18", "Field19",
        ];

        let damaged_idm = damaged_di_recs.iter().find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS);
        let fixed_idm = fixed_di_recs.iter().find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS);

        if let (Some(d_rec), Some(f_rec)) = (damaged_idm, fixed_idm) {
            eprintln!("\n  ID_MAPPINGS record: damaged={}B, fixed={}B", d_rec.data.len(), f_rec.data.len());
            let max_fields = (d_rec.data.len().max(f_rec.data.len())) / 4;
            let max_fields = max_fields.min(20);

            eprintln!("  {:<5} {:<20} {:>10} {:>10} {:>10}", "Idx", "Field", "Damaged", "Fixed", "Diff");
            eprintln!("  {}", "-".repeat(60));

            for i in 0..max_fields {
                let d_val = if i * 4 + 4 <= d_rec.data.len() {
                    u32::from_le_bytes([d_rec.data[i*4], d_rec.data[i*4+1], d_rec.data[i*4+2], d_rec.data[i*4+3]])
                } else { 0 };
                let f_val = if i * 4 + 4 <= f_rec.data.len() {
                    u32::from_le_bytes([f_rec.data[i*4], f_rec.data[i*4+1], f_rec.data[i*4+2], f_rec.data[i*4+3]])
                } else { 0 };
                let diff = f_val as i64 - d_val as i64;
                let name = if i < id_mappings_field_names.len() { id_mappings_field_names[i] } else { "???" };
                let marker = if diff != 0 { " <== DIFF" } else { "" };
                if diff != 0 { docinfo_diff_count += 1; }
                eprintln!("  [{:>2}] {:<20} {:>10} {:>10} {:>+10}{}", i, name, d_val, f_val, diff, marker);
            }
        } else {
            eprintln!("  ERROR: ID_MAPPINGS record not found in one or both files!");
        }

        // =====================================================================
        // Part 2: BodyText Section 0 - every record comparison
        // =====================================================================
        eprintln!("\n{}", "=".repeat(90));
        eprintln!("  PART 2: BodyText Section 0 - Record-by-Record Comparison");
        eprintln!("{}", "=".repeat(90));

        let damaged_bt = damaged_cfb.read_body_text_section(0, true, false).expect("damaged BodyText read failed");
        let fixed_bt = fixed_cfb.read_body_text_section(0, true, false).expect("fixed BodyText read failed");

        let damaged_bt_recs = Record::read_all(&damaged_bt).unwrap();
        let fixed_bt_recs = Record::read_all(&fixed_bt).unwrap();

        eprintln!("  Damaged BodyText: {} records, {} bytes", damaged_bt_recs.len(), damaged_bt.len());
        eprintln!("  Fixed   BodyText: {} records, {} bytes", fixed_bt_recs.len(), fixed_bt.len());

        let max_recs = damaged_bt_recs.len().max(fixed_bt_recs.len());
        let mut body_diff_count = 0;

        eprintln!("\n  --- Record-by-record comparison ---");
        eprintln!("  {:<6} {:<25} {:<6} {:<8} | {:<25} {:<6} {:<8} | {}",
            "Idx", "Damaged Tag", "Lvl", "Size", "Fixed Tag", "Lvl", "Size", "Differences");
        eprintln!("  {}", "-".repeat(120));

        for i in 0..max_recs {
            let d_rec = damaged_bt_recs.get(i);
            let f_rec = fixed_bt_recs.get(i);

            match (d_rec, f_rec) {
                (Some(d), Some(f)) => {
                    let d_tag_name = tags::tag_name(d.tag_id);
                    let f_tag_name = tags::tag_name(f.tag_id);
                    let mut diffs: Vec<String> = Vec::new();

                    if d.tag_id != f.tag_id {
                        diffs.push(format!("tag: {}!={}", d.tag_id, f.tag_id));
                    }
                    if d.level != f.level {
                        diffs.push(format!("level: {}!={}", d.level, f.level));
                    }
                    if d.data.len() != f.data.len() {
                        diffs.push(format!("size: {}!={}", d.data.len(), f.data.len()));
                    }
                    if d.data != f.data {
                        diffs.push("bytes differ".to_string());
                    }

                    // PARA_HEADER detailed comparison
                    if d.tag_id == tags::HWPTAG_PARA_HEADER && f.tag_id == tags::HWPTAG_PARA_HEADER {
                        if d.data.len() >= 11 && f.data.len() >= 11 {
                            let d_cc_raw = u32::from_le_bytes([d.data[0], d.data[1], d.data[2], d.data[3]]);
                            let f_cc_raw = u32::from_le_bytes([f.data[0], f.data[1], f.data[2], f.data[3]]);
                            let d_char_count = d_cc_raw & 0x7FFFFFFF;
                            let f_char_count = f_cc_raw & 0x7FFFFFFF;
                            let d_msb = d_cc_raw >> 31;
                            let f_msb = f_cc_raw >> 31;
                            let d_cm = u32::from_le_bytes([d.data[4], d.data[5], d.data[6], d.data[7]]);
                            let f_cm = u32::from_le_bytes([f.data[4], f.data[5], f.data[6], f.data[7]]);
                            let d_ps_id = u16::from_le_bytes([d.data[8], d.data[9]]);
                            let f_ps_id = u16::from_le_bytes([f.data[8], f.data[9]]);
                            let d_style = d.data[10];
                            let f_style = f.data[10];

                            if d_char_count != f_char_count {
                                diffs.push(format!("char_count: {}!={}", d_char_count, f_char_count));
                            }
                            if d_msb != f_msb {
                                diffs.push(format!("msb: {}!={}", d_msb, f_msb));
                            }
                            if d_cm != f_cm {
                                diffs.push(format!("ctrl_mask: 0x{:08X}!=0x{:08X}", d_cm, f_cm));
                            }
                            if d_ps_id != f_ps_id {
                                diffs.push(format!("para_shape_id: {}!={}", d_ps_id, f_ps_id));
                            }
                            if d_style != f_style {
                                diffs.push(format!("style_id: {}!={}", d_style, f_style));
                            }
                        }
                    }

                    let diff_str = if diffs.is_empty() {
                        "OK".to_string()
                    } else {
                        body_diff_count += 1;
                        format!("DIFF: {}", diffs.join(", "))
                    };

                    // Always print if there is a difference; for matching records print a compact line
                    if !diffs.is_empty() {
                        eprintln!("  [{:>4}] {:<25} L{:<4} {:>6}B | {:<25} L{:<4} {:>6}B | {}",
                            i, d_tag_name, d.level, d.data.len(),
                            f_tag_name, f.level, f.data.len(), diff_str);
                    } else {
                        eprintln!("  [{:>4}] {:<25} L{:<4} {:>6}B | {:<25} L{:<4} {:>6}B | OK",
                            i, d_tag_name, d.level, d.data.len(),
                            f_tag_name, f.level, f.data.len());
                    }
                }
                (Some(d), None) => {
                    body_diff_count += 1;
                    let d_tag_name = tags::tag_name(d.tag_id);
                    eprintln!("  [{:>4}] {:<25} L{:<4} {:>6}B | {:<25}                    | ONLY IN DAMAGED",
                        i, d_tag_name, d.level, d.data.len(), "---");
                }
                (None, Some(f)) => {
                    body_diff_count += 1;
                    let f_tag_name = tags::tag_name(f.tag_id);
                    eprintln!("  [{:>4}] {:<25}                    | {:<25} L{:<4} {:>6}B | ONLY IN FIXED",
                        i, "---", f_tag_name, f.level, f.data.len());
                }
                (None, None) => {}
            }
        }

        // =====================================================================
        // Part 3: TABLE/CTRL_HEADER raw bytes comparison
        // =====================================================================
        eprintln!("\n{}", "=".repeat(90));
        eprintln!("  PART 3: TABLE / CTRL_HEADER Raw Bytes Comparison");
        eprintln!("{}", "=".repeat(90));

        // Collect TABLE and CTRL_HEADER records from both files
        let interesting_tags = [tags::HWPTAG_TABLE, tags::HWPTAG_CTRL_HEADER];

        let damaged_interesting: Vec<(usize, &Record)> = damaged_bt_recs.iter().enumerate()
            .filter(|(_, r)| interesting_tags.contains(&r.tag_id))
            .collect();
        let fixed_interesting: Vec<(usize, &Record)> = fixed_bt_recs.iter().enumerate()
            .filter(|(_, r)| interesting_tags.contains(&r.tag_id))
            .collect();

        // Match records by index position in the record stream
        let max_interesting = damaged_interesting.len().max(fixed_interesting.len());

        for j in 0..max_interesting {
            let d_item = damaged_interesting.get(j);
            let f_item = fixed_interesting.get(j);

            match (d_item, f_item) {
                (Some(&(d_idx, d_rec)), Some(&(f_idx, f_rec))) => {
                    let d_tag_name = tags::tag_name(d_rec.tag_id);
                    let f_tag_name = tags::tag_name(f_rec.tag_id);

                    // For CTRL_HEADER, show the ctrl type string
                    let d_ctrl_type = if d_rec.tag_id == tags::HWPTAG_CTRL_HEADER && d_rec.data.len() >= 4 {
                        let rev: Vec<u8> = d_rec.data[0..4].iter().rev().cloned().collect();
                        format!(" '{}'", String::from_utf8_lossy(&rev))
                    } else { String::new() };
                    let f_ctrl_type = if f_rec.tag_id == tags::HWPTAG_CTRL_HEADER && f_rec.data.len() >= 4 {
                        let rev: Vec<u8> = f_rec.data[0..4].iter().rev().cloned().collect();
                        format!(" '{}'", String::from_utf8_lossy(&rev))
                    } else { String::new() };

                    let same = d_rec.data == f_rec.data;
                    eprintln!("\n  Pair {}: damaged[{}] {}{} ({}B) vs fixed[{}] {}{} ({}B) => {}",
                        j, d_idx, d_tag_name, d_ctrl_type, d_rec.data.len(),
                        f_idx, f_tag_name, f_ctrl_type, f_rec.data.len(),
                        if same { "IDENTICAL" } else { "DIFFERENT" });

                    if !same {
                        body_diff_count += 1;
                        // Show byte-level diff
                        let max_len = d_rec.data.len().max(f_rec.data.len());
                        let mut diff_positions: Vec<usize> = Vec::new();
                        for pos in 0..max_len {
                            let d_byte = d_rec.data.get(pos);
                            let f_byte = f_rec.data.get(pos);
                            if d_byte != f_byte {
                                diff_positions.push(pos);
                            }
                        }
                        eprintln!("    {} byte(s) differ at positions: {:?}",
                            diff_positions.len(),
                            if diff_positions.len() <= 30 { &diff_positions[..] } else { &diff_positions[..30] });

                        // Hex dump of first 80 bytes for both
                        let dump_len = 80.min(max_len);
                        let d_hex: String = d_rec.data.iter().take(dump_len)
                            .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        let f_hex: String = f_rec.data.iter().take(dump_len)
                            .map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        eprintln!("    Damaged (first {}B): {}{}", dump_len, d_hex,
                            if d_rec.data.len() > dump_len { "..." } else { "" });
                        eprintln!("    Fixed   (first {}B): {}{}", dump_len, f_hex,
                            if f_rec.data.len() > dump_len { "..." } else { "" });
                    }
                }
                (Some(&(d_idx, d_rec)), None) => {
                    let d_tag_name = tags::tag_name(d_rec.tag_id);
                    eprintln!("\n  Pair {}: damaged[{}] {} ({}B) -- NO MATCH IN FIXED",
                        j, d_idx, d_tag_name, d_rec.data.len());
                }
                (None, Some(&(f_idx, f_rec))) => {
                    let f_tag_name = tags::tag_name(f_rec.tag_id);
                    eprintln!("\n  Pair {}: -- NO MATCH IN DAMAGED -- fixed[{}] {} ({}B)",
                        j, f_idx, f_tag_name, f_rec.data.len());
                }
                _ => {}
            }
        }

        // =====================================================================
        // Summary
        // =====================================================================
        eprintln!("\n{}", "=".repeat(90));
        eprintln!("  SUMMARY");
        eprintln!("  DocInfo differences:  {}", docinfo_diff_count);
        eprintln!("  BodyText differences: {}", body_diff_count);
        eprintln!("  Total records: damaged={}, fixed={}", damaged_bt_recs.len(), fixed_bt_recs.len());
        eprintln!("{}", "=".repeat(90));
    }


    /// rp-004 저장 파일의 BodyText 레코드를 분석하여 붙여넣기된 표의 구조적 문제를 찾는다.
    #[test]
    fn test_rp004_bodytext_table_analysis() {
        use crate::parser::record::Record;
        use crate::parser::tags;

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        let saved_path = "pasts/20250130-hongbo_saved-rp-004.hwp";

        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 원본 파일 없음 ({})", orig_path);
            return;
        }
        if !std::path::Path::new(saved_path).exists() {
            eprintln!("SKIP: 저장 파일 없음 ({})", saved_path);
            return;
        }

        // ============================================================
        // 1. 두 파일 파싱
        // ============================================================
        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();
        eprintln!("원본 파일 크기: {} bytes", orig_data.len());
        eprintln!("저장 파일 크기: {} bytes", saved_data.len());

        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        eprintln!("원본: sections={}, compressed={}", orig_doc.sections.len(), orig_doc.header.compressed);
        eprintln!("저장: sections={}, compressed={}", saved_doc.sections.len(), saved_doc.header.compressed);

        // ============================================================
        // 2. BodyText Section[0] raw stream 읽기 및 Record 스캔
        // ============================================================
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();
        eprintln!("\n원본 BodyText Section[0]: {} bytes, {} records", orig_bt.len(), orig_recs.len());

        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();
        eprintln!("저장 BodyText Section[0]: {} bytes, {} records", saved_bt.len(), saved_recs.len());

        // ============================================================
        // 2a. 모든 레코드 목록 출력 (저장 파일)
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 저장 파일 전체 레코드 목록 (Section[0]) ===");
        eprintln!("{}", "=".repeat(120));
        for (i, r) in saved_recs.iter().enumerate() {
            let tname = tags::tag_name(r.tag_id);
            let extra = if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                let ctrl_bytes = [r.data[0], r.data[1], r.data[2], r.data[3]];
                let ctrl_str: String = ctrl_bytes.iter().rev().map(|&b| {
                    if b >= 0x20 && b <= 0x7e { b as char } else { '.' }
                }).collect();
                format!(" ctrl_id=0x{:08X} \"{}\" ({})", ctrl_id, ctrl_str, tags::ctrl_name(ctrl_id))
            } else if r.tag_id == tags::HWPTAG_PARA_HEADER && r.data.len() >= 4 {
                let nchars = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                let nchars_val = nchars & 0x7FFFFFFF;
                let control_mask = if r.data.len() >= 8 {
                    u32::from_le_bytes([r.data[4], r.data[5], r.data[6], r.data[7]])
                } else { 0 };
                format!(" char_count={} (raw=0x{:08X}) control_mask=0x{:08X}", nchars_val, nchars, control_mask)
            } else if r.tag_id == tags::HWPTAG_TABLE && r.data.len() >= 8 {
                let flags = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                let nrows = u16::from_le_bytes([r.data[4], r.data[5]]);
                let ncols = u16::from_le_bytes([r.data[6], r.data[7]]);
                format!(" flags=0x{:08X} rows={} cols={}", flags, nrows, ncols)
            } else if r.tag_id == tags::HWPTAG_LIST_HEADER && r.data.len() >= 4 {
                let nparas = u16::from_le_bytes([r.data[0], r.data[1]]);
                let flags = u16::from_le_bytes([r.data[2], r.data[3]]);
                format!(" nparas={} flags=0x{:04X}", nparas, flags)
            } else {
                String::new()
            };
            eprintln!("  [{:4}] tag=0x{:04X} {:30} L{:<3} {:6}B{}",
                i, r.tag_id, tname, r.level, r.data.len(), extra);
        }

        // ============================================================
        // 2b. 레코드 타입별 카운트 비교
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 레코드 타입별 카운트 비교 (원본 vs 저장) ===");
        eprintln!("{}", "=".repeat(120));
        let count_tag = |recs: &[Record], tag: u16| recs.iter().filter(|r| r.tag_id == tag).count();
        let tags_to_check: [(u16, &str); 10] = [
            (tags::HWPTAG_PARA_HEADER, "PARA_HEADER"),
            (tags::HWPTAG_PARA_TEXT, "PARA_TEXT"),
            (tags::HWPTAG_PARA_CHAR_SHAPE, "PARA_CHAR_SHAPE"),
            (tags::HWPTAG_PARA_LINE_SEG, "PARA_LINE_SEG"),
            (tags::HWPTAG_PARA_RANGE_TAG, "PARA_RANGE_TAG"),
            (tags::HWPTAG_CTRL_HEADER, "CTRL_HEADER"),
            (tags::HWPTAG_LIST_HEADER, "LIST_HEADER"),
            (tags::HWPTAG_TABLE, "TABLE"),
            (tags::HWPTAG_CTRL_DATA, "CTRL_DATA"),
            (tags::HWPTAG_PAGE_DEF, "PAGE_DEF"),
        ];
        for (tag, name) in &tags_to_check {
            let orig_cnt = count_tag(&orig_recs, *tag);
            let saved_cnt = count_tag(&saved_recs, *tag);
            let diff = saved_cnt as i64 - orig_cnt as i64;
            eprintln!("  {:25} orig={:4}  saved={:4}  diff={}{:+}{}",
                name, orig_cnt, saved_cnt,
                if diff != 0 { "<<< " } else { "" },
                diff,
                if diff != 0 { " >>>" } else { "" });
        }

        // ============================================================
        // 2c. 표(Table) 분석
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 표(Table) 분석 ===");
        eprintln!("{}", "=".repeat(120));

        // 원본의 표 찾기
        let orig_tables: Vec<usize> = orig_recs.iter().enumerate()
            .filter(|(_, r)| r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 && {
                let ctrl_id = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                ctrl_id == tags::CTRL_TABLE
            })
            .map(|(i, _)| i)
            .collect();

        let saved_tables: Vec<usize> = saved_recs.iter().enumerate()
            .filter(|(_, r)| r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 && {
                let ctrl_id = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                ctrl_id == tags::CTRL_TABLE
            })
            .map(|(i, _)| i)
            .collect();

        eprintln!("원본 표 개수: {}", orig_tables.len());
        eprintln!("저장 표 개수: {}", saved_tables.len());

        // 각 표의 구조 분석 함수
        let analyze_table = |recs: &[Record], tbl_start: usize, label: &str| {
            let tbl_level = recs[tbl_start].level;
            let mut tbl_end = tbl_start + 1;
            while tbl_end < recs.len() && recs[tbl_end].level > tbl_level {
                tbl_end += 1;
            }
            let tbl_record_count = tbl_end - tbl_start;

            eprintln!("\n--- {} (rec[{}..{}], {} records) ---", label, tbl_start, tbl_end, tbl_record_count);

            // CTRL_HEADER 바이트 덤프 (처음 최대 50바이트)
            let ctrl_hdr = &recs[tbl_start];
            let dump_len = ctrl_hdr.data.len().min(50);
            eprintln!("  CTRL_HEADER ({}B): {:02X?}", ctrl_hdr.data.len(), &ctrl_hdr.data[..dump_len]);

            // TABLE 레코드 찾기
            let mut table_rec_idx = None;
            let mut list_headers: Vec<usize> = Vec::new();

            for ri in tbl_start+1..tbl_end {
                if recs[ri].tag_id == tags::HWPTAG_TABLE && recs[ri].level == tbl_level + 1 {
                    table_rec_idx = Some(ri);
                }
                if recs[ri].tag_id == tags::HWPTAG_LIST_HEADER && recs[ri].level == tbl_level + 1 {
                    list_headers.push(ri);
                }
            }

            if let Some(tri) = table_rec_idx {
                let td = &recs[tri].data;
                let dump_len2 = td.len().min(80);
                eprintln!("  TABLE record (rec[{}], {}B): {:02X?}", tri, td.len(), &td[..dump_len2]);
                if td.len() >= 8 {
                    let flags = u32::from_le_bytes([td[0], td[1], td[2], td[3]]);
                    let nrows = u16::from_le_bytes([td[4], td[5]]);
                    let ncols = u16::from_le_bytes([td[6], td[7]]);
                    eprintln!("    flags=0x{:08X} rows={} cols={} (expected cells={})", flags, nrows, ncols, nrows as u32 * ncols as u32);

                    if td.len() >= 10 {
                        let border_fill_id = u16::from_le_bytes([td[8], td[9]]);
                        eprintln!("    border_fill_id={}", border_fill_id);
                    }
                    if td.len() > 10 {
                        eprintln!("    remaining bytes (offset 10..): {:02X?}", &td[10..td.len().min(80)]);
                    }
                }
            } else {
                eprintln!("  TABLE record: NOT FOUND!");
            }

            // 각 셀(LIST_HEADER) 분석
            eprintln!("  셀 개수 (LIST_HEADER at tbl_level+1): {}", list_headers.len());
            for (ci, &lhi) in list_headers.iter().enumerate() {
                let lh = &recs[lhi];
                let dump_len3 = lh.data.len().min(40);
                eprintln!("  Cell[{}] LIST_HEADER (rec[{}], {}B): {:02X?}", ci, lhi, lh.data.len(), &lh.data[..dump_len3]);

                if lh.data.len() >= 4 {
                    let nparas = u16::from_le_bytes([lh.data[0], lh.data[1]]);
                    let flags = u16::from_le_bytes([lh.data[2], lh.data[3]]);
                    eprintln!("    nparas={} flags=0x{:04X}", nparas, flags);
                }

                // 이 셀에 속하는 PARA_HEADER 찾기
                let cell_level = lh.level;
                let next_boundary = if ci + 1 < list_headers.len() {
                    list_headers[ci + 1]
                } else {
                    tbl_end
                };

                let mut para_count = 0;
                for ri2 in lhi+1..next_boundary {
                    if recs[ri2].tag_id == tags::HWPTAG_PARA_HEADER && recs[ri2].level == cell_level + 1 {
                        let ph = &recs[ri2];
                        let nchars = if ph.data.len() >= 4 {
                            u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]]) & 0x7FFFFFFF
                        } else { 0 };
                        let control_mask = if ph.data.len() >= 8 {
                            u32::from_le_bytes([ph.data[4], ph.data[5], ph.data[6], ph.data[7]])
                        } else { 0 };
                        eprintln!("      PARA[{}] (rec[{}]) char_count={} control_mask=0x{:08X}", para_count, ri2, nchars, control_mask);
                        para_count += 1;
                    }
                }
            }

            (tbl_start, tbl_end, tbl_record_count)
        };

        // 원본 표 분석
        for (ti, &tbl_start) in orig_tables.iter().enumerate() {
            analyze_table(&orig_recs, tbl_start, &format!("원본 표[{}]", ti));
        }

        // 저장 표 분석
        for (ti, &tbl_start) in saved_tables.iter().enumerate() {
            analyze_table(&saved_recs, tbl_start, &format!("저장 표[{}]", ti));
        }

        // ============================================================
        // 2d. 마지막 표 (붙여넣기된 것) 전체 레코드 덤프
        // ============================================================
        if let Some(&last_tbl_idx) = saved_tables.last() {
            let tbl_level = saved_recs[last_tbl_idx].level;
            let mut tbl_end = last_tbl_idx + 1;
            while tbl_end < saved_recs.len() && saved_recs[tbl_end].level > tbl_level {
                tbl_end += 1;
            }
            eprintln!("\n{}", "=".repeat(120));
            eprintln!("=== 마지막(붙여넣기) 표: rec[{}..{}] 전체 레코드 덤프 ===", last_tbl_idx, tbl_end);
            eprintln!("{}", "=".repeat(120));
            for ri in last_tbl_idx..tbl_end {
                let r = &saved_recs[ri];
                let tname = tags::tag_name(r.tag_id);
                let dump_len = r.data.len().min(64);
                let extra_info = if r.tag_id == tags::HWPTAG_PARA_HEADER && r.data.len() >= 4 {
                    let nchars = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                    let nchars_val = nchars & 0x7FFFFFFF;
                    let control_mask = if r.data.len() >= 8 {
                        u32::from_le_bytes([r.data[4], r.data[5], r.data[6], r.data[7]])
                    } else { 0 };
                    format!(" | char_count={} control_mask=0x{:08X}", nchars_val, control_mask)
                } else if r.tag_id == tags::HWPTAG_PARA_TEXT {
                    let u16_chars: Vec<u16> = r.data.chunks(2)
                        .filter(|c| c.len() == 2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect();
                    let text = String::from_utf16_lossy(&u16_chars);
                    let preview: String = text.chars().take(40).collect();
                    format!(" | text_preview=\"{}\"", preview)
                } else {
                    String::new()
                };
                eprintln!("  [{:4}] {:30} L{:<3} {:6}B  data[..{}]: {:02X?}{}",
                    ri, tname, r.level, r.data.len(), dump_len, &r.data[..dump_len], extra_info);
            }
        }

        // ============================================================
        // 3. 문단 일관성 검사 (PARA_HEADER <-> PARA_TEXT)
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 문단 일관성 검사 (저장 파일) ===");
        eprintln!("{}", "=".repeat(120));
        let mut mismatch_count = 0;
        let mut para_idx = 0;
        let mut i = 0;
        while i < saved_recs.len() {
            if saved_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph = &saved_recs[i];
                let ph_level = ph.level;
                let nchars = if ph.data.len() >= 4 {
                    u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]]) & 0x7FFFFFFF
                } else { 0 };
                let control_mask = if ph.data.len() >= 8 {
                    u32::from_le_bytes([ph.data[4], ph.data[5], ph.data[6], ph.data[7]])
                } else { 0 };

                // 다음 레코드가 PARA_TEXT인지 확인
                let has_text = i + 1 < saved_recs.len()
                    && saved_recs[i + 1].tag_id == tags::HWPTAG_PARA_TEXT
                    && saved_recs[i + 1].level == ph_level + 1;

                if has_text {
                    let pt = &saved_recs[i + 1];
                    let pt_byte_len = pt.data.len();
                    let expected_byte_len = (nchars as usize) * 2;

                    if pt_byte_len != expected_byte_len {
                        eprintln!("  MISMATCH para[{}] rec[{}]: char_count={} => expected PARA_TEXT={}B, actual={}B (diff={})",
                            para_idx, i, nchars, expected_byte_len, pt_byte_len,
                            pt_byte_len as i64 - expected_byte_len as i64);
                        // 텍스트 미리보기
                        let u16_chars: Vec<u16> = pt.data.chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        let text = String::from_utf16_lossy(&u16_chars);
                        let preview: String = text.chars().take(50).collect();
                        eprintln!("    text_preview: \"{}\"", preview);
                        mismatch_count += 1;
                    }
                } else if nchars > 1 {
                    eprintln!("  MISSING PARA_TEXT para[{}] rec[{}]: char_count={} control_mask=0x{:08X} but NO PARA_TEXT follows (next tag={})",
                        para_idx, i, nchars, control_mask,
                        if i + 1 < saved_recs.len() {
                            format!("0x{:04X} ({})", saved_recs[i+1].tag_id, tags::tag_name(saved_recs[i+1].tag_id))
                        } else { "EOF".to_string() }
                    );
                    mismatch_count += 1;
                } else if nchars == 0 {
                    // char_count=0인 PARA_HEADER (빈 문단) 확인
                    if has_text {
                        eprintln!("  UNEXPECTED para[{}] rec[{}]: char_count=0 but PARA_TEXT exists ({}B)",
                            para_idx, i, saved_recs[i+1].data.len());
                        mismatch_count += 1;
                    }
                }

                para_idx += 1;
            }
            i += 1;
        }
        eprintln!("\n  총 문단 수: {}", para_idx);
        eprintln!("  불일치 개수: {}", mismatch_count);

        // ============================================================
        // 3b. 원본 파일도 동일 검사 (비교용)
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 문단 일관성 검사 (원본 파일) ===");
        eprintln!("{}", "=".repeat(120));
        let mut orig_mismatch_count = 0;
        let mut orig_para_idx = 0;
        i = 0;
        while i < orig_recs.len() {
            if orig_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph = &orig_recs[i];
                let ph_level = ph.level;
                let nchars = if ph.data.len() >= 4 {
                    u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]]) & 0x7FFFFFFF
                } else { 0 };
                let control_mask = if ph.data.len() >= 8 {
                    u32::from_le_bytes([ph.data[4], ph.data[5], ph.data[6], ph.data[7]])
                } else { 0 };

                let has_text = i + 1 < orig_recs.len()
                    && orig_recs[i + 1].tag_id == tags::HWPTAG_PARA_TEXT
                    && orig_recs[i + 1].level == ph_level + 1;

                if has_text {
                    let pt = &orig_recs[i + 1];
                    let pt_byte_len = pt.data.len();
                    let expected_byte_len = (nchars as usize) * 2;

                    if pt_byte_len != expected_byte_len {
                        eprintln!("  MISMATCH para[{}] rec[{}]: char_count={} => expected PARA_TEXT={}B, actual={}B (diff={})",
                            orig_para_idx, i, nchars, expected_byte_len, pt_byte_len,
                            pt_byte_len as i64 - expected_byte_len as i64);
                        orig_mismatch_count += 1;
                    }
                } else if nchars > 1 {
                    eprintln!("  MISSING PARA_TEXT para[{}] rec[{}]: char_count={} control_mask=0x{:08X}",
                        orig_para_idx, i, nchars, control_mask);
                    orig_mismatch_count += 1;
                }

                orig_para_idx += 1;
            }
            i += 1;
        }
        eprintln!("\n  총 문단 수: {}", orig_para_idx);
        eprintln!("  불일치 개수: {}", orig_mismatch_count);

        // ============================================================
        // 요약
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 요약 ===");
        eprintln!("{}", "=".repeat(120));
        eprintln!("원본 표: {}개, 저장 표: {}개 (차이: {})",
            orig_tables.len(), saved_tables.len(),
            saved_tables.len() as i64 - orig_tables.len() as i64);
        eprintln!("원본 레코드: {}개, 저장 레코드: {}개 (차이: {})",
            orig_recs.len(), saved_recs.len(),
            saved_recs.len() as i64 - orig_recs.len() as i64);
        eprintln!("원본 문단 불일치: {}, 저장 문단 불일치: {}",
            orig_mismatch_count, mismatch_count);
    }

    /// CharShape 보존 검증: 붙여넣기 후 내보내기 시 원본 CharShape가 모두 보존되는지 확인
    #[test]
    fn test_charshape_preservation_after_paste() {
        use crate::parser::tags;
        use crate::parser::record::Record;

        // raw_stream에서 특정 tag의 레코드 수 세기
        fn count_tag_in_raw(raw: &[u8], target_tag: u16) -> usize {
            Record::read_all(raw).unwrap_or_default()
                .iter().filter(|r| r.tag_id == target_tag).count()
        }

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 파일 없음");
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // 원본 CharShape 개수 확인
        let orig_cs_count = doc.document.doc_info.char_shapes.len();
        let orig_ps_count = doc.document.doc_info.para_shapes.len();
        eprintln!("원본 CharShape: {}, ParaShape: {}", orig_cs_count, orig_ps_count);

        // raw_stream에서 CharShape 레코드 개수 확인
        let orig_raw_cs = doc.document.doc_info.raw_stream.as_ref()
            .map(|raw| count_tag_in_raw(raw, tags::HWPTAG_CHAR_SHAPE))
            .unwrap_or(0);
        eprintln!("원본 raw_stream CharShape 레코드: {}", orig_raw_cs);
        assert_eq!(orig_cs_count, orig_raw_cs, "모델과 raw_stream의 CharShape 개수 불일치");

        // HTML 테이블 붙여넣기
        let table_html = r#"<table><tr><td style="font-weight:bold">Bold A</td><td style="color:red">Red B</td></tr><tr><td>Cell C</td><td style="font-style:italic">Italic D</td></tr></table>"#;
        let last_para = doc.document.sections[0].paragraphs.len() - 1;
        doc.paste_html_native(0, last_para, 0, table_html).unwrap();

        // 붙여넣기 후 CharShape 개수 확인
        let post_cs_count = doc.document.doc_info.char_shapes.len();
        let post_ps_count = doc.document.doc_info.para_shapes.len();
        eprintln!("붙여넣기 후 CharShape: {}, ParaShape: {}", post_cs_count, post_ps_count);
        assert!(post_cs_count >= orig_cs_count,
            "CharShape 개수 감소! {} → {}", orig_cs_count, post_cs_count);

        // raw_stream CharShape 레코드 확인
        let post_raw_cs = doc.document.doc_info.raw_stream.as_ref()
            .map(|raw| count_tag_in_raw(raw, tags::HWPTAG_CHAR_SHAPE))
            .unwrap_or(0);
        eprintln!("붙여넣기 후 raw_stream CharShape 레코드: {}", post_raw_cs);
        assert!(post_raw_cs >= orig_raw_cs,
            "raw_stream CharShape 감소! {} → {}", orig_raw_cs, post_raw_cs);
        assert_eq!(post_cs_count, post_raw_cs,
            "붙여넣기 후 모델({})과 raw_stream({})의 CharShape 불일치", post_cs_count, post_raw_cs);

        // 내보내기 후 재파싱하여 CharShape 확인
        let saved_data = doc.export_hwp_native().unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
        let saved_cs_count = saved_doc.doc_info.char_shapes.len();
        let saved_ps_count = saved_doc.doc_info.para_shapes.len();
        eprintln!("재파싱 CharShape: {}, ParaShape: {}", saved_cs_count, saved_ps_count);
        assert!(saved_cs_count >= orig_cs_count,
            "저장 후 CharShape 감소! 원본 {} → 저장 {}", orig_cs_count, saved_cs_count);

        // 모든 PARA_CHAR_SHAPE가 유효한 CharShape ID를 참조하는지 확인
        let mut max_cs_id: u32 = 0;
        for section in &saved_doc.sections {
            for para in &section.paragraphs {
                for cs_ref in &para.char_shapes {
                    if cs_ref.char_shape_id > max_cs_id {
                        max_cs_id = cs_ref.char_shape_id;
                    }
                }
                for ctrl in &para.controls {
                    if let Control::Table(tbl) = ctrl {
                        for cell in &tbl.cells {
                            for cp in &cell.paragraphs {
                                for cs_ref in &cp.char_shapes {
                                    if cs_ref.char_shape_id > max_cs_id {
                                        max_cs_id = cs_ref.char_shape_id;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        eprintln!("최대 CharShape ID 참조: {}, 사용 가능 범위: 0..{}", max_cs_id, saved_cs_count);
        assert!((max_cs_id as usize) < saved_cs_count,
            "CharShape ID {} 참조 but 가용 개수 {}! (dangling reference)", max_cs_id, saved_cs_count);

        eprintln!("=== CharShape 보존 검증 통과 ===");
    }

    /// rp-005 저장 파일과 원본을 비교하여 붙여넣기된 표의 구조를 깊이 분석한다.
    /// DocInfo 일관성, 표 구조, 문단 char_count vs PARA_TEXT 길이, CharShape ID 유효성 검사.
    #[test]
    fn test_rp005_pasted_table_analysis() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use std::collections::HashMap;

        let orig_path = "pasts/20250130-hongbo-p2.hwp";
        let saved_path = "pasts/20250130-hongbo_saved-rp-005.hwp";

        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: 원본 파일 없음 ({})", orig_path);
            return;
        }
        if !std::path::Path::new(saved_path).exists() {
            eprintln!("SKIP: 저장 파일 없음 ({})", saved_path);
            return;
        }

        let orig_data = std::fs::read(orig_path).unwrap();
        let saved_data = std::fs::read(saved_path).unwrap();
        eprintln!("원본 파일 크기: {} bytes", orig_data.len());
        eprintln!("저장 파일 크기: {} bytes", saved_data.len());

        // ============================================================
        // 1. 두 파일 파싱 (고수준 IR)
        // ============================================================
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 1. DocInfo 비교 ===");
        eprintln!("{}", "=".repeat(120));

        let orig_cs = orig_doc.doc_info.char_shapes.len();
        let saved_cs = saved_doc.doc_info.char_shapes.len();
        let orig_ps = orig_doc.doc_info.para_shapes.len();
        let saved_ps = saved_doc.doc_info.para_shapes.len();
        let orig_bf = orig_doc.doc_info.border_fills.len();
        let saved_bf = saved_doc.doc_info.border_fills.len();
        let orig_st = orig_doc.doc_info.styles.len();
        let saved_st = saved_doc.doc_info.styles.len();

        eprintln!("  CharShape:  orig={:<5} saved={:<5} diff={:+}", orig_cs, saved_cs, saved_cs as i64 - orig_cs as i64);
        eprintln!("  ParaShape:  orig={:<5} saved={:<5} diff={:+}", orig_ps, saved_ps, saved_ps as i64 - orig_ps as i64);
        eprintln!("  BorderFill: orig={:<5} saved={:<5} diff={:+}", orig_bf, saved_bf, saved_bf as i64 - orig_bf as i64);
        eprintln!("  Styles:     orig={:<5} saved={:<5} diff={:+}", orig_st, saved_st, saved_st as i64 - orig_st as i64);

        // ID_MAPPINGS 일관성 (raw DocInfo 스트림에서 직접 파싱)
        eprintln!("\n--- ID_MAPPINGS consistency check ---");
        let check_id_mappings = |raw: &[u8], label: &str, cs_count: usize, ps_count: usize, bf_count: usize| {
            let recs = Record::read_all(raw).unwrap();
            let idm_rec = recs.iter().find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS);
            if let Some(idm) = idm_rec {
                let d = &idm.data;
                // ID_MAPPINGS fields (u32 each):
                // [0]=BinData, [1..7]=FontFace(7lang), [8]=BorderFill, [9]=CharShape,
                // [10]=TabDef, [11]=Numbering, [12]=Bullet, [13]=ParaShape, [14]=Style
                if d.len() >= 60 {
                    let bf_map = u32::from_le_bytes([d[32], d[33], d[34], d[35]]);
                    let cs_map = u32::from_le_bytes([d[36], d[37], d[38], d[39]]);
                    let ps_map = u32::from_le_bytes([d[52], d[53], d[54], d[55]]);

                    let bf_actual = recs.iter().filter(|r| r.tag_id == tags::HWPTAG_BORDER_FILL).count();
                    let cs_actual = recs.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
                    let ps_actual = recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_SHAPE).count();

                    eprintln!("  [{}] ID_MAPPINGS: BorderFill={}, CharShape={}, ParaShape={}", label, bf_map, cs_map, ps_map);
                    eprintln!("  [{}] actual recs: BorderFill={}, CharShape={}, ParaShape={}", label, bf_actual, cs_actual, ps_actual);
                    eprintln!("  [{}] model count: BorderFill={}, CharShape={}, ParaShape={}", label, bf_count, ps_count, cs_count);

                    let bf_ok = bf_map as usize == bf_actual && bf_actual == bf_count;
                    let cs_ok = cs_map as usize == cs_actual && cs_actual == cs_count;
                    let ps_ok = ps_map as usize == ps_actual && ps_actual == ps_count;
                    eprintln!("  [{}] consistency: BF={} CS={} PS={}", label,
                        if bf_ok { "OK" } else { "MISMATCH!" },
                        if cs_ok { "OK" } else { "MISMATCH!" },
                        if ps_ok { "OK" } else { "MISMATCH!" });
                }
            } else {
                eprintln!("  [{}] ID_MAPPINGS not found!", label);
            }
        };

        if let Some(ref raw) = orig_doc.doc_info.raw_stream {
            check_id_mappings(raw, "orig", orig_cs, orig_ps, orig_bf);
        }
        if let Some(ref raw) = saved_doc.doc_info.raw_stream {
            check_id_mappings(raw, "saved", saved_cs, saved_ps, saved_bf);
        }

        // ============================================================
        // 2. BodyText raw records 읽기
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 2. BodyText 레코드 분석 ===");
        eprintln!("{}", "=".repeat(120));

        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("  원본 BodyText: {} bytes, {} records", orig_bt.len(), orig_recs.len());
        eprintln!("  저장 BodyText: {} bytes, {} records", saved_bt.len(), saved_recs.len());

        // ============================================================
        // 3. 모든 표 찾기
        // ============================================================
        let find_tables = |recs: &[Record]| -> Vec<usize> {
            recs.iter().enumerate()
                .filter(|(_, r)| {
                    r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 && {
                        let ctrl_id = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                        ctrl_id == tags::CTRL_TABLE
                    }
                })
                .map(|(i, _)| i)
                .collect()
        };

        let orig_tables = find_tables(&orig_recs);
        let saved_tables = find_tables(&saved_recs);

        eprintln!("\n  원본 표 개수: {}", orig_tables.len());
        eprintln!("  저장 표 개수: {}", saved_tables.len());
        eprintln!("  차이: {:+}", saved_tables.len() as i64 - orig_tables.len() as i64);

        // 붙여넣기된 표 = 저장에만 있는 표 (인덱스가 원본 표 개수 이상인 것)
        let pasted_table_indices: Vec<usize> = if saved_tables.len() > orig_tables.len() {
            saved_tables[orig_tables.len()..].to_vec()
        } else {
            vec![]
        };
        eprintln!("  붙여넣기된 표 시작 인덱스: {:?}", pasted_table_indices);

        // ============================================================
        // 4. 표 구조 분석 함수
        // ============================================================
        let analyze_table_deep = |recs: &[Record], tbl_start: usize, label: &str, cs_count: usize| {
            let tbl_level = recs[tbl_start].level;
            let mut tbl_end = tbl_start + 1;
            while tbl_end < recs.len() && recs[tbl_end].level > tbl_level {
                tbl_end += 1;
            }

            eprintln!("\n  --- {} (rec[{}..{}], {} records, level={}) ---", label, tbl_start, tbl_end, tbl_end - tbl_start, tbl_level);

            // CTRL_HEADER dump
            let ctrl_hdr = &recs[tbl_start];
            let ctrl_id = u32::from_le_bytes([ctrl_hdr.data[0], ctrl_hdr.data[1], ctrl_hdr.data[2], ctrl_hdr.data[3]]);
            let ctrl_bytes = [ctrl_hdr.data[3], ctrl_hdr.data[2], ctrl_hdr.data[1], ctrl_hdr.data[0]];
            let ctrl_str: String = ctrl_bytes.iter().map(|&b| if b >= 0x20 && b <= 0x7e { b as char } else { '.' }).collect();
            let dump_len = ctrl_hdr.data.len().min(64);
            eprintln!("    CTRL_HEADER ({}B): ctrl_id=0x{:08X} \"{}\"", ctrl_hdr.data.len(), ctrl_id, ctrl_str);
            eprintln!("      data[..{}]: {:02X?}", dump_len, &ctrl_hdr.data[..dump_len]);

            // TABLE record
            let mut table_rec_idx = None;
            let mut list_headers: Vec<usize> = Vec::new();

            for ri in tbl_start+1..tbl_end {
                if recs[ri].tag_id == tags::HWPTAG_TABLE && recs[ri].level == tbl_level + 1 {
                    table_rec_idx = Some(ri);
                }
                if recs[ri].tag_id == tags::HWPTAG_LIST_HEADER && recs[ri].level == tbl_level + 1 {
                    list_headers.push(ri);
                }
            }

            if let Some(tri) = table_rec_idx {
                let td = &recs[tri].data;
                eprintln!("    TABLE record (rec[{}], {}B):", tri, td.len());
                let dump_len2 = td.len().min(80);
                eprintln!("      data[..{}]: {:02X?}", dump_len2, &td[..dump_len2]);
                if td.len() >= 8 {
                    let flags = u32::from_le_bytes([td[0], td[1], td[2], td[3]]);
                    let nrows = u16::from_le_bytes([td[4], td[5]]);
                    let ncols = u16::from_le_bytes([td[6], td[7]]);
                    eprintln!("      flags=0x{:08X} rows={} cols={} (expected_cells={})", flags, nrows, ncols, nrows as u32 * ncols as u32);

                    // Cell spacing, padding
                    if td.len() >= 10 {
                        let cell_spacing = u16::from_le_bytes([td[8], td[9]]);
                        eprintln!("      cell_spacing={}", cell_spacing);
                    }
                    // padding: left, right, top, bottom (u16 each) at offset 10..18
                    if td.len() >= 18 {
                        let pad_l = u16::from_le_bytes([td[10], td[11]]);
                        let pad_r = u16::from_le_bytes([td[12], td[13]]);
                        let pad_t = u16::from_le_bytes([td[14], td[15]]);
                        let pad_b = u16::from_le_bytes([td[16], td[17]]);
                        eprintln!("      padding: L={} R={} T={} B={}", pad_l, pad_r, pad_t, pad_b);
                    }
                    // Row sizes
                    if td.len() >= 18 + nrows as usize * 2 {
                        let mut row_sizes = Vec::new();
                        for r in 0..nrows as usize {
                            let off = 18 + r * 2;
                            let rs = u16::from_le_bytes([td[off], td[off+1]]);
                            row_sizes.push(rs);
                        }
                        eprintln!("      row_sizes: {:?}", row_sizes);
                    }
                    // border_fill_id
                    let bf_off = 18 + nrows as usize * 2;
                    if td.len() >= bf_off + 2 {
                        let bf_id = u16::from_le_bytes([td[bf_off], td[bf_off+1]]);
                        eprintln!("      border_fill_id={}", bf_id);
                    }
                }
            } else {
                eprintln!("    TABLE record: NOT FOUND!");
            }

            // LIST_HEADER (cells) and their paragraphs
            eprintln!("    셀 개수 (LIST_HEADER): {}", list_headers.len());

            let mut cell_issues: Vec<String> = Vec::new();

            for (ci, &lhi) in list_headers.iter().enumerate() {
                let lh = &recs[lhi];
                let cell_level = lh.level;
                let dump_len3 = lh.data.len().min(48);
                eprintln!("\n    Cell[{}] LIST_HEADER (rec[{}], {}B, level={}):", ci, lhi, lh.data.len(), cell_level);
                eprintln!("      data[..{}]: {:02X?}", dump_len3, &lh.data[..dump_len3]);

                if lh.data.len() >= 4 {
                    let nparas = u16::from_le_bytes([lh.data[0], lh.data[1]]);
                    let flags = u16::from_le_bytes([lh.data[2], lh.data[3]]);
                    eprintln!("      nparas={} flags=0x{:04X}", nparas, flags);
                }
                // Cell-specific data: col, row, col_span, row_span, width, height at offsets in LIST_HEADER
                // After the generic LIST_HEADER (first ~14 bytes): col(u16) row(u16) col_span(u16) row_span(u16) width(u32) height(u32) padding(u16x4) border_fill_id(u16)
                if lh.data.len() >= 34 {
                    let col_addr = u16::from_le_bytes([lh.data[14], lh.data[15]]);
                    let row_addr = u16::from_le_bytes([lh.data[16], lh.data[17]]);
                    let col_span = u16::from_le_bytes([lh.data[18], lh.data[19]]);
                    let row_span = u16::from_le_bytes([lh.data[20], lh.data[21]]);
                    let width = u32::from_le_bytes([lh.data[22], lh.data[23], lh.data[24], lh.data[25]]);
                    let height = u32::from_le_bytes([lh.data[26], lh.data[27], lh.data[28], lh.data[29]]);
                    eprintln!("      cell: col={} row={} col_span={} row_span={} width={} height={}", col_addr, row_addr, col_span, row_span, width, height);

                    let bf_id = u16::from_le_bytes([lh.data[32], lh.data[33]]);
                    eprintln!("      border_fill_id={}", bf_id);
                }

                // Find paragraphs belonging to this cell
                let next_boundary = if ci + 1 < list_headers.len() {
                    list_headers[ci + 1]
                } else {
                    tbl_end
                };

                let mut para_count = 0;
                for ri2 in lhi+1..next_boundary {
                    if recs[ri2].tag_id == tags::HWPTAG_PARA_HEADER && recs[ri2].level == cell_level + 1 {
                        let ph = &recs[ri2];
                        let raw_char_count = if ph.data.len() >= 4 {
                            u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]])
                        } else { 0 };
                        let char_count = raw_char_count & 0x7FFFFFFF;
                        let msb = raw_char_count >> 31;
                        let control_mask = if ph.data.len() >= 8 {
                            u32::from_le_bytes([ph.data[4], ph.data[5], ph.data[6], ph.data[7]])
                        } else { 0 };
                        let para_shape_id = if ph.data.len() >= 10 {
                            u16::from_le_bytes([ph.data[8], ph.data[9]])
                        } else { 0 };
                        let style_id = if ph.data.len() >= 11 { ph.data[10] } else { 0 };
                        let num_char_shapes = if ph.data.len() >= 14 {
                            u16::from_le_bytes([ph.data[12], ph.data[13]])
                        } else { 0 };

                        eprintln!("      PARA[{}] (rec[{}]): char_count={} (msb={}) control_mask=0x{:08X} para_shape_id={} style_id={} numCharShapes={}",
                            para_count, ri2, char_count, msb, control_mask, para_shape_id, style_id, num_char_shapes);

                        // para_shape_id validity
                        if (para_shape_id as usize) >= saved_ps {
                            let msg = format!("Cell[{}] PARA[{}] rec[{}]: para_shape_id={} >= para_shapes.len()={}", ci, para_count, ri2, para_shape_id, saved_ps);
                            eprintln!("        *** INVALID para_shape_id: {} ***", msg);
                            cell_issues.push(msg);
                        }

                        // PARA_TEXT check
                        let has_text = ri2 + 1 < next_boundary
                            && recs[ri2 + 1].tag_id == tags::HWPTAG_PARA_TEXT
                            && recs[ri2 + 1].level == cell_level + 2;

                        if has_text {
                            let pt = &recs[ri2 + 1];
                            let pt_u16_count = pt.data.len() / 2;
                            let expected_u16 = char_count as usize;
                            let u16_chars: Vec<u16> = pt.data.chunks(2)
                                .filter(|c| c.len() == 2)
                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                .collect();
                            let text = String::from_utf16_lossy(&u16_chars);
                            let preview: String = text.chars().take(60).collect();

                            if pt_u16_count != expected_u16 {
                                let msg = format!("Cell[{}] PARA[{}] rec[{}]: char_count={} but PARA_TEXT has {} u16 units (diff={})",
                                    ci, para_count, ri2, char_count, pt_u16_count, pt_u16_count as i64 - expected_u16 as i64);
                                eprintln!("        *** MISMATCH: {} ***", msg);
                                cell_issues.push(msg);
                            }
                            eprintln!("        PARA_TEXT ({}B, {} u16): \"{}\"", pt.data.len(), pt_u16_count, preview);
                        } else if char_count > 0 {
                            // char_count > 0 but no PARA_TEXT (might have char_count=1 for empty para end marker only in HEADER)
                            if char_count > 1 {
                                let msg = format!("Cell[{}] PARA[{}] rec[{}]: char_count={} but NO PARA_TEXT", ci, para_count, ri2, char_count);
                                eprintln!("        *** MISSING PARA_TEXT: {} ***", msg);
                                cell_issues.push(msg);
                            }
                        }

                        // PARA_CHAR_SHAPE check
                        // Look for PARA_CHAR_SHAPE following PARA_TEXT (or PARA_HEADER if no text)
                        let mut pcs_idx = None;
                        for ri3 in ri2+1..next_boundary {
                            if recs[ri3].level <= cell_level + 1 { break; } // left this para's children
                            if recs[ri3].tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && recs[ri3].level == cell_level + 2 {
                                pcs_idx = Some(ri3);
                                break;
                            }
                        }

                        if let Some(pcs_ri) = pcs_idx {
                            let pcs = &recs[pcs_ri];
                            let num_entries = pcs.data.len() / 8;
                            eprintln!("        PARA_CHAR_SHAPE (rec[{}], {}B, {} entries):", pcs_ri, pcs.data.len(), num_entries);

                            for ei in 0..num_entries {
                                let off = ei * 8;
                                if off + 8 <= pcs.data.len() {
                                    let start_pos = u32::from_le_bytes([pcs.data[off], pcs.data[off+1], pcs.data[off+2], pcs.data[off+3]]);
                                    let cs_id = u32::from_le_bytes([pcs.data[off+4], pcs.data[off+5], pcs.data[off+6], pcs.data[off+7]]);
                                    let valid = (cs_id as usize) < cs_count;
                                    eprintln!("          [{}] start_pos={} char_shape_id={} {}",
                                        ei, start_pos, cs_id, if valid { "OK" } else { "*** INVALID ***" });
                                    if !valid {
                                        cell_issues.push(format!("Cell[{}] PARA[{}] PARA_CHAR_SHAPE entry[{}]: char_shape_id={} >= {} (invalid)",
                                            ci, para_count, ei, cs_id, cs_count));
                                    }
                                }
                            }
                        } else if num_char_shapes > 0 {
                            eprintln!("        PARA_CHAR_SHAPE: NOT FOUND (numCharShapes={})", num_char_shapes);
                        }

                        para_count += 1;
                    }
                }
            }

            cell_issues
        };

        // ============================================================
        // 5. 모든 표 분석
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 3. 표(Table) 상세 분석 ===");
        eprintln!("{}", "=".repeat(120));

        for (ti, &tbl_start) in orig_tables.iter().enumerate() {
            analyze_table_deep(&orig_recs, tbl_start, &format!("원본 표[{}]", ti), orig_cs);
        }

        let mut all_pasted_issues: Vec<String> = Vec::new();
        for (ti, &tbl_start) in saved_tables.iter().enumerate() {
            let is_pasted = pasted_table_indices.contains(&tbl_start);
            let label = if is_pasted {
                format!("저장 표[{}] (*** PASTED ***)", ti)
            } else {
                format!("저장 표[{}]", ti)
            };
            let issues = analyze_table_deep(&saved_recs, tbl_start, &label, saved_cs);
            if is_pasted {
                all_pasted_issues.extend(issues);
            }
        }

        // ============================================================
        // 6. 전체 저장 파일 문단 일관성 검사
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 4. 전체 문단 일관성 검사 (저장 파일) ===");
        eprintln!("{}", "=".repeat(120));

        let mut total_paras = 0u32;
        let mut char_count_mismatches = 0u32;
        let mut missing_para_text = 0u32;
        let mut invalid_cs_refs = 0u32;
        let mut invalid_ps_refs = 0u32;
        let mut max_cs_id: u32 = 0;
        let mut max_ps_id: u16 = 0;

        let mut i = 0;
        while i < saved_recs.len() {
            if saved_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph = &saved_recs[i];
                let ph_level = ph.level;
                let raw_char_count = if ph.data.len() >= 4 {
                    u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]])
                } else { 0 };
                let char_count = raw_char_count & 0x7FFFFFFF;
                let control_mask = if ph.data.len() >= 8 {
                    u32::from_le_bytes([ph.data[4], ph.data[5], ph.data[6], ph.data[7]])
                } else { 0 };
                let para_shape_id = if ph.data.len() >= 10 {
                    u16::from_le_bytes([ph.data[8], ph.data[9]])
                } else { 0 };
                let style_id = if ph.data.len() >= 11 { ph.data[10] } else { 0 };

                // para_shape_id validity
                if (para_shape_id as usize) >= saved_ps {
                    eprintln!("  *** para[{}] rec[{}]: para_shape_id={} >= {} (INVALID) ***",
                        total_paras, i, para_shape_id, saved_ps);
                    invalid_ps_refs += 1;
                }
                if para_shape_id > max_ps_id {
                    max_ps_id = para_shape_id;
                }

                // PARA_TEXT check
                let has_text = i + 1 < saved_recs.len()
                    && saved_recs[i + 1].tag_id == tags::HWPTAG_PARA_TEXT
                    && saved_recs[i + 1].level == ph_level + 1;

                if has_text {
                    let pt = &saved_recs[i + 1];
                    let pt_u16_count = pt.data.len() / 2;
                    let expected_u16 = char_count as usize;

                    if pt_u16_count != expected_u16 {
                        eprintln!("  MISMATCH para[{}] rec[{}]: char_count={} but PARA_TEXT has {} u16 (diff={})",
                            total_paras, i, char_count, pt_u16_count, pt_u16_count as i64 - expected_u16 as i64);
                        let u16_chars: Vec<u16> = pt.data.chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        let text = String::from_utf16_lossy(&u16_chars);
                        let preview: String = text.chars().take(60).collect();
                        eprintln!("    text_preview: \"{}\"", preview);
                        char_count_mismatches += 1;
                    }
                } else if char_count > 1 {
                    eprintln!("  MISSING PARA_TEXT para[{}] rec[{}]: char_count={} control_mask=0x{:08X}",
                        total_paras, i, char_count, control_mask);
                    missing_para_text += 1;
                }

                // PARA_CHAR_SHAPE check for all paragraphs
                for ri3 in i+1..saved_recs.len() {
                    if saved_recs[ri3].level <= ph_level { break; }
                    if saved_recs[ri3].tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && saved_recs[ri3].level == ph_level + 1 {
                        let pcs = &saved_recs[ri3];
                        let num_entries = pcs.data.len() / 8;
                        for ei in 0..num_entries {
                            let off = ei * 8;
                            if off + 8 <= pcs.data.len() {
                                let cs_id = u32::from_le_bytes([pcs.data[off+4], pcs.data[off+5], pcs.data[off+6], pcs.data[off+7]]);
                                if cs_id > max_cs_id {
                                    max_cs_id = cs_id;
                                }
                                if (cs_id as usize) >= saved_cs {
                                    eprintln!("  *** para[{}] rec[{}] PARA_CHAR_SHAPE entry[{}]: char_shape_id={} >= {} (INVALID) ***",
                                        total_paras, i, ei, cs_id, saved_cs);
                                    invalid_cs_refs += 1;
                                }
                            }
                        }
                        break;
                    }
                }

                total_paras += 1;
            }
            i += 1;
        }

        // ============================================================
        // 7. 원본 파일도 동일 검사 (비교용)
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 5. 전체 문단 일관성 검사 (원본 파일) ===");
        eprintln!("{}", "=".repeat(120));

        let mut orig_total_paras = 0u32;
        let mut orig_char_count_mismatches = 0u32;
        let mut orig_invalid_cs_refs = 0u32;
        let mut orig_invalid_ps_refs = 0u32;

        i = 0;
        while i < orig_recs.len() {
            if orig_recs[i].tag_id == tags::HWPTAG_PARA_HEADER {
                let ph = &orig_recs[i];
                let ph_level = ph.level;
                let raw_char_count = if ph.data.len() >= 4 {
                    u32::from_le_bytes([ph.data[0], ph.data[1], ph.data[2], ph.data[3]])
                } else { 0 };
                let char_count = raw_char_count & 0x7FFFFFFF;
                let para_shape_id = if ph.data.len() >= 10 {
                    u16::from_le_bytes([ph.data[8], ph.data[9]])
                } else { 0 };

                if (para_shape_id as usize) >= orig_ps {
                    eprintln!("  *** orig para[{}] rec[{}]: para_shape_id={} >= {} (INVALID) ***",
                        orig_total_paras, i, para_shape_id, orig_ps);
                    orig_invalid_ps_refs += 1;
                }

                let has_text = i + 1 < orig_recs.len()
                    && orig_recs[i + 1].tag_id == tags::HWPTAG_PARA_TEXT
                    && orig_recs[i + 1].level == ph_level + 1;

                if has_text {
                    let pt = &orig_recs[i + 1];
                    let pt_u16_count = pt.data.len() / 2;
                    if pt_u16_count != char_count as usize {
                        eprintln!("  MISMATCH orig para[{}] rec[{}]: char_count={} but PARA_TEXT has {} u16",
                            orig_total_paras, i, char_count, pt_u16_count);
                        orig_char_count_mismatches += 1;
                    }
                }

                // PARA_CHAR_SHAPE
                for ri3 in i+1..orig_recs.len() {
                    if orig_recs[ri3].level <= ph_level { break; }
                    if orig_recs[ri3].tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && orig_recs[ri3].level == ph_level + 1 {
                        let pcs = &orig_recs[ri3];
                        let num_entries = pcs.data.len() / 8;
                        for ei in 0..num_entries {
                            let off = ei * 8;
                            if off + 8 <= pcs.data.len() {
                                let cs_id = u32::from_le_bytes([pcs.data[off+4], pcs.data[off+5], pcs.data[off+6], pcs.data[off+7]]);
                                if (cs_id as usize) >= orig_cs {
                                    eprintln!("  *** orig para[{}] rec[{}] PARA_CHAR_SHAPE entry[{}]: char_shape_id={} >= {} (INVALID) ***",
                                        orig_total_paras, i, ei, cs_id, orig_cs);
                                    orig_invalid_cs_refs += 1;
                                }
                            }
                        }
                        break;
                    }
                }

                orig_total_paras += 1;
            }
            i += 1;
        }

        // ============================================================
        // 8. 레코드 타입별 카운트 비교
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== 6. 레코드 타입별 카운트 비교 ===");
        eprintln!("{}", "=".repeat(120));
        let count_tag = |recs: &[Record], tag: u16| recs.iter().filter(|r| r.tag_id == tag).count();
        let tags_to_check: [(u16, &str); 11] = [
            (tags::HWPTAG_PARA_HEADER, "PARA_HEADER"),
            (tags::HWPTAG_PARA_TEXT, "PARA_TEXT"),
            (tags::HWPTAG_PARA_CHAR_SHAPE, "PARA_CHAR_SHAPE"),
            (tags::HWPTAG_PARA_LINE_SEG, "PARA_LINE_SEG"),
            (tags::HWPTAG_PARA_RANGE_TAG, "PARA_RANGE_TAG"),
            (tags::HWPTAG_CTRL_HEADER, "CTRL_HEADER"),
            (tags::HWPTAG_LIST_HEADER, "LIST_HEADER"),
            (tags::HWPTAG_TABLE, "TABLE"),
            (tags::HWPTAG_CTRL_DATA, "CTRL_DATA"),
            (tags::HWPTAG_PAGE_DEF, "PAGE_DEF"),
            (tags::HWPTAG_SHAPE_COMPONENT, "SHAPE_COMPONENT"),
        ];
        for (tag, name) in &tags_to_check {
            let orig_cnt = count_tag(&orig_recs, *tag);
            let saved_cnt = count_tag(&saved_recs, *tag);
            let diff = saved_cnt as i64 - orig_cnt as i64;
            eprintln!("  {:25} orig={:4}  saved={:4}  diff={}{:+}{}",
                name, orig_cnt, saved_cnt,
                if diff != 0 { "<<< " } else { "" },
                diff,
                if diff != 0 { " >>>" } else { "" });
        }

        // ============================================================
        // 9. 요약
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("=== SUMMARY ===");
        eprintln!("{}", "=".repeat(120));
        eprintln!("  원본: tables={}, paragraphs={}, char_count_mismatches={}, invalid_cs_refs={}, invalid_ps_refs={}",
            orig_tables.len(), orig_total_paras, orig_char_count_mismatches, orig_invalid_cs_refs, orig_invalid_ps_refs);
        eprintln!("  저장: tables={}, paragraphs={}, char_count_mismatches={}, invalid_cs_refs={}, invalid_ps_refs={}, missing_para_text={}",
            saved_tables.len(), total_paras, char_count_mismatches, invalid_cs_refs, invalid_ps_refs, missing_para_text);
        eprintln!("  붙여넣기 표 issues: {}", all_pasted_issues.len());
        for (idx, issue) in all_pasted_issues.iter().enumerate() {
            eprintln!("    [{}] {}", idx, issue);
        }
        eprintln!("  max CharShape ID referenced: {} (available: 0..{})", max_cs_id, saved_cs);
        eprintln!("  max ParaShape ID referenced: {} (available: 0..{})", max_ps_id, saved_ps);

        // Assertions
        // We do NOT assert zero mismatches here because the purpose is analysis/reporting.
        // But we flag truly fatal issues.
        if invalid_cs_refs > 0 {
            eprintln!("\n  *** FATAL: {} invalid CharShape ID references in saved file ***", invalid_cs_refs);
        }
        if invalid_ps_refs > 0 {
            eprintln!("\n  *** FATAL: {} invalid ParaShape ID references in saved file ***", invalid_ps_refs);
        }
        if char_count_mismatches > 0 {
            eprintln!("\n  *** WARNING: {} char_count vs PARA_TEXT mismatches in saved file ***", char_count_mismatches);
        }

        eprintln!("\n=== test_rp005_pasted_table_analysis complete ===");
    }

    /// 3개 HWP 파일 비교: empty-step2 원본, HWP 프로그램 붙여넣기, 뷰어 붙여넣기(손상)
    ///
    /// DocInfo ID_MAPPINGS, BodyText 레코드 전체 덤프, 레코드별 차이 비교
    #[test]
    fn test_step2_comparison() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;

        // ============================================================
        // 파일 로드
        // ============================================================
        let files: Vec<(&str, &str)> = vec![
            ("template/empty-step2.hwp", "ORIGINAL"),
            ("template/empty-step2-p.hwp", "HWP_PASTE (VALID)"),
            ("template/empty-step2_saved_err.hwp", "VIEWER_PASTE (DAMAGED)"),
        ];

        struct FileData {
            label: String,
            path: String,
            doc_info_records: Vec<Record>,
            body_records: Vec<Record>,
            body_raw_len: usize,
        }

        let mut all_files: Vec<FileData> = Vec::new();

        for (path, label) in &files {
            let bytes = std::fs::read(path)
                .unwrap_or_else(|e| panic!("파일 읽기 실패: {} - {}", path, e));
            eprintln!("\n=== Loading {} ({}) - {} bytes ===", label, path, bytes.len());

            let mut cfb = CfbReader::open(&bytes)
                .unwrap_or_else(|e| panic!("CFB 열기 실패: {} - {}", path, e));

            // DocInfo (compressed=true)
            let doc_info_data = cfb.read_doc_info(true)
                .unwrap_or_else(|e| panic!("DocInfo 읽기 실패: {} - {}", path, e));
            let doc_info_records = Record::read_all(&doc_info_data)
                .unwrap_or_else(|e| panic!("DocInfo 레코드 파싱 실패: {} - {}", path, e));

            // BodyText Section 0 (compressed=true, distribution=false)
            let body_data = cfb.read_body_text_section(0, true, false)
                .unwrap_or_else(|e| panic!("BodyText 읽기 실패: {} - {}", path, e));
            let body_raw_len = body_data.len();
            let body_records = Record::read_all(&body_data)
                .unwrap_or_else(|e| panic!("BodyText 레코드 파싱 실패: {} - {}", path, e));

            all_files.push(FileData {
                label: label.to_string(),
                path: path.to_string(),
                doc_info_records,
                body_records,
                body_raw_len,
            });
        }

        // ============================================================
        // Helper functions
        // ============================================================

        fn read_u32_le(data: &[u8], offset: usize) -> u32 {
            if offset + 4 <= data.len() {
                u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
            } else {
                0
            }
        }

        fn read_u16_le(data: &[u8], offset: usize) -> u16 {
            if offset + 2 <= data.len() {
                u16::from_le_bytes([data[offset], data[offset+1]])
            } else {
                0
            }
        }

        fn ctrl_id_string(data: &[u8]) -> String {
            if data.len() >= 4 {
                // ctrl_id stored as LE u32, but represents big-endian character ordering
                let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let be_bytes = id.to_be_bytes();
                let ascii: String = be_bytes.iter().map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
                }).collect();
                format!("\"{}\" (0x{:08X})", ascii, id)
            } else {
                format!("(data too short: {} bytes)", data.len())
            }
        }

        fn hex_preview(data: &[u8], max: usize) -> String {
            let show = std::cmp::min(data.len(), max);
            let hex: Vec<String> = data[..show].iter().map(|b| format!("{:02x}", b)).collect();
            let mut s = hex.join(" ");
            if data.len() > max {
                s.push_str(&format!(" ...({} more)", data.len() - max));
            }
            s
        }

        // ============================================================
        // 1. DocInfo ID_MAPPINGS summary for each file
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  PART 1: DocInfo ID_MAPPINGS Summary");
        eprintln!("{}", "=".repeat(120));

        let id_map_field_names = [
            "bin_data_count",       // 0
            "font_han",             // 1
            "font_eng",             // 2
            "font_hanja",           // 3
            "font_jpn",             // 4
            "font_other",           // 5
            "font_symbol",          // 6
            "font_user",            // 7
            "border_fill_count",    // 8
            "char_shape_count",     // 9
            "tab_def_count",        // 10
            "numbering_count",      // 11
            "bullet_count",         // 12
            "para_shape_count",     // 13
            "style_count",          // 14
            "memo_shape_count",     // 15
        ];

        for fd in &all_files {
            eprintln!("\n--- {} ({}) ---", fd.label, fd.path);
            eprintln!("  DocInfo records: {}", fd.doc_info_records.len());

            if let Some(id_rec) = fd.doc_info_records.iter().find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS) {
                eprintln!("  ID_MAPPINGS record size: {} bytes", id_rec.data.len());
                let num_values = id_rec.data.len() / 4;
                for i in 0..std::cmp::min(num_values, id_map_field_names.len()) {
                    let val = read_u32_le(&id_rec.data, i * 4);
                    eprintln!("    [{:>2}] {:<25} = {}", i, id_map_field_names[i], val);
                }

                // Highlight key counts
                let cs = if num_values > 9 { read_u32_le(&id_rec.data, 9 * 4) } else { 0 };
                let ps = if num_values > 13 { read_u32_le(&id_rec.data, 13 * 4) } else { 0 };
                let bf = if num_values > 8 { read_u32_le(&id_rec.data, 8 * 4) } else { 0 };
                eprintln!("  >>> CharShape(CS)={}, ParaShape(PS)={}, BorderFill(BF)={}", cs, ps, bf);
            } else {
                eprintln!("  WARNING: No ID_MAPPINGS record found!");
            }
        }

        // ============================================================
        // 2. BodyText record full dump for each file
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  PART 2: BodyText Section0 - Full Record Dump (ALL files)");
        eprintln!("{}", "=".repeat(120));

        for fd in &all_files {
            eprintln!("\n{}", "#".repeat(100));
            eprintln!("### {} ({}) ###", fd.label, fd.path);
            eprintln!("### BodyText decompressed: {} bytes, records: {} ###",
                fd.body_raw_len, fd.body_records.len());
            eprintln!("{}", "#".repeat(100));

            // Total bytes in records
            let total_data_bytes: usize = fd.body_records.iter().map(|r| r.data.len()).sum();
            eprintln!("  Total record data bytes: {}", total_data_bytes);

            eprintln!("\n{:<5} {:<5} {:<25} {:>8}  {}", "Idx", "Lvl", "Tag", "Size", "Details");
            eprintln!("{:-<120}", "");

            for (i, rec) in fd.body_records.iter().enumerate() {
                let indent = "  ".repeat(std::cmp::min(rec.level as usize, 8));
                let tag_str = format!("{}{}", indent, tags::tag_name(rec.tag_id));

                let mut details = String::new();

                // PARA_HEADER details
                if rec.tag_id == tags::HWPTAG_PARA_HEADER {
                    if rec.data.len() >= 22 {
                        // PARA_HEADER layout:
                        // u32 nChars (bit31=MSB)
                        // u32 controlMask
                        // u16 paraShapeId
                        // u8  styleId (or u16)
                        // u8  breakType
                        // u16 charShapeCount (?)
                        // ...
                        let raw_char_count = read_u32_le(&rec.data, 0);
                        let msb = (raw_char_count >> 31) & 1;
                        let char_count = raw_char_count & 0x7FFFFFFF;
                        let control_mask = read_u32_le(&rec.data, 4);
                        let para_shape_id = read_u16_le(&rec.data, 8);
                        let style_id = rec.data[10];
                        details = format!("char_count={} msb={} control_mask=0x{:08X} para_shape_id={} style_id={}",
                            char_count, msb, control_mask, para_shape_id, style_id);
                    } else {
                        details = format!("(short data: {} bytes) hex={}", rec.data.len(), hex_preview(&rec.data, 32));
                    }
                }
                // CTRL_HEADER details
                else if rec.tag_id == tags::HWPTAG_CTRL_HEADER {
                    details = format!("ctrl={} hex={}", ctrl_id_string(&rec.data), hex_preview(&rec.data, 40));
                }
                // TABLE details
                else if rec.tag_id == tags::HWPTAG_TABLE {
                    if rec.data.len() >= 8 {
                        let attr = read_u32_le(&rec.data, 0);
                        let rows = read_u16_le(&rec.data, 4);
                        let cols = read_u16_le(&rec.data, 6);
                        details = format!("attr=0x{:08X} rows={} cols={} hex={}",
                            attr, rows, cols, hex_preview(&rec.data, 40));
                    } else {
                        details = format!("hex={}", hex_preview(&rec.data, 40));
                    }
                }
                // LIST_HEADER details
                else if rec.tag_id == tags::HWPTAG_LIST_HEADER {
                    if rec.data.len() >= 6 {
                        let para_count = read_u16_le(&rec.data, 0);
                        let attr = read_u32_le(&rec.data, 2);
                        details = format!("paraCount={} attr=0x{:08X} hex={}",
                            para_count, attr, hex_preview(&rec.data, 40));
                    } else {
                        details = format!("hex={}", hex_preview(&rec.data, 40));
                    }
                }
                // PARA_TEXT: show char codes
                else if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                    // UTF-16LE text
                    let mut chars_preview = String::new();
                    let max_chars = 30;
                    let mut n = 0;
                    let mut pos = 0;
                    while pos + 1 < rec.data.len() && n < max_chars {
                        let code = u16::from_le_bytes([rec.data[pos], rec.data[pos+1]]);
                        if code == 0x000D { chars_preview.push_str("\\r"); }
                        else if code == 0x000A { chars_preview.push_str("\\n"); }
                        else if code == 0x000B { chars_preview.push_str("{CTRL}"); }
                        else if code == 0x0002 { chars_preview.push_str("{SECD}"); }
                        else if code == 0x0003 { chars_preview.push_str("{FLD_BEGIN}"); }
                        else if code == 0x0004 { chars_preview.push_str("{FLD_END}"); }
                        else if code == 0x0008 { chars_preview.push_str("{INLINE}"); }
                        else if code < 0x0020 {
                            chars_preview.push_str(&format!("{{0x{:04X}}}", code));
                        }
                        else if let Some(ch) = char::from_u32(code as u32) {
                            chars_preview.push(ch);
                        } else {
                            chars_preview.push_str(&format!("{{0x{:04X}}}", code));
                        }
                        pos += 2;
                        // Extended control chars take 16 bytes total (skip the inline data)
                        if code == 0x000B || code == 0x0002 || code == 0x0003 || code == 0x0008 {
                            pos += 14; // skip 14 more bytes (total 16 for extended char)
                        }
                        n += 1;
                    }
                    details = format!("text=\"{}\" hex={}", chars_preview, hex_preview(&rec.data, 32));
                }
                // PARA_CHAR_SHAPE
                else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                    // pairs of (u32 pos, u32 charShapeId)
                    let n_pairs = rec.data.len() / 8;
                    let mut pairs_str = String::new();
                    for p in 0..std::cmp::min(n_pairs, 8) {
                        let pos_val = read_u32_le(&rec.data, p * 8);
                        let cs_id = read_u32_le(&rec.data, p * 8 + 4);
                        if !pairs_str.is_empty() { pairs_str.push_str(", "); }
                        pairs_str.push_str(&format!("pos{}=>CS{}", pos_val, cs_id));
                    }
                    if n_pairs > 8 { pairs_str.push_str(&format!(" ...({} more)", n_pairs - 8)); }
                    details = format!("[{}]", pairs_str);
                }
                // SHAPE_COMPONENT
                else if rec.tag_id == tags::HWPTAG_SHAPE_COMPONENT {
                    details = format!("hex={}", hex_preview(&rec.data, 40));
                }
                // SHAPE_COMPONENT_PICTURE
                else if rec.tag_id == tags::HWPTAG_SHAPE_COMPONENT_PICTURE {
                    details = format!("hex={}", hex_preview(&rec.data, 40));
                }
                // PAGE_DEF
                else if rec.tag_id == tags::HWPTAG_PAGE_DEF {
                    if rec.data.len() >= 40 {
                        let w = read_u32_le(&rec.data, 0);
                        let h = read_u32_le(&rec.data, 4);
                        details = format!("width={} height={} (hwpunit)", w, h);
                    }
                }
                // FOOTNOTE_SHAPE
                else if rec.tag_id == tags::HWPTAG_FOOTNOTE_SHAPE {
                    details = format!("hex={}", hex_preview(&rec.data, 32));
                }
                // PAGE_BORDER_FILL
                else if rec.tag_id == tags::HWPTAG_PAGE_BORDER_FILL {
                    details = format!("hex={}", hex_preview(&rec.data, 32));
                }
                // Default: show hex for smaller records
                else if rec.data.len() <= 64 {
                    details = format!("hex={}", hex_preview(&rec.data, 48));
                } else {
                    details = format!("hex={}", hex_preview(&rec.data, 32));
                }

                eprintln!("{:<5} {:<5} {:<25} {:>8}  {}",
                    i, rec.level, tag_str, rec.size, details);
            }
        }

        // ============================================================
        // 3. Side-by-side comparison: HWP_PASTE (VALID) vs VIEWER_PASTE (DAMAGED)
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  PART 3: Side-by-Side Comparison - HWP_PASTE vs VIEWER_PASTE");
        eprintln!("{}", "=".repeat(120));

        let valid = &all_files[1]; // HWP_PASTE
        let damaged = &all_files[2]; // VIEWER_PASTE

        eprintln!("\n  VALID (HWP_PASTE):   {} records, {} data bytes",
            valid.body_records.len(), valid.body_raw_len);
        eprintln!("  DAMAGED (VIEWER_PASTE): {} records, {} data bytes",
            damaged.body_records.len(), damaged.body_raw_len);

        let max_recs = std::cmp::max(valid.body_records.len(), damaged.body_records.len());
        let mut total_diffs = 0;

        eprintln!("\n{:<5} | {:<30} {:>4} {:>6} | {:<30} {:>4} {:>6} | {}",
            "#", "VALID Tag", "Lvl", "Size", "DAMAGED Tag", "Lvl", "Size", "Status");
        eprintln!("{:-<130}", "");

        for i in 0..max_recs {
            let have_valid = i < valid.body_records.len();
            let have_damaged = i < damaged.body_records.len();

            let (v_tag_str, v_lvl, v_size) = if have_valid {
                let r = &valid.body_records[i];
                (format!("{}", tags::tag_name(r.tag_id)), r.level, r.size)
            } else {
                ("---".to_string(), 0u16, 0u32)
            };

            let (d_tag_str, d_lvl, d_size) = if have_damaged {
                let r = &damaged.body_records[i];
                (format!("{}", tags::tag_name(r.tag_id)), r.level, r.size)
            } else {
                ("---".to_string(), 0u16, 0u32)
            };

            let status = if !have_valid {
                total_diffs += 1;
                "EXTRA_IN_DAMAGED"
            } else if !have_damaged {
                total_diffs += 1;
                "MISSING_IN_DAMAGED"
            } else {
                let rv = &valid.body_records[i];
                let rd = &damaged.body_records[i];
                if rv.tag_id != rd.tag_id {
                    total_diffs += 1;
                    "TAG_DIFF"
                } else if rv.level != rd.level {
                    total_diffs += 1;
                    "LEVEL_DIFF"
                } else if rv.data != rd.data {
                    total_diffs += 1;
                    "DATA_DIFF"
                } else {
                    "OK"
                }
            };

            // For diffs, always print. For OK, also print (full dump requested).
            let marker = if status != "OK" { ">>>" } else { "   " };
            eprintln!("{} {:<5} | {:<30} {:>4} {:>6} | {:<30} {:>4} {:>6} | {}",
                marker, i, v_tag_str, v_lvl, v_size, d_tag_str, d_lvl, d_size, status);

            // For diffs, show detailed comparison
            if status != "OK" && have_valid && have_damaged {
                let rv = &valid.body_records[i];
                let rd = &damaged.body_records[i];

                // PARA_HEADER diff details
                if rv.tag_id == tags::HWPTAG_PARA_HEADER || rd.tag_id == tags::HWPTAG_PARA_HEADER {
                    if rv.tag_id == tags::HWPTAG_PARA_HEADER && rd.tag_id == tags::HWPTAG_PARA_HEADER {
                        let v_char = read_u32_le(&rv.data, 0);
                        let d_char = read_u32_le(&rd.data, 0);
                        let v_mask = read_u32_le(&rv.data, 4);
                        let d_mask = read_u32_le(&rd.data, 4);
                        let v_ps = read_u16_le(&rv.data, 8);
                        let d_ps = read_u16_le(&rd.data, 8);
                        let v_st = if rv.data.len() > 10 { rv.data[10] } else { 0 };
                        let d_st = if rd.data.len() > 10 { rd.data[10] } else { 0 };
                        eprintln!("          PARA_HEADER diff: char_count {}={} vs {}={}, mask 0x{:08X} vs 0x{:08X}, ps {} vs {}, style {} vs {}",
                            if v_char != d_char { "DIFF" } else { "same" }, v_char & 0x7FFFFFFF,
                            if v_char != d_char { "DIFF" } else { "same" }, d_char & 0x7FFFFFFF,
                            v_mask, d_mask, v_ps, d_ps, v_st, d_st);
                    }
                }

                // CTRL_HEADER diff details
                if rv.tag_id == tags::HWPTAG_CTRL_HEADER && rd.tag_id == tags::HWPTAG_CTRL_HEADER {
                    eprintln!("          VALID ctrl={}  DAMAGED ctrl={}",
                        ctrl_id_string(&rv.data), ctrl_id_string(&rd.data));
                }

                // Show hex of both
                eprintln!("          VALID  hex: {}", hex_preview(&rv.data, 48));
                eprintln!("          DAMAGED hex: {}", hex_preview(&rd.data, 48));

                // Show first byte diff position
                let min_len = std::cmp::min(rv.data.len(), rd.data.len());
                if let Some(pos) = (0..min_len).find(|&j| rv.data[j] != rd.data[j]) {
                    eprintln!("          First byte diff at offset {}: VALID=0x{:02x} DAMAGED=0x{:02x}",
                        pos, rv.data[pos], rd.data[pos]);
                }
                if rv.data.len() != rd.data.len() {
                    eprintln!("          Size diff: VALID={} DAMAGED={}", rv.data.len(), rd.data.len());
                }
            }
        }

        // ============================================================
        // 4. DocInfo comparison: VALID vs DAMAGED
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  PART 4: DocInfo Comparison - HWP_PASTE vs VIEWER_PASTE");
        eprintln!("{}", "=".repeat(120));

        let v_doc = &all_files[1].doc_info_records;
        let d_doc = &all_files[2].doc_info_records;
        eprintln!("\n  VALID DocInfo records: {}", v_doc.len());
        eprintln!("  DAMAGED DocInfo records: {}", d_doc.len());

        let max_doc = std::cmp::max(v_doc.len(), d_doc.len());
        let mut doc_diffs = 0;
        for i in 0..max_doc {
            let have_v = i < v_doc.len();
            let have_d = i < d_doc.len();
            let status = if !have_v { "EXTRA_IN_DAMAGED" }
                else if !have_d { "MISSING_IN_DAMAGED" }
                else if v_doc[i].tag_id != d_doc[i].tag_id { "TAG_DIFF" }
                else if v_doc[i].level != d_doc[i].level { "LEVEL_DIFF" }
                else if v_doc[i].data != d_doc[i].data { "DATA_DIFF" }
                else { "OK" };

            if status != "OK" {
                doc_diffs += 1;
                let v_str = if have_v {
                    format!("{} lvl={} sz={}", tags::tag_name(v_doc[i].tag_id), v_doc[i].level, v_doc[i].size)
                } else { "---".to_string() };
                let d_str = if have_d {
                    format!("{} lvl={} sz={}", tags::tag_name(d_doc[i].tag_id), d_doc[i].level, d_doc[i].size)
                } else { "---".to_string() };
                eprintln!("  [{}] {} | VALID: {} | DAMAGED: {} |",
                    i, status, v_str, d_str);
            }
        }
        if doc_diffs == 0 {
            eprintln!("  DocInfo records are IDENTICAL between VALID and DAMAGED");
        } else {
            eprintln!("  Total DocInfo differences: {}", doc_diffs);
        }

        // ============================================================
        // 5. Summary
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  SUMMARY");
        eprintln!("{}", "=".repeat(120));

        for fd in &all_files {
            let total_data: usize = fd.body_records.iter().map(|r| r.data.len()).sum();
            eprintln!("  {:<25} DocInfo recs={:<5} BodyText recs={:<5} body_bytes={:<8} data_bytes={}",
                fd.label, fd.doc_info_records.len(), fd.body_records.len(), fd.body_raw_len, total_data);
        }

        eprintln!("\n  BodyText record-by-record diffs (VALID vs DAMAGED): {}", total_diffs);
        eprintln!("  DocInfo record-by-record diffs (VALID vs DAMAGED): {}", doc_diffs);

        eprintln!("\n=== test_step2_comparison complete ===");
    }

    #[test]
    fn test_step2_paste_area() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;

        // ============================================================
        // Helper functions
        // ============================================================
        fn read_u32_le(data: &[u8], offset: usize) -> u32 {
            if offset + 4 <= data.len() {
                u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
            } else { 0 }
        }
        fn read_u16_le(data: &[u8], offset: usize) -> u16 {
            if offset + 2 <= data.len() {
                u16::from_le_bytes([data[offset], data[offset+1]])
            } else { 0 }
        }
        fn hex_dump(data: &[u8], max: usize) -> String {
            let show = std::cmp::min(data.len(), max);
            let hex: Vec<String> = data[..show].iter().map(|b| format!("{:02x}", b)).collect();
            let mut s = hex.join(" ");
            if data.len() > max { s.push_str(&format!(" ...({} more)", data.len() - max)); }
            s
        }
        fn hex_full(data: &[u8]) -> String {
            data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
        }
        fn utf16le_decode(data: &[u8]) -> String {
            let mut result = String::new();
            let mut pos = 0;
            while pos + 1 < data.len() {
                let code = u16::from_le_bytes([data[pos], data[pos+1]]);
                if code == 0x000D { result.push_str("\\r"); }
                else if code == 0x000A { result.push_str("\\n"); }
                else if code == 0x000B { result.push_str("{CTRL}"); pos += 14; }
                else if code == 0x0002 { result.push_str("{SECD}"); pos += 14; }
                else if code == 0x0003 { result.push_str("{FLD_BEGIN}"); pos += 14; }
                else if code == 0x0004 { result.push_str("{FLD_END}"); pos += 14; }
                else if code == 0x0008 { result.push_str("{INLINE}"); pos += 14; }
                else if code < 0x0020 { result.push_str(&format!("{{0x{:04X}}}", code)); }
                else if let Some(ch) = char::from_u32(code as u32) { result.push(ch); }
                else { result.push_str(&format!("{{0x{:04X}}}", code)); }
                pos += 2;
            }
            result
        }

        // ============================================================
        // Print detailed record info
        // ============================================================
        fn print_record_detail(label: &str, idx: usize, rec: &Record) {
            let tag_name = tags::tag_name(rec.tag_id);
            eprintln!("  [{:>3}] lvl={} tag={:<20} size={:<6} tag_id={}", idx, rec.level, tag_name, rec.size, rec.tag_id);

            if rec.tag_id == tags::HWPTAG_PARA_HEADER {
                if rec.data.len() >= 22 {
                    let raw_nchars = read_u32_le(&rec.data, 0);
                    let msb = (raw_nchars >> 31) & 1;
                    let char_count = raw_nchars & 0x7FFFFFFF;
                    let control_mask = read_u32_le(&rec.data, 4);
                    let para_shape_id = read_u16_le(&rec.data, 8);
                    let style_id = rec.data[10];
                    let break_type = rec.data[11];
                    let num_char_shapes = read_u16_le(&rec.data, 12);
                    let num_range_tags = read_u16_le(&rec.data, 14);
                    let num_line_segs = read_u16_le(&rec.data, 16);
                    let para_inst_id = read_u32_le(&rec.data, 18);
                    eprintln!("         PARA_HEADER: char_count={} msb={} control_mask=0x{:08X}", char_count, msb, control_mask);
                    eprintln!("           para_shape_id={} style_id={} break_type={}", para_shape_id, style_id, break_type);
                    eprintln!("           numCharShapes={} numRangeTags={} numLineSegs={} paraInstId={}",
                        num_char_shapes, num_range_tags, num_line_segs, para_inst_id);
                    eprintln!("           full hex: {}", hex_full(&rec.data));
                } else {
                    eprintln!("         PARA_HEADER (short {}b): {}", rec.data.len(), hex_full(&rec.data));
                }
            }
            else if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                let show_bytes = std::cmp::min(rec.data.len(), 100);
                eprintln!("         PARA_TEXT hex(first {}): {}", show_bytes, hex_dump(&rec.data, 100));
                eprintln!("         PARA_TEXT decoded: \"{}\"", utf16le_decode(&rec.data));
            }
            else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                let n_pairs = rec.data.len() / 8;
                for p in 0..n_pairs {
                    let pos_val = read_u32_le(&rec.data, p * 8);
                    let cs_id = read_u32_le(&rec.data, p * 8 + 4);
                    eprintln!("         PARA_CHAR_SHAPE[{}]: pos={} => CS_id={}", p, pos_val, cs_id);
                }
                eprintln!("         full hex: {}", hex_full(&rec.data));
            }
            else if rec.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                eprintln!("         PARA_LINE_SEG hex: {}", hex_full(&rec.data));
                // Decode line seg entries (each 36 bytes)
                let entry_size = 36;
                let n_entries = rec.data.len() / entry_size;
                for e in 0..n_entries {
                    let off = e * entry_size;
                    let text_start = read_u32_le(&rec.data, off);
                    let y_pos = read_u32_le(&rec.data, off + 4) as i32;
                    let height = read_u32_le(&rec.data, off + 8);
                    let text_height = read_u32_le(&rec.data, off + 12);
                    let baseline = read_u32_le(&rec.data, off + 16);
                    let spacing = read_u32_le(&rec.data, off + 20);
                    let x_pos = read_u32_le(&rec.data, off + 24) as i32;
                    let seg_width = read_u32_le(&rec.data, off + 28);
                    let tag_flags = read_u32_le(&rec.data, off + 32);
                    eprintln!("           seg[{}]: textStart={} yPos={} h={} textH={} baseline={} spacing={} xPos={} segW={} flags=0x{:08X}",
                        e, text_start, y_pos, height, text_height, baseline, spacing, x_pos, seg_width, tag_flags);
                }
            }
            else if rec.tag_id == tags::HWPTAG_CTRL_HEADER {
                if rec.data.len() >= 4 {
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    let be = ctrl_id.to_be_bytes();
                    let ascii: String = be.iter().map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }).collect();
                    eprintln!("         CTRL_HEADER: ctrl=\"{}\" (0x{:08X})", ascii, ctrl_id);
                }
                eprintln!("         full hex: {}", hex_dump(&rec.data, 80));
            }
            else if rec.tag_id == tags::HWPTAG_TABLE {
                if rec.data.len() >= 8 {
                    let attr = read_u32_le(&rec.data, 0);
                    let rows = read_u16_le(&rec.data, 4);
                    let cols = read_u16_le(&rec.data, 6);
                    eprintln!("         TABLE: attr=0x{:08X} rows={} cols={}", attr, rows, cols);
                    if rec.data.len() >= 10 {
                        let cell_spacing = read_u16_le(&rec.data, 8);
                        eprintln!("           cellSpacing={}", cell_spacing);
                    }
                    // border fill id per row: after cell_spacing(2) + padding(2*rows for row sizes)
                    let row_sizes_start = 10;
                    for r in 0..rows as usize {
                        if row_sizes_start + (r + 1) * 2 <= rec.data.len() {
                            let rs = read_u16_le(&rec.data, row_sizes_start + r * 2);
                            eprintln!("           rowSize[{}]={}", r, rs);
                        }
                    }
                }
                eprintln!("         full hex(40): {}", hex_dump(&rec.data, 40));
                eprintln!("         full hex(all): {}", hex_full(&rec.data));
            }
            else if rec.tag_id == tags::HWPTAG_LIST_HEADER {
                if rec.data.len() >= 6 {
                    let para_count = read_u16_le(&rec.data, 0);
                    let attr = read_u32_le(&rec.data, 2);
                    eprintln!("         LIST_HEADER: paraCount={} attr=0x{:08X}", para_count, attr);
                    if rec.data.len() >= 47 {
                        // Cell-specific fields (for table cells)
                        let col_addr = read_u16_le(&rec.data, 8);
                        let row_addr = read_u16_le(&rec.data, 10);
                        let col_span = read_u16_le(&rec.data, 12);
                        let row_span = read_u16_le(&rec.data, 14);
                        let cell_w = read_u32_le(&rec.data, 16);
                        let cell_h = read_u32_le(&rec.data, 20);
                        let padding_l = read_u16_le(&rec.data, 24);
                        let padding_r = read_u16_le(&rec.data, 26);
                        let padding_t = read_u16_le(&rec.data, 28);
                        let padding_b = read_u16_le(&rec.data, 30);
                        let border_fill_id = read_u16_le(&rec.data, 32);
                        let cell_w2 = read_u32_le(&rec.data, 34);
                        eprintln!("           col={} row={} colSpan={} rowSpan={}", col_addr, row_addr, col_span, row_span);
                        eprintln!("           cellW={} cellH={} pad=({},{},{},{}) borderFillId={} cellW2={}",
                            cell_w, cell_h, padding_l, padding_r, padding_t, padding_b, border_fill_id, cell_w2);
                    }
                }
                eprintln!("         full hex: {}", hex_full(&rec.data));
            }
            else {
                eprintln!("         hex: {}", hex_dump(&rec.data, 60));
            }
        }

        // ============================================================
        // Load files
        // ============================================================
        let files: Vec<(&str, &str)> = vec![
            ("template/empty-step2-p.hwp", "VALID"),
            ("template/empty-step2_saved_err.hwp", "DAMAGED"),
        ];

        struct FileData {
            label: String,
            body_records: Vec<Record>,
            body_raw_len: usize,
        }

        let mut all_files: Vec<FileData> = Vec::new();

        for (path, label) in &files {
            let bytes = std::fs::read(path)
                .unwrap_or_else(|e| panic!("File read failed: {} - {}", path, e));
            let mut cfb = CfbReader::open(&bytes)
                .unwrap_or_else(|e| panic!("CFB open failed: {} - {}", path, e));
            let body_data = cfb.read_body_text_section(0, true, false)
                .unwrap_or_else(|e| panic!("BodyText read failed: {} - {}", path, e));
            let body_raw_len = body_data.len();
            let body_records = Record::read_all(&body_data)
                .unwrap_or_else(|e| panic!("Record parse failed: {} - {}", path, e));
            let rec_count = body_records.len();
            all_files.push(FileData { label: label.to_string(), body_records, body_raw_len });
            eprintln!("[{}] {} loaded: {} bytes decompressed, {} records", label, path, body_raw_len, rec_count);
        }

        let valid_recs = &all_files[0].body_records;
        let damaged_recs = &all_files[1].body_records;

        // ============================================================
        // Find the pasted table area: PARA_HEADER with control_mask containing 0x800
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  Finding pasted table PARA_HEADER (control_mask has bit 0x800 = table control)");
        eprintln!("{}", "=".repeat(120));

        let mut valid_table_start: Option<usize> = None;
        let mut damaged_table_start: Option<usize> = None;

        for (i, rec) in valid_recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.data.len() >= 8 {
                let mask = read_u32_le(&rec.data, 4);
                if mask & 0x800 != 0 {
                    eprintln!("  VALID: pasted table PARA_HEADER found at record {} (control_mask=0x{:08X})", i, mask);
                    if valid_table_start.is_none() {
                        valid_table_start = Some(i);
                    }
                }
            }
        }
        for (i, rec) in damaged_recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.data.len() >= 8 {
                let mask = read_u32_le(&rec.data, 4);
                if mask & 0x800 != 0 {
                    eprintln!("  DAMAGED: pasted table PARA_HEADER found at record {} (control_mask=0x{:08X})", i, mask);
                    if damaged_table_start.is_none() {
                        damaged_table_start = Some(i);
                    }
                }
            }
        }

        // Also find CTRL_HEADER with "tbl " to identify the second table
        eprintln!("\n  Scanning for ALL CTRL_HEADER 'tbl ' records:");
        for (i, rec) in valid_recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                if ctrl_id == 0x7462_6C20 { // " lbt" = "tbl " in big endian display
                    eprintln!("    VALID: tbl CTRL_HEADER at record {}", i);
                }
            }
        }
        for (i, rec) in damaged_recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_CTRL_HEADER && rec.data.len() >= 4 {
                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                if ctrl_id == 0x7462_6C20 {
                    eprintln!("    DAMAGED: tbl CTRL_HEADER at record {}", i);
                }
            }
        }

        // ============================================================
        // Dump records around the pasted table area in both files
        // ============================================================
        // Use the second table start if found, otherwise start from first table para header
        let v_start = valid_table_start.unwrap_or(30);
        let d_start = damaged_table_start.unwrap_or(30);

        // Print a generous range: from 4 records before the table para to end or +200 records
        let v_range_start = if v_start >= 4 { v_start - 4 } else { 0 };
        let d_range_start = if d_start >= 4 { d_start - 4 } else { 0 };
        let v_range_end = std::cmp::min(valid_recs.len(), v_start + 200);
        let d_range_end = std::cmp::min(damaged_recs.len(), d_start + 200);

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  VALID FILE: Records {} - {} (around pasted table)", v_range_start, v_range_end - 1);
        eprintln!("{}", "=".repeat(120));

        for i in v_range_start..v_range_end {
            print_record_detail("VALID", i, &valid_recs[i]);
        }

        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  DAMAGED FILE: Records {} - {} (around pasted table)", d_range_start, d_range_end - 1);
        eprintln!("{}", "=".repeat(120));

        for i in d_range_start..d_range_end {
            print_record_detail("DAMAGED", i, &damaged_recs[i]);
        }

        // ============================================================
        // Side-by-side comparison of matching region
        // ============================================================
        eprintln!("\n{}", "=".repeat(120));
        eprintln!("  SIDE-BY-SIDE: VALID[{}..] vs DAMAGED[{}..] - first 60 records", v_start, d_start);
        eprintln!("{}", "=".repeat(120));

        let compare_count = 60;
        for offset in 0..compare_count {
            let vi = v_start + offset;
            let di = d_start + offset;
            if vi >= valid_recs.len() && di >= damaged_recs.len() { break; }

            let have_v = vi < valid_recs.len();
            let have_d = di < damaged_recs.len();

            let (v_tag, v_lvl, v_sz) = if have_v {
                (tags::tag_name(valid_recs[vi].tag_id).to_string(), valid_recs[vi].level, valid_recs[vi].size)
            } else { ("---".to_string(), 0u16, 0u32) };

            let (d_tag, d_lvl, d_sz) = if have_d {
                (tags::tag_name(damaged_recs[di].tag_id).to_string(), damaged_recs[di].level, damaged_recs[di].size)
            } else { ("---".to_string(), 0u16, 0u32) };

            let status = if !have_v || !have_d { "MISSING" }
                else if valid_recs[vi].tag_id != damaged_recs[di].tag_id { "TAG_DIFF" }
                else if valid_recs[vi].level != damaged_recs[di].level { "LVL_DIFF" }
                else if valid_recs[vi].data != damaged_recs[di].data { "DATA_DIFF" }
                else { "OK" };

            let marker = if status != "OK" { ">>>" } else { "   " };
            eprintln!("{} off={:<3} V[{:>3}] {:<20} lvl={} sz={:<5} | D[{:>3}] {:<20} lvl={} sz={:<5} | {}",
                marker, offset, vi, v_tag, v_lvl, v_sz, di, d_tag, d_lvl, d_sz, status);

            if status != "OK" && have_v && have_d {
                // Show critical details for differing records
                let vr = &valid_recs[vi];
                let dr = &damaged_recs[di];

                if vr.tag_id == tags::HWPTAG_PARA_HEADER && dr.tag_id == tags::HWPTAG_PARA_HEADER {
                    let v_cc = read_u32_le(&vr.data, 0);
                    let d_cc = read_u32_le(&dr.data, 0);
                    let v_mask = read_u32_le(&vr.data, 4);
                    let d_mask = read_u32_le(&dr.data, 4);
                    let v_ps = read_u16_le(&vr.data, 8);
                    let d_ps = read_u16_le(&dr.data, 8);
                    eprintln!("       V: cc={} mask=0x{:08X} ps={}  D: cc={} mask=0x{:08X} ps={}",
                        v_cc & 0x7FFFFFFF, v_mask, v_ps, d_cc & 0x7FFFFFFF, d_mask, d_ps);
                }

                if vr.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && dr.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                    let v_pairs = vr.data.len() / 8;
                    let d_pairs = dr.data.len() / 8;
                    eprintln!("       V pairs: {} D pairs: {}", v_pairs, d_pairs);
                    for p in 0..std::cmp::max(v_pairs, d_pairs) {
                        let v_str = if p < v_pairs {
                            format!("pos{}=>CS{}", read_u32_le(&vr.data, p*8), read_u32_le(&vr.data, p*8+4))
                        } else { "---".to_string() };
                        let d_str = if p < d_pairs {
                            format!("pos{}=>CS{}", read_u32_le(&dr.data, p*8), read_u32_le(&dr.data, p*8+4))
                        } else { "---".to_string() };
                        eprintln!("         [{}] V: {}  D: {}", p, v_str, d_str);
                    }
                }

                if vr.tag_id == tags::HWPTAG_LIST_HEADER && dr.tag_id == tags::HWPTAG_LIST_HEADER {
                    let v_pc = read_u16_le(&vr.data, 0);
                    let d_pc = read_u16_le(&dr.data, 0);
                    let v_bf = if vr.data.len() >= 34 { read_u16_le(&vr.data, 32) } else { 0 };
                    let d_bf = if dr.data.len() >= 34 { read_u16_le(&dr.data, 32) } else { 0 };
                    eprintln!("       V: paraCount={} borderFillId={}  D: paraCount={} borderFillId={}",
                        v_pc, v_bf, d_pc, d_bf);
                    if vr.data.len() >= 14 && dr.data.len() >= 14 {
                        let v_col = read_u16_le(&vr.data, 8);
                        let v_row = read_u16_le(&vr.data, 10);
                        let d_col = read_u16_le(&dr.data, 8);
                        let d_row = read_u16_le(&dr.data, 10);
                        eprintln!("       V: col={} row={}  D: col={} row={}", v_col, v_row, d_col, d_row);
                    }
                }

                // Show hex diff
                eprintln!("       V hex: {}", hex_dump(&vr.data, 60));
                eprintln!("       D hex: {}", hex_dump(&dr.data, 60));

                // Find first diff byte
                let min_len = std::cmp::min(vr.data.len(), dr.data.len());
                if let Some(pos) = (0..min_len).find(|&j| vr.data[j] != dr.data[j]) {
                    eprintln!("       First diff at byte {}: V=0x{:02x} D=0x{:02x}", pos, vr.data[pos], dr.data[pos]);
                }
            }
        }

        eprintln!("\n=== test_step2_paste_area complete ===");
    }

    #[test]
    fn test_simple_text_insert_and_save() {
        // template/empty.hwp 로드 → 텍스트 삽입 → 저장
        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("=== 단순 텍스트 삽입 + 저장 테스트 ===");
        eprintln!("원본: {} bytes, {}페이지, {}개 구역",
            data.len(), doc.page_count(),
            doc.document.sections.len());

        // 첫 번째 구역, 첫 번째 문단에 텍스트 삽입
        let section = &doc.document.sections[0];
        eprintln!("문단 수: {}", section.paragraphs.len());
        for (i, p) in section.paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text='{}' controls={} line_segs={}",
                i, p.text, p.controls.len(), p.line_segs.len());
        }

        // "가나다라마바사아" 삽입
        let result = doc.insert_text_native(0, 0, 0, "가나다라마바사아");
        assert!(result.is_ok(), "텍스트 삽입 실패: {:?}", result.err());
        eprintln!("텍스트 삽입 결과: {}", result.unwrap());

        // 삽입 후 상태 확인
        let section = &doc.document.sections[0];
        eprintln!("삽입 후 문단[0]: text='{}'", section.paragraphs[0].text);

        // HWP 내보내기
        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "HWP 내보내기 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();
        eprintln!("저장된 파일: {} bytes", saved_data.len());

        // output/ 폴더에 저장
        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/empty_with_text.hwp", &saved_data).unwrap();
        eprintln!("output/empty_with_text.hwp 저장 완료");

        // 저장된 파일 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "저장된 파일 재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();
        eprintln!("재파싱 성공: {}페이지", doc2.page_count());

        let section2 = &doc2.document.sections[0];
        eprintln!("재파싱 문단[0]: text='{}'", section2.paragraphs[0].text);
        assert!(section2.paragraphs[0].text.contains("가나다라마바사아"),
            "저장된 파일에 삽입한 텍스트가 없음");
    }

    #[test]
    fn test_empty_save_analysis() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::header;
        use std::collections::BTreeMap;

        let orig_path = "template/empty.hwp";
        let saved_path = "output/empty_with_text.hwp";

        if !std::path::Path::new(orig_path).exists() {
            eprintln!("SKIP: {} 없음", orig_path);
            return;
        }
        if !std::path::Path::new(saved_path).exists() {
            // 먼저 저장 파일을 생성한다
            eprintln!("output/empty_with_text.hwp 없음 - 생성 시도...");
            let data = std::fs::read(orig_path).unwrap();
            let mut doc = HwpDocument::from_bytes(&data).unwrap();
            let _ = doc.insert_text_native(0, 0, 0, "가나다라마바사아");
            let saved = doc.export_hwp_native().unwrap();
            let _ = std::fs::create_dir_all("output");
            std::fs::write(saved_path, &saved).unwrap();
            eprintln!("output/empty_with_text.hwp 생성 완료");
        }

        let orig_data = std::fs::read(orig_path)
            .unwrap_or_else(|e| panic!("원본 파일 읽기 실패: {}", e));
        let saved_data = std::fs::read(saved_path)
            .unwrap_or_else(|e| panic!("저장 파일 읽기 실패: {}", e));

        println!("\n{}", "=".repeat(80));
        println!("  EMPTY HWP vs SAVED-WITH-TEXT HWP ANALYSIS");
        println!("  Original:  {} ({} bytes)", orig_path, orig_data.len());
        println!("  Saved:     {} ({} bytes)", saved_path, saved_data.len());
        println!("{}", "=".repeat(80));

        // ============================================================
        // 1. FILE SIZE COMPARISON
        // ============================================================
        println!("\n--- 1. FILE SIZE COMPARISON ---");
        println!("Original: {} bytes", orig_data.len());
        println!("Saved:    {} bytes", saved_data.len());
        println!("Diff:     {} bytes", saved_data.len() as i64 - orig_data.len() as i64);

        // ============================================================
        // 2. CFB STREAM LIST AND SIZES
        // ============================================================
        println!("\n--- 2. CFB STREAM LIST AND SIZES ---");

        let orig_cfb = CfbReader::open(&orig_data).expect("원본 CFB 열기 실패");
        let saved_cfb = CfbReader::open(&saved_data).expect("저장 CFB 열기 실패");

        let orig_entries: BTreeMap<String, (u64, bool)> = orig_cfb
            .list_all_entries()
            .into_iter()
            .map(|(path, size, is_stream)| (path, (size, is_stream)))
            .collect();

        let saved_entries: BTreeMap<String, (u64, bool)> = saved_cfb
            .list_all_entries()
            .into_iter()
            .map(|(path, size, is_stream)| (path, (size, is_stream)))
            .collect();

        println!("\n{:<40} {:>10} {:>10} {:>8}", "Path", "Orig Size", "Saved Size", "Type");
        println!("{:-<72}", "");

        let all_paths: std::collections::BTreeSet<&String> =
            orig_entries.keys().chain(saved_entries.keys()).collect();

        for path in &all_paths {
            let orig_info = orig_entries.get(*path);
            let saved_info = saved_entries.get(*path);
            let orig_size = orig_info.map(|(s, _)| format!("{}", s)).unwrap_or_else(|| "---".to_string());
            let saved_size = saved_info.map(|(s, _)| format!("{}", s)).unwrap_or_else(|| "---".to_string());
            let type_str = orig_info.or(saved_info).map(|(_, is)| if *is { "stream" } else { "storage" }).unwrap_or("?");
            let marker = if orig_info.is_none() {
                " [NEW]"
            } else if saved_info.is_none() {
                " [MISSING]"
            } else if orig_info.map(|(s,_)| s) != saved_info.map(|(s,_)| s) {
                " [CHANGED]"
            } else {
                ""
            };
            println!("{:<40} {:>10} {:>10} {:>8}{}", path, orig_size, saved_size, type_str, marker);
        }

        // ============================================================
        // 3. FileHeader COMPARISON
        // ============================================================
        println!("\n--- 3. FileHeader COMPARISON ---");
        let mut orig_cfb2 = CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb2 = CfbReader::open(&saved_data).unwrap();

        let orig_header_raw = orig_cfb2.read_file_header().unwrap();
        let saved_header_raw = saved_cfb2.read_file_header().unwrap();

        let orig_fh = header::parse_file_header(&orig_header_raw).unwrap();
        let saved_fh = header::parse_file_header(&saved_header_raw).unwrap();

        println!("Original: version={}.{}.{}.{} flags=0x{:08X} compressed={} encrypted={} distribution={}",
            orig_fh.version.major, orig_fh.version.minor, orig_fh.version.build, orig_fh.version.revision,
            orig_fh.flags.raw, orig_fh.flags.compressed, orig_fh.flags.encrypted, orig_fh.flags.distribution);
        println!("Saved:    version={}.{}.{}.{} flags=0x{:08X} compressed={} encrypted={} distribution={}",
            saved_fh.version.major, saved_fh.version.minor, saved_fh.version.build, saved_fh.version.revision,
            saved_fh.flags.raw, saved_fh.flags.compressed, saved_fh.flags.encrypted, saved_fh.flags.distribution);

        if orig_header_raw == saved_header_raw {
            println!("FileHeaders are IDENTICAL.");
        } else {
            println!("FileHeaders DIFFER:");
            for i in 0..std::cmp::max(orig_header_raw.len(), saved_header_raw.len()) {
                let ob = orig_header_raw.get(i).copied();
                let sb = saved_header_raw.get(i).copied();
                if ob != sb {
                    println!("  offset {:#06x}: orig=0x{:02X} saved=0x{:02X}",
                        i, ob.unwrap_or(0), sb.unwrap_or(0));
                }
            }
        }

        // ============================================================
        // 4. DocInfo STREAM COMPARISON (byte-level)
        // ============================================================
        println!("\n--- 4. DocInfo STREAM COMPARISON (byte-level) ---");
        let orig_compressed = orig_fh.flags.compressed;
        let saved_compressed = saved_fh.flags.compressed;

        let mut orig_cfb3 = CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb3 = CfbReader::open(&saved_data).unwrap();

        let orig_docinfo = orig_cfb3.read_doc_info(orig_compressed).unwrap();
        let saved_docinfo = saved_cfb3.read_doc_info(saved_compressed).unwrap();

        println!("Original DocInfo (decompressed): {} bytes", orig_docinfo.len());
        println!("Saved DocInfo (decompressed):    {} bytes", saved_docinfo.len());

        if orig_docinfo == saved_docinfo {
            println!("DocInfo streams are IDENTICAL.");
        } else {
            println!("DocInfo streams DIFFER!");
            let min_len = std::cmp::min(orig_docinfo.len(), saved_docinfo.len());
            let mut diff_count = 0;
            let mut first_diff_pos = None;
            for i in 0..min_len {
                if orig_docinfo[i] != saved_docinfo[i] {
                    if diff_count < 20 {
                        println!("  offset {:#06x}: orig=0x{:02X} saved=0x{:02X}", i, orig_docinfo[i], saved_docinfo[i]);
                    }
                    if first_diff_pos.is_none() {
                        first_diff_pos = Some(i);
                    }
                    diff_count += 1;
                }
            }
            if orig_docinfo.len() != saved_docinfo.len() {
                println!("  Size difference: orig={} saved={} (diff={})",
                    orig_docinfo.len(), saved_docinfo.len(),
                    saved_docinfo.len() as i64 - orig_docinfo.len() as i64);
            }
            println!("  Total differing bytes: {} (first at offset {:?})", diff_count, first_diff_pos);

            // Parse DocInfo records for comparison
            println!("\n  --- DocInfo Record-by-Record ---");
            let orig_di_records = Record::read_all(&orig_docinfo).unwrap_or_default();
            let saved_di_records = Record::read_all(&saved_docinfo).unwrap_or_default();
            println!("  Original DocInfo records: {}", orig_di_records.len());
            println!("  Saved DocInfo records:    {}", saved_di_records.len());

            let max_di = std::cmp::max(orig_di_records.len(), saved_di_records.len());
            for i in 0..max_di {
                let orig_r = orig_di_records.get(i);
                let saved_r = saved_di_records.get(i);
                let matches = match (orig_r, saved_r) {
                    (Some(o), Some(s)) => o.tag_id == s.tag_id && o.level == s.level && o.size == s.size && o.data == s.data,
                    _ => false,
                };
                if !matches {
                    let orig_str = orig_r.map(|r| format!("{} lvl={} sz={}", r.tag_name(), r.level, r.size))
                        .unwrap_or_else(|| "---".to_string());
                    let saved_str = saved_r.map(|r| format!("{} lvl={} sz={}", r.tag_name(), r.level, r.size))
                        .unwrap_or_else(|| "---".to_string());
                    println!("  [{}] ORIG: {:<40} SAVED: {}", i, orig_str, saved_str);
                    // If tags match but data differs, show data diff
                    if let (Some(o), Some(s)) = (orig_r, saved_r) {
                        if o.tag_id == s.tag_id && o.data != s.data {
                            let show = std::cmp::min(40, std::cmp::max(o.data.len(), s.data.len()));
                            print!("       orig data: ");
                            for b in &o.data[..std::cmp::min(show, o.data.len())] { print!("{:02x} ", b); }
                            println!();
                            print!("       saved data: ");
                            for b in &s.data[..std::cmp::min(show, s.data.len())] { print!("{:02x} ", b); }
                            println!();
                        }
                    }
                }
            }
        }

        // ============================================================
        // 5. BodyText/Section0 RECORD-BY-RECORD COMPARISON
        // ============================================================
        println!("\n--- 5. BodyText/Section0 RECORD-BY-RECORD COMPARISON ---");
        let mut orig_cfb4 = CfbReader::open(&orig_data).unwrap();
        let mut saved_cfb4 = CfbReader::open(&saved_data).unwrap();

        let orig_section = orig_cfb4.read_body_text_section(0, orig_compressed, false).unwrap();
        let saved_section = saved_cfb4.read_body_text_section(0, saved_compressed, false).unwrap();

        println!("Original Section0 (decompressed): {} bytes", orig_section.len());
        println!("Saved Section0 (decompressed):    {} bytes", saved_section.len());

        let orig_records = Record::read_all(&orig_section).unwrap();
        let saved_records = Record::read_all(&saved_section).unwrap();

        println!("Original records: {}", orig_records.len());
        println!("Saved records:    {}", saved_records.len());

        // Helper functions
        fn hex_dump_n(data: &[u8], max: usize) -> String {
            let show = std::cmp::min(data.len(), max);
            let hex: Vec<String> = data[..show].iter().map(|b| format!("{:02x}", b)).collect();
            let mut result = hex.join(" ");
            if data.len() > max {
                result.push_str(&format!(" ...({} more)", data.len() - max));
            }
            result
        }

        fn decode_para_header(data: &[u8]) -> String {
            if data.len() < 8 {
                return format!("(too short: {} bytes)", data.len());
            }
            // PARA_HEADER structure:
            // 0-3: nCharCount (lower 31 bits = char count, bit 31 = char_count_msb)
            // 4-7: controlMask (u32)
            // 8-9: paraShapeId (u16)
            // 10: paraStyleId (u8)
            // 11: columnSplit (u8)
            // 12-13: charShapeCount (u16)
            // 14-15: rangeTagCount (u16)
            // 16-17: lineAlignCount (u16)
            // 18-19: instanceID (u16)
            let raw = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let char_count = raw & 0x7FFFFFFF;
            let char_count_msb = (raw >> 31) & 1;
            let ctrl_mask = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            let mut result = format!("char_count={} msb={} ctrl_mask=0x{:08X}", char_count, char_count_msb, ctrl_mask);
            if data.len() >= 10 {
                let para_shape_id = u16::from_le_bytes([data[8], data[9]]);
                result.push_str(&format!(" paraShapeId={}", para_shape_id));
            }
            if data.len() >= 11 {
                result.push_str(&format!(" paraStyleId={}", data[10]));
            }
            if data.len() >= 14 {
                let cs_count = u16::from_le_bytes([data[12], data[13]]);
                result.push_str(&format!(" charShapeCount={}", cs_count));
            }
            if data.len() >= 16 {
                let rt_count = u16::from_le_bytes([data[14], data[15]]);
                result.push_str(&format!(" rangeTagCount={}", rt_count));
            }
            if data.len() >= 18 {
                let la_count = u16::from_le_bytes([data[16], data[17]]);
                result.push_str(&format!(" lineAlignCount={}", la_count));
            }
            if data.len() >= 20 {
                let inst_id = u16::from_le_bytes([data[18], data[19]]);
                result.push_str(&format!(" instanceID={}", inst_id));
            }
            result
        }

        fn decode_para_text(data: &[u8]) -> String {
            // UTF-16LE text stream with control chars
            let mut result = String::new();
            let mut i = 0;
            let mut char_pos = 0;
            while i + 1 < data.len() {
                let ch = u16::from_le_bytes([data[i], data[i + 1]]);
                match ch {
                    // Extended controls take 8 WCHARs (16 bytes)
                    0x0001 | 0x0002 | 0x0003 | 0x000B | 0x000C | 0x000D | 0x000E | 0x000F
                    | 0x0004 | 0x0005 | 0x0006 | 0x0007 | 0x0008 | 0x0009 | 0x000A => {
                        let name = match ch {
                            0x0002 => "SEC/COL",
                            0x0003 => "FIELD_BEGIN",
                            0x0004 => "FIELD_END",
                            0x0008 => "INLINE",
                            0x000B => "EXT_CTRL",
                            0x000D => "PARA_BREAK",
                            0x000A => "LINE_BREAK",
                            _ => "CTRL",
                        };
                        result.push_str(&format!("[{}@{}]", name, char_pos));
                        // Extended controls (2,3,11,12,13,14,15) occupy 8 WCHARs
                        if ch >= 1 && ch <= 9 {
                            // These chars occupy 8 WCHARs (16 bytes)
                            i += 16;
                            char_pos += 8;
                        } else {
                            i += 2;
                            char_pos += 1;
                        }
                    }
                    _ => {
                        if let Some(c) = char::from_u32(ch as u32) {
                            result.push(c);
                        } else {
                            result.push_str(&format!("\\u{:04X}", ch));
                        }
                        i += 2;
                        char_pos += 1;
                    }
                }
            }
            result
        }

        fn decode_para_char_shape(data: &[u8]) -> String {
            // Array of (position: u32, charShapeId: u32) pairs
            let pair_count = data.len() / 8;
            let mut result = format!("{} pairs: ", pair_count);
            for p in 0..pair_count {
                let off = p * 8;
                if off + 8 > data.len() { break; }
                let pos = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                let id = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
                if p > 0 { result.push_str(", "); }
                result.push_str(&format!("(pos={}, id={})", pos, id));
            }
            result
        }

        fn decode_para_line_seg(data: &[u8]) -> String {
            // Each line segment is 36 bytes:
            // textStartPos(4) + lineVPos(4) + lineHPos(4) + lineHeight(4)
            // + textPartHeight(4) + distBaseline(4) + lineSpacing(4) + colStartPos(4) + segWidth(4)
            // Some versions use 32 bytes per segment
            let seg_size = if data.len() % 36 == 0 { 36 } else if data.len() % 32 == 0 { 32 } else { 36 };
            let seg_count = if seg_size > 0 { data.len() / seg_size } else { 0 };
            let mut result = format!("{} segments ({}B each): ", seg_count, seg_size);
            for s in 0..std::cmp::min(seg_count, 4) {
                let off = s * seg_size;
                if off + 16 > data.len() { break; }
                let text_start = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                let line_vpos = i32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
                let line_hpos = i32::from_le_bytes([data[off+8], data[off+9], data[off+10], data[off+11]]);
                let line_height = u32::from_le_bytes([data[off+12], data[off+13], data[off+14], data[off+15]]);
                if s > 0 { result.push_str(", "); }
                result.push_str(&format!("[start={} v={} h={} h={}]", text_start, line_vpos, line_hpos, line_height));
            }
            if seg_count > 4 {
                result.push_str(&format!(" ...({} more)", seg_count - 4));
            }
            result
        }

        fn decode_ctrl_header(data: &[u8]) -> String {
            if data.len() < 4 {
                return format!("(too short: {} bytes)", data.len());
            }
            let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let be_bytes = ctrl_id.to_be_bytes();
            let ascii: String = be_bytes.iter().map(|&b| {
                if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
            }).collect();
            format!("ctrl_id=0x{:08X} \"{}\" ({})", ctrl_id, ascii, tags::ctrl_name(ctrl_id))
        }

        // Print all records with decoded details
        println!("\n  {:<4} {:<22} {:>3} {:>6}  |  {:<22} {:>3} {:>6}  | Status",
            "#", "Orig Tag", "Lvl", "Size", "Saved Tag", "Lvl", "Size");
        println!("  {:-<110}", "");

        let max_recs = std::cmp::max(orig_records.len(), saved_records.len());
        for i in 0..max_recs {
            let orig_r = orig_records.get(i);
            let saved_r = saved_records.get(i);

            let orig_str = orig_r.map(|r| format!("{:<22} {:>3} {:>6}", r.tag_name(), r.level, r.size))
                .unwrap_or_else(|| format!("{:<22} {:>3} {:>6}", "---", "", ""));
            let saved_str = saved_r.map(|r| format!("{:<22} {:>3} {:>6}", r.tag_name(), r.level, r.size))
                .unwrap_or_else(|| format!("{:<22} {:>3} {:>6}", "---", "", ""));

            let status = match (orig_r, saved_r) {
                (Some(o), Some(s)) => {
                    if o.tag_id == s.tag_id && o.level == s.level && o.size == s.size && o.data == s.data {
                        "OK"
                    } else if o.tag_id == s.tag_id && o.level == s.level && o.size == s.size {
                        "DATA_DIFF"
                    } else if o.tag_id == s.tag_id && o.level == s.level {
                        "SIZE_DIFF"
                    } else if o.tag_id == s.tag_id {
                        "LEVEL_DIFF"
                    } else {
                        "TAG_DIFF"
                    }
                }
                (Some(_), None) => "ORIG_ONLY",
                (None, Some(_)) => "SAVED_ONLY",
                (None, None) => "???",
            };

            // Always print, even OK, so we can see the full record layout
            println!("  {:<4} {}  |  {}  | {}", i, orig_str, saved_str, status);

            // Decode details for non-OK records
            if status != "OK" {
                // Show decoded info for both
                for (label, rec) in [("ORIG", orig_r), ("SAVED", saved_r)] {
                    if let Some(r) = rec {
                        let detail = match r.tag_id {
                            t if t == tags::HWPTAG_PARA_HEADER => decode_para_header(&r.data),
                            t if t == tags::HWPTAG_PARA_TEXT => {
                                let text_decoded = decode_para_text(&r.data);
                                format!("hex[0..40]: {}  text: {}", hex_dump_n(&r.data, 40), text_decoded)
                            }
                            t if t == tags::HWPTAG_PARA_CHAR_SHAPE => decode_para_char_shape(&r.data),
                            t if t == tags::HWPTAG_PARA_LINE_SEG => decode_para_line_seg(&r.data),
                            t if t == tags::HWPTAG_CTRL_HEADER => decode_ctrl_header(&r.data),
                            _ => format!("hex: {}", hex_dump_n(&r.data, 40)),
                        };
                        println!("       {}: {}", label, detail);
                    }
                }
            }
        }

        // ============================================================
        // 6. Record SUMMARY & STATISTICS
        // ============================================================
        println!("\n--- 6. RECORD SUMMARY ---");
        println!("\nOriginal record types:");
        let mut orig_tag_counts: BTreeMap<u16, usize> = BTreeMap::new();
        for r in &orig_records {
            *orig_tag_counts.entry(r.tag_id).or_insert(0) += 1;
        }
        for (tag, count) in &orig_tag_counts {
            println!("  {:>3} ({:<22}): {}", tag, tags::tag_name(*tag), count);
        }

        println!("\nSaved record types:");
        let mut saved_tag_counts: BTreeMap<u16, usize> = BTreeMap::new();
        for r in &saved_records {
            *saved_tag_counts.entry(r.tag_id).or_insert(0) += 1;
        }
        for (tag, count) in &saved_tag_counts {
            println!("  {:>3} ({:<22}): {}", tag, tags::tag_name(*tag), count);
        }

        // ============================================================
        // 7. PARA_HEADER DETAIL for ALL paragraphs
        // ============================================================
        println!("\n--- 7. ALL PARA_HEADER DETAILS ---");
        println!("\n  Original paragraphs:");
        for (i, r) in orig_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_HEADER {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_header(&r.data));
                println!("         hex: {}", hex_dump_n(&r.data, 40));
            }
        }
        println!("\n  Saved paragraphs:");
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_HEADER {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_header(&r.data));
                println!("         hex: {}", hex_dump_n(&r.data, 40));
            }
        }

        // ============================================================
        // 8. PARA_TEXT DETAIL (hex + decoded)
        // ============================================================
        println!("\n--- 8. ALL PARA_TEXT DETAILS ---");
        println!("\n  Original PARA_TEXT:");
        for (i, r) in orig_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_TEXT {
                println!("    [{}] lvl={} sz={}", i, r.level, r.size);
                println!("      hex: {}", hex_dump_n(&r.data, 60));
                println!("      decoded: {}", decode_para_text(&r.data));
            }
        }
        println!("\n  Saved PARA_TEXT:");
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_TEXT {
                println!("    [{}] lvl={} sz={}", i, r.level, r.size);
                println!("      hex: {}", hex_dump_n(&r.data, 60));
                println!("      decoded: {}", decode_para_text(&r.data));
            }
        }

        // ============================================================
        // 9. PARA_CHAR_SHAPE DETAIL
        // ============================================================
        println!("\n--- 9. ALL PARA_CHAR_SHAPE DETAILS ---");
        println!("\n  Original PARA_CHAR_SHAPE:");
        for (i, r) in orig_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_char_shape(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 40));
            }
        }
        println!("\n  Saved PARA_CHAR_SHAPE:");
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_char_shape(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 40));
            }
        }

        // ============================================================
        // 10. PARA_LINE_SEG DETAIL
        // ============================================================
        println!("\n--- 10. ALL PARA_LINE_SEG DETAILS ---");
        println!("\n  Original PARA_LINE_SEG:");
        for (i, r) in orig_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_line_seg(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 60));
            }
        }
        println!("\n  Saved PARA_LINE_SEG:");
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_para_line_seg(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 60));
            }
        }

        // ============================================================
        // 11. CTRL_HEADER DETAIL
        // ============================================================
        println!("\n--- 11. ALL CTRL_HEADER DETAILS ---");
        println!("\n  Original CTRL_HEADER:");
        for (i, r) in orig_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_CTRL_HEADER {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_ctrl_header(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 60));
            }
        }
        println!("\n  Saved CTRL_HEADER:");
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_CTRL_HEADER {
                println!("    [{}] lvl={} sz={}: {}", i, r.level, r.size, decode_ctrl_header(&r.data));
                println!("      hex: {}", hex_dump_n(&r.data, 60));
            }
        }

        // ============================================================
        // 12. BinData STREAMS
        // ============================================================
        println!("\n--- 12. BinData STREAMS ---");
        let orig_bins: Vec<_> = orig_entries.keys().filter(|k| k.starts_with("/BinData/")).collect();
        let saved_bins: Vec<_> = saved_entries.keys().filter(|k| k.starts_with("/BinData/")).collect();
        println!("Original BinData: {} streams", orig_bins.len());
        for p in &orig_bins {
            println!("  {} (size: {})", p, orig_entries[*p].0);
        }
        println!("Saved BinData: {} streams", saved_bins.len());
        for p in &saved_bins {
            println!("  {} (size: {})", p, saved_entries[*p].0);
        }

        // ============================================================
        // 13. FULL RAW HEX DUMP of Section0 for small files
        // ============================================================
        if saved_section.len() <= 2000 {
            println!("\n--- 13. FULL RAW HEX DUMP (Saved Section0, {} bytes) ---", saved_section.len());
            for chunk_start in (0..saved_section.len()).step_by(32) {
                let chunk_end = std::cmp::min(chunk_start + 32, saved_section.len());
                print!("  {:06x}: ", chunk_start);
                for i in chunk_start..chunk_end {
                    print!("{:02x} ", saved_section[i]);
                }
                // ASCII view
                print!("  ");
                for i in chunk_start..chunk_end {
                    let b = saved_section[i];
                    if b >= 0x20 && b < 0x7f {
                        print!("{}", b as char);
                    } else {
                        print!(".");
                    }
                }
                println!();
            }
        }

        if orig_section.len() <= 2000 {
            println!("\n--- 13b. FULL RAW HEX DUMP (Original Section0, {} bytes) ---", orig_section.len());
            for chunk_start in (0..orig_section.len()).step_by(32) {
                let chunk_end = std::cmp::min(chunk_start + 32, orig_section.len());
                print!("  {:06x}: ", chunk_start);
                for i in chunk_start..chunk_end {
                    print!("{:02x} ", orig_section[i]);
                }
                print!("  ");
                for i in chunk_start..chunk_end {
                    let b = orig_section[i];
                    if b >= 0x20 && b < 0x7f {
                        print!("{}", b as char);
                    } else {
                        print!(".");
                    }
                }
                println!();
            }
        }

        // ============================================================
        // 14. DIAGNOSIS SUMMARY
        // ============================================================
        println!("\n{}", "=".repeat(80));
        println!("  DIAGNOSIS SUMMARY");
        println!("{}", "=".repeat(80));

        let mut issues: Vec<String> = Vec::new();

        // Check record count differences
        if orig_records.len() != saved_records.len() {
            issues.push(format!("Record count differs: orig={} saved={}", orig_records.len(), saved_records.len()));
        }

        // Check for PARA_HEADER with invalid char_count_msb
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_HEADER && r.data.len() >= 4 {
                let raw = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                let msb = (raw >> 31) & 1;
                let char_count = raw & 0x7FFFFFFF;
                if msb != 0 && msb != 1 {
                    issues.push(format!("Record {}: PARA_HEADER invalid msb={}", i, msb));
                }
                // Check if char_count matches PARA_TEXT length
                if let Some(text_rec) = saved_records.get(i + 1) {
                    if text_rec.tag_id == tags::HWPTAG_PARA_TEXT {
                        let text_wchars = text_rec.data.len() / 2;
                        // The char_count includes control character widths
                        // (extended ctrls = 8 WCHARs each)
                        if char_count == 0 && text_wchars > 0 {
                            issues.push(format!("Record {}: PARA_HEADER char_count=0 but PARA_TEXT has {} WCHARs", i, text_wchars));
                        }
                    }
                }
            }
        }

        // Check PARA_LINE_SEG size validity
        for (i, r) in saved_records.iter().enumerate() {
            if r.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                if r.data.len() % 36 != 0 && r.data.len() % 32 != 0 {
                    issues.push(format!("Record {}: PARA_LINE_SEG invalid size {} (not multiple of 32 or 36)", i, r.data.len()));
                }
            }
        }

        // Check for data differences in matching records
        let min_count = std::cmp::min(orig_records.len(), saved_records.len());
        let mut diff_records: Vec<usize> = Vec::new();
        for i in 0..min_count {
            if orig_records[i].tag_id == saved_records[i].tag_id
                && orig_records[i].data != saved_records[i].data
            {
                diff_records.push(i);
            }
        }
        if !diff_records.is_empty() {
            issues.push(format!("Records with data differences (same tag): {:?}", diff_records));
        }

        // Check for missing or extra records
        if orig_records.len() > saved_records.len() {
            issues.push(format!("Saved file is MISSING {} records from original", orig_records.len() - saved_records.len()));
        } else if saved_records.len() > orig_records.len() {
            issues.push(format!("Saved file has {} EXTRA records compared to original", saved_records.len() - orig_records.len()));
        }

        // DocInfo differences
        if orig_docinfo != saved_docinfo {
            issues.push(format!("DocInfo streams differ: orig={} bytes, saved={} bytes",
                orig_docinfo.len(), saved_docinfo.len()));
        }

        if issues.is_empty() {
            println!("\n  No obvious issues found.");
        } else {
            println!("\n  Found {} potential issues:", issues.len());
            for (i, issue) in issues.iter().enumerate() {
                println!("  {}. {}", i + 1, issue);
            }
        }

        println!("\n  Analysis complete.");
    }

    #[test]
    fn test_text_insert_detailed_diff() {
        // 텍스트 삽입 후 저장된 파일의 모든 레코드를 원본과 상세 비교
        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let orig_data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // 텍스트 삽입
        doc.insert_text_native(0, 0, 0, "가나다라마바사아");
        let saved = doc.export_hwp_native().unwrap();

        // 레코드 파싱
        use crate::parser::record::Record;
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        let saved_doc = crate::parser::parse_hwp(&saved).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\n=== 원본 레코드 ===");
        for (i, r) in orig_recs.iter().enumerate() {
            eprintln!("[{:2}] tag={:3} (0x{:04x}) level={} size={}", i, r.tag_id, r.tag_id, r.level, r.data.len());
        }
        eprintln!("\n=== 저장 레코드 ===");
        for (i, r) in saved_recs.iter().enumerate() {
            eprintln!("[{:2}] tag={:3} (0x{:04x}) level={} size={}", i, r.tag_id, r.tag_id, r.level, r.data.len());
        }

        // 레코드별 상세 비교
        let max = orig_recs.len().max(saved_recs.len());
        eprintln!("\n=== 바이트 비교 ===");
        for i in 0..max {
            let o = orig_recs.get(i);
            let s = saved_recs.get(i);
            match (o, s) {
                (Some(or), Some(sr)) => {
                    if or.tag_id != sr.tag_id || or.level != sr.level || or.data != sr.data {
                        eprintln!("DIFF [{}]: tag={}/{} level={}/{} size={}/{}",
                            i, or.tag_id, sr.tag_id, or.level, sr.level, or.data.len(), sr.data.len());
                        // HWP 레코드 태그 이름 매핑
                        let tag_name = match or.tag_id {
                            66 => "PARA_HEADER",
                            67 => "PARA_TEXT",
                            68 => "PARA_CHAR_SHAPE",
                            69 => "PARA_LINE_SEG",
                            70 => "CTRL_HEADER",
                            71 => "LIST_HEADER",
                            _ => "UNKNOWN",
                        };
                        eprintln!("  Record type: {}", tag_name);

                        // 전체 데이터 헥스 덤프
                        let orig_hex: Vec<String> = or.data.iter().map(|b| format!("{:02x}", b)).collect();
                        let save_hex: Vec<String> = sr.data.iter().map(|b| format!("{:02x}", b)).collect();
                        eprintln!("  ORIG[{}]: {}", or.data.len(), orig_hex.join(" "));
                        eprintln!("  SAVE[{}]: {}", sr.data.len(), save_hex.join(" "));

                        // 바이트별 차이 표시
                        let min_len = or.data.len().min(sr.data.len());
                        for pos in 0..min_len {
                            if or.data[pos] != sr.data[pos] {
                                eprintln!("  Byte {}: 0x{:02x} → 0x{:02x}", pos, or.data[pos], sr.data[pos]);
                            }
                        }
                        if or.data.len() != sr.data.len() {
                            eprintln!("  Size diff: {} → {} (delta {})", or.data.len(), sr.data.len(),
                                sr.data.len() as i64 - or.data.len() as i64);
                        }
                    } else {
                        eprintln!("OK   [{}]: tag={} level={} size={}", i, or.tag_id, or.level, or.data.len());
                    }
                }
                (Some(or), None) => eprintln!("MISSING [{}]: tag={}", i, or.tag_id),
                (None, Some(sr)) => eprintln!("EXTRA   [{}]: tag={}", i, sr.tag_id),
                _ => {}
            }
        }
    }

    #[test]
    fn test_roundtrip_no_edit() {
        // 편집 없이 raw_stream 무효화 → 재직렬화 → 저장
        // 재직렬화 자체에 문제가 있는지 분리 확인
        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let orig_data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        // raw_stream 무효화 (재직렬화 유도)
        doc.document.sections[0].raw_stream = None;

        let saved = doc.export_hwp_native().unwrap();
        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/empty_roundtrip.hwp", &saved).unwrap();
        eprintln!("output/empty_roundtrip.hwp 저장 ({} bytes)", saved.len());

        // 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());

        // 레코드별 비교
        use crate::parser::record::Record;
        let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
        let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
        let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
        let orig_recs = Record::read_all(&orig_bt).unwrap();

        let saved_doc = crate::parser::parse_hwp(&saved).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("원본 레코드: {}, 재직렬화 레코드: {}", orig_recs.len(), saved_recs.len());

        let max = orig_recs.len().max(saved_recs.len());
        for i in 0..max {
            let o = orig_recs.get(i);
            let s = saved_recs.get(i);
            match (o, s) {
                (Some(or), Some(sr)) => {
                    if or.tag_id != sr.tag_id || or.level != sr.level || or.data != sr.data {
                        eprintln!("DIFF [{}]: tag={}/{} level={}/{} size={}/{}",
                            i, or.tag_id, sr.tag_id, or.level, sr.level, or.data.len(), sr.data.len());
                        if or.data != sr.data {
                            let show = or.data.len().min(sr.data.len()).min(36);
                            eprintln!("  ORIG: {:02x?}", &or.data[..show]);
                            eprintln!("  SAVE: {:02x?}", &sr.data[..show]);
                            // 첫 번째 다른 바이트 위치
                            for (pos, (a, b)) in or.data.iter().zip(sr.data.iter()).enumerate() {
                                if a != b {
                                    eprintln!("  First diff at byte {}: 0x{:02x} vs 0x{:02x}", pos, a, b);
                                    break;
                                }
                            }
                        }
                    }
                }
                (Some(or), None) => eprintln!("MISSING in saved [{}]: tag={}", i, or.tag_id),
                (None, Some(sr)) => eprintln!("EXTRA in saved [{}]: tag={}", i, sr.tag_id),
                _ => {}
            }
        }
        eprintln!("비교 완료");
    }

    #[test]
    fn test_empty_hwp_editing_area() {
        // template/empty.hwp의 편집 영역, 캐럿 위치, LineSeg 값을 분석
        use crate::model::page::PageAreas;

        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("\n{}", "=".repeat(70));
        eprintln!("  EMPTY HWP 편집 영역 분석");
        eprintln!("{}", "=".repeat(70));

        // 1. DocProperties 캐럿 정보
        let props = &doc.document.doc_properties;
        eprintln!("\n--- DocProperties 캐럿 정보 ---");
        eprintln!("  caret_list_id:  {}", props.caret_list_id);
        eprintln!("  caret_para_id:  {}", props.caret_para_id);
        eprintln!("  caret_char_pos: {}", props.caret_char_pos);

        // 2. PageDef (용지 설정)
        let section = &doc.document.sections[0];
        let page_def = &section.section_def.page_def;
        eprintln!("\n--- PageDef (용지 설정) ---");
        eprintln!("  width:          {} HWPUNIT ({:.1}mm)", page_def.width, page_def.width as f64 / 283.46);
        eprintln!("  height:         {} HWPUNIT ({:.1}mm)", page_def.height, page_def.height as f64 / 283.46);
        eprintln!("  margin_left:    {} HWPUNIT ({:.1}mm)", page_def.margin_left, page_def.margin_left as f64 / 283.46);
        eprintln!("  margin_right:   {} HWPUNIT ({:.1}mm)", page_def.margin_right, page_def.margin_right as f64 / 283.46);
        eprintln!("  margin_top:     {} HWPUNIT ({:.1}mm)", page_def.margin_top, page_def.margin_top as f64 / 283.46);
        eprintln!("  margin_bottom:  {} HWPUNIT ({:.1}mm)", page_def.margin_bottom, page_def.margin_bottom as f64 / 283.46);
        eprintln!("  margin_header:  {} HWPUNIT ({:.1}mm)", page_def.margin_header, page_def.margin_header as f64 / 283.46);
        eprintln!("  margin_footer:  {} HWPUNIT ({:.1}mm)", page_def.margin_footer, page_def.margin_footer as f64 / 283.46);
        eprintln!("  margin_gutter:  {} HWPUNIT ({:.1}mm)", page_def.margin_gutter, page_def.margin_gutter as f64 / 283.46);
        eprintln!("  landscape:      {}", page_def.landscape);

        // 3. PageAreas (계산된 편집 영역)
        let areas = PageAreas::from_page_def(page_def);
        eprintln!("\n--- PageAreas (계산된 영역) ---");
        eprintln!("  header_area:    left={} top={} right={} bottom={}", areas.header_area.left, areas.header_area.top, areas.header_area.right, areas.header_area.bottom);
        eprintln!("  body_area:      left={} top={} right={} bottom={}", areas.body_area.left, areas.body_area.top, areas.body_area.right, areas.body_area.bottom);
        eprintln!("  body_area size: width={} height={}", areas.body_area.right - areas.body_area.left, areas.body_area.bottom - areas.body_area.top);
        eprintln!("  footer_area:    left={} top={} right={} bottom={}", areas.footer_area.left, areas.footer_area.top, areas.footer_area.right, areas.footer_area.bottom);

        // 4. 모든 문단의 LineSeg 정보
        eprintln!("\n--- 문단별 LineSeg 정보 ---");
        for (pi, para) in section.paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text='{}' char_count={} controls={}", pi, para.text, para.char_count, para.controls.len());
            eprintln!("    char_shapes: {:?}", para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>());
            for (li, ls) in para.line_segs.iter().enumerate() {
                eprintln!("    LineSeg[{}]:", li);
                eprintln!("      text_start:         {}", ls.text_start);
                eprintln!("      vertical_pos:       {} ({:.1}mm)", ls.vertical_pos, ls.vertical_pos as f64 / 283.46);
                eprintln!("      line_height:        {} ({:.1}mm)", ls.line_height, ls.line_height as f64 / 283.46);
                eprintln!("      text_height:        {} ({:.1}mm)", ls.text_height, ls.text_height as f64 / 283.46);
                eprintln!("      baseline_distance:  {} ({:.1}mm)", ls.baseline_distance, ls.baseline_distance as f64 / 283.46);
                eprintln!("      line_spacing:       {} ({:.1}mm)", ls.line_spacing, ls.line_spacing as f64 / 283.46);
                eprintln!("      column_start:       {} ({:.1}mm)", ls.column_start, ls.column_start as f64 / 283.46);
                eprintln!("      segment_width:      {} ({:.1}mm)", ls.segment_width, ls.segment_width as f64 / 283.46);
                eprintln!("      tag:                0x{:08x} (first_of_page={} first_of_col={})",
                    ls.tag, ls.is_first_line_of_page(), ls.is_first_line_of_column());
            }
        }

        // 5. 편집 영역 첫 줄 캐럿 위치 분석
        if let Some(first_para) = section.paragraphs.first() {
            if let Some(first_ls) = first_para.line_segs.first() {
                eprintln!("\n--- 편집 영역 첫 줄 캐럿 위치 분석 ---");
                eprintln!("  body_area.top (계산값):     {}", areas.body_area.top);
                eprintln!("  LineSeg.vertical_pos (실제): {}", first_ls.vertical_pos);
                eprintln!("  차이:                        {}", first_ls.vertical_pos - areas.body_area.top);
                eprintln!("  body_area.left (계산값):    {}", areas.body_area.left);
                eprintln!("  LineSeg.column_start (실제): {}", first_ls.column_start);
                eprintln!("  차이:                        {}", first_ls.column_start - areas.body_area.left);
                let body_width = areas.body_area.right - areas.body_area.left;
                eprintln!("  body_area.width (계산값):   {}", body_width);
                eprintln!("  LineSeg.segment_width (실제): {}", first_ls.segment_width);
                eprintln!("  차이:                        {}", first_ls.segment_width - body_width);
            }
        }

        // 6. ParaShape 정보 (첫 문단의 줄간격 등)
        if let Some(first_para) = section.paragraphs.first() {
            let ps_id = first_para.para_shape_id as usize;
            if ps_id < doc.document.doc_info.para_shapes.len() {
                let ps = &doc.document.doc_info.para_shapes[ps_id];
                eprintln!("\n--- ParaShape[{}] (첫 문단 문단모양) ---", ps_id);
                eprintln!("  line_spacing_type:  {:?}", ps.line_spacing_type);
                eprintln!("  line_spacing:       {}", ps.line_spacing);
                eprintln!("  line_spacing_v2:    {}", ps.line_spacing_v2);
                eprintln!("  margin_left:        {}", ps.margin_left);
                eprintln!("  margin_right:       {}", ps.margin_right);
            }
        }

        // 7. CharShape 정보 (첫 문단의 글자 크기)
        if let Some(first_para) = section.paragraphs.first() {
            if let Some(first_cs) = first_para.char_shapes.first() {
                let cs_id = first_cs.char_shape_id as usize;
                if cs_id < doc.document.doc_info.char_shapes.len() {
                    let cs = &doc.document.doc_info.char_shapes[cs_id];
                    eprintln!("\n--- CharShape[{}] (첫 문단 글자모양) ---", cs_id);
                    eprintln!("  base_size:  {} ({:.1}pt)", cs.base_size, cs.base_size as f64 / 100.0);
                }
            }
        }

        eprintln!("\n{}", "=".repeat(70));
    }

    #[test]
    fn test_save_text_only() {
        // 단계 2: 빈 HWP에 텍스트만 삽입 → 저장 → 바이트 비교
        use crate::parser::record::Record;
        use crate::parser::tags;

        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let orig_data = std::fs::read(path).unwrap();

        // 테스트 케이스: (파일명, 삽입 텍스트)
        let test_cases = vec![
            ("save_test_korean.hwp", "가나다라마바사아"),
            ("save_test_english.hwp", "Hello World"),
            ("save_test_mixed.hwp", "안녕 Hello 123 !@#"),
        ];

        for (filename, text) in &test_cases {
            eprintln!("\n{}", "=".repeat(60));
            eprintln!("  테스트: {} → '{}'", filename, text);
            eprintln!("{}", "=".repeat(60));

            let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

            // 텍스트 삽입 (첫 구역, 첫 문단, 캐럿 위치 0)
            let result = doc.insert_text_native(0, 0, 0, text);
            assert!(result.is_ok(), "텍스트 삽입 실패: {:?}", result.err());

            // 삽입 후 문단 상태 확인
            let para = &doc.document.sections[0].paragraphs[0];
            eprintln!("  삽입 후: text='{}' char_count={}", para.text, para.char_count);
            eprintln!("  char_offsets: {:?}", &para.char_offsets);
            eprintln!("  char_shapes: {:?}", para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>());
            for (i, ls) in para.line_segs.iter().enumerate() {
                eprintln!("  LineSeg[{}]: text_start={} vpos={} lh={} th={} bd={} ls={} cs={} sw={} tag=0x{:08x}",
                    i, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                    ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
            }

            // HWP 저장
            let saved = doc.export_hwp_native();
            assert!(saved.is_ok(), "HWP 저장 실패: {:?}", saved.err());
            let saved_data = saved.unwrap();

            // 파일 출력
            let _ = std::fs::create_dir_all("output");
            let out_path = format!("output/{}", filename);
            std::fs::write(&out_path, &saved_data).unwrap();
            eprintln!("  저장: {} ({} bytes)", out_path, saved_data.len());

            // 재파싱 검증
            let doc2 = HwpDocument::from_bytes(&saved_data);
            assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
            let doc2 = doc2.unwrap();
            let para2 = &doc2.document.sections[0].paragraphs[0];
            eprintln!("  재파싱: text='{}' char_count={}", para2.text, para2.char_count);
            assert!(para2.text.contains(text), "재파싱 텍스트 불일치: expected '{}', got '{}'", text, para2.text);

            // 캐럿 위치 검증
            let caret = &doc2.document.doc_properties;
            eprintln!("  캐럿: list_id={} para_id={} char_pos={}", caret.caret_list_id, caret.caret_para_id, caret.caret_char_pos);
            // 삽입 후 캐럿은 텍스트 마지막 글자 뒤여야 함
            let expected_caret_pos = 16u32 + text.chars().map(|c| if (c as u32) > 0xFFFF { 2u32 } else { 1u32 }).sum::<u32>();
            assert_eq!(caret.caret_char_pos, expected_caret_pos,
                "캐럿 위치 불일치: expected {} got {}", expected_caret_pos, caret.caret_char_pos);

            // BodyText 레코드 비교 (원본 vs 저장)
            let orig_doc = crate::parser::parse_hwp(&orig_data).unwrap();
            let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
            let orig_bt = orig_cfb.read_body_text_section(0, orig_doc.header.compressed, false).unwrap();
            let orig_recs = Record::read_all(&orig_bt).unwrap();

            let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
            let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
            let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
            let saved_recs = Record::read_all(&saved_bt).unwrap();

            eprintln!("\n  --- 레코드 비교 (원본: {} / 저장: {}) ---", orig_recs.len(), saved_recs.len());
            let tag_name = |id: u16| -> &str {
                match id {
                    66 => "PARA_HEADER",
                    67 => "PARA_TEXT",
                    68 => "PARA_CHAR_SHAPE",
                    69 => "PARA_LINE_SEG",
                    70 => "CTRL_HEADER",
                    71 => "LIST_HEADER",
                    _ => "OTHER",
                }
            };

            let max = orig_recs.len().max(saved_recs.len());
            for i in 0..max {
                let o = orig_recs.get(i);
                let s = saved_recs.get(i);
                match (o, s) {
                    (Some(or), Some(sr)) => {
                        let same = or.tag_id == sr.tag_id && or.level == sr.level && or.data == sr.data;
                        let status = if same { "OK  " } else { "DIFF" };
                        eprintln!("  [{}] {} tag={:3}({}) level={}/{} size={}/{}",
                            i, status, or.tag_id, tag_name(or.tag_id),
                            or.level, sr.level, or.data.len(), sr.data.len());
                        if !same {
                            let show = or.data.len().min(sr.data.len()).min(48);
                            let orig_hex: String = or.data[..show].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                            let save_hex: String = sr.data[..show].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                            eprintln!("         ORIG: {}", orig_hex);
                            eprintln!("         SAVE: {}", save_hex);
                        }
                    }
                    (Some(or), None) => eprintln!("  [{}] MISSING tag={}({})", i, or.tag_id, tag_name(or.tag_id)),
                    (None, Some(sr)) => eprintln!("  [{}] EXTRA   tag={}({})", i, sr.tag_id, tag_name(sr.tag_id)),
                    _ => {}
                }
            }
        }
        eprintln!("\n=== 단계 2 텍스트 저장 검증 완료 ===");
    }

    #[test]
    fn test_analyze_reference_table() {
        // 참조 파일 분석: HWP 프로그램으로 표 1개만 삽입한 파일
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::cfb_reader::LenientCfbReader;

        let path = "output/1by1-table.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();

        // 표준 cfb 크레이트로 열기 시도, 실패하면 lenient 리더 사용
        let doc = match HwpDocument::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  표준 파서 실패 ({}), LenientCfbReader로 분석합니다.", e);
                // LenientCfbReader로 직접 스트림 추출 후 분석
                let lcfb = LenientCfbReader::open(&data).unwrap();

                eprintln!("\n  [LenientCFB 엔트리 목록]");
                for (name, start, size, otype) in lcfb.list_entries() {
                    let tname = match otype { 1 => "storage", 2 => "stream", 5 => "root", _ => "?" };
                    eprintln!("  {:20} start={:5} size={:8} type={}", name, start, size, tname);
                }

                // FileHeader 읽기
                let fh = lcfb.read_stream("FileHeader").unwrap();
                let compressed = fh.len() >= 37 && (fh[36] & 0x01) != 0;
                eprintln!("\n  FileHeader: {} bytes, compressed={}", fh.len(), compressed);

                // DocInfo 읽기 & 파싱
                let di_data = lcfb.read_doc_info(compressed).unwrap();
                let di_recs = Record::read_all(&di_data).unwrap();
                eprintln!("  DocInfo: {} bytes → {} 레코드", di_data.len(), di_recs.len());

                // DocProperties (첫 번째 레코드)
                if let Some(dp_rec) = di_recs.first() {
                    if dp_rec.tag_id == tags::HWPTAG_DOCUMENT_PROPERTIES && dp_rec.data.len() >= 26 {
                        let d = &dp_rec.data;
                        let caret_list_id = u32::from_le_bytes([d[14], d[15], d[16], d[17]]);
                        let caret_para_id = u32::from_le_bytes([d[18], d[19], d[20], d[21]]);
                        let caret_char_pos = u32::from_le_bytes([d[22], d[23], d[24], d[25]]);
                        eprintln!("\n  [캐럿 위치 (raw)]");
                        eprintln!("  caret_list_id:  {}", caret_list_id);
                        eprintln!("  caret_para_id:  {}", caret_para_id);
                        eprintln!("  caret_char_pos: {}", caret_char_pos);
                    }
                }

                // ID_MAPPINGS (두 번째 레코드)
                if di_recs.len() > 1 && di_recs[1].tag_id == tags::HWPTAG_ID_MAPPINGS {
                    let d = &di_recs[1].data;
                    if d.len() >= 72 {
                        eprintln!("\n  [ID_MAPPINGS]");
                        let labels = ["bin_data", "font_kr", "font_en", "font_cn", "font_jp",
                            "font_etc", "font_sym", "font_usr", "border_fill", "char_shape",
                            "tab_def", "numbering", "bullet", "para_shape", "style",
                            "memo_shape", "trackchange", "trackchange_author"];
                        for (i, label) in labels.iter().enumerate() {
                            let off = i * 4;
                            let val = u32::from_le_bytes([d[off], d[off+1], d[off+2], d[off+3]]);
                            if val > 0 {
                                eprintln!("  {:20}: {}", label, val);
                            }
                        }
                    }
                }

                // BorderFill 레코드 덤프
                eprintln!("\n  [DocInfo BORDER_FILL 레코드]");
                for (i, r) in di_recs.iter().enumerate() {
                    if r.tag_id == tags::HWPTAG_BORDER_FILL {
                        eprintln!("  [{:2}] BORDER_FILL size={} data: {:02x?}",
                            i, r.data.len(), &r.data[..r.data.len().min(60)]);
                    }
                }

                // BodyText/Section0 읽기
                let bt_data = lcfb.read_body_text_section(0, compressed).unwrap();
                let bt_recs = Record::read_all(&bt_data).unwrap();

                eprintln!("\n  [BodyText 레코드 덤프] ({} 개)", bt_recs.len());
                for (i, r) in bt_recs.iter().enumerate() {
                    let tname = tags::tag_name(r.tag_id);
                    let mut extra = String::new();
                    if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                        let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                        extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
                    }
                    eprintln!("  [{:2}] tag={:3}({:22}) level={} size={}{}",
                        i, r.tag_id, tname, r.level, r.data.len(), extra);
                    // 주요 레코드 데이터 덤프
                    if matches!(r.tag_id,
                        71 | 72 | 77 | // CTRL_HEADER, LIST_HEADER, TABLE
                        66 | 67 | 68 | 69 // PARA_HEADER, PARA_TEXT, PARA_CHAR_SHAPE, PARA_LINE_SEG
                    ) {
                        let show = r.data.len().min(80);
                        eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
                    }
                }

                // empty.hwp와 비교
                let empty_path = "template/empty.hwp";
                if std::path::Path::new(empty_path).exists() {
                    let empty_data = std::fs::read(empty_path).unwrap();
                    let empty_parsed = crate::parser::parse_hwp(&empty_data).unwrap();
                    let mut empty_cfb = crate::parser::cfb_reader::CfbReader::open(&empty_data).unwrap();
                    let empty_bt = empty_cfb.read_body_text_section(0, empty_parsed.header.compressed, false).unwrap();
                    let empty_recs = Record::read_all(&empty_bt).unwrap();
                    eprintln!("\n  [비교] empty.hwp={} 개, roundtrip.hwp={} 개 → 추가={} 개",
                        empty_recs.len(), bt_recs.len(), bt_recs.len() as i32 - empty_recs.len() as i32);
                }

                eprintln!("\n=== 참조 파일 분석 완료 (LenientCfbReader) ===");
                return;
            }
        };
        let doc = doc;

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  참조 파일 분석: {}", path);
        eprintln!("{}", "=".repeat(60));

        // 1. 캐럿 위치 정보
        let dp = &doc.document.doc_properties;
        eprintln!("\n  [캐럿 위치]");
        eprintln!("  caret_list_id:  {}", dp.caret_list_id);
        eprintln!("  caret_para_id:  {}", dp.caret_para_id);
        eprintln!("  caret_char_pos: {}", dp.caret_char_pos);

        // 2. 섹션/문단 구조
        for (si, sec) in doc.document.sections.iter().enumerate() {
            eprintln!("\n  [섹션 {}] 문단 수: {}", si, sec.paragraphs.len());
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                eprintln!("  문단[{}]: text='{}' char_count={} controls={} char_offsets={:?}",
                    pi, para.text, para.char_count, para.controls.len(), para.char_offsets);
                eprintln!("    control_mask=0x{:08X} para_shape_id={} style_id={}",
                    para.control_mask, para.para_shape_id, para.style_id);
                eprintln!("    char_shapes: {:?}", para.char_shapes);
                eprintln!("    line_segs: {:?}", para.line_segs);
                eprintln!("    raw_header_extra({} bytes): {:02x?}",
                    para.raw_header_extra.len(),
                    &para.raw_header_extra[..para.raw_header_extra.len().min(20)]);

                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        crate::model::control::Control::SectionDef(_) =>
                            eprintln!("    ctrl[{}]: SectionDef", ci),
                        crate::model::control::Control::ColumnDef(_) =>
                            eprintln!("    ctrl[{}]: ColumnDef", ci),
                        crate::model::control::Control::Table(t) => {
                            eprintln!("    ctrl[{}]: Table {}x{} cells={} attr=0x{:08X}",
                                ci, t.row_count, t.col_count, t.cells.len(), t.attr);
                            eprintln!("      raw_ctrl_data({} bytes): {:02x?}",
                                t.raw_ctrl_data.len(), &t.raw_ctrl_data[..t.raw_ctrl_data.len().min(40)]);
                            eprintln!("      raw_table_record_attr=0x{:08X}", t.raw_table_record_attr);
                            eprintln!("      raw_table_record_extra({} bytes): {:02x?}",
                                t.raw_table_record_extra.len(), &t.raw_table_record_extra[..t.raw_table_record_extra.len().min(20)]);
                            eprintln!("      padding: l={} r={} t={} b={}", t.padding.left, t.padding.right, t.padding.top, t.padding.bottom);
                            eprintln!("      cell_spacing={} border_fill_id={} row_sizes={:?}",
                                t.cell_spacing, t.border_fill_id, t.row_sizes);
                            for (celli, cell) in t.cells.iter().enumerate() {
                                eprintln!("      cell[{}]: col={} row={} span={}x{} w={} h={} bfid={}",
                                    celli, cell.col, cell.row, cell.col_span, cell.row_span,
                                    cell.width, cell.height, cell.border_fill_id);
                                eprintln!("        padding: l={} r={} t={} b={}",
                                    cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom);
                                eprintln!("        list_header_width_ref={} raw_list_extra({} bytes): {:02x?}",
                                    cell.list_header_width_ref, cell.raw_list_extra.len(),
                                    &cell.raw_list_extra[..cell.raw_list_extra.len().min(20)]);
                                for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                    eprintln!("        para[{}]: text='{}' cc={} cs={:?} ls={:?}",
                                        cpi, cp.text, cp.char_count, cp.char_shapes, cp.line_segs);
                                    eprintln!("          control_mask=0x{:08X} raw_header_extra({} bytes): {:02x?}",
                                        cp.control_mask, cp.raw_header_extra.len(),
                                        &cp.raw_header_extra[..cp.raw_header_extra.len().min(20)]);
                                }
                            }
                        },
                        _ => eprintln!("    ctrl[{}]: {:?}", ci, std::mem::discriminant(ctrl)),
                    }
                }
            }
        }

        // 3. BodyText 레코드 덤프
        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&data).unwrap();
        let parsed = crate::parser::parse_hwp(&data).unwrap();
        let bt = cfb.read_body_text_section(0, parsed.header.compressed, false).unwrap();
        let recs = Record::read_all(&bt).unwrap();

        eprintln!("\n  [BodyText 레코드 덤프] ({} 개)", recs.len());
        for (i, r) in recs.iter().enumerate() {
            let tname = tags::tag_name(r.tag_id);
            let mut extra = String::new();
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
            }
            eprintln!("  [{:2}] tag={:3}({:22}) level={} size={}{}",
                i, r.tag_id, tname, r.level, r.data.len(), extra);
            // CTRL_HEADER, TABLE, LIST_HEADER 데이터 덤프
            if matches!(r.tag_id, 71 | 72 | 77) { // CTRL_HEADER, LIST_HEADER, TABLE
                let show = r.data.len().min(60);
                eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
            }
        }

        // 4. 원본 empty.hwp 레코드 수 비교
        let empty_path = "template/empty.hwp";
        if std::path::Path::new(empty_path).exists() {
            let empty_data = std::fs::read(empty_path).unwrap();
            let empty_parsed = crate::parser::parse_hwp(&empty_data).unwrap();
            let mut empty_cfb = crate::parser::cfb_reader::CfbReader::open(&empty_data).unwrap();
            let empty_bt = empty_cfb.read_body_text_section(0, empty_parsed.header.compressed, false).unwrap();
            let empty_recs = Record::read_all(&empty_bt).unwrap();
            eprintln!("\n  [비교] empty.hwp={} 개, roundtrip.hwp={} 개 → 차이={} 개",
                empty_recs.len(), recs.len(), recs.len() as i32 - empty_recs.len() as i32);
        }

        // 5. DocInfo 분석
        eprintln!("\n  [DocInfo]");
        eprintln!("  bin_data_count: {}", doc.document.doc_info.bin_data_list.len());
        eprintln!("  border_fill_count: {}", doc.document.doc_info.border_fills.len());
        eprintln!("  char_shape_count: {}", doc.document.doc_info.char_shapes.len());
        eprintln!("  para_shape_count: {}", doc.document.doc_info.para_shapes.len());

        // 6. BorderFill 상세 분석
        eprintln!("\n  [BorderFill 상세]");
        for (bi, bf) in doc.document.doc_info.border_fills.iter().enumerate() {
            eprintln!("  bf[{}]: borders=[{:?}, {:?}, {:?}, {:?}] diag={:?}",
                bi, bf.borders[0], bf.borders[1], bf.borders[2], bf.borders[3], bf.diagonal);
            eprintln!("    attr={} fill={:?}", bf.attr, bf.fill);
            if let Some(ref raw) = bf.raw_data {
                let show = raw.len().min(60);
                eprintln!("    raw_data({} bytes): {:02x?}", raw.len(), &raw[..show]);
            }
        }

        eprintln!("\n=== 참조 파일 분석 완료 ===");
    }

    #[test]
    fn test_save_table_1x1() {
        // 단계 3: 빈 HWP에 1×1 표 삽입 → 저장
        // 참조: output/1by1-table.hwp (HWP 프로그램으로 생성한 1x1 표)
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        use crate::model::Padding;
        use crate::parser::record::Record;

        let path = "template/empty.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let orig_data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  단계 3: 1×1 표 삽입 → 저장 (참조파일 기반)");
        eprintln!("{}", "=".repeat(60));

        // 참조 파일의 값을 사용하여 표 생성
        // cell_width=41954, cell_height=282 (참조 파일 기준)
        let table_width: u32 = 41954;  // 참조 파일과 동일
        let table_height: u32 = 1282;  // 참조 파일과 동일
        let cell_width: u32 = 41954;
        let cell_height: u32 = 282;

        // 셀 내부 문단: 빈 문단 (CR만, char_count=1, MSB set)
        let cell_seg_width = 40932; // 참조: cell_width - 패딩(510+510) - 2
        let cell_para = Paragraph {
            text: String::new(),
            char_count: 1,
            char_count_msb: true,    // 참조: 0x80000001
            control_mask: 0,
            para_shape_id: 0,        // empty.hwp의 기존 para_shape 사용
            style_id: 0,
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,    // empty.hwp의 기존 char_shape 사용
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                vertical_pos: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                column_start: 0,
                segment_width: cell_seg_width,
                tag: 0x00060000,
            }],
            has_para_text: false,    // 빈 문단: PARA_TEXT 없음
            raw_header_extra: vec![0x01,0x00,0x00,0x00, 0x01,0x00, 0x00,0x00,0x00,0x00],
            ..Default::default()
        };

        let cell = Cell {
            col: 0, row: 0, col_span: 1, row_span: 1,
            width: cell_width,
            height: cell_height,
            border_fill_id: 1,
            padding: Padding { left: 510, right: 510, top: 141, bottom: 141 }, // 참조값
            list_header_width_ref: 0,
            // raw_list_extra: 참조파일의 13바이트 (width + zeros)
            raw_list_extra: {
                let mut v = Vec::new();
                v.extend_from_slice(&cell_width.to_le_bytes()); // [e2,a3,00,00]
                v.extend_from_slice(&[0u8; 9]); // zeros
                v
            },
            paragraphs: vec![cell_para],
            ..Default::default()
        };

        // CommonObjAttr 바이너리 생성 (참조 파일의 raw_ctrl_data 38바이트)
        let raw_ctrl_data = {
            let mut v = Vec::new();
            v.extend_from_slice(&0u32.to_le_bytes());  // y_offset = 0
            v.extend_from_slice(&0u32.to_le_bytes());  // x_offset = 0
            v.extend_from_slice(&table_width.to_le_bytes());  // width
            v.extend_from_slice(&table_height.to_le_bytes()); // height
            v.extend_from_slice(&1u32.to_le_bytes());  // z_order = 1
            v.extend_from_slice(&283u16.to_le_bytes()); // margin_left
            v.extend_from_slice(&283u16.to_le_bytes()); // margin_right
            v.extend_from_slice(&283u16.to_le_bytes()); // margin_top
            v.extend_from_slice(&283u16.to_le_bytes()); // margin_bottom
            v.extend_from_slice(&0x7C1E9738u32.to_le_bytes()); // instance_id
            v.extend_from_slice(&0u32.to_le_bytes());  // unknown1
            v.extend_from_slice(&0u16.to_le_bytes());  // unknown2
            v
        };

        // DocInfo에 실선 테두리 BorderFill 추가 (참조: bf[0])
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};
        let solid_border = BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 };
        let new_bf = BorderFill {
            raw_data: None,
            attr: 0,
            borders: [solid_border, solid_border, solid_border, solid_border],
            diagonal: DiagonalLine { diagonal_type: 1, width: 0, color: 0 },
            fill: Fill::default(),
        };
        doc.document.doc_info.border_fills.push(new_bf);
        let table_bf_id = doc.document.doc_info.border_fills.len() as u16; // 1-based ID

        let table = Table {
            attr: 0x082A2210, // 참조: CommonObjAttr flags
            row_count: 1,
            col_count: 1,
            cell_spacing: 0,
            padding: Padding { left: 510, right: 510, top: 141, bottom: 141 }, // 참조값
            row_sizes: vec![1],
            border_fill_id: table_bf_id,
            cells: {
                // cell의 border_fill_id도 갱신
                let mut c = cell;
                c.border_fill_id = table_bf_id;
                vec![c]
            },
            raw_ctrl_data,
            raw_table_record_attr: 6,  // 참조: attr=6
            raw_table_record_extra: vec![0x00, 0x00], // 참조: 2바이트
            ..Default::default()
        };
        eprintln!("  DocInfo: border_fill_count={}, table_bf_id={}", doc.document.doc_info.border_fills.len(), table_bf_id);

        // 첫 번째 문단에 Table 컨트롤 추가
        {
            let para = &mut doc.document.sections[0].paragraphs[0];
            para.controls.push(Control::Table(Box::new(table)));
            para.ctrl_data_records.push(None);
            para.char_count += 8; // 표 제어문자 8 code units
            para.control_mask = 0x00000804; // 참조: 표가 있는 문단의 control_mask

            // 표가 있는 문단의 segment_width는 0 (참조 파일)
            if let Some(ls) = para.line_segs.first_mut() {
                ls.segment_width = 0;
            }
        }

        // 두 번째 빈 문단 추가 (HWP는 표 삽입 시 아래에 빈 문단을 자동 추가)
        let empty_para = Paragraph {
            text: String::new(),
            char_count: 1,           // CR만
            char_count_msb: true,    // 참조: 0x80000001
            control_mask: 0,
            para_shape_id: 0,        // empty.hwp의 기존 para_shape 사용
            style_id: 0,
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                vertical_pos: 1848,  // 참조: 표 아래 위치
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                column_start: 0,
                segment_width: 42520, // 참조: 편집 영역 전체 너비
                tag: 0x00060000,
            }],
            has_para_text: false,
            raw_header_extra: vec![0x01,0x00,0x00,0x00, 0x01,0x00, 0x00,0x00,0x00,0x00],
            ..Default::default()
        };
        doc.document.sections[0].paragraphs.push(empty_para);

        // raw_stream 무효화 (재직렬화 유도)
        doc.document.sections[0].raw_stream = None;

        // 캐럿 위치: 두 번째 문단(표 아래 빈 줄) 시작
        doc.document.doc_properties.caret_list_id = 1;  // 문단 인덱스 1
        doc.document.doc_properties.caret_para_id = 0;
        doc.document.doc_properties.caret_char_pos = 0;
        doc.document.doc_info.raw_stream = None;
        doc.document.doc_properties.raw_data = None;

        let para = &doc.document.sections[0].paragraphs[0];
        eprintln!("  문단[0]: text='{}' char_count={} controls={} seg_width={}",
            para.text, para.char_count, para.controls.len(),
            para.line_segs.first().map(|ls| ls.segment_width).unwrap_or(-1));
        let para1 = &doc.document.sections[0].paragraphs[1];
        eprintln!("  문단[1]: text='{}' char_count={} vpos={}",
            para1.text, para1.char_count,
            para1.line_segs.first().map(|ls| ls.vertical_pos).unwrap_or(-1));

        // HWP 저장
        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "HWP 저장 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/save_test_table_1x1.hwp", &saved_data).unwrap();
        eprintln!("  저장: output/save_test_table_1x1.hwp ({} bytes)", saved_data.len());

        // 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();

        // 표 컨트롤 존재 검증
        let para2 = &doc2.document.sections[0].paragraphs[0];
        eprintln!("  재파싱: text='{}' char_count={} controls={}", para2.text, para2.char_count, para2.controls.len());
        let table_found = para2.controls.iter().any(|c| matches!(c, Control::Table(_)));
        assert!(table_found, "재파싱된 문서에 표 컨트롤이 없음");

        // 표 내용 검증
        if let Some(Control::Table(t)) = para2.controls.iter().find(|c| matches!(c, Control::Table(_))) {
            eprintln!("  표: {}×{} cells={}", t.row_count, t.col_count, t.cells.len());
            for (ci, cell) in t.cells.iter().enumerate() {
                eprintln!("  셀[{}]: col={} row={} w={} h={} text='{}'",
                    ci, cell.col, cell.row, cell.width, cell.height,
                    cell.paragraphs.first().map(|p| p.text.as_str()).unwrap_or(""));
            }
            assert_eq!(t.row_count, 1);
            assert_eq!(t.col_count, 1);
            assert_eq!(t.cells.len(), 1);
            assert_eq!(t.cells[0].paragraphs.len(), 1);
            // 빈 셀 확인 (참조 파일 기반)
            assert_eq!(t.cells[0].paragraphs[0].char_count, 1); // CR만
        }

        // 두 번째 문단 (표 아래 빈 줄) 검증
        assert!(doc2.document.sections[0].paragraphs.len() >= 2,
            "표 아래 빈 문단이 없음");
        let para_below = &doc2.document.sections[0].paragraphs[1];
        eprintln!("  문단[1]: char_count={} controls={}", para_below.char_count, para_below.controls.len());

        // 저장 레코드 덤프 (참조 파일과 비교)
        let saved_doc = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_doc.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        eprintln!("\n  --- 저장 레코드 덤프 ({} 개) ---", saved_recs.len());
        use crate::parser::tags as t;
        for (i, r) in saved_recs.iter().enumerate() {
            let tname = t::tag_name(r.tag_id);
            let mut extra = String::new();
            if r.tag_id == t::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                extra = format!(" ctrl='{}'", t::ctrl_name(cid));
            }
            eprintln!("  [{:2}] tag={:3}({:22}) level={} size={}{}",
                i, r.tag_id, tname, r.level, r.data.len(), extra);
        }

        // 참조 파일과 레코드 수 비교
        eprintln!("\n  [참조 비교] 참조=21개, 저장={}개", saved_recs.len());

        eprintln!("\n=== 단계 3 표 저장 검증 완료 ===");
    }

    /// 단계 4-1: HWP 프로그램으로 생성한 이미지 참조 파일 분석
    /// output/pic-01-as-text.hwp: 빈 문서 → 이미지 1개 → 글자처리로 삽입
    #[test]
    fn test_analyze_reference_picture() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::cfb_reader::LenientCfbReader;
        use crate::model::control::Control;

        let path = "output/pic-01-as-text.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  이미지 참조 파일 분석: {}", path);
        eprintln!("  파일 크기: {} bytes", data.len());
        eprintln!("{}", "=".repeat(60));

        // 표준 파서 시도, 실패 시 LenientCfbReader
        let doc = match HwpDocument::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  표준 파서 실패 ({}), LenientCfbReader로 분석합니다.", e);
                let lcfb = LenientCfbReader::open(&data).unwrap();

                eprintln!("\n  [LenientCFB 엔트리 목록]");
                for (name, start, size, otype) in lcfb.list_entries() {
                    let tname = match otype { 1 => "storage", 2 => "stream", 5 => "root", _ => "?" };
                    eprintln!("  {:30} start={:5} size={:8} type={}", name, start, size, tname);
                }

                // FileHeader
                let fh = lcfb.read_stream("FileHeader").unwrap();
                let compressed = fh.len() >= 37 && (fh[36] & 0x01) != 0;
                eprintln!("\n  FileHeader: {} bytes, compressed={}", fh.len(), compressed);

                // DocInfo
                let di_data = lcfb.read_doc_info(compressed).unwrap();
                let di_recs = Record::read_all(&di_data).unwrap();
                eprintln!("  DocInfo: {} bytes → {} 레코드", di_data.len(), di_recs.len());

                // DocProperties (캐럿 위치)
                if let Some(dp_rec) = di_recs.first() {
                    if dp_rec.tag_id == tags::HWPTAG_DOCUMENT_PROPERTIES && dp_rec.data.len() >= 26 {
                        let d = &dp_rec.data;
                        let caret_list_id = u32::from_le_bytes([d[14], d[15], d[16], d[17]]);
                        let caret_para_id = u32::from_le_bytes([d[18], d[19], d[20], d[21]]);
                        let caret_char_pos = u32::from_le_bytes([d[22], d[23], d[24], d[25]]);
                        eprintln!("\n  [캐럿 위치 (raw)]");
                        eprintln!("  caret_list_id:  {}", caret_list_id);
                        eprintln!("  caret_para_id:  {}", caret_para_id);
                        eprintln!("  caret_char_pos: {}", caret_char_pos);
                    }
                }

                // ID_MAPPINGS
                if di_recs.len() > 1 && di_recs[1].tag_id == tags::HWPTAG_ID_MAPPINGS {
                    let d = &di_recs[1].data;
                    if d.len() >= 72 {
                        eprintln!("\n  [ID_MAPPINGS]");
                        let labels = ["bin_data", "font_kr", "font_en", "font_cn", "font_jp",
                            "font_etc", "font_sym", "font_usr", "border_fill", "char_shape",
                            "tab_def", "numbering", "bullet", "para_shape", "style",
                            "memo_shape", "trackchange", "trackchange_author"];
                        for (i, label) in labels.iter().enumerate() {
                            let off = i * 4;
                            let val = u32::from_le_bytes([d[off], d[off+1], d[off+2], d[off+3]]);
                            if val > 0 {
                                eprintln!("  {:20}: {}", label, val);
                            }
                        }
                    }
                }

                // BIN_DATA 레코드 덤프
                eprintln!("\n  [DocInfo BIN_DATA 레코드]");
                for (i, r) in di_recs.iter().enumerate() {
                    if r.tag_id == tags::HWPTAG_BIN_DATA {
                        eprintln!("  [{:2}] BIN_DATA size={} data: {:02x?}",
                            i, r.data.len(), &r.data[..r.data.len().min(60)]);
                    }
                }

                // BodyText
                let bt_data = lcfb.read_body_text_section(0, compressed).unwrap();
                let bt_recs = Record::read_all(&bt_data).unwrap();

                eprintln!("\n  [BodyText 레코드 덤프] ({} 개)", bt_recs.len());
                for (i, r) in bt_recs.iter().enumerate() {
                    let tname = tags::tag_name(r.tag_id);
                    let mut extra = String::new();
                    if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                        let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                        extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
                    }
                    eprintln!("  [{:2}] tag={:3}({:25}) level={} size={}{}",
                        i, r.tag_id, tname, r.level, r.data.len(), extra);
                    // 주요 레코드 데이터 덤프
                    if matches!(r.tag_id,
                        66 | 67 | 68 | 69 | 71 | // PARA_HEADER, TEXT, CHAR_SHAPE, LINE_SEG, CTRL_HEADER
                        76 | 79 // SHAPE_COMPONENT, SHAPE_COMPONENT_PICTURE
                    ) {
                        let show = r.data.len().min(100);
                        eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
                        if r.data.len() > 100 {
                            eprintln!("        total: {} bytes", r.data.len());
                        }
                    }
                }

                // BinData 스트림 확인
                eprintln!("\n  [BinData 스트림]");
                for (name, _start, size, otype) in lcfb.list_entries() {
                    if *otype == 2 && name.contains("BIN") {
                        eprintln!("  {} size={}", name, size);
                        if let Ok(stream) = lcfb.read_stream(&name) {
                            let sig_show = stream.len().min(16);
                            eprintln!("    sig[..{}]: {:02x?}", sig_show, &stream[..sig_show]);
                        }
                    }
                }

                // empty.hwp 비교
                let empty_path = "template/empty.hwp";
                if std::path::Path::new(empty_path).exists() {
                    let empty_data = std::fs::read(empty_path).unwrap();
                    let empty_parsed = crate::parser::parse_hwp(&empty_data).unwrap();
                    let mut empty_cfb = crate::parser::cfb_reader::CfbReader::open(&empty_data).unwrap();
                    let empty_bt = empty_cfb.read_body_text_section(0, empty_parsed.header.compressed, false).unwrap();
                    let empty_recs = Record::read_all(&empty_bt).unwrap();
                    eprintln!("\n  [비교] empty.hwp={} 개, pic-01.hwp={} 개 → 차이={} 개",
                        empty_recs.len(), bt_recs.len(), bt_recs.len() as i32 - empty_recs.len() as i32);
                }

                eprintln!("\n=== 참조 파일 분석 완료 (LenientCfbReader) ===");
                return;
            }
        };

        // === 표준 파서 성공 경로 ===

        // 1. 캐럿 위치
        let dp = &doc.document.doc_properties;
        eprintln!("\n  [캐럿 위치]");
        eprintln!("  caret_list_id:  {}", dp.caret_list_id);
        eprintln!("  caret_para_id:  {}", dp.caret_para_id);
        eprintln!("  caret_char_pos: {}", dp.caret_char_pos);

        // 2. BinData 목록
        eprintln!("\n  [BinData 목록] ({} 개)", doc.document.doc_info.bin_data_list.len());
        for (i, bd) in doc.document.doc_info.bin_data_list.iter().enumerate() {
            eprintln!("  [{}] attr=0x{:04X} type={:?} storage_id={} ext={:?} compression={:?} status={:?}",
                i, bd.attr, bd.data_type, bd.storage_id, bd.extension, bd.compression, bd.status);
            if let Some(ref raw) = bd.raw_data {
                let show = raw.len().min(60);
                eprintln!("       raw_data({} bytes): {:02x?}", raw.len(), &raw[..show]);
            }
        }

        // 3. BinDataContent 목록
        eprintln!("\n  [BinDataContent 목록] ({} 개)", doc.document.bin_data_content.len());
        for (i, bc) in doc.document.bin_data_content.iter().enumerate() {
            eprintln!("  [{}] id={} ext='{}' data_size={} bytes",
                i, bc.id, bc.extension, bc.data.len());
            if bc.data.len() >= 8 {
                let sig = &bc.data[..8];
                let format = if sig[0..2] == [0xFF, 0xD8] { "JPEG" }
                    else if sig[0..4] == [0x89, 0x50, 0x4E, 0x47] { "PNG" }
                    else if sig[0..2] == [0x42, 0x4D] { "BMP" }
                    else if sig[0..4] == [0x47, 0x49, 0x46, 0x38] { "GIF" }
                    else { "Unknown" };
                eprintln!("       시그니처: {:02x?} → {}", &sig[..4], format);
            }
        }

        // 4. 섹션/문단 구조 상세
        for (si, sec) in doc.document.sections.iter().enumerate() {
            eprintln!("\n  [섹션 {}] 문단 수: {}", si, sec.paragraphs.len());
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                eprintln!("  문단[{}]: text='{}' char_count={} msb={} controls={} char_offsets={:?}",
                    pi, para.text, para.char_count, para.char_count_msb, para.controls.len(), para.char_offsets);
                eprintln!("    control_mask=0x{:08X} para_shape_id={} style_id={}",
                    para.control_mask, para.para_shape_id, para.style_id);
                eprintln!("    char_shapes: {:?}", para.char_shapes);
                eprintln!("    line_segs: {:?}", para.line_segs);
                eprintln!("    raw_header_extra({} bytes): {:02x?}",
                    para.raw_header_extra.len(),
                    &para.raw_header_extra[..para.raw_header_extra.len().min(30)]);

                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::SectionDef(_) => eprintln!("    ctrl[{}]: SectionDef", ci),
                        Control::ColumnDef(_) => eprintln!("    ctrl[{}]: ColumnDef", ci),
                        Control::Picture(pic) => {
                            eprintln!("    ctrl[{}]: Picture", ci);
                            eprintln!("      CommonObjAttr:");
                            eprintln!("        ctrl_id: 0x{:08X}", pic.common.ctrl_id);
                            eprintln!("        attr: 0x{:08X}", pic.common.attr);
                            eprintln!("        vertical_offset: {}", pic.common.vertical_offset);
                            eprintln!("        horizontal_offset: {}", pic.common.horizontal_offset);
                            eprintln!("        width: {}", pic.common.width);
                            eprintln!("        height: {}", pic.common.height);
                            eprintln!("        z_order: {}", pic.common.z_order);
                            eprintln!("        margin: L={} R={} T={} B={}",
                                pic.common.margin.left, pic.common.margin.right,
                                pic.common.margin.top, pic.common.margin.bottom);
                            eprintln!("        instance_id: 0x{:08X}", pic.common.instance_id);
                            eprintln!("        description: '{}'", pic.common.description);
                            eprintln!("        raw_extra({} bytes): {:02x?}",
                                pic.common.raw_extra.len(), &pic.common.raw_extra[..pic.common.raw_extra.len().min(40)]);
                            eprintln!("      ShapeComponentAttr:");
                            eprintln!("        ctrl_id: 0x{:08X}", pic.shape_attr.ctrl_id);
                            eprintln!("        is_two_ctrl_id: {}", pic.shape_attr.is_two_ctrl_id);
                            eprintln!("        offset: ({}, {})", pic.shape_attr.offset_x, pic.shape_attr.offset_y);
                            eprintln!("        group_level: {}", pic.shape_attr.group_level);
                            eprintln!("        local_file_version: {}", pic.shape_attr.local_file_version);
                            eprintln!("        original: {}×{}", pic.shape_attr.original_width, pic.shape_attr.original_height);
                            eprintln!("        current: {}×{}", pic.shape_attr.current_width, pic.shape_attr.current_height);
                            eprintln!("        flip: 0x{:08X}", pic.shape_attr.flip);
                            eprintln!("        rotation_angle: {}", pic.shape_attr.rotation_angle);
                            eprintln!("        raw_rendering({} bytes): {:02x?}",
                                pic.shape_attr.raw_rendering.len(),
                                &pic.shape_attr.raw_rendering[..pic.shape_attr.raw_rendering.len().min(80)]);
                            eprintln!("      PictureData:");
                            eprintln!("        border_color: 0x{:08X}", pic.border_color);
                            eprintln!("        border_width: {}", pic.border_width);
                            eprintln!("        border_x: {:?}", pic.border_x);
                            eprintln!("        border_y: {:?}", pic.border_y);
                            eprintln!("        crop: L={} T={} R={} B={}", pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom);
                            eprintln!("        padding: L={} R={} T={} B={}",
                                pic.padding.left, pic.padding.right, pic.padding.top, pic.padding.bottom);
                            eprintln!("        image_attr: brightness={} contrast={} effect={:?} bin_data_id={}",
                                pic.image_attr.brightness, pic.image_attr.contrast, pic.image_attr.effect, pic.image_attr.bin_data_id);
                            eprintln!("        border_opacity: {}", pic.border_opacity);
                            eprintln!("        instance_id: {}", pic.instance_id);
                            eprintln!("        raw_picture_extra({} bytes): {:02x?}",
                                pic.raw_picture_extra.len(), &pic.raw_picture_extra[..pic.raw_picture_extra.len().min(40)]);
                        },
                        _ => eprintln!("    ctrl[{}]: {:?}", ci, std::mem::discriminant(ctrl)),
                    }
                }
            }
        }

        // 5. BodyText 레코드 덤프
        let parsed_doc = crate::parser::parse_hwp(&data).unwrap();
        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&data).unwrap();
        let bt_data = cfb.read_body_text_section(0, parsed_doc.header.compressed, false).unwrap();
        let bt_recs = Record::read_all(&bt_data).unwrap();

        eprintln!("\n  [BodyText 레코드 덤프] ({} 개)", bt_recs.len());
        for (i, r) in bt_recs.iter().enumerate() {
            let tname = tags::tag_name(r.tag_id);
            let mut extra = String::new();
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
            }
            eprintln!("  [{:2}] tag={:3}({:25}) level={} size={}{}",
                i, r.tag_id, tname, r.level, r.data.len(), extra);
            // 주요 레코드 데이터 상세 덤프
            if matches!(r.tag_id,
                66 | 67 | 68 | 69 | 71 | // PARA_HEADER, TEXT, CHAR_SHAPE, LINE_SEG, CTRL_HEADER
                76 | 85 // SHAPE_COMPONENT, SHAPE_COMPONENT_PICTURE (tag 85)
            ) {
                let show = r.data.len().min(120);
                eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
                if r.data.len() > 120 {
                    eprintln!("        total: {} bytes", r.data.len());
                }
            }
        }

        // 6. empty.hwp 비교
        let empty_path = "template/empty.hwp";
        if std::path::Path::new(empty_path).exists() {
            let empty_data = std::fs::read(empty_path).unwrap();
            let empty_parsed = crate::parser::parse_hwp(&empty_data).unwrap();
            let mut empty_cfb = crate::parser::cfb_reader::CfbReader::open(&empty_data).unwrap();
            let empty_bt = empty_cfb.read_body_text_section(0, empty_parsed.header.compressed, false).unwrap();
            let empty_recs = Record::read_all(&empty_bt).unwrap();
            eprintln!("\n  [비교] empty.hwp={} 개, pic-01.hwp={} 개 → 차이={} 개",
                empty_recs.len(), bt_recs.len(), bt_recs.len() as i32 - empty_recs.len() as i32);
        }

        // 7. DocInfo 상세
        eprintln!("\n  [DocInfo]");
        eprintln!("  bin_data_count: {}", doc.document.doc_info.bin_data_list.len());
        eprintln!("  border_fill_count: {}", doc.document.doc_info.border_fills.len());
        eprintln!("  char_shape_count: {}", doc.document.doc_info.char_shapes.len());
        eprintln!("  para_shape_count: {}", doc.document.doc_info.para_shapes.len());

        // 8. CFB 스트림 목록
        let streams = cfb.list_streams();
        eprintln!("\n  [CFB 스트림 목록]");
        for s in &streams {
            eprintln!("  {}", s);
        }

        eprintln!("\n=== 이미지 참조 파일 분석 완료 ===");
    }

    /// 단계 4-2: 빈 HWP에 이미지 삽입 후 저장 검증
    /// 참조: output/pic-01-as-text.hwp (HWP 프로그램으로 생성, 3tigers.jpg 글자처리 삽입)
    #[test]
    fn test_save_picture() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::model::control::Control;
        use crate::model::bin_data::{BinData, BinDataType, BinDataCompression, BinDataStatus, BinDataContent};
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        eprintln!("\n=== 단계 4-2: 이미지 저장 검증 시작 ===");

        // 1. 참조 파일에서 Picture 구조 및 이미지 추출
        let ref_path = "output/pic-01-as-text.hwp";
        if !std::path::Path::new(ref_path).exists() {
            eprintln!("SKIP: {} 없음", ref_path);
            return;
        }
        let ref_data = std::fs::read(ref_path).unwrap();
        let ref_doc = HwpDocument::from_bytes(&ref_data).unwrap();

        // 참조 파일에서 Picture 컨트롤 추출
        let ref_pic = ref_doc.document.sections[0].paragraphs[0].controls.iter()
            .find_map(|c| if let Control::Picture(p) = c { Some(p) } else { None })
            .expect("참조 파일에 Picture 컨트롤 없음");

        let ref_bindata = &ref_doc.document.doc_info.bin_data_list[0];
        let ref_bincontent = &ref_doc.document.bin_data_content[0];

        let pic_width = ref_pic.common.width;
        let pic_height = ref_pic.common.height;
        eprintln!("  참조 Picture: {}×{} bin_data_id={} image={} bytes",
            pic_width, pic_height, ref_pic.image_attr.bin_data_id, ref_bincontent.data.len());
        eprintln!("  참조 캐럿: list_id={} para_id={} char_pos={}",
            ref_doc.document.doc_properties.caret_list_id,
            ref_doc.document.doc_properties.caret_para_id,
            ref_doc.document.doc_properties.caret_char_pos);

        // 2. empty.hwp 로드
        let empty_path = "template/empty.hwp";
        assert!(std::path::Path::new(empty_path).exists(), "template/empty.hwp 없음");
        let empty_data = std::fs::read(empty_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&empty_data).unwrap();

        // 3. DocInfo에 BinData 추가
        // 참조 파일: attr=0x0001 (Embedding), status=NotAccessed
        let bin_data_entry = BinData {
            attr: ref_bindata.attr,
            data_type: BinDataType::Embedding,
            compression: BinDataCompression::Default,
            status: BinDataStatus::NotAccessed, // 참조 파일과 동일
            storage_id: 1,
            extension: Some(ref_bincontent.extension.clone()),
            raw_data: None,
            ..Default::default()
        };
        doc.document.doc_info.bin_data_list.push(bin_data_entry);

        // BinDataContent 추가
        doc.document.bin_data_content.push(BinDataContent {
            id: 1,
            data: ref_bincontent.data.clone(),
            extension: ref_bincontent.extension.clone(),
        });

        // 4. Picture 컨트롤 구성 (참조 파일의 정확한 값 사용)
        let picture = crate::model::image::Picture {
            common: ref_pic.common.clone(),
            shape_attr: ref_pic.shape_attr.clone(),
            border_color: ref_pic.border_color,
            border_width: ref_pic.border_width,
            border_attr: ref_pic.border_attr.clone(),
            border_x: ref_pic.border_x,
            border_y: ref_pic.border_y,
            crop: ref_pic.crop.clone(),
            padding: ref_pic.padding.clone(),
            image_attr: ref_pic.image_attr.clone(),
            border_opacity: ref_pic.border_opacity,
            instance_id: ref_pic.instance_id,
            raw_picture_extra: ref_pic.raw_picture_extra.clone(),
            caption: None,
        };

        // 5. 문단 구성 (참조 파일: 단일 문단에 SectionDef + ColumnDef + Picture)
        let first_para = &doc.document.sections[0].paragraphs[0];
        let existing_controls: Vec<Control> = first_para.controls.clone();

        // 참조: char_count=25 (msb=true), control_mask=0x00000804
        // PARA_TEXT: secd(0~7) + cold(8~15) + gso(16~23) + CR(24) = 25 chars
        let mut new_controls = existing_controls;
        new_controls.push(Control::Picture(Box::new(picture)));

        // 참조 문단의 정확한 값 사용
        let ref_para = &ref_doc.document.sections[0].paragraphs[0];
        let pic_para = Paragraph {
            text: String::new(),
            char_count: 25, // secd(8) + cold(8) + gso(8) + CR(1) = 25
            char_count_msb: true, // 참조: msb=true
            control_mask: 0x00000804,
            para_shape_id: first_para.para_shape_id, // empty.hwp 기본값 사용
            style_id: first_para.style_id,
            raw_break_type: ref_para.raw_break_type, // 참조: 0x03
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: first_para.char_shapes.first()
                    .map(|cs| cs.char_shape_id).unwrap_or(0),
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                vertical_pos: 0,
                line_height: ref_para.line_segs[0].line_height, // 참조: 14775 (=이미지 높이)
                text_height: ref_para.line_segs[0].text_height,
                baseline_distance: ref_para.line_segs[0].baseline_distance,
                line_spacing: ref_para.line_segs[0].line_spacing,
                column_start: 0,
                segment_width: ref_para.line_segs[0].segment_width, // 참조: 42520
                tag: ref_para.line_segs[0].tag, // 참조: 0x00060000
            }],
            has_para_text: true,
            controls: new_controls,
            raw_header_extra: first_para.raw_header_extra.clone(),
            ..Default::default()
        };

        // 참조: 문단 1개만 (참조 파일에는 두 번째 문단이 없음)
        doc.document.sections[0].paragraphs = vec![pic_para];

        // 6. raw_stream 무효화 (재직렬화)
        doc.document.sections[0].raw_stream = None;
        doc.document.doc_info.raw_stream = None;
        doc.document.doc_properties.raw_data = None;

        // 캐럿 위치 (참조: list_id=0, para_id=0, char_pos=24)
        doc.document.doc_properties.caret_list_id = 0;
        doc.document.doc_properties.caret_para_id = 0;
        doc.document.doc_properties.caret_char_pos = 24;

        // 7. 저장
        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "HWP 저장 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/save_test_picture.hwp", &saved_data).unwrap();
        eprintln!("  저장: output/save_test_picture.hwp ({} bytes)", saved_data.len());

        // 8. 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();

        // Picture 컨트롤 존재 검증
        let para2 = &doc2.document.sections[0].paragraphs[0];
        eprintln!("  재파싱: char_count={} msb={} controls={}",
            para2.char_count, para2.char_count_msb, para2.controls.len());
        let pic_found = para2.controls.iter().any(|c| matches!(c, Control::Picture(_)));
        assert!(pic_found, "재파싱된 문서에 Picture 컨트롤이 없음");

        // Picture 속성 검증
        if let Some(Control::Picture(p)) = para2.controls.iter().find(|c| matches!(c, Control::Picture(_))) {
            eprintln!("  Picture: {}×{} bin_data_id={}",
                p.common.width, p.common.height, p.image_attr.bin_data_id);
            eprintln!("    border_x={:?} border_y={:?}", p.border_x, p.border_y);
            eprintln!("    crop: L={} T={} R={} B={}", p.crop.left, p.crop.top, p.crop.right, p.crop.bottom);
            eprintln!("    shape_attr: ctrl_id=0x{:08X} two={} orig={}×{} cur={}×{}",
                p.shape_attr.ctrl_id, p.shape_attr.is_two_ctrl_id,
                p.shape_attr.original_width, p.shape_attr.original_height,
                p.shape_attr.current_width, p.shape_attr.current_height);
            assert_eq!(p.image_attr.bin_data_id, 1);
            assert_eq!(p.common.width, pic_width);
            assert_eq!(p.common.height, pic_height);
        }

        // BinData 검증
        assert_eq!(doc2.document.doc_info.bin_data_list.len(), 1, "BinData 없음");
        assert_eq!(doc2.document.doc_info.bin_data_list[0].data_type, BinDataType::Embedding);
        assert_eq!(doc2.document.doc_info.bin_data_list[0].storage_id, 1);

        // BinDataContent 검증
        assert_eq!(doc2.document.bin_data_content.len(), 1, "BinDataContent 없음");
        assert_eq!(doc2.document.bin_data_content[0].data.len(), ref_bincontent.data.len(),
            "이미지 데이터 크기 불일치");

        // 캐럿 위치 검증
        eprintln!("  캐럿: list_id={} para_id={} char_pos={}",
            doc2.document.doc_properties.caret_list_id,
            doc2.document.doc_properties.caret_para_id,
            doc2.document.doc_properties.caret_char_pos);

        // 9. 저장 레코드 덤프 (참조 파일과 비교)
        let saved_parsed = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_parsed.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        // 참조 파일 레코드도 덤프
        let ref_parsed = crate::parser::parse_hwp(&ref_data).unwrap();
        let mut ref_cfb = crate::parser::cfb_reader::CfbReader::open(&ref_data).unwrap();
        let ref_bt = ref_cfb.read_body_text_section(0, ref_parsed.header.compressed, false).unwrap();
        let ref_recs = Record::read_all(&ref_bt).unwrap();

        eprintln!("\n  --- 레코드 비교 (참조={} 개, 저장={} 개) ---", ref_recs.len(), saved_recs.len());
        let max_recs = ref_recs.len().max(saved_recs.len());
        for i in 0..max_recs {
            let ref_info = if i < ref_recs.len() {
                let r = &ref_recs[i];
                format!("tag={:3}({:22}) lv={} sz={}", r.tag_id, tags::tag_name(r.tag_id), r.level, r.data.len())
            } else { "---".to_string() };
            let saved_info = if i < saved_recs.len() {
                let r = &saved_recs[i];
                format!("tag={:3}({:22}) lv={} sz={}", r.tag_id, tags::tag_name(r.tag_id), r.level, r.data.len())
            } else { "---".to_string() };
            let match_mark = if i < ref_recs.len() && i < saved_recs.len() {
                let r = &ref_recs[i];
                let s = &saved_recs[i];
                if r.tag_id == s.tag_id && r.level == s.level && r.data.len() == s.data.len() {
                    if r.data == s.data { "==" } else { "~=" }
                } else { "!=" }
            } else { "!=" };
            eprintln!("  [{:2}] {} {} | {}", i, match_mark, ref_info, saved_info);
        }

        // 주요 레코드 바이트 비교
        for i in 0..ref_recs.len().min(saved_recs.len()) {
            let r = &ref_recs[i];
            let s = &saved_recs[i];
            if r.tag_id == s.tag_id && r.data != s.data {
                eprintln!("\n  [차이 상세] 레코드 {}: {}", i, tags::tag_name(r.tag_id));
                let max_show = r.data.len().max(s.data.len()).min(120);
                eprintln!("    참조: {:02x?}", &r.data[..r.data.len().min(max_show)]);
                eprintln!("    저장: {:02x?}", &s.data[..s.data.len().min(max_show)]);
                // 첫 번째 차이 위치
                for j in 0..r.data.len().min(s.data.len()) {
                    if r.data[j] != s.data[j] {
                        eprintln!("    첫 차이: offset {} (참조=0x{:02x}, 저장=0x{:02x})", j, r.data[j], s.data[j]);
                        break;
                    }
                }
            }
        }

        // CFB 스트림 목록 확인
        let streams = saved_cfb.list_streams();
        eprintln!("\n  --- CFB 스트림 목록 ---");
        for s in &streams {
            eprintln!("  {}", s);
        }
        let has_bindata = streams.iter().any(|s| s.contains("BinData") || s.contains("BIN"));
        assert!(has_bindata, "BinData 스트림이 없음");

        eprintln!("\n=== 단계 4-2 이미지 저장 검증 완료 ===");
    }

    /// 추가 검증: 표 안에 이미지 삽입 — 참조 파일 분석
    /// output/pic-in-tb-01.hwp: 빈 문서 → 1×1 표 → 셀 안에 이미지 삽입
    #[test]
    fn test_analyze_pic_in_table() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::parser::cfb_reader::LenientCfbReader;
        use crate::model::control::Control;

        let path = "output/pic-in-tb-01.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();

        eprintln!("\n{}", "=".repeat(70));
        eprintln!("  표 안 이미지 참조 파일 분석: {}", path);
        eprintln!("  파일 크기: {} bytes", data.len());
        eprintln!("{}", "=".repeat(70));

        let doc = match HwpDocument::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  표준 파서 실패 ({}), LenientCfbReader로 분석합니다.", e);
                let lcfb = LenientCfbReader::open(&data).unwrap();

                eprintln!("\n  [LenientCFB 엔트리]");
                for (name, start, size, otype) in lcfb.list_entries() {
                    let tname = match otype { 1 => "storage", 2 => "stream", 5 => "root", _ => "?" };
                    eprintln!("  {:30} start={:5} size={:8} type={}", name, start, size, tname);
                }

                let fh = lcfb.read_stream("FileHeader").unwrap();
                let compressed = fh.len() >= 37 && (fh[36] & 0x01) != 0;

                // DocInfo
                let di_data = lcfb.read_doc_info(compressed).unwrap();
                let di_recs = Record::read_all(&di_data).unwrap();

                // 캐럿 위치
                if let Some(dp_rec) = di_recs.first() {
                    if dp_rec.tag_id == tags::HWPTAG_DOCUMENT_PROPERTIES && dp_rec.data.len() >= 26 {
                        let d = &dp_rec.data;
                        eprintln!("\n  [캐럿 위치]");
                        eprintln!("  list_id={} para_id={} char_pos={}",
                            u32::from_le_bytes([d[14], d[15], d[16], d[17]]),
                            u32::from_le_bytes([d[18], d[19], d[20], d[21]]),
                            u32::from_le_bytes([d[22], d[23], d[24], d[25]]));
                    }
                }

                // ID_MAPPINGS
                if di_recs.len() > 1 && di_recs[1].tag_id == tags::HWPTAG_ID_MAPPINGS {
                    let d = &di_recs[1].data;
                    if d.len() >= 72 {
                        eprintln!("\n  [ID_MAPPINGS]");
                        let labels = ["bin_data", "font_kr", "font_en", "font_cn", "font_jp",
                            "font_etc", "font_sym", "font_usr", "border_fill", "char_shape",
                            "tab_def", "numbering", "bullet", "para_shape", "style",
                            "memo_shape", "trackchange", "trackchange_author"];
                        for (i, label) in labels.iter().enumerate() {
                            let off = i * 4;
                            let val = u32::from_le_bytes([d[off], d[off+1], d[off+2], d[off+3]]);
                            if val > 0 { eprintln!("  {:20}: {}", label, val); }
                        }
                    }
                }

                // BIN_DATA 레코드
                eprintln!("\n  [DocInfo BIN_DATA 레코드]");
                for (i, r) in di_recs.iter().enumerate() {
                    if r.tag_id == tags::HWPTAG_BIN_DATA {
                        eprintln!("  [{:2}] BIN_DATA size={} data: {:02x?}",
                            i, r.data.len(), &r.data[..r.data.len().min(60)]);
                    }
                }

                // BodyText
                let bt_data = lcfb.read_body_text_section(0, compressed).unwrap();
                let bt_recs = Record::read_all(&bt_data).unwrap();

                eprintln!("\n  [BodyText 레코드] ({} 개)", bt_recs.len());
                for (i, r) in bt_recs.iter().enumerate() {
                    let tname = tags::tag_name(r.tag_id);
                    let mut extra = String::new();
                    if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                        let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                        extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
                    }
                    eprintln!("  [{:2}] tag={:3}({:22}) level={} size={}{}",
                        i, r.tag_id, tname, r.level, r.data.len(), extra);
                    if matches!(r.tag_id, 66 | 67 | 68 | 69 | 71 | 72 | 76 | 77 | 85) {
                        let show = r.data.len().min(100);
                        eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
                        if r.data.len() > 100 { eprintln!("        total: {} bytes", r.data.len()); }
                    }
                }

                eprintln!("\n=== 분석 완료 (LenientCfbReader) ===");
                return;
            }
        };

        // === 표준 파서 성공 ===

        // 1. 캐럿 위치
        let dp = &doc.document.doc_properties;
        eprintln!("\n  [캐럿 위치]");
        eprintln!("  list_id={} para_id={} char_pos={}", dp.caret_list_id, dp.caret_para_id, dp.caret_char_pos);

        // 2. BinData
        eprintln!("\n  [BinData] ({} 개)", doc.document.doc_info.bin_data_list.len());
        for (i, bd) in doc.document.doc_info.bin_data_list.iter().enumerate() {
            eprintln!("  [{}] attr=0x{:04X} type={:?} storage_id={} ext={:?}",
                i, bd.attr, bd.data_type, bd.storage_id, bd.extension);
        }
        eprintln!("  [BinDataContent] ({} 개)", doc.document.bin_data_content.len());
        for (i, bc) in doc.document.bin_data_content.iter().enumerate() {
            eprintln!("  [{}] id={} ext='{}' size={}", i, bc.id, bc.extension, bc.data.len());
        }

        // 3. 문단/컨트롤 구조 (재귀적)
        for (si, sec) in doc.document.sections.iter().enumerate() {
            eprintln!("\n  [섹션 {}] 문단: {}", si, sec.paragraphs.len());
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                eprintln!("  문단[{}]: cc={} msb={} ctrls={} mask=0x{:08X} ps={} ss={}",
                    pi, para.char_count, para.char_count_msb, para.controls.len(),
                    para.control_mask, para.para_shape_id, para.style_id);
                eprintln!("    cs={:?} ls={:?}", para.char_shapes, para.line_segs);
                eprintln!("    raw_header_extra({} bytes): {:02x?}",
                    para.raw_header_extra.len(), &para.raw_header_extra[..para.raw_header_extra.len().min(20)]);

                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::SectionDef(_) => eprintln!("    ctrl[{}]: SectionDef", ci),
                        Control::ColumnDef(_) => eprintln!("    ctrl[{}]: ColumnDef", ci),
                        Control::Table(t) => {
                            eprintln!("    ctrl[{}]: Table {}×{} cells={} bfid={} attr=0x{:08X}",
                                ci, t.row_count, t.col_count, t.cells.len(), t.border_fill_id, t.attr);
                            eprintln!("      padding: l={} r={} t={} b={}", t.padding.left, t.padding.right, t.padding.top, t.padding.bottom);
                            eprintln!("      cell_spacing={} row_sizes={:?}", t.cell_spacing, t.row_sizes);
                            eprintln!("      raw_ctrl_data({} bytes): {:02x?}", t.raw_ctrl_data.len(), &t.raw_ctrl_data[..t.raw_ctrl_data.len().min(40)]);
                            for (celli, cell) in t.cells.iter().enumerate() {
                                eprintln!("      cell[{}]: col={} row={} span={}×{} w={} h={} bfid={}",
                                    celli, cell.col, cell.row, cell.col_span, cell.row_span,
                                    cell.width, cell.height, cell.border_fill_id);
                                eprintln!("        padding: l={} r={} t={} b={} paras={}",
                                    cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
                                    cell.paragraphs.len());
                                // 셀 내 문단/컨트롤
                                for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                    eprintln!("        para[{}]: cc={} msb={} ctrls={} mask=0x{:08X}",
                                        cpi, cp.char_count, cp.char_count_msb, cp.controls.len(), cp.control_mask);
                                    eprintln!("          cs={:?}", cp.char_shapes);
                                    eprintln!("          ls={:?}", cp.line_segs);
                                    for (cci, cctrl) in cp.controls.iter().enumerate() {
                                        match cctrl {
                                            Control::Picture(pic) => {
                                                eprintln!("          ctrl[{}]: Picture {}×{} bid={}",
                                                    cci, pic.common.width, pic.common.height, pic.image_attr.bin_data_id);
                                                eprintln!("            attr=0x{:08X} z={} margins=({},{},{},{})",
                                                    pic.common.attr, pic.common.z_order,
                                                    pic.common.margin.left, pic.common.margin.right,
                                                    pic.common.margin.top, pic.common.margin.bottom);
                                                eprintln!("            shape: ctrl_id=0x{:08X} two={} orig={}×{} cur={}×{}",
                                                    pic.shape_attr.ctrl_id, pic.shape_attr.is_two_ctrl_id,
                                                    pic.shape_attr.original_width, pic.shape_attr.original_height,
                                                    pic.shape_attr.current_width, pic.shape_attr.current_height);
                                                eprintln!("            border_x={:?} border_y={:?}",
                                                    pic.border_x, pic.border_y);
                                                eprintln!("            crop: l={} t={} r={} b={}",
                                                    pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom);
                                                eprintln!("            raw_extra({} bytes) raw_rendering({} bytes) raw_pic_extra({} bytes)",
                                                    pic.common.raw_extra.len(), pic.shape_attr.raw_rendering.len(), pic.raw_picture_extra.len());
                                            },
                                            _ => eprintln!("          ctrl[{}]: {:?}", cci, std::mem::discriminant(cctrl)),
                                        }
                                    }
                                }
                            }
                        },
                        Control::Picture(pic) => {
                            eprintln!("    ctrl[{}]: Picture {}×{} bid={}",
                                ci, pic.common.width, pic.common.height, pic.image_attr.bin_data_id);
                        },
                        _ => eprintln!("    ctrl[{}]: {:?}", ci, std::mem::discriminant(ctrl)),
                    }
                }
            }
        }

        // 4. BodyText 레코드 덤프
        let parsed = crate::parser::parse_hwp(&data).unwrap();
        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&data).unwrap();
        let bt = cfb.read_body_text_section(0, parsed.header.compressed, false).unwrap();
        let recs = Record::read_all(&bt).unwrap();

        eprintln!("\n  [BodyText 레코드] ({} 개)", recs.len());
        for (i, r) in recs.iter().enumerate() {
            let tname = tags::tag_name(r.tag_id);
            let mut extra = String::new();
            if r.tag_id == tags::HWPTAG_CTRL_HEADER && r.data.len() >= 4 {
                let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                extra = format!(" ctrl='{}'", tags::ctrl_name(cid));
            }
            eprintln!("  [{:2}] tag={:3}({:22}) level={} size={}{}",
                i, r.tag_id, tname, r.level, r.data.len(), extra);
            if matches!(r.tag_id, 66 | 67 | 68 | 69 | 71 | 72 | 76 | 77 | 85) {
                let show = r.data.len().min(100);
                eprintln!("        data[..{}]: {:02x?}", show, &r.data[..show]);
                if r.data.len() > 100 { eprintln!("        total: {} bytes", r.data.len()); }
            }
        }

        // 5. 비교
        let empty_path = "template/empty.hwp";
        if std::path::Path::new(empty_path).exists() {
            let empty_data = std::fs::read(empty_path).unwrap();
            let empty_parsed = crate::parser::parse_hwp(&empty_data).unwrap();
            let mut empty_cfb = crate::parser::cfb_reader::CfbReader::open(&empty_data).unwrap();
            let empty_bt = empty_cfb.read_body_text_section(0, empty_parsed.header.compressed, false).unwrap();
            let empty_recs = Record::read_all(&empty_bt).unwrap();
            eprintln!("\n  [비교] empty.hwp={} 개, pic-in-tb={} 개 → 차이={} 개",
                empty_recs.len(), recs.len(), recs.len() as i32 - empty_recs.len() as i32);
        }

        // 6. 라운드트립 검증
        eprintln!("\n  [라운드트립 검증]");
        let mut doc_mut = HwpDocument::from_bytes(&data).unwrap();
        for sec in &mut doc_mut.document.sections {
            sec.raw_stream = None;
        }
        doc_mut.document.doc_info.raw_stream = None;
        doc_mut.document.doc_properties.raw_data = None;

        let saved = doc_mut.export_hwp_native();
        match saved {
            Ok(saved_data) => {
                let _ = std::fs::create_dir_all("output");
                std::fs::write("output/roundtrip_pic_in_tb.hwp", &saved_data).unwrap();
                eprintln!("  저장: output/roundtrip_pic_in_tb.hwp ({} bytes)", saved_data.len());

                // 재파싱
                match HwpDocument::from_bytes(&saved_data) {
                    Ok(doc2) => {
                        eprintln!("  재파싱 성공 ✓");

                        // 레코드 비교
                        let saved_parsed = crate::parser::parse_hwp(&saved_data).unwrap();
                        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
                        let saved_bt = saved_cfb.read_body_text_section(0, saved_parsed.header.compressed, false).unwrap();
                        let saved_recs = Record::read_all(&saved_bt).unwrap();

                        eprintln!("  레코드: 원본={} 저장={}", recs.len(), saved_recs.len());

                        let max = recs.len().max(saved_recs.len());
                        let mut diff_count = 0;
                        for i in 0..max {
                            if i < recs.len() && i < saved_recs.len() {
                                let o = &recs[i];
                                let s = &saved_recs[i];
                                let mark = if o.tag_id == s.tag_id && o.level == s.level && o.data == s.data { "==" }
                                    else if o.tag_id == s.tag_id && o.level == s.level { "~=" }
                                    else { "!=" };
                                if mark != "==" {
                                    diff_count += 1;
                                    if diff_count <= 10 {
                                        eprintln!("  [{:2}] {} {}(lv{} sz{}) vs {}(lv{} sz{})",
                                            i, mark, tags::tag_name(o.tag_id), o.level, o.data.len(),
                                            tags::tag_name(s.tag_id), s.level, s.data.len());
                                    }
                                }
                            }
                        }
                        if diff_count > 10 { eprintln!("  ... 외 {} 개", diff_count - 10); }
                        eprintln!("  일치: {}/{} ({}%)",
                            max.saturating_sub(diff_count), max,
                            if max > 0 { (max.saturating_sub(diff_count)) * 100 / max } else { 100 });

                        // 표 안 이미지 보존 확인
                        let mut pic_in_cell = false;
                        for sec in &doc2.document.sections {
                            for para in &sec.paragraphs {
                                for ctrl in &para.controls {
                                    if let Control::Table(t) = ctrl {
                                        for cell in &t.cells {
                                            for cp in &cell.paragraphs {
                                                for cc in &cp.controls {
                                                    if matches!(cc, Control::Picture(_)) {
                                                        pic_in_cell = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        eprintln!("  표 안 이미지 보존: {}", if pic_in_cell { "✓" } else { "✗" });
                        assert!(pic_in_cell, "라운드트립 후 표 안 이미지가 사라짐!");
                    }
                    Err(e) => eprintln!("  재파싱 실패: {}", e),
                }
            }
            Err(e) => eprintln!("  저장 실패: {}", e),
        }

        eprintln!("\n=== 표 안 이미지 참조 파일 분석 완료 ===");
    }

    /// 단계 5: 기타 컨트롤 라운드트립 검증
    /// 여러 샘플 파일에서 Header/Footer/Footnote/Endnote/Shape/Bookmark 라운드트립
    #[test]
    fn test_roundtrip_all_controls() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::model::control::Control;

        let samples = [
            "samples/k-water-rfp.hwp",
            "samples/20250130-hongbo.hwp",
            "samples/hwp-multi-001.hwp",
            "samples/hwp-multi-002.hwp",
            "samples/2010-01-06.hwp",
        ];

        eprintln!("\n{}", "=".repeat(70));
        eprintln!("  단계 5: 기타 컨트롤 라운드트립 검증");
        eprintln!("{}", "=".repeat(70));

        for sample_path in &samples {
            if !std::path::Path::new(sample_path).exists() {
                eprintln!("\n  SKIP: {} 없음", sample_path);
                continue;
            }

            let orig_data = std::fs::read(sample_path).unwrap();
            let doc = match HwpDocument::from_bytes(&orig_data) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("\n  SKIP: {} 파싱 실패: {}", sample_path, e);
                    continue;
                }
            };

            // 컨트롤 종류 카운트
            let mut ctrl_counts = std::collections::HashMap::new();
            for sec in &doc.document.sections {
                fn count_controls(paras: &[crate::model::paragraph::Paragraph], counts: &mut std::collections::HashMap<String, usize>) {
                    for para in paras {
                        for ctrl in &para.controls {
                            let name = match ctrl {
                                Control::SectionDef(_) => "SectionDef",
                                Control::ColumnDef(_) => "ColumnDef",
                                Control::Table(t) => {
                                    // 표 안의 컨트롤도 카운트
                                    for cell in &t.cells {
                                        count_controls(&cell.paragraphs, counts);
                                    }
                                    "Table"
                                },
                                Control::Picture(_) => "Picture",
                                Control::Shape(_) => "Shape",
                                Control::Header(h) => {
                                    count_controls(&h.paragraphs, counts);
                                    "Header"
                                },
                                Control::Footer(f) => {
                                    count_controls(&f.paragraphs, counts);
                                    "Footer"
                                },
                                Control::Footnote(f) => {
                                    count_controls(&f.paragraphs, counts);
                                    "Footnote"
                                },
                                Control::Endnote(e) => {
                                    count_controls(&e.paragraphs, counts);
                                    "Endnote"
                                },
                                Control::HiddenComment(_) => "HiddenComment",
                                Control::AutoNumber(_) => "AutoNumber",
                                Control::NewNumber(_) => "NewNumber",
                                Control::PageNumberPos(_) => "PageNumberPos",
                                Control::Bookmark(_) => "Bookmark",
                                _ => "Other",
                            };
                            *counts.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                count_controls(&sec.paragraphs, &mut ctrl_counts);
            }

            // 관심 대상 컨트롤만 필터링
            let target_ctrls = ["Header", "Footer", "Footnote", "Endnote", "Shape", "Bookmark", "Picture", "Table"];
            let has_target = target_ctrls.iter().any(|t| ctrl_counts.contains_key(*t));

            eprintln!("\n  --- {} ---", sample_path);
            eprintln!("  섹션: {} 문단: {}",
                doc.document.sections.len(),
                doc.document.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>());
            eprintln!("  컨트롤: {:?}", ctrl_counts);

            if !has_target {
                eprintln!("  → 대상 컨트롤 없음, 건너뜀");
                continue;
            }

            // 라운드트립: 원본 → 재직렬화 → 저장 → 재파싱
            let mut doc_mut = doc;
            for sec in &mut doc_mut.document.sections {
                sec.raw_stream = None;
            }
            doc_mut.document.doc_info.raw_stream = None;
            doc_mut.document.doc_properties.raw_data = None;

            let saved = match doc_mut.export_hwp_native() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("  저장 실패: {}", e);
                    continue;
                }
            };

            // 재파싱
            let doc2 = match HwpDocument::from_bytes(&saved) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("  재파싱 실패: {}", e);
                    // 저장 파일 기록 (디버그용)
                    let fname = format!("output/roundtrip_fail_{}.hwp",
                        std::path::Path::new(sample_path).file_stem().unwrap().to_str().unwrap());
                    let _ = std::fs::create_dir_all("output");
                    std::fs::write(&fname, &saved).unwrap();
                    eprintln!("  디버그 파일: {} ({} bytes)", fname, saved.len());
                    continue;
                }
            };

            // 재파싱 후 컨트롤 카운트 비교
            let mut ctrl_counts2 = std::collections::HashMap::new();
            for sec in &doc2.document.sections {
                fn count_controls2(paras: &[crate::model::paragraph::Paragraph], counts: &mut std::collections::HashMap<String, usize>) {
                    for para in paras {
                        for ctrl in &para.controls {
                            let name = match ctrl {
                                Control::SectionDef(_) => "SectionDef",
                                Control::ColumnDef(_) => "ColumnDef",
                                Control::Table(t) => {
                                    for cell in &t.cells {
                                        count_controls2(&cell.paragraphs, counts);
                                    }
                                    "Table"
                                },
                                Control::Picture(_) => "Picture",
                                Control::Shape(_) => "Shape",
                                Control::Header(h) => {
                                    count_controls2(&h.paragraphs, counts);
                                    "Header"
                                },
                                Control::Footer(f) => {
                                    count_controls2(&f.paragraphs, counts);
                                    "Footer"
                                },
                                Control::Footnote(f) => {
                                    count_controls2(&f.paragraphs, counts);
                                    "Footnote"
                                },
                                Control::Endnote(e) => {
                                    count_controls2(&e.paragraphs, counts);
                                    "Endnote"
                                },
                                Control::HiddenComment(_) => "HiddenComment",
                                Control::AutoNumber(_) => "AutoNumber",
                                Control::NewNumber(_) => "NewNumber",
                                Control::PageNumberPos(_) => "PageNumberPos",
                                Control::Bookmark(_) => "Bookmark",
                                _ => "Other",
                            };
                            *counts.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                count_controls2(&sec.paragraphs, &mut ctrl_counts2);
            }

            // 대상 컨트롤별 보존 여부 확인
            let mut all_match = true;
            for target in &target_ctrls {
                let orig_count = ctrl_counts.get(*target).copied().unwrap_or(0);
                let saved_count = ctrl_counts2.get(*target).copied().unwrap_or(0);
                if orig_count > 0 || saved_count > 0 {
                    let status = if orig_count == saved_count { "✓" } else { "✗" };
                    eprintln!("  {:12} 원본={:2} 저장={:2} {}", target, orig_count, saved_count, status);
                    if orig_count != saved_count { all_match = false; }
                }
            }

            // 레코드 수 비교 (섹션 0)
            let orig_parsed = crate::parser::parse_hwp(&orig_data).unwrap();
            let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
            let orig_bt = orig_cfb.read_body_text_section(0, orig_parsed.header.compressed, false).unwrap();
            let orig_recs = Record::read_all(&orig_bt).unwrap();

            let saved_parsed = crate::parser::parse_hwp(&saved).unwrap();
            let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved).unwrap();
            let saved_bt = saved_cfb.read_body_text_section(0, saved_parsed.header.compressed, false).unwrap();
            let saved_recs = Record::read_all(&saved_bt).unwrap();

            eprintln!("  레코드: 원본={} 저장={} {}",
                orig_recs.len(), saved_recs.len(),
                if orig_recs.len() == saved_recs.len() { "✓" } else { "✗" });

            // 레코드 차이 요약 (다른 레코드만)
            let mut diff_count = 0;
            let max_recs = orig_recs.len().min(saved_recs.len());
            for i in 0..max_recs {
                let o = &orig_recs[i];
                let s = &saved_recs[i];
                if o.tag_id != s.tag_id || o.level != s.level || o.data != s.data {
                    diff_count += 1;
                    if diff_count <= 5 {
                        let match_type = if o.tag_id != s.tag_id || o.level != s.level { "구조" } else { "데이터" };
                        eprintln!("  DIFF[{:3}] {} {} lv{} sz{} vs {} lv{} sz{}",
                            i, match_type, tags::tag_name(o.tag_id), o.level, o.data.len(),
                            tags::tag_name(s.tag_id), s.level, s.data.len());
                    }
                }
            }
            if diff_count > 5 {
                eprintln!("  ... 외 {} 개 차이", diff_count - 5);
            }
            eprintln!("  일치: {}/{} 레코드 ({}%)", max_recs - diff_count, max_recs,
                if max_recs > 0 { (max_recs - diff_count) * 100 / max_recs } else { 100 });

            if all_match {
                eprintln!("  → 라운드트립 성공 ✓");
            }
        }

        eprintln!("\n=== 단계 5 기타 컨트롤 라운드트립 검증 완료 ===");
    }

    /// 추가 검증: 빈 HWP → 1×1 표 → 셀 안에 이미지 삽입 (FROM SCRATCH)
    /// 참조: output/pic-in-tb-01.hwp (HWP 프로그램으로 생성)
    #[test]
    fn test_save_pic_in_table() {
        use crate::parser::record::Record;
        use crate::parser::tags;
        use crate::model::control::Control;
        use crate::model::table::{Table, Cell};
        use crate::model::Padding;
        use crate::model::bin_data::{BinData, BinDataType, BinDataCompression, BinDataStatus, BinDataContent};
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        eprintln!("\n=== 추가 검증: 표 안 이미지 저장 (FROM SCRATCH) ===");

        // 1. 참조 파일에서 Table, Picture, BinData 구조 추출
        let ref_path = "output/pic-in-tb-01.hwp";
        if !std::path::Path::new(ref_path).exists() {
            eprintln!("SKIP: {} 없음", ref_path);
            return;
        }
        let ref_data = std::fs::read(ref_path).unwrap();
        let ref_doc = HwpDocument::from_bytes(&ref_data).unwrap();

        // 참조 파일에서 Table 컨트롤 추출
        let ref_table = ref_doc.document.sections[0].paragraphs[0].controls.iter()
            .find_map(|c| if let Control::Table(t) = c { Some(t) } else { None })
            .expect("참조 파일에 Table 컨트롤 없음");

        // 참조 파일에서 셀 안 Picture 컨트롤 추출
        let ref_pic = ref_table.cells[0].paragraphs[0].controls.iter()
            .find_map(|c| if let Control::Picture(p) = c { Some(p) } else { None })
            .expect("참조 파일 셀 안에 Picture 컨트롤 없음");

        let ref_bindata = &ref_doc.document.doc_info.bin_data_list[0];
        let ref_bincontent = &ref_doc.document.bin_data_content[0];
        let ref_cell = &ref_table.cells[0];
        let ref_cell_para = &ref_cell.paragraphs[0];
        let ref_para0 = &ref_doc.document.sections[0].paragraphs[0];
        let ref_para1 = &ref_doc.document.sections[0].paragraphs[1];

        eprintln!("  참조 Table: {}×{} bfid={} attr=0x{:08X}",
            ref_table.row_count, ref_table.col_count, ref_table.border_fill_id, ref_table.attr);
        eprintln!("  참조 Cell: col={} row={} w={} h={} bfid={}",
            ref_cell.col, ref_cell.row, ref_cell.width, ref_cell.height, ref_cell.border_fill_id);
        eprintln!("  참조 Cell 문단: cc={} msb={} mask=0x{:08X} ctrls={}",
            ref_cell_para.char_count, ref_cell_para.char_count_msb,
            ref_cell_para.control_mask, ref_cell_para.controls.len());
        eprintln!("  참조 Picture: {}×{} bid={} z={}",
            ref_pic.common.width, ref_pic.common.height,
            ref_pic.image_attr.bin_data_id, ref_pic.common.z_order);
        eprintln!("  참조 캐럿: list_id={} para_id={} char_pos={}",
            ref_doc.document.doc_properties.caret_list_id,
            ref_doc.document.doc_properties.caret_para_id,
            ref_doc.document.doc_properties.caret_char_pos);

        // 2. empty.hwp 로드
        let empty_path = "template/empty.hwp";
        assert!(std::path::Path::new(empty_path).exists(), "template/empty.hwp 없음");
        let empty_data = std::fs::read(empty_path).unwrap();
        let mut doc = HwpDocument::from_bytes(&empty_data).unwrap();

        // 3. DocInfo에 BinData 추가
        let bin_data_entry = BinData {
            attr: ref_bindata.attr,
            data_type: BinDataType::Embedding,
            compression: BinDataCompression::Default,
            status: BinDataStatus::NotAccessed,
            storage_id: 1,
            extension: Some(ref_bincontent.extension.clone()),
            raw_data: None,
            ..Default::default()
        };
        doc.document.doc_info.bin_data_list.push(bin_data_entry);

        // BinDataContent 추가
        doc.document.bin_data_content.push(BinDataContent {
            id: 1,
            data: ref_bincontent.data.clone(),
            extension: ref_bincontent.extension.clone(),
        });

        // 4. DocInfo에 BorderFill 추가 (표 테두리용)
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};
        let solid_border = BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 };
        let new_bf = BorderFill {
            raw_data: None,
            attr: 0,
            borders: [solid_border, solid_border, solid_border, solid_border],
            diagonal: DiagonalLine { diagonal_type: 1, width: 0, color: 0 },
            fill: Fill::default(),
        };
        doc.document.doc_info.border_fills.push(new_bf);
        let table_bf_id = doc.document.doc_info.border_fills.len() as u16;
        eprintln!("  DocInfo: border_fill_count={}, table_bf_id={}", doc.document.doc_info.border_fills.len(), table_bf_id);

        // 5. Picture 컨트롤 구성 (참조 파일의 정확한 값 사용)
        let picture = crate::model::image::Picture {
            common: ref_pic.common.clone(),
            shape_attr: ref_pic.shape_attr.clone(),
            border_color: ref_pic.border_color,
            border_width: ref_pic.border_width,
            border_attr: ref_pic.border_attr.clone(),
            border_x: ref_pic.border_x,
            border_y: ref_pic.border_y,
            crop: ref_pic.crop.clone(),
            padding: ref_pic.padding.clone(),
            image_attr: ref_pic.image_attr.clone(),
            border_opacity: ref_pic.border_opacity,
            instance_id: ref_pic.instance_id,
            raw_picture_extra: ref_pic.raw_picture_extra.clone(),
            caption: None,
        };

        // 6. 셀 내부 문단 구성 (cc=9: gso(8)+CR(1), mask=0x00000800)
        let cell_para = Paragraph {
            text: String::new(),
            char_count: ref_cell_para.char_count,       // 9
            char_count_msb: ref_cell_para.char_count_msb, // true
            control_mask: ref_cell_para.control_mask,    // 0x00000800
            para_shape_id: 0,
            style_id: 0,
            raw_break_type: ref_cell_para.raw_break_type,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: ref_cell_para.line_segs[0].text_start,
                vertical_pos: ref_cell_para.line_segs[0].vertical_pos,
                line_height: ref_cell_para.line_segs[0].line_height,     // 15600 (= image height)
                text_height: ref_cell_para.line_segs[0].text_height,
                baseline_distance: ref_cell_para.line_segs[0].baseline_distance,
                line_spacing: ref_cell_para.line_segs[0].line_spacing,
                column_start: ref_cell_para.line_segs[0].column_start,
                segment_width: ref_cell_para.line_segs[0].segment_width, // 40932
                tag: ref_cell_para.line_segs[0].tag,
            }],
            has_para_text: true,  // gso 제어문자 있으므로 PARA_TEXT 필요
            controls: vec![Control::Picture(Box::new(picture))],
            raw_header_extra: ref_cell_para.raw_header_extra.clone(),
            ..Default::default()
        };

        // 7. Cell 구성
        let cell = Cell {
            col: ref_cell.col,
            row: ref_cell.row,
            col_span: ref_cell.col_span,
            row_span: ref_cell.row_span,
            width: ref_cell.width,
            height: ref_cell.height,
            border_fill_id: table_bf_id,
            padding: Padding {
                left: ref_cell.padding.left,
                right: ref_cell.padding.right,
                top: ref_cell.padding.top,
                bottom: ref_cell.padding.bottom,
            },
            list_header_width_ref: ref_cell.list_header_width_ref,
            raw_list_extra: ref_cell.raw_list_extra.clone(),
            paragraphs: vec![cell_para],
            ..Default::default()
        };

        // 8. Table 구성
        let table = Table {
            attr: ref_table.attr,
            row_count: ref_table.row_count,
            col_count: ref_table.col_count,
            cell_spacing: ref_table.cell_spacing,
            padding: Padding {
                left: ref_table.padding.left,
                right: ref_table.padding.right,
                top: ref_table.padding.top,
                bottom: ref_table.padding.bottom,
            },
            row_sizes: ref_table.row_sizes.clone(),
            border_fill_id: table_bf_id,
            cells: vec![cell],
            raw_ctrl_data: ref_table.raw_ctrl_data.clone(),
            raw_table_record_attr: ref_table.raw_table_record_attr,
            raw_table_record_extra: ref_table.raw_table_record_extra.clone(),
            ..Default::default()
        };

        // 9. 첫 번째 문단에 Table 컨트롤 추가
        {
            let para = &mut doc.document.sections[0].paragraphs[0];
            para.controls.push(Control::Table(Box::new(table)));
            para.ctrl_data_records.push(None);
            para.char_count += 8;  // 표 제어문자 8 code units
            para.control_mask = ref_para0.control_mask; // 0x00000804

            // 표가 있는 문단의 segment_width는 0 (참조 파일)
            if let Some(ls) = para.line_segs.first_mut() {
                ls.segment_width = 0;
            }
        }

        // 10. 두 번째 빈 문단 추가 (표 아래)
        let empty_para = Paragraph {
            text: String::new(),
            char_count: ref_para1.char_count,           // 1
            char_count_msb: ref_para1.char_count_msb,   // true
            control_mask: ref_para1.control_mask,       // 0
            para_shape_id: 0,
            style_id: 0,
            raw_break_type: ref_para1.raw_break_type,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: ref_para1.line_segs[0].text_start,
                vertical_pos: ref_para1.line_segs[0].vertical_pos,  // 16448
                line_height: ref_para1.line_segs[0].line_height,
                text_height: ref_para1.line_segs[0].text_height,
                baseline_distance: ref_para1.line_segs[0].baseline_distance,
                line_spacing: ref_para1.line_segs[0].line_spacing,
                column_start: ref_para1.line_segs[0].column_start,
                segment_width: ref_para1.line_segs[0].segment_width,
                tag: ref_para1.line_segs[0].tag,
            }],
            has_para_text: false,
            raw_header_extra: ref_para1.raw_header_extra.clone(),
            ..Default::default()
        };
        doc.document.sections[0].paragraphs.push(empty_para);

        // 11. raw_stream 무효화 (재직렬화)
        doc.document.sections[0].raw_stream = None;
        doc.document.doc_info.raw_stream = None;
        doc.document.doc_properties.raw_data = None;

        // 캐럿 위치 (참조: list_id=0, para_id=1, char_pos=0)
        doc.document.doc_properties.caret_list_id = ref_doc.document.doc_properties.caret_list_id;
        doc.document.doc_properties.caret_para_id = ref_doc.document.doc_properties.caret_para_id;
        doc.document.doc_properties.caret_char_pos = ref_doc.document.doc_properties.caret_char_pos;

        let para = &doc.document.sections[0].paragraphs[0];
        eprintln!("  구성 문단[0]: cc={} ctrls={} mask=0x{:08X} seg_w={}",
            para.char_count, para.controls.len(), para.control_mask,
            para.line_segs.first().map(|ls| ls.segment_width).unwrap_or(-1));
        let para1 = &doc.document.sections[0].paragraphs[1];
        eprintln!("  구성 문단[1]: cc={} vpos={}",
            para1.char_count,
            para1.line_segs.first().map(|ls| ls.vertical_pos).unwrap_or(-1));

        // 12. 저장
        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "HWP 저장 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/save_test_pic_in_table.hwp", &saved_data).unwrap();
        eprintln!("  저장: output/save_test_pic_in_table.hwp ({} bytes)", saved_data.len());

        // 13. 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();

        // 표 컨트롤 존재 검증
        let para2 = &doc2.document.sections[0].paragraphs[0];
        let table_found = para2.controls.iter().any(|c| matches!(c, Control::Table(_)));
        assert!(table_found, "재파싱된 문서에 표 컨트롤이 없음");

        // 표 안 이미지 보존 검증
        let mut pic_in_cell = false;
        if let Some(Control::Table(t)) = para2.controls.iter().find(|c| matches!(c, Control::Table(_))) {
            eprintln!("  재파싱 표: {}×{} cells={}", t.row_count, t.col_count, t.cells.len());
            assert_eq!(t.row_count, 1);
            assert_eq!(t.col_count, 1);
            assert_eq!(t.cells.len(), 1);

            for cp in &t.cells[0].paragraphs {
                for cc in &cp.controls {
                    if let Control::Picture(p) = cc {
                        pic_in_cell = true;
                        eprintln!("  셀 안 Picture: {}×{} bid={}",
                            p.common.width, p.common.height, p.image_attr.bin_data_id);
                        assert_eq!(p.image_attr.bin_data_id, ref_pic.image_attr.bin_data_id);
                        assert_eq!(p.common.width, ref_pic.common.width);
                        assert_eq!(p.common.height, ref_pic.common.height);
                    }
                }
            }
        }
        assert!(pic_in_cell, "재파싱 후 표 안 이미지가 없음");

        // BinData 검증
        assert_eq!(doc2.document.doc_info.bin_data_list.len(), 1, "BinData 없음");
        assert_eq!(doc2.document.doc_info.bin_data_list[0].data_type, BinDataType::Embedding);
        assert_eq!(doc2.document.bin_data_content.len(), 1, "BinDataContent 없음");
        assert_eq!(doc2.document.bin_data_content[0].data.len(), ref_bincontent.data.len(),
            "이미지 데이터 크기 불일치");

        // 두 번째 문단 검증
        assert!(doc2.document.sections[0].paragraphs.len() >= 2, "표 아래 빈 문단이 없음");

        // 캐럿 위치 검증
        eprintln!("  캐럿: list_id={} para_id={} char_pos={}",
            doc2.document.doc_properties.caret_list_id,
            doc2.document.doc_properties.caret_para_id,
            doc2.document.doc_properties.caret_char_pos);

        // 14. 참조 파일과 레코드 비교
        let saved_parsed = crate::parser::parse_hwp(&saved_data).unwrap();
        let mut saved_cfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        let saved_bt = saved_cfb.read_body_text_section(0, saved_parsed.header.compressed, false).unwrap();
        let saved_recs = Record::read_all(&saved_bt).unwrap();

        let ref_parsed = crate::parser::parse_hwp(&ref_data).unwrap();
        let mut ref_cfb = crate::parser::cfb_reader::CfbReader::open(&ref_data).unwrap();
        let ref_bt = ref_cfb.read_body_text_section(0, ref_parsed.header.compressed, false).unwrap();
        let ref_recs = Record::read_all(&ref_bt).unwrap();

        eprintln!("\n  --- 레코드 비교 (참조={} 개, 저장={} 개) ---", ref_recs.len(), saved_recs.len());
        let max_recs = ref_recs.len().max(saved_recs.len());
        let mut diff_count = 0;
        for i in 0..max_recs {
            let ref_info = if i < ref_recs.len() {
                let r = &ref_recs[i];
                format!("tag={:3}({:22}) lv={} sz={}", r.tag_id, tags::tag_name(r.tag_id), r.level, r.data.len())
            } else { "---".to_string() };
            let saved_info = if i < saved_recs.len() {
                let r = &saved_recs[i];
                format!("tag={:3}({:22}) lv={} sz={}", r.tag_id, tags::tag_name(r.tag_id), r.level, r.data.len())
            } else { "---".to_string() };
            let match_mark = if i < ref_recs.len() && i < saved_recs.len() {
                let r = &ref_recs[i];
                let s = &saved_recs[i];
                if r.tag_id == s.tag_id && r.level == s.level && r.data == s.data { "==" }
                else if r.tag_id == s.tag_id && r.level == s.level { "~=" }
                else { "!=" }
            } else { "!=" };
            if match_mark != "==" { diff_count += 1; }
            eprintln!("  [{:2}] {} {} | {}", i, match_mark, ref_info, saved_info);
        }

        // 차이 상세
        for i in 0..ref_recs.len().min(saved_recs.len()) {
            let r = &ref_recs[i];
            let s = &saved_recs[i];
            if r.tag_id == s.tag_id && r.data != s.data {
                eprintln!("\n  [차이 상세] 레코드 {}: {}", i, tags::tag_name(r.tag_id));
                let max_show = r.data.len().max(s.data.len()).min(120);
                eprintln!("    참조: {:02x?}", &r.data[..r.data.len().min(max_show)]);
                eprintln!("    저장: {:02x?}", &s.data[..s.data.len().min(max_show)]);
                for j in 0..r.data.len().min(s.data.len()) {
                    if r.data[j] != s.data[j] {
                        eprintln!("    첫 차이: offset {} (참조=0x{:02x}, 저장=0x{:02x})", j, r.data[j], s.data[j]);
                        break;
                    }
                }
            }
        }

        eprintln!("  일치: {}/{} 레코드", max_recs - diff_count, max_recs);

        // CFB 스트림 확인
        let streams = saved_cfb.list_streams();
        let has_bindata = streams.iter().any(|s| s.contains("BinData") || s.contains("BIN"));
        assert!(has_bindata, "BinData 스트림이 없음");

        eprintln!("\n=== 표 안 이미지 저장 검증 완료 ===");
    }

    /// 타스크 41 단계 1: 기존 HWP에 프로그래밍 방식으로 2×2 표 삽입 → 저장
    /// 직렬화 코드 자체의 정상 동작을 먼저 확인
    #[test]
    fn test_inject_table_into_existing() {
        use crate::model::table::{Table, Cell};
        use crate::model::control::Control;
        use crate::model::Padding;
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  타스크 41 단계 1: 기존 HWP에 2×2 표 삽입");
        eprintln!("{}", "=".repeat(60));

        let orig_data = std::fs::read(path).unwrap();

        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        let sec = &doc.document.sections[0];
        let orig_para_count = sec.paragraphs.len();
        eprintln!("  원본: {} 문단, {} 컨트롤",
            orig_para_count,
            sec.paragraphs.iter().map(|p| p.controls.len()).sum::<usize>());

        // 캐럿 위치 확인
        let caret_list_id = doc.document.doc_properties.caret_list_id;
        let caret_para_id = doc.document.doc_properties.caret_para_id;
        let caret_char_pos = doc.document.doc_properties.caret_char_pos;
        eprintln!("  캐럿 위치: list_id={}, para_id={}, char_pos={}",
            caret_list_id, caret_para_id, caret_char_pos);

        // 삽입 위치: 캐럿이 가리키는 문단
        let insert_para_idx = caret_para_id as usize;
        assert!(insert_para_idx < orig_para_count,
            "캐럿 para_id({})가 문단 범위({})를 초과", insert_para_idx, orig_para_count);
        eprintln!("  삽입 위치: 문단[{}] (캐럿 기반)", insert_para_idx);

        // 삽입 위치 근처 문단 구조 출력
        let start = if insert_para_idx > 2 { insert_para_idx - 2 } else { 0 };
        let end = (insert_para_idx + 4).min(orig_para_count);
        for i in start..end {
            let p = &sec.paragraphs[i];
            let ctrl_types: Vec<&str> = p.controls.iter().map(|c| match c {
                Control::Table(_) => "Table",
                Control::Picture(_) => "Picture",
                _ => "Other",
            }).collect();
            let marker = if i == insert_para_idx { " ← 캐럿" } else { "" };
            eprintln!("    문단[{}]: cc={} mask=0x{:08X} text='{}' ctrls={:?}{}",
                i, p.char_count, p.control_mask,
                if p.text.len() > 30 { &p.text[..30] } else { &p.text },
                ctrl_types, marker);
        }

        // === 방법: 기존 표 문단을 복제하여 삽입 (직렬화 문제 격리) ===
        // 문단[2]의 표를 그대로 복제
        let source_para_idx = 2;
        let table_para = doc.document.sections[0].paragraphs[source_para_idx].clone();
        eprintln!("  복제 원본: 문단[{}] cc={} controls={}",
            source_para_idx, table_para.char_count, table_para.controls.len());
        if let Some(Control::Table(t)) = table_para.controls.first() {
            eprintln!("    표: {}×{} cells={} attr=0x{:08X}",
                t.row_count, t.col_count, t.cells.len(), t.attr);
        }

        // 캐럿 위치 뒤에 표 문단 삽입
        doc.document.sections[0].paragraphs.insert(insert_para_idx + 1, table_para);

        // 기존 콘텐츠 사이 삽입 → 빈 문단 불필요 (기존 문단이 이어짐)

        // raw_stream 무효화: 섹션만 (DocInfo raw 유지 → 손상 방지)
        doc.document.sections[0].raw_stream = None;

        eprintln!("  수정: {} 문단 (원본 {} + 표 문단 1개)",
            doc.document.sections[0].paragraphs.len(), orig_para_count);

        // 저장
        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "HWP 저장 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/save_test_table_inject.hwp", &saved_data).unwrap();
        eprintln!("  저장: output/save_test_table_inject.hwp ({} bytes)", saved_data.len());

        // 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();

        // 문단 수 검증 (표 문단 = +1)
        let new_para_count = doc2.document.sections[0].paragraphs.len();
        eprintln!("  재파싱: {} 문단", new_para_count);
        assert_eq!(new_para_count, orig_para_count + 1, "문단 수 불일치");

        // 삽입된 표 검증 (캐럿 문단 다음 위치)
        let table_para_idx = insert_para_idx + 1;
        let injected = &doc2.document.sections[0].paragraphs[table_para_idx];
        let table_found = injected.controls.iter().any(|c| matches!(c, Control::Table(_)));
        assert!(table_found, "삽입된 표 컨트롤이 없음 (문단[{}])", table_para_idx);

        if let Some(Control::Table(t)) = injected.controls.iter().find(|c| matches!(c, Control::Table(_))) {
            eprintln!("  복제 표: {}×{} cells={} attr=0x{:08X}",
                t.row_count, t.col_count, t.cells.len(), t.attr);
        }

        // 기존 컨트롤 보존 검증
        let orig_doc = HwpDocument::from_bytes(&orig_data).unwrap();
        let mut orig_tables = 0;
        let mut orig_pics = 0;
        for para in &orig_doc.document.sections[0].paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(_) => orig_tables += 1,
                    Control::Picture(_) => orig_pics += 1,
                    _ => {}
                }
            }
        }
        let mut new_tables = 0;
        let mut new_pics = 0;
        for para in &doc2.document.sections[0].paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(_) => new_tables += 1,
                    Control::Picture(_) => new_pics += 1,
                    _ => {}
                }
            }
        }
        eprintln!("  컨트롤 보존: Table {}→{}, Picture {}→{}",
            orig_tables, new_tables, orig_pics, new_pics);
        assert_eq!(new_tables, orig_tables + 1, "표 개수 불일치");
        assert_eq!(new_pics, orig_pics, "이미지 개수 변경됨");

        eprintln!("\n=== 타스크 41 단계 1 완료 ===");

        // === 진단: 저장된 파일에서 삽입된 표 제거 후 재저장 ===
        eprintln!("\n  [진단] 표 제거 후 재저장...");
        let mut doc3 = HwpDocument::from_bytes(&saved_data).unwrap();
        let para_count_before = doc3.document.sections[0].paragraphs.len();
        // 삽입된 표 문단 제거 (index = insert_para_idx + 1 = 9)
        doc3.document.sections[0].paragraphs.remove(insert_para_idx + 1);
        doc3.document.sections[0].raw_stream = None;
        let saved3 = doc3.export_hwp_native().unwrap();
        std::fs::write("output/save_test_table_removed.hwp", &saved3).unwrap();
        eprintln!("  [진단] 표 제거: {} → {} 문단, output/save_test_table_removed.hwp ({} bytes)",
            para_count_before, doc3.document.sections[0].paragraphs.len(), saved3.len());
    }

    /// 진단: 복제 표 vs parse_table_html 표의 raw_ctrl_data 및 직렬화 바이트 비교
    #[test]
    fn test_diag_clone_vs_parsed_table() {
        use crate::model::control::Control;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  진단: 복제 표 vs parse_table_html 표 비교");
        eprintln!("{}", "=".repeat(60));

        let orig_data = std::fs::read(path).unwrap();

        // === A: 복제 표 (정상 동작) ===
        let doc_a = HwpDocument::from_bytes(&orig_data).unwrap();
        let clone_para = doc_a.document.sections[0].paragraphs[2].clone();

        // === B: parse_table_html 표 (내용 사라짐) ===
        let mut doc_b = HwpDocument::from_bytes(&orig_data).unwrap();
        let table_html = r#"<table><tr><td style="border:1px solid black;">테스트A</td><td style="border:1px solid black;">&nbsp;</td></tr><tr><td style="border:1px solid black;">&nbsp;</td><td style="border:1px solid black;">테스트D</td></tr></table>"#;
        let mut parsed_paras = Vec::new();
        doc_b.parse_table_html(&mut parsed_paras, table_html);
        let parsed_para = &parsed_paras[0];

        // 문단 헤더 비교
        eprintln!("\n  [문단 헤더 비교]");
        eprintln!("  복제: cc={} msb={} cm=0x{:08X} ps={} sid={} rhe={:02x?}",
            clone_para.char_count, clone_para.char_count_msb,
            clone_para.control_mask, clone_para.para_shape_id, clone_para.style_id,
            &clone_para.raw_header_extra);
        eprintln!("  생성: cc={} msb={} cm=0x{:08X} ps={} sid={} rhe={:02x?}",
            parsed_para.char_count, parsed_para.char_count_msb,
            parsed_para.control_mask, parsed_para.para_shape_id, parsed_para.style_id,
            &parsed_para.raw_header_extra);

        // raw_ctrl_data 비교
        if let Some(Control::Table(ref t_a)) = clone_para.controls.first() {
            if let Some(Control::Table(ref t_b)) = parsed_para.controls.first() {
                eprintln!("\n  [table.attr 비교]");
                eprintln!("  복제: attr=0x{:08X}", t_a.attr);
                eprintln!("  생성: attr=0x{:08X}", t_b.attr);

                eprintln!("\n  [raw_ctrl_data 비교] (CommonObjAttr after attr)");
                eprintln!("  복제 ({} bytes): {:02x?}", t_a.raw_ctrl_data.len(), &t_a.raw_ctrl_data);
                eprintln!("  생성 ({} bytes): {:02x?}", t_b.raw_ctrl_data.len(), &t_b.raw_ctrl_data);

                // 필드별 해석
                fn read_i32(d: &[u8], o: usize) -> i32 {
                    if o + 4 <= d.len() { i32::from_le_bytes([d[o],d[o+1],d[o+2],d[o+3]]) } else { 0 }
                }
                fn read_u32(d: &[u8], o: usize) -> u32 {
                    if o + 4 <= d.len() { u32::from_le_bytes([d[o],d[o+1],d[o+2],d[o+3]]) } else { 0 }
                }
                fn read_i16(d: &[u8], o: usize) -> i16 {
                    if o + 2 <= d.len() { i16::from_le_bytes([d[o],d[o+1]]) } else { 0 }
                }
                fn read_u16(d: &[u8], o: usize) -> u16 {
                    if o + 2 <= d.len() { u16::from_le_bytes([d[o],d[o+1]]) } else { 0 }
                }

                for (label, d) in [("복제", t_a.raw_ctrl_data.as_slice()), ("생성", t_b.raw_ctrl_data.as_slice())] {
                    eprintln!("\n  [{}] CommonObjAttr 필드:", label);
                    eprintln!("    [0..4] vert_offset  = {}", read_i32(d, 0));
                    eprintln!("    [4..8] horz_offset  = {}", read_i32(d, 4));
                    eprintln!("    [8..12] width       = {}", read_u32(d, 8));
                    eprintln!("    [12..16] height     = {}", read_u32(d, 12));
                    eprintln!("    [16..20] z_order    = {}", read_i32(d, 16));
                    eprintln!("    [20..22] margin_l   = {}", read_i16(d, 20));
                    eprintln!("    [22..24] margin_r   = {}", read_i16(d, 22));
                    eprintln!("    [24..26] margin_t   = {}", read_i16(d, 24));
                    eprintln!("    [26..28] margin_b   = {}", read_i16(d, 26));
                    eprintln!("    [28..32] inst_id    = 0x{:08X}", read_u32(d, 28));
                    eprintln!("    [32..34] desc_len   = {}", read_u16(d, 32));
                    if d.len() > 34 {
                        eprintln!("    [34..] extra        = {:02x?}", &d[34..]);
                    }
                }

                // 직렬화 바이트 비교 (CTRL_HEADER + TABLE + cells)
                eprintln!("\n  [직렬화 레코드 비교]");
                let mut recs_a: Vec<Record> = Vec::new();
                crate::serializer::control::serialize_control(
                    &clone_para.controls[0], 1, None, &mut recs_a);
                let mut recs_b: Vec<Record> = Vec::new();
                crate::serializer::control::serialize_control(
                    &parsed_para.controls[0], 1, None, &mut recs_b);

                eprintln!("  복제: {} 레코드, 생성: {} 레코드", recs_a.len(), recs_b.len());

                // 처음 5개 레코드 비교
                let max_show = recs_a.len().max(recs_b.len()).min(10);
                for i in 0..max_show {
                    let a_info = if i < recs_a.len() {
                        format!("{:22} lv={} sz={}", tags::tag_name(recs_a[i].tag_id), recs_a[i].level, recs_a[i].data.len())
                    } else { "---".to_string() };
                    let b_info = if i < recs_b.len() {
                        format!("{:22} lv={} sz={}", tags::tag_name(recs_b[i].tag_id), recs_b[i].level, recs_b[i].data.len())
                    } else { "---".to_string() };
                    let status = if i < recs_a.len() && i < recs_b.len() {
                        if recs_a[i].tag_id == recs_b[i].tag_id && recs_a[i].data == recs_b[i].data { "==" }
                        else if recs_a[i].tag_id == recs_b[i].tag_id { "~=" }
                        else { "!=" }
                    } else { "!=" };
                    eprintln!("  [{:2}] {} | {} | {}", i, status, a_info, b_info);

                    // 데이터 차이 상세
                    if i < recs_a.len() && i < recs_b.len() && recs_a[i].tag_id == recs_b[i].tag_id && recs_a[i].data != recs_b[i].data {
                        let max_d = recs_a[i].data.len().max(recs_b[i].data.len()).min(80);
                        eprintln!("       복제: {:02x?}", &recs_a[i].data[..recs_a[i].data.len().min(max_d)]);
                        eprintln!("       생성: {:02x?}", &recs_b[i].data[..recs_b[i].data.len().min(max_d)]);
                    }
                }

                // raw_table_record_attr 비교
                eprintln!("\n  [TABLE record attr 비교]");
                eprintln!("  복제: tbl_rec_attr=0x{:08X}", t_a.raw_table_record_attr);
                eprintln!("  생성: tbl_rec_attr=0x{:08X}", t_b.raw_table_record_attr);
            }
        }

        // === 전체 문단 직렬화 비교 (PARA_HEADER + PARA_TEXT + ... + CTRL_HEADER + ...) ===
        eprintln!("\n  [전체 문단 직렬화 비교]");
        let mut full_recs_a: Vec<Record> = Vec::new();
        crate::serializer::body_text::serialize_paragraph_list(
            std::slice::from_ref(&clone_para), 0, &mut full_recs_a);
        let mut full_recs_b: Vec<Record> = Vec::new();
        crate::serializer::body_text::serialize_paragraph_list(
            std::slice::from_ref(parsed_para), 0, &mut full_recs_b);

        eprintln!("  복제 전체: {} 레코드, 생성 전체: {} 레코드", full_recs_a.len(), full_recs_b.len());
        let max = full_recs_a.len().max(full_recs_b.len());
        for i in 0..max {
            let a = full_recs_a.get(i);
            let b = full_recs_b.get(i);
            let a_info = a.map(|r| format!("{:22} lv={} sz={}", tags::tag_name(r.tag_id), r.level, r.data.len()))
                .unwrap_or_else(|| "---".to_string());
            let b_info = b.map(|r| format!("{:22} lv={} sz={}", tags::tag_name(r.tag_id), r.level, r.data.len()))
                .unwrap_or_else(|| "---".to_string());
            let status = match (a, b) {
                (Some(ra), Some(rb)) if ra.tag_id == rb.tag_id && ra.data == rb.data => "==",
                (Some(ra), Some(rb)) if ra.tag_id == rb.tag_id => "~=",
                _ => "!!",
            };
            eprintln!("  [{:2}] {} | {} | {}", i, status, a_info, b_info);

            // PARA_HEADER와 PARA_TEXT 데이터 상세
            if let (Some(ra), Some(rb)) = (a, b) {
                if ra.tag_id == rb.tag_id && ra.data != rb.data {
                    if ra.tag_id == tags::HWPTAG_PARA_HEADER || ra.tag_id == tags::HWPTAG_PARA_TEXT {
                        eprintln!("       복제: {:02x?}", &ra.data[..ra.data.len().min(60)]);
                        eprintln!("       생성: {:02x?}", &rb.data[..rb.data.len().min(60)]);
                    }
                }
            }
        }

        eprintln!("\n=== 진단 완료 ===");
    }

    /// 타스크 41 단계 3: parse_table_html()로 생성한 표를 기존 문서에 삽입 → 저장 → 검증
    /// DIFF-1~8 수정 사항이 모두 반영된 통합 테스트
    #[test]
    fn test_parse_table_html_save() {
        use crate::model::control::Control;

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  타스크 41 단계 3: parse_table_html 표 삽입 저장 검증");
        eprintln!("{}", "=".repeat(60));

        let orig_data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&orig_data).unwrap();

        let orig_para_count = doc.document.sections[0].paragraphs.len();
        let caret_para_id = doc.document.doc_properties.caret_para_id as usize;
        eprintln!("  원본: {} 문단, 캐럿 위치: 문단[{}]", orig_para_count, caret_para_id);

        // HTML 표 생성 (2×2, 빈 셀 포함)
        let table_html = r#"<table style="border-collapse:collapse;">
            <tr>
                <td style="border:1px solid black; padding:5px; width:200px;">테스트 셀 A</td>
                <td style="border:1px solid black; padding:5px; width:200px;">&nbsp;</td>
            </tr>
            <tr>
                <td style="border:1px solid black; padding:5px;">&nbsp;&nbsp;</td>
                <td style="border:1px solid black; padding:5px;">테스트 셀 D</td>
            </tr>
        </table>"#;

        // parse_table_html으로 표 문단 생성
        let mut table_paragraphs = Vec::new();
        doc.parse_table_html(&mut table_paragraphs, table_html);
        assert_eq!(table_paragraphs.len(), 1, "표 문단 1개 생성");

        let table_para = &table_paragraphs[0];
        eprintln!("  표 문단: cc={} msb={} cm=0x{:08X} cs={} ls={}",
            table_para.char_count, table_para.char_count_msb,
            table_para.control_mask, table_para.char_shapes.len(), table_para.line_segs.len());

        // DIFF 검증
        if let Some(Control::Table(ref tbl)) = table_para.controls.first() {
            eprintln!("  표: {}×{} cells={} attr=0x{:08X}", tbl.row_count, tbl.col_count, tbl.cells.len(), tbl.attr);
            eprintln!("  DIFF-5: tbl_rec_attr=0x{:08X}", tbl.raw_table_record_attr);
            assert_eq!(tbl.raw_table_record_attr, 0x04000006, "DIFF-5: 셀분리금지 항상 설정");

            // DIFF-7: instance_id
            let inst = u32::from_le_bytes([
                tbl.raw_ctrl_data[28], tbl.raw_ctrl_data[29],
                tbl.raw_ctrl_data[30], tbl.raw_ctrl_data[31],
            ]);
            eprintln!("  DIFF-7: instance_id=0x{:08X}", inst);
            assert_ne!(inst, 0, "DIFF-7: instance_id != 0");

            // DIFF-1: 빈 셀 검증
            for (i, cell) in tbl.cells.iter().enumerate() {
                let p = &cell.paragraphs[0];
                eprintln!("  셀[{}]({},{}): cc={} text='{}' cs={} ls={} has_pt={}",
                    i, cell.row, cell.col, p.char_count,
                    if p.text.len() > 20 { &p.text[..20] } else { &p.text },
                    p.char_shapes.len(), p.line_segs.len(), p.has_para_text);

                // DIFF-2: 모든 셀 문단은 char_shapes가 있어야 함
                assert!(!p.char_shapes.is_empty(), "DIFF-2: 셀[{}] char_shapes 비어있음", i);
                // DIFF-3: para_shape_id=0 (기본 본문 스타일)
                assert_eq!(p.para_shape_id, 0, "DIFF-3: 셀[{}] para_shape_id=0", i);
                // DIFF-6: line_segs의 tag
                if !p.line_segs.is_empty() {
                    assert_eq!(p.line_segs[0].tag, 0x00060000, "DIFF-6: 셀[{}] line_seg tag", i);
                    assert!(p.line_segs[0].segment_width > 0, "DIFF-6: 셀[{}] seg_width > 0", i);
                }
            }

            // DIFF-1: 빈 셀 (셀[1], 셀[2]) 확인
            assert_eq!(tbl.cells[1].paragraphs[0].char_count, 1, "DIFF-1: 빈 셀[1] cc=1");
            assert!(tbl.cells[1].paragraphs[0].text.is_empty(), "DIFF-1: 빈 셀[1] text empty");
            assert_eq!(tbl.cells[2].paragraphs[0].char_count, 1, "DIFF-1: 빈 셀[2] cc=1");
        }

        // DIFF-8: 표 컨테이너 문단 LineSeg
        assert!(!table_para.line_segs.is_empty(), "DIFF-8: 표 문단 line_segs 비어있음");
        eprintln!("  DIFF-8: line_seg h={} tw={} seg_w={} tag=0x{:08X}",
            table_para.line_segs[0].line_height,
            table_para.line_segs[0].text_height,
            table_para.line_segs[0].segment_width,
            table_para.line_segs[0].tag);
        assert!(table_para.line_segs[0].line_height > 0, "DIFF-8: line_height > 0");
        assert!(table_para.line_segs[0].segment_width > 0, "DIFF-8: seg_width > 0");
        assert_eq!(table_para.line_segs[0].tag, 0x00060000, "DIFF-8: tag=0x00060000");

        // 삽입 및 저장
        doc.document.sections[0].paragraphs.insert(caret_para_id + 1, table_paragraphs.remove(0));
        doc.document.sections[0].raw_stream = None;

        let saved = doc.export_hwp_native();
        assert!(saved.is_ok(), "저장 실패: {:?}", saved.err());
        let saved_data = saved.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/save_test_parsed_table.hwp", &saved_data).unwrap();
        eprintln!("  저장: output/save_test_parsed_table.hwp ({} bytes)", saved_data.len());

        // 재파싱 검증
        let doc2 = HwpDocument::from_bytes(&saved_data);
        assert!(doc2.is_ok(), "재파싱 실패: {:?}", doc2.err());
        let doc2 = doc2.unwrap();
        let new_para_count = doc2.document.sections[0].paragraphs.len();
        eprintln!("  재파싱: {} 문단 (원본 {} + 1)", new_para_count, orig_para_count);
        assert_eq!(new_para_count, orig_para_count + 1);

        // 삽입된 표 확인
        let injected = &doc2.document.sections[0].paragraphs[caret_para_id + 1];
        assert!(injected.controls.iter().any(|c| matches!(c, Control::Table(_))),
            "삽입된 표 컨트롤 없음");

        eprintln!("\n=== 타스크 41 단계 3 완료 ===");
        eprintln!("  output/save_test_parsed_table.hwp 를 HWP 프로그램에서 확인해 주세요");
    }

    /// 진단: k-water-rfp.hwp 전체 문단의 char_count_msb 패턴 분석
    #[test]
    fn test_diag_msb_pattern_kwater() {
        use crate::model::control::Control;

        let path = "samples/k-water-rfp.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("\n{}", "=".repeat(70));
        eprintln!("  k-water-rfp.hwp MSB 패턴 분석");
        eprintln!("{}", "=".repeat(70));

        for (si, section) in doc.document.sections.iter().enumerate() {
            let para_count = section.paragraphs.len();
            eprintln!("\n  Section {} ({} paragraphs)", si, para_count);
            eprintln!("  {:>4} | {:>5} | {:>3} | {:>5} | {:>3} | {:>8} | {}",
                "idx", "cc", "msb", "psid", "sid", "ctrl", "text_preview");
            eprintln!("  {}", "-".repeat(65));

            for (pi, para) in section.paragraphs.iter().enumerate() {
                let is_last = pi == para_count - 1;
                let ctrl_info = if para.controls.is_empty() {
                    String::new()
                } else {
                    let ctrl_names: Vec<&str> = para.controls.iter().map(|c| match c {
                        Control::Table(_) => "TABLE",
                        Control::SectionDef(_) => "SECD",
                        Control::ColumnDef(_) => "COLD",
                        Control::Shape(_) => "SHAPE",
                        Control::Picture(_) => "PIC",
                        _ => "OTHER",
                    }).collect();
                    ctrl_names.join(",")
                };

                let text_preview: String = para.text.chars().take(30).collect();
                let msb_mark = if para.char_count_msb { "T" } else { "F" };
                let last_mark = if is_last { " <LAST>" } else { "" };

                eprintln!("  {:>4} | {:>5} | {:>3} | {:>5} | {:>3} | {:>8} | {}{}",
                    pi, para.char_count, msb_mark,
                    para.para_shape_id, para.style_id,
                    ctrl_info, text_preview, last_mark);

                // 컨트롤 내부 문단도 출력
                for ctrl in &para.controls {
                    match ctrl {
                        Control::Table(tbl) => {
                            for (ci, cell) in tbl.cells.iter().enumerate() {
                                for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                    let cp_msb = if cp.char_count_msb { "T" } else { "F" };
                                    let cp_last = cpi == cell.paragraphs.len() - 1;
                                    let cp_text: String = cp.text.chars().take(20).collect();
                                    eprintln!("        cell[{}].p[{}]: cc={} msb={} psid={} sid={} text='{}'{}",
                                        ci, cpi, cp.char_count, cp_msb,
                                        cp.para_shape_id, cp.style_id, cp_text,
                                        if cp_last { " <CELL_LAST>" } else { "" });
                                }
                            }
                        }
                        Control::Shape(s) => {
                            // ShapeObject enum 에서 drawing.text_box 접근
                            let tb_opt = match s.as_ref() {
                                crate::model::shape::ShapeObject::Line(l) => l.drawing.text_box.as_ref(),
                                crate::model::shape::ShapeObject::Rectangle(r) => r.drawing.text_box.as_ref(),
                                crate::model::shape::ShapeObject::Ellipse(e) => e.drawing.text_box.as_ref(),
                                crate::model::shape::ShapeObject::Arc(a) => a.drawing.text_box.as_ref(),
                                crate::model::shape::ShapeObject::Polygon(p) => p.drawing.text_box.as_ref(),
                                crate::model::shape::ShapeObject::Curve(c) => c.drawing.text_box.as_ref(),
                                _ => None,
                            };
                            if let Some(tb) = tb_opt {
                                for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                    let tp_msb = if tp.char_count_msb { "T" } else { "F" };
                                    let tp_last = tpi == tb.paragraphs.len() - 1;
                                    let tp_text: String = tp.text.chars().take(20).collect();
                                    eprintln!("        textbox.p[{}]: cc={} msb={} psid={} text='{}'{}",
                                        tpi, tp.char_count, tp_msb,
                                        tp.para_shape_id, tp_text,
                                        if tp_last { " <TB_LAST>" } else { "" });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 통계 집계
        eprintln!("\n{}", "=".repeat(70));
        eprintln!("  MSB 패턴 통계");
        eprintln!("{}", "=".repeat(70));

        for (si, section) in doc.document.sections.iter().enumerate() {
            let para_count = section.paragraphs.len();
            let mut msb_true_count = 0;
            let mut msb_false_count = 0;
            let mut last_para_msb = false;
            let mut mid_para_msb_true = Vec::new(); // MSB=T인 중간 문단

            for (pi, para) in section.paragraphs.iter().enumerate() {
                if para.char_count_msb {
                    msb_true_count += 1;
                    if pi < para_count - 1 {
                        mid_para_msb_true.push(pi);
                    }
                } else {
                    msb_false_count += 1;
                }
                if pi == para_count - 1 {
                    last_para_msb = para.char_count_msb;
                }
            }

            eprintln!("  Section {}: total={} MSB_T={} MSB_F={} last_msb={}",
                si, para_count, msb_true_count, msb_false_count,
                if last_para_msb { "T" } else { "F" });

            if !mid_para_msb_true.is_empty() {
                eprintln!("    ** 중간 문단에서 MSB=T: {:?}", mid_para_msb_true);
                for &pi in &mid_para_msb_true {
                    let para = &section.paragraphs[pi];
                    let ctrl_info: Vec<&str> = para.controls.iter().map(|c| match c {
                        Control::Table(_) => "TABLE",
                        Control::Shape(_) => "SHAPE",
                        Control::Picture(_) => "PIC",
                        Control::SectionDef(_) => "SECD",
                        _ => "OTHER",
                    }).collect();
                    eprintln!("    para[{}]: cc={} ctrl=[{}] psid={} sid={}",
                        pi, para.char_count, ctrl_info.join(","),
                        para.para_shape_id, para.style_id);
                }
            }
        }
    }

    /// 엔터 2회 후 저장 시 파일 손상 재현 진단 테스트
    #[test]
    fn test_diag_double_enter_save() {
        use crate::parser::record::Record;
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::tags;

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("=== 엔터 2회 후 저장 파일 손상 진단 ===");
        let section = &doc.document.sections[0];
        eprintln!("원본 문단 수: {}", section.paragraphs.len());

        // 텍스트가 있고 컨트롤이 없는 문단 찾기
        let mut target_para = 0;
        for (i, p) in section.paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text_len={} cc={} ctrl={} has_pt={}",
                i, p.text.chars().count(), p.char_count, p.controls.len(), p.has_para_text);
            if p.text.chars().count() >= 10 && p.controls.is_empty() && target_para == 0 {
                target_para = i;
            }
        }
        assert!(target_para > 0, "텍스트가 있는 문단을 찾을 수 없음");
        let para = &section.paragraphs[target_para];
        let text_len = para.text.chars().count();
        eprintln!("\n대상 문단[{}]: text_len={} cc={} controls={} has_para_text={}",
            target_para, text_len, para.char_count, para.controls.len(), para.has_para_text);
        eprintln!("  text(앞40)='{}'", para.text.chars().take(40).collect::<String>());

        let split_offset = 4; // 4번째 글자 뒤에서 분할 (사용자 시나리오)

        // === 엔터 1회 ===
        let result1 = doc.split_paragraph_native(0, target_para, split_offset);
        assert!(result1.is_ok(), "1차 분할 실패: {:?}", result1.err());
        eprintln!("\n--- 1차 분할 (offset={}) ---", split_offset);

        let section = &doc.document.sections[0];
        for i in target_para..=(target_para+1).min(section.paragraphs.len()-1) {
            let p = &section.paragraphs[i];
            eprintln!("  문단[{}]: cc={} text_len={} controls={} has_para_text={} line_segs={}",
                i, p.char_count, p.text.chars().count(), p.controls.len(), p.has_para_text, p.line_segs.len());
        }

        // 1회 분할 후 저장 테스트
        let saved1 = doc.export_hwp_native();
        assert!(saved1.is_ok(), "1차 저장 실패");
        let saved1_data = saved1.unwrap();
        let parse1 = HwpDocument::from_bytes(&saved1_data);
        eprintln!("1회 분할 후 저장+재파싱: {}", if parse1.is_ok() { "성공" } else { "실패" });

        // === 엔터 2회 (새 문단의 시작에서 다시 분할) ===
        let new_para_idx = target_para + 1;
        let result2 = doc.split_paragraph_native(0, new_para_idx, 0);
        assert!(result2.is_ok(), "2차 분할 실패: {:?}", result2.err());
        eprintln!("\n--- 2차 분할 (문단[{}], offset=0) ---", new_para_idx);

        let section = &doc.document.sections[0];
        eprintln!("문단 수: {}", section.paragraphs.len());
        for i in target_para..=(target_para+2).min(section.paragraphs.len()-1) {
            let p = &section.paragraphs[i];
            eprintln!("  문단[{}]: cc={} text_len={} controls={} has_para_text={} raw_extra_len={}",
                i, p.char_count, p.text.chars().count(), p.controls.len(), p.has_para_text, p.raw_header_extra.len());
        }

        // 2회 분할 후 저장 테스트
        let saved2 = doc.export_hwp_native();
        assert!(saved2.is_ok(), "2차 저장 실패");
        let saved2_data = saved2.unwrap();

        let _ = std::fs::create_dir_all("output");
        std::fs::write("output/diag_double_enter.hwp", &saved2_data).unwrap();
        eprintln!("\noutput/diag_double_enter.hwp 저장 ({} bytes)", saved2_data.len());

        // 재파싱 테스트
        let parse2 = HwpDocument::from_bytes(&saved2_data);
        eprintln!("2회 분할 후 저장+재파싱: {}", if parse2.is_ok() { "성공" } else { "실패" });

        // 직렬화된 Section0 레코드 분석 - 분할 영역 주변만 상세 출력
        eprintln!("\n=== Section0 직렬화 레코드 분석 (level 0만, 분할 영역) ===");
        let section_bytes = crate::serializer::body_text::serialize_section(
            &doc.document.sections[0]);
        let recs = Record::read_all(&section_bytes).unwrap();
        let mut top_para_idx = 0;
        for (ri, rec) in recs.iter().enumerate() {
            if rec.tag_id == tags::HWPTAG_PARA_HEADER && rec.level == 0 {
                let cc_raw = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                let cc = cc_raw & 0x7FFFFFFF;
                let msb = cc_raw & 0x80000000 != 0;
                let ctrl_mask = u32::from_le_bytes(rec.data[4..8].try_into().unwrap());
                // 분할 영역 (target_para-1 ~ target_para+4) 표시
                if top_para_idx >= target_para.saturating_sub(1) && top_para_idx <= target_para + 4 {
                    eprintln!("rec[{}] PARA_HEADER(L0): model_para={} cc={} msb={} ctrl=0x{:08X}",
                        ri, top_para_idx, cc, msb, ctrl_mask);
                }
                top_para_idx += 1;
            } else if rec.tag_id == tags::HWPTAG_PARA_TEXT && rec.level == 1 {
                // 바로 앞의 PARA_HEADER가 분할 영역이면 표시
                if top_para_idx > target_para.saturating_sub(1) && top_para_idx <= target_para + 5 {
                    let code_units = rec.data.len() / 2;
                    eprintln!("rec[{}]   PARA_TEXT(L1): {} code_units ({} bytes)",
                        ri, code_units, rec.data.len());
                }
            } else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE && rec.level == 1 {
                if top_para_idx > target_para.saturating_sub(1) && top_para_idx <= target_para + 5 {
                    let entries = rec.data.len() / 8;
                    eprintln!("rec[{}]   PARA_CHAR_SHAPE(L1): {} entries", ri, entries);
                }
            } else if rec.tag_id == tags::HWPTAG_PARA_LINE_SEG && rec.level == 1 {
                if top_para_idx > target_para.saturating_sub(1) && top_para_idx <= target_para + 5 {
                    let entries = rec.data.len() / 36;
                    eprintln!("rec[{}]   PARA_LINE_SEG(L1): {} entries", ri, entries);
                }
            }
        }
        eprintln!("총 top-level 문단: {}", top_para_idx);

        if parse2.is_err() {
            panic!("2회 분할 후 저장된 파일 재파싱 실패!");
        }
    }

    #[test]
    fn test_textbox_render_tree_debug() {
        use std::path::Path;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = Path::new("samples/img-start-001.hwp");
        if !path.exists() {
            eprintln!("img-start-001.hwp 없음 — 건너뜀");
            return;
        }

        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();
        doc.convert_to_editable_native();

        // 문서 구조 확인: Shape 컨트롤 찾기
        let mut shape_found = false;
        for (si, sec) in doc.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(shape) = ctrl {
                        let has_textbox = match shape.as_ref() {
                            crate::model::shape::ShapeObject::Rectangle(r) => r.drawing.text_box.is_some(),
                            crate::model::shape::ShapeObject::Ellipse(e) => e.drawing.text_box.is_some(),
                            crate::model::shape::ShapeObject::Polygon(p) => p.drawing.text_box.is_some(),
                            crate::model::shape::ShapeObject::Curve(c) => c.drawing.text_box.is_some(),
                            _ => false,
                        };
                        if has_textbox {
                            let tb = get_textbox_from_shape(shape.as_ref()).unwrap();
                            let drawing = match shape.as_ref() {
                                crate::model::shape::ShapeObject::Rectangle(r) => Some(&r.drawing),
                                crate::model::shape::ShapeObject::Ellipse(e) => Some(&e.drawing),
                                crate::model::shape::ShapeObject::Polygon(p) => Some(&p.drawing),
                                crate::model::shape::ShapeObject::Curve(c) => Some(&c.drawing),
                                _ => None,
                            };
                            eprintln!("Shape 발견: sec={} para={} ctrl={} type={:?} textbox_paras={}",
                                si, pi, ci,
                                match shape.as_ref() {
                                    crate::model::shape::ShapeObject::Rectangle(_) => "Rectangle",
                                    crate::model::shape::ShapeObject::Ellipse(_) => "Ellipse",
                                    crate::model::shape::ShapeObject::Polygon(_) => "Polygon",
                                    crate::model::shape::ShapeObject::Curve(_) => "Curve",
                                    _ => "Other",
                                },
                                tb.paragraphs.len(),
                            );
                            if let Some(d) = drawing {
                                eprintln!("  fill_type={:?}", d.fill.fill_type);
                                let sa = &d.shape_attr;
                                eprintln!("  shape_attr: orig_w={} orig_h={} cur_w={} cur_h={}",
                                    sa.original_width, sa.original_height, sa.current_width, sa.current_height);
                                if let Some(ref tb) = d.text_box {
                                    eprintln!("  textbox margins: left={} right={} top={} bottom={} max_w={}",
                                        tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom, tb.max_width);
                                }
                                if let Some(ref g) = d.fill.gradient {
                                    eprintln!("  gradient: type={} angle={} cx={} cy={} blur={} colors={:?} positions={:?}",
                                        g.gradient_type, g.angle, g.center_x, g.center_y, g.blur,
                                        g.colors.iter().map(|c| format!("#{:06X}", c)).collect::<Vec<_>>(),
                                        g.positions,
                                    );
                                }
                            }
                            let common = match shape.as_ref() {
                                crate::model::shape::ShapeObject::Rectangle(r) => Some(&r.common),
                                crate::model::shape::ShapeObject::Ellipse(e) => Some(&e.common),
                                crate::model::shape::ShapeObject::Polygon(p) => Some(&p.common),
                                crate::model::shape::ShapeObject::Curve(c) => Some(&c.common),
                                _ => None,
                            };
                            if let Some(c) = common {
                                eprintln!("  common: width={} height={} treat_as_char={} horz_rel={:?} vert_rel={:?} h_off={} v_off={}",
                                    c.width, c.height, c.treat_as_char, c.horz_rel_to, c.vert_rel_to,
                                    c.horizontal_offset, c.vertical_offset);
                            }
                            for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                let text: String = tp.text.chars().take(30).collect();
                                eprintln!("  tb_para[{}]: text={:?} total_chars={}", tpi, text, tp.text.chars().count());
                            }
                            shape_found = true;
                        }
                    }
                }
            }
        }
        assert!(shape_found, "글상자가 있는 Shape 컨트롤을 찾지 못했습니다");

        // 모든 문단 내용 덤프
        eprintln!("\n=== 문단 목록 (섹션 0) ===");
        let sec = &doc.document.sections[0];
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            let text: String = para.text.chars().take(60).collect();
            let ctrl_types: Vec<String> = para.controls.iter().map(|c| match c {
                Control::Table(_) => "Table".to_string(),
                Control::Shape(s) => format!("Shape({:?})", match s.as_ref() {
                    crate::model::shape::ShapeObject::Rectangle(_) => "Rect",
                    crate::model::shape::ShapeObject::Ellipse(_) => "Ellipse",
                    crate::model::shape::ShapeObject::Line(_) => "Line",
                    _ => "Other",
                }),
                Control::SectionDef(_) => "SectionDef".to_string(),
                Control::ColumnDef(_) => "ColumnDef".to_string(),
                _ => "Other".to_string(),
            }).collect();
            eprintln!("  para[{}]: text_len={} line_segs={} char_shapes={} ctrls={:?} text={:?}",
                pi, para.text.chars().count(), para.line_segs.len(), para.char_shapes.len(), ctrl_types, text);
        }

        // 렌더 트리에서 TextRun의 cell context 확인
        let page_count = doc.page_count();
        eprintln!("\n페이지 수: {}", page_count);

        fn count_textruns(node: &RenderNode, body_runs: &mut Vec<String>, cell_runs: &mut Vec<String>) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                let (ppi, ci, cei, cpi) = tr.cell_context.as_ref().map_or(
                    (None, None, None, None),
                    |ctx| (
                        Some(ctx.parent_para_index),
                        Some(ctx.path[0].control_index),
                        Some(ctx.path[0].cell_index),
                        Some(ctx.path[0].cell_para_index),
                    ),
                );
                let info = format!(
                    "text={:?} sec={:?} para={:?} char_start={:?} ppi={:?} ci={:?} cei={:?} cpi={:?} bbox=({:.1},{:.1},{:.1},{:.1})",
                    tr.text.chars().take(15).collect::<String>(),
                    tr.section_index, tr.para_index, tr.char_start,
                    ppi, ci, cei, cpi,
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                );
                if tr.cell_context.is_some() {
                    cell_runs.push(info);
                } else {
                    body_runs.push(info);
                }
            }
            for child in &node.children {
                count_textruns(child, body_runs, cell_runs);
            }
        }

        for page in 0..page_count {
            let tree = doc.build_page_tree(page as u32).unwrap();
            let mut body_runs = Vec::new();
            let mut cell_runs = Vec::new();
            count_textruns(&tree.root, &mut body_runs, &mut cell_runs);
            eprintln!("\n--- 페이지 {} ---", page);
            eprintln!("본문 TextRun: {}개", body_runs.len());
            for r in &body_runs {
                eprintln!("  [body] {}", r);
            }
            eprintln!("셀/글상자 TextRun: {}개", cell_runs.len());
            for r in &cell_runs {
                eprintln!("  [cell] {}", r);
            }
        }
    }

    /// 타스크66: 텍스트+Table(treat_as_char) 혼합 문단의 인라인 렌더링 검증
    /// treat_as_char 표는 텍스트와 같은 줄에 인라인 배치되어야 함
    #[test]
    fn test_task66_table_text_mixed_paragraph_rendering() {
        use crate::renderer::composer::compose_paragraph;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = "samples/img-start-001.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // para[1]: 텍스트와 Table 컨트롤이 공존하는 문단
        let para1 = &doc.document.sections[0].paragraphs[1];
        assert!(!para1.text.is_empty(), "para[1]에 텍스트가 있어야 함");
        let has_treat_as_char_table = para1.controls.iter().any(|c| {
            matches!(c, Control::Table(t) if t.attr & 0x01 != 0)
        });
        assert!(has_treat_as_char_table, "para[1]에 treat_as_char Table이 있어야 함");

        // compose: 2개 줄 이상
        let composed = compose_paragraph(para1);
        assert!(composed.lines.len() >= 2, "최소 2줄 이상이어야 함");
        let line1_text: String = composed.lines[1].runs.iter().map(|r| r.text.as_str()).collect();
        assert!(line1_text.contains("주관부서"), "두 번째 줄에 '주관부서' 텍스트가 있어야 함");

        // pagination: 블록형 treat_as_char 표(2+ line_segs)는 PageItem::Table로 emit
        // truly inline(1 line_seg + 텍스트)만 FullParagraph로 처리
        assert!(para1.line_segs.len() >= 2, "para[1]은 2+ line_segs (블록형 treat_as_char)");
        let mut found_block_table = false;
        let mut found_partial_para = false;
        for pr in doc.pagination.iter() {
            for page in &pr.pages {
                for col in &page.column_contents {
                    for item in &col.items {
                        match item {
                            crate::renderer::pagination::PageItem::Table { para_index, .. } if *para_index == 1 => {
                                found_block_table = true;
                            }
                            crate::renderer::pagination::PageItem::PartialParagraph { para_index, .. } if *para_index == 1 => {
                                found_partial_para = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        assert!(found_block_table, "블록형 treat_as_char 표는 PageItem::Table로 emit되어야 함");
        assert!(found_partial_para, "블록형 treat_as_char 표의 텍스트는 PartialParagraph로 emit되어야 함");

        // 렌더 트리: Table과 TextRun이 모두 존재해야 함
        let tree = doc.build_page_tree(0).unwrap();
        fn find_table_and_text(node: &RenderNode, table_found: &mut bool, text_found: &mut bool) {
            match &node.node_type {
                RenderNodeType::Table(_) => {
                    *table_found = true;
                }
                RenderNodeType::TextRun(ref tr) => {
                    if tr.para_index == Some(1) && tr.cell_context.is_none() && !tr.text.is_empty() {
                        *text_found = true;
                    }
                }
                _ => {}
            }
            for child in &node.children {
                find_table_and_text(child, table_found, text_found);
            }
        }
        let mut table_found = false;
        let mut text_found = false;
        find_table_and_text(&tree.root, &mut table_found, &mut text_found);
        assert!(table_found, "렌더 트리에 표가 있어야 함");
        assert!(text_found, "렌더 트리에 para[1] 텍스트가 있어야 함");

        // SVG: 개별 문자가 <text> 요소로 출력되는지 확인
        let svg = doc.render_page_svg_native(0).unwrap();
        assert!(svg.contains("주"), "SVG에 '주' 문자가 포함되어야 함");
        assert!(svg.contains("【"), "SVG에 '【' 문자가 포함되어야 함");
    }

    /// 타스크 76: hwp-multi-001.hwp 2페이지에 그룹 이미지 3장이 존재하는지 검증
    #[test]
    fn test_task76_multi_001_group_images() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = "samples/hwp-multi-001.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();
        assert!(doc.page_count() >= 2, "최소 2페이지 이상이어야 함");

        // 2페이지 렌더 트리에서 Image 노드 수 확인
        let tree = doc.build_page_tree(1).unwrap();
        fn count_images(node: &RenderNode) -> usize {
            let mut count = match &node.node_type {
                RenderNodeType::Image(_) => 1,
                _ => 0,
            };
            for child in &node.children {
                count += count_images(child);
            }
            count
        }
        let image_count = count_images(&tree.root);
        assert!(image_count >= 3,
            "hwp-multi-001.hwp 2페이지에 Image 노드가 3개 이상이어야 함 (실제: {})", image_count);
    }

    /// 타스크 76: hwp-3.0-HWPML.hwp 1페이지 배경 이미지가 body clip 바깥에 위치하는지 검증
    #[test]
    fn test_task76_background_image_outside_body_clip() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = "samples/hwp-3.0-HWPML.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        let tree = doc.build_page_tree(0).unwrap();

        // root의 직접 자식(Body 바깥)에 Image 노드가 있어야 함
        let root_image_count = tree.root.children.iter().filter(|child| {
            matches!(&child.node_type, RenderNodeType::Image(_))
                || child.children.iter().any(|c| matches!(&c.node_type, RenderNodeType::Image(_)))
        }).count();
        assert!(root_image_count >= 1,
            "배경 이미지가 body clip 바깥(root 직접 자식)에 있어야 함 (실제: {})", root_image_count);

        // 배경 이미지 좌표 검증: (0, 0) 근처
        fn find_root_image(node: &RenderNode) -> Option<(f64, f64)> {
            if let RenderNodeType::Image(_) = &node.node_type {
                return Some((node.bbox.x, node.bbox.y));
            }
            for child in &node.children {
                if let Some(pos) = find_root_image(child) {
                    return Some(pos);
                }
            }
            None
        }
        for child in &tree.root.children {
            if let Some((x, y)) = find_root_image(child) {
                assert!(x.abs() < 1.0 && y.abs() < 1.0,
                    "배경 이미지는 (0,0) 근처여야 함 (실제: ({:.1}, {:.1}))", x, y);
                break;
            }
        }
    }

    /// 타스크 76: hwp-img-001.hwp에 독립 이미지 4장이 존재하는지 검증
    #[test]
    fn test_task76_img_001_four_pictures() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = "samples/hwp-img-001.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        let tree = doc.build_page_tree(0).unwrap();
        fn count_images(node: &RenderNode) -> usize {
            let mut count = match &node.node_type {
                RenderNodeType::Image(_) => 1,
                _ => 0,
            };
            for child in &node.children {
                count += count_images(child);
            }
            count
        }
        let image_count = count_images(&tree.root);
        assert_eq!(image_count, 4,
            "hwp-img-001.hwp에 Image 노드가 4개여야 함 (실제: {})", image_count);
    }

    /// 타스크 77: 이미지 셀 행이 인트라-로우 분할되지 않고 다음 페이지로 이동하는지 검증
    #[test]
    fn test_task77_image_cell_no_intra_row_split() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // 표6(4행×1열)의 PartialTable 페이지네이션 검증
        // 행2(이미지 셀)는 인트라-로우 분할되지 않아야 함
        // 표6의 para_index는 29 (task78에서 para[25] GSO 파싱 정상화 후)
        let table_para_index = 29;
        let mut found_table_pages: Vec<(usize, usize, f64, f64)> = Vec::new();
        for pr in &doc.pagination {
            for page in &pr.pages {
                for col in &page.column_contents {
                    for item in &col.items {
                        if let crate::renderer::pagination::PageItem::PartialTable {
                            para_index, start_row, end_row,
                            split_start_content_offset, split_end_content_limit, ..
                        } = item {
                            if *para_index == table_para_index {
                                found_table_pages.push((*start_row, *end_row, *split_start_content_offset, *split_end_content_limit));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(found_table_pages.len(), 2, "표6이 2페이지에 걸쳐야 함");

        // 첫 번째 페이지: rows 0..2 (행0, 행1만), split_end=0 (인트라-로우 분할 없음)
        let (s1, e1, _ss1, se1) = found_table_pages[0];
        assert_eq!(s1, 0, "첫 번째 PartialTable 시작 행");
        assert_eq!(e1, 2, "첫 번째 PartialTable 끝 행 (행2 미포함)");
        assert_eq!(se1, 0.0, "인트라-로우 분할 없어야 함");

        // 두 번째 페이지: rows 2..4 (행2, 행3), split_start=0 (연속 오프셋 없음)
        let (s2, e2, ss2, _se2) = found_table_pages[1];
        assert_eq!(s2, 2, "두 번째 PartialTable 시작 행");
        assert_eq!(e2, 4, "두 번째 PartialTable 끝 행");
        assert_eq!(ss2, 0.0, "연속 오프셋 없어야 함");

        // 두 페이지 모두에서 이미지가 렌더링되는지 확인
        fn find_images(node: &RenderNode) -> Vec<u16> {
            let mut ids = Vec::new();
            if let RenderNodeType::Image(img) = &node.node_type {
                ids.push(img.bin_data_id);
            }
            for child in &node.children {
                ids.extend(find_images(child));
            }
            ids
        }

        // 표6이 있는 페이지에서 이미지 확인 (PAGE 2, PAGE 3)
        let page_count = doc.page_count();
        let mut pages_with_table30_images: Vec<Vec<u16>> = Vec::new();
        for pi in 0..page_count {
            let tree = doc.build_page_tree(pi).unwrap();
            let images = find_images(&tree.root);
            // bin_data_id=6 (셀0 그림3) 또는 bin_data_id=1 (셀2 그림4)
            let table30_imgs: Vec<u16> = images.into_iter()
                .filter(|&id| id == 6 || id == 1)
                .collect();
            if !table30_imgs.is_empty() {
                pages_with_table30_images.push(table30_imgs);
            }
        }

        assert_eq!(pages_with_table30_images.len(), 2,
            "표6 이미지가 2개 페이지에 분산되어야 함");
        assert!(pages_with_table30_images[0].contains(&6),
            "첫 번째 페이지에 셀0 이미지(bin_data_id=6) 있어야 함");
        assert!(pages_with_table30_images[1].contains(&1),
            "두 번째 페이지에 셀2 이미지(bin_data_id=1) 있어야 함");
    }

    #[test]
    fn test_task78_rectangle_textbox_inline_images() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::model::shape::ShapeObject;

        let path = "samples/20250130-hongbo.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // para[25]의 GSO 컨트롤이 Rectangle (Group이 아닌)으로 파싱되는지 검증
        let section = &doc.document.sections[0];
        let para25 = &section.paragraphs[25];
        assert_eq!(para25.controls.len(), 1, "para[25]에 컨트롤 1개 있어야 함");

        if let Control::Shape(shape) = &para25.controls[0] {
            if let ShapeObject::Rectangle(rect) = shape.as_ref() {
                // Rectangle으로 올바르게 파싱됨
                assert!(rect.common.treat_as_char, "treat_as_char=true");
                // TextBox가 있어야 함
                assert!(rect.drawing.text_box.is_some(), "Rectangle에 TextBox가 있어야 함");
                let tb = rect.drawing.text_box.as_ref().unwrap();
                assert!(!tb.paragraphs.is_empty(), "TextBox에 문단이 있어야 함");
                // TextBox 문단에 인라인 Picture 컨트롤 2개
                let pic_count: usize = tb.paragraphs.iter()
                    .flat_map(|p| &p.controls)
                    .filter(|c| matches!(c, Control::Picture(_)))
                    .count();
                assert_eq!(pic_count, 2, "TextBox에 인라인 Picture 2개 있어야 함");
            } else {
                panic!("para[25]의 컨트롤이 Rectangle이어야 함 (Group이 아닌)");
            }
        } else {
            panic!("para[25]의 컨트롤이 Shape이어야 함");
        }

        // 페이지 2 렌더 트리에서 이미지 2개 렌더링 확인
        fn find_images(node: &RenderNode) -> Vec<u16> {
            let mut ids = Vec::new();
            if let RenderNodeType::Image(img) = &node.node_type {
                ids.push(img.bin_data_id);
            }
            for child in &node.children {
                ids.extend(find_images(child));
            }
            ids
        }

        let tree = doc.build_page_tree(1).unwrap(); // 페이지 2 (인덱스 1)
        let images = find_images(&tree.root);
        assert!(images.len() >= 2, "페이지 2에 이미지 2개 이상 렌더링되어야 함 (실제: {}개)", images.len());
    }

    /// 타스크 79: 투명선 표시 기능 — show_transparent_borders=true 시 추가 Line 노드 생성 검증
    #[test]
    fn test_task79_transparent_border_lines() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::model::style::BorderLineType;

        fn count_lines(node: &RenderNode) -> usize {
            let mut count = 0;
            if matches!(&node.node_type, RenderNodeType::Line(_)) {
                count += 1;
            }
            for child in &node.children {
                count += count_lines(child);
            }
            count
        }

        // 여러 표 포함 파일로 검증
        let files = [
            "samples/table-001.hwp",
            "samples/hwp_table_test.hwp",
            "samples/table-complex.hwp",
            "samples/hwpers_test4_complex_table.hwp",
            "samples/table-ipc.hwp",
        ];

        let mut tested = false;
        for path in &files {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            let data = std::fs::read(path).unwrap();
            let mut doc = HwpDocument::from_bytes(&data).unwrap();

            // 문서 내 None 테두리 존재 여부 확인
            let has_none_border = doc.document.doc_info.border_fills.iter().any(|bf|
                bf.borders.iter().any(|b| b.line_type == BorderLineType::None)
            );

            // 투명선 OFF
            doc.show_transparent_borders = false;
            let tree_off = doc.build_page_tree(0).unwrap();
            let lines_off = count_lines(&tree_off.root);

            // 투명선 ON
            doc.show_transparent_borders = true;
            let tree_on = doc.build_page_tree(0).unwrap();
            let lines_on = count_lines(&tree_on.root);

            // 회귀 없음: ON >= OFF
            assert!(lines_on >= lines_off,
                "{}: 투명선 ON({})이 OFF({}) 이상이어야 함", path, lines_on, lines_off);

            // SVG 렌더링 정상 확인
            let svg = doc.render_page_svg_native(0).unwrap();
            assert!(svg.contains("<svg"), "{}: SVG 렌더링 실패", path);

            eprintln!("{}: OFF={} ON={} (+{}) has_none_border={}",
                path, lines_off, lines_on, lines_on - lines_off, has_none_border);
            tested = true;
        }
        assert!(tested, "테스트할 수 있는 파일이 없음");
    }

    #[test]
    fn test_task80_cell_height_matches_hwp() {
        // 셀 높이 검증: 단일 줄/단일 문단 셀의 컨텐츠 높이 + 패딩 ≈ HWP 선언 높이
        // (마지막 줄 line_spacing이 제외되었는지 확인)
        use crate::renderer::composer::compose_paragraph;

        let path = "samples/table-001.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} not found", path);
            return;
        }
        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();
        let dpi = 96.0;

        let mut checked = 0;
        for sec in &doc.document.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Table(table) = ctrl {
                        for cell in &table.cells {
                            // 단일 행, 단일 문단, 유효한 높이만 검증
                            if cell.row_span != 1 { continue; }
                            if cell.paragraphs.len() != 1 { continue; }
                            if cell.height == 0 || cell.height >= 0x80000000 { continue; }

                            let comp = compose_paragraph(&cell.paragraphs[0]);
                            if comp.lines.is_empty() { continue; }

                            let pad_top = if cell.padding.top != 0 {
                                crate::renderer::hwpunit_to_px(cell.padding.top as i32, dpi)
                            } else {
                                crate::renderer::hwpunit_to_px(table.padding.top as i32, dpi)
                            };
                            let pad_bottom = if cell.padding.bottom != 0 {
                                crate::renderer::hwpunit_to_px(cell.padding.bottom as i32, dpi)
                            } else {
                                crate::renderer::hwpunit_to_px(table.padding.bottom as i32, dpi)
                            };

                            // 마지막 줄 line_spacing 제외
                            let lc = comp.lines.len();
                            let content: f64 = comp.lines.iter().enumerate().map(|(i, line)| {
                                let h = crate::renderer::hwpunit_to_px(line.line_height, dpi);
                                if i + 1 < lc {
                                    h + crate::renderer::hwpunit_to_px(line.line_spacing, dpi)
                                } else { h }
                            }).sum();

                            let required = content + pad_top + pad_bottom;
                            let declared = crate::renderer::hwpunit_to_px(cell.height as i32, dpi);

                            // 우리 계산값이 HWP 선언값 이하여야 함 (2px 허용)
                            assert!(required <= declared + 2.0,
                                "Cell row={} col={}: required={:.1}px > declared={:.1}px (diff={:.1}px)",
                                cell.row, cell.col, required, declared, required - declared);
                            checked += 1;
                        }
                    }
                }
            }
        }
        eprintln!("task80: {}개 셀 높이 검증 통과", checked);
        assert!(checked > 0, "검증할 셀이 없음");
    }



    /// 타스크 81: table-004.hwp의 세로쓰기 셀 파싱 및 렌더 트리 검증
    #[test]
    fn test_task81_vertical_cell_text() {
        let path = "samples/table-004.hwp";
        if !std::path::Path::new(path).exists() { return; }
        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // 1. 파서 검증: text_direction=2인 셀이 3개 존재
        let mut vertical_cells = Vec::new();
        for sec in &doc.document.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let crate::model::control::Control::Table(table) = ctrl {
                        for cell in &table.cells {
                            if cell.text_direction != 0 {
                                vertical_cells.push((cell.text_direction, cell.row, cell.col));
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(vertical_cells.len(), 3, "세로쓰기 셀이 3개여야 함");
        for (td, _r, _c) in &vertical_cells {
            assert_eq!(*td, 2, "text_direction은 2(영문세움)이어야 함");
        }

        // 2. 렌더 트리 검증: SVG 내보내기로 세로 배치 확인
        let dpi = 96.0;
        let styles = crate::renderer::style_resolver::resolve_styles(&doc.document.doc_info, dpi);
        let engine = crate::renderer::layout::LayoutEngine::new(dpi);

        // pagination → render tree
        assert!(!doc.pagination.is_empty(), "pagination 결과가 비어있으면 안 됨");
        let pr = &doc.pagination[0];
        assert!(!pr.pages.is_empty(), "페이지가 비어있으면 안 됨");

        let section = &doc.document.sections[0];
        let composed: Vec<_> = section.paragraphs.iter()
            .map(crate::renderer::composer::compose_paragraph)
            .collect();
        let sec_mt = doc.measured_tables.get(0).map(|v| v.as_slice()).unwrap_or(&[]);
        let tree = engine.build_render_tree(
            &pr.pages[0],
            &section.paragraphs,
            &section.paragraphs,
            &section.paragraphs,
            &composed,
            &styles,
            &section.section_def.footnote_shape,
            &doc.document.bin_data_content,
            None,
            sec_mt,
            Some(&section.section_def.page_border_fill),
            section.section_def.outline_numbering_id,
            &[],
        );

        // 렌더 트리에서 text_direction != 0인 TableCell 노드 찾기
        fn find_vertical_cells(node: &crate::renderer::render_tree::RenderNode) -> Vec<&crate::renderer::render_tree::RenderNode> {
            let mut result = Vec::new();
            if let crate::renderer::render_tree::RenderNodeType::TableCell(ref tc) = node.node_type {
                if tc.text_direction != 0 {
                    result.push(node);
                }
            }
            for child in &node.children {
                result.extend(find_vertical_cells(child));
            }
            result
        }

        let vc_nodes = find_vertical_cells(&tree.root);
        assert!(vc_nodes.len() >= 3, "렌더 트리에 세로쓰기 셀이 3개 이상이어야 함, found: {}", vc_nodes.len());

        // 각 세로쓰기 셀의 TextRun이 세로 방향으로 배치되었는지 확인
        for vc in &vc_nodes {
            let mut run_ys: Vec<f64> = Vec::new();
            for line_node in &vc.children {
                if let crate::renderer::render_tree::RenderNodeType::TextLine(_) = &line_node.node_type {
                    for run_node in &line_node.children {
                        if let crate::renderer::render_tree::RenderNodeType::TextRun(ref tr) = run_node.node_type {
                            if !tr.text.trim().is_empty() {
                                run_ys.push(run_node.bbox.y);
                            }
                        }
                    }
                }
            }
            // y좌표가 순차 증가해야 세로 배치
            assert!(run_ys.len() >= 2, "세로쓰기 셀에 TextRun이 2개 이상이어야 함");
            for i in 1..run_ys.len() {
                assert!(run_ys[i] > run_ys[i - 1],
                    "세로쓰기 글자의 y좌표가 순차 증가해야 함: y[{}]={} <= y[{}]={}",
                    i, run_ys[i], i - 1, run_ys[i - 1]);
            }
        }
    }

    /// 표 바운딩박스 조회 테스트
    #[test]
    fn test_get_table_bbox() {
        use std::path::Path;

        let path = Path::new("samples/hwp_table_test.hwp");
        if !path.exists() {
            eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
            return;
        }

        let data = std::fs::read(path).unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        let result = doc.get_table_bbox_native(0, 3, 0);
        assert!(result.is_ok(), "표 bbox 조회 실패: {:?}", result.err());

        let json = result.unwrap();
        assert!(json.contains("pageIndex"), "pageIndex 필드 존재 확인");
        assert!(json.contains("width"), "width 필드 존재 확인");
        assert!(json.contains("height"), "height 필드 존재 확인");
        eprintln!("표 bbox: {}", json);
    }

    /// 표 컨트롤 삭제 테스트 (wasm_api 내부 접근)
    #[test]
    fn test_delete_table_control() {
        use std::path::Path;

        let path = Path::new("samples/hwp_table_test.hwp");
        if !path.exists() {
            eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
            return;
        }

        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();
        let _ = doc.convert_to_editable_native();

        // 삭제 전 컨트롤 수 확인
        let before_count = doc.document.sections[0].paragraphs[3].controls.len();
        assert!(before_count > 0, "테스트 파일에 표가 없음");

        // 삭제 전 char_count
        let before_char_count = doc.document.sections[0].paragraphs[3].char_count;

        // 표 bbox 조회 성공 확인
        let bbox_result = doc.get_table_bbox_native(0, 3, 0);
        assert!(bbox_result.is_ok(), "삭제 전 bbox 조회 실패");

        // 표 삭제
        let result = doc.delete_table_control_native(0, 3, 0);
        assert!(result.is_ok(), "표 삭제 실패: {:?}", result.err());

        // 삭제 후 컨트롤 수 감소 확인
        let after_count = doc.document.sections[0].paragraphs[3].controls.len();
        assert_eq!(after_count, before_count - 1, "컨트롤 수 감소 확인");

        // char_count가 8 감소했는지 확인
        let after_char_count = doc.document.sections[0].paragraphs[3].char_count;
        assert_eq!(after_char_count, before_char_count - 8, "char_count 8 감소 확인");

        eprintln!("표 삭제: 컨트롤 {}→{}, char_count {}→{}", before_count, after_count, before_char_count, after_char_count);
    }

    #[test]
    /// B6: 표 구조 변경 후 저장 시 빈 셀 문단의 PARA_TEXT/char_count/LineSeg 검증
    fn test_table_modification_empty_cell_serialization() {
        use std::path::Path;
        use crate::parser::record::Record;

        let path = Path::new("samples/hwp_table_test.hwp");
        if !path.exists() {
            eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
            return;
        }

        let data = std::fs::read(path).unwrap();

        // 행 추가 후 내보내기
        let mut doc = HwpDocument::from_bytes(&data).unwrap();
        doc.insert_table_row_native(0, 3, 0, 0, true).unwrap();
        let exported = doc.export_hwp_native().unwrap();

        // 재파싱
        let parsed = crate::parser::parse_hwp(&exported).unwrap();
        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&exported).unwrap();
        let bt = cfb.read_body_text_section(0, parsed.header.compressed, false).unwrap();
        let recs = Record::read_all(&bt).unwrap();

        // 표 범위 내 PARA_HEADER → PARA_TEXT 패턴 검사
        // cc=1인 문단(빈 셀)은 PARA_TEXT가 없어야 한다
        let mut empty_cell_count = 0;
        let mut violation_count = 0;

        for (i, rec) in recs.iter().enumerate() {
            if rec.tag_id == crate::parser::tags::HWPTAG_PARA_HEADER && rec.data.len() >= 4 {
                let cc_raw = u32::from_le_bytes(rec.data[0..4].try_into().unwrap());
                let cc = cc_raw & 0x7FFFFFFF;

                // 빈 문단 (cc == 0 또는 1)
                if cc <= 1 {
                    empty_cell_count += 1;

                    // 다음 레코드가 PARA_TEXT이면 안 됨
                    if i + 1 < recs.len() && recs[i + 1].tag_id == crate::parser::tags::HWPTAG_PARA_TEXT {
                        violation_count += 1;
                        eprintln!("!! 위반: rec[{}] cc={} 다음에 PARA_TEXT({}B) 존재",
                            i, cc, recs[i + 1].data.len());
                    }

                    // cc=0이면 안 됨 (HWP 스펙: 최소 cc=1)
                    if cc == 0 {
                        eprintln!("!! 위반: rec[{}] cc=0 (HWP 스펙 위반, 최소 1이어야 함)", i);
                        violation_count += 1;
                    }

                    // PARA_LINE_SEG가 존재해야 함 — PARA_CHAR_SHAPE 다음에
                    let mut has_line_seg = false;
                    for j in (i + 1)..recs.len() {
                        if recs[j].tag_id == crate::parser::tags::HWPTAG_PARA_HEADER
                            || recs[j].level <= rec.level
                        {
                            break;
                        }
                        if recs[j].tag_id == crate::parser::tags::HWPTAG_PARA_LINE_SEG {
                            has_line_seg = true;
                            break;
                        }
                    }
                    if !has_line_seg {
                        eprintln!("!! 위반: rec[{}] cc={} PARA_LINE_SEG 없음", i, cc);
                        violation_count += 1;
                    }
                }
            }
        }

        eprintln!("빈 문단 수: {}, 위반: {}", empty_cell_count, violation_count);
        assert!(empty_cell_count > 0, "빈 셀 문단이 없음 — 테스트 유효성 확인 필요");
        assert_eq!(violation_count, 0, "빈 셀 문단 직렬화 위반이 {}건 발견됨", violation_count);
    }

    #[test]
    fn test_task105_nested_table_path_api() {
        let data = std::fs::read("samples/inner-table-01.hwp").unwrap();
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // 1. hitTest로 중첩 표 셀의 cellPath 확인
        let page_count = doc.page_count();
        eprintln!("페이지 수: {}", page_count);

        // 문서 구조 확인: 중첩 표 위치
        let sec = &doc.document.sections[0];
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(t) = ctrl {
                    eprintln!("문단[{}] 컨트롤[{}]: 표 {}행x{}열 셀{}개", pi, ci, t.row_count, t.col_count, t.cells.len());
                    for (cell_idx, cell) in t.cells.iter().enumerate() {
                        for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                            for (cci, cctrl) in cp.controls.iter().enumerate() {
                                if let Control::Table(nt) = cctrl {
                                    eprintln!("  셀[{}] 문단[{}] 컨트롤[{}]: 중첩 표 {}행x{}열 셀{}개",
                                        cell_idx, cp_idx, cci, nt.row_count, nt.col_count, nt.cells.len());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 렌더 트리에서 중첩 표 TextRun 찾기
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn find_nested_run(node: &RenderNode) -> Option<(usize, Vec<(usize, usize, usize)>)> {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if let Some(ref ctx) = tr.cell_context {
                    if ctx.path.len() >= 2 {
                        let path: Vec<(usize, usize, usize)> = ctx.path.iter()
                            .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
                            .collect();
                        return Some((ctx.parent_para_index, path));
                    }
                }
            }
            for child in &node.children {
                if let Some(r) = find_nested_run(child) {
                    return Some(r);
                }
            }
            None
        }

        // 모든 페이지에서 중첩 TextRun 탐색
        let mut nested = None;
        for page in 0..page_count {
            let tree = doc.build_page_tree(page as u32).unwrap();
            fn dump_runs(node: &RenderNode, page: u32) {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    let ctx_info = tr.cell_context.as_ref().map(|ctx| {
                        format!("ppi={}, path_len={}, path={:?}", ctx.parent_para_index, ctx.path.len(),
                            ctx.path.iter().map(|e| (e.control_index, e.cell_index, e.cell_para_index)).collect::<Vec<_>>())
                    }).unwrap_or_else(|| "None".to_string());
                    eprintln!("  p{} TextRun: text={:?} ctx={}", page, tr.text.chars().take(10).collect::<String>(), ctx_info);
                }
                for child in &node.children { dump_runs(child, page); }
            }
            dump_runs(&tree.root, page as u32);
            if nested.is_none() { nested = find_nested_run(&tree.root); }
        }
        assert!(nested.is_some(), "중첩 표 TextRun이 있어야 합니다");
        let (parent_para, path) = nested.unwrap();
        eprintln!("중첩 표 경로: parent_para={}, path={:?}", parent_para, path);

        // 2. resolve_table_by_path로 중첩 표 접근
        let table = doc.resolve_table_by_path(0, parent_para, &path);
        assert!(table.is_ok(), "resolve_table_by_path 실패: {:?}", table.err());
        let table = table.unwrap();
        eprintln!("중첩 표: {}행 x {}열, 셀 {}개", table.row_count, table.col_count, table.cells.len());

        // 3. resolve_cell_by_path로 셀 접근
        let cell = doc.resolve_cell_by_path(0, parent_para, &path);
        assert!(cell.is_ok(), "resolve_cell_by_path 실패: {:?}", cell.err());

        // 4. getCellInfoByPath 경로 API
        let path_json = format!("[{}]", path.iter().map(|(ci, cei, cpi)| {
            format!("{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}", ci, cei, cpi)
        }).collect::<Vec<_>>().join(","));
        eprintln!("path_json: {}", path_json);

        let cell_info = doc.get_cell_info_by_path_native(0, parent_para, &path_json);
        assert!(cell_info.is_ok(), "getCellInfoByPath 실패: {:?}", cell_info.err());
        eprintln!("셀 정보: {}", cell_info.unwrap());

        // 5. getTableDimensionsByPath 경로 API
        let dims = doc.get_table_dimensions_by_path_native(0, parent_para, &path_json);
        assert!(dims.is_ok(), "getTableDimensionsByPath 실패: {:?}", dims.err());
        eprintln!("표 차원: {}", dims.unwrap());

        // 6. getCursorRectByPath 경로 API
        let cursor = doc.get_cursor_rect_by_path_native(0, parent_para, &path_json, 0);
        assert!(cursor.is_ok(), "getCursorRectByPath 실패: {:?}", cursor.err());
        eprintln!("커서 위치: {}", cursor.unwrap());

        // 7. getTableCellBboxesByPath 경로 API
        let bboxes = doc.get_table_cell_bboxes_by_path_native(0, parent_para, &path_json);
        assert!(bboxes.is_ok(), "getTableCellBboxesByPath 실패: {:?}", bboxes.err());
        eprintln!("셀 bbox: {}", bboxes.unwrap());

        // 8. hitTest에서 cellPath 포함 확인
        let hit_json = doc.hit_test_native(0, 400.0, 600.0);
        if let Ok(ref json) = hit_json {
            eprintln!("hitTest 결과: {}", json);
            if json.contains("cellPath") {
                eprintln!("✓ hitTest에 cellPath 포함됨");
            } else {
                eprintln!("✗ hitTest에 cellPath 없음 — 본문 영역 클릭일 수 있음");
            }
        }
    }

    #[test]
    fn test_task110_multi_column_reflow_diag() {
        let path = "samples/basic/KTX.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }
        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("=== KTX.hwp 다단 리플로우 진단 ===");
        eprintln!("페이지 수: {}", doc.page_count());
        eprintln!("구역 수: {}", doc.document.sections.len());

        // ColumnDef 확인
        {
            let section = &doc.document.sections[0];
            let column_def = HwpDocument::find_initial_column_def(&section.paragraphs);
            eprintln!("ColumnDef: count={}, same_width={}, widths={:?}, gaps={:?}",
                column_def.column_count, column_def.same_width,
                column_def.widths, column_def.gaps);

            // PageLayoutInfo 확인
            let layout = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                &section.section_def.page_def, &column_def, doc.dpi);
            eprintln!("column_areas 수: {}", layout.column_areas.len());
            for (i, ca) in layout.column_areas.iter().enumerate() {
                let w_hu = crate::renderer::px_to_hwpunit(ca.width, doc.dpi);
                eprintln!("  column_areas[{}]: x={:.1} w={:.1}px ({}hu)", i, ca.x, ca.width, w_hu);
            }

            // para_column_map 확인
            let map = &doc.para_column_map;
            eprintln!("para_column_map 구역 수: {}", map.len());
            if !map.is_empty() && !map[0].is_empty() {
                eprintln!("para_column_map[0] 길이: {}", map[0].len());
                for (pi, &ci) in map[0].iter().enumerate() {
                    let seg_w = section.paragraphs.get(pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|ls| ls.segment_width).unwrap_or(0);
                    eprintln!("  para[{}] → col_idx={}, seg_w={}", pi, ci, seg_w);
                }
            } else {
                eprintln!("para_column_map[0] 비어있음!");
            }

            // 본문 전체 너비 (단일 단) 비교
            let layout_single = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                &section.section_def.page_def,
                &crate::model::page::ColumnDef::default(),
                doc.dpi);
            let body_w_hu = crate::renderer::px_to_hwpunit(layout_single.column_areas[0].width, doc.dpi);
            eprintln!("단일 단 body width: {:.1}px ({}hu)", layout_single.column_areas[0].width, body_w_hu);
        }

        // SVG 내보내기: 편집 전
        let svg_before = doc.render_page_svg_native(0).unwrap();
        std::fs::write("output/ktx_before_edit.svg", &svg_before).ok();
        eprintln!("\n편집 전 SVG: output/ktx_before_edit.svg ({} bytes)", svg_before.len());

        // 문단 1에 텍스트 삽입
        let result = doc.insert_text_native(0, 1, 0, "테스트입력 ");
        eprintln!("insert_text 결과: {:?}", result);

        // 편집 후 line_segs 확인
        let para1 = &doc.document.sections[0].paragraphs[1];
        eprintln!("편집 후 para[1] line_segs:");
        for (i, ls) in para1.line_segs.iter().enumerate() {
            eprintln!("  line[{}]: seg_w={} text_start={}", i, ls.segment_width, ls.text_start);
        }

        // SVG 내보내기: 편집 후
        let svg_after = doc.render_page_svg_native(0).unwrap();
        std::fs::write("output/ktx_after_edit.svg", &svg_after).ok();
        eprintln!("편집 후 SVG: output/ktx_after_edit.svg ({} bytes)", svg_after.len());
    }

    /// tb-err-003.hwp: 저장→로드→재저장 시 control_mask/has_para_text 보정 검증
    #[test]
    fn test_diag_tb_err_003() {
        use crate::model::control::Control;
        use crate::serializer::body_text::serialize_section;
        use crate::parser::body_text::parse_body_text_section;

        // 두 파일 모두 분석
        let files = vec!["saved/tb-err-003.hwp", "saved/tb-err-003-s.hwp"];
        for path in &files {
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            continue;
        }

        let data = std::fs::read(path).unwrap();
        let doc = crate::parser::parse_hwp(&data).unwrap();

        eprintln!("\n{}", "=".repeat(80));
        eprintln!("=== {} 진단 ({} bytes) ===", path, data.len());
        eprintln!("섹션 수: {}", doc.sections.len());

        for (si, section) in doc.sections.iter().enumerate() {
            eprintln!("\n--- Section {} (문단 {}개) ---", si, section.paragraphs.len());
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let ctrl_types: Vec<String> = para.controls.iter().map(|c| match c {
                    Control::Table(t) => format!("Table({}x{})", t.row_count, t.col_count),
                    _ => format!("{:?}", std::mem::discriminant(c)),
                }).collect();
                eprintln!("  문단[{}]: text={:?} char_count={} msb={} ctrl_mask=0x{:08X} controls=[{}] line_segs={} has_para_text={} raw_header_extra({})={:02x?}",
                    pi, &para.text.chars().take(30).collect::<String>(),
                    para.char_count, para.char_count_msb, para.control_mask,
                    ctrl_types.join(", "), para.line_segs.len(), para.has_para_text,
                    para.raw_header_extra.len(), &para.raw_header_extra);
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(t) = ctrl {
                        eprintln!("\n  문단[{}] 컨트롤[{}]: 표 {}행×{}열 (셀 {}개)", pi, ci, t.row_count, t.col_count, t.cells.len());
                        eprintln!("  row_sizes: {:?}", t.row_sizes);
                        eprintln!("  raw_table_record_attr: 0x{:08X}", t.raw_table_record_attr);
                        eprintln!("  raw_table_record_extra ({} bytes): {:02x?}", t.raw_table_record_extra.len(), &t.raw_table_record_extra);

                        // 각 셀 상세
                        for (cell_idx, cell) in t.cells.iter().enumerate() {
                            eprintln!("  셀[{}]: col={} row={} cs={} rs={} w={} h={} bfid={} paras={}",
                                cell_idx, cell.col, cell.row, cell.col_span, cell.row_span,
                                cell.width, cell.height, cell.border_fill_id, cell.paragraphs.len());
                            eprintln!("    raw_list_extra ({} bytes): {:02x?}", cell.raw_list_extra.len(), &cell.raw_list_extra);
                            for (pp, para) in cell.paragraphs.iter().enumerate() {
                                eprintln!("    para[{}]: text={:?} char_count={} msb={} line_segs={} char_shapes={} has_para_text={}",
                                    pp, &para.text.chars().take(20).collect::<String>(),
                                    para.char_count, para.char_count_msb,
                                    para.line_segs.len(), para.char_shapes.len(), para.has_para_text);
                                eprintln!("      raw_header_extra ({} bytes): {:02x?}", para.raw_header_extra.len(), &para.raw_header_extra);
                            }
                        }

                        // row_sizes 검증: 각 행의 실제 셀 수와 row_sizes 비교
                        eprintln!("\n  --- row_sizes 검증 ---");
                        for r in 0..t.row_count {
                            let actual_count = t.cells.iter().filter(|c| c.row == r).count();
                            let expected = if (r as usize) < t.row_sizes.len() { t.row_sizes[r as usize] } else { -1 };
                            let match_str = if actual_count as i16 == expected { "OK" } else { "*** MISMATCH ***" };
                            eprintln!("  행[{}]: row_sizes={} 실제셀수={} {}", r, expected, actual_count, match_str);
                        }

                        // col_count 검증: 셀들의 최대 col+col_span
                        let max_col_extent = t.cells.iter().map(|c| c.col + c.col_span).max().unwrap_or(0);
                        eprintln!("  col_count={} 최대열범위={} {}",
                            t.col_count, max_col_extent,
                            if t.col_count == max_col_extent { "OK" } else { "*** MISMATCH ***" });

                        let max_row_extent = t.cells.iter().map(|c| c.row + c.row_span).max().unwrap_or(0);
                        eprintln!("  row_count={} 최대행범위={} {}",
                            t.row_count, max_row_extent,
                            if t.row_count == max_row_extent { "OK" } else { "*** MISMATCH ***" });
                    }
                }
            }

            // 직렬화 → 재파싱 검증 (raw_stream이 있으면 원본 그대로이므로, 없는 것처럼 재직렬화)
            use crate::serializer::record_writer::write_records;
            let mut records = Vec::new();
            crate::serializer::body_text::serialize_paragraph_list(&section.paragraphs, 0, &mut records);
            let serialized = write_records(&records);
            match parse_body_text_section(&serialized) {
                Ok(reparsed) => {
                    eprintln!("\n  직렬화→재파싱: OK ({} → {} 문단)", section.paragraphs.len(), reparsed.paragraphs.len());
                    // 재파싱된 문단의 control_mask 검증
                    for (pi, para) in reparsed.paragraphs.iter().enumerate() {
                        let expected_mask: u32 = para.controls.iter().fold(0u32, |mask, ctrl| {
                            let bit = match ctrl {
                                Control::SectionDef(_) | Control::ColumnDef(_) => 2,
                                Control::Table(_) | Control::Shape(_) | Control::Picture(_) => 11,
                                _ => 0,
                            };
                            mask | (1u32 << bit)
                        });
                        let mask_ok = if para.controls.is_empty() {
                            para.control_mask == 0
                        } else {
                            para.control_mask == expected_mask
                        };
                        if !mask_ok {
                            eprintln!("  *** 재파싱 문단[{}] control_mask 불일치: 0x{:08X} (expected 0x{:08X}, controls={}) ***",
                                pi, para.control_mask, expected_mask, para.controls.len());
                        }
                        // has_para_text 검증: 빈 문단에 PARA_TEXT 없어야 함
                        if para.text.is_empty() && para.controls.is_empty() && para.has_para_text {
                            eprintln!("  *** 재파싱 문단[{}] has_para_text=true on empty para (char_count={}) ***",
                                pi, para.char_count);
                        }
                        for (ci, ctrl) in para.controls.iter().enumerate() {
                            if let Control::Table(t2) = ctrl {
                                eprintln!("  재파싱 표[{},{}]: {}행×{}열 (셀 {}개) row_sizes={:?}",
                                    pi, ci, t2.row_count, t2.col_count, t2.cells.len(), t2.row_sizes);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n  *** 재파싱 실패: {} ***", e);
                }
            }
        }
        } // for files loop
    }

    #[test]
    fn test_task110_treatise_diag() {
        let path = "samples/basic/treatise sample.hwp";
        if !std::path::Path::new(path).exists() {
            eprintln!("SKIP: {} 없음", path);
            return;
        }
        let data = std::fs::read(path).unwrap();
        let mut doc = HwpDocument::from_bytes(&data).unwrap();

        eprintln!("=== treatise sample.hwp 다단 구조 진단 ===");
        eprintln!("구역 수: {}", doc.document.sections.len());

        for (sec_idx, section) in doc.document.sections.iter().enumerate() {
            eprintln!("\n--- 구역 {} ---", sec_idx);
            eprintln!("문단 수: {}", section.paragraphs.len());

            // ColumnDef 확인
            let column_def = HwpDocument::find_initial_column_def(&section.paragraphs);
            eprintln!("initial ColumnDef: count={}, same_width={}, spacing={}, widths={:?}, gaps={:?}",
                column_def.column_count, column_def.same_width,
                column_def.spacing,
                column_def.widths, column_def.gaps);
            // 2단 ColumnDef 검색
            if section.paragraphs.len() > 14 {
                let cd2 = HwpDocument::find_column_def_for_paragraph(&section.paragraphs, 14);
                eprintln!("para[14] ColumnDef: count={}, same_width={}, spacing={}, widths={:?}, gaps={:?}",
                    cd2.column_count, cd2.same_width, cd2.spacing, cd2.widths, cd2.gaps);
                let layout2 = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                    &section.section_def.page_def, &cd2, doc.dpi);
                for (i, ca) in layout2.column_areas.iter().enumerate() {
                    let w_hu = crate::renderer::px_to_hwpunit(ca.width, doc.dpi);
                    eprintln!("  2단 column_areas[{}]: x={:.1}px w={:.1}px ({}hu)", i, ca.x, ca.width, w_hu);
                }
            }

            // PageLayoutInfo 확인
            let layout = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                &section.section_def.page_def, &column_def, doc.dpi);
            eprintln!("column_areas 수: {}", layout.column_areas.len());
            for (i, ca) in layout.column_areas.iter().enumerate() {
                let w_hu = crate::renderer::px_to_hwpunit(ca.width, doc.dpi);
                eprintln!("  column_areas[{}]: x={:.1}px w={:.1}px ({}hu)", i, ca.x, ca.width, w_hu);
            }

            // para_column_map 확인
            let map = &doc.para_column_map;
            if sec_idx < map.len() && !map[sec_idx].is_empty() {
                eprintln!("para_column_map[{}] 길이: {}", sec_idx, map[sec_idx].len());
                for (pi, &ci) in map[sec_idx].iter().enumerate() {
                    let seg_w = section.paragraphs.get(pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|ls| ls.segment_width).unwrap_or(0);
                    eprintln!("  para[{}] → col_idx={}, first_line seg_w={}", pi, ci, seg_w);
                }
            } else {
                eprintln!("para_column_map[{}] 비어있음!", sec_idx);
            }

            // 첫 10개 문단의 첫 줄 segment_width
            eprintln!("첫 10개 문단 segment_width:");
            for pi in 0..std::cmp::min(10, section.paragraphs.len()) {
                let para = &section.paragraphs[pi];
                let seg_w = para.line_segs.first().map(|ls| ls.segment_width).unwrap_or(0);
                let text_preview: String = para.text.chars().take(30).collect();
                eprintln!("  para[{}]: seg_w={}, text={:?}", pi, seg_w, text_preview);
            }
        }

        // 편집 시뮬레이션: 구역0, 문단1, 오프셋0에 "X" 삽입
        eprintln!("\n=== 편집 시뮬레이션: insert_text_native(0, 1, 0, \"X\") ===");
        let result = doc.insert_text_native(0, 1, 0, "X");
        eprintln!("insert_text 결과: {:?}", result);

        // 편집 후 문단1의 첫 줄 segment_width 확인
        let para1 = &doc.document.sections[0].paragraphs[1];
        eprintln!("편집 후 para[1] line_segs:");
        for (i, ls) in para1.line_segs.iter().enumerate() {
            eprintln!("  line[{}]: seg_w={} text_start={} line_height={}", 
                i, ls.segment_width, ls.text_start, ls.line_height);
        }

        // available_width 비교: 단 너비 vs 페이지 너비
        let section = &doc.document.sections[0];
        let column_def = HwpDocument::find_initial_column_def(&section.paragraphs);
        let layout = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
            &section.section_def.page_def, &column_def, doc.dpi);
        let layout_single = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
            &section.section_def.page_def,
            &crate::model::page::ColumnDef::default(),
            doc.dpi);

        let col_w_hu = if !layout.column_areas.is_empty() {
            crate::renderer::px_to_hwpunit(layout.column_areas[0].width, doc.dpi)
        } else { 0 };
        let page_w_hu = if !layout_single.column_areas.is_empty() {
            crate::renderer::px_to_hwpunit(layout_single.column_areas[0].width, doc.dpi)
        } else { 0 };

        let actual_seg_w = para1.line_segs.first().map(|ls| ls.segment_width).unwrap_or(0);
        eprintln!("\n=== para[1] available_width 비교 (1단 영역) ===");
        eprintln!("단 너비 (column_areas[0]): {}hu", col_w_hu);
        eprintln!("페이지 전체 너비 (단일 단): {}hu", page_w_hu);
        eprintln!("실제 seg_w: {}hu", actual_seg_w);

        let diff_col = (actual_seg_w as i64 - col_w_hu as i64).abs();
        let diff_page = (actual_seg_w as i64 - page_w_hu as i64).abs();
        if diff_col < diff_page {
            eprintln!("→ seg_w가 단 너비에 가까움 (차이: {}hu)", diff_col);
        } else {
            eprintln!("→ seg_w가 페이지 너비에 가까움 (차이: {}hu)", diff_page);
        }

        // 2단 영역 편집 시뮬레이션: para[14] (col_idx=1, 2단 영역)
        eprintln!("\n=== 2단 영역 편집: insert_text_native(0, 14, 0, \"Y\") ===");
        let col_idx_14_before = doc.para_column_map.get(0)
            .and_then(|m| m.get(14)).copied().unwrap_or(0);
        eprintln!("편집 전 para[14] col_idx: {}", col_idx_14_before);

        let result2 = doc.insert_text_native(0, 14, 0, "Y");
        eprintln!("insert_text 결과: {:?}", result2);

        let para14 = &doc.document.sections[0].paragraphs[14];
        eprintln!("편집 후 para[14] line_segs:");
        for (i, ls) in para14.line_segs.iter().enumerate() {
            eprintln!("  line[{}]: seg_w={} text_start={}", i, ls.segment_width, ls.text_start);
        }

        // find_column_def_for_paragraph 결과 확인
        let cd_for_14 = HwpDocument::find_column_def_for_paragraph(
            &doc.document.sections[0].paragraphs, 14);
        eprintln!("para[14]에 적용되는 ColumnDef: count={}, same_width={}, widths={:?}",
            cd_for_14.column_count, cd_for_14.same_width, cd_for_14.widths);

        let layout14 = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
            &doc.document.sections[0].section_def.page_def, &cd_for_14, doc.dpi);
        eprintln!("layout14 column_areas:");
        for (i, ca) in layout14.column_areas.iter().enumerate() {
            let w_hu = crate::renderer::px_to_hwpunit(ca.width, doc.dpi);
            eprintln!("  [{}]: x={:.1}px w={:.1}px ({}hu)", i, ca.x, ca.width, w_hu);
        }

        let seg_w_14 = para14.line_segs.first().map(|ls| ls.segment_width).unwrap_or(0);
        let orig_seg_w_14 = 22960i32; // 편집 전 원본 seg_w
        eprintln!("\n=== para[14] 결과 비교 ===");
        eprintln!("원본 seg_w: {}hu", orig_seg_w_14);
        eprintln!("편집 후 seg_w: {}hu", seg_w_14);
        eprintln!("페이지 전체 너비: {}hu", page_w_hu);
        if (seg_w_14 - orig_seg_w_14).abs() < 1000 {
            eprintln!("→ 올바름: 2단 너비로 리플로우됨");
        } else if (seg_w_14 as i64 - page_w_hu as i64).abs() < 1000 {
            eprintln!("→ 오류: 1단 전체 너비로 리플로우됨!");
        } else {
            eprintln!("→ 알수없는 너비: {}hu", seg_w_14);
        }

        // === 양쪽 정렬 진단: 원본 2단 문단의 LineSeg 데이터 ===
        eprintln!("\n=== 양쪽 정렬 진단: 2단 문단 LineSeg 분석 ===");
        // 원본 데이터 재로드 (편집 전)
        let data2 = std::fs::read(path).unwrap();
        let doc2 = HwpDocument::from_bytes(&data2).unwrap();
        let section2 = &doc2.document.sections[0];

        // 2단 영역의 모든 문단의 LineSeg column_start, segment_width 출력
        for pi in 9..std::cmp::min(20, section2.paragraphs.len()) {
            let para = &section2.paragraphs[pi];
            let text_preview: String = para.text.chars().take(40).collect();
            eprintln!("\npara[{}]: text={:?}", pi, text_preview);
            eprintln!("  line_segs 수: {}", para.line_segs.len());
            for (li, ls) in para.line_segs.iter().enumerate() {
                eprintln!("  line[{}]: seg_w={} col_start={} text_start={} vpos={} line_h={} line_sp={}",
                    li, ls.segment_width, ls.column_start, ls.text_start,
                    ls.vertical_pos, ls.line_height, ls.line_spacing);
            }
            // 문단 정렬 확인
            let ps = doc2.styles.para_styles.get(para.para_shape_id as usize);
            if let Some(ps) = ps {
                eprintln!("  alignment: {:?}", ps.alignment);
            }
        }

        // 페이지네이션 결과 확인: 2단 문단이 어떤 단에 배치되는지
        eprintln!("\n=== 페이지네이션 결과 분석 ===");
        let paginator = crate::renderer::pagination::Paginator::new(doc2.dpi);
        let composed2: Vec<_> = section2.paragraphs.iter()
            .map(|p| crate::renderer::composer::compose_paragraph(p))
            .collect();
        // 2단 ColumnDef 찾기 (para[9]+ 영역)
        let cd_for_9 = HwpDocument::find_column_def_for_paragraph(&section2.paragraphs, 9);
        eprintln!("para[9]+ ColumnDef: count={}", cd_for_9.column_count);

        // 페이지네이션 실행 (전체 섹션)
        let (pag_result, measured_sec) = paginator.paginate(
            &section2.paragraphs, &composed2, &doc2.styles,
            &section2.section_def.page_def,
            &crate::model::page::ColumnDef::default(), // 초기 ColumnDef
            0);

        // 측정 높이 진단
        eprintln!("\n=== 문단별 측정 높이 (para 0~20) ===");
        let mut zone1_sum: f64 = 0.0;
        for pi in 0..std::cmp::min(20, section2.paragraphs.len()) {
            let h = measured_sec.get_paragraph_height(pi).unwrap_or(0.0);
            let mp = measured_sec.get_measured_paragraph(pi);
            let sp_b = mp.map(|m| m.spacing_before).unwrap_or(0.0);
            let sp_a = mp.map(|m| m.spacing_after).unwrap_or(0.0);
            let lh_sum: f64 = mp.map(|m| m.line_heights.iter().sum()).unwrap_or(0.0);
            let line_ct = mp.map(|m| m.line_heights.len()).unwrap_or(0);
            eprintln!("  para[{}] h={:.2}px (sp_b={:.2} + lines({})={:.2} + sp_a={:.2})", pi, h, sp_b, line_ct, lh_sum, sp_a);
            if pi < 9 { zone1_sum += h; }
        }
        eprintln!("  zone1(para 0-8) sum={:.2}px", zone1_sum);
        let layout1 = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
            &section2.section_def.page_def, &crate::model::page::ColumnDef::default(), doc2.dpi);
        eprintln!("body_area.height={:.1}px, available_body_height={:.1}px",
            layout1.body_area.height, layout1.available_body_height());

        for (pg_idx, page) in pag_result.pages.iter().enumerate() {
            eprintln!("\n페이지 {} (단 수: {}):", pg_idx, page.column_contents.len());
            for col_content in &page.column_contents {
                eprintln!("  단 {} (zone_y_offset={:.1}):", col_content.column_index, col_content.zone_y_offset);
                for item in &col_content.items {
                    match item {
                        crate::renderer::pagination::PageItem::FullParagraph { para_index } => {
                            eprintln!("    FullParagraph(para={})", para_index);
                        }
                        crate::renderer::pagination::PageItem::PartialParagraph { para_index, start_line, end_line } => {
                            eprintln!("    PartialParagraph(para={}, lines={}..{})", para_index, start_line, end_line);
                        }
                        crate::renderer::pagination::PageItem::Table { para_index, control_index } => {
                            eprintln!("    Table(para={}, ctrl={})", para_index, control_index);
                        }
                        _ => {
                            eprintln!("    기타 항목");
                        }
                    }
                }
            }
        }

        // 검증: 페이지 0에 1단 + 2단 존이 공존해야 함 (다단 설정 나누기)
        let page0 = &pag_result.pages[0];
        let has_zone_offset = page0.column_contents.iter()
            .any(|cc| cc.zone_y_offset > 0.0);
        assert!(has_zone_offset,
            "페이지 0에 zone_y_offset > 0인 ColumnContent가 있어야 함 (1단+2단 공존)");
        let has_multi_col = page0.column_contents.iter()
            .any(|cc| cc.column_index > 0);
        assert!(has_multi_col,
            "페이지 0에 column_index > 0인 ColumnContent가 있어야 함 (2단 렌더링)");

        // === 페이지 1 높이 오버플로 진단 ===
        if pag_result.pages.len() > 1 {
            let page1 = &pag_result.pages[1];
            let avail = page1.layout.available_body_height();
            eprintln!("\n=== 페이지 1 높이 오버플로 진단 ===");
            eprintln!("available_body_height={:.2}px", avail);
            eprintln!("body_area: y={:.2}, h={:.2}, bottom={:.2}",
                page1.layout.body_area.y, page1.layout.body_area.height,
                page1.layout.body_area.y + page1.layout.body_area.height);

            for col_content in &page1.column_contents {
                eprintln!("\n  단 {} (zone_y_offset={:.1}):", col_content.column_index, col_content.zone_y_offset);
                let mut cumulative: f64 = 0.0;
                for item in &col_content.items {
                    match item {
                        crate::renderer::pagination::PageItem::FullParagraph { para_index } => {
                            let h = measured_sec.get_paragraph_height(*para_index).unwrap_or(0.0);
                            cumulative += h;
                            let mp = measured_sec.get_measured_paragraph(*para_index);
                            let sp_b = mp.map(|m| m.spacing_before).unwrap_or(0.0);
                            let sp_a = mp.map(|m| m.spacing_after).unwrap_or(0.0);
                            let lh_sum: f64 = mp.map(|m| m.line_heights.iter().sum()).unwrap_or(0.0);
                            let line_ct = mp.map(|m| m.line_heights.len()).unwrap_or(0);
                            eprintln!("    FullParagraph(para={}) h={:.2}px (sp_b={:.2} + lines({})={:.2} + sp_a={:.2}) cum={:.2}",
                                para_index, h, sp_b, line_ct, lh_sum, sp_a, cumulative);
                        }
                        crate::renderer::pagination::PageItem::PartialParagraph { para_index, start_line, end_line } => {
                            let mp = measured_sec.get_measured_paragraph(*para_index);
                            let (part_h, sp_b, sp_a, lh_sum) = if let Some(mp) = mp {
                                let sp_b = if *start_line == 0 { mp.spacing_before } else { 0.0 };
                                let sp_a = if *end_line >= mp.line_heights.len() { mp.spacing_after } else { 0.0 };
                                let safe_s = (*start_line).min(mp.line_heights.len());
                                let safe_e = (*end_line).min(mp.line_heights.len());
                                let lh: f64 = mp.line_heights[safe_s..safe_e].iter().sum();
                                (sp_b + lh + sp_a, sp_b, sp_a, lh)
                            } else {
                                (0.0, 0.0, 0.0, 0.0)
                            };
                            cumulative += part_h;
                            eprintln!("    PartialParagraph(para={}, lines={}..{}) h={:.2}px (sp_b={:.2} + lines={:.2} + sp_a={:.2}) cum={:.2}",
                                para_index, start_line, end_line, part_h, sp_b, lh_sum, sp_a, cumulative);
                        }
                        crate::renderer::pagination::PageItem::Table { para_index, control_index } => {
                            let h = measured_sec.get_paragraph_height(*para_index).unwrap_or(0.0);
                            cumulative += h;
                            eprintln!("    Table(para={}, ctrl={}) h={:.2}px cum={:.2}",
                                para_index, control_index, h, cumulative);
                        }
                        _ => {
                            eprintln!("    기타 항목");
                        }
                    }
                }
                let overflow = cumulative - avail;
                if overflow > 0.0 {
                    eprintln!("  *** 오버플로: {:.2}px (누적 {:.2} > 가용 {:.2})", overflow, cumulative, avail);
                } else {
                    eprintln!("  여유: {:.2}px (누적 {:.2} <= 가용 {:.2})", -overflow, cumulative, avail);
                }
            }
        }
    }

    /// 엔터키 후 저장 시 파일 손상 진단: blanK2020 원본 vs 손상 파일 비교
    #[test]
    fn test_blank2020_enter_corruption_diagnosis() {
        use crate::parser::cfb_reader::CfbReader;
        use crate::parser::record::Record;
        use crate::parser::tags;

        let files = [
            ("blanK2020 원본", "saved/blanK2020.hwp"),
            ("blanK2020 엔터후저장(손상)", "saved/blanK2020_enter_saved_currupt.hwp"),
        ];

        for (label, path) in &files {
            if !std::path::Path::new(path).exists() {
                eprintln!("SKIP: {} 파일 없음", path);
                continue;
            }

            let bytes = std::fs::read(path).unwrap();
            let mut cfb = CfbReader::open(&bytes).expect(&format!("{} CFB 열기 실패", label));

            eprintln!("\n{}", "=".repeat(80));
            eprintln!("  {} ({} bytes)", label, bytes.len());

            // DocInfo
            let doc_info_data = cfb.read_doc_info(true).expect("DocInfo 읽기 실패");
            let doc_recs = Record::read_all(&doc_info_data).unwrap();
            let cs_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_CHAR_SHAPE).count();
            let ps_count = doc_recs.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_SHAPE).count();
            eprintln!("  DocInfo: CS={} PS={} records={} bytes={}", cs_count, ps_count, doc_recs.len(), doc_info_data.len());

            // BodyText Section0
            let body_data = cfb.read_body_text_section(0, true, false).expect("BodyText 읽기 실패");
            let body_recs = Record::read_all(&body_data).unwrap();
            eprintln!("  BodyText: {} records, {} bytes", body_recs.len(), body_data.len());

            for (i, rec) in body_recs.iter().enumerate() {
                let indent = "  ".repeat(rec.level as usize);
                let tag_name = tags::tag_name(rec.tag_id);

                let extra = if rec.tag_id == tags::HWPTAG_PARA_HEADER {
                    let cc = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    let cm = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                    let ps_id = u16::from_le_bytes([rec.data[8], rec.data[9]]);
                    let char_count = cc & 0x7FFFFFFF;
                    let msb = cc >> 31;
                    format!(" cc={} msb={} cm=0x{:08X} ps={} data_len={} raw_extra={}",
                        char_count, msb, cm, ps_id, rec.data.len(),
                        rec.data.iter().skip(12).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "))
                } else if rec.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                    let mut segs_info = String::new();
                    let mut pos = 0;
                    let mut seg_idx = 0;
                    while pos + 36 <= rec.data.len() {
                        let lh = i32::from_le_bytes([rec.data[pos+8], rec.data[pos+9], rec.data[pos+10], rec.data[pos+11]]);
                        let th = i32::from_le_bytes([rec.data[pos+12], rec.data[pos+13], rec.data[pos+14], rec.data[pos+15]]);
                        let sw = i32::from_le_bytes([rec.data[pos+28], rec.data[pos+29], rec.data[pos+30], rec.data[pos+31]]);
                        let tag = u32::from_le_bytes([rec.data[pos+32], rec.data[pos+33], rec.data[pos+34], rec.data[pos+35]]);
                        segs_info += &format!(" [seg{}: lh={} th={} sw={} tag=0x{:08X}]", seg_idx, lh, th, sw, tag);
                        seg_idx += 1;
                        pos += 36;
                    }
                    segs_info
                } else if rec.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE {
                    let mut ids = Vec::new();
                    let mut pos = 0;
                    while pos + 8 <= rec.data.len() {
                        let cs_id = u32::from_le_bytes([rec.data[pos+4], rec.data[pos+5], rec.data[pos+6], rec.data[pos+7]]);
                        ids.push(cs_id);
                        pos += 8;
                    }
                    format!(" cs_ids={:?}", ids)
                } else if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                    let hex: String = rec.data.iter().take(20).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    format!(" [{}]", hex)
                } else {
                    String::new()
                };

                eprintln!("  rec[{:3}] {}L{} {} ({}B){}", i, indent, rec.level, tag_name, rec.data.len(), extra);
            }
        }

        // 추가: 우리 파서로 로드 → split → export → 다시 레코드 비교
        let blank_path = "saved/blanK2020.hwp";
        if std::path::Path::new(blank_path).exists() {
            eprintln!("\n{}", "=".repeat(80));
            eprintln!("  === split_at 라운드트립 테스트 ===");
            let bytes = std::fs::read(blank_path).unwrap();
            let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
            doc.convert_to_editable_native().unwrap();

            // 원본 문단 정보
            let para = &doc.document.sections[0].paragraphs[0];
            eprintln!("  원본 para[0]: text='{}' cc={} raw_header_extra({} bytes): {:02x?}",
                para.text, para.char_count, para.raw_header_extra.len(), &para.raw_header_extra);
            eprintln!("  원본 para[0] line_segs[0].tag = 0x{:08X}", para.line_segs.first().map(|ls| ls.tag).unwrap_or(0));

            // 엔터 (split at 0)
            let result = doc.split_paragraph_native(0, 0, 0);
            eprintln!("  split result: {:?}", result);

            // 분할 후 문단 정보
            for (i, p) in doc.document.sections[0].paragraphs.iter().enumerate() {
                eprintln!("  split 후 para[{}]: text='{}' cc={} has_para_text={} raw_header_extra({} bytes): {:02x?}",
                    i, p.text, p.char_count, p.has_para_text, p.raw_header_extra.len(), &p.raw_header_extra);
                if let Some(ls) = p.line_segs.first() {
                    eprintln!("    line_seg: lh={} th={} bd={} sw={} tag=0x{:08X}",
                        ls.line_height, ls.text_height, ls.baseline_distance, ls.segment_width, ls.tag);
                }
            }

            // export
            let exported = doc.export_hwp_native().unwrap();
            eprintln!("  exported: {} bytes", exported.len());

            // re-parse exported
            let mut cfb2 = CfbReader::open(&exported).expect("재파싱 CFB 열기 실패");
            let body2 = cfb2.read_body_text_section(0, true, false).expect("재파싱 BodyText 실패");
            let recs2 = Record::read_all(&body2).unwrap();
            eprintln!("  재파싱 BodyText: {} records, {} bytes", recs2.len(), body2.len());

            for (i, rec) in recs2.iter().enumerate() {
                let tag_name = tags::tag_name(rec.tag_id);
                if rec.tag_id == tags::HWPTAG_PARA_HEADER {
                    eprintln!("  re-rec[{:3}] L{} {} ({}B) raw_extra={}",
                        i, rec.level, tag_name, rec.data.len(),
                        rec.data.iter().skip(12).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                } else if rec.tag_id == tags::HWPTAG_PARA_LINE_SEG {
                    let mut pos = 0;
                    while pos + 36 <= rec.data.len() {
                        let tag = u32::from_le_bytes([rec.data[pos+32], rec.data[pos+33], rec.data[pos+34], rec.data[pos+35]]);
                        eprintln!("  re-rec[{:3}] L{} {} ({}B) tag=0x{:08X}",
                            i, rec.level, tag_name, rec.data.len(), tag);
                        pos += 36;
                    }
                }
            }
        }
    }

    /// 빈 문단에서 반복 Enter + getCursorRect 동작 검증
    #[test]
    fn test_repeated_enter_on_empty_paragraph() {
        let bytes = std::fs::read("saved/blank2010.hwp").expect("blank2010.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        // 1. 텍스트 입력
        let result = doc.insert_text_native(0, 0, 0, "테스트").unwrap();
        println!("Insert: {}", result);

        // 2. 첫 번째 Enter (텍스트 끝에서)
        let result1 = doc.split_paragraph_native(0, 0, 3).unwrap();
        println!("Split 1 (para=0, offset=3): {}", result1);
        assert!(result1.contains("\"ok\":true"));
        assert_eq!(doc.document.sections[0].paragraphs.len(), 2);

        // getCursorRect para 1, offset 0
        let rect1 = doc.get_cursor_rect_native(0, 1, 0);
        println!("CursorRect(0,1,0): {:?}", rect1);
        assert!(rect1.is_ok(), "빈 문단(para=1) 커서 실패: {:?}", rect1.err());

        // 3. 두 번째 Enter (빈 문단에서)
        let result2 = doc.split_paragraph_native(0, 1, 0).unwrap();
        println!("Split 2 (para=1, offset=0): {}", result2);
        assert!(result2.contains("\"ok\":true"));
        assert!(result2.contains("\"paraIdx\":2"));
        assert_eq!(doc.document.sections[0].paragraphs.len(), 3);

        let rect2 = doc.get_cursor_rect_native(0, 2, 0);
        println!("CursorRect(0,2,0): {:?}", rect2);
        assert!(rect2.is_ok(), "빈 문단(para=2) 커서 실패: {:?}", rect2.err());

        // 4. 세 번째 Enter
        let result3 = doc.split_paragraph_native(0, 2, 0).unwrap();
        println!("Split 3 (para=2, offset=0): {}", result3);
        assert!(result3.contains("\"ok\":true"));

        let rect3 = doc.get_cursor_rect_native(0, 3, 0);
        println!("CursorRect(0,3,0): {:?}", rect3);
        assert!(rect3.is_ok(), "빈 문단(para=3) 커서 실패: {:?}", rect3.err());

        // y좌표 순증 검증
        let parse_y = |json: &str| -> f64 {
            let y_start = json.find("\"y\":").unwrap() + 4;
            let y_end = json[y_start..].find(|c: char| c == ',' || c == '}').unwrap();
            json[y_start..y_start + y_end].parse::<f64>().unwrap()
        };
        let y1 = parse_y(&rect1.unwrap());
        let y2 = parse_y(&rect2.unwrap());
        let y3 = parse_y(&rect3.unwrap());
        println!("y좌표: y1={:.1}, y2={:.1}, y3={:.1}", y1, y2, y3);
        assert!(y2 > y1, "para2 y({:.1}) > para1 y({:.1})", y2, y1);
        assert!(y3 > y2, "para3 y({:.1}) > para2 y({:.1})", y3, y2);
    }

    /// 강제 줄바꿈(\n) 삽입 후 getCursorRect가 두 번째 줄 좌표를 반환하는지 검증
    #[test]
    fn test_cursor_rect_after_line_break() {
        let bytes = std::fs::read("saved/blank2010.hwp").expect("blank2010.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        // "가나다라마바" 입력
        doc.insert_text_native(0, 0, 0, "가나다라마바").unwrap();

        // offset 3에 \n 삽입 → "가나다\n라마바"
        doc.insert_text_native(0, 0, 3, "\n").unwrap();

        // offset 3 → \n 이전 (첫 줄)
        let rect_before = doc.get_cursor_rect_native(0, 0, 3);
        assert!(rect_before.is_ok(), "offset 3 커서 실패: {:?}", rect_before.err());

        // offset 4 → \n 이후 (두 번째 줄)
        let rect_after = doc.get_cursor_rect_native(0, 0, 4);
        assert!(rect_after.is_ok(), "offset 4 커서 실패: {:?}", rect_after.err());

        let parse_y = |json: &str| -> f64 {
            let y_start = json.find("\"y\":").unwrap() + 4;
            let y_end = json[y_start..].find(|c: char| c == ',' || c == '}').unwrap();
            json[y_start..y_start + y_end].parse::<f64>().unwrap()
        };
        let y_before = parse_y(&rect_before.unwrap());
        let y_after = parse_y(&rect_after.unwrap());
        assert!(y_after > y_before,
                "줄바꿈 후 커서 y({:.1})가 줄바꿈 전 y({:.1})보다 커야 함", y_after, y_before);
    }

    /// 텍스트 끝에 \n 삽입 후 빈 두 번째 줄에서 getCursorRect 검증
    #[test]
    fn test_cursor_rect_after_line_break_at_end() {
        let bytes = std::fs::read("saved/blank2010.hwp").expect("blank2010.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        // "가나다라" 입력 후 끝에 \n 삽입 → "가나다라\n"
        doc.insert_text_native(0, 0, 0, "가나다라").unwrap();
        doc.insert_text_native(0, 0, 4, "\n").unwrap();

        let para = &doc.document.sections[0].paragraphs[0];
        assert!(para.line_segs.len() >= 2, "line_segs가 2개 이상이어야 함");

        // composed lines 순서 검증: 첫 줄=텍스트, 둘째 줄=빈 줄
        let comp = &doc.composed[0][0];
        assert_eq!(comp.lines.len(), 2);
        assert!(comp.lines[0].has_line_break, "첫 줄에 line_break 플래그 있어야 함");
        assert_eq!(comp.lines[1].runs.len(), 0, "둘째 줄은 빈 줄이어야 함");

        // offset 4 → \n 위치 (첫 줄 끝)
        let rect_at_newline = doc.get_cursor_rect_native(0, 0, 4);
        assert!(rect_at_newline.is_ok());

        // offset 5 → \n 직후, 빈 두 번째 줄
        let rect_after = doc.get_cursor_rect_native(0, 0, 5);
        assert!(rect_after.is_ok(), "빈 줄 offset 5 커서 실패: {:?}", rect_after.err());

        let parse_y = |json: &str| -> f64 {
            let y_start = json.find("\"y\":").unwrap() + 4;
            let y_end = json[y_start..].find(|c: char| c == ',' || c == '}').unwrap();
            json[y_start..y_start + y_end].parse::<f64>().unwrap()
        };
        let y_newline = parse_y(&rect_at_newline.unwrap());
        let y_after = parse_y(&rect_after.unwrap());
        assert!(y_after > y_newline,
                "빈 줄 커서 y({:.1})가 첫 줄 y({:.1})보다 커야 함", y_after, y_newline);
    }

    // ── Event Sourcing + Batch Mode 테스트 ──

    /// 편집 가능한 빈 문서 생성 헬퍼 (blank 템플릿 기반)
    fn create_editable_doc() -> HwpDocument {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        doc
    }

    #[test]
    fn test_single_command_emits_event() {
        let mut doc = create_editable_doc();
        assert!(doc.event_log.is_empty());

        let result = doc.insert_text_native(0, 0, 0, "Hello");
        assert!(result.is_ok(), "insert_text_native failed: {:?}", result);
        assert_eq!(doc.event_log.len(), 1);

        let json = doc.event_log[0].to_json();
        assert!(json.contains("\"type\":\"TextInserted\""));
        assert!(json.contains("\"section\":0"));
        assert!(json.contains("\"para\":0"));
        assert!(json.contains("\"offset\":0"));
        assert!(json.contains("\"len\":5"));
    }

    #[test]
    fn test_batch_mode_events_collected() {
        let mut doc = create_editable_doc();

        let r = doc.begin_batch_native();
        assert!(r.is_ok());
        assert!(doc.batch_mode);
        assert!(doc.event_log.is_empty());

        // Batch 중 여러 편집
        let r1 = doc.insert_text_native(0, 0, 0, "Hello");
        assert!(r1.is_ok(), "1st insert failed: {:?}", r1);
        let r2 = doc.insert_text_native(0, 0, 5, " World");
        assert!(r2.is_ok(), "2nd insert failed: {:?}", r2);

        assert_eq!(doc.event_log.len(), 2);
        assert!(doc.event_log[0].to_json().contains("\"type\":\"TextInserted\""));
        assert!(doc.event_log[1].to_json().contains("\"type\":\"TextInserted\""));
    }

    #[test]
    fn test_end_batch_returns_events_and_clears() {
        let mut doc = create_editable_doc();

        let _ = doc.begin_batch_native();
        let r = doc.insert_text_native(0, 0, 0, "Test");
        assert!(r.is_ok(), "insert failed: {:?}", r);
        assert_eq!(doc.event_log.len(), 1);

        let result = doc.end_batch_native();
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"events\":["));
        assert!(json.contains("\"type\":\"TextInserted\""));

        // end_batch 후 event_log 비워짐 + batch_mode 해제
        assert!(!doc.batch_mode);
        assert!(doc.event_log.is_empty());
    }

    #[test]
    fn test_batch_multiple_edit_types() {
        let mut doc = create_editable_doc();

        let _ = doc.begin_batch_native();
        let r1 = doc.insert_text_native(0, 0, 0, "Hello World");
        assert!(r1.is_ok(), "insert failed: {:?}", r1);
        let r2 = doc.delete_text_native(0, 0, 5, 6);
        assert!(r2.is_ok(), "delete failed: {:?}", r2);

        assert_eq!(doc.event_log.len(), 2);
        assert!(doc.event_log[0].to_json().contains("\"type\":\"TextInserted\""));
        assert!(doc.event_log[1].to_json().contains("\"type\":\"TextDeleted\""));

        let result = doc.end_batch_native();
        assert!(result.is_ok());
        // 종료 후 paginate 실행되므로 페이지 수 유효
        assert!(doc.page_count() >= 1);
    }

    #[test]
    fn test_serialize_event_log_format() {
        let mut doc = create_editable_doc();
        let r = doc.insert_text_native(0, 0, 0, "A");
        assert!(r.is_ok(), "insert failed: {:?}", r);

        let json = doc.serialize_event_log();
        assert!(json.starts_with("{\"ok\":true,\"events\":["));
        assert!(json.ends_with("]}"));
        assert!(json.contains("\"type\":\"TextInserted\""));
    }

    #[test]
    fn test_find_next_editable_control_bookreview() {
        let data = std::fs::read("samples/basic/BookReview.hwp")
            .expect("BookReview.hwp not found");
        let doc = HwpDocument::from_bytes(&data).unwrap();

        // Section 1, Para 0: controls 0-8 중 textbox는 ci=3,4,5,6,7,8
        // ci=3에서 앞으로 → ci=4 (textbox)
        let r = doc.find_next_editable_control_native(1, 0, 3, 1);
        println!("sec1 para0 ci=3 → next: {}", r);
        assert!(r.contains("\"type\":\"textbox\""));
        assert!(r.contains("\"ci\":4"));

        // ci=8에서 앞으로 → 같은 문단에 더 이상 없음 → 다음 문단/섹션
        let r = doc.find_next_editable_control_native(1, 0, 8, 1);
        println!("sec1 para0 ci=8 → next: {}", r);
        // section 1에 paragraph가 1개뿐이므로 다음 섹션도 없음 → none
        assert!(r.contains("\"type\":\"none\""));

        // ci=3에서 뒤로 → 같은 문단에 ci=3 이전 textbox 없음 → 이전 섹션
        let r = doc.find_next_editable_control_native(1, 0, 3, -1);
        println!("sec1 para0 ci=3 → prev: {}", r);
        // section 0의 마지막에서 편집 가능한 위치
        assert!(r.contains("\"sec\":0"));

        // Section 0에서 앞으로: section 0의 마지막 문단에서 section 1로 이동
        let sec0_paras = doc.core.document.sections[0].paragraphs.len();
        let r = doc.find_next_editable_control_native(0, sec0_paras - 1, -1, 1);
        println!("sec0 last_para body → next: {}", r);

        // ci=5에서 앞으로 → ci=6
        let r = doc.find_next_editable_control_native(1, 0, 5, 1);
        println!("sec1 para0 ci=5 → next: {}", r);
        assert!(r.contains("\"ci\":6"));

        // ci=6에서 앞으로 → ci=7
        let r = doc.find_next_editable_control_native(1, 0, 6, 1);
        println!("sec1 para0 ci=6 → next: {}", r);
        assert!(r.contains("\"ci\":7"));

        // ci=7에서 앞으로 → ci=8
        let r = doc.find_next_editable_control_native(1, 0, 7, 1);
        println!("sec1 para0 ci=7 → next: {}", r);
        assert!(r.contains("\"ci\":8"));

        // ci=8에서 뒤로 → ci=7
        let r = doc.find_next_editable_control_native(1, 0, 8, -1);
        println!("sec1 para0 ci=8 → prev: {}", r);
        assert!(r.contains("\"ci\":7"));
    }

    #[test]
    fn test_superscript_in_new_document() {
        // 새 문서 생성 → 텍스트 입력 → 숫자 삽입 → 위첨자 적용 → 이후 글자 정상 확인
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();

        // 1. "가나다라마바사" 입력 (실제로는 한 번에 삽입)
        let _ = doc.insert_text_native(0, 0, 0, "가나다라마바사");

        let para = &doc.document.sections[0].paragraphs[0];
        eprintln!("Step1: text='{}' char_offsets={:?} char_shapes={:?}",
            para.text,
            para.char_offsets,
            para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>(),
        );

        // 2. 위치 2에 "123" 삽입 → "가나123다라마바사"
        let _ = doc.insert_text_native(0, 0, 2, "123");

        let para = &doc.document.sections[0].paragraphs[0];
        eprintln!("Step2: text='{}' char_offsets={:?} char_shapes={:?}",
            para.text,
            para.char_offsets,
            para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>(),
        );

        // 3. "123" (chars 2-5)에 위첨자 적용
        let result = doc.apply_char_format_native(0, 0, 2, 5, r#"{"superscript":true}"#);
        assert!(result.is_ok(), "위첨자 적용 실패: {:?}", result.err());

        let para = &doc.document.sections[0].paragraphs[0];
        eprintln!("Step3: text='{}' char_offsets={:?} char_shapes={:?}",
            para.text,
            para.char_offsets,
            para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>(),
        );

        // 검증: char_shapes가 3개여야 함 (원본, 위첨자, 원본)
        assert!(para.char_shapes.len() >= 3,
            "char_shapes should have at least 3 segments, got {}: {:?}",
            para.char_shapes.len(),
            para.char_shapes.iter().map(|cs| (cs.start_pos, cs.char_shape_id)).collect::<Vec<_>>(),
        );

        // 위첨자가 적용된 CharShape와 원본 CharShape가 다른 ID인지 확인
        let original_id = para.char_shapes[0].char_shape_id;
        let superscript_id = para.char_shapes[1].char_shape_id;
        assert_ne!(original_id, superscript_id, "위첨자 CharShape ID는 원본과 달라야 함");

        // 마지막 세그먼트는 원본 ID로 복원되어야 함
        let last_id = para.char_shapes.last().unwrap().char_shape_id;
        assert_eq!(last_id, original_id, "위첨자 이후 원본 ID로 복원되어야 함");

        // 위첨자 CharShape의 superscript 필드 확인
        let sup_cs = &doc.document.doc_info.char_shapes[superscript_id as usize];
        assert!(sup_cs.superscript, "위첨자 CharShape의 superscript가 true여야 함");

        // 원본 CharShape의 superscript 필드 확인
        let orig_cs = &doc.document.doc_info.char_shapes[original_id as usize];
        assert!(!orig_cs.superscript, "원본 CharShape의 superscript가 false여야 함");
    }

    /// Task 227: 빈 문서에서 텍스트 입력 → 전체선택 → 복사 → End → 붙여넣기 시
    /// 새 페이지 생성 버그 재현 및 원인 분석
    #[test]
    fn test_task227_blank_doc_copy_paste_bug() {
        let mut doc = HwpDocument::create_empty();
        let result = doc.create_blank_document_native();
        assert!(result.is_ok(), "빈 문서 생성 실패");

        // 1. 빈 문서의 문단 수 확인
        let para_count = doc.document.sections[0].paragraphs.len();
        eprintln!("[Task227] 빈 문서 문단 수: {}", para_count);
        for (i, p) in doc.document.sections[0].paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text={:?}, chars={}, controls={}, has_para_text={}",
                i, p.text, p.text.chars().count(), p.controls.len(), p.has_para_text);
        }

        // 2. 텍스트 삽입
        let result = doc.insert_text_native(0, 0, 0, "abcdefg");
        assert!(result.is_ok(), "텍스트 삽입 실패");

        let para_count_after_insert = doc.document.sections[0].paragraphs.len();
        eprintln!("[Task227] 텍스트 삽입 후 문단 수: {}", para_count_after_insert);
        for (i, p) in doc.document.sections[0].paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text={:?}, chars={}, controls={}, has_para_text={}",
                i, p.text, p.text.chars().count(), p.controls.len(), p.has_para_text);
        }

        // 3. 전체 선택 시뮬레이션: start=(0,0,0), end=(last_para, last_char)
        let last_para = para_count_after_insert - 1;
        let last_char = doc.document.sections[0].paragraphs[last_para].text.chars().count();
        eprintln!("[Task227] 전체 선택: start=(0,0), end=({},{})", last_para, last_char);

        // 4. 복사
        let result = doc.copy_selection_native(0, 0, 0, last_para, last_char);
        assert!(result.is_ok(), "복사 실패: {:?}", result.err());
        let clip_text = doc.get_clipboard_text_native();
        eprintln!("[Task227] 클립보드 텍스트: {:?}", clip_text);

        // 클립보드 문단 수 확인
        if let Some(ref clip) = doc.clipboard {
            eprintln!("[Task227] 클립보드 문단 수: {}", clip.paragraphs.len());
            for (i, p) in clip.paragraphs.iter().enumerate() {
                eprintln!("  클립[{}]: text={:?}, chars={}, controls={}",
                    i, p.text, p.text.chars().count(), p.controls.len());
            }
        }

        // 5. End 키 시뮬레이션: 커서를 문단 0의 텍스트 끝으로 이동
        //    (원래 커서는 문단 0, offset 7 = "abcdefg" 끝)
        let paste_offset = doc.document.sections[0].paragraphs[0].text.chars().count();
        eprintln!("[Task227] 붙여넣기 위치: para=0, offset={}", paste_offset);

        // 6. 붙여넣기
        let result = doc.paste_internal_native(0, 0, paste_offset);
        assert!(result.is_ok(), "붙여넣기 실패: {:?}", result.err());
        let json = result.unwrap();
        eprintln!("[Task227] 붙여넣기 결과: {}", json);

        // 7. 결과 확인
        let final_para_count = doc.document.sections[0].paragraphs.len();
        eprintln!("[Task227] 붙여넣기 후 문단 수: {}", final_para_count);
        for (i, p) in doc.document.sections[0].paragraphs.iter().enumerate() {
            eprintln!("  문단[{}]: text={:?}, chars={}", i, p.text, p.text.chars().count());
        }

        let page_count = doc.page_count();
        eprintln!("[Task227] 붙여넣기 후 페이지 수: {}", page_count);

        // 기대: 1개 문단, 1 페이지
        assert_eq!(final_para_count, 1, "문단 수가 1이어야 함 (실제: {})", final_para_count);
        assert_eq!(page_count, 1, "페이지 수가 1이어야 함 (실제: {})", page_count);
    }

    /// Task 228: h-pen-01.hwp 형광펜 데이터 분석
    #[test]
    fn test_task228_highlight_data_analysis() {
        let data = std::fs::read("samples/h-pen-01.hwp").expect("파일 읽기 실패");
        let doc = crate::parser::parse_hwp(&data).expect("파싱 실패");

        // CharShape shade_color 확인
        eprintln!("[Task228] CharShape 수: {}", doc.doc_info.char_shapes.len());
        for (i, cs) in doc.doc_info.char_shapes.iter().enumerate() {
            eprintln!("  CS[{}]: shade_color=0x{:06X}", i, cs.shade_color);
        }

        // 문단별 range_tags 확인
        for (si, section) in doc.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                if !para.range_tags.is_empty() {
                    eprintln!("[Task228] 구역[{}] 문단[{}] range_tags:", si, pi);
                    for rt in &para.range_tags {
                        let tag_type = (rt.tag >> 24) & 0xFF;
                        let tag_data = rt.tag & 0x00FFFFFF;
                        eprintln!("  start={}, end={}, tag=0x{:08X} (type={}, data=0x{:06X})",
                            rt.start, rt.end, rt.tag, tag_type, tag_data);
                    }
                }
                // char_shapes 참조 확인
                for csr in &para.char_shapes {
                    let cs_id = csr.char_shape_id as usize;
                    if cs_id < doc.doc_info.char_shapes.len() {
                        let sc = doc.doc_info.char_shapes[cs_id].shade_color;
                        if sc != 0xFFFFFF && sc != 0x00FFFFFF {
                            eprintln!("  문단[{}] char_shape_ref: start={}, cs_id={}, shade_color=0x{:06X}",
                                pi, csr.start_pos, cs_id, sc);
                        }
                    }
                }
            }
        }
    }

    /// Task 228: 형광펜 렌더링 - 페이지 트리에 Rectangle 노드 확인
    #[test]
    fn test_task228_highlight_render_tree() {
        let data = std::fs::read("samples/h-pen-01.hwp").expect("파일 읽기 실패");
        let mut doc = crate::DocumentCore::from_bytes(&data).expect("파싱 실패");
        let svg = doc.render_page_svg_native(0).expect("SVG 렌더링 실패");
        // 형광펜 사각형 색상이 SVG에 포함되어야 함
        assert!(svg.contains("#ad71a1"), "2번째 문단 형광펜 색상(#ad71a1)이 SVG에 없음");
        assert!(svg.contains("#ffff65"), "3번째 문단 형광펜 색상(#ffff65)이 SVG에 없음");
        eprintln!("[Task228 RenderTree] SVG에 형광펜 색상 확인됨");
    }

    /// Task 229: field-01.hwp 필드 컨트롤 파싱 분석
    #[test]
    fn test_task229_field_parsing() {
        use crate::model::control::{Control, FieldType};

        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc = crate::parser::parse_hwp(&data).expect("파싱 실패");

        let mut field_count = 0;
        let mut unknown_count = 0;

        for (si, section) in doc.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Field(f) => {
                            field_count += 1;
                            eprintln!(
                                "[Task229] 구역[{}] 문단[{}] 컨트롤[{}]: Field type={:?}, command=\"{}\", id={}, props=0x{:08X}",
                                si, pi, ci, f.field_type, f.command, f.field_id, f.properties
                            );
                        }
                        Control::Unknown(u) => {
                            let id_bytes = u.ctrl_id.to_be_bytes();
                            if id_bytes[0] == b'%' {
                                unknown_count += 1;
                                eprintln!(
                                    "[Task229] 구역[{}] 문단[{}] 컨트롤[{}]: Unknown 필드 ctrl_id=0x{:08X} ({})",
                                    si, pi, ci, u.ctrl_id,
                                    String::from_utf8_lossy(&id_bytes)
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        eprintln!("[Task229] 총 필드: {}, Unknown 필드: {}", field_count, unknown_count);
        assert!(field_count > 0, "필드 컨트롤이 파싱되어야 함");
        assert_eq!(unknown_count, 0, "모든 필드가 파싱되어야 함 (Unknown 없어야 함)");

        // 필드 범위 추적 검증
        let mut total_field_ranges = 0;
        for (si, section) in doc.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                if !para.field_ranges.is_empty() {
                    eprintln!("[Task229] 구역[{}] 문단[{}] text=\"{}\" (len={})", si, pi, para.text, para.text.chars().count());
                }
                for fr in &para.field_ranges {
                    total_field_ranges += 1;
                    let field_text: String = para.text.chars()
                        .skip(fr.start_char_idx)
                        .take(fr.end_char_idx - fr.start_char_idx)
                        .collect();
                    let field_type = match &para.controls[fr.control_idx] {
                        Control::Field(f) => format!("{:?}", f.field_type),
                        _ => "N/A".to_string(),
                    };
                    eprintln!(
                        "[Task229] 구역[{}] 문단[{}] field_range: chars[{}..{}] ctrl[{}] type={} text=\"{}\"",
                        si, pi, fr.start_char_idx, fr.end_char_idx, fr.control_idx, field_type, field_text
                    );
                }
            }
        }
        eprintln!("[Task229] 총 필드 범위: {}", total_field_ranges);
        assert_eq!(total_field_ranges, field_count, "필드 수와 필드 범위 수가 일치해야 함");
    }

    #[test]
    fn test_task229_field_roundtrip() {
        use crate::model::control::{Control, FieldType};

        // 원본 파싱
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc1 = crate::parser::parse_hwp(&data).expect("파싱 실패");

        // 직렬화 → 재파싱
        let saved = crate::serializer::serialize_hwp(&doc1).expect("직렬화 실패");
        let doc2 = crate::parser::parse_hwp(&saved).expect("재파싱 실패");

        // 필드 컨트롤 비교
        let fields1: Vec<_> = doc1.sections.iter().flat_map(|s| &s.paragraphs)
            .flat_map(|p| p.controls.iter())
            .filter_map(|c| if let Control::Field(f) = c { Some(f) } else { None })
            .collect();
        let fields2: Vec<_> = doc2.sections.iter().flat_map(|s| &s.paragraphs)
            .flat_map(|p| p.controls.iter())
            .filter_map(|c| if let Control::Field(f) = c { Some(f) } else { None })
            .collect();

        assert_eq!(fields1.len(), fields2.len(), "필드 수 불일치");
        for (i, (f1, f2)) in fields1.iter().zip(fields2.iter()).enumerate() {
            assert_eq!(f1.field_type, f2.field_type, "필드[{}] 타입 불일치", i);
            assert_eq!(f1.ctrl_id, f2.ctrl_id, "필드[{}] ctrl_id 불일치", i);
        }
    }

    #[test]
    fn test_task229_field_svg_guide_text() {
        use crate::model::control::Control;

        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc = crate::parser::parse_hwp(&data).expect("파싱 실패");

        // 글상자(Shape) 내 ClickHere 필드 검증
        let mut shape_field_count = 0usize;
        for sec in &doc.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Shape(s) = ctrl {
                        if let Some(drawing) = s.drawing() {
                            if let Some(tb) = &drawing.text_box {
                                for tb_para in &tb.paragraphs {
                                    shape_field_count += tb_para.field_ranges.len();
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(shape_field_count >= 5, "글상자 내 필드가 5개 이상이어야 함 (실제: {})", shape_field_count);

        let mut hwp_doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");
        let svg = hwp_doc.render_page_svg_native(0).expect("SVG 렌더링 실패");

        // SVG에 안내문 텍스트가 빨간색 기울임체로 렌더링되는지 확인
        assert!(svg.contains("ff0000"), "SVG에 빨간색(#ff0000) 텍스트가 있어야 함");
        assert!(svg.contains("italic"), "SVG에 기울임체 텍스트가 있어야 함");
        assert!(svg.contains(">여</text>"), "SVG에 '여' 글자가 있어야 함");
        assert!(svg.contains(">입</text>"), "SVG에 '입' 글자가 있어야 함");
    }

    // ─── Task 230: 필드 WASM API 테스트 ─────────────────────────

    #[test]
    fn test_task230_get_field_list() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let hwp_doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        let json = hwp_doc.get_field_list_json();
        eprintln!("[Task230] getFieldList: {}", json);

        // JSON 배열이어야 함
        assert!(json.starts_with('[') && json.ends_with(']'), "JSON 배열이어야 함");
        // 최소 6개 필드 (본문 5 + 글상자 내 5 + 기타)
        let field_count = json.matches("\"fieldId\"").count();
        assert!(field_count >= 6, "필드가 6개 이상이어야 함 (실제: {})", field_count);
        // ClickHere 필드 포함 확인
        assert!(json.contains("\"clickhere\""), "ClickHere 필드가 있어야 함");
    }

    #[test]
    fn test_task230_get_field_value() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let hwp_doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        // 필드 목록에서 첫 번째 필드 ID 추출
        let json = hwp_doc.get_field_list_json();
        let fields = hwp_doc.collect_all_fields();
        assert!(!fields.is_empty(), "필드가 있어야 함");

        let first_field = &fields[0];
        eprintln!("[Task230] 첫 번째 필드: id={}, type={:?}, name={:?}, value='{}'",
            first_field.field.field_id, first_field.field.field_type,
            first_field.field.field_name(), first_field.value);

        // field_id로 조회
        let result = hwp_doc.get_field_value_by_id(first_field.field.field_id)
            .expect("필드 값 조회 실패");
        assert!(result.contains("\"ok\":true"), "조회 성공이어야 함");

        // 이름으로 조회
        if let Some(name) = first_field.field.field_name() {
            let result = hwp_doc.get_field_value_by_name(name)
                .expect("이름으로 필드 값 조회 실패");
            assert!(result.contains("\"ok\":true"), "이름 조회 성공이어야 함");
        }
    }

    #[test]
    fn test_task230_set_field_value() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let mut hwp_doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        let fields = hwp_doc.collect_all_fields();
        // 빈 ClickHere 필드 찾기 (value가 빈 것)
        let empty_field = fields.iter().find(|f| {
            f.field.field_type == crate::model::control::FieldType::ClickHere && f.value.is_empty()
        }).expect("빈 ClickHere 필드가 있어야 함");

        let field_id = empty_field.field.field_id;
        eprintln!("[Task230] 빈 필드에 값 설정: id={}, name={:?}",
            field_id, empty_field.field.field_name());

        // 값 설정
        let result = hwp_doc.set_field_value_by_id(field_id, "테스트 입력값")
            .expect("필드 값 설정 실패");
        eprintln!("[Task230] setFieldValue 결과: {}", result);
        assert!(result.contains("\"ok\":true"), "설정 성공이어야 함");
        assert!(result.contains("테스트 입력값"), "새 값이 포함되어야 함");

        // 값이 변경되었는지 확인
        let check = hwp_doc.get_field_value_by_id(field_id)
            .expect("변경 후 조회 실패");
        assert!(check.contains("테스트 입력값"), "변경된 값이 반영되어야 함");

        // SVG 렌더링에서 변경된 값이 보이는지 확인
        let svg = hwp_doc.render_page_svg_native(0).expect("SVG 렌더링 실패");
        // "테스트 입력값"의 개별 글자가 SVG에 포함되어야 함
        assert!(svg.contains(">테</text>") || svg.contains("테스트"),
            "SVG에 변경된 텍스트가 있어야 함");
    }

    #[test]
    fn test_task231_field_survives_text_insert() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        // Section 0, Para 7: 빈 누름틀 필드 (start=7, end=7)
        let info_before = doc.get_field_info_at(0, 7, 7);
        eprintln!("[Before] field_info_at(0,7,7): {}", info_before);
        assert!(info_before.contains("\"inField\":true"), "삽입 전 필드가 있어야 함");

        // 필드 위치(charOffset=7)에 "A" 삽입
        let result = doc.insert_text_native(0, 7, 7, "A").expect("텍스트 삽입 실패");
        eprintln!("[After insert] result: {}", result);

        // 삽입 후 커서 위치(charOffset=8)에서 필드 확인
        let info_after = doc.get_field_info_at(0, 7, 8);
        eprintln!("[After] field_info_at(0,7,8): {}", info_after);
        assert!(info_after.contains("\"inField\":true"), "삽입 후에도 필드가 있어야 함");

        // 필드 시작 위치에서도 확인
        let info_start = doc.get_field_info_at(0, 7, 7);
        eprintln!("[After] field_info_at(0,7,7): {}", info_start);
        assert!(info_start.contains("\"inField\":true"), "삽입 후 필드 시작도 감지되어야 함");

        // field_ranges 직접 확인
        let para = &doc.document.sections[0].paragraphs[7];
        eprintln!("[After] field_ranges: {:?}", para.field_ranges);
        assert!(!para.field_ranges.is_empty(), "field_ranges가 비어있으면 안됨");
        let fr = &para.field_ranges[0];
        assert_eq!(fr.start_char_idx, 7, "필드 시작은 7");
        assert_eq!(fr.end_char_idx, 8, "필드 끝은 8 (1글자 삽입 후)");
    }

    /// IME 조합 사이클 시뮬레이션: delete→insert 반복 시 필드가 사라지지 않는지 검증
    #[test]
    fn test_task231_field_survives_ime_cycle() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        // Section 0, Para 7: 빈 누름틀 필드 (start=7, end=7)
        let info = doc.get_field_info_at(0, 7, 7);
        assert!(info.contains("\"inField\":true"), "초기 필드 존재 확인");

        // IME 1단계: "ㅁ" 삽입 (compositionLength=0이므로 삭제 없음)
        doc.insert_text_native(0, 7, 7, "ㅁ").expect("삽입 실패");
        let fr = &doc.document.sections[0].paragraphs[7].field_ranges[0];
        assert_eq!((fr.start_char_idx, fr.end_char_idx), (7, 8), "1단계 후 필드 범위");

        // IME 2단계: "ㅁ" 삭제 → "마" 삽입 (delete→insert cycle)
        doc.delete_text_native(0, 7, 7, 1).expect("삭제 실패");
        // *** 핵심: 삭제 후 필드가 비어도 field_ranges가 유지되어야 함 ***
        let para = &doc.document.sections[0].paragraphs[7];
        eprintln!("[After delete] field_ranges: {:?}", para.field_ranges);
        assert!(!para.field_ranges.is_empty(), "삭제 후에도 빈 필드 범위가 유지되어야 함");
        let fr = &para.field_ranges[0];
        assert_eq!((fr.start_char_idx, fr.end_char_idx), (7, 7), "삭제 후 빈 필드");

        doc.insert_text_native(0, 7, 7, "마").expect("삽입 실패");
        let fr = &doc.document.sections[0].paragraphs[7].field_ranges[0];
        assert_eq!((fr.start_char_idx, fr.end_char_idx), (7, 8), "2단계 후 필드 범위");

        // IME 3단계: "마" 삭제 → "만" 삽입
        doc.delete_text_native(0, 7, 7, 1).expect("삭제 실패");
        assert!(!doc.document.sections[0].paragraphs[7].field_ranges.is_empty(), "3단계 삭제 후 필드 유지");
        doc.insert_text_native(0, 7, 7, "만").expect("삽입 실패");
        let fr = &doc.document.sections[0].paragraphs[7].field_ranges[0];
        assert_eq!((fr.start_char_idx, fr.end_char_idx), (7, 8), "3단계 후 필드 범위");

        // IME 완료 후 필드 정보 확인
        let info = doc.get_field_info_at(0, 7, 8);
        assert!(info.contains("\"inField\":true"), "IME 완료 후 필드 내 커서 확인");
    }

    /// getClickHereProps가 유효한 JSON을 반환하는지 검증
    #[test]
    fn test_task231_get_click_here_props() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        let result = doc.get_click_here_props(1584999796);
        eprintln!("[getClickHereProps] {}", result);
        // 유효한 JSON인지 확인
        assert!(result.contains("\"ok\":true"), "ok=true 이어야 함");
        assert!(result.contains("\"guide\":\""), "guide 필드가 따옴표로 감싸져야 함");
        assert!(result.contains("여기에 입력"), "안내문이 포함되어야 함");
        // JSON 구조 검증 (따옴표 포함)
        assert!(result.starts_with("{\"ok\":true,"), "JSON 시작 구조");
    }

    /// updateClickHereProps 후 field_name() 매핑이 동작하는지 검증
    #[test]
    fn test_task231_update_click_here_props_name_mapping() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");
        let field_id = 1584999796u32;

        // 초기 상태: command에 Name 키 없음, CTRL_DATA에서 "회사명" 로드
        let para = &doc.document.sections[0].paragraphs[7];
        if let crate::model::control::Control::Field(f) = &para.controls[0] {
            assert_eq!(f.field_name(), Some("회사명"), "초기: CTRL_DATA 필드 이름");
            assert_eq!(f.ctrl_data_name.as_deref(), Some("회사명"), "초기: ctrl_data_name");
            assert_eq!(f.extract_wstring_value("Name:"), None, "초기: command에 Name 키 없음");
        }

        // 필드 이름을 "목차1"로 설정
        let result = doc.update_click_here_props(field_id, "여기에 입력", "", "목차1", true);
        assert!(result.contains("\"ok\":true"), "업데이트 성공");

        // 업데이트 후: 이름은 ctrl_data_name에만, command에는 Name: 없음
        let para = &doc.document.sections[0].paragraphs[7];
        if let crate::model::control::Control::Field(f) = &para.controls[0] {
            eprintln!("[After update] command: {:?}", f.command);
            assert_eq!(f.field_name(), Some("목차1"), "업데이트 후: ctrl_data_name 우선");
            assert_eq!(f.ctrl_data_name.as_deref(), Some("목차1"), "ctrl_data_name 설정됨");
            assert_eq!(f.extract_wstring_value("Name:"), None, "command에 Name: 없음 (한컴 호환)");
            assert_eq!(f.guide_text(), Some("여기에 입력"), "안내문 유지됨");
        }

        // getFieldValueByName으로 새 이름 조회 가능
        let val = doc.get_field_value_by_name("목차1");
        eprintln!("[ByName] 목차1: {:?}", val);
        assert!(val.is_ok(), "새 이름으로 조회 가능");

        // getClickHereProps에서 name이 비어있지 않은지 확인
        let props = doc.get_click_here_props(field_id);
        eprintln!("[Props after] {}", props);
        assert!(props.contains("\"name\":\"목차1\""), "props에 새 이름 표시");
    }

    /// [진단용] HWP 파일의 모든 ClickHere 필드 command + CTRL_DATA 덤프
    #[test]
    fn diag_dump_all_clickhere_commands() {
        // field-01.hwp와 saved/field-02.hwp 비교
        for path in &["samples/field-01.hwp", "saved/field-02.hwp"] {
            eprintln!("\n=== {} ===", path);
            let Ok(data) = std::fs::read(path) else {
                eprintln!("  (파일 없음)");
                continue;
            };
            let Ok(doc) = HwpDocument::from_bytes(&data) else {
                eprintln!("  (파싱 실패)");
                continue;
            };
            dump_clickhere_fields(&doc);
        }
    }

    fn dump_clickhere_fields(doc: &HwpDocument) {
        use crate::model::control::{Control, FieldType};
        for (si, sec) in doc.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Field(f) = ctrl {
                        if f.field_type == FieldType::ClickHere {
                            eprintln!("[sec={} para={} ctrl={}] id={} ctrl_data_name={:?}",
                                si, pi, ci, f.field_id, f.ctrl_data_name);
                            eprintln!("  command={:?}", f.command);
                            // CTRL_DATA 확인
                            if let Some(Some(cd)) = para.ctrl_data_records.get(ci) {
                                eprintln!("  CTRL_DATA({} bytes): {:02x?}", cd.len(), &cd[..cd.len().min(24)]);
                            } else {
                                eprintln!("  CTRL_DATA: None");
                            }
                        }
                    }
                }
            }
        }
    }

    /// 필드 직렬화 라운드트립: 저장 후 다시 읽으면 필드가 보존되는지 검증
    #[test]
    fn test_task231_field_roundtrip() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");

        // 저장 (직렬화 → CFB 바이트)
        let saved = doc.core.export_hwp_native().expect("저장 실패");

        // 다시 읽기
        let doc2 = HwpDocument::from_bytes(&saved).expect("다시 읽기 실패");

        use crate::model::control::{Control, FieldType};
        // sec=0 para=7의 필드 확인
        let para = &doc2.document.sections[0].paragraphs[7];
        let ctrl = &para.controls[0];
        if let Control::Field(f) = ctrl {
            assert_eq!(f.field_type, FieldType::ClickHere);
            assert_eq!(f.field_id, 1584999796);
            assert!(f.command.contains("Direction:wstring:6:여기에 입력"), "command 보존: {:?}", f.command);
            assert_eq!(f.ctrl_data_name.as_deref(), Some("회사명"), "CTRL_DATA 필드 이름 보존");
            eprintln!("[roundtrip] id={} command={:?} ctrl_data_name={:?}", f.field_id, f.command, f.ctrl_data_name);
        } else {
            panic!("sec=0 para=7 ctrl=0이 Field가 아님: {:?}", ctrl);
        }
        // field_ranges 보존 확인
        let orig_para = &doc.document.sections[0].paragraphs[7];
        eprintln!("[roundtrip] orig field_ranges={:?}", orig_para.field_ranges);
        eprintln!("[roundtrip] reload field_ranges={:?}", para.field_ranges);
        assert_eq!(para.field_ranges.len(), orig_para.field_ranges.len(), "field_ranges 개수 보존");
        for (i, (a, b)) in orig_para.field_ranges.iter().zip(para.field_ranges.iter()).enumerate() {
            assert_eq!(a.start_char_idx, b.start_char_idx, "field_range[{}].start 보존", i);
            assert_eq!(a.end_char_idx, b.end_char_idx, "field_range[{}].end 보존", i);
            assert_eq!(a.control_idx, b.control_idx, "field_range[{}].ctrl_idx 보존", i);
        }
    }

    /// [진단] 직렬화된 PARA_TEXT에서 FIELD_BEGIN/END 순서 확인
    #[test]
    fn diag_para_text_field_markers() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let doc = HwpDocument::from_bytes(&data).expect("파싱");
        let sec = &doc.document.sections[0];

        for (pi, para) in sec.paragraphs.iter().enumerate() {
            let has_field = para.controls.iter().any(|c|
                matches!(c, crate::model::control::Control::Field(_)));
            if !has_field { continue; }

            let serialized = crate::serializer::body_text::test_serialize_para_text(para);

            eprintln!("\n[para={}] text={:?}", pi, para.text);
            eprintln!("  field_ranges={:?}", para.field_ranges);
            eprintln!("  char_offsets={:?}", para.char_offsets);

            // 직렬화된 바이트에서 컨트롤 문자 위치 추출
            let code_units: Vec<u16> = serialized.chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();

            eprintln!("  serialized code_units({}):", code_units.len());
            let mut pos = 0;
            while pos < code_units.len() {
                let cu = code_units[pos];
                if cu == 0x0003 {
                    let ctrl_id = if pos + 4 < code_units.len() {
                        (code_units[pos+1] as u32) | ((code_units[pos+2] as u32) << 16)
                    } else { 0 };
                    eprintln!("    [{}] FIELD_BEGIN ctrl_id=0x{:08x}", pos, ctrl_id);
                    pos += 8;
                } else if cu == 0x0004 {
                    let ctrl_id = if pos + 4 < code_units.len() {
                        (code_units[pos+1] as u32) | ((code_units[pos+2] as u32) << 16)
                    } else { 0 };
                    eprintln!("    [{}] FIELD_END ctrl_id=0x{:08x}", pos, ctrl_id);
                    pos += 8;
                } else if cu == 0x000D {
                    eprintln!("    [{}] PARA_END", pos);
                    pos += 1;
                } else if cu == 0x000A {
                    eprintln!("    [{}] NEWLINE", pos);
                    pos += 1;
                } else if cu == 0x0009 {
                    eprintln!("    [{}] TAB", pos);
                    pos += 8;
                } else if cu < 0x0020 {
                    eprintln!("    [{}] CTRL 0x{:04x}", pos, cu);
                    if cu >= 0x0008 { pos += 8; } else { pos += 1; }
                } else {
                    // 일반 문자: 연속 출력
                    let start = pos;
                    while pos < code_units.len() && code_units[pos] >= 0x0020 {
                        pos += 1;
                    }
                    let text = String::from_utf16_lossy(&code_units[start..pos]);
                    eprintln!("    [{}..{}] TEXT {:?}", start, pos, text);
                }
            }
        }
    }

    /// 필드 이름만 변경 후 저장 → 안내문이 보존되는지 검증
    #[test]
    fn test_task231_field_name_change_preserves_guide() {
        let data = std::fs::read("samples/field-01.hwp").expect("파일 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&data).expect("HwpDocument 생성 실패");
        let field_id = 1584999796u32; // "이메일" 필드가 아닌 "회사명" 필드

        // 변경 전 상태
        let props_before = doc.get_click_here_props(field_id);
        eprintln!("[before] {}", props_before);

        // 필드 이름만 변경 (안내문, 메모는 그대로)
        let result = doc.update_click_here_props(field_id, "여기에 입력", "", "회사명1", true);
        eprintln!("[update] {}", result);

        // 변경 후 command 확인
        {
            use crate::model::control::{Control, FieldType};
            let para = &doc.document.sections[0].paragraphs[7];
            if let Control::Field(f) = &para.controls[0] {
                eprintln!("[after update] command={:?}", f.command);
                eprintln!("[after update] ctrl_data_name={:?}", f.ctrl_data_name);
            }
        }

        // 저장
        let saved = doc.core.export_hwp_native().expect("저장 실패");

        // 다시 읽기
        let doc2 = HwpDocument::from_bytes(&saved).expect("다시 읽기 실패");

        use crate::model::control::{Control, FieldType};
        let para = &doc2.document.sections[0].paragraphs[7];
        if let Control::Field(f) = &para.controls[0] {
            eprintln!("[reloaded] command={:?}", f.command);
            eprintln!("[reloaded] ctrl_data_name={:?}", f.ctrl_data_name);
            eprintln!("[reloaded] guide_text={:?}", f.guide_text());
            eprintln!("[reloaded] field_name={:?}", f.field_name());
            assert_eq!(f.field_id, field_id, "field_id 보존");
            assert_eq!(f.guide_text(), Some("여기에 입력"), "안내문 보존");
            assert_eq!(f.ctrl_data_name.as_deref(), Some("회사명1"), "변경된 필드 이름");
        } else {
            panic!("필드가 아님");
        }

        // getClickHereProps로도 확인
        let props_after = doc2.get_click_here_props(field_id);
        eprintln!("[reloaded props] {}", props_after);
        assert!(props_after.contains("\"guide\":\"여기에 입력\""), "안내문 보존");
        assert!(props_after.contains("\"name\":\"회사명1\""), "변경된 이름");
    }

    /// [진단] field-06.hwp (우리가 저장) vs field-01.hwp (원본) vs field-01-h.hwp (한컴 저장 참조)
    /// 누름틀 필드의 CTRL_HEADER / CTRL_DATA 비교
    #[test]
    fn diag_field06_vs_reference() {
        use crate::model::control::{Control, FieldType};
        use crate::parser::record::Record;
        use crate::parser::tags;

        let files: &[(&str, &str)] = &[
            ("samples/field-01.hwp", "ORIGINAL"),
            ("saved/field-01-h.hwp", "HANCOM_REF"),
            ("saved/field-06.hwp", "OUR_SAVED"),
        ];

        for (path, label) in files {
            eprintln!("\n{}", "=".repeat(60));
            eprintln!("=== {} ({}) ===", label, path);
            eprintln!("{}", "=".repeat(60));

            let Ok(data) = std::fs::read(path) else {
                eprintln!("  (파일 없음 - 건너뜀)");
                continue;
            };
            let Ok(doc) = HwpDocument::from_bytes(&data) else {
                eprintln!("  (파싱 실패)");
                continue;
            };

            // 1) 파싱된 모델에서 ClickHere 필드 정보 출력
            for (si, sec) in doc.document.sections.iter().enumerate() {
                for (pi, para) in sec.paragraphs.iter().enumerate() {
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Field(f) = ctrl {
                            if f.field_type != FieldType::ClickHere {
                                continue;
                            }
                            eprintln!("\n  [sec={} para={} ctrl={}]", si, pi, ci);
                            eprintln!("    field_type: {:?}", f.field_type);
                            eprintln!("    ctrl_id: 0x{:08x} ({})", f.ctrl_id,
                                String::from_utf8_lossy(&f.ctrl_id.to_le_bytes()));
                            eprintln!("    field_id: {} (0x{:08x})", f.field_id, f.field_id);
                            eprintln!("    properties: 0x{:08x} ({})", f.properties, f.properties);
                            eprintln!("    extra_properties: 0x{:02x} ({})", f.extra_properties, f.extra_properties);
                            eprintln!("    command({} chars): {:?}", f.command.len(), f.command);
                            eprintln!("    ctrl_data_name: {:?}", f.ctrl_data_name);
                            eprintln!("    guide_text: {:?}", f.guide_text());
                            eprintln!("    field_name: {:?}", f.field_name());
                            eprintln!("    memo_text: {:?}", f.memo_text());

                            // command를 UTF-16LE 바이트로 덤프
                            let cmd_utf16: Vec<u16> = f.command.encode_utf16().collect();
                            eprintln!("    command UTF-16 len: {}", cmd_utf16.len());

                            // CTRL_DATA 원본 바이트 덤프
                            if let Some(Some(cd)) = para.ctrl_data_records.get(ci) {
                                eprintln!("    CTRL_DATA({} bytes): {:02x?}", cd.len(), cd);
                                // 필드 이름 파싱 상세
                                if cd.len() >= 12 {
                                    let name_len = u16::from_le_bytes([cd[10], cd[11]]) as usize;
                                    eprintln!("    CTRL_DATA header(0..10): {:02x?}", &cd[..10]);
                                    eprintln!("    CTRL_DATA name_len: {}", name_len);
                                    if name_len > 0 && cd.len() >= 12 + name_len * 2 {
                                        let wchars: Vec<u16> = cd[12..12 + name_len * 2]
                                            .chunks_exact(2)
                                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                            .collect();
                                        let name = String::from_utf16_lossy(&wchars);
                                        eprintln!("    CTRL_DATA name: {:?}", name);
                                    }
                                    // 이름 이후 남은 바이트
                                    let after_name = 12 + name_len * 2;
                                    if cd.len() > after_name {
                                        eprintln!("    CTRL_DATA after_name({} bytes): {:02x?}",
                                            cd.len() - after_name, &cd[after_name..]);
                                    }
                                }
                            } else {
                                eprintln!("    CTRL_DATA: None");
                            }

                            // 직렬화 결과: 우리가 쓰는 CTRL_HEADER 데이터 생성
                            let ser_cmd_utf16: Vec<u16> = f.command.encode_utf16().collect();
                            let ser_cmd_len = ser_cmd_utf16.len();
                            let mut ser_data = Vec::new();
                            ser_data.extend_from_slice(&f.ctrl_id.to_le_bytes());
                            ser_data.extend_from_slice(&f.properties.to_le_bytes());
                            ser_data.push(f.extra_properties);
                            ser_data.extend_from_slice(&(ser_cmd_len as u16).to_le_bytes());
                            for ch in &ser_cmd_utf16 {
                                ser_data.extend_from_slice(&ch.to_le_bytes());
                            }
                            ser_data.extend_from_slice(&f.field_id.to_le_bytes());
                            eprintln!("    SERIALIZED CTRL_HEADER({} bytes): {:02x?}",
                                ser_data.len(), &ser_data[..ser_data.len().min(80)]);
                            if ser_data.len() > 80 {
                                eprintln!("    ... (truncated, {} more bytes)", ser_data.len() - 80);
                            }
                        }
                    }
                }
            }

            // 2) 원본 바이너리에서 직접 레코드 읽어 CTRL_HEADER 비교
            eprintln!("\n  --- Raw records from BodyText/Section0 ---");
            let mut cfb = match crate::parser::cfb_reader::CfbReader::open(&data) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  CFB open error: {:?}", e);
                    continue;
                }
            };

            let section_data = match cfb.read_body_text_section(0, true, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("  Section read error: {:?}", e);
                    continue;
                }
            };

            let records = match Record::read_all(&section_data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("  Record parse error: {:?}", e);
                    continue;
                }
            };

            // CTRL_HEADER 중 필드인 것만 찾기
            for (ri, rec) in records.iter().enumerate() {
                if rec.tag_id != tags::HWPTAG_CTRL_HEADER || rec.data.len() < 4 {
                    continue;
                }
                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                if !tags::is_field_ctrl_id(ctrl_id) {
                    continue;
                }
                let ctrl_id_bytes = ctrl_id.to_le_bytes();
                let ctrl_id_str = String::from_utf8_lossy(&ctrl_id_bytes);
                eprintln!("\n  [raw rec={}] CTRL_HEADER ctrl_id=0x{:08x}({}) level={} size={}",
                    ri, ctrl_id, ctrl_id_str, rec.level, rec.size);
                eprintln!("    raw data({} bytes): {:02x?}", rec.data.len(),
                    &rec.data[..rec.data.len().min(120)]);
                if rec.data.len() > 120 {
                    eprintln!("    ... (truncated, {} more bytes)", rec.data.len() - 120);
                }

                // 바로 다음 레코드가 CTRL_DATA인지 확인
                if ri + 1 < records.len() && records[ri + 1].tag_id == tags::HWPTAG_CTRL_DATA {
                    let cd = &records[ri + 1];
                    eprintln!("    CTRL_DATA[rec={}]({} bytes): {:02x?}",
                        ri + 1, cd.data.len(), &cd.data[..cd.data.len().min(80)]);
                }
            }
        }

        eprintln!("\n\n=== COMPARISON SUMMARY ===");
        eprintln!("Check above for differences in:");
        eprintln!("  - CTRL_HEADER raw data sizes and content");
        eprintln!("  - CTRL_DATA presence and content");
        eprintln!("  - command string differences");
        eprintln!("  - field_id / properties / extra_properties differences");
    }

    #[test]
    fn diag_field07_vs_field03h() {
        use crate::model::control::{Control, FieldType};
        use crate::parser::record::Record;
        use crate::parser::tags;

        let files: &[(&str, &str)] = &[
            ("samples/field-01.hwp", "ORIGINAL"),
            ("saved/field-03-h.hwp", "HANCOM_SAVED"),
            ("saved/field-07.hwp", "OUR_SAVED"),
        ];

        // ============================================================
        // PHASE 1: High-level model comparison
        // ============================================================
        for (path, label) in files {
            eprintln!("\n{}", "=".repeat(70));
            eprintln!("=== PHASE 1: MODEL — {} ({}) ===", label, path);
            eprintln!("{}", "=".repeat(70));

            let Ok(data) = std::fs::read(path) else {
                eprintln!("  (파일 없음 - 건너뜀)");
                continue;
            };
            let Ok(doc) = HwpDocument::from_bytes(&data) else {
                eprintln!("  (파싱 실패)");
                continue;
            };

            // Section-level summary
            for (si, sec) in doc.document.sections.iter().enumerate() {
                eprintln!("\n  Section {}: {} paragraphs", si, sec.paragraphs.len());
                eprintln!("    section_def page_def: {}x{}", sec.section_def.page_def.width, sec.section_def.page_def.height);

                for (pi, para) in sec.paragraphs.iter().enumerate() {
                    // Find paragraphs with ClickHere fields (E-Mail)
                    let has_email_field = para.controls.iter().any(|c| {
                        if let Control::Field(f) = c {
                            f.field_type == FieldType::ClickHere &&
                            (f.command.contains("메일") || f.command.contains("mail") || f.command.contains("Mail") ||
                             f.field_name().map_or(false, |n| n.contains("메일") || n.contains("mail") || n.contains("Mail")) ||
                             f.ctrl_data_name.as_ref().map_or(false, |n| n.contains("메일") || n.contains("mail") || n.contains("Mail")))
                        } else {
                            false
                        }
                    });

                    if !has_email_field {
                        // Brief summary for non-email paragraphs
                        eprintln!("    para[{}]: char_count={} control_mask=0x{:08x} text={:?} controls={}",
                            pi, para.char_count, para.control_mask,
                            if para.text.len() > 40 { format!("{}...", &para.text[..40]) } else { para.text.clone() },
                            para.controls.len());
                        continue;
                    }

                    // DETAILED dump for E-Mail paragraph
                    eprintln!("\n  *** E-MAIL PARAGRAPH [sec={} para={}] ***", si, pi);
                    eprintln!("    char_count: {} (0x{:08x})", para.char_count, para.char_count);
                    eprintln!("    char_count_msb: {}", para.char_count_msb);
                    eprintln!("    control_mask: 0x{:08x}", para.control_mask);
                    eprintln!("    para_shape_id: {}", para.para_shape_id);
                    eprintln!("    style_id: {}", para.style_id);
                    eprintln!("    column_type: {:?}", para.column_type);
                    eprintln!("    raw_break_type: 0x{:02x}", para.raw_break_type);
                    eprintln!("    raw_header_extra({} bytes): {:02x?}", para.raw_header_extra.len(), para.raw_header_extra);
                    eprintln!("    has_para_text: {}", para.has_para_text);
                    eprintln!("    text({} chars): {:?}", para.text.chars().count(), para.text);
                    eprintln!("    char_offsets({} entries): {:?}", para.char_offsets.len(), para.char_offsets);

                    // PARA_CHAR_SHAPE
                    eprintln!("    char_shapes({} entries):", para.char_shapes.len());
                    for (i, cs) in para.char_shapes.iter().enumerate() {
                        eprintln!("      [{}] pos={} shape_id={}", i, cs.start_pos, cs.char_shape_id);
                    }

                    // PARA_LINE_SEG
                    eprintln!("    line_segs({} entries):", para.line_segs.len());
                    for (i, ls) in para.line_segs.iter().enumerate() {
                        eprintln!("      [{}] start={} vpos={} height={} text_h={} baseline={} spacing={} col_start={} seg_w={} tag=0x{:08x}",
                            i, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                            ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
                    }

                    // PARA_RANGE_TAG
                    eprintln!("    range_tags({} entries):", para.range_tags.len());
                    for (i, rt) in para.range_tags.iter().enumerate() {
                        eprintln!("      [{}] start={} end={} tag=0x{:08x}", i, rt.start, rt.end, rt.tag);
                    }

                    // field_ranges
                    eprintln!("    field_ranges({} entries):", para.field_ranges.len());
                    for (i, fr) in para.field_ranges.iter().enumerate() {
                        eprintln!("      [{}] start_char={} end_char={} ctrl_idx={}", i, fr.start_char_idx, fr.end_char_idx, fr.control_idx);
                    }

                    // Controls + CTRL_DATA
                    eprintln!("    controls({} entries):", para.controls.len());
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Field(f) = ctrl {
                            eprintln!("      [{}] FIELD: type={:?} ctrl_id=0x{:08x}({}) field_id={} props=0x{:08x} extra=0x{:02x}",
                                ci, f.field_type, f.ctrl_id,
                                String::from_utf8_lossy(&f.ctrl_id.to_le_bytes()),
                                f.field_id, f.properties, f.extra_properties);
                            eprintln!("           command({} chars): {:?}", f.command.len(), f.command);
                            eprintln!("           ctrl_data_name: {:?}", f.ctrl_data_name);
                            eprintln!("           field_name(): {:?}", f.field_name());
                            eprintln!("           guide_text(): {:?}", f.guide_text());
                        } else {
                            eprintln!("      [{}] {:?}", ci, std::mem::discriminant(ctrl));
                        }

                        // CTRL_DATA
                        if let Some(Some(cd)) = para.ctrl_data_records.get(ci) {
                            eprintln!("      CTRL_DATA[{}]({} bytes): {:02x?}", ci, cd.len(), cd);
                        } else {
                            eprintln!("      CTRL_DATA[{}]: None", ci);
                        }
                    }

                    // Serialize PARA_TEXT and dump
                    let text_data = crate::serializer::body_text::test_serialize_para_text(para);
                    eprintln!("    SERIALIZED PARA_TEXT({} bytes = {} code units):", text_data.len(), text_data.len() / 2);
                    // Hex dump in lines of 32 bytes
                    for chunk_start in (0..text_data.len()).step_by(32) {
                        let chunk_end = (chunk_start + 32).min(text_data.len());
                        let chunk = &text_data[chunk_start..chunk_end];
                        eprint!("      {:04x}: ", chunk_start);
                        for b in chunk {
                            eprint!("{:02x} ", b);
                        }
                        // Also show as u16 code units
                        eprint!(" | ");
                        for pair in chunk.chunks_exact(2) {
                            let cu = u16::from_le_bytes([pair[0], pair[1]]);
                            if cu >= 0x20 && cu < 0x7F {
                                eprint!("{} ", cu as u8 as char);
                            } else if cu >= 0xAC00 && cu <= 0xD7A3 {
                                eprint!("K ");  // Korean
                            } else {
                                eprint!("{:04x} ", cu);
                            }
                        }
                        eprintln!();
                    }
                }
            }
        }

        // ============================================================
        // PHASE 2: Raw binary record comparison
        // ============================================================
        for (path, label) in files {
            eprintln!("\n{}", "=".repeat(70));
            eprintln!("=== PHASE 2: RAW RECORDS — {} ({}) ===", label, path);
            eprintln!("{}", "=".repeat(70));

            let Ok(data) = std::fs::read(path) else {
                eprintln!("  (파일 없음 - 건너뜀)");
                continue;
            };

            let mut cfb = match crate::parser::cfb_reader::CfbReader::open(&data) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  CFB open error: {:?}", e);
                    continue;
                }
            };

            let section_data = match cfb.read_body_text_section(0, true, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("  Section read error: {:?}", e);
                    continue;
                }
            };

            let records = match Record::read_all(&section_data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("  Record parse error: {:?}", e);
                    continue;
                }
            };

            eprintln!("  Total records: {}", records.len());

            // Find the E-Mail paragraph by scanning for field CTRL_HEADER with email-related command
            // First pass: identify all PARA_HEADER positions
            let mut para_starts: Vec<usize> = Vec::new();
            for (ri, rec) in records.iter().enumerate() {
                if rec.tag_id == tags::HWPTAG_PARA_HEADER {
                    para_starts.push(ri);
                }
            }
            eprintln!("  Paragraph count (from PARA_HEADER records): {}", para_starts.len());

            // For each paragraph, check if it has a field CTRL_HEADER with email command
            for (pi, &para_start) in para_starts.iter().enumerate() {
                let para_end = if pi + 1 < para_starts.len() { para_starts[pi + 1] } else { records.len() };
                let para_records = &records[para_start..para_end];

                // Check for email-related field
                let has_email = para_records.iter().any(|rec| {
                    if rec.tag_id != tags::HWPTAG_CTRL_HEADER || rec.data.len() < 11 {
                        return false;
                    }
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    if !tags::is_field_ctrl_id(ctrl_id) {
                        return false;
                    }
                    // Check command string for email
                    if rec.data.len() >= 11 {
                        let cmd_len = u16::from_le_bytes([rec.data[9], rec.data[10]]) as usize;
                        if cmd_len > 0 && rec.data.len() >= 11 + cmd_len * 2 {
                            let wchars: Vec<u16> = rec.data[11..11 + cmd_len * 2]
                                .chunks_exact(2)
                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                .collect();
                            let cmd = String::from_utf16_lossy(&wchars);
                            return cmd.contains("메일") || cmd.contains("mail") || cmd.contains("Mail");
                        }
                    }
                    false
                });

                if !has_email {
                    continue;
                }

                eprintln!("\n  *** RAW E-MAIL PARAGRAPH [para_idx={}] (records {}..{}) ***", pi, para_start, para_end);

                for (offset, rec) in para_records.iter().enumerate() {
                    let ri = para_start + offset;
                    let tag_name = tags::tag_name(rec.tag_id);
                    eprintln!("\n    [rec={}] {} (tag=0x{:04x}) level={} size={}",
                        ri, tag_name, rec.tag_id, rec.level, rec.data.len());

                    match rec.tag_id {
                        tags::HWPTAG_PARA_HEADER => {
                            eprintln!("      raw({} bytes): {:02x?}", rec.data.len(), rec.data);
                            if rec.data.len() >= 12 {
                                let char_count_raw = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                                let control_mask = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                                let para_shape_id = u16::from_le_bytes([rec.data[8], rec.data[9]]);
                                let style_id = rec.data[10];
                                let break_type = rec.data[11];
                                eprintln!("      char_count_raw=0x{:08x} (count={}, msb={})",
                                    char_count_raw, char_count_raw & 0x7FFFFFFF, char_count_raw >> 31);
                                eprintln!("      control_mask=0x{:08x}", control_mask);
                                eprintln!("      para_shape_id={} style_id={} break_type=0x{:02x}", para_shape_id, style_id, break_type);
                            }
                            if rec.data.len() >= 18 {
                                let num_cs = u16::from_le_bytes([rec.data[12], rec.data[13]]);
                                let num_rt = u16::from_le_bytes([rec.data[14], rec.data[15]]);
                                let num_ls = u16::from_le_bytes([rec.data[16], rec.data[17]]);
                                eprintln!("      numCharShapes={} numRangeTags={} numLineSegs={}", num_cs, num_rt, num_ls);
                            }
                            if rec.data.len() >= 22 {
                                let instance_id = u32::from_le_bytes([rec.data[18], rec.data[19], rec.data[20], rec.data[21]]);
                                eprintln!("      instanceId={}", instance_id);
                            }
                            if rec.data.len() > 22 {
                                eprintln!("      extra bytes after instanceId: {:02x?}", &rec.data[22..]);
                            }
                        }
                        tags::HWPTAG_PARA_TEXT => {
                            eprintln!("      raw({} bytes = {} code units):", rec.data.len(), rec.data.len() / 2);
                            for chunk_start in (0..rec.data.len()).step_by(32) {
                                let chunk_end = (chunk_start + 32).min(rec.data.len());
                                let chunk = &rec.data[chunk_start..chunk_end];
                                eprint!("        {:04x}: ", chunk_start);
                                for b in chunk {
                                    eprint!("{:02x} ", b);
                                }
                                eprint!(" | ");
                                for pair in chunk.chunks_exact(2) {
                                    let cu = u16::from_le_bytes([pair[0], pair[1]]);
                                    if cu >= 0x20 && cu < 0x7F {
                                        eprint!("{} ", cu as u8 as char);
                                    } else if cu >= 0xAC00 && cu <= 0xD7A3 {
                                        eprint!("K ");
                                    } else {
                                        eprint!("{:04x} ", cu);
                                    }
                                }
                                eprintln!();
                            }
                        }
                        tags::HWPTAG_PARA_CHAR_SHAPE => {
                            eprintln!("      raw({} bytes, {} entries):", rec.data.len(), rec.data.len() / 8);
                            for i in (0..rec.data.len()).step_by(8) {
                                if i + 8 <= rec.data.len() {
                                    let pos = u32::from_le_bytes([rec.data[i], rec.data[i+1], rec.data[i+2], rec.data[i+3]]);
                                    let id = u32::from_le_bytes([rec.data[i+4], rec.data[i+5], rec.data[i+6], rec.data[i+7]]);
                                    eprintln!("        pos={} shape_id={}", pos, id);
                                }
                            }
                        }
                        tags::HWPTAG_PARA_LINE_SEG => {
                            eprintln!("      raw({} bytes, {} entries):", rec.data.len(), rec.data.len() / 36);
                            for i in (0..rec.data.len()).step_by(36) {
                                if i + 36 <= rec.data.len() {
                                    let d = &rec.data[i..i+36];
                                    let text_start = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
                                    let vpos = i32::from_le_bytes([d[4], d[5], d[6], d[7]]);
                                    let height = i32::from_le_bytes([d[8], d[9], d[10], d[11]]);
                                    let text_h = i32::from_le_bytes([d[12], d[13], d[14], d[15]]);
                                    let baseline = i32::from_le_bytes([d[16], d[17], d[18], d[19]]);
                                    let spacing = i32::from_le_bytes([d[20], d[21], d[22], d[23]]);
                                    let col_start = i32::from_le_bytes([d[24], d[25], d[26], d[27]]);
                                    let seg_w = i32::from_le_bytes([d[28], d[29], d[30], d[31]]);
                                    let tag_val = u32::from_le_bytes([d[32], d[33], d[34], d[35]]);
                                    eprintln!("        start={} vpos={} h={} th={} bl={} sp={} cs={} sw={} tag=0x{:08x}",
                                        text_start, vpos, height, text_h, baseline, spacing, col_start, seg_w, tag_val);
                                }
                            }
                        }
                        tags::HWPTAG_PARA_RANGE_TAG => {
                            eprintln!("      raw({} bytes, {} entries):", rec.data.len(), rec.data.len() / 12);
                            for i in (0..rec.data.len()).step_by(12) {
                                if i + 12 <= rec.data.len() {
                                    let start = u32::from_le_bytes([rec.data[i], rec.data[i+1], rec.data[i+2], rec.data[i+3]]);
                                    let end = u32::from_le_bytes([rec.data[i+4], rec.data[i+5], rec.data[i+6], rec.data[i+7]]);
                                    let tag_val = u32::from_le_bytes([rec.data[i+8], rec.data[i+9], rec.data[i+10], rec.data[i+11]]);
                                    eprintln!("        start={} end={} tag=0x{:08x}", start, end, tag_val);
                                }
                            }
                        }
                        tags::HWPTAG_CTRL_HEADER => {
                            if rec.data.len() >= 4 {
                                let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                                let ctrl_id_bytes = ctrl_id.to_le_bytes();
                                let ctrl_id_str = String::from_utf8_lossy(&ctrl_id_bytes);
                                eprintln!("      ctrl_id=0x{:08x}({})", ctrl_id, ctrl_id_str);

                                if tags::is_field_ctrl_id(ctrl_id) {
                                    eprintln!("      *** FIELD CTRL_HEADER DETAIL ***");
                                    eprintln!("      full raw({} bytes): {:02x?}", rec.data.len(), rec.data);
                                    if rec.data.len() >= 11 {
                                        let props = u32::from_le_bytes([rec.data[4], rec.data[5], rec.data[6], rec.data[7]]);
                                        let extra = rec.data[8];
                                        let cmd_len = u16::from_le_bytes([rec.data[9], rec.data[10]]) as usize;
                                        eprintln!("      properties=0x{:08x} extra_properties=0x{:02x} command_len={}", props, extra, cmd_len);
                                        if cmd_len > 0 && rec.data.len() >= 11 + cmd_len * 2 {
                                            let wchars: Vec<u16> = rec.data[11..11 + cmd_len * 2]
                                                .chunks_exact(2)
                                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                                .collect();
                                            let cmd = String::from_utf16_lossy(&wchars);
                                            eprintln!("      command: {:?}", cmd);
                                        }
                                        let field_id_offset = 11 + cmd_len * 2;
                                        if rec.data.len() >= field_id_offset + 4 {
                                            let field_id = u32::from_le_bytes([
                                                rec.data[field_id_offset], rec.data[field_id_offset+1],
                                                rec.data[field_id_offset+2], rec.data[field_id_offset+3]]);
                                            eprintln!("      field_id={} (0x{:08x})", field_id, field_id);
                                        }
                                        // Any bytes after field_id?
                                        let expected_end = field_id_offset + 4;
                                        if rec.data.len() > expected_end {
                                            eprintln!("      *** EXTRA BYTES AFTER field_id ({} bytes): {:02x?}",
                                                rec.data.len() - expected_end, &rec.data[expected_end..]);
                                        }
                                    }
                                } else {
                                    eprintln!("      raw({} bytes): {:02x?}", rec.data.len(),
                                        &rec.data[..rec.data.len().min(80)]);
                                }
                            }
                        }
                        tags::HWPTAG_CTRL_DATA => {
                            eprintln!("      raw({} bytes): {:02x?}", rec.data.len(), rec.data);
                            // Parse CTRL_DATA for field: typically has name
                            if rec.data.len() >= 12 {
                                eprintln!("      header(0..10): {:02x?}", &rec.data[..10]);
                                let name_len = u16::from_le_bytes([rec.data[10], rec.data[11]]) as usize;
                                eprintln!("      name_len={}", name_len);
                                if name_len > 0 && rec.data.len() >= 12 + name_len * 2 {
                                    let wchars: Vec<u16> = rec.data[12..12 + name_len * 2]
                                        .chunks_exact(2)
                                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                        .collect();
                                    let name = String::from_utf16_lossy(&wchars);
                                    eprintln!("      name: {:?}", name);
                                }
                            }
                        }
                        _ => {
                            eprintln!("      raw({} bytes): {:02x?}", rec.data.len(),
                                &rec.data[..rec.data.len().min(40)]);
                        }
                    }
                }
            }
        }

        // ============================================================
        // PHASE 3: TAB extended data comparison
        // ============================================================
        eprintln!("\n{}", "=".repeat(70));
        eprintln!("=== PHASE 3: TAB EXTENDED DATA COMPARISON ===");
        eprintln!("{}", "=".repeat(70));

        for (path, label) in files {
            eprintln!("\n  --- {} ({}) ---", label, path);

            let Ok(data) = std::fs::read(path) else {
                eprintln!("    (파일 없음 - 건너뜀)");
                continue;
            };

            let mut cfb = match crate::parser::cfb_reader::CfbReader::open(&data) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("    CFB open error: {:?}", e);
                    continue;
                }
            };

            let section_data = match cfb.read_body_text_section(0, true, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("    Section read error: {:?}", e);
                    continue;
                }
            };

            let records = match Record::read_all(&section_data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("    Record parse error: {:?}", e);
                    continue;
                }
            };

            // Find all PARA_TEXT records and check TAB extended data
            let mut tab_count = 0;
            let mut tab_zeroed = 0;
            let mut tab_nonzero = 0;
            for rec in &records {
                if rec.tag_id != tags::HWPTAG_PARA_TEXT { continue; }
                // Scan for TAB characters (0x0009)
                let code_units: Vec<u16> = rec.data.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                for (i, &cu) in code_units.iter().enumerate() {
                    if cu == 0x0009 && i + 7 < code_units.len() {
                        tab_count += 1;
                        let ext: Vec<u16> = code_units[i+1..i+8].to_vec();
                        let all_zero = ext.iter().all(|&x| x == 0);
                        if all_zero {
                            tab_zeroed += 1;
                        } else {
                            tab_nonzero += 1;
                            eprintln!("    TAB at cu_offset={}: extended data = {:04x?}", i, ext);
                        }
                    }
                }
            }
            eprintln!("    TABs total={} zeroed={} nonzero={}", tab_count, tab_zeroed, tab_nonzero);
        }

        // ============================================================
        // PHASE 4: Full section record count comparison
        // ============================================================
        eprintln!("\n{}", "=".repeat(70));
        eprintln!("=== PHASE 4: SECTION RECORD SUMMARY ===");
        eprintln!("{}", "=".repeat(70));

        for (path, label) in files {
            eprintln!("\n  --- {} ({}) ---", label, path);

            let Ok(data) = std::fs::read(path) else {
                eprintln!("    (파일 없음 - 건너뜀)");
                continue;
            };

            let mut cfb = match crate::parser::cfb_reader::CfbReader::open(&data) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("    CFB open error: {:?}", e);
                    continue;
                }
            };

            let section_data = match cfb.read_body_text_section(0, true, false) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("    Section read error: {:?}", e);
                    continue;
                }
            };

            let records = match Record::read_all(&section_data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("    Record parse error: {:?}", e);
                    continue;
                }
            };

            // Count by tag type
            use std::collections::BTreeMap;
            let mut tag_counts: BTreeMap<u16, usize> = BTreeMap::new();
            for rec in &records {
                *tag_counts.entry(rec.tag_id).or_insert(0) += 1;
            }
            for (tag_id, count) in &tag_counts {
                eprintln!("    {} (0x{:04x}): {} records", tags::tag_name(*tag_id), tag_id, count);
            }
            eprintln!("    Total: {} records", records.len());
        }

        // ============================================================
        // PHASE 5: Byte-level diff of serialized vs original PARA_TEXT for E-Mail paragraph
        // ============================================================
        eprintln!("\n{}", "=".repeat(70));
        eprintln!("=== PHASE 5: SERIALIZED vs ORIGINAL PARA_TEXT DIFF ===");
        eprintln!("{}", "=".repeat(70));

        // Compare our serialized output for saved/field-07.hwp against its raw PARA_TEXT
        if let Ok(data) = std::fs::read("saved/field-07.hwp") {
            if let Ok(doc) = HwpDocument::from_bytes(&data) {
                // Find the E-Mail paragraph
                for sec in &doc.document.sections {
                    for para in &sec.paragraphs {
                        let has_email = para.controls.iter().any(|c| {
                            if let Control::Field(f) = c {
                                f.field_type == FieldType::ClickHere &&
                                (f.command.contains("메일") || f.command.contains("mail") || f.command.contains("Mail") ||
                                 f.ctrl_data_name.as_ref().map_or(false, |n| n.contains("메일") || n.contains("mail") || n.contains("Mail")))
                            } else { false }
                        });
                        if !has_email { continue; }

                        // Re-serialize
                        let serialized = crate::serializer::body_text::test_serialize_para_text(para);
                        eprintln!("  Serialized PARA_TEXT: {} bytes", serialized.len());

                        // Get original raw
                        let mut cfb = crate::parser::cfb_reader::CfbReader::open(&data).unwrap();
                        let section_data = cfb.read_body_text_section(0, true, false).unwrap();
                        let records = Record::read_all(&section_data).unwrap();

                        // Find the matching PARA_TEXT
                        let mut para_starts: Vec<usize> = Vec::new();
                        for (ri, rec) in records.iter().enumerate() {
                            if rec.tag_id == tags::HWPTAG_PARA_HEADER { para_starts.push(ri); }
                        }
                        for (pi, &ps) in para_starts.iter().enumerate() {
                            let pe = if pi+1 < para_starts.len() { para_starts[pi+1] } else { records.len() };
                            let has_field = records[ps..pe].iter().any(|r| {
                                if r.tag_id != tags::HWPTAG_CTRL_HEADER || r.data.len() < 11 { return false; }
                                let cid = u32::from_le_bytes([r.data[0], r.data[1], r.data[2], r.data[3]]);
                                if !tags::is_field_ctrl_id(cid) { return false; }
                                let cl = u16::from_le_bytes([r.data[9], r.data[10]]) as usize;
                                if cl > 0 && r.data.len() >= 11 + cl*2 {
                                    let w: Vec<u16> = r.data[11..11+cl*2].chunks_exact(2)
                                        .map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                                    let cmd = String::from_utf16_lossy(&w);
                                    cmd.contains("메일") || cmd.contains("mail") || cmd.contains("Mail")
                                } else { false }
                            });
                            if !has_field { continue; }

                            // Find PARA_TEXT in this paragraph
                            for rec in &records[ps..pe] {
                                if rec.tag_id == tags::HWPTAG_PARA_TEXT {
                                    let original = &rec.data;
                                    eprintln!("  Original PARA_TEXT: {} bytes", original.len());

                                    if serialized.len() != original.len() {
                                        eprintln!("  *** SIZE MISMATCH: serialized={} vs original={} ***",
                                            serialized.len(), original.len());
                                    }

                                    // Find byte-level differences
                                    let min_len = serialized.len().min(original.len());
                                    let mut diff_count = 0;
                                    for i in 0..min_len {
                                        if serialized[i] != original[i] {
                                            if diff_count < 30 {
                                                eprintln!("    DIFF at byte {}: serialized=0x{:02x} original=0x{:02x}",
                                                    i, serialized[i], original[i]);
                                            }
                                            diff_count += 1;
                                        }
                                    }
                                    if diff_count > 30 {
                                        eprintln!("    ... and {} more differences", diff_count - 30);
                                    }
                                    eprintln!("  Total byte differences: {}", diff_count);
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        eprintln!("\n=== DIAGNOSTIC COMPLETE ===");
    }

    /// [진단] field-10.hwp (우리 저장) vs field-10-2010.hwp (한컴 2010 저장) 비교
    /// 한컴에서 page 2 ClickHere 필드의 안내문이 빈 문자열로 표시되는 원인 분석
    #[test]
    fn diag_field10_comparison() {
        use crate::model::control::{Control, FieldType};

        let files: &[(&str, &str)] = &[
            ("saved/field-10.hwp", "OUR_SAVED"),
            ("saved/field-10-2010.hwp", "HANCOM_2010"),
        ];

        for (path, label) in files {
            eprintln!("\n{}", "=".repeat(70));
            eprintln!("=== {} ({}) ===", label, path);
            eprintln!("{}", "=".repeat(70));

            let Ok(data) = std::fs::read(path) else {
                eprintln!("  (파일 없음 - 건너뜀)");
                continue;
            };
            let Ok(doc) = HwpDocument::from_bytes(&data) else {
                eprintln!("  (파싱 실패)");
                continue;
            };

            eprintln!("  Sections: {}", doc.document.sections.len());

            for (si, sec) in doc.document.sections.iter().enumerate() {
                eprintln!("\n  --- Section {} ({} paragraphs) ---", si, sec.paragraphs.len());

                // 1) 섹션 최상위 문단의 ClickHere 필드
                for (pi, para) in sec.paragraphs.iter().enumerate() {
                    diag_field10_print_clickhere_in_para(
                        &format!("sec={} para={}", si, pi),
                        para,
                    );

                    // 2) 표 셀 내부 문단
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Table(t) = ctrl {
                            for (cell_i, cell) in t.cells.iter().enumerate() {
                                for (cp, cpara) in cell.paragraphs.iter().enumerate() {
                                    diag_field10_print_clickhere_in_para(
                                        &format!("sec={} para={} table_ctrl={} cell={} cell_para={}",
                                            si, pi, ci, cell_i, cp),
                                        cpara,
                                    );
                                    // 표 셀 안의 표/글상자도 확인 (중첩)
                                    for (cci, cctrl) in cpara.controls.iter().enumerate() {
                                        diag_field10_check_nested(
                                            &format!("sec={} para={} table_ctrl={} cell={} cell_para={} nested_ctrl={}",
                                                si, pi, ci, cell_i, cp, cci),
                                            cctrl,
                                        );
                                    }
                                }
                            }
                        }
                        // 3) 글상자(Shape) 내부 문단
                        if let Control::Shape(s) = ctrl {
                            if let Some(drawing) = s.drawing() {
                                if let Some(tb) = &drawing.text_box {
                                    for (tp, tpara) in tb.paragraphs.iter().enumerate() {
                                        diag_field10_print_clickhere_in_para(
                                            &format!("sec={} para={} shape_ctrl={} textbox_para={}",
                                                si, pi, ci, tp),
                                            tpara,
                                        );
                                        // 글상자 안의 표/글상자도 확인
                                        for (tci, tctrl) in tpara.controls.iter().enumerate() {
                                            diag_field10_check_nested(
                                                &format!("sec={} para={} shape_ctrl={} textbox_para={} nested_ctrl={}",
                                                    si, pi, ci, tp, tci),
                                                tctrl,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // 4) Picture 내부 (caption 등은 별도, Picture는 보통 텍스트 없음)
                    }
                }
            }

            // Raw record 분석: CTRL_HEADER에서 필드 레코드 추출
            eprintln!("\n  --- Raw CTRL_HEADER records (all sections) ---");
            let mut cfb = match crate::parser::cfb_reader::CfbReader::open(&data) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  CFB open error: {:?}", e);
                    continue;
                }
            };
            for sec_idx in 0..doc.document.sections.len() {
                let Ok(section_data) = cfb.read_body_text_section(sec_idx as u32, true, false) else {
                    eprintln!("  Section {} read error", sec_idx);
                    continue;
                };
                let Ok(records) = crate::parser::record::Record::read_all(&section_data) else {
                    eprintln!("  Section {} record parse error", sec_idx);
                    continue;
                };

                let mut field_count = 0;
                for (ri, rec) in records.iter().enumerate() {
                    if rec.tag_id != crate::parser::tags::HWPTAG_CTRL_HEADER || rec.data.len() < 4 {
                        continue;
                    }
                    let ctrl_id = u32::from_le_bytes([rec.data[0], rec.data[1], rec.data[2], rec.data[3]]);
                    if !crate::parser::tags::is_field_ctrl_id(ctrl_id) {
                        continue;
                    }
                    let ctrl_id_bytes = ctrl_id.to_le_bytes();
                    let ctrl_id_str = String::from_utf8_lossy(&ctrl_id_bytes);
                    // ClickHere 필드인지 확인: ctrl_id == '%clk' = 0x25636c6b
                    let is_clickhere = ctrl_id == crate::parser::tags::FIELD_CLICKHERE;

                    if is_clickhere {
                        field_count += 1;
                        eprintln!("\n  [raw sec={} rec={}] ClickHere CTRL_HEADER ctrl_id=0x{:08x}({}) size={}",
                            sec_idx, ri, ctrl_id, ctrl_id_str, rec.data.len());
                        // 처음 120바이트 출력
                        let dump_len = rec.data.len().min(200);
                        eprintln!("    raw({} bytes): {:02x?}", rec.data.len(), &rec.data[..dump_len]);
                        if rec.data.len() > dump_len {
                            eprintln!("    ... ({} more bytes)", rec.data.len() - dump_len);
                        }

                        // command 문자열 추출 시도: offset 9에 command_len(u16), 이후 UTF-16LE
                        if rec.data.len() >= 11 {
                            let cmd_len = u16::from_le_bytes([rec.data[9], rec.data[10]]) as usize;
                            let cmd_byte_start = 11;
                            let cmd_byte_end = cmd_byte_start + cmd_len * 2;
                            if rec.data.len() >= cmd_byte_end {
                                let wchars: Vec<u16> = rec.data[cmd_byte_start..cmd_byte_end]
                                    .chunks_exact(2)
                                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                    .collect();
                                let cmd = String::from_utf16_lossy(&wchars);
                                eprintln!("    command({} chars): {:?}", cmd.len(), cmd);

                                // set:N 값 추출
                                if let Some(set_start) = cmd.find("set:") {
                                    let rest = &cmd[set_start + 4..];
                                    if let Some(colon) = rest.find(':') {
                                        let n_str = &rest[..colon];
                                        eprintln!("    set:N value: {:?}", n_str);
                                    }
                                }
                            }
                        }

                        // CTRL_DATA 확인
                        if ri + 1 < records.len() && records[ri + 1].tag_id == crate::parser::tags::HWPTAG_CTRL_DATA {
                            let cd = &records[ri + 1];
                            eprintln!("    CTRL_DATA[rec={}]({} bytes): {:02x?}",
                                ri + 1, cd.data.len(), &cd.data[..cd.data.len().min(80)]);
                        }
                    }
                }
                if field_count == 0 {
                    eprintln!("  [sec={}] No ClickHere fields in raw records", sec_idx);
                }
            }
        }

        eprintln!("\n\n{}", "=".repeat(70));
        eprintln!("=== COMPARISON SUMMARY ===");
        eprintln!("{}", "=".repeat(70));
        eprintln!("Compare the command strings, set:N values, guide_text, memo_text");
        eprintln!("between OUR_SAVED and HANCOM_2010 for page 2 fields.");
        eprintln!("Look for: trailing spaces, empty guide text, different set:N counts");
    }

    fn diag_field10_print_clickhere_in_para(
        location: &str,
        para: &crate::model::paragraph::Paragraph,
    ) {
        use crate::model::control::{Control, FieldType};

        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Field(f) = ctrl {
                if f.field_type != FieldType::ClickHere {
                    continue;
                }
                eprintln!("\n  [{}  ctrl={}] ClickHere", location, ci);
                eprintln!("    field_id: {} (0x{:08x})", f.field_id, f.field_id);
                eprintln!("    ctrl_id: 0x{:08x} ({})", f.ctrl_id,
                    String::from_utf8_lossy(&f.ctrl_id.to_le_bytes()));
                eprintln!("    properties: 0x{:08x} ({})", f.properties, f.properties);
                eprintln!("    extra_properties: 0x{:02x} ({})", f.extra_properties, f.extra_properties);
                eprintln!("    command_len(bytes): {}", f.command.len());
                eprintln!("    command_len(chars): {}", f.command.chars().count());
                eprintln!("    command: {:?}", f.command);

                // command의 각 바이트를 escape하여 trailing space 등 확인
                let escaped: String = f.command.chars().map(|c| {
                    if c == ' ' { "·".to_string() }
                    else if c == '\t' { "\\t".to_string() }
                    else if c == '\n' { "\\n".to_string() }
                    else if c == '\r' { "\\r".to_string() }
                    else { c.to_string() }
                }).collect();
                eprintln!("    command(escaped): {}", escaped);

                // set:N 값 추출
                if let Some(set_start) = f.command.find("set:") {
                    let rest = &f.command[set_start + 4..];
                    if let Some(colon) = rest.find(':') {
                        let n_str = &rest[..colon];
                        eprintln!("    set:N value: {:?} (parsed: {:?})", n_str, n_str.parse::<usize>().ok());
                    }
                }

                // guide/memo 추출 결과
                eprintln!("    guide_text(): {:?}", f.guide_text());
                eprintln!("    memo_text(): {:?}", f.memo_text());
                eprintln!("    field_name(): {:?}", f.field_name());
                eprintln!("    ctrl_data_name: {:?}", f.ctrl_data_name);

                // extract_wstring_value 상세 (Direction/HelpState/Name)
                for key in &["Direction:", "HelpState:", "Name:"] {
                    let val = f.extract_wstring_value(key);
                    eprintln!("    extract_wstring_value({:?}): {:?}", key, val);
                }

                // CTRL_DATA 원본 바이트
                if let Some(Some(cd)) = para.ctrl_data_records.get(ci) {
                    eprintln!("    CTRL_DATA({} bytes): {:02x?}", cd.len(), &cd[..cd.len().min(80)]);
                    if cd.len() >= 12 {
                        let name_len = u16::from_le_bytes([cd[10], cd[11]]) as usize;
                        eprintln!("    CTRL_DATA name_len: {}", name_len);
                        if name_len > 0 && cd.len() >= 12 + name_len * 2 {
                            let wchars: Vec<u16> = cd[12..12 + name_len * 2]
                                .chunks_exact(2)
                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                .collect();
                            let name = String::from_utf16_lossy(&wchars);
                            eprintln!("    CTRL_DATA name: {:?}", name);
                        }
                    }
                } else {
                    eprintln!("    CTRL_DATA: None");
                }

                // UTF-16 command 길이 (직렬화 시 사용)
                let cmd_utf16: Vec<u16> = f.command.encode_utf16().collect();
                eprintln!("    command UTF-16 len: {}", cmd_utf16.len());
            }
        }
    }

    fn diag_field10_check_nested(
        location: &str,
        ctrl: &crate::model::control::Control,
    ) {
        use crate::model::control::Control;
        match ctrl {
            Control::Table(t) => {
                for (cell_i, cell) in t.cells.iter().enumerate() {
                    for (cp, cpara) in cell.paragraphs.iter().enumerate() {
                        diag_field10_print_clickhere_in_para(
                            &format!("{} nested_table cell={} para={}", location, cell_i, cp),
                            cpara,
                        );
                    }
                }
            }
            Control::Shape(s) => {
                if let Some(drawing) = s.drawing() {
                    if let Some(tb) = &drawing.text_box {
                        for (tp, tpara) in tb.paragraphs.iter().enumerate() {
                            diag_field10_print_clickhere_in_para(
                                &format!("{} nested_shape textbox_para={}", location, tp),
                                tpara,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }


    #[test]
    fn diag_raw_tail_dump() {
        for path in &[
            "samples/field-01.hwp",
            "samples/field-01-memo.hwp",
            "saved/field-01-h.hwp",
            "saved/field-10.hwp",
            "saved/field-10-2010.hwp",
        ] {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => { eprintln!("[SKIP] {}", path); continue; }
            };
            let doc = HwpDocument::from_bytes(&data).expect("파싱");
            eprintln!("\n=== {} ===", path);
            for (si, sec) in doc.document.sections.iter().enumerate() {
                fn check_para(si: usize, loc: &str, para: &crate::model::paragraph::Paragraph) {
                    for ctrl in &para.controls {
                        if let crate::model::control::Control::Field(f) = ctrl {
                            eprintln!("  [sec={} {}] field_type={:?} ctrl_id=0x{:08x} field_id=0x{:08x} memo_index={:02x?}",
                                si, loc, f.field_type, f.ctrl_id, f.field_id,
                                f.memo_index);
                        }
                    }
                }
                for (pi, para) in sec.paragraphs.iter().enumerate() {
                    check_para(si, &format!("para={}", pi), para);
                    for ctrl in &para.controls {
                        match ctrl {
                            crate::model::control::Control::Table(t) => {
                                for (ci, cell) in t.cells.iter().enumerate() {
                                    for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                        check_para(si, &format!("tbl cell={} para={}", ci, cpi), cp);
                                    }
                                }
                            }
                            crate::model::control::Control::Shape(s) => {
                                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                        check_para(si, &format!("shape para={}", tpi), tp);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn diag_memo_controls() {
        for path in &["samples/field-01.hwp", "samples/field-01-memo.hwp"] {
            let data = std::fs::read(path).expect("read");
            let doc = HwpDocument::from_bytes(&data).expect("parse");
            eprintln!("\n=== {} ===", path);
            for (si, sec) in doc.document.sections.iter().enumerate() {
                for (pi, para) in sec.paragraphs.iter().enumerate() {
                    if para.controls.is_empty() { continue; }
                    eprint!("  [sec={} para={}] controls:", si, pi);
                    for ctrl in &para.controls {
                        let name = match ctrl {
                            crate::model::control::Control::SectionDef(_) => "SectionDef",
                            crate::model::control::Control::ColumnDef(_) => "ColumnDef",
                            crate::model::control::Control::Table(_) => "Table",
                            crate::model::control::Control::Shape(_) => "Shape",
                            crate::model::control::Control::Picture(_) => "Picture",
                            crate::model::control::Control::Header(_) => "Header",
                            crate::model::control::Control::Footer(_) => "Footer",
                            crate::model::control::Control::Footnote(_) => "Footnote",
                            crate::model::control::Control::Endnote(_) => "Endnote",
                            crate::model::control::Control::AutoNumber(_) => "AutoNumber",
                            crate::model::control::Control::NewNumber(_) => "NewNumber",
                            crate::model::control::Control::PageNumberPos(_) => "PageNumPos",
                            crate::model::control::Control::PageHide(_) => "PageHide",
                            crate::model::control::Control::Bookmark(_) => "Bookmark",
                            crate::model::control::Control::Hyperlink(_) => "Hyperlink",
                            crate::model::control::Control::Ruby(_) => "Ruby",
                            crate::model::control::Control::CharOverlap(_) => "CharOverlap",
                            crate::model::control::Control::HiddenComment(_) => "HiddenComment",
                            crate::model::control::Control::Field(f) => {
                                eprint!(" Field({:?},id=0x{:08x},props=0x{:08x},extra=0x{:02x},memo={},guide={:?},memo_text={:?})",
                                    f.field_type, f.field_id, f.properties, f.extra_properties, f.memo_index,
                                    f.guide_text(), f.memo_text());
                                continue;
                            }
                            crate::model::control::Control::Equation(_) => "Equation",
                            crate::model::control::Control::Form(_) => "Form",
                            crate::model::control::Control::Unknown(u) => {
                                eprint!(" Unknown(0x{:08x})", u.ctrl_id);
                                continue;
                            }
                        };
                        eprint!(" {}", name);
                    }
                    eprintln!();
                    // field_ranges 정보
                    let chars: Vec<char> = para.text.chars().collect();
                    for (fri, fr) in para.field_ranges.iter().enumerate() {
                        let field_text: String = if fr.start_char_idx < fr.end_char_idx && fr.end_char_idx <= chars.len() {
                            chars[fr.start_char_idx..fr.end_char_idx].iter().collect()
                        } else { String::new() };
                        eprintln!("    field_range[{}]: ctrl_idx={} start={} end={} text={:?}", fri, fr.control_idx, fr.start_char_idx, fr.end_char_idx, field_text);
                    }
                    eprintln!("    para.text({} chars): {:?}", chars.len(), &para.text[..para.text.len().min(80)]);
                }
            }
        }
    }

    /// 13페이지 엔터 후 페이지 전파 범위 분석
    #[test]
    fn test_page13_enter_propagation() {
        use crate::renderer::pagination::PageItem;

        let bytes = std::fs::read("samples/kps-ai.hwp").expect("kps-ai.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        let pages_before = doc.pagination[0].pages.len();

        // 분할 전: 각 페이지의 첫 번째/마지막 아이템의 para_index 기록
        let mut before_pages: Vec<(usize, usize, usize)> = Vec::new(); // (first_pi, last_pi, item_count)
        for page in &doc.pagination[0].pages {
            let items = &page.column_contents[0].items;
            let first = items.first().map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            }).unwrap_or(0);
            let last = items.last().map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            }).unwrap_or(0);
            before_pages.push((first, last, items.len()));
        }

        // page 13 (idx=12)의 pi=199 앞에서 엔터
        eprintln!("=== splitParagraph(0, 199, 0) ===");
        let result = doc.split_paragraph_native(0, 199, 0).unwrap();
        assert!(result.contains("\"ok\":true"));

        let pages_after = doc.pagination[0].pages.len();
        eprintln!("pages: {} → {}", pages_before, pages_after);

        // 분할 후: 각 페이지 비교
        let mut last_diff_page = 0;
        for (pidx, page) in doc.pagination[0].pages.iter().enumerate() {
            let items = &page.column_contents[0].items;
            let first = items.first().map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            }).unwrap_or(0);
            let last = items.last().map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            }).unwrap_or(0);

            let before = before_pages.get(pidx);
            let changed = before.map(|b| b.0 != first || b.1 != last || b.2 != items.len()).unwrap_or(true);

            if changed {
                last_diff_page = pidx;
                let before_str = before.map(|b| format!("pi={}-{} ({}items)", b.0, b.1, b.2))
                    .unwrap_or_else(|| "(신규)".to_string());
                eprintln!("  page {:2}: {} → pi={}-{} ({}items) ← CHANGED",
                    pidx + 1, before_str, first, last, items.len());
            }
        }
        eprintln!("전파 범위: page 13 ~ page {} (총 {} 페이지 영향)",
            last_diff_page + 1, last_diff_page + 1 - 12);

        // 저장 후 재로드와 비교
        eprintln!("\n=== 저장 후 재로드 비교 ===");
        let exported = doc.export_hwp_native().unwrap();
        let mut doc2 = HwpDocument::from_bytes(&exported).unwrap();
        doc2.convert_to_editable_native().unwrap();
        doc2.paginate();

        let pages_reload = doc2.pagination[0].pages.len();
        eprintln!("재로드 pages: {}", pages_reload);

        let mut diff_count = 0;
        for pidx in 0..doc.pagination[0].pages.len().max(doc2.pagination[0].pages.len()) {
            let items1 = doc.pagination[0].pages.get(pidx).map(|p| &p.column_contents[0].items);
            let items2 = doc2.pagination[0].pages.get(pidx).map(|p| &p.column_contents[0].items);

            let pi1_first = items1.and_then(|i| i.first()).map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            });
            let pi2_first = items2.and_then(|i| i.first()).map(|it| match it {
                PageItem::FullParagraph { para_index } |
                PageItem::Table { para_index, .. } |
                PageItem::PartialParagraph { para_index, .. } |
                PageItem::PartialTable { para_index, .. } |
                PageItem::Shape { para_index, .. } => *para_index,
            });
            let count1 = items1.map(|i| i.len()).unwrap_or(0);
            let count2 = items2.map(|i| i.len()).unwrap_or(0);

            if pi1_first != pi2_first || count1 != count2 {
                diff_count += 1;
                eprintln!("  page {:2}: 편집={:?}({}items) vs 재로드={:?}({}items)",
                    pidx + 1, pi1_first, count1, pi2_first, count2);
            }
        }
        if diff_count == 0 {
            eprintln!("  차이 없음 — 편집 결과와 재로드 결과 일치");
        } else {
            eprintln!("  {} 페이지에서 차이 발견", diff_count);
        }
    }

    /// 12페이지 각 문단에서 엔터 후 13페이지 표 배치 검증
    #[test]
    fn test_page12_enter_table_placement_scan() {
        use crate::renderer::pagination::PageItem;

        // 12페이지의 각 문단 끝에서 엔터를 입력하는 시나리오
        for split_pi in [194, 196] {
            let bytes = std::fs::read("samples/kps-ai.hwp").expect("kps-ai.hwp 읽기 실패");
            let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
            doc.convert_to_editable_native().unwrap();
            doc.paginate();

            let text_len = doc.document.sections[0].paragraphs[split_pi].text.chars().count();
            let offset = text_len; // 문단 끝에서 분할

            eprintln!("\n=== split pi={} offset={} ===", split_pi, offset);

            // 분할 전 page 13 (idx=12) 확인
            let table_pi_before = 198; // 원래 pi=198의 표
            let p13_before = &doc.pagination[0].pages[12];
            let has_table_before = p13_before.column_contents[0].items.iter()
                .any(|it| matches!(it, PageItem::Table { para_index, .. } if *para_index == table_pi_before));
            eprintln!("  before: pi={} table on page 13: {}", table_pi_before, has_table_before);

            let result = doc.split_paragraph_native(0, split_pi, offset).unwrap();
            assert!(result.contains("\"ok\":true"), "split failed at pi={}: {}", split_pi, result);

            let pages_after = doc.pagination[0].pages.len();
            let table_pi_after = if split_pi < table_pi_before { table_pi_before + 1 } else { table_pi_before };

            // 분할 후: 표가 어느 페이지에 있는지 탐색
            let mut table_page = None;
            for (pidx, page) in doc.pagination[0].pages.iter().enumerate() {
                for item in &page.column_contents[0].items {
                    if matches!(item, PageItem::Table { para_index, .. } if *para_index == table_pi_after) {
                        table_page = Some(pidx);
                    }
                }
            }
            eprintln!("  after: pi={} table on page {} (total {})",
                table_pi_after, table_page.map(|p| p + 1).unwrap_or(0), pages_after);

            // 페이지 12-15 내용 출력
            for pidx in 11..15.min(pages_after) {
                let p = &doc.pagination[0].pages[pidx];
                eprintln!("  page {} items:", pidx + 1);
                for item in &p.column_contents[0].items {
                    match item {
                        PageItem::Table { para_index, control_index } => {
                            let text = &doc.document.sections[0].paragraphs[*para_index].text;
                            eprintln!("    Table pi={} ci={} text='{}'", para_index, control_index,
                                &text[..text.len().min(30)]);
                        }
                        PageItem::FullParagraph { para_index } => {
                            let text = &doc.document.sections[0].paragraphs[*para_index].text;
                            let display: String = if text.is_empty() { "(빈)".to_string() } else { text.chars().take(40).collect() };
                            eprintln!("    FullPara pi={} '{}'", para_index, display);
                        }
                        _ => eprintln!("    {:?}", item),
                    }
                }
            }
        }
    }

    /// 12페이지 엔터 후 13페이지의 표 배치 검증
    #[test]
    fn test_page12_enter_table_placement() {
        use crate::renderer::pagination::PageItem;

        let bytes = std::fs::read("samples/kps-ai.hwp").expect("kps-ai.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        let pages_before = doc.pagination[0].pages.len();
        eprintln!("  pages_before = {}", pages_before);

        // page 12 (idx=11) 내용 확인
        let p12 = &doc.pagination[0].pages[11];
        eprintln!("  page 12 items:");
        for item in &p12.column_contents[0].items {
            eprintln!("    {:?}", item);
        }

        // page 13 (idx=12): pi=197(text), pi=198(table), pi=199(text)
        let p13_before = &doc.pagination[0].pages[12];
        eprintln!("  page 13 items (before):");
        for item in &p13_before.column_contents[0].items {
            eprintln!("    {:?}", item);
        }
        // pi=198 표가 page 13에 있는지 확인
        let has_table_198_on_p13 = p13_before.column_contents[0].items.iter()
            .any(|it| matches!(it, PageItem::Table { para_index: 198, .. }));
        assert!(has_table_198_on_p13, "수정 전: pi=198 표가 page 13에 있어야 함");

        // pi=199 앞에서 엔터 (pi=199를 분할하여 빈 문단 삽입)
        let result = doc.split_paragraph_native(0, 199, 0).unwrap();
        assert!(result.contains("\"ok\":true"), "split failed: {}", result);

        let pages_after = doc.pagination[0].pages.len();
        eprintln!("  pages_after = {}", pages_after);

        // page 13 (idx=12): pi=198 표가 여전히 page 13에 있어야 함
        if doc.pagination[0].pages.len() > 12 {
            let p13_after = &doc.pagination[0].pages[12];
            eprintln!("  page 13 items (after):");
            for item in &p13_after.column_contents[0].items {
                eprintln!("    {:?}", item);
            }
            let has_table_198_after = p13_after.column_contents[0].items.iter()
                .any(|it| matches!(it, PageItem::Table { para_index: 198, .. }));

            // page 14도 확인
            if doc.pagination[0].pages.len() > 13 {
                let p14_after = &doc.pagination[0].pages[13];
                eprintln!("  page 14 items (after):");
                for item in &p14_after.column_contents[0].items {
                    eprintln!("    {:?}", item);
                }
            }

            assert!(has_table_198_after,
                "pi=198 표가 page 13에 있어야 하지만 다음 페이지로 밀려남");
        }
    }

    /// 문단 분할 후 페이지 수가 과도하게 증가하지 않는지 검증
    /// (measure_section_selective의 off-by-one 인덱싱 버그 회귀 방지)
    #[test]
    fn test_split_paragraph_page_count_stability() {
        let bytes = std::fs::read("samples/kps-ai.hwp").expect("kps-ai.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        let pages_before = doc.pagination.iter().map(|r| r.pages.len()).sum::<usize>();
        eprintln!("  pages_before = {}", pages_before);

        // pi=199 앞에서 엔터 (offset=0으로 분할)
        let result = doc.split_paragraph_native(0, 199, 0).unwrap();
        assert!(result.contains("\"ok\":true"), "split failed: {}", result);

        let pages_after = doc.pagination.iter().map(|r| r.pages.len()).sum::<usize>();
        eprintln!("  pages_after = {}", pages_after);

        // 한 줄 추가이므로 페이지 수 증가는 최대 2 이내여야 함
        let delta = pages_after as i64 - pages_before as i64;
        eprintln!("  delta = {}", delta);
        assert!(
            delta <= 2,
            "문단 분할 후 페이지 수가 {}에서 {}로 {}만큼 증가 (최대 2 예상)",
            pages_before, pages_after, delta
        );
    }

    /// 논리적 오프셋: 인라인 TAC 표 뒤에서 텍스트 삽입 검증
    #[test]
    fn test_logical_offset_insert_after_inline_table() {
        let bytes = std::fs::read("saved/blank2010.hwp").expect("blank2010.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        // Enter로 새 문단 생성 (기존 컨트롤이 있는 pi=0 대신 깨끗한 pi=1 사용)
        doc.insert_text_native(0, 0, 0, "test").unwrap();
        doc.split_paragraph_native(0, 0, 4).unwrap();

        // pi=1에 "abc" 입력
        doc.insert_text_native(0, 1, 0, "abc").unwrap();
        let para = &doc.document.sections[0].paragraphs[1];
        assert_eq!(para.text, "abc");
        eprintln!("  pi=1 controls={} (표 삽입 전)", para.controls.len());

        // offset=3 위치에 인라인 TAC 2×2 표 삽입
        let result = doc.create_table_ex_native(0, 1, 3, 2, 2, true, Some(&[6777, 6777])).unwrap();
        eprintln!("  createTableEx result: {}", result);
        // logicalOffset: "abc"(3) + [표](1) = 4
        assert!(result.contains("\"logicalOffset\":4"), "logicalOffset=4 예상: {}", result);

        let para = &doc.document.sections[0].paragraphs[1];
        eprintln!("  text='{}' controls={} char_offsets={:?}", para.text, para.controls.len(), para.char_offsets);

        // 논리적 길이: "abc"(3) + [표](1) = 4
        let logical_len = crate::document_core::helpers::logical_paragraph_length(para);
        eprintln!("  논리적 길이: {}", logical_len);
        assert_eq!(logical_len, 4, "논리적 길이 4 예상, 실제: {}", logical_len);

        // 논리적 offset 4에 "XYZ" 삽입 → 표 뒤에 삽입되어야 함
        let (text_off, after_ctrl) = crate::document_core::helpers::logical_to_text_offset(para, 4);
        eprintln!("  logical 4 → text_off={} after_ctrl={}", text_off, after_ctrl);
        assert_eq!(text_off, 3, "text_off=3 예상 (abc 뒤)");

        doc.insert_text_native(0, 1, text_off, "XYZ").unwrap();
        let para = &doc.document.sections[0].paragraphs[1];
        assert_eq!(para.text, "abcXYZ", "표 뒤에 XYZ 삽입 예상, 실제: '{}'", para.text);
        eprintln!("  삽입 후 text='{}' ✓", para.text);

        // 논리적 길이: "abcXYZ"(6) + [표](1) = 7
        let logical_len2 = crate::document_core::helpers::logical_paragraph_length(para);
        assert_eq!(logical_len2, 7, "논리적 길이 7 예상, 실제: {}", logical_len2);

        // logical offset 변환 검증 (삽입 후: "abcXYZ" + [표at3])
        // a(0) b(1) c(2) [표](3) X(4) Y(5) Z(6)
        let (t0, _) = crate::document_core::helpers::logical_to_text_offset(para, 0);
        let (t3, _) = crate::document_core::helpers::logical_to_text_offset(para, 3);
        let (t4, _) = crate::document_core::helpers::logical_to_text_offset(para, 4);
        let (t7, _) = crate::document_core::helpers::logical_to_text_offset(para, 7);
        eprintln!("  logical→text: 0→{} 3→{} 4→{} 7→{}", t0, t3, t4, t7);
        assert_eq!(t0, 0, "logical 0 → text 0");
        assert_eq!(t3, 3, "logical 3 → text 3 (표 위치, [표] = ctrl at text pos 3)");
        assert_eq!(t4, 3 + 1, "logical 4 → text 4 (X, 표 뒤 첫 텍스트)");
        assert_eq!(t7, 6, "logical 7 → text 6 (끝)");

        // ── 핵심 검증: charOffset > text_len으로 직접 삽입 ──
        // 새 문서에서 "가나다" + [표] 구조 생성, charOffset=4로 삽입
        doc.split_paragraph_native(0, 1, 6).unwrap(); // pi=2 생성
        doc.insert_text_native(0, 2, 0, "가나다").unwrap();
        doc.create_table_ex_native(0, 2, 3, 1, 1, true, Some(&[5000])).unwrap();
        let para2 = &doc.document.sections[0].paragraphs[2];
        let tl = para2.text.chars().count();
        eprintln!("  pi=2: text='{}' len={} controls={}", para2.text, tl, para2.controls.len());
        // charOffset=4 (> text_len=3) → 표 뒤에 삽입
        doc.insert_text_native(0, 2, 4, "라마바").unwrap();
        let para2 = &doc.document.sections[0].paragraphs[2];
        eprintln!("  charOffset=4 삽입 후: '{}'", para2.text);
        assert_eq!(para2.text, "가나다라마바", "표 뒤에 '라마바' 삽입, 실제: '{}'", para2.text);

        eprintln!("  논리적 오프셋 테스트 통과 ✓");
    }

    /// createTableEx: 빈 문서에서 인라인 TAC 표를 생성하여 tac-case-001.hwp와 동일한 구조 검증
    #[test]
    fn test_create_inline_tac_table() {
        let bytes = std::fs::read("saved/blank2010.hwp").expect("blank2010.hwp 읽기 실패");
        let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
        doc.convert_to_editable_native().unwrap();
        doc.paginate();

        // 1. pi=0에 "TC #20" 입력
        doc.insert_text_native(0, 0, 0, "TC #20").unwrap();
        // 2. Enter → pi=1 생성
        doc.split_paragraph_native(0, 0, 6).unwrap();
        // 3. pi=1에 "tacglkj 표 3 배치 시작" 입력
        doc.insert_text_native(0, 1, 0, "tacglkj 표 3 배치 시작").unwrap();

        let text_len = doc.document.sections[0].paragraphs[1].text.chars().count();
        eprintln!("  pi=1 text='{}' len={}", doc.document.sections[0].paragraphs[1].text, text_len);

        // 4. pi=1, char_offset=text_len 위치에 인라인 TAC 2×2 표 생성
        // 열 폭: 6777 HU × 2 = 13554 HU (tac-case-001.hwp과 동일)
        let result = doc.create_table_ex_native(0, 1, text_len, 2, 2, true, Some(&[6777, 6777])).unwrap();
        eprintln!("  createTableEx result: {}", result);
        assert!(result.contains("\"ok\":true"), "createTableEx 실패: {}", result);

        // 5. 표 뒤에 "4 tacglkj 표 다음" 텍스트 추가
        let para = &doc.document.sections[0].paragraphs[1];
        let new_text_offset = para.text.chars().count();
        doc.insert_text_native(0, 1, new_text_offset, "4 tacglkj 표 다음").unwrap();

        // 6. 검증
        let para = &doc.document.sections[0].paragraphs[1];
        eprintln!("  pi=1 final text='{}' controls={}", para.text, para.controls.len());

        // 표가 controls에 추가되었는지
        assert_eq!(para.controls.len(), 1, "pi=1에 표 컨트롤 1개 예상");
        if let crate::model::control::Control::Table(t) = &para.controls[0] {
            assert!(t.common.treat_as_char, "treat_as_char=true 예상");
            assert_eq!(t.row_count, 2, "행 수 2 예상");
            assert_eq!(t.col_count, 2, "열 수 2 예상");
            eprintln!("  표: {}×{} tac={} width={} height={}",
                t.row_count, t.col_count, t.common.treat_as_char, t.common.width, t.common.height);
        } else {
            panic!("pi=1의 첫 컨트롤이 Table이 아님");
        }

        // 셀에 텍스트 입력
        doc.insert_text_in_cell_native(0, 1, 0, 0, 0, 0, "1").unwrap();
        doc.insert_text_in_cell_native(0, 1, 0, 1, 0, 0, "2").unwrap();
        doc.insert_text_in_cell_native(0, 1, 0, 2, 0, 0, "3 tacglkj").unwrap();
        doc.insert_text_in_cell_native(0, 1, 0, 3, 0, 0, "4 tacglkj").unwrap();

        // Enter → pi=2
        let pi1_len = doc.document.sections[0].paragraphs[1].text.chars().count();
        doc.split_paragraph_native(0, 1, pi1_len).unwrap();
        // pi=2에 텍스트
        doc.insert_text_native(0, 2, 0, "tacglkj 가나 옮").unwrap();

        // 페이지네이션
        doc.paginate();
        let page_count: usize = doc.pagination.iter().map(|r| r.pages.len()).sum();
        eprintln!("  최종 페이지 수: {}", page_count);
        assert_eq!(page_count, 1, "1페이지 문서 예상");

        // 텍스트에 표가 포함된 인라인 배치 확인
        let para = &doc.document.sections[0].paragraphs[1];
        assert!(!para.text.is_empty(), "pi=1에 텍스트가 있어야 함");
        assert_eq!(para.controls.len(), 1, "pi=1에 인라인 표 1개");

        // is_tac_table_inline 확인
        let seg_w = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
        if let crate::model::control::Control::Table(t) = &para.controls[0] {
            let is_inline = crate::renderer::height_measurer::is_tac_table_inline(
                t, seg_w, &para.text, &para.controls);
            eprintln!("  is_tac_table_inline: {} (seg_w={})", is_inline, seg_w);
            assert!(is_inline, "인라인 TAC 표로 판별되어야 함");
        }

        eprintln!("  인라인 TAC 표 생성 테스트 통과");
    }

    #[test]
    fn test_extract_thumbnail_with_preview() {
        // PrvImage가 있는 HWP 파일 테스트
        let data = std::fs::read("samples/biz_plan.hwp").expect("biz_plan.hwp 읽기 실패");
        let result = crate::parser::extract_thumbnail_only(&data);
        if let Some(ref r) = result {
            eprintln!("  biz_plan.hwp 썸네일: format={}, size={}bytes, {}x{}", r.format, r.data.len(), r.width, r.height);
            eprintln!("  매직 바이트: {:02x?}", &r.data[..std::cmp::min(16, r.data.len())]);
        } else {
            eprintln!("  biz_plan.hwp 썸네일: None");
        }
        // PrvImage 유무와 상관없이 패닉하지 않아야 함
    }

    #[test]
    fn test_extract_thumbnail_without_preview() {
        // 잘못된 데이터에서는 None 반환
        let result = crate::parser::extract_thumbnail_only(&[0u8; 100]);
        assert!(result.is_none(), "잘못된 데이터에서는 None이어야 함");

        // 빈 바이트에서도 패닉하지 않아야 함
        let result = crate::parser::extract_thumbnail_only(&[]);
        assert!(result.is_none(), "빈 데이터에서는 None이어야 함");
        eprintln!("  잘못된/빈 데이터 썸네일: None (정상)");
    }

    // ---------- #177: getValidationWarnings / reflowLinesegs WASM API ----------

    #[test]
    fn test_get_validation_warnings_empty_document() {
        // 빈 문서는 경고 없음.
        let doc = HwpDocument::create_empty();
        let json = doc.get_validation_warnings();
        assert!(json.contains(r#""count":0"#), "empty doc must have count:0, got: {}", json);
        assert!(json.contains(r#""warnings":[]"#));
    }

    #[test]
    fn test_get_validation_warnings_json_shape() {
        // JSON 구조 검증 — 빈 문서라도 최소 형태를 갖춰야 함.
        let doc = HwpDocument::create_empty();
        let json = doc.get_validation_warnings();
        // 필수 키: count, summary, warnings
        assert!(json.contains(r#""count":"#));
        assert!(json.contains(r#""summary":"#));
        assert!(json.contains(r#""warnings":"#));
    }

    #[test]
    fn test_reflow_linesegs_empty_document_returns_zero() {
        // 빈 문서에선 reflow 대상 없음 → 0 반환.
        let mut doc = HwpDocument::create_empty();
        let count = doc.reflow_linesegs();
        assert_eq!(count, 0);
    }
