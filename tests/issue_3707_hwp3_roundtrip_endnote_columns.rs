//! Issue #3707: HWP3 왕복본에서 미주가 한 쪽씩 뒤로 밀린다.
//!
//! ## 근인
//!
//! HWP3 파서는 `page_def.pagination_bottom_tolerance` 를 **1600 HU**(21.3px)로 세운다 —
//! 한글97 의 마지막 줄 tolerance 를 흉내 내 페이지네이터에게만 여유를 주는 **렌더러 내부
//! 값**이고 파일 포맷 필드가 아니다. HWP5 로 저장·재파싱하면 0 이 되어 본문 가용이 그만큼
//! 짧아진다.
//!
//! ```text
//! SO-SUEOP 왕복본: 문단 1,037 · 고유 ps 733 · 고유 cs 1,156
//!                  ps 0.707 · cs 1.115   (임계 0.05 / 0.15)
//! ```
//!
//! 보정이 사라지면 본문이 21.3px 짧아지고, 그만큼 미주 단 가용이 줄어 단 전환이 일찍
//! 걸린다. 2단 미주의 왼쪽 단이 조기에 닫혀 미주가 다음 쪽으로 밀린다.
//!
//! ```text
//!                     허용치      본문(미주 단 가용)
//! 원본 (HWP3)          1600 HU        877.8 px
//! 왕복본 (수정 전)         0 HU        856.4 px   ← 21.3px 짧다
//! 왕복본 (수정 후)      1600 HU        877.8 px
//!
//! 미주 128·129   원본 44쪽  ·  수정 전 45쪽  ·  수정 후 44쪽
//! ```
//!
//! 한컴은 원본·왕복본 모두 44쪽에 싣는다(PDF 실측). 즉 보정이 유지되는 쪽이 정답지다.
//!
//! ## 계약
//!
//! 파일에 실리는 여백은 **원본 그대로** 둔다 — 그래야 한컴이 보는 기하가 안 바뀌고
//! `convert --verify` 의 IR 비교도 깨지지 않는다. 대신 출처 마커(`/RhwpHwp3Origin`)를
//! 남겨 재파싱이 렌더러 내부 허용치만 되돌린다. 계약은 **미주가 원본과 같은 쪽에 실리는
//! 것**이다(한컴 정답지와 일치).
#![cfg(not(target_arch = "wasm32"))]

const SAMPLE: &str = "samples/SO-SUEOP.hwp";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// HWP3 원본 파싱 → 저장 → 재파싱.
fn parse_and_roundtrip() -> (rhwp::model::document::Document, Vec<u8>) {
    let raw = std::fs::read(sample_path()).expect("표본 읽기");
    let original = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    let mut to_save = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut to_save,
        rhwp::parser::FileFormat::Hwp3,
    );
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(&to_save).expect("HWP5 직렬화");
    (original, bytes)
}

fn margin_bottoms(doc: &rhwp::model::document::Document) -> Vec<u32> {
    doc.sections
        .iter()
        .map(|s| s.section_def.page_def.margin_bottom)
        .collect()
}

/// 파일에 실리는 여백은 건드리지 않는다 — 한컴이 보는 쪽 기하가 원본과 같아야 한다.
#[test]
fn stored_bottom_margin_is_untouched() {
    let raw = std::fs::read(sample_path()).expect("표본 읽기");
    let original = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    let (_, bytes) = parse_and_roundtrip();
    let reparsed = rhwp::parser::parse_document(&bytes).expect("왕복본 재파싱");
    assert_eq!(
        margin_bottoms(&reparsed),
        margin_bottoms(&original),
        "왕복 후 margin_bottom 이 달라졌다. 여백을 줄여 보정하면 한컴이 보는 쪽 기하가          원본과 달라지고 `convert --verify` 의 IR 비교에도 잡힌다 — 보정은 렌더러 내부          허용치로만 걸어야 한다."
    );
}

/// 재파싱이 출처 마커를 보고 쪽나눔 허용치를 되돌린다.
#[test]
fn reparse_restores_pagination_bottom_tolerance_via_origin_marker() {
    let raw = std::fs::read(sample_path()).expect("표본 읽기");
    let original = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    let (_, bytes) = parse_and_roundtrip();
    let reparsed = rhwp::parser::parse_document(&bytes).expect("왕복본 재파싱");

    let tol = |d: &rhwp::model::document::Document| -> Vec<u32> {
        d.sections
            .iter()
            .map(|s| s.section_def.page_def.pagination_bottom_tolerance)
            .collect()
    };
    let before = tol(&original);
    assert!(
        before.iter().any(|&v| v > 0),
        "HWP3 원본이 허용치를 세우지 않는다 — 표본/파서 확인 (실측 1600 HU)"
    );
    assert_eq!(
        tol(&reparsed),
        before,
        "왕복본이 쪽나눔 허용치를 잃었다. 그만큼(21.3px) 본문 가용이 짧아져 2단 미주의          왼쪽 단이 조기에 닫히고 미주가 다음 쪽으로 밀린다."
    );
}

/// 미주가 원본과 같은 쪽에 실린다 (한컴 정답지: 128·129 → 44쪽).
#[test]
fn endnote_bodies_land_on_the_same_pages_after_roundtrip() {
    let raw = std::fs::read(sample_path()).expect("표본 읽기");
    let (_, bytes) = parse_and_roundtrip();

    let page_of = |data: &[u8], needle: &str| -> Option<u32> {
        let core = rhwp::document_core::DocumentCore::from_bytes(data).expect("파싱");
        let key: String = needle.chars().filter(|c| !c.is_whitespace()).collect();
        let pages = core.page_count();
        (0..pages).find(|&p| {
            core.extract_page_text_native(p)
                .map(|t| {
                    t.chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>()
                        .contains(&key)
                })
                .unwrap_or(false)
        })
    };

    for needle in ["128) 염상섭의 삼대", "129) <태평천하"] {
        let a = page_of(&raw, needle);
        let b = page_of(&bytes, needle);
        assert!(
            a.is_some(),
            "원본에서 미주 {needle:?} 를 찾지 못했다 — 표본 확인"
        );
        assert_eq!(
            b, a,
            "미주 {needle:?} 가 왕복 후 다른 쪽으로 갔다 (원본 {a:?} → 왕복본 {b:?}).\n\
             한컴은 두 파일 모두 같은 쪽에 싣는다. 아래 여백 보정이 왕복에서 사라지면 \
             미주 단 가용이 21.3px 줄어 왼쪽 단이 조기에 닫힌다."
        );
    }
}
