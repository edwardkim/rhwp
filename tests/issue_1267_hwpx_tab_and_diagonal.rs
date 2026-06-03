//! HWPX 시리얼라이저 roundtrip 검증 — issue #1267
//!
//! 버그: 탭 폭/leader/type 이 4000/0/1 고정값으로 출력되어 roundtrip 시 손실.
//!
//! 수정: 탭은 para.tab_extended 참조.

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::serialize_hwpx;

// ─── 탭 roundtrip 검증 ────────────────────────────────────────────────────────

/// ref_mixed.hwpx 의 탭 width/leader/type 이 HWPX 재직렬화 후 보존되는지 확인한다.
///
/// 수정 전: 항상 width=4000, leader=0, type=1 로 덮어씌워짐.
/// 수정 후: para.tab_extended[i] 에서 width(ext[0]), leader(ext[2]&ff), type(ext[2]>>8) 복원.
#[test]
fn issue_1267_tab_width_leader_type_preserved_after_hwpx_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");
    let doc1 = parse_hwpx(bytes).expect("원본 HWPX 파싱 실패");

    // 원본 tab_extended 수집
    let orig: Vec<[u16; 7]> = doc1
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| p.tab_extended.iter().copied())
        .collect();
    assert!(
        !orig.is_empty(),
        "ref_mixed.hwpx 에 탭이 없음 — 테스트 전제 불충족"
    );

    // HWPX 재직렬화 → 재파싱
    let out = serialize_hwpx(&doc1).expect("HWPX 직렬화 실패");
    let doc2 = parse_hwpx(&out).expect("재직렬화 HWPX 파싱 실패");

    let round: Vec<[u16; 7]> = doc2
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| p.tab_extended.iter().copied())
        .collect();

    assert_eq!(
        orig.len(),
        round.len(),
        "roundtrip 후 탭 개수 불일치: {} → {}",
        orig.len(),
        round.len()
    );

    for (i, (o, r)) in orig.iter().zip(round.iter()).enumerate() {
        assert_eq!(
            o[0], r[0],
            "탭[{i}] width 불일치 (issue #1267): {} → {} (4000 고정 버그?)",
            o[0], r[0]
        );
        assert_eq!(
            o[2], r[2],
            "탭[{i}] leader/type 불일치 (issue #1267): {:#06x} → {:#06x}",
            o[2], r[2]
        );
    }
}

/// 탭 여러 개가 있을 때 각각의 width/leader/type 이 독립적으로 보존되는지 확인한다.
#[test]
fn issue_1267_multiple_tabs_each_value_preserved() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_mixed.hwpx");
    let doc1 = parse_hwpx(bytes).expect("원본 HWPX 파싱 실패");

    let out = serialize_hwpx(&doc1).expect("HWPX 직렬화 실패");
    let doc2 = parse_hwpx(&out).expect("재직렬화 HWPX 파싱 실패");

    // 문단별로 탭 매핑 비교
    let secs1 = &doc1.sections;
    let secs2 = &doc2.sections;
    for (si, (s1, s2)) in secs1.iter().zip(secs2.iter()).enumerate() {
        for (pi, (p1, p2)) in s1.paragraphs.iter().zip(s2.paragraphs.iter()).enumerate() {
            assert_eq!(
                p1.tab_extended.len(),
                p2.tab_extended.len(),
                "섹션{si} 문단{pi}: roundtrip 후 tab_extended 개수 불일치"
            );
            for (ti, (e1, e2)) in p1
                .tab_extended
                .iter()
                .zip(p2.tab_extended.iter())
                .enumerate()
            {
                assert_eq!(
                    e1[0], e2[0],
                    "섹션{si} 문단{pi} 탭{ti}: width {}->{} (issue #1267)",
                    e1[0], e2[0]
                );
                assert_eq!(
                    e1[2], e2[2],
                    "섹션{si} 문단{pi} 탭{ti}: leader/type {:#06x}->{:#06x} (issue #1267)",
                    e1[2], e2[2]
                );
            }
        }
    }
}
