//! Issue #1880: convert-HWP 자리차지 표 `host_spacing.before` 비대칭 — pagination 자기정합.
//!
//! `format_table`(typeset.rs)의 자리차지(비-TAC TopAndBottom) 판정이 원시
//! `table.attr` 비트((attr>>21)&7==1)를 읽었는데, HWPX 파서는 `table.attr` 에
//! bit0 만 미러(section.rs)하고 HWP5 파서는 원시 attr 전체(control.rs)를 채워,
//! 같은 IR 의 convert-HWP 재파스에서만 `host_spacing.before` 가 spacing_before 를
//! 잃었다(2780073 pi=2/4/6 host_before 6.7↔0.0px → defer 가드 플립 → PI_MOVED).
//! 의미 필드(`common.text_wrap`) + `is_hwpx_source` 게이트로 교체해 해소.
//!
//! fixtures (정부 행정규칙, issue1770 선례):
//! - `issue1880_takeplace_host_before.hwpx` (2780073, 유기식품 인증기관 지정기준):
//!   visible-host 자리차지 표 스택 — 수정 전 pi=4 defer 가드 플립.
//! - `issue1880_takeplace_oracle_p13.hwpx` (3075729, USB 저장매체 보안관리지침):
//!   한컴 2022 oracle 확정 문서 — sec1 pi=121 heading("휴대용 저장매체 불용처리
//!   확인서")이 **p13**(한컴이 원본 HWPX·convert-HWP 모두 p13 렌더, 이슈 #1880
//!   표). 수정 전 convert-HWP 렌더만 p12 로 어긋났다.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn load(path: &str) -> rhwp::wasm_api::HwpDocument {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {path}: {e:?}"))
}

/// dump-pages 출력에서 `(section, pi) → global page index` 매핑을 뽑는다.
/// 같은 (section, pi) 가 여러 페이지에 걸치면(분할 표) 첫 등장 페이지를 취한다.
fn pi_page_map(dump: &str) -> BTreeMap<(u32, u32), u32> {
    let mut map = BTreeMap::new();
    let mut cur: Option<(u32, u32)> = None; // (section, global_idx)
    for line in dump.lines() {
        if let Some(rest) = line.strip_prefix("=== 페이지 ") {
            let global_idx = rest
                .split("global_idx=")
                .nth(1)
                .and_then(|s| s.split([',', ')']).next())
                .and_then(|s| s.trim().parse::<u32>().ok());
            let section = rest
                .split("section=")
                .nth(1)
                .and_then(|s| s.split([',', ')']).next())
                .and_then(|s| s.trim().parse::<u32>().ok());
            cur = match (section, global_idx) {
                (Some(s), Some(g)) => Some((s, g)),
                _ => None,
            };
            continue;
        }
        let Some((sec, page)) = cur else { continue };
        if let Some(pi_str) = line.split("pi=").nth(1) {
            if let Ok(pi) = pi_str
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or("")
                .parse::<u32>()
            {
                map.entry((sec, pi)).or_insert(page);
            }
        }
    }
    map
}

/// 자리차지 표 스택 문서: HWPX 렌더와 convert-HWP 재파스 렌더의
/// (section, pi)→page 배치가 완전히 일치해야 한다 (자기정합).
#[test]
fn takeplace_host_before_pagination_self_consistent() {
    let mut hwpx = load("samples/issue1880_takeplace_host_before.hwpx");
    let hwpx_dump = hwpx.dump_page_items(None);

    let hwp_bytes = hwpx.export_hwp_with_adapter().expect("convert");
    let mut conv = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes).expect("reparse");
    let conv_dump = conv.dump_page_items(None);

    let a = pi_page_map(&hwpx_dump);
    let b = pi_page_map(&conv_dump);
    assert!(!a.is_empty(), "#1880 전제: dump-pages 파싱 결과가 비어있음");
    let moved: Vec<String> = a
        .iter()
        .filter(|(k, v)| b.get(k) != Some(v))
        .map(|((s, pi), pg)| format!("s{s}:pi={pi} hwpx_p={pg} conv_p={:?}", b.get(&(*s, *pi))))
        .collect();
    assert!(
        moved.is_empty(),
        "#1880: convert-HWP 재파스 배치가 HWPX 와 다름 (host_before 비대칭): {:?}",
        moved
    );
    assert_eq!(
        conv.page_count(),
        hwpx.page_count(),
        "#1880: 총 쪽수 불일치"
    );
}

/// 한컴 oracle 확정 문서: heading(sec1 pi=121)이 양 경로 모두 global_idx=12
/// (13쪽째)에 있어야 한다. 수정 전 convert-HWP 는 11(12쪽째)로 당겨졌다.
#[test]
fn oracle_3075729_heading_on_page13_both_paths() {
    let mut hwpx = load("samples/issue1880_takeplace_oracle_p13.hwpx");
    let hwpx_map = pi_page_map(&hwpx.dump_page_items(None));
    assert_eq!(
        hwpx_map.get(&(1, 121)).copied(),
        Some(12),
        "#1880 전제: HWPX 렌더 heading 이 global_idx=12(13쪽째, 한컴 oracle)"
    );

    let hwp_bytes = hwpx.export_hwp_with_adapter().expect("convert");
    let mut conv = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes).expect("reparse");
    let conv_map = pi_page_map(&conv.dump_page_items(None));
    assert_eq!(
        conv_map.get(&(1, 121)).copied(),
        Some(12),
        "#1880: convert-HWP 렌더 heading 이 한컴 oracle(13쪽째)에서 이탈"
    );
}
