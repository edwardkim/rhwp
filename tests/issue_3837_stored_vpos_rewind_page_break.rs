//! Issue #3837: 저장 `vpos` 가 되돌아가는 자리를 쪽 경계로 인정한다.
//!
//! 한글이 어떤 항목을 다음 쪽 맨 위에 뒀다면 그 항목의 저장 `vpos` 는 쪽 상단 값으로
//! **되돌아간다**. rhwp 는 그 신호를 쪽 경계로 쓰지 않아, 잔여 공간에 들어가기만 하면
//! 현재 쪽 끝에 얹었다. 쪽 총수는 맞는 채 문단 하나만 어긋나므로 쪽수 지표로는 침묵한다.
//!
//! ```text
//! samples/issue3837/stored_vpos_rewind_form.hwp — 응시원서 서식 (한글 4쪽)
//!   1쪽 마지막 항목 pi=12  vpos=41645
//!   그 다음   항목 pi=13  vpos=1000   <- 되돌아감. 한글은 2쪽 맨 위에 뒀다
//!   수정 전   rhwp 는 1쪽 잔여(10.9px 슬랙)에 42.5px 표를 얹었다
//! ```
//!
//! 판별력은 r29 서베이 `PI_MISMATCH` n=1 코호트 66건으로 쟀다 — 어긋난 항목의 36% 가
//! 되돌아감인데, 같은 문서 **다른 쪽**의 마지막 항목은 1,134개 중 2개(0.2%)뿐이다.
//! 쪽이 90% 이상 찼을 때만 인정한다(쪽 중간 인정은 연쇄 발동으로 쪽수를 늘린다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/issue3837/stored_vpos_rewind_form.hwp";
/// 되돌아간 표가 있는 문단. 한글은 2쪽(1-based)에 둔다.
const REWOUND_PARA: &str = "pi=13 ";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn dump_pages() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["dump-pages", sample().to_str().unwrap()])
        .output()
        .expect("rhwp 실행 실패");
    assert!(
        out.status.success(),
        "dump-pages 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 항목이 놓인 쪽(1-based)을 찾는다.
fn page_of(dump: &str, marker: &str) -> Option<usize> {
    dump.split("=== 페이지 ")
        .skip(1)
        .position(|block| block.contains(marker))
        .map(|i| i + 1)
}

/// 되돌아간 항목은 다음 쪽에서 시작해야 한다.
#[test]
fn rewound_item_starts_the_next_page() {
    let dump = dump_pages();
    let page = page_of(&dump, REWOUND_PARA).expect("되돌아간 문단을 어느 쪽에서도 못 찾았다");

    assert_eq!(
        page, 2,
        "되돌아간 표({REWOUND_PARA})가 {page}쪽에 놓였다 — 저장 vpos 는 1000 으로 \n         \
         되돌아가 한글이 2쪽 맨 위에 뒀음을 말한다. 잔여 공간에 들어간다는 이유로 \n         \
         1쪽 끝에 얹으면 안 된다."
    );
}

/// 표본이 계약을 시험하는 형태인지 못박는다.
///
/// 되돌아감은 **직전 항목의 vpos 가 충분히 큰 자리**에서만 신호가 된다. 표본의 1쪽이
/// 그 형태가 아니면 이 테스트는 수정 전에도 통과한다.
#[test]
fn the_fixture_has_a_large_preceding_vpos() {
    let dump = dump_pages();
    let prior = dump
        .lines()
        .find(|l| l.contains("pi=12 ") && l.contains("Table"))
        .expect("직전 표(pi=12)를 못 찾았다");
    let vpos: i64 = prior
        .split("vpos=")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse().ok())
        .expect("직전 항목 vpos 파싱 실패");

    assert!(
        vpos > 5000,
        "직전 항목의 저장 vpos 가 {vpos} 다 — 5000 이하면 되돌아감 가드가 애초에 \n         \
         발동하지 않아 계약을 시험하지 못한다."
    );
}
