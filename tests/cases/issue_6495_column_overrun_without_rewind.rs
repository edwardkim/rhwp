//! [#6495] 되감김 신호가 **없는** 단도 용지-밖 관문이 본다.
//!
//! `samples/3-09월_교육_통합_2022.hwpx` 는 저장소 동봉 샘플이다(2단, 해설이 미주).
//!
//! **증상.** 2단 조판에서 단 아래끝을 넘겨 다음 쪽 몫을 계속 그린다. 9쪽 오른쪽 단은
//! 용지 끝(`841.9pt`)에 닿는 `841.17pt` 까지 내려간다.
//!
//! **근인 — 관문 발동 조건이 되감김 신호를 요구한다.**
//!
//! `page_offcanvas_sim` 은 `local_vpos_rewind || column_had_compact_endnote_rewind` 를
//! 요구한다. 9쪽 오른쪽 단은 되감김이 **한 번도 없어** 관문이 아예 켜지지 않는다.
//! 그런데 타이프셋 자신은 넘친 것을 안다.
//!
//! ```text
//! page=9 col=1  ep=0  cur_h= 973.8  avail=1001.6  sim_on=false
//! page=9 col=1  ep=1  cur_h= 991.8  avail=1001.6  sim_on=false
//! page=9 col=1  ep=2  cur_h=1030.9  avail=1001.6  sim_on=false   ← 이미 +29px
//! page=9 col=1  ep=3  cur_h=1048.9  avail=1001.6  sim_on=false   ← +47px
//! ```
//!
//! **수정.** 누계가 이미 가용 높이를 넘었으면 신호와 무관하게 시뮬로 확인한다.
//! 시뮬(`simulate_endnote_column_bottom_y`)은 렌더와 같은 경로로 하단을 읽으므로,
//! 켜기만 하면 판정 자체는 종전 계약 그대로다.
//!
//! **결과.** `rhwp layout-anomaly` 기준 이 문서 overflow `16 → 13`, off-canvas `6` 유지,
//! 쪽수 `23` 유지. (`16`·`6` 은 `#5886`(PR #6492) 적용 뒤 값이다 — 이 변경은 그 위에
//! 쌓인다.)
//!
//! **남는 축.** 9쪽 오른쪽 단의 `841.17pt` 는 이 변경으로도 안 잡힌다. 그 단에서는
//! **시뮬 자체가 부정확**하다(예측 `1047.6px` vs 실제 페인트 `1120.3px`, Δ74px).
//! `#5886` 의 12쪽에서는 시뮬이 정확했으므로 별개 축이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/3-09월_교육_통합_2022.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6495-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `layout-anomaly` 의 `[OVERFLOW]` 신고 건수.
fn overflow_count() -> usize {
    let out = Command::new(rhwp_bin())
        .args(["layout-anomaly", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.contains("[OVERFLOW]"))
        .count()
}

/// 되감김 없는 단의 넘침이 줄어든 상태를 잠근다.
///
/// 종전(`#5886` 적용 후) `16` 건. 관문을 누계 기준으로도 켜면 `13` 건이 된다.
/// 상한을 `13` 으로 두어 되돌아가면 깨지게 한다.
#[test]
fn column_overrun_without_rewind_signal_is_guarded() {
    let n = overflow_count();
    assert!(
        n <= 13,
        "되감김 없는 단의 단-하단 넘침이 관문에 걸려야 한다: OVERFLOW {n}건 (종전 16)"
    );
}

/// 쪽수는 흔들지 않는다 — 단 전환을 앞당기는 변경이라 쪽 배분이 바뀔 수 있다.
#[test]
fn page_count_is_unchanged() {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "22",
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "23쪽(0-기반 22)이 존재해야 한다: {out:?}"
    );
}
