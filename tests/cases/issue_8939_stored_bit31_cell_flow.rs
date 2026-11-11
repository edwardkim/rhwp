//! [#8939] 관계형·고정형 회귀 — 한/글이 저장한 LineSeg tag bit 31 은 저장 줄이고, 중첩 표를 품은
//! 글자처럼 취급 표의 칸은 저장 사다리가 칸 선언 높이에 담기면 그 높이로 잰다.
//!
//! 주석 갱신: 2026-11-11 23:10 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8939/attach_form_reset.hwp` — 서울시교육청 공개 문서
//!   「2018학년도 후기고등학교 입학전 전학 시행 계획(안)」 12·13쪽(붙임 3·4 신청서)만 남기고
//!   한/글 2022(12.0.0.4547)로 다시 저장한 2쪽 문서. 한/글은 문단 5~8 의 첫 저장 줄 tag 를
//!   `0x80060000`(bit 31)으로 적고, 문단 6 은 저장 vpos 0 으로 2쪽 맨 위에서 시작한다.
//! - 독립 기준: `pdf/issue8939/attach_form_reset-2020.pdf`(같은 저장본의 한/글 2022 Print, 2쪽).
//! - 수정 전 rhwp(devel `e118de8aba`)는 bit 31 을 자기 합성 줄 표지로 읽어 문단 6~8 을 1쪽에
//!   겹쳐 그리고, 1쪽 신청서 표의 문단 16개 칸(행 11)을 재조판 줄 합(336.9px)으로 키워 다른 행을
//!   선언보다 1~2px 줄였다(한/글은 선언 321.1px).
//! - rhwp 가 저장한 HWP 는 `/RhwpLineSegTags` 표지 스트림(구역 레코드 SHA-256)이 있을 때만
//!   bit 31 을 합성 줄 표지로 되읽는다. 세 번째 검사가 이 왕복을 지킨다.
//!
//! 측정: 2026-11-11, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 `b28f2edc9a`(base
//! `e118de8aba` + 이 수정). 1쪽 Native 98.34 / fresh WASM 98.38, 2쪽 Native 98.36 / fresh WASM
//! 98.40. 전쪽 최저 Native 98.34·fresh WASM 98.38(수정 전 Native 63.89·28.81). 전체 2쪽 = PDF 2쪽.
//! 증거: `mydocs/pr/assets/issue_8939/{native,wasm,base-native}/`.
//!
//! 검사: 원본 계약(문단 6 첫 저장 줄 vpos 0·원본 파일 tag bit 31, 표 칸 저장 사다리가 칸 선언 높이에 맞음),
//! IR 줄 tag 에 합성 표지 없음, 쪽수 2 = 한/글(고정형), 문단 6~8 이 2쪽 소속이고 2쪽 첫 줄은
//! 문단 6, HWP5 재저장본의 본문 줄 tag = 원본이고 되읽은 tag 에 합성 표지 없음, 1쪽 표 칸의 그린
//! 높이 = 칸 선언 높이, rhwp 편집 저장본은 표지와 함께 합성 줄 tag 를 그대로 되읽음(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8939/attach_form_reset.hwp";
/// 2쪽 맨 위에서 시작하는 「<붙임 4>」 문단.
const RESET: usize = 6;
/// 1쪽 신청서 표의 host 문단.
const FORM_HOST: usize = 4;

fn load_bytes(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn sample_bytes() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn load() -> DocumentCore {
    load_bytes(&sample_bytes())
}

/// HWP5 바이트의 첫 구역에서 본문 직속 문단마다 `PARA_LINE_SEG` 의 원본 tag 목록을 뽑는다.
fn top_level_line_seg_tags(hwp: &[u8]) -> Vec<Vec<u32>> {
    const HWPTAG_PARA_HEADER: u16 = 0x010 + 50;
    const HWPTAG_PARA_LINE_SEG: u16 = 0x010 + 53;
    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec())).expect("CFB 열기");
    let compressed = {
        let mut header = cfb.open_stream("/FileHeader").expect("FileHeader");
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut header, &mut buf).expect("FileHeader 읽기");
        u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]) & 1 == 1
    };
    let mut stream = cfb.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    std::io::Read::read_to_end(&mut stream, &mut raw).expect("Section0 읽기");
    let body = if compressed {
        let mut decoded = Vec::new();
        let mut dec = flate2::read::DeflateDecoder::new(&raw[..]);
        std::io::Read::read_to_end(&mut dec, &mut decoded).expect("Section0 압축 해제");
        decoded
    } else {
        raw
    };
    let mut out: Vec<Vec<u32>> = Vec::new();
    let mut off = 0usize;
    while off + 4 <= body.len() {
        let h = u32::from_le_bytes(body[off..off + 4].try_into().unwrap());
        off += 4;
        let tag = (h & 0x3FF) as u16;
        let level = (h >> 10) & 0x3FF;
        let mut size = ((h >> 20) & 0xFFF) as usize;
        if size == 0xFFF {
            size = u32::from_le_bytes(body[off..off + 4].try_into().unwrap()) as usize;
            off += 4;
        }
        let data = &body[off..(off + size).min(body.len())];
        off += size;
        if tag == HWPTAG_PARA_HEADER && level == 0 {
            out.push(Vec::new());
        } else if tag == HWPTAG_PARA_LINE_SEG && level == 1 {
            let last = out.last_mut().expect("문단 머리 뒤 줄 레코드");
            last.extend(
                data.chunks_exact(36)
                    .map(|seg| u32::from_le_bytes(seg[32..36].try_into().unwrap())),
            );
        }
    }
    out
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn body_lines(node: &RenderNode, out: &mut Vec<(Option<usize>, BoundingBox)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => return,
        RenderNodeType::TextLine(line) => out.push((line.para_index, node.bbox)),
        _ => {}
    }
    for child in &node.children {
        body_lines(child, out);
    }
}

fn find_table<'a>(node: &'a RenderNode, para: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.cell_context.is_none() {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para))
}

#[test]
fn stored_bit31_line_keeps_saved_page_reset_and_round_trips() {
    let core = load();
    let paragraphs = &core.document().sections[0].paragraphs;

    // 원본 계약: 문단 6 의 첫 저장 줄은 vpos 0(쪽 되감김)이고, 한/글은 그 줄 tag 에 bit 31 을 싣는다.
    let source_tags = top_level_line_seg_tags(&sample_bytes());
    assert_eq!(source_tags.len(), paragraphs.len(), "본문 문단 수");
    assert_ne!(
        source_tags[RESET][0] & LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "원본 문단 6 tag bit 31"
    );
    let reset = &paragraphs[RESET];
    assert_eq!(reset.line_segs[0].vertical_pos, 0, "문단 6 저장 vpos 0");
    assert!(paragraphs[RESET - 1].line_segs[0].vertical_pos > 0);
    // IR tag 의 bit 31 은 rhwp 가 만든 합성 줄 표지다. 원본 저장 줄은 거기서 비워 둔다.
    assert_eq!(
        reset.line_segs[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "저장 줄은 합성 줄 표지를 갖지 않는다"
    );

    assert_eq!(core.page_count(), 2, "전체 쪽수 = 한/글 2쪽");

    let mut first = Vec::new();
    body_lines(
        &core.build_page_render_tree(0).expect("1쪽").root,
        &mut first,
    );
    let mut second = Vec::new();
    body_lines(
        &core.build_page_render_tree(1).expect("2쪽").root,
        &mut second,
    );
    for para in RESET..=RESET + 2 {
        assert!(
            first.iter().all(|(owner, _)| *owner != Some(para)),
            "문단 {para} 는 1쪽에 그려지지 않는다"
        );
        assert!(
            second.iter().any(|(owner, _)| *owner == Some(para)),
            "문단 {para} 는 2쪽 소속"
        );
    }
    let top = second
        .iter()
        .min_by(|a, b| a.1.y.total_cmp(&b.1.y))
        .expect("2쪽 첫 줄");
    assert_eq!(top.0, Some(RESET), "2쪽 첫 줄은 문단 6");

    // HWP5 로 다시 저장해도 원본 tag(bit 31 포함)가 그대로 돌아간다.
    let saved = core.export_hwp_native().expect("HWP 저장");
    assert_eq!(
        top_level_line_seg_tags(&saved),
        source_tags,
        "재저장본의 본문 줄 tag = 원본"
    );
    let reloaded = load_bytes(&saved);
    assert_eq!(
        reloaded.document().sections[0].paragraphs[RESET].line_segs[0].tag
            & LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "원본 바이트를 그대로 다시 쓴 구역의 bit 31 도 저장 줄"
    );
    assert_eq!(reloaded.page_count(), 2, "재저장본 쪽수");
}

#[test]
fn nested_tac_form_cell_uses_stored_ladder_that_fits_declared_height() {
    let core = load();
    let paragraphs = &core.document().sections[0].paragraphs;
    let table = paragraphs[FORM_HOST]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("신청서 표");
    assert!(table.common.treat_as_char, "글자처럼 취급 표");

    // 원본 계약: 중첩 표를 품은 칸의 저장 사다리 끝 + 위아래 안 여백 = 칸 선언 높이.
    let (index, cell) = table
        .cells
        .iter()
        .enumerate()
        .find(|(_, cell)| {
            cell.paragraphs
                .iter()
                .any(|para| para.controls.iter().any(|c| matches!(c, Control::Table(_))))
        })
        .expect("중첩 표 칸");
    let pad = cell.effective_padding(&table.padding);
    let ladder_end = cell
        .paragraphs
        .iter()
        .flat_map(|para| para.line_segs.iter())
        .map(|seg| seg.vertical_pos + seg.line_height)
        .max()
        .unwrap();
    assert_eq!(
        ladder_end + i32::from(pad.top) + i32::from(pad.bottom),
        cell.height as i32,
        "칸 {index} 저장 사다리가 선언 높이에 맞음"
    );

    let page = core.build_page_render_tree(0).expect("1쪽");
    let node = find_table(&page.root, FORM_HOST).expect("신청서 표 상자");
    let mut checked = 0;
    for child in &node.children {
        let RenderNodeType::TableCell(cell_node) = &child.node_type else {
            continue;
        };
        let model = &table.cells[cell_node.model_cell_index.expect("칸 인덱스") as usize];
        let declared = hu_to_px(model.height as i32);
        assert!(
            (child.bbox.height - declared).abs() <= 0.5,
            "칸 r{} c{} 그린 높이 {:.1} = 선언 {:.1}",
            cell_node.row,
            cell_node.col,
            child.bbox.height,
            declared
        );
        checked += 1;
    }
    assert_eq!(checked, table.cells.len(), "표 칸 전부 검사");
}

/// HWP5 바이트에 rhwp 저장 표지 스트림이 있는가.
fn has_rhwp_line_seg_marker(hwp: &[u8]) -> bool {
    cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec()))
        .expect("CFB 열기")
        .is_stream("/RhwpLineSegTags")
}

/// rhwp 가 편집해 다시 쓴 구역은 저장 표지로 인증되어, 다시 열어도 bit 31 이 rhwp 합성 줄
/// 표지 그대로다. 한/글 원본 구역(표지 없음)은 위 검사처럼 bit 31 을 저장 줄로 읽는다.
#[test]
fn rhwp_saved_section_keeps_synthetic_line_tags_through_marker() {
    let source = sample_bytes();
    assert!(
        !has_rhwp_line_seg_marker(&source),
        "한/글 원본에는 표지가 없다"
    );

    let mut core = load_bytes(&source);
    // 공개 표 생성 API 가 만드는 칸 문단의 줄은 rhwp 가 계산한 합성 줄이다.
    core.create_table_native(0, RESET + 1, 0, 2, 2)
        .expect("표 만들기");
    let tags = |core: &DocumentCore| -> Vec<Vec<u32>> {
        let mut out = Vec::new();
        for para in &core.document().sections[0].paragraphs {
            out.push(para.line_segs.iter().map(|seg| seg.tag).collect());
            for control in &para.controls {
                if let Control::Table(table) = control {
                    for cell in &table.cells {
                        for cell_para in &cell.paragraphs {
                            out.push(cell_para.line_segs.iter().map(|seg| seg.tag).collect());
                        }
                    }
                }
            }
        }
        out
    };
    let edited = tags(&core);
    assert!(
        edited
            .iter()
            .flatten()
            .any(|tag| tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0),
        "새 표 칸은 rhwp 합성 줄을 갖는다"
    );

    let saved = core.export_hwp_native().expect("HWP 저장");
    assert!(
        has_rhwp_line_seg_marker(&saved),
        "rhwp 가 다시 쓴 구역은 표지를 남긴다"
    );
    let reloaded = load_bytes(&saved);
    assert_eq!(tags(&reloaded), edited, "다시 연 줄 tag = 저장 전 IR tag");
}
