//! [#6907] `content.hpf` 의 항목 **순서**가 바탕쪽의 구역 소속이다 — 저장이 그 순서를 지킨다.
//!
//! # 무엇이 깨져 있었나
//!
//! HWPX 직렬화기가 `opf:item` 을 **종류별로 묶어**(구역 전부 → 바탕쪽 전부) 다시 썼다.
//!
//! ```text
//!   원본:  masterpage0,1,2 · section0 · masterpage3,4,5 · section1 · masterpage6,7,8 · section2
//!   저장:  section0 · section1 · section2 · masterpage0 … masterpage8
//! ```
//!
//! 파서는 `sectionN.xml` 항목을 만나면 **그때까지 모인 masterpage 를 그 구역에** 배정한다.
//! 종류별로 몰아 쓰면 모든 구역의 그룹이 비고, 파서가 「전부 비면 균등 배분」 폴백으로
//! 떨어져 **원래 바탕쪽이 없던 구역에도 소속을 만들어 낸다**(구역 XML 에 `masterPage idRef`
//! 가 없는 구역이 직격탄이다). 신고에 따르면 편집 **0회** 저장만으로 실문서 3~8쪽이
//! 달라지고 없던 머리말·장 표지가 생겼다.
//!
//! # 이 검사가 보는 것
//!
//! 저장소 안 표본 `samples/hwpx/exam_kor.hwpx` 로 두 가지를 본다.
//!
//! 1. `content.hpf` 의 `section*`/`masterpage*` 항목 **순서**가 원본과 같다.
//! 2. 그래서 다시 읽었을 때 **구역별 바탕쪽 개수**가 보존된다(3·3·3).
//!
//! 순서만 보면 "종류별로 묶여도 개수는 같지 않나" 를 놓치고, 개수만 보면 이 표본처럼
//! 균등 배분이 우연히 원래 소속과 일치하는 경우를 놓친다 — 둘을 함께 본다.
//!
//! ⚠ `exam_kor` 표본의 한계: 이 문서는 구역마다 `masterPage idRef` 가 있어 파서의 1차 경로로도
//! 붙고, 바탕쪽 9개·구역 3개라 균등 배분(3개씩)이 원래 소속과 **우연히 같다**. 그래서
//! 렌더 산출로는 차이가 보이지 않는다. 저장소의 한컴오피스 2024 저장본
//! `2025 행정업무운영 편람(최종).hwpx` 는 14구역의 바탕쪽 분포가 비균등하며,
//! 0·13구역에는 `idRef` 도 없다. 아래 별도 검사가 이 실제 반례를 확인한다.
//!
//! 그래서 **수정 전 실패하는 것은 순서 검사 하나**다. 소속 개수 검사는 이 표본에서
//! 균등 배분(9÷3=3)이 원래 소속과 같아 수정 전에도 통과한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

const SAMPLE: &str = "samples/hwpx/exam_kor.hwpx";
const NONUNIFORM_SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";
const NONUNIFORM_COUNTS: [usize; 14] = [0, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 0];

/// `content.hpf` 에서 `section*`/`masterpage*` 항목의 href 를 **등장 순서대로** 뽑는다.
fn manifest_order(package: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(package)).expect("hwpx 패키지 열기");
    let mut xml = String::new();
    {
        use std::io::Read;
        let mut entry = zip
            .by_name("Contents/content.hpf")
            .expect("content.hpf 찾기");
        entry.read_to_string(&mut xml).expect("content.hpf 읽기");
    }
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(pos) = rest.find("href=\"") {
        rest = &rest[pos + 6..];
        let Some(end) = rest.find('"') else { break };
        let href = &rest[..end];
        let lower = href.to_ascii_lowercase();
        if lower.contains("masterpage") || lower.contains("section") {
            let name = href.rsplit('/').next().unwrap_or(href);
            out.push(name.trim_end_matches(".xml").to_string());
        }
        rest = &rest[end..];
    }
    out
}

fn master_counts_per_section(package: &[u8]) -> Vec<usize> {
    parse_hwpx(package)
        .expect("hwpx 파싱")
        .sections
        .iter()
        .map(|s| s.section_def.master_pages.len())
        .collect()
}

fn manifest_master_counts_per_section(package: &[u8]) -> Vec<usize> {
    let mut counts = Vec::new();
    let mut pending = 0;
    for item in manifest_order(package) {
        if item.starts_with("masterpage") {
            pending += 1;
        } else if item.starts_with("section") {
            counts.push(pending);
            pending = 0;
        }
    }
    assert_eq!(pending, 0, "마지막 구역 뒤에 소속 없는 바탕쪽이 있다");
    counts
}

fn section_master_page_ref_count(package: &[u8], section_idx: usize) -> usize {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(package)).expect("hwpx 패키지 열기");
    let mut xml = String::new();
    use std::io::Read;
    zip.by_name(&format!("Contents/section{section_idx}.xml"))
        .expect("구역 XML 찾기")
        .read_to_string(&mut xml)
        .expect("구역 XML 읽기");
    roxmltree::Document::parse(&xml)
        .expect("구역 XML 파싱")
        .descendants()
        .filter(|node| node.tag_name().name() == "masterPage" && node.attribute("idRef").is_some())
        .count()
}

/// 저장이 매니페스트 순서를 그대로 지킨다.
#[test]
fn saving_preserves_the_manifest_item_order() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let original = std::fs::read(&path).expect("재현물 읽기");

    let before = manifest_order(&original);
    assert!(
        before.iter().any(|n| n.starts_with("masterpage"))
            && before.first().is_some_and(|n| n.starts_with("masterpage")),
        "재현물 전제가 깨졌다 — 바탕쪽이 구역보다 앞서는 매니페스트여야 한다: {before:?}"
    );

    let doc = parse_hwpx(&original).expect("hwpx 파싱");
    let saved = serialize_hwpx(&doc).expect("hwpx 저장");
    let after = manifest_order(&saved);

    assert_eq!(
        after, before,
        "저장이 content.hpf 항목 순서를 바꿨다 — 이 순서가 바탕쪽의 구역 소속이라, \
         종류별로 몰아 쓰면 파서가 «전부 비면 균등 배분» 폴백으로 떨어져 없던 소속을 만든다."
    );
}

/// 그래서 다시 읽었을 때 구역별 바탕쪽 소속이 보존된다.
#[test]
fn saving_preserves_master_page_ownership_per_section() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let original = std::fs::read(&path).expect("재현물 읽기");

    let before = master_counts_per_section(&original);
    assert!(
        before.len() >= 2 && before.iter().any(|n| *n > 0),
        "재현물 전제가 깨졌다 — 구역이 둘 이상이고 바탕쪽이 있어야 한다: {before:?}"
    );

    let doc = parse_hwpx(&original).expect("hwpx 파싱");
    let saved = serialize_hwpx(&doc).expect("hwpx 저장");
    let after = master_counts_per_section(&saved);

    assert_eq!(
        after, before,
        "저장·재열기 뒤 구역별 바탕쪽 개수가 달라졌다 — 편집 0회 저장이 문서를 바꾸면 안 된다."
    );
}

/// 한컴 저장본의 0·13구역에는 idRef가 없고, 14구역의 바탕쪽 분포가 비균등하다.
/// 종류별 묶음 직렬화는 이 문서의 빈 0구역에 바탕쪽을 잘못 배정한다.
#[test]
fn saving_preserves_nonuniform_real_document_master_page_ownership() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(NONUNIFORM_SAMPLE);
    let original = std::fs::read(&path).expect("한컴 저장 HWPX 읽기");

    assert_eq!(
        manifest_master_counts_per_section(&original),
        NONUNIFORM_COUNTS,
        "원본 매니페스트 분포가 달라졌다"
    );
    assert_eq!(section_master_page_ref_count(&original, 0), 0);
    assert_eq!(section_master_page_ref_count(&original, 13), 0);
    assert!(section_master_page_ref_count(&original, 1) > 0);
    assert_eq!(
        master_counts_per_section(&original),
        NONUNIFORM_COUNTS,
        "원본 파서의 바탕쪽 소속이 달라졌다"
    );

    let doc = parse_hwpx(&original).expect("한컴 저장 HWPX 파싱");
    let before_master_pages: Vec<_> = doc
        .sections
        .iter()
        .map(|section| {
            section
                .section_def
                .master_pages
                .iter()
                .map(|page| {
                    (
                        page.apply_to,
                        page.page_front,
                        page.replace_base,
                        page.text_direction,
                        page.paragraphs
                            .iter()
                            .map(|paragraph| (paragraph.text.clone(), paragraph.controls.len()))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let saved = serialize_hwpx(&doc).expect("편집 없이 HWPX 저장");
    assert_eq!(
        master_counts_per_section(&saved),
        NONUNIFORM_COUNTS,
        "저장·재열기 뒤 빈 0구역에 바탕쪽이 잘못 붙거나 다른 소속이 바뀌었다"
    );
    assert_eq!(
        manifest_master_counts_per_section(&saved),
        NONUNIFORM_COUNTS,
        "저장된 매니페스트의 비균등 소속이 달라졌다"
    );
    let reopened = parse_hwpx(&saved).expect("저장된 HWPX 재열기");
    let after_master_pages: Vec<_> = reopened
        .sections
        .iter()
        .map(|section| {
            section
                .section_def
                .master_pages
                .iter()
                .map(|page| {
                    (
                        page.apply_to,
                        page.page_front,
                        page.replace_base,
                        page.text_direction,
                        page.paragraphs
                            .iter()
                            .map(|paragraph| (paragraph.text.clone(), paragraph.controls.len()))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    for (section_idx, (before, after)) in before_master_pages
        .iter()
        .zip(after_master_pages.iter())
        .enumerate()
    {
        assert_eq!(
            after, before,
            "{section_idx}구역의 바탕쪽 적용 범위·텍스트·개체 수가 달라졌다"
        );
    }
}
