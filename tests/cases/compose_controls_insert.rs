//! 관계형 회귀: 덧말·겹침 컨트롤 생성과 원문·참조·저장 보존. 실물 절대 배치 핀은 없다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! hancom-mac-shapes.hwp (8616-hancom-mac-shapes-hwp-2022): 전 1쪽, Native 최저 99.2128%, fresh WASM 최저 99.22558%.
//! hancom-mac-shapes.hwpx (8616-hancom-mac-shapes-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! hancom-mac-dutmal.hwp (8616-hancom-mac-dutmal-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! hancom-mac-dutmal.hwpx (8616-hancom-mac-dutmal-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 덧말 넣기·글자 겹치기(한컴 Mac 12.30 입력 메뉴)로 만든 컨트롤을 그리고 저장한다.
//!
//! 한컴 12.30 저장본 기준: 10pt 줄에 위 덧말을 넣으면 줄 높이 1500·기준선 1350·간격 900이고,
//! HWP 원문 배열은 겹칠 글자 앞에 테두리 글자를 둔다. HWPX `composeText`는 친 글 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::{ComposeHost, DocumentCore};
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn document() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가나").unwrap();
    doc
}

fn page_text(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

fn saved(doc: &HwpDocument) -> [(&'static str, DocumentCore); 2] {
    [
        (
            "HWP",
            DocumentCore::from_bytes(&doc.export_hwp_with_adapter_snapshot().unwrap()).unwrap(),
        ),
        (
            "HWPX",
            DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap(),
        ),
    ]
}

#[test]
fn ruby_draws_both_texts_raises_the_line_and_survives_saving() {
    let mut doc = document();
    let result = doc
        .insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    assert!(result.contains(r#""controlIdx":"#), "{result}");
    let line = doc.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert_eq!(
        (line.line_height, line.baseline_distance, line.line_spacing),
        (1500, 1350, 900)
    );
    let text = page_text(&doc);
    for glyph in ['가', '漢', '字', '한', '자', '나'] {
        assert!(text.contains(glyph), "{glyph} 없음: {text}");
    }
    for (format, core) in saved(&doc) {
        let ruby = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Ruby(ruby) => Some(ruby.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{format}: 덧말 없음"));
        assert_eq!(
            (
                ruby.main_text.as_str(),
                ruby.ruby_text.as_str(),
                ruby.pos_type,
                ruby.align
            ),
            ("漢字", "한자", 0, 3),
            "{format}"
        );
        assert_eq!(core.document().sections[0].paragraphs[0].text, "가나");
    }
}

#[test]
fn ruby_edit_moves_it_below_with_a_style_and_reads_back() {
    let mut doc = document();
    doc.insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Ruby(_)))
        .unwrap();
    doc.set_compose_control_native(
        0,
        0,
        index,
        r#"{"mainText":"大韓","rubyText":"대한","below":true,"styleId":1}"#,
    )
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&doc.get_compose_control_native(0, 0, index).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"kind":"ruby","mainText":"大韓","rubyText":"대한","below":true,"styleId":1})
    );
    // 아래 덧말은 기준선을 두고 줄을 늘인다.
    let line = doc.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert_eq!((line.line_height, line.baseline_distance), (1408, 850));
}

#[test]
fn char_overlap_insert_writes_hancom_wire_chars_and_plain_hwpx_text() {
    let mut doc = document();
    doc.insert_char_overlap_native(0, 0, 1, r#"{"text":"12","shape":5}"#)
        .unwrap();
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::CharOverlap(_)))
        .unwrap();
    assert!(page_text(&doc).contains('△'));
    for (format, core) in saved(&doc) {
        let Control::CharOverlap(overlap) =
            &core.document().sections[0].paragraphs[0].controls[index]
        else {
            panic!("{format}: 겹침 없음");
        };
        let expected: &[char] = if format == "HWP" {
            &['△', '1', '2']
        } else {
            &['1', '2']
        };
        assert_eq!(overlap.chars, expected, "{format}");
        assert_eq!((overlap.border_type, overlap.inner_char_size), (5, -3));
        let value: serde_json::Value =
            serde_json::from_str(&core.get_compose_control_native(0, 0, index).unwrap()).unwrap();
        assert_eq!(value["text"], "12", "{format}");
    }
    // HWP 저장본을 다시 HWPX로 내도 표지 글자는 빠진다.
    let hwp = DocumentCore::from_bytes(&doc.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&hwp.export_hwpx_native().unwrap()).unwrap();
    let Control::CharOverlap(overlap) = &hwpx.document().sections[0].paragraphs[0].controls[index]
    else {
        panic!("HWP→HWPX: 겹침 없음");
    };
    assert_eq!(overlap.chars, ['1', '2']);
}

#[test]
fn deleting_keeps_the_ruby_main_text_only_when_asked() {
    let mut doc = document();
    doc.insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    doc.insert_char_overlap_native(0, 0, 2, r#"{"text":"가","shape":1}"#)
        .unwrap();
    let index = |doc: &HwpDocument, ruby: bool| {
        doc.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|control| match control {
                Control::Ruby(_) => ruby,
                Control::CharOverlap(_) => !ruby,
                _ => false,
            })
            .unwrap()
    };
    let overlap = index(&doc, false);
    doc.delete_compose_control_native(0, 0, overlap, false)
        .unwrap();
    assert_eq!(doc.document().sections[0].paragraphs[0].text, "가나");
    let ruby = index(&doc, true);
    doc.delete_compose_control_native(0, 0, ruby, true).unwrap();
    let para = &doc.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "가漢字나");
    assert!(!para
        .controls
        .iter()
        .any(|control| matches!(control, Control::Ruby(_) | Control::CharOverlap(_))));
    for (format, core) in saved(&doc) {
        assert_eq!(
            core.document().sections[0].paragraphs[0].text,
            "가漢字나",
            "{format}"
        );
    }
}

/// 모든 쪽의 그린 글. 미주는 문서 끝 쪽에 놓인다.
fn all_pages_text(core: &DocumentCore) -> String {
    (0..core.page_count())
        .map(|page| {
            let svg = core.render_page_svg_native(page).unwrap();
            svg.split("<text ")
                .skip(1)
                .map(|run| &run[run.find('>').unwrap() + 1..run.find("</text>").unwrap()])
                .collect::<String>()
        })
        .collect()
}

/// JSON 결과의 정수 값.
fn number(json: &str, key: &str) -> usize {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    value[key]
        .as_u64()
        .unwrap_or_else(|| panic!("{key} 없음: {json}")) as usize
}

/// 덧말·겹침의 컨트롤 번호들. 값을 읽을 수 있는 컨트롤만 센다.
fn compose_values(core: &DocumentCore, host: ComposeHost) -> Vec<(usize, serde_json::Value)> {
    (0..8)
        .filter_map(|index| {
            let value = core.get_compose_control_at(host, index).ok()?;
            Some((index, serde_json::from_str(&value).unwrap()))
        })
        .collect()
}

#[test]
fn compose_controls_work_in_cells_text_boxes_and_notes() {
    let mut doc = document();
    doc.split_paragraph_native(0, 0, 2, None).unwrap();
    let table = doc.create_table_native(0, 1, 0, 1, 2).unwrap();
    let (tp, tc) = (number(&table, "paraIdx"), number(&table, "controlIdx"));
    let cell = [(tc, 0, 0)];
    doc.insert_text_in_cell_native(0, tp, tc, 0, 0, 0, "셀글")
        .unwrap();
    let nested = doc
        .create_table_in_cell_by_path_native(0, tp, &[(tc, 1, 0)], 0, 1, 1, None, None, true)
        .unwrap();
    let nested: serde_json::Value = serde_json::from_str(&nested).unwrap();
    let nested_path: Vec<(usize, usize, usize)> = nested["cellPath"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let at = |key: &str| entry[key].as_u64().unwrap() as usize;
            (at("controlIndex"), at("cellIndex"), at("cellParaIndex"))
        })
        .collect();
    doc.insert_text_in_cell_by_path(0, tp, &nested_path, 0, "중첩글")
        .unwrap();
    let shape = doc
        .create_shape_control_native(
            0,
            0,
            0,
            9000,
            6000,
            0,
            20000,
            false,
            "InFrontOfText",
            "textbox",
            false,
            false,
            &[],
        )
        .unwrap();
    let (sp, sc) = (number(&shape, "paraIdx"), number(&shape, "controlIdx"));
    let text_box = [(sc, 0, 0)];
    doc.insert_text_in_cell_by_path(0, sp, &text_box, 0, "상자글")
        .unwrap();
    let footnote = doc.insert_footnote_native(0, 0, 1).unwrap();
    let (fp, fc) = (
        number(&footnote, "paraIdx"),
        number(&footnote, "controlIdx"),
    );
    doc.insert_text_in_footnote_native(0, fp, fc, 0, 0, "각주글")
        .unwrap();
    let endnote = doc.insert_endnote_native(0, 0, 2).unwrap();
    let (ep, ec) = (number(&endnote, "paraIdx"), number(&endnote, "controlIdx"));
    doc.insert_text_in_footnote_native(0, ep, ec, 0, 0, "미주글")
        .unwrap();
    let hosts = [
        (
            "셀",
            ComposeHost::Cell {
                sec: 0,
                parent_para: tp,
                path: &cell,
            },
        ),
        (
            "중첩 셀",
            ComposeHost::Cell {
                sec: 0,
                parent_para: tp,
                path: &nested_path,
            },
        ),
        (
            "글상자",
            ComposeHost::Cell {
                sec: 0,
                parent_para: sp,
                path: &text_box,
            },
        ),
        (
            "각주",
            ComposeHost::Note {
                sec: 0,
                para: fp,
                control: fc,
                note_para: 0,
            },
        ),
        (
            "미주",
            ComposeHost::Note {
                sec: 0,
                para: ep,
                control: ec,
                note_para: 0,
            },
        ),
    ];
    for (name, host) in hosts {
        doc.insert_ruby_at(host, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        doc.insert_char_overlap_at(host, 2, r#"{"text":"12","shape":5}"#)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let values = compose_values(&doc, host);
        assert_eq!(values.len(), 2, "{name}: {values:?}");
        let (ruby, _) = values[0];
        assert_eq!(values[0].1["kind"], "ruby", "{name}");
        assert_eq!(values[1].1["kind"], "overlap", "{name}");
        doc.set_compose_control_at(
            host,
            ruby,
            r#"{"mainText":"大韓","rubyText":"대한","below":true}"#,
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(compose_values(&doc, host)[0].1["below"], true, "{name}");
    }
    // 본말 뒤에 덧말을 그리고 겹침은 테두리 글자와 친 글이다.
    let text = all_pages_text(&doc);
    for drawn in [
        "셀大韓대한글△12",
        "중大韓대한첩△12",
        "상大韓대한자△12",
        "각大韓대한주△12",
        "미大韓대한주△12",
    ] {
        assert!(text.contains(drawn), "{drawn}: {text}");
    }
    for (format, core) in saved(&doc) {
        for (name, host) in hosts {
            let values = compose_values(&core, host);
            assert_eq!(
                values
                    .iter()
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<_>>(),
                [
                    serde_json::json!({"kind":"ruby","mainText":"大韓","rubyText":"대한","below":true,"styleId":null}),
                    serde_json::json!({"kind":"overlap","text":"12","shape":5,"innerSize":-3,"overlap":false}),
                ],
                "{format} {name}"
            );
        }
    }
    // [덧말 지움]은 본말을 그 자리 글로 남기고, 겹침 지우기는 글을 남기지 않는다.
    for (name, host) in hosts {
        let values = compose_values(&doc, host);
        doc.delete_compose_control_at(host, values[1].0, false)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        doc.delete_compose_control_at(host, values[0].0, true)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(compose_values(&doc, host).is_empty(), "{name}");
    }
    let text = all_pages_text(&doc);
    assert!(!text.contains('△'), "{text}");
    for expected in [
        "셀大韓글",
        "중大韓첩글",
        "상大韓자글",
        "각大韓주글",
        "미大韓주글",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

#[test]
fn ruby_only_paragraph_raises_its_line_in_the_body_cells_and_notes() {
    // 한컴 12.30: 셀 글 전체를 덧말로 바꿔도 줄이 덧말 높이만큼 커진다.
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.split_paragraph_native(0, 0, 0, None).unwrap();
    let table = doc.create_table_native(0, 1, 0, 2, 1).unwrap();
    let (tp, tc) = (number(&table, "paraIdx"), number(&table, "controlIdx"));
    let cell = [(tc, 0, 0)];
    let footnote = doc.insert_footnote_native(0, 0, 0).unwrap();
    let (fp, fc) = (
        number(&footnote, "paraIdx"),
        number(&footnote, "controlIdx"),
    );
    let top = r#"{"mainText":"cell","rubyText":"top"}"#;
    let below = r#"{"mainText":"cell","rubyText":"top","below":true}"#;
    doc.insert_ruby_at(ComposeHost::Body { sec: 0, para: 2 }, 0, top)
        .unwrap();
    doc.insert_ruby_at(
        ComposeHost::Cell {
            sec: 0,
            parent_para: tp,
            path: &cell,
        },
        0,
        top,
    )
    .unwrap();
    // 각주 첫 문단은 번호 뒤에 넣는다.
    let note = ComposeHost::Note {
        sec: 0,
        para: fp,
        control: fc,
        note_para: 0,
    };
    let note_text = doc.document().sections[0].paragraphs[fp].controls[fc].clone();
    let Control::Footnote(note_model) = note_text else {
        panic!("각주 없음");
    };
    let note_offset = note_model.paragraphs[0].text.chars().count();
    doc.insert_ruby_at(note, note_offset, below).unwrap();
    let rows = |core: &DocumentCore| {
        let section = &core.document().sections[0];
        let body = section.paragraphs[2].line_segs[0].clone();
        let Control::Table(table) = &section.paragraphs[tp].controls[tc] else {
            panic!("표 없음");
        };
        let cell = table.cells[0].paragraphs[0].line_segs[0].clone();
        let Control::Footnote(note) = &section.paragraphs[fp].controls[fc] else {
            panic!("각주 없음");
        };
        let note = note.paragraphs[0].line_segs[0].clone();
        [body, cell, note].map(|line| (line.line_height, line.baseline_distance, line.line_spacing))
    };
    // 각주 문단은 각주 모양의 줄 간격(130%)을 쓴다.
    let expected = [(1500, 1350, 900), (1500, 1350, 900), (1408, 850, 424)];
    assert_eq!(rows(&doc), expected);
    for (format, bytes) in [
        ("HWP", doc.export_hwp_with_adapter_snapshot().unwrap()),
        ("HWPX", doc.export_hwpx_native().unwrap()),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        // HWPX는 글자 없는 본문 문단의 줄을 쓰지 않아 열 때 다시 짠다. 셀 행 높이로 본다.
        if format == "HWP" {
            assert_eq!(rows(&reopened), expected, "{format}");
        }
    }
}

/// 도움말의 공백 제외·아홉 글자 제한을 공개 입력 경로에서도 지킨다.
#[test]
fn overlap_input_drops_spaces_and_keeps_hancom_limits() {
    let mut doc = document();
    doc.insert_char_overlap_native(0, 0, 1, r#"{"text":"1 2","shape":1,"innerSize":-3}"#)
        .unwrap();
    let overlap = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::CharOverlap(overlap) => Some(overlap),
            _ => None,
        })
        .unwrap();
    assert_eq!(overlap.chars, vec!['1', '2']);
    assert_eq!(overlap.char_shape_ids, vec![0, 0, 0]);
    assert!(!overlap.hwp_wire_chars);
    for input in [
        r#"{"text":"   ","shape":1,"innerSize":-3}"#,
        r#"{"text":"가나다라마바사아자차","shape":1,"innerSize":-3}"#,
        r#"{"text":"가","shape":14,"innerSize":-3}"#,
        r#"{"text":"가","shape":1,"innerSize":3}"#,
    ] {
        assert!(
            doc.insert_char_overlap_native(0, 0, 1, input).is_err(),
            "{input}"
        );
    }
}

/// 덧말의 스타일 선택과 해제는 공개 편집 경로에서도 option 비트를 함께 바꾼다.
#[test]
fn ruby_style_sets_option_bit_and_clears_it() {
    let mut doc = document();
    doc.insert_ruby_native(
        0,
        0,
        1,
        r#"{"mainText":"漢字","rubyText":"한자","below":true,"styleId":17}"#,
    )
    .unwrap();
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Ruby(_)))
        .unwrap();
    let ruby = |doc: &HwpDocument| match &doc.document().sections[0].paragraphs[0].controls[index] {
        Control::Ruby(ruby) => ruby.clone(),
        _ => panic!("덧말 소속 유지"),
    };
    let styled = ruby(&doc);
    assert_eq!(
        (
            styled.pos_type,
            styled.option,
            styled.style_id_ref,
            styled.align
        ),
        (1, 4, 17, 3)
    );
    doc.set_compose_control_native(0, 0, index, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    let plain = ruby(&doc);
    assert_eq!(
        (plain.pos_type, plain.option, plain.style_id_ref),
        (0, 0, 0)
    );
    assert!(doc
        .insert_ruby_native(0, 0, 1, r#"{"mainText":" ","rubyText":"x"}"#)
        .is_err());
}
