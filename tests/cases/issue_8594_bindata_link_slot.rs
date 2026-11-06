//! 검사 유형: 관계형.
//! 주석 갱신: 2026-11-06T11:54:54+09:00; 직전 파일 커밋: d60381eb473a3d56a47866b7ef2c124cd45f9e37 (2026-11-05T14:42:30+09:00).
//! 2026-11-06 Visual Sweep: 96dpi·PDFium·색상 경계 보정 전 2px raw 실루엣.
//! 같은 입력의 저장 버전에 맞는 Desktop Print(2020→compat 2022, 2024→compat 2024)와 대조.
//! bindata_link_shift: 전 1쪽 Native 99.98529% / fresh optimized WASM 99.98529%; 전쪽 최저 99.98529% / 99.98529%.
//! 증거: mydocs/pr/assets/hyunmyungjanelee-20261106/{native,wasm}-<입력-stem>.tsv,
//! input-reference-pairs.json·측정 JSON·대표 PNG 및 해당 원 PR review 문서.
//! Native/fresh WASM의 직접 확대 판독과 입력·PDF·제품 해시는 공통 기록에 연결한다.
//!
//! [#8594] HWP5 BinData 연결(Link) 레코드 뒤 그림이 다른 그림을 그리거나 사라지던
//! 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! BinData 적재가 연결 레코드를 배열에서 빼고 넘어갔다. 그림의 `bin_data_id` 는 BIN_DATA
//! 레코드 순번이고 조회(`content_index`)는 그 순번을 배열 위치로 쓰므로, 연결 레코드 뒤
//! 그림이 한 칸씩 밀려 다른 그림을 가리키거나 범위를 넘어 사라졌다.
//!
//! # 기대값의 출처
//!
//! 재현물 `samples/issue8594/bindata_link_shift.hwp` 는 공개 샘플 `samples/ta-pic-001-r.hwp`
//! (표 안 그림 2개)의 DocInfo BIN_DATA 맨 앞에 연결 레코드를 끼우고, ID_MAPPINGS 의 BinData
//! 수와 그림의 `bin_data_id` 를 1씩 늘린 것이다. 그래서 그림마다 원본 샘플의 같은 그림을
//! 그려야 한다. 웹한글기안기 인쇄 PDF(`pdf/issue8594/bindata_link_shift-webhwp-print.pdf`)도
//! 두 그림을 원본 샘플과 같게 그린다.
//!
//! # 검사하는 것
//!
//! - 그림마다 그려지는 바이트·원본 크기·자르기가 원본 샘플의 같은 그림과 같다.
//! - HWP5·HWPX 저장 결과에 연결 레코드 자리 항목이 BinData 스트림·항목으로 생기지 않고,
//!   다시 읽어도 같다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{ImageNode, RenderNode, RenderNodeType};

const BASE: &str = "samples/ta-pic-001-r.hwp";
const LINK_SHIFT: &str = "samples/issue8594/bindata_link_shift.hwp";
/// 원본 샘플의 그림(=내장 BinData) 수.
const PICTURES: usize = 2;

#[derive(Debug, PartialEq)]
struct Pic {
    data: Option<Vec<u8>>,
    dim: Option<(u32, u32)>,
    crop: Option<(i32, i32, i32, i32)>,
    opacity: f64,
}

impl Pic {
    fn of(image: &ImageNode) -> Self {
        Self {
            data: image.data.clone(),
            dim: image.original_size_hu,
            crop: image.crop,
            opacity: image.opacity,
        }
    }
}

fn collect(node: &RenderNode, out: &mut Vec<Pic>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.fill_mode.is_none() {
            out.push(Pic::of(image));
        }
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"))
}

fn pictures(doc: &DocumentCore) -> Vec<Pic> {
    let mut out = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("렌더 트리");
        collect(&tree.root, &mut out);
    }
    out
}

fn load(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("문서 로드")
}

fn base_pictures() -> Vec<Pic> {
    let pics = pictures(&load(&read(BASE)));
    assert_eq!(pics.len(), PICTURES, "원본 샘플 그림 수 — 시험 설정 오류");
    assert!(
        pics.iter()
            .all(|p| p.data.as_ref().is_some_and(|d| !d.is_empty())),
        "원본 샘플 그림은 모두 바이트가 있어야 한다 — 시험 설정 오류"
    );
    pics
}

fn assert_same_pictures(label: &str, actual: &[Pic], expected: &[Pic]) {
    assert_eq!(actual.len(), expected.len(), "{label}: 그림 수");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            a.dim, e.dim,
            "{label}: 그림 {i} 원본 크기(imgDim)가 원본 샘플과 다르다 — 원본 크기가 바뀌었다"
        );
        assert_eq!(a.crop, e.crop, "{label}: 그림 {i} 자르기");
        assert_eq!(a.opacity, e.opacity, "{label}: 그림 {i} 투명도");
        assert!(
            a.data == e.data,
            "{label}: 그림 {i} 가 원본 샘플의 같은 그림과 다른 바이트를 그린다 — \
             연결 레코드 뒤 그림이 다른 BinData 를 가리킨다"
        );
    }
}

#[test]
fn pictures_after_link_record_keep_their_own_bin_data() {
    let expected = base_pictures();
    assert_same_pictures(
        "연결 레코드",
        &pictures(&load(&read(LINK_SHIFT))),
        &expected,
    );
}

#[test]
fn link_record_slot_is_not_saved_as_bin_data_stream() {
    let expected = base_pictures();
    let source = load(&read(LINK_SHIFT));
    let saved = source.export_hwp_native().expect("HWP5 저장");
    let reloaded = load(&saved);
    assert_eq!(
        reloaded.document().doc_info.bin_data_list.len(),
        source.document().doc_info.bin_data_list.len(),
        "저장 후 BIN_DATA 레코드 수"
    );
    let non_empty = |doc: &DocumentCore| {
        doc.document()
            .bin_data_content
            .iter()
            .filter(|c| !c.data.is_empty())
            .count()
    };
    assert_eq!(non_empty(&reloaded), PICTURES, "저장 후 내장 BinData 수");
    let cfb = cfb::CompoundFile::open(std::io::Cursor::new(&saved)).expect("CFB 열기");
    let mut streams: Vec<String> = cfb
        .walk_storage("/BinData")
        .expect("BinData 저장소")
        .filter(|e| e.is_stream())
        .map(|e| e.name().to_string())
        .collect();
    streams.sort();
    let mut original: Vec<String> = cfb::CompoundFile::open(std::io::Cursor::new(read(LINK_SHIFT)))
        .expect("CFB 열기")
        .walk_storage("/BinData")
        .expect("BinData 저장소")
        .filter(|e| e.is_stream())
        .map(|e| e.name().to_string())
        .collect();
    original.sort();
    assert_eq!(
        streams, original,
        "저장 결과의 BinData 스트림이 원본과 달라졌다 — 연결 레코드 자리 항목이 스트림으로 저장됐다"
    );
    assert_same_pictures("연결 레코드 저장 후", &pictures(&reloaded), &expected);
}

#[test]
fn link_record_slot_is_not_saved_as_hwpx_bin_data_entry() {
    let expected = base_pictures();
    let saved = load(&read(LINK_SHIFT))
        .export_hwpx_native()
        .expect("HWPX 저장");
    let zip = zip::ZipArchive::new(std::io::Cursor::new(&saved)).expect("ZIP 열기");
    let entries: Vec<String> = zip
        .file_names()
        .map(|n| n.expect("ZIP 항목 이름").into_owned())
        .filter(|n| n.starts_with("BinData/"))
        .collect();
    assert_eq!(
        entries.len(),
        PICTURES,
        "HWPX BinData 항목은 내장 그림 수와 같아야 한다(연결 레코드 자리 항목 제외): {entries:?}"
    );
    assert_same_pictures(
        "연결 레코드 HWPX 저장 후",
        &pictures(&load(&saved)),
        &expected,
    );
}
