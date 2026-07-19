//! Issue #2320: 문단 중간 vpos 되감김(쪽 경계 인코딩) 미인식.
//!
//! treatise sample.hwp p2 오른쪽 단(단 1)의 문단 29: 저장 LINE_SEG 가
//! line 0 vpos=67050(단 하단) → line 1 vpos=4926 으로 되감긴다 — 한컴의
//! "문단 중간 쪽 경계" 인코딩 (한컴 2022 PDF: 문단 29 첫 줄에서 끊고 p3).
//!
//! 되감김 감지(`detect_column_breaks_in_paragraph`)는 존재하지만 호출 게이트가
//! `current_column == 0` 이라 단 1 에서 시작하는 문단은 미감지 → 문단 통째
//! 배치 → 하단 106.8px 잘림 + 내용 소실 (LAYOUT_OVERFLOW).
//!
//! 정정 계약: 비-0 단에서 시작하는 문단의 문단 내 되감김을 분할점으로 처리 —
//! 마지막 단이면 쪽 넘김 + 다음 페이지 단 0 에서 잔여 줄 계속.

use rhwp::document_core::DocumentCore;

fn load_treatise() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join("samples/basic/treatise sample.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse treatise sample.hwp")
}

/// dump_page_items 출력에서 특정 pi 의 항목 줄을 찾는다.
fn item_lines_for(dump: &str, pi: usize) -> Vec<String> {
    let needle = format!("pi={pi} ");
    dump.lines()
        .filter(|l| {
            (l.contains("FullParagraph") || l.contains("PartialParagraph"))
                && (l.contains(&needle) || l.trim_end().ends_with(&format!("pi={pi}")))
        })
        .map(|l| l.trim().to_string())
        .collect()
}

#[test]
fn issue_2320_last_column_rewind_splits_to_next_page() {
    let core = load_treatise();
    assert_eq!(core.page_count(), 7, "treatise 전체 페이지 수 불변");

    let p2 = core.dump_page_items(Some(1));
    let p3 = core.dump_page_items(Some(2));

    // 문단 29 는 p2 에서 line 1 에서 끊겨야 한다 (PartialParagraph lines=0..1)
    let p2_29 = item_lines_for(&p2, 29);
    assert_eq!(p2_29.len(), 1, "p2 에 문단 29 항목 1개: {:?}", p2_29);
    assert!(
        p2_29[0].contains("PartialParagraph") && p2_29[0].contains("lines=0..1"),
        "문단 29 는 p2 단 1 에서 첫 줄만 배치 (되감김 분할): {}",
        p2_29[0]
    );

    // 문단 30 은 p2 에 없어야 한다 (정정 전: PartialParagraph pi=30 이 잘린 채 배치)
    let p2_30 = item_lines_for(&p2, 30);
    assert!(
        p2_30.is_empty(),
        "문단 30 은 p2 에 배치되지 않아야 함 (하단 잘림 소멸): {:?}",
        p2_30
    );

    // 문단 29 잔여(lines 1..)는 p3 에서 계속되어야 한다
    let p3_29 = item_lines_for(&p3, 29);
    assert_eq!(p3_29.len(), 1, "p3 에 문단 29 잔여 항목 1개: {:?}", p3_29);
    assert!(
        p3_29[0].contains("PartialParagraph") && p3_29[0].contains("lines=1.."),
        "문단 29 잔여 줄이 p3 에서 계속: {}",
        p3_29[0]
    );
}

/// 페이지 중간으로 되감기는 비-0 단 문단은 경계로 오인하지 않아야 한다.
/// 143E 신문 스크랩: 단 1 시작 문단 pi=9 의 되감김 목표가 본문 높이의 약 40%
/// (26644HU) — 개체 어울림 흐름의 잔재이지 쪽 경계 인코딩이 아니다.
/// (detect_near_top_rewind_breaks 의 단 상단 근방 가드 핀)
#[test]
fn issue_2320_mid_page_rewind_is_not_boundary() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join("samples/143E433F503322BD33.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let core = DocumentCore::from_bytes(&bytes).expect("parse 143E433F503322BD33.hwp");
    assert_eq!(
        core.page_count(),
        1,
        "페이지 중간 되감김을 쪽 경계로 오인하면 1쪽 문서가 2쪽이 된다"
    );
}

/// 기존 0-리셋 단 경계 분할(단 0 시작 문단)은 불변이어야 한다.
#[test]
fn issue_2320_existing_column_zero_split_unchanged() {
    let core = load_treatise();
    let p2 = core.dump_page_items(Some(1));

    // 문단 21: 단 0 lines=0..2 / 단 1 lines=2..5 분할 (기존 동작 고정)
    let p2_21 = item_lines_for(&p2, 21);
    assert_eq!(p2_21.len(), 2, "문단 21 은 단 0/1 에 2개 항목: {:?}", p2_21);
    assert!(
        p2_21[0].contains("lines=0..2") && p2_21[1].contains("lines=2..5"),
        "문단 21 의 기존 단 경계 분할 불변: {:?}",
        p2_21
    );
}
