//! [#6171] 한양견고딕/한양견명조가 설치 face 를 건너뛰고 generic 으로 떨어지지 않게 잡는다.
//!
//! 3146683 1쪽 `『별표 7』` 의 `별표` 획이 한글 2020 출력보다 가늘었던 원인은 조판이
//! 아니라 face 선택이었다. SVG `font-family` 사슬을 실제로 만드는 것은
//! `renderer::render_font_family_chain`(내부 `installed_render_font_aliases`)이며,
//! 견고딕/견명조 arm 이 없으면 사슬이 `'한양견고딕'` 하나 뒤에 곧바로 generic
//! (고딕 계열의 첫 항목이 `Malgun Gothic`)으로 떨어져, `HY견고딕` 이 설치된 호스트에서도
//! Malgun Regular 로 그려졌다.
//!
//! `svg.rs` 의 `font_local_aliases`/`known_font_filenames` 는 그 모듈의 단위 테스트가
//! 지키고, SVG golden 두 건(issue-617/issue-677)은 사슬 문자열 전체를 통째로 고정한다.
//! 여기서는 golden 문서와 무관하게 (1) 사슬 앞머리와 (2) 배치 폭을 정하는 메트릭 별칭이
//! 같은 face 계열로 이어지는지를 직접 고정한다.

use rhwp::renderer::font_metrics_data::find_metric;
use rhwp::renderer::{render_font_family_chain, render_font_family_chain_for_weight};

/// 원 face 바로 뒤에 설치 face 이름이 순서대로 와야 한다.
///
/// generic 폴백은 계열마다 다르므로(고딕=`Malgun Gothic`…, 명조=`Batang`…) 사슬 전체가
/// 아니라 앞머리만 계약으로 잡는다 — 설치 face 가 어떤 generic 보다도 앞선다는 뜻이다.
fn assert_chain_head(chain: &str, expected: &[&str]) {
    let head: String = expected
        .iter()
        .map(|face| format!("'{face}'"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(
        chain.starts_with(&format!("{head},")),
        "사슬 앞머리가 `{head}` 여야 한다: {chain}"
    );
}

#[test]
fn hanyang_gyeongothic_chain_reaches_installed_hy_face() {
    assert_chain_head(
        &render_font_family_chain("한양견고딕"),
        &["한양견고딕", "HY견고딕", "HYGothic-Extra"],
    );
}

#[test]
fn hanyang_gyeonmyeongjo_chain_reaches_installed_hy_face() {
    assert_chain_head(
        &render_font_family_chain("한양견명조"),
        &["한양견명조", "HY견명조", "HYMyeongJo-Extra"],
    );
}

#[test]
fn bold_chain_keeps_gyeongothic_alias() {
    // bold 경로는 ExtraLight 를 걸러내는 별도 함수라 별칭 삽입이 빠질 수 있다.
    assert_chain_head(
        &render_font_family_chain_for_weight("한양견고딕", true),
        &["한양견고딕", "HY견고딕", "HYGothic-Extra"],
    );
}

#[test]
fn gyeongothic_metric_alias_stays_paired_with_render_face() {
    // 사슬만 고치고 메트릭 별칭을 두면 화면 face 와 배치 폭이 어긋난다.
    // legacy 한양* 이름은 [#2430] 한글 COM 실측 ASCII 를 덧씌운 overlay 를 쓰고,
    // 한글 음절 폭은 설치 face(HY*)의 메트릭을 그대로 공유한다. 두 축이 갈라지면
    // 이 테스트가 먼저 깨진다.
    let pairs = [
        (
            "한양견고딕",
            "HanyangKyunGothic",
            "HY견고딕",
            "HYGothic-Extra",
        ),
        (
            "한양견명조",
            "HanyangKyunMyeongJo",
            "HY견명조",
            "HYMyeongJo-Extra",
        ),
    ];
    for (legacy_name, legacy_metric, installed_name, installed_metric) in pairs {
        let legacy = find_metric(legacy_name, false, false)
            .unwrap_or_else(|| panic!("`{legacy_name}` 메트릭 별칭이 해소되지 않는다"));
        assert_eq!(legacy.metric.name, legacy_metric);
        let installed = find_metric(installed_name, false, false)
            .unwrap_or_else(|| panic!("`{installed_name}` 메트릭 별칭이 해소되지 않는다"));
        assert_eq!(installed.metric.name, installed_metric);
        assert_eq!(
            legacy.metric.em_size, installed.metric.em_size,
            "`{legacy_name}` overlay 와 `{installed_name}` 의 em 이 달라지면 폭이 갈라진다"
        );
        for ch in ['별', '표', '가', '힣'] {
            assert_eq!(
                legacy.metric.get_width(ch),
                installed.metric.get_width(ch),
                "`{legacy_name}` 의 한글 폭은 `{installed_metric}` 과 같아야 한다 ({ch})"
            );
        }
    }
}
