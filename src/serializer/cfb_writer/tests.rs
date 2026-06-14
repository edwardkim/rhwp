use super::*;
use crate::model::document::*;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::style::*;
use crate::parser::cfb_reader::decompress_stream;

#[test]
fn test_compress_decompress_roundtrip() {
    let original = b"Hello, HWP World! Test data for compression roundtrip.";
    let compressed = compress_stream(original).unwrap();
    let decompressed = decompress_stream(&compressed).unwrap();
    assert_eq!(decompressed, original);
}

#[test]
fn test_compress_empty_data() {
    let original = b"";
    let compressed = compress_stream(original).unwrap();
    let decompressed = decompress_stream(&compressed).unwrap();
    assert_eq!(decompressed, original);
}

#[test]
fn test_serialize_hwp_empty_document() {
    let doc = Document::default();
    let bytes = serialize_hwp(&doc).unwrap();
    // CFB 시그니처 확인 (0xD0CF11E0A1B11AE1)
    assert!(bytes.len() > 512);
    assert_eq!(&bytes[0..4], &[0xD0, 0xCF, 0x11, 0xE0]);
}

#[test]
fn test_serialize_hwp_cfb_streams() {
    let doc = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0,
            compressed: false,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            ..Default::default()
        },
        doc_info: DocInfo::default(),
        sections: vec![crate::model::document::Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph {
                text: "테스트".to_string(),
                line_segs: vec![LineSeg {
                    line_height: 400,
                    baseline_distance: 320,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            raw_stream: None,
        }],
        preview: None,
        bin_data_content: Vec::new(),
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    let bytes = serialize_hwp(&doc).unwrap();

    // CFB로 읽어서 스트림 확인
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&bytes).unwrap();
    assert!(cfb.has_stream("/FileHeader"));
    assert!(cfb.has_stream("/DocInfo"));
    assert!(cfb.has_stream("/BodyText/Section0"));

    // FileHeader 크기 확인
    let header = cfb.read_file_header().unwrap();
    assert_eq!(header.len(), 256);
}

#[test]
fn test_serialize_hwp_compressed() {
    let doc = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0x01,
            compressed: true,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            ..Default::default()
        },
        doc_info: DocInfo::default(),
        sections: vec![crate::model::document::Section::default()],
        preview: None,
        bin_data_content: Vec::new(),
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    let bytes = serialize_hwp(&doc).unwrap();

    // CFB로 읽고 DocInfo가 압축 해제 가능한지 확인
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&bytes).unwrap();
    let doc_info_raw = cfb.read_stream_raw("/DocInfo").unwrap();
    let decompressed = decompress_stream(&doc_info_raw).unwrap();
    assert!(!decompressed.is_empty());
}

#[test]
fn test_full_roundtrip_uncompressed() {
    // 최소 Document 구성
    let mut doc_info = DocInfo::default();
    doc_info.font_faces = vec![Vec::new(); 7];
    doc_info.font_faces[0].push(Font {
        raw_data: None,
        name: "함초롬바탕".to_string(),
        alt_type: 0,
        alt_name: None,
        type_info: None,
        default_name: None,
        subst_font: None,
    });
    doc_info.char_shapes.push(CharShape {
        font_ids: [0; 7],
        ratios: [100; 7],
        spacings: [0; 7],
        relative_sizes: [100; 7],
        char_offsets: [0; 7],
        base_size: 1000,
        attr: 0,
        text_color: 0,
        underline_color: 0,
        shade_color: 0x00FFFFFF,
        shadow_color: 0x00B2B2B2,
        strike_color: 0,
        ..Default::default()
    });
    doc_info.para_shapes.push(ParaShape {
        line_spacing: 160,
        ..Default::default()
    });
    doc_info.styles.push(Style {
        local_name: "바탕글".to_string(),
        english_name: "Normal".to_string(),
        ..Default::default()
    });

    let original = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0,
            compressed: false,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            footnote_start_num: 1,
            endnote_start_num: 1,
            picture_start_num: 1,
            table_start_num: 1,
            equation_start_num: 1,
            raw_data: None,
            caret_list_id: 0,
            caret_para_id: 0,
            caret_char_pos: 0,
        },
        doc_info,
        sections: vec![crate::model::document::Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph {
                text: "안녕하세요".to_string(),
                char_count: 6, // 5문자 + 문단 끝
                line_segs: vec![LineSeg {
                    line_height: 400,
                    baseline_distance: 320,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            raw_stream: None,
        }],
        preview: None,
        bin_data_content: Vec::new(),
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    // Document → HWP bytes
    let hwp_bytes = serialize_hwp(&original).unwrap();

    // HWP bytes → CFB → 스트림 읽기
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&hwp_bytes).unwrap();

    // FileHeader 라운드트립
    let header_data = cfb.read_file_header().unwrap();
    let parsed_header = crate::parser::header::parse_file_header(&header_data).unwrap();
    assert_eq!(parsed_header.version.major, 5);
    assert!(!parsed_header.flags.compressed);

    // DocInfo 라운드트립
    let doc_info_data = cfb.read_doc_info(false).unwrap();
    let (parsed_info, parsed_props) =
        crate::parser::doc_info::parse_doc_info(&doc_info_data).unwrap();
    assert_eq!(parsed_props.section_count, 1);
    assert_eq!(parsed_info.font_faces[0][0].name, "함초롬바탕");
    assert_eq!(parsed_info.styles[0].local_name, "바탕글");

    // BodyText 라운드트립
    let section_data = cfb.read_body_text_section(0, false, false).unwrap();
    let parsed_section = crate::parser::body_text::parse_body_text_section(&section_data).unwrap();
    assert_eq!(parsed_section.paragraphs.len(), 1);
    assert_eq!(parsed_section.paragraphs[0].text, "안녕하세요");
}

#[test]
fn test_full_roundtrip_compressed() {
    let original = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0x01,
            compressed: true,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            footnote_start_num: 1,
            endnote_start_num: 1,
            picture_start_num: 1,
            table_start_num: 1,
            equation_start_num: 1,
            raw_data: None,
            caret_list_id: 0,
            caret_para_id: 0,
            caret_char_pos: 0,
        },
        doc_info: DocInfo::default(),
        sections: vec![crate::model::document::Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph {
                text: "Hello World".to_string(),
                char_count: 12,
                line_segs: vec![LineSeg {
                    line_height: 400,
                    baseline_distance: 320,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            raw_stream: None,
        }],
        preview: None,
        bin_data_content: Vec::new(),
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    // Document → HWP bytes (compressed)
    let hwp_bytes = serialize_hwp(&original).unwrap();

    // HWP bytes → CFB → 압축 해제 → 스트림 읽기
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&hwp_bytes).unwrap();

    // DocInfo 라운드트립 (압축 해제)
    let doc_info_data = cfb.read_doc_info(true).unwrap();
    let (_parsed_info, parsed_props) =
        crate::parser::doc_info::parse_doc_info(&doc_info_data).unwrap();
    assert_eq!(parsed_props.section_count, 1);

    // BodyText 라운드트립 (압축 해제)
    let section_data = cfb.read_body_text_section(0, true, false).unwrap();
    let parsed_section = crate::parser::body_text::parse_body_text_section(&section_data).unwrap();
    assert_eq!(parsed_section.paragraphs[0].text, "Hello World");
}

#[test]
fn test_serialize_after_edit() {
    use std::path::Path;

    let path = Path::new("samples/hwp-3.0-HWPML.hwp");
    if !path.exists() {
        eprintln!("샘플 파일 없음 — 건너뜀");
        return;
    }

    let data = std::fs::read(path).unwrap();
    let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();

    // 첫 번째 문단에 텍스트 삽입
    let result = doc.insert_text_native(0, 0, 0, "테스트");
    eprintln!("insert result: {:?}", result);
    assert!(result.is_ok());

    // 직렬화
    match doc.export_hwp_native() {
        Ok(bytes) => {
            eprintln!("편집 후 직렬화 성공: {}KB", bytes.len() / 1024);
            assert_eq!(&bytes[0..4], &[0xD0, 0xCF, 0x11, 0xE0]);
        }
        Err(e) => {
            panic!("편집 후 직렬화 실패: {}", e);
        }
    }
}

#[test]
fn test_serialize_after_edit_roundtrip() {
    use std::path::Path;

    // 여러 샘플 파일에 대해 편집 후 라운드트립 검증
    let files = [
        "samples/hwp-3.0-HWPML.hwp",
        "samples/hwp-multi-001.hwp",
        "samples/20250130-hongbo.hwp",
    ];

    for file_path in &files {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("{} 없음 — 건너뜀", file_path);
            continue;
        }

        let data = std::fs::read(path).unwrap();
        let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();

        // rhwp-studio와 동일하게 convertToEditable 호출
        let _ = doc.convert_to_editable_native();

        // 텍스트 삽입
        let result = doc.insert_text_native(0, 0, 0, "테스트추가");
        assert!(result.is_ok(), "{}: 텍스트 삽입 실패", file_path);

        // 직렬화
        let bytes = doc
            .export_hwp_native()
            .unwrap_or_else(|e| panic!("{}: 직렬화 실패: {}", file_path, e));

        // CFB 매직 확인
        assert_eq!(
            &bytes[0..4],
            &[0xD0, 0xCF, 0x11, 0xE0],
            "{}: CFB 매직 불일치",
            file_path
        );

        // 라운드트립: 다시 파싱 가능한지 검증
        let parsed = crate::parser::parse_hwp(&bytes);
        assert!(
            parsed.is_ok(),
            "{}: 라운드트립 파싱 실패: {:?}",
            file_path,
            parsed.err()
        );

        let parsed = parsed.unwrap();
        let para_text = &parsed.sections[0].paragraphs[0].text;
        assert!(
            para_text.starts_with("테스트추가"),
            "{}: 삽입된 텍스트 미발견, 실제: '{}'",
            file_path,
            &para_text[..para_text.len().min(30)]
        );

        eprintln!("{}: 라운드트립 성공 ({}KB)", file_path, bytes.len() / 1024);
    }
}

#[test]
fn test_serialize_real_hwp_files() {
    use std::path::Path;

    let sample_dir = Path::new("samples");
    if !sample_dir.exists() {
        eprintln!("samples/ 디렉토리 없음 — 건너뜀");
        return;
    }

    for entry in std::fs::read_dir(sample_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("hwp") {
            continue;
        }
        let fname = path.file_name().unwrap().to_string_lossy().to_string();
        eprintln!("테스트: {}", fname);

        let data = std::fs::read(&path).unwrap();
        let doc = match crate::parser::parse_hwp(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  파싱 실패 (건너뜀): {}", e);
                continue;
            }
        };

        match serialize_hwp(&doc) {
            Ok(bytes) => {
                eprintln!("  직렬화 성공: {}KB", bytes.len() / 1024);
                // CFB 시그니처 확인
                assert_eq!(
                    &bytes[0..4],
                    &[0xD0, 0xCF, 0x11, 0xE0],
                    "{}: CFB 시그니처 불일치",
                    fname
                );
            }
            Err(e) => {
                panic!("{}: 직렬화 실패: {}", fname, e);
            }
        }
    }
}

/// 표 구조 변경(행/열 추가/삭제) 후 라운드트립 검증
#[test]
fn test_table_structure_change_roundtrip() {
    use std::path::Path;

    let path = Path::new("samples/hwp_table_test.hwp");
    if !path.exists() {
        eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
        return;
    }

    let data = std::fs::read(path).unwrap();

    // 행 추가 라운드트립
    {
        let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();
        let _ = doc.convert_to_editable_native();
        doc.insert_table_row_native(0, 3, 0, 0, true).unwrap();
        let bytes = doc.export_hwp_native().unwrap();
        let parsed = crate::parser::parse_hwp(&bytes);
        assert!(
            parsed.is_ok(),
            "행 추가 후 라운드트립 실패: {:?}",
            parsed.err()
        );
        eprintln!("행 추가 라운드트립: 성공");
    }

    // 열 추가 라운드트립
    {
        let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();
        let _ = doc.convert_to_editable_native();
        doc.insert_table_column_native(0, 3, 0, 0, true).unwrap();
        let bytes = doc.export_hwp_native().unwrap();
        let parsed = crate::parser::parse_hwp(&bytes);
        assert!(
            parsed.is_ok(),
            "열 추가 후 라운드트립 실패: {:?}",
            parsed.err()
        );
        eprintln!("열 추가 라운드트립: 성공");
    }

    // 행 삭제 라운드트립
    {
        let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();
        let _ = doc.convert_to_editable_native();
        doc.delete_table_row_native(0, 3, 0, 0).unwrap();
        let bytes = doc.export_hwp_native().unwrap();
        let parsed = crate::parser::parse_hwp(&bytes);
        assert!(
            parsed.is_ok(),
            "행 삭제 후 라운드트립 실패: {:?}",
            parsed.err()
        );
        eprintln!("행 삭제 라운드트립: 성공");
    }

    // 열 삭제 라운드트립
    {
        let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();
        let _ = doc.convert_to_editable_native();
        doc.delete_table_column_native(0, 3, 0, 0).unwrap();
        let bytes = doc.export_hwp_native().unwrap();
        let parsed = crate::parser::parse_hwp(&bytes);
        assert!(
            parsed.is_ok(),
            "열 삭제 후 라운드트립 실패: {:?}",
            parsed.err()
        );
        eprintln!("열 삭제 라운드트립: 성공");
    }
}

/// 표 컨트롤 삭제 + 라운드트립 테스트
#[test]
fn test_delete_table_control_roundtrip() {
    use std::path::Path;

    let path = Path::new("samples/hwp_table_test.hwp");
    if !path.exists() {
        eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
        return;
    }

    let data = std::fs::read(path).unwrap();
    let mut doc = crate::wasm_api::HwpDocument::from_bytes(&data).unwrap();
    let _ = doc.convert_to_editable_native();

    // 표 삭제
    let result = doc.delete_table_control_native(0, 3, 0);
    assert!(result.is_ok(), "표 삭제 실패: {:?}", result.err());

    // 라운드트립: 직렬화 → 파싱
    let bytes = doc.export_hwp_native().unwrap();
    let parsed = crate::parser::parse_hwp(&bytes);
    assert!(
        parsed.is_ok(),
        "표 삭제 후 라운드트립 실패: {:?}",
        parsed.err()
    );
    eprintln!("표 삭제 라운드트립: 성공");
}

/// 원본 HWP와 직렬화 결과를 스트림별로 비교하는 진단 테스트
#[test]
fn test_roundtrip_stream_comparison() {
    use crate::parser::record::Record;
    use std::path::Path;

    let path = Path::new("samples/hwp_table_test.hwp");
    if !path.exists() {
        eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
        return;
    }

    let original_data = std::fs::read(path).unwrap();

    // 원본 스트림 읽기
    let mut orig_cfb = crate::parser::cfb_reader::CfbReader::open(&original_data).unwrap();
    let orig_streams = orig_cfb.list_streams();
    eprintln!("=== 원본 스트림 목록 ===");
    for s in &orig_streams {
        eprintln!("  {}", s);
    }

    let orig_header = orig_cfb.read_file_header().unwrap();
    let orig_doc_info_raw = orig_cfb.read_stream_raw("/DocInfo").unwrap();
    let orig_section_raw = orig_cfb.read_stream_raw("/BodyText/Section0").unwrap();

    // 파싱
    let doc = crate::parser::parse_hwp(&original_data).unwrap();
    eprintln!("\n=== Document 구조 ===");
    eprintln!(
        "  버전: {}.{}.{}.{}",
        doc.header.version.major,
        doc.header.version.minor,
        doc.header.version.build,
        doc.header.version.revision
    );
    eprintln!("  flags: 0x{:08X}", doc.header.flags);
    eprintln!("  compressed: {}", doc.header.compressed);
    eprintln!("  섹션수: {}", doc.sections.len());
    eprintln!("  폰트그룹수: {}", doc.doc_info.font_faces.len());
    for (i, fg) in doc.doc_info.font_faces.iter().enumerate() {
        if !fg.is_empty() {
            eprintln!("    그룹{}: {} 폰트", i, fg.len());
        }
    }
    eprintln!("  char_shapes: {}", doc.doc_info.char_shapes.len());
    eprintln!("  para_shapes: {}", doc.doc_info.para_shapes.len());
    eprintln!("  border_fills: {}", doc.doc_info.border_fills.len());
    eprintln!("  styles: {}", doc.doc_info.styles.len());
    eprintln!("  tab_defs: {}", doc.doc_info.tab_defs.len());
    eprintln!("  numberings: {}", doc.doc_info.numberings.len());
    eprintln!("  bin_data: {}", doc.doc_info.bin_data_list.len());
    eprintln!("  preview: {:?}", doc.preview.is_some());

    // 직렬화
    let serialized = serialize_hwp(&doc).unwrap();

    // 직렬화 결과 스트림 읽기
    let mut ser_cfb = crate::parser::cfb_reader::CfbReader::open(&serialized).unwrap();
    let ser_streams = ser_cfb.list_streams();
    eprintln!("\n=== 직렬화 결과 스트림 목록 ===");
    for s in &ser_streams {
        eprintln!("  {}", s);
    }

    // 누락된 스트림
    eprintln!("\n=== 누락된 스트림 ===");
    for s in &orig_streams {
        if !ser_streams.contains(s) {
            eprintln!("  MISSING: {}", s);
        }
    }

    // FileHeader 비교
    let ser_header = ser_cfb.read_file_header().unwrap();
    eprintln!("\n=== FileHeader 비교 (256바이트) ===");
    eprintln!(
        "  원본 크기: {}, 직렬화 크기: {}",
        orig_header.len(),
        ser_header.len()
    );
    let mut header_diffs = 0;
    for i in 0..256.min(orig_header.len()).min(ser_header.len()) {
        if orig_header[i] != ser_header[i] {
            header_diffs += 1;
            if header_diffs <= 20 {
                eprintln!(
                    "  [{}] 원본=0x{:02X} 직렬화=0x{:02X}",
                    i, orig_header[i], ser_header[i]
                );
            }
        }
    }
    eprintln!("  FileHeader 차이: {} 바이트", header_diffs);

    // DocInfo 비교 (raw = 압축 상태)
    let ser_doc_info_raw = ser_cfb.read_stream_raw("/DocInfo").unwrap();
    eprintln!("\n=== DocInfo 스트림 비교 ===");
    eprintln!("  원본 raw 크기: {}", orig_doc_info_raw.len());
    eprintln!("  직렬화 raw 크기: {}", ser_doc_info_raw.len());

    // 압축 해제 후 레코드 비교
    let orig_doc_info = if doc.header.compressed {
        decompress_stream(&orig_doc_info_raw).unwrap()
    } else {
        orig_doc_info_raw.clone()
    };
    let ser_doc_info = if doc.header.compressed {
        decompress_stream(&ser_doc_info_raw).unwrap()
    } else {
        ser_doc_info_raw.clone()
    };
    eprintln!("  원본 해제 크기: {}", orig_doc_info.len());
    eprintln!("  직렬화 해제 크기: {}", ser_doc_info.len());

    // 레코드별 비교
    let orig_records = Record::read_all(&orig_doc_info).unwrap();
    let ser_records = Record::read_all(&ser_doc_info).unwrap();
    eprintln!("\n=== DocInfo 레코드 비교 ===");
    eprintln!("  원본 레코드 수: {}", orig_records.len());
    eprintln!("  직렬화 레코드 수: {}", ser_records.len());
    let max = orig_records.len().max(ser_records.len());
    for i in 0..max {
        let orig_r = orig_records.get(i);
        let ser_r = ser_records.get(i);
        match (orig_r, ser_r) {
            (Some(o), Some(s)) => {
                let tag_match = o.tag_id == s.tag_id;
                let level_match = o.level == s.level;
                let data_match = o.data == s.data;
                if !tag_match || !level_match || !data_match {
                    let tag_name = crate::parser::tags::tag_name(o.tag_id);
                    eprintln!(
                        "  [{}] {} (tag={}, level={}, size={})",
                        i,
                        tag_name,
                        o.tag_id,
                        o.level,
                        o.data.len()
                    );
                    if !tag_match {
                        eprintln!("       TAG 불일치: 원본={} 직렬화={}", o.tag_id, s.tag_id);
                    }
                    if !level_match {
                        eprintln!("       LEVEL 불일치: 원본={} 직렬화={}", o.level, s.level);
                    }
                    if !data_match {
                        eprintln!(
                            "       DATA 불일치: 원본 {}B vs 직렬화 {}B",
                            o.data.len(),
                            s.data.len()
                        );
                        let min_len = o.data.len().min(s.data.len()).min(64);
                        for j in 0..min_len {
                            if o.data[j] != s.data[j] {
                                eprintln!(
                                    "         첫 차이 offset={}: 0x{:02X} vs 0x{:02X}",
                                    j, o.data[j], s.data[j]
                                );
                                break;
                            }
                        }
                    }
                }
            }
            (Some(o), None) => {
                let tag_name = crate::parser::tags::tag_name(o.tag_id);
                eprintln!(
                    "  [{}] 직렬화에 누락: {} (tag={}, size={})",
                    i,
                    tag_name,
                    o.tag_id,
                    o.data.len()
                );
            }
            (None, Some(s)) => {
                let tag_name = crate::parser::tags::tag_name(s.tag_id);
                eprintln!(
                    "  [{}] 직렬화에 추가: {} (tag={}, size={})",
                    i,
                    tag_name,
                    s.tag_id,
                    s.data.len()
                );
            }
            _ => {}
        }
    }

    // BodyText/Section0 비교
    let ser_section_raw = ser_cfb.read_stream_raw("/BodyText/Section0").unwrap();
    eprintln!("\n=== BodyText/Section0 스트림 비교 ===");
    eprintln!("  원본 raw 크기: {}", orig_section_raw.len());
    eprintln!("  직렬화 raw 크기: {}", ser_section_raw.len());

    let orig_section = if doc.header.compressed {
        decompress_stream(&orig_section_raw).unwrap()
    } else {
        orig_section_raw.clone()
    };
    let ser_section = if doc.header.compressed {
        decompress_stream(&ser_section_raw).unwrap()
    } else {
        ser_section_raw.clone()
    };
    eprintln!("  원본 해제 크기: {}", orig_section.len());
    eprintln!("  직렬화 해제 크기: {}", ser_section.len());

    let orig_sec_records = Record::read_all(&orig_section).unwrap();
    let ser_sec_records = Record::read_all(&ser_section).unwrap();
    eprintln!("\n=== BodyText 레코드 비교 ===");
    eprintln!("  원본 레코드 수: {}", orig_sec_records.len());
    eprintln!("  직렬화 레코드 수: {}", ser_sec_records.len());
    let max = orig_sec_records.len().max(ser_sec_records.len());
    for i in 0..max {
        let orig_r = orig_sec_records.get(i);
        let ser_r = ser_sec_records.get(i);
        match (orig_r, ser_r) {
            (Some(o), Some(s)) => {
                let tag_match = o.tag_id == s.tag_id;
                let level_match = o.level == s.level;
                let data_match = o.data == s.data;
                if !tag_match || !level_match || !data_match {
                    let tag_name = crate::parser::tags::tag_name(o.tag_id);
                    eprintln!(
                        "  [{}] {} (tag={}, level={}, size={})",
                        i,
                        tag_name,
                        o.tag_id,
                        o.level,
                        o.data.len()
                    );
                    if !tag_match {
                        let s_tag_name = crate::parser::tags::tag_name(s.tag_id);
                        eprintln!(
                            "       TAG 불일치: 원본={}({}) 직렬화={}({})",
                            o.tag_id, tag_name, s.tag_id, s_tag_name
                        );
                    }
                    if !level_match {
                        eprintln!("       LEVEL 불일치: 원본={} 직렬화={}", o.level, s.level);
                    }
                    if !data_match {
                        eprintln!(
                            "       DATA 불일치: 원본 {}B vs 직렬화 {}B",
                            o.data.len(),
                            s.data.len()
                        );
                        let min_len = o.data.len().min(s.data.len()).min(64);
                        for j in 0..min_len {
                            if o.data[j] != s.data[j] {
                                eprintln!(
                                    "         첫 차이 offset={}: 0x{:02X} vs 0x{:02X}",
                                    j, o.data[j], s.data[j]
                                );
                                break;
                            }
                        }
                    }
                }
            }
            (Some(o), None) => {
                let tag_name = crate::parser::tags::tag_name(o.tag_id);
                eprintln!(
                    "  [{}] 직렬화에 누락: {} (tag={}, level={}, size={})",
                    i,
                    tag_name,
                    o.tag_id,
                    o.level,
                    o.data.len()
                );
            }
            (None, Some(s)) => {
                let tag_name = crate::parser::tags::tag_name(s.tag_id);
                eprintln!(
                    "  [{}] 직렬화에 추가: {} (tag={}, level={}, size={})",
                    i,
                    tag_name,
                    s.tag_id,
                    s.level,
                    s.data.len()
                );
            }
            _ => {}
        }
    }

    // 직렬화 결과를 다시 파싱해서 구조 확인
    eprintln!("\n=== 직렬화 결과 재파싱 ===");
    match crate::parser::parse_hwp(&serialized) {
        Ok(reparsed) => {
            eprintln!("  재파싱 성공!");
            eprintln!("  섹션수: {}", reparsed.sections.len());
            for (i, sec) in reparsed.sections.iter().enumerate() {
                eprintln!("  Section{}: {} 문단", i, sec.paragraphs.len());
            }
        }
        Err(e) => {
            eprintln!("  재파싱 실패: {}", e);
        }
    }

    // 저장본 파일 비교 (있으면)
    let saved_path = Path::new("samples/hwp_table_test_saved.hwp");
    if saved_path.exists() {
        let saved_data = std::fs::read(saved_path).unwrap();
        eprintln!("\n=== 브라우저 저장본 분석 ===");
        eprintln!("  원본 크기: {} bytes", original_data.len());
        eprintln!("  저장본 크기: {} bytes", saved_data.len());
        eprintln!("  네이티브 직렬화 크기: {} bytes", serialized.len());

        match crate::parser::cfb_reader::CfbReader::open(&saved_data) {
            Ok(mut saved_cfb) => {
                let saved_streams = saved_cfb.list_streams();
                eprintln!("  저장본 스트림: {:?}", saved_streams);

                if saved_cfb.has_stream("/FileHeader") {
                    let saved_fh = saved_cfb.read_file_header().unwrap();
                    eprintln!("  FileHeader: {} bytes", saved_fh.len());
                    // 시그니처 확인
                    if saved_fh.len() >= 32 {
                        let sig = std::str::from_utf8(&saved_fh[0..17]).unwrap_or("?");
                        eprintln!("    시그니처: '{}'", sig);
                        eprintln!(
                            "    버전: {}.{}.{}.{}",
                            saved_fh[35], saved_fh[34], saved_fh[33], saved_fh[32]
                        );
                        let flags = u32::from_le_bytes([
                            saved_fh[36],
                            saved_fh[37],
                            saved_fh[38],
                            saved_fh[39],
                        ]);
                        eprintln!("    flags: 0x{:08X} (compressed={})", flags, flags & 1 != 0);
                    }
                }
                if saved_cfb.has_stream("/DocInfo") {
                    let saved_di = saved_cfb.read_stream_raw("/DocInfo").unwrap();
                    eprintln!("  DocInfo raw: {} bytes", saved_di.len());
                }
                if saved_cfb.has_stream("/BodyText/Section0") {
                    let saved_bt = saved_cfb.read_stream_raw("/BodyText/Section0").unwrap();
                    eprintln!("  BodyText/Section0 raw: {} bytes", saved_bt.len());
                }

                // 저장본 파싱 시도
                match crate::parser::parse_hwp(&saved_data) {
                    Ok(saved_doc) => {
                        eprintln!("  저장본 파싱 성공!");
                        eprintln!("    섹션수: {}", saved_doc.sections.len());
                        for (i, sec) in saved_doc.sections.iter().enumerate() {
                            eprintln!("    Section{}: {} 문단", i, sec.paragraphs.len());
                            for (j, p) in sec.paragraphs.iter().enumerate() {
                                let truncated: String = p.text.chars().take(40).collect();
                                eprintln!("      P{}: '{}' (chars={})", j, truncated, p.char_count);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  저장본 파싱 실패: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  저장본 CFB 열기 실패: {}", e);
            }
        }
    }

    // 직렬화 결과를 파일에 저장 (디버깅용)
    let _ = std::fs::write("output/roundtrip_test.hwp", &serialized);
    eprintln!("\n직렬화 결과: output/roundtrip_test.hwp");
}

#[test]
fn test_cfb_structure_comparison() {
    use std::path::Path;

    let orig_path = Path::new("samples/hwp_table_test.hwp");
    let saved_path = Path::new("samples/hwp_table_test_saved.hwp");
    if !orig_path.exists() || !saved_path.exists() {
        eprintln!("파일 없음 — 건너뜀");
        return;
    }

    let orig_data = std::fs::read(orig_path).unwrap();
    let saved_data = std::fs::read(saved_path).unwrap();

    eprintln!("\n=== CFB 헤더 비교 (512바이트) ===");
    eprintln!(
        "원본 크기: {} bytes ({} sectors)",
        orig_data.len(),
        (orig_data.len() - 512) / 512
    );
    eprintln!(
        "저장본 크기: {} bytes ({} sectors)",
        saved_data.len(),
        (saved_data.len() - 512) / 512
    );

    // 헤더 주요 필드 비교
    let orig_hdr = &orig_data[..512];
    let saved_hdr = &saved_data[..512];

    let fields = [
        (0, 8, "Signature"),
        (24, 26, "Minor Version"),
        (26, 28, "Major Version"),
        (28, 30, "Byte Order"),
        (30, 32, "Sector Shift"),
        (32, 34, "Mini Sector Shift"),
        (40, 44, "Total Dir Sectors"),
        (44, 48, "Total FAT Sectors"),
        (48, 52, "First Dir Sector SID"),
        (56, 60, "Mini Stream Cutoff"),
        (60, 64, "First Mini FAT Sector"),
        (64, 68, "Total Mini FAT Sectors"),
        (68, 72, "First DIFAT Sector"),
        (72, 76, "Total DIFAT Sectors"),
    ];

    for (start, end, name) in &fields {
        let o = &orig_hdr[*start..*end];
        let s = &saved_hdr[*start..*end];
        if o != s {
            eprintln!(
                "  {} [{}..{}]: 원본={:?} 저장본={:?}",
                name, start, end, o, s
            );
        }
    }

    // DIFAT 배열 비교
    eprintln!("\n--- DIFAT ---");
    for i in 0..5 {
        let offset = 76 + i * 4;
        let o = u32::from_le_bytes([
            orig_hdr[offset],
            orig_hdr[offset + 1],
            orig_hdr[offset + 2],
            orig_hdr[offset + 3],
        ]);
        let s = u32::from_le_bytes([
            saved_hdr[offset],
            saved_hdr[offset + 1],
            saved_hdr[offset + 2],
            saved_hdr[offset + 3],
        ]);
        if o != 0xFFFFFFFF || s != 0xFFFFFFFF {
            eprintln!("  DIFAT[{}]: 원본=0x{:08X} 저장본=0x{:08X}", i, o, s);
        }
    }

    // 디렉토리 엔트리 비교 (cfb 크레이트로)
    eprintln!("\n=== 디렉토리 엔트리 비교 ===");
    let orig_cfb = cfb::CompoundFile::open(std::io::Cursor::new(&orig_data)).unwrap();
    let saved_cfb = cfb::CompoundFile::open(std::io::Cursor::new(&saved_data)).unwrap();

    fn walk_entries(cf: &cfb::CompoundFile<std::io::Cursor<&Vec<u8>>>, label: &str) {
        eprintln!("  [{}]", label);
        for entry in cf.walk() {
            eprintln!(
                "    {:?} path={} len={}",
                entry.name(),
                entry.path().display(),
                entry.len()
            );
        }
    }
    walk_entries(&orig_cfb, "원본");
    walk_entries(&saved_cfb, "저장본");

    // 원본의 Raw 디렉토리 엔트리 바이트 비교
    eprintln!("\n=== Raw 디렉토리 엔트리 비교 ===");
    // 원본 디렉토리: 첫 섹터부터
    let orig_first_dir =
        u32::from_le_bytes([orig_hdr[48], orig_hdr[49], orig_hdr[50], orig_hdr[51]]) as usize;
    let saved_first_dir =
        u32::from_le_bytes([saved_hdr[48], saved_hdr[49], saved_hdr[50], saved_hdr[51]]) as usize;
    eprintln!("  원본 첫 Dir 섹터: {}", orig_first_dir);
    eprintln!("  저장본 첫 Dir 섹터: {}", saved_first_dir);

    // 각 디렉토리 엔트리 상세 비교
    fn read_entry_name(entry: &[u8]) -> String {
        let name_size = u16::from_le_bytes([entry[64], entry[65]]) as usize;
        if name_size <= 2 {
            return "(empty)".to_string();
        }
        let char_count = (name_size / 2) - 1;
        let mut chars = Vec::new();
        for j in 0..char_count {
            let ch = u16::from_le_bytes([entry[j * 2], entry[j * 2 + 1]]);
            chars.push(ch);
        }
        String::from_utf16_lossy(&chars)
    }

    fn read_entry_at(data: &[u8], off: usize) -> Option<(String, u8)> {
        if off + 128 > data.len() {
            return None;
        }
        let e = &data[off..off + 128];
        let obj_type = e[66];
        if obj_type == 0 {
            return None;
        }
        let name_size = u16::from_le_bytes([e[64], e[65]]) as usize;
        if name_size <= 2 {
            return Some(("(empty)".to_string(), obj_type));
        }
        let char_count = ((name_size / 2) - 1).min(31);
        let mut chars = Vec::new();
        for j in 0..char_count {
            let pos = j * 2;
            if pos + 1 < 64 {
                let ch = u16::from_le_bytes([e[pos], e[pos + 1]]);
                chars.push(ch);
            }
        }
        Some((String::from_utf16_lossy(&chars), obj_type))
    }

    fn dump_entries(data: &[u8], label: &str) {
        eprintln!("  [{}] Dir entries:", label);
        // FAT 체인을 따라 디렉토리 섹터 수집
        let fat_sectors = u32::from_le_bytes([data[44], data[45], data[46], data[47]]) as usize;
        let first_dir = u32::from_le_bytes([data[48], data[49], data[50], data[51]]) as usize;
        // DIFAT에서 FAT 섹터 위치 읽기
        let mut fat = Vec::new();
        for fi in 0..fat_sectors {
            let difat_off = 76 + fi * 4;
            let fat_sid = u32::from_le_bytes([
                data[difat_off],
                data[difat_off + 1],
                data[difat_off + 2],
                data[difat_off + 3],
            ]) as usize;
            let fat_off = 512 + fat_sid * 512;
            for j in 0..128 {
                let entry_off = fat_off + j * 4;
                if entry_off + 4 <= data.len() {
                    let v = u32::from_le_bytes([
                        data[entry_off],
                        data[entry_off + 1],
                        data[entry_off + 2],
                        data[entry_off + 3],
                    ]);
                    fat.push(v);
                }
            }
        }
        // 디렉토리 섹터 체인 따라가기
        let mut dir_sectors = Vec::new();
        let mut cur = first_dir;
        while cur < fat.len() && fat[cur] != 0xFFFFFFFE && fat[cur] != 0xFFFFFFFF {
            dir_sectors.push(cur);
            cur = fat[cur] as usize;
        }
        if cur < fat.len() {
            dir_sectors.push(cur);
        }
        // 각 디렉토리 섹터의 엔트리 덤프
        let mut entry_idx = 0;
        for &sec in &dir_sectors {
            for slot in 0..4 {
                let off = 512 + sec * 512 + slot * 128;
                if off + 128 > data.len() {
                    continue;
                }
                let e = &data[off..off + 128];
                let obj_type = e[66];
                if obj_type == 0 {
                    entry_idx += 1;
                    continue;
                }
                let name = match read_entry_at(data, off) {
                    Some((n, _)) => n,
                    None => {
                        entry_idx += 1;
                        continue;
                    }
                };
                let color = e[67];
                let left = u32::from_le_bytes([e[68], e[69], e[70], e[71]]);
                let right = u32::from_le_bytes([e[72], e[73], e[74], e[75]]);
                let child = u32::from_le_bytes([e[76], e[77], e[78], e[79]]);
                let start = u32::from_le_bytes([e[116], e[117], e[118], e[119]]);
                let size = u32::from_le_bytes([e[120], e[121], e[122], e[123]]);
                let clsid = &e[80..96];
                let clsid_nonzero = clsid.iter().any(|&b| b != 0);
                eprintln!(
                    "    [{}] '{}' type={} color={} start={} size={} L/R/C={}/{}/{}{}",
                    entry_idx,
                    name,
                    obj_type,
                    color,
                    start,
                    size,
                    left,
                    right,
                    child,
                    if clsid_nonzero {
                        format!(" CLSID={:02X?}", clsid)
                    } else {
                        String::new()
                    }
                );
                let ctime = &e[100..108];
                let mtime = &e[108..116];
                let has_time = ctime.iter().any(|&b| b != 0) || mtime.iter().any(|&b| b != 0);
                if has_time {
                    eprintln!("         create={:02X?} mod={:02X?}", ctime, mtime);
                }
                entry_idx += 1;
            }
        }
    }

    dump_entries(&orig_data, "원본");
    dump_entries(&saved_data, "브라우저 저장본");

    // 네이티브 직렬화 결과 생성 및 비교
    let doc = crate::parser::parse_hwp(&orig_data).unwrap();
    let serialized = super::serialize_hwp(&doc).unwrap();
    eprintln!(
        "\n  네이티브 직렬화(mini_cfb) 크기: {} bytes",
        serialized.len()
    );
    dump_entries(&serialized, "네이티브 직렬화(mini_cfb)");

    // cfb 크레이트로 생성한 파일과 비교
    let cfb_bytes = write_hwp_with_cfb_crate(&orig_data);
    eprintln!("\n  cfb 크레이트 직렬화 크기: {} bytes", cfb_bytes.len());
    let _ = std::fs::write("output/roundtrip_cfb_crate.hwp", &cfb_bytes);
    let _ = std::fs::write("output/roundtrip_mini_cfb.hwp", &serialized);
    eprintln!("cfb 크레이트 결과: output/roundtrip_cfb_crate.hwp");
    eprintln!("mini_cfb 결과: output/roundtrip_mini_cfb.hwp");

    // cfb 크레이트 출력 재파싱 검증
    match crate::parser::parse_hwp(&cfb_bytes) {
        Ok(reparsed) => {
            eprintln!(
                "  cfb 크레이트 결과 재파싱 성공: {} 섹션, {} 문단",
                reparsed.sections.len(),
                reparsed
                    .sections
                    .iter()
                    .map(|s| s.paragraphs.len())
                    .sum::<usize>()
            );
        }
        Err(e) => eprintln!("  cfb 크레이트 결과 재파싱 실패: {}", e),
    }

    // raw_stream 없이 재직렬화 (편집 후 저장 시뮬레이션)
    let mut doc_no_raw = crate::parser::parse_hwp(&orig_data).unwrap();
    doc_no_raw.doc_info.raw_stream = None;
    for sec in &mut doc_no_raw.sections {
        sec.raw_stream = None;
    }
    let no_raw_bytes = super::serialize_hwp(&doc_no_raw).unwrap();
    let _ = std::fs::write("output/roundtrip_no_raw.hwp", &no_raw_bytes);
    eprintln!(
        "\n  raw_stream 없이 재직렬화 크기: {} bytes",
        no_raw_bytes.len()
    );
    dump_entries(&no_raw_bytes, "재직렬화(raw 없음)");

    // 재직렬화 결과 재파싱 검증
    match crate::parser::parse_hwp(&no_raw_bytes) {
        Ok(reparsed) => {
            eprintln!(
                "  재직렬화(raw 없음) 재파싱 성공: {} 섹션, {} 문단",
                reparsed.sections.len(),
                reparsed
                    .sections
                    .iter()
                    .map(|s| s.paragraphs.len())
                    .sum::<usize>()
            );
        }
        Err(e) => eprintln!("  재직렬화(raw 없음) 재파싱 실패: {}", e),
    }

    // 브라우저 저장본과 재직렬화(raw 없음) 비교
    eprintln!("\n=== 브라우저 저장본 vs 재직렬화(raw 없음) 비교 ===");
    let saved_decompressed_di = {
        let mut scfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        scfb.read_doc_info(true).unwrap()
    };
    let noraw_decompressed_di = {
        let mut ncfb = crate::parser::cfb_reader::CfbReader::open(&no_raw_bytes).unwrap();
        ncfb.read_doc_info(true).unwrap()
    };
    eprintln!(
        "  DocInfo: 저장본={}B  재직렬화={}B  동일={}",
        saved_decompressed_di.len(),
        noraw_decompressed_di.len(),
        saved_decompressed_di == noraw_decompressed_di
    );

    let saved_decompressed_bt = {
        let mut scfb = crate::parser::cfb_reader::CfbReader::open(&saved_data).unwrap();
        scfb.read_body_text_section(0, true, false).unwrap()
    };
    let noraw_decompressed_bt = {
        let mut ncfb = crate::parser::cfb_reader::CfbReader::open(&no_raw_bytes).unwrap();
        ncfb.read_body_text_section(0, true, false).unwrap()
    };
    eprintln!(
        "  BodyText: 저장본={}B  재직렬화={}B  동일={}",
        saved_decompressed_bt.len(),
        noraw_decompressed_bt.len(),
        saved_decompressed_bt == noraw_decompressed_bt
    );

    if saved_decompressed_bt != noraw_decompressed_bt {
        // 레코드별 비교
        let saved_recs = crate::parser::record::Record::read_all(&saved_decompressed_bt).unwrap();
        let noraw_recs = crate::parser::record::Record::read_all(&noraw_decompressed_bt).unwrap();
        eprintln!(
            "  레코드 수: 저장본={}  재직렬화={}",
            saved_recs.len(),
            noraw_recs.len()
        );
        let max = saved_recs.len().max(noraw_recs.len());
        let mut diff_count = 0;
        for i in 0..max {
            match (saved_recs.get(i), noraw_recs.get(i)) {
                (Some(s), Some(n)) => {
                    if s.tag_id != n.tag_id || s.level != n.level || s.data != n.data {
                        diff_count += 1;
                        if diff_count <= 10 {
                            let tag = crate::parser::tags::tag_name(s.tag_id);
                            eprintln!(
                                "  [{}] {} tag={}/{} level={}/{} size={}/{}",
                                i,
                                tag,
                                s.tag_id,
                                n.tag_id,
                                s.level,
                                n.level,
                                s.data.len(),
                                n.data.len()
                            );
                        }
                    }
                }
                (Some(s), None) => {
                    let tag = crate::parser::tags::tag_name(s.tag_id);
                    eprintln!("  [{}] 재직렬화에 없음: {} size={}", i, tag, s.data.len());
                }
                (None, Some(n)) => {
                    let tag = crate::parser::tags::tag_name(n.tag_id);
                    eprintln!("  [{}] 저장본에 없음: {} size={}", i, tag, n.data.len());
                }
                _ => {}
            }
        }
        eprintln!("  총 {} 레코드 차이", diff_count);
    }

    // 브라우저 편집 시나리오 시뮬레이션: DocInfo는 보존, Section만 재직렬화
    let mut doc_browser_sim = crate::parser::parse_hwp(&orig_data).unwrap();
    // DocInfo raw_stream은 보존 (편집해도 클리어하지 않음)
    // Section raw_stream만 클리어
    for sec in &mut doc_browser_sim.sections {
        sec.raw_stream = None;
    }
    let browser_sim_bytes = super::serialize_hwp(&doc_browser_sim).unwrap();
    let _ = std::fs::write("output/roundtrip_browser_sim.hwp", &browser_sim_bytes);
    eprintln!(
        "\n  브라우저 시뮬레이션 크기: {} bytes",
        browser_sim_bytes.len()
    );
    dump_entries(&browser_sim_bytes, "브라우저 시뮬레이션");
    match crate::parser::parse_hwp(&browser_sim_bytes) {
        Ok(reparsed) => {
            eprintln!(
                "  브라우저 시뮬레이션 재파싱 성공: {} 섹션, {} 문단",
                reparsed.sections.len(),
                reparsed
                    .sections
                    .iter()
                    .map(|s| s.paragraphs.len())
                    .sum::<usize>()
            );
        }
        Err(e) => eprintln!("  브라우저 시뮬레이션 재파싱 실패: {}", e),
    }
}

/// 원본 BodyText와 재직렬화 BodyText를 레코드 단위로 비교
#[test]
fn test_bodytext_reserialization_diff() {
    use crate::parser::record::Record;
    use std::path::Path;

    let path = Path::new("samples/hwp_table_test.hwp");
    if !path.exists() {
        eprintln!("hwp_table_test.hwp 없음 — 건너뜀");
        return;
    }

    let orig_data = std::fs::read(path).unwrap();
    let mut doc = crate::parser::parse_hwp(&orig_data).unwrap();

    // 원본 decompressed BodyText
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&orig_data).unwrap();
    let orig_bt = cfb
        .read_body_text_section(0, doc.header.compressed, false)
        .unwrap();

    // 재직렬화 BodyText (raw_stream = None으로 강제)
    doc.sections[0].raw_stream = None;
    let reser_bt = crate::serializer::body_text::serialize_section(&doc.sections[0]);

    eprintln!("=== 원본 vs 재직렬화 BodyText 비교 ===");
    eprintln!("  원본: {} bytes", orig_bt.len());
    eprintln!("  재직렬화: {} bytes", reser_bt.len());

    let orig_recs = Record::read_all(&orig_bt).unwrap();
    let reser_recs = Record::read_all(&reser_bt).unwrap();
    eprintln!("  원본 레코드 수: {}", orig_recs.len());
    eprintln!("  재직렬화 레코드 수: {}", reser_recs.len());

    let max = orig_recs.len().max(reser_recs.len());
    let mut diff_count = 0;
    let mut missing_count = 0;
    let mut extra_count = 0;
    for i in 0..max {
        match (orig_recs.get(i), reser_recs.get(i)) {
            (Some(o), Some(r)) => {
                let tag_match = o.tag_id == r.tag_id;
                let level_match = o.level == r.level;
                let data_match = o.data == r.data;
                if !tag_match || !level_match || !data_match {
                    diff_count += 1;
                    if diff_count <= 30 {
                        let tag = crate::parser::tags::tag_name(o.tag_id);
                        let rtag = crate::parser::tags::tag_name(r.tag_id);
                        eprintln!(
                            "  [{}] 원본: {} L{} {}B | 재직렬화: {} L{} {}B",
                            i,
                            tag,
                            o.level,
                            o.data.len(),
                            rtag,
                            r.level,
                            r.data.len()
                        );
                        if o.tag_id == r.tag_id && o.data.len() == r.data.len() {
                            // 같은 크기면 바이트 차이 표시
                            for j in 0..o.data.len().min(64) {
                                if o.data[j] != r.data[j] {
                                    eprintln!(
                                        "    offset {}: 0x{:02X} → 0x{:02X}",
                                        j, o.data[j], r.data[j]
                                    );
                                }
                            }
                        } else if o.tag_id == r.tag_id && o.data.len() != r.data.len() {
                            eprintln!(
                                "    크기 차이: {}B vs {}B ({}B)",
                                o.data.len(),
                                r.data.len(),
                                r.data.len() as i64 - o.data.len() as i64
                            );
                            // 앞부분 비교
                            let min_len = o.data.len().min(r.data.len()).min(32);
                            for j in 0..min_len {
                                if o.data[j] != r.data[j] {
                                    eprintln!(
                                        "    첫 차이 offset {}: 0x{:02X} → 0x{:02X}",
                                        j, o.data[j], r.data[j]
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            (Some(o), None) => {
                missing_count += 1;
                let tag = crate::parser::tags::tag_name(o.tag_id);
                if missing_count <= 20 {
                    eprintln!(
                        "  [{}] 재직렬화에 없음: {} L{} {}B",
                        i,
                        tag,
                        o.level,
                        o.data.len()
                    );
                }
            }
            (None, Some(r)) => {
                extra_count += 1;
                let tag = crate::parser::tags::tag_name(r.tag_id);
                if extra_count <= 20 {
                    eprintln!(
                        "  [{}] 원본에 없음: {} L{} {}B",
                        i,
                        tag,
                        r.level,
                        r.data.len()
                    );
                }
            }
            _ => {}
        }
    }
    eprintln!(
        "  차이: {} 레코드, 누락: {}, 추가: {}",
        diff_count, missing_count, extra_count
    );

    // DocInfo도 비교
    let orig_di = cfb.read_doc_info(doc.header.compressed).unwrap();
    let reser_di =
        crate::serializer::doc_info::serialize_doc_info(&doc.doc_info, &doc.doc_properties);
    let orig_di_recs = Record::read_all(&orig_di).unwrap();
    let reser_di_recs = Record::read_all(&reser_di).unwrap();
    eprintln!("\n=== DocInfo 비교 ===");
    eprintln!(
        "  원본: {} records, {} bytes",
        orig_di_recs.len(),
        orig_di.len()
    );
    eprintln!(
        "  재직렬화: {} records, {} bytes",
        reser_di_recs.len(),
        reser_di.len()
    );

    // DocInfo에서 누락된 태그 식별
    let max_di = orig_di_recs.len().max(reser_di_recs.len());
    let mut di_diff = 0;
    for i in 0..max_di {
        match (orig_di_recs.get(i), reser_di_recs.get(i)) {
            (Some(o), Some(r)) if o.tag_id != r.tag_id || o.data.len() != r.data.len() => {
                di_diff += 1;
                if di_diff <= 10 {
                    let otag = crate::parser::tags::tag_name(o.tag_id);
                    let rtag = crate::parser::tags::tag_name(r.tag_id);
                    eprintln!(
                        "  [{}] 원본: {} L{} {}B | 재직렬화: {} L{} {}B",
                        i,
                        otag,
                        o.level,
                        o.data.len(),
                        rtag,
                        r.level,
                        r.data.len()
                    );
                }
            }
            (Some(o), None) => {
                let tag = crate::parser::tags::tag_name(o.tag_id);
                eprintln!(
                    "  [{}] 재직렬화에 없음: {} L{} {}B",
                    i,
                    tag,
                    o.level,
                    o.data.len()
                );
            }
            (None, Some(r)) => {
                let tag = crate::parser::tags::tag_name(r.tag_id);
                eprintln!(
                    "  [{}] 원본에 없음: {} L{} {}B",
                    i,
                    tag,
                    r.level,
                    r.data.len()
                );
            }
            _ => {}
        }
    }
}

/// cfb 크레이트를 사용하여 원본 HWP의 모든 스트림을 새 CFB로 복사
fn write_hwp_with_cfb_crate(orig_data: &[u8]) -> Vec<u8> {
    use std::io::{Cursor, Read as IoRead, Write as IoWrite};

    let doc = crate::parser::parse_hwp(orig_data).unwrap();
    let compressed = doc.header.compressed;

    // 스트림 데이터 준비 (serialize_hwp과 동일 로직)
    let header_bytes = super::serialize_file_header(&doc.header);
    let doc_info_bytes = super::serialize_doc_info(&doc.doc_info, &doc.doc_properties);
    let doc_info_data = if compressed {
        super::compress_stream(&doc_info_bytes).unwrap()
    } else {
        doc_info_bytes
    };

    let mut section_data_list = Vec::new();
    for section in &doc.sections {
        let section_bytes = super::serialize_section(section);
        let section_data = if compressed {
            super::compress_stream(&section_bytes).unwrap()
        } else {
            section_bytes
        };
        section_data_list.push(section_data);
    }

    // cfb 크레이트로 CFB 생성
    let cursor = Cursor::new(Vec::new());
    let mut cfb = cfb::CompoundFile::create(cursor).unwrap();

    // /FileHeader
    {
        let mut stream = cfb.create_stream("/FileHeader").unwrap();
        stream.write_all(&header_bytes).unwrap();
    }

    // /DocInfo
    {
        let mut stream = cfb.create_stream("/DocInfo").unwrap();
        stream.write_all(&doc_info_data).unwrap();
    }

    // /BodyText/Section{N}
    cfb.create_storage("/BodyText").unwrap();
    for (i, data) in section_data_list.iter().enumerate() {
        let path = format!("/BodyText/Section{}", i);
        let mut stream = cfb.create_stream(&path).unwrap();
        stream.write_all(data).unwrap();
    }

    // /PrvImage, /PrvText
    if let Some(ref prv) = doc.preview {
        if let Some(ref image) = prv.image {
            let mut stream = cfb.create_stream("/PrvImage").unwrap();
            stream.write_all(&image.data).unwrap();
        }
        if let Some(ref text) = prv.text {
            let utf16: Vec<u16> = text.encode_utf16().collect();
            let mut bytes = Vec::with_capacity(utf16.len() * 2);
            for ch in &utf16 {
                bytes.extend_from_slice(&ch.to_le_bytes());
            }
            let mut stream = cfb.create_stream("/PrvText").unwrap();
            stream.write_all(&bytes).unwrap();
        }
    }

    // 추가 스트림 (Scripts, DocOptions, HwpSummaryInformation 등)
    for (path, data) in &doc.extra_streams {
        // 중간 스토리지 생성 필요
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.len() > 1 {
            let storage = format!("/{}", parts[0]);
            let _ = cfb.create_storage(&storage); // 이미 있으면 무시
        }
        let mut stream = cfb.create_stream(path).unwrap();
        stream.write_all(data).unwrap();
    }

    cfb.flush().unwrap();
    let cursor = cfb.into_inner();
    cursor.into_inner()
}

/// OLE Storage BinData 는 `[4-byte LE size][CFB 컨테이너]` 형식이다.
/// 파서(`load_bin_data_content`)가 내부 CFB 노출을 위해 size prefix 를 제거하므로,
/// 직렬화 시 다시 복원해야 한다. 미복원 시 한컴이 CFB 매직(D0CF11E0)을 OLE 개체
/// 크기(~3.75GB)로 오인하여 "메모리 부족" 오류로 문서를 열지 못한다.
#[test]
fn test_ole_storage_size_prefix_restored() {
    use crate::model::bin_data::{BinData, BinDataContent, BinDataType};

    // 가짜 OLE 내부 CFB: CFB 매직 + 임의 페이로드
    let mut ole_cfb = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    ole_cfb.extend_from_slice(&[0x42u8; 64]);

    let mut doc_info = DocInfo::default();
    doc_info.bin_data_list.push(BinData {
        data_type: BinDataType::Storage,
        storage_id: 1,
        extension: Some("OLE".to_string()),
        ..Default::default()
    });

    let doc = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0,
            compressed: false,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            ..Default::default()
        },
        doc_info,
        sections: vec![crate::model::document::Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph {
                line_segs: vec![LineSeg {
                    line_height: 400,
                    baseline_distance: 320,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            raw_stream: None,
        }],
        preview: None,
        bin_data_content: vec![BinDataContent {
            id: 1,
            data: ole_cfb.clone(),
            extension: "OLE".to_string(),
        }],
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    let bytes = serialize_hwp(&doc).unwrap();
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&bytes).unwrap();
    let stream = cfb.read_bin_data("BIN0001.OLE").unwrap();

    // 선두 4바이트 = OLE CFB 길이의 LE size prefix
    assert!(
        stream.len() >= 12,
        "OLE 스트림이 prefix + CFB 매직 길이 이상이어야 한다"
    );
    let prefix = u32::from_le_bytes([stream[0], stream[1], stream[2], stream[3]]);
    assert_eq!(
        prefix as usize,
        ole_cfb.len(),
        "4-byte size prefix 가 OLE CFB 길이와 일치해야 한다"
    );
    // prefix 직후가 내부 CFB 매직
    assert_eq!(
        &stream[4..12],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        "size prefix 다음은 CFB 매직이어야 한다"
    );
    // prefix 를 제외한 본문이 원본 OLE CFB 와 동일
    assert_eq!(&stream[4..], &ole_cfb[..], "OLE CFB 본문이 보존되어야 한다");
}

/// 압축 문서에서는 OLE Storage 도 `[size][CFB]` payload 를 만든 뒤 BinData 압축 정책에 따라
/// raw deflate 로 저장해야 한다. 한컴 저장본의 chart OLE Storage 가 이 형태를 사용한다.
#[test]
fn test_compressed_ole_storage_payload_is_deflated() {
    use crate::model::bin_data::{BinData, BinDataContent, BinDataType};

    let mut ole_cfb = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    ole_cfb.extend_from_slice(&[0x42u8; 64]);

    let mut doc_info = DocInfo::default();
    doc_info.bin_data_list.push(BinData {
        data_type: BinDataType::Storage,
        storage_id: 1,
        extension: Some("OLE".to_string()),
        ..Default::default()
    });

    let doc = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0x01,
            compressed: true,
            encrypted: false,
            distribution: false,
            raw_data: None,
        },
        doc_properties: DocProperties {
            section_count: 1,
            page_start_num: 1,
            ..Default::default()
        },
        doc_info,
        sections: vec![crate::model::document::Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph {
                line_segs: vec![LineSeg {
                    line_height: 400,
                    baseline_distance: 320,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            raw_stream: None,
        }],
        preview: None,
        bin_data_content: vec![BinDataContent {
            id: 1,
            data: ole_cfb.clone(),
            extension: "OLE".to_string(),
        }],
        extra_streams: Vec::new(),
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    let bytes = serialize_hwp(&doc).unwrap();
    let mut cfb = crate::parser::cfb_reader::CfbReader::open(&bytes).unwrap();
    let stream = cfb.read_bin_data("BIN0001.OLE").unwrap();
    assert!(
        !stream.starts_with(&(ole_cfb.len() as u32).to_le_bytes()),
        "compressed OLE Storage stream should not expose the size prefix before decompression"
    );

    let payload = decompress_stream(&stream).expect("OLE Storage stream must be deflated");
    let prefix = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(prefix as usize, ole_cfb.len());
    assert_eq!(
        &payload[4..12],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
    );
    assert_eq!(&payload[4..], &ole_cfb[..]);
}
