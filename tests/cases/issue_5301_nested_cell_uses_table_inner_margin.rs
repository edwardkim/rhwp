//! [#5301] `aim=false` 중첩 칸은 표의 안 여백을 쓴다 — 저장된 셀 여백이 아니라.
//!
//! `samples/issue1891/76076_regulatory_analysis.hwpx` 는 저장소 안 규제영향분석서다
//! (확장자와 달리 **HWP5 이진**). 같은 폴더에 한글 2024 정답지 PDF 가 있다.
//!
//! **증상.** 문서 전체에서 세 글자(`로`·`예`·`상`)가 어디에도 그려지지 않는다.
//! 66쪽 근거설명 칸의 마지막 줄이 `…없을 것으` 에서 끊기고 `로 예상` 이 사라진다.
//! 쪽수는 82 로 정답지와 같아 쪽 경계 이동으로는 설명되지 않는다.
//!
//! **근인 — 여백 축 선택이 뒤집혀 있었다.**
//!
//! ```text
//! 바깥 칸[9]  w=38245  pad=(510,510,…)
//!   내부표    pad=(0,0,141,141)          ← 좌우 안 여백 0
//!     칸[0]   w=36572  pad=(510,510,…)  aim=false
//! ```
//!
//! `aim=false` 면 표 여백(0)을 써야 한다. 그런데 `Cell::use_cell_padding_axis` 에
//! "중첩 비글자표는 셀의 작은 저장 여백을 한컴이 쓴다"는 예외(`#2308 p34`)가 있어
//! `510` 이 적용됐고, 안쪽 폭이 `36572 → 35552 HWPUNIT`(10.2pt) 좁아졌다. 좁아진 만큼
//! 줄이 하나 더 생기고 그 초과 줄이 조각 용량을 넘어 꼬리가 소실된다.
//!
//! **그 예외의 근거를 오라클로 반증했다.** 예외가 인용한 문서가 바로 이 문서다.
//! 한글 2024 가 그린 글자 상자는 34쪽·66쪽 **둘 다** `156.96..522.60pt = 365.6pt` 로
//! **칸 폭 36572HU(365.72pt) 전부**다 — 안 여백 0 이다.
//!
//! | | 종전 | **수정 후** | 한글 2024 |
//! |---|---:|---:|---:|
//! | 66쪽 글자 상자 | 163.95..519.5 (355.5pt) | **158.85..526.5 (367pt)** | 156.96..522.60 (365.6pt) |
//! | 문서 전체 소실 문자 | **3** | **0** | 0 |
//! | 쪽수 | 82 | 82 | 82 |
//!
//! 주석이 함께 인용하던 `KTX 목차`·`exam_kor 보기 박스` 는 저장소 오라클
//! (`pdf/KTX-2022.pdf`·`pdf/exam_kor-2022.pdf`) 대비 **수정 전후가 완전히 동일**하다 —
//! 그 근거는 이미 무효였다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::table::Cell;
use rhwp::model::Padding;

/// `aim=false` + 표 여백 0 + 셀 저장 여백 510 → **표 여백(0)** 을 쓴다.
///
/// 종전에는 호출부가 "중첩 비글자표" 문맥이면 셀의 510 을 썼고, 그것이 76076 66쪽에서
/// 글자를 소실시켰다.
#[test]
fn nested_cell_with_zero_table_margin_ignores_stored_cell_margin() {
    let mut cell = Cell::default();
    cell.apply_inner_margin = false;
    cell.padding = Padding {
        left: 510,
        right: 510,
        top: 141,
        bottom: 141,
    };
    assert!(
        !cell.use_cell_padding_axis(cell.padding.left, 0),
        "aim=false 셀은 표의 좌우 안 여백(0)을 써야 한다"
    );
    assert!(
        !cell.use_cell_padding_axis(cell.padding.right, 0),
        "aim=false 셀은 표의 좌우 안 여백(0)을 써야 한다"
    );
}

/// `aim=true` 계약은 그대로다 — 사용자가 지정한 셀 고유 여백은 0 이어도 존중한다(#2070).
#[test]
fn explicit_inner_margin_cell_still_wins() {
    let mut cell = Cell::default();
    cell.apply_inner_margin = true;
    cell.padding = Padding {
        left: 0,
        right: 0,
        top: 141,
        bottom: 141,
    };
    assert!(
        cell.use_cell_padding_axis(0, 510),
        "aim=true 의 0 은 사용자가 지정한 값이라 표 폴백으로 덮으면 안 된다"
    );
    assert!(
        !cell.use_cell_padding_axis(-1, 510),
        "음수는 결측 센티널이라 표 폴백을 유지한다"
    );
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1891/76076_regulatory_analysis.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-5301-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 66쪽 근거설명 칸의 마지막 줄이 끝까지 **그려진다**.
///
/// 종전에는 안쪽 폭이 10.2pt 좁아 줄이 하나 더 생겼고, 그 초과 줄이 조각 용량을 넘어
/// `로 예상` 세 글자가 문서 어디에도 남지 않았다(다음 쪽에도 없다 — 진짜 소실).
/// 한글 2024 정답지는 `… 없을 것으로 예상` 으로 끝난다.
///
/// **판정은 render tree 로 한다** — `export-text` 는 IR 문단 텍스트를 내보내므로
/// 조판에서 떨어진 글자도 그대로 보여 이 결함에 대해 공허하다.
#[test]
fn page66_rationale_cell_keeps_its_sentence_tail() {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "65",
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .expect("66쪽 render tree JSON 이 없다");
    let json: String = std::fs::read_to_string(path)
        .unwrap()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert!(
        json.contains("없을것으로예상"),
        "66쪽 근거설명 칸의 마지막 줄이 `… 없을 것으로 예상` 까지 그려져야 한다"
    );
}
