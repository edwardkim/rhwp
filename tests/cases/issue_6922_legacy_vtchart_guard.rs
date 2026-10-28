//! [#6922] `w1 == w2` 가드가 레거시 한/글 차트를 걸러 낸다.
//!
//! `probe_ole_chart_contents` 는 레거시 차트 판정에 앞 16바이트 네 워드 중
//! **둘째·셋째가 같을 것**을 요구했다. 그 조건이 왜 성립해야 하는지는 근거로 적혀 있지
//! 않았고, 실측하면 두 값은 **서로 다른 두 크기값**이다.
//!
//! ```text
//!   148735526 3쪽 차트 Contents 앞 16바이트
//!     00 00 01 00 | 65 3a 00 00 | d4 22 00 00 | 60 00 00 00
//!     w0 = 0x00010000  ✔        w1 = 0x00003a65
//!     w3 = 0x00000060  ✔        w2 = 0x000022d4   ← w1 != w2 라 떨어졌다
//!   offset 46 "VtChart\0" · offset 66 "VtDataGrid\0"  — 진짜 레거시 차트다
//! ```
//!
//! 남는 술어는 그대로다 — `w0`, `w3`(개체 시작 오프셋)가 스트림 안의 합리적 값, 그리고
//! **`VtDataGrid` 표지**. 코퍼스 10,000건의 OLE `Contents` 1,046개를 전수로 재면 표지를
//! 가진 것이 75개이고 그 **75개 전부**가 완화된 술어를 통과한다(현행 47 → 뒤집힘 28).
//! 표지가 없는 971개는 하나도 들어오지 않으므로 `#5724`·`#5725`(차트 아닌 것을 차트로
//! 본 갈래)의 반대 방향은 생기지 않는다.
//!
//! ⚠ 이 수정만으로는 차트가 그려지지 않는다 — 판정을 통과한 뒤 그리드 스캔이
//! `NumberCellCountMismatch` 로 막힌다(별개 축, 이슈에 실측을 적었다). 이 시험은 **판정
//! 경계**만 잠근다.

use rhwp::ole_chart::probe_ole_chart_contents;

/// `VtChart` · `VtDataGrid` 표지를 가진 최소 레거시 차트 `Contents`.
///
/// `w1 != w2` 로 두어 종전 가드가 떨어뜨리던 형상을 그대로 만든다.
fn legacy_contents(w1: u32, w2: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&0x0001_0000u32.to_le_bytes()); // w0
    b.extend_from_slice(&w1.to_le_bytes());
    b.extend_from_slice(&w2.to_le_bytes());
    b.extend_from_slice(&0x0000_0060u32.to_le_bytes()); // w3 — 개체 시작 오프셋
    b.extend_from_slice(b"VtChart\0");
    while b.len() < 0x40 {
        b.push(0);
    }
    b.extend_from_slice(b"VtDataGrid\0");
    while b.len() < 0x200 {
        b.push(0);
    }
    b
}

#[test]
fn issue_6922_legacy_chart_is_recognized_when_w1_differs_from_w2() {
    // 실측값 그대로 — 148735526 3쪽 차트.
    let probe = probe_ole_chart_contents(&legacy_contents(0x0000_3a65, 0x0000_22d4))
        .expect("probe 는 성공해야 한다");
    assert!(
        probe.has_vt_chart_marker && probe.has_vt_data_grid_marker,
        "표지 두 개를 다 읽어야 한다"
    );
    assert_eq!(
        probe.legacy_chart_object_start,
        Some(0x60),
        "개체 시작 오프셋은 w3 다"
    );
    assert!(
        probe.likely_legacy_hwp_chart_contents,
        "w1 != w2 여도 레거시 한/글 차트로 판정해야 한다"
    );
}

#[test]
fn issue_6922_equal_words_case_is_unchanged() {
    // **음성 대조 ①** — 종전에 통과하던 `w1 == w2` 형상은 그대로 통과한다.
    let probe = probe_ole_chart_contents(&legacy_contents(0x0000_1234, 0x0000_1234))
        .expect("probe 는 성공해야 한다");
    assert!(
        probe.likely_legacy_hwp_chart_contents,
        "종전 통과 형상은 불변이어야 한다"
    );
}

#[test]
fn issue_6922_without_the_data_grid_marker_nothing_is_admitted() {
    // **음성 대조 ②** — 표지가 없으면 들어오지 않는다. `#5724`(StaticMetafile)·
    // `#5725`(수식 OLE)가 걸렸던 반대 방향을 이 가드가 계속 막는다.
    let mut bytes = legacy_contents(0x0000_3a65, 0x0000_22d4);
    // "VtDataGrid\0" 표지를 지운다.
    if let Some(at) = bytes
        .windows(b"VtDataGrid\0".len())
        .position(|w| w == b"VtDataGrid\0")
    {
        for slot in bytes[at..at + b"VtDataGrid\0".len()].iter_mut() {
            *slot = 0;
        }
    }
    let probe = probe_ole_chart_contents(&bytes).expect("probe 는 성공해야 한다");
    assert!(!probe.has_vt_data_grid_marker, "표지를 지웠다");
    assert!(
        !probe.likely_legacy_hwp_chart_contents,
        "표지가 없으면 레거시 차트로 보지 않는다"
    );
}
