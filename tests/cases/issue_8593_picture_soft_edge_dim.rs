//! 검사 유형: 관계형.
//! 주석 갱신: 2026-11-06T11:54:54+09:00; 직전 파일 커밋: 88f902646a4f0fa4dee822821a27fb848e50cdb3 (2026-11-06T09:26:03+09:00).
//! 2026-11-06 Visual Sweep: 96dpi·PDFium·색상 경계 보정 전 2px raw 실루엣.
//! 같은 입력의 저장 버전에 맞는 Desktop Print(2020→compat 2022, 2024→compat 2024)와 대조.
//! picture_soft_edge: 전 1쪽 Native 99.91617% / fresh optimized WASM 99.91621%; 전쪽 최저 99.91617% / 99.91621%.
//! 증거: mydocs/pr/assets/hyunmyungjanelee-20261106/{native,wasm}-<입력-stem>.tsv,
//! input-reference-pairs.json·측정 JSON·대표 PNG 및 해당 원 PR review 문서.
//! Native/fresh WASM의 직접 확대 판독과 입력·PDF·제품 해시는 공통 기록에 연결한다.
//!
//! [#8593] HWP5 부드러운 가장자리 효과 그림이 원본의 극히 일부만 잘려 단색 네모로
//! 그려지던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 그림 레코드(SHAPE_COMPONENT_PICTURE) 꼬리는 [테두리 투명도 u8][인스턴스 id u32]
//! [그림 효과 정보][dimwidth u32][dimheight u32][선택적 투명도 u8] 이다(실제 확장 꼬리). 그림 효과 정보는
//! [효과 플래그 u32][효과별 정보] 이고(표 108), 효과가 없으면 플래그 0 뿐이라 원본 크기가
//! 고정 위치 9..17 에 온다. 이 고정 위치로만 읽어서, 부드러운 가장자리(플래그 4, 반경 f32,
//! 표 111) 그림에서는 반경 300.0 의 비트(1133903872)가 dimwidth 로 읽혀 자르기 영역이
//! 원본의 극히 일부가 됐다. 투명도 바이트가 생략된 21바이트 합성 대조군에서는
//! 마지막 바이트(dimheight 일부)를 투명도로 읽었다.
//!
//! # 기대값의 출처
//!
//! 재현물 `samples/issue8593/picture_soft_edge.hwp` 는 공개 샘플 `samples/ta-pic-001-r.hwp`
//! (표 안 그림 2개)의 그림 꼬리에 부드러운 가장자리 효과 정보만 끼운 것이다(반경 300·500,
//! 22·21바이트 꼬리). 그래서 그림마다 원본 크기·자르기·투명도·그림 바이트가 원본 샘플과
//! 같아야 한다.
//!
//! 기준은 동일 입력의 한컴 2020 Desktop Print다:
//! `pdf/issue8593/picture_soft_edge-2020.pdf` 및 동명 provenance JSON.
//! 실제 11.0.0.9136, Print method 0, 한 쪽씩 PDF driver 출력임을 확인했다.
//! 웹 화면과 같은 크기로 두 그림을 그린다. 웹 Print의 약 75% 축소 차이를
//! 별도 비교했으며, 화면 PNG로 Print 게이트를 대체하지 않는다.
//! 이 회귀는 크기·자르기·alpha 해석을 보호하며 blur 구현을 주장하지 않는다.
//!
//! # 검사하는 것
//!
//! 그림마다 원본 크기(imgDim)·자르기·투명도·바이트가 원본 샘플과 같고, 자르기 영역이 원본
//! 크기 안에 있다. HWP5 저장 후 다시 읽어도 같다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{ImageNode, RenderNode, RenderNodeType};

const BASE: &str = "samples/ta-pic-001-r.hwp";
const SOFT_EDGE: &str = "samples/issue8593/picture_soft_edge.hwp";
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
            "{label}: 그림 {i} 원본 크기(imgDim)가 원본 샘플과 다르다 — 그림 효과 정보를 크기로 읽었다"
        );
        assert_eq!(a.crop, e.crop, "{label}: 그림 {i} 자르기");
        assert_eq!(a.opacity, e.opacity, "{label}: 그림 {i} 투명도");
        assert!(
            a.data == e.data,
            "{label}: 그림 {i} 가 원본 샘플의 같은 그림과 다른 바이트를 그린다 — \
             그림 바이트가 바뀌었다"
        );
    }
}

#[test]
fn soft_edge_picture_keeps_original_image_dimensions() {
    let expected = base_pictures();
    let actual = pictures(&load(&read(SOFT_EDGE)));
    for (i, p) in actual.iter().enumerate() {
        let (Some((w, h)), Some((l, t, r, b))) = (p.dim, p.crop) else {
            continue;
        };
        assert!(
            l >= 0 && t >= 0 && r as i64 <= w as i64 && b as i64 <= h as i64,
            "그림 {i} 자르기 영역 {:?} 가 원본 크기 {:?} 밖이다",
            p.crop,
            p.dim
        );
    }
    assert_same_pictures("부드러운 가장자리", &actual, &expected);
}

#[test]
fn soft_edge_picture_survives_hwp5_save() {
    let expected = base_pictures();
    let saved = load(&read(SOFT_EDGE))
        .export_hwp_native()
        .expect("HWP5 저장");
    assert_same_pictures(
        "부드러운 가장자리 저장 후",
        &pictures(&load(&saved)),
        &expected,
    );
}
