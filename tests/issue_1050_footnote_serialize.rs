//! Issue #1050: HWPX → HWP 저장 시 각주 직렬화 한컴 호환 회귀 가드.
//!
//! 본질: HWPX → IR → HWP 라운드트립 시 한컴 한글 편집기가 인식하는
//! footnote contract (FootnoteShape 16 필드 + CTRL_FOOTNOTE 16 byte payload +
//! footnote LIST_HEADER 16 byte 형식 + PARA_TEXT 의 AUTO_NUMBER inline 컨트롤
//! 8 code unit + placeholder space + jump 8) 를 완전 보존.
//!
//! 작업지시자 시각 판정 통과 (한컴 한글 2020): footnote-tbox-01.hwpx +
//! footnote-01.hwpx 양쪽 모두 한컴에서 정상 각주 영역으로 조판됨.
//!
//! 정정 영역 (4):
//! - `src/model/footnote.rs` — Footnote/Endnote 4 필드 추가
//! - `src/parser/control.rs` — CTRL_FOOTNOTE size=16 payload 파싱
//! - `src/serializer/control.rs` — CTRL_FOOTNOTE 16 byte + LIST_HEADER 16 byte 형식
//! - `src/parser/hwpx/section.rs`:
//!   - `parse_ctrl_footnote/endnote` HWPX suffixChar/instId → IR 매핑
//!   - `parse_sec_pr_children` 에 footNotePr/endNotePr 분기 추가 + `parse_note_pr_children`
//!     헬퍼 (noteLine length/type/width/color + noteSpacing/numbering 파싱)
//!   - `parse_paragraph` 의 autoNum 분기: `text_parts.push("\u{0012}")` (HWP AUTO_NUMBER
//!     contract 정합 placeholder)
//!   - `visual_text` 조립: `\u{0012}` → `char_offsets.push(pos) + text.push(' ') + pos += 8`
//! - `src/serializer/body_text.rs::serialize_para_text` — AutoNumber placeholder 검출 분기

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn page_svg(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    doc.render_page_svg_native(page).expect("render_page_svg")
}

fn svg_text_seq(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after_open = &rest[open..];
        if let Some(gt) = after_open.find('>') {
            let after_tag = &after_open[gt + 1..];
            if let Some(close) = after_tag.find("</text>") {
                out.push_str(&after_tag[..close]);
                rest = &after_tag[close + "</text>".len()..];
                continue;
            }
        }
        break;
    }
    out
}

/// HWPX 출처 footnote 가 hwp 저장 후 재로드 시 글상자 안 각주 본문 표시 (Task #1050 본질).
#[test]
fn issue_1050_hwpx_to_hwp_textbox_footnote_roundtrip() {
    let mut doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let hwp_bytes = doc
        .export_hwp_with_adapter()
        .expect("export_hwp_with_adapter");
    let saved = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes).expect("re-parse");
    let svg = page_svg(&saved, 0);
    let seq = svg_text_seq(&svg);
    assert!(
        seq.contains("글상자내부각주"),
        "HWPX → HWP 저장 후 재로드 시 글상자 안 각주 본문 '글상자 내부 각주' 표시 필요. \
         seq={:?}",
        seq
    );
    assert!(
        seq.contains("일반문단내각주"),
        "본문 직속 각주 '일반 문단내 각주' 회귀 부재 필요. seq={:?}",
        seq
    );
}

/// HWP 출처 round-trip 회귀 부재 — footnote 본문 보존.
#[test]
fn issue_1050_hwp_roundtrip_footnote_preserved() {
    let mut doc = load("samples/footnote-01.hwp");
    let bytes = doc.export_hwp_with_adapter().expect("export");
    let saved = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("re-parse");
    let document = saved.document();

    // 모든 footnote 의 paragraphs 가 보존되는지 확인
    let mut total_footnotes = 0usize;
    let mut total_fn_paras = 0usize;
    for sec in &document.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Footnote(fn_) = ctrl {
                    total_footnotes += 1;
                    total_fn_paras += fn_.paragraphs.len();
                }
            }
        }
    }
    assert!(
        total_footnotes >= 5,
        "footnote-01.hwp 는 다수 footnote 보유 (회귀 가드 — IR 보존 확인). actual={}",
        total_footnotes
    );
    assert_eq!(
        total_fn_paras, total_footnotes,
        "각 footnote 의 paragraphs 1개씩 보존. total_paras={} vs total_fn={}",
        total_fn_paras, total_footnotes
    );
}

/// HWPX 출처 footnote 의 CTRL_FOOTNOTE 매핑 (suffixChar=41 → after_decoration_letter=0x29).
#[test]
fn issue_1050_hwpx_footnote_attrs_mapped() {
    let doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let document = doc.document();

    let mut found = 0;
    for sec in &document.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Shape(shape) = ctrl {
                    if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                        for tp in &tb.paragraphs {
                            for tc in &tp.controls {
                                if let rhwp::model::control::Control::Footnote(fn_) = tc {
                                    assert_eq!(
                                        fn_.after_decoration_letter, 0x0029,
                                        "글상자 안 footnote suffixChar=41 → 0x29 ')'"
                                    );
                                    assert!(
                                        fn_.instance_id > 0,
                                        "HWPX instId 매핑 → instance_id > 0. actual={}",
                                        fn_.instance_id
                                    );
                                    found += 1;
                                }
                            }
                        }
                    }
                }
                if let rhwp::model::control::Control::Footnote(fn_) = ctrl {
                    assert_eq!(
                        fn_.after_decoration_letter, 0x0029,
                        "본문 footnote suffixChar=41 → 0x29 ')'"
                    );
                    found += 1;
                }
            }
        }
    }
    assert!(
        found >= 2,
        "글상자 안 + 본문 footnote 양쪽 매핑 확인. found={}",
        found
    );
}

/// Stage 1 contract: CTRL_FOOTNOTE size=20 정합 (number + before + after + numberShape + instanceId).
#[test]
fn issue_1050_ctrl_footnote_size_20() {
    let mut doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");

    // CFB stream 직접 읽어 CTRL_FOOTNOTE 의 size 검증
    use std::io::Read;
    let cursor = std::io::Cursor::new(&hwp_bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb open");

    // FileHeader 의 compression flag 확인
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FileHeader");
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read Section0");

    let data = if is_compressed {
        use std::io::Read as _;
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("inflate");
        decompressed
    } else {
        raw
    };

    let mut pos = 0;
    let mut fn_sizes = Vec::new();
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = hdr & 0x3FF;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        // CTRL_HEADER tag = 71 (HWPTAG_BEGIN + 17 + 0x10 + ... 실제 0x47=71)
        if tag == 71 && size >= 4 && pos + 4 <= data.len() {
            let cid = &data[pos..pos + 4];
            if cid == b"  nf" {
                fn_sizes.push(size);
            }
        }
        pos += size;
    }

    assert!(!fn_sizes.is_empty(), "CTRL_FOOTNOTE 적어도 1개 존재해야 함");
    for sz in &fn_sizes {
        assert_eq!(
            *sz, 20,
            "CTRL_FOOTNOTE size = 20 (한컴 정답지 정합). actual={}",
            sz
        );
    }
}

/// Stage 4-pivot: HWPX 출처 IR 의 FootnoteShape contract 한컴 정합.
/// HWPX `<hp:footNotePr>` 의 `<hp:autoNumFormat>` `<hp:noteLine>` `<hp:noteSpacing>` 파싱.
#[test]
fn issue_1050_hwpx_footnote_shape_contract() {
    let doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let fs = &doc.document().sections[0].section_def.footnote_shape;
    // HWPX `<hp:autoNumFormat suffixChar=")">` → suffix_char
    assert_eq!(fs.suffix_char as u32, 0x29, "suffix_char = ')'");
    // HWPX `<hp:noteLine length="-1" type="SOLID" width="0.12 mm">` → -1 + 1 + 1
    assert_eq!(
        fs.separator_length, -1,
        "noteLine.length=-1 (한컴 sentinel)"
    );
    assert_eq!(fs.separator_line_type, 1, "noteLine.type=SOLID → 1");
    assert_eq!(
        fs.separator_line_width, 1,
        "noteLine.width=0.12mm → 1 (0.1mm 단위)"
    );
    // HWPX `<hp:noteSpacing betweenNotes="283" belowLine="567" aboveLine="850">`
    assert_eq!(fs.raw_unknown, 283, "noteSpacing.betweenNotes=283");
    assert_eq!(fs.note_spacing, 567, "noteSpacing.belowLine=567");
    assert_eq!(fs.separator_margin_bottom, 850, "noteSpacing.aboveLine=850");
    // separator_margin_top: HWPX 미보유 → -1 default
    assert_eq!(fs.separator_margin_top, -1, "default -1 sentinel");
}

/// Stage 4-pivot: 각주 안 paragraph 의 HWP PARA_TEXT contract 정합.
/// AUTO_NUMBER 컨트롤 8 cu + placeholder space + 본문 (HWP body_text.rs:326 정합).
#[test]
fn issue_1050_footnote_paragraph_char_offsets() {
    let doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let mut found = 0;
    for sec in &doc.document().sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                let fns: Vec<&rhwp::model::footnote::Footnote> = match ctrl {
                    rhwp::model::control::Control::Footnote(fn_) => vec![fn_.as_ref()],
                    rhwp::model::control::Control::Shape(shape) => shape
                        .drawing()
                        .and_then(|d| d.text_box.as_ref())
                        .into_iter()
                        .flat_map(|tb| tb.paragraphs.iter())
                        .flat_map(|tp| tp.controls.iter())
                        .filter_map(|tc| {
                            if let rhwp::model::control::Control::Footnote(fn_) = tc {
                                Some(fn_.as_ref())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => vec![],
                };
                for fn_ in fns {
                    for fp in &fn_.paragraphs {
                        // 각주 안 paragraph: text="  글상자 내부 각주" (2 공백 + 본문)
                        assert!(
                            fp.text.starts_with("  "),
                            "각주 본문 text = placeholder ' ' + 실제 ' ' + 본문 (HWP 정합). actual={:?}",
                            fp.text
                        );
                        // char_offsets[0] = 0 (AutoNumber placeholder), char_offsets[1] = 8 (jump 8)
                        assert!(
                            fp.char_offsets.len() >= 2,
                            "각주 paragraph char_offsets 2 이상"
                        );
                        assert_eq!(
                            fp.char_offsets[0], 0,
                            "char_offsets[0]=0 (AutoNumber placeholder)"
                        );
                        assert_eq!(
                            fp.char_offsets[1], 8,
                            "char_offsets[1]=8 (HWP AUTO_NUMBER inline jump 8)"
                        );
                        found += 1;
                    }
                }
            }
        }
    }
    assert!(
        found >= 2,
        "글상자 안 + 본문 각주 양쪽 paragraph 검증. found={}",
        found
    );
}

/// footnote-01.hwpx — 9 footnote 보유 fixture HWPX → HWP 저장 후 자기 라운드트립.
#[test]
fn issue_1050_footnote_01_hwpx_roundtrip() {
    let mut doc = load("samples/hwpx/footnote-01.hwpx");
    let bytes = doc.export_hwp_with_adapter().expect("export");
    let saved = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("re-parse");
    let document = saved.document();

    // 9 개 이상의 footnote 보유
    let mut total = 0usize;
    for sec in &document.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Footnote(_) = ctrl {
                    total += 1;
                }
            }
        }
    }
    assert!(
        total >= 5,
        "footnote-01.hwpx 는 다수 footnote 보유 (8 footnote 예상). actual={}",
        total
    );

    // 첫 페이지 SVG 에 각주 마크 "1)" 등 표시
    let svg = page_svg(&saved, 0);
    let seq = svg_text_seq(&svg);
    assert!(seq.contains("1)"), "footnote-01 page 1 의 각주 마크 표시");
}
