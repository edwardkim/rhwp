//! [Issue #5941 축 B] `#5921` 의 near-top 리셋 완화가 **이미 찬 쪽**에도 걸려 저장 쪽
//! 경계를 지우던 회귀의 가드.
//!
//! `#5921`(`12074fbea`)은 `native_near_top_reset` 에 "이번 쪽 잔여에 들어가면 리셋을
//! 버린다" 를 더했다. 그 완화는 **리셋을 지켰을 때 거의 빈 쪽이 남는** 형상에서 나왔다.
//! 그런데 쪽이 이미 차 있어도 걸려, 작성 엔진이 기록한 쪽 경계를 지운다.
//!
//! 리셋 지점의 쪽 채움 실측:
//!
//! ```text
//!   #5921 픽스처 `neartop_reset_sb2500`   items= 1   채움  2%   ← 완화 대상
//!   1480000-201900698                     items= 3   채움 27%
//!                                         items= 4   채움 46%
//!                                         items=15   채움 74%   ← 완화하면 안 됨
//! ```
//!
//! `1480000-201900698` 은 그 완화로 리셋 3개가 지워져 **202 → 200** 이 됐다. 한/글 2024
//! 는 **205** 이므로 거리가 3 → 5 로 멀어진 회귀다(`#5941` 축 B bisect 로 커밋 특정).
//!
//! ⚠ 이 시험은 **양쪽을 함께** 잠근다 — `#5921` 의 원 픽스처는 계속 1쪽이어야 한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/prism_downloads/기후에너지환경부/
///  1480000-201900698_D0150004-1-001_자생생물유래독성물질의유용성탐색4차년도_최종보고_프리즘 제출.hwp`
///
/// ⚠ `.hwp` 를 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/` 전체를 스윕해
/// 무관한 직렬화 발산을 끌고 온다. `RHWP_ISSUE5941_SAMPLE` 로 경로를 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE5941_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(
            r"C:\Users\planet\hwpdocs_10k_share",
            r"\prism_downloads\기후에너지환경부"
        ),
        concat!(r"D:\hwpdocs_10k_share", r"\prism_downloads\기후에너지환경부"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("1480000-201900698") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// 이미 찬 쪽의 저장 near-top 리셋은 살아 있어야 한다 — 지우면 202 → 200 이 된다.
#[test]
fn stored_neartop_reset_survives_on_a_filled_page() {
    let Some(bytes) = sample() else {
        return;
    };
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let pages = doc.page_count();
    assert_eq!(
        pages, 202,
        "이미 찬 쪽의 저장 near-top 리셋을 지우면 202 → 200 이 된다 — #5941 축 B 회귀 \
         (한/글 2024 는 205). got {pages}"
    );
}

/// `#5921` 의 원 계약은 그대로 — 거의 빈 쪽에서는 완화가 걸려 1쪽이어야 한다.
#[test]
fn issue_5921_empty_page_relaxation_still_applies() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/task2136/neartop_reset_sb2500.hwpx");
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let doc = HwpDocument::from_bytes(&bytes).expect("parse fixture");
    assert_eq!(
        doc.page_count(),
        1,
        "거의 빈 쪽(채움 2%)에서는 #5921 완화가 그대로 걸려 1쪽이어야 한다"
    );
}
