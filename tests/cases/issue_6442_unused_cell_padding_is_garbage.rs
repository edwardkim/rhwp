//! [#6442] "안 여백 지정"이 꺼진 셀의 저장 padding 은 읽지 않는다.
//!
//! `samples/issue6442/access_pass_form.hwp` 는 해양수산부 「대산항 출입절차 예규」
//! [별지 1] 출입증 서식(법제처 국가법령정보센터 2316361, 61KB) 원본이다.
//!
//! **증상.** 2쪽 오른쪽 【뒷면/임시】 칸이 통째로 비어 그려진다. 같은 구조인 왼쪽
//! 【뒷면/상시】와 3쪽 두 칸은 정상이다.
//!
//! **근인 — 한글이 읽지 않는 필드를 rhwp 가 읽었다.**
//!
//! LIST_HEADER 의 "안 여백 지정"(`width_ref` bit0, `apply_inner_margin`) 이 **꺼진**
//! 셀에서 한글은 `padding` 필드를 무시하고 표 기본 여백을 쓴다. 무시되는 필드이므로
//! 파일에 무엇이 들어 있어도 무방하고, 이 문서의 중첩표가 실제로 좌표성 쓰레기를
//! 담고 있다.
//!
//! | 중첩표 | `aim` | `pad.top` | `pad.bottom` | 셀 선언 높이 |
//! |---|---|---|---|---|
//! | 【뒷면/상시】 | `true` | 0 | 0 | 683HU (9.1px) |
//! | 【뒷면/임시】 | **`false`** | **29693HU** | **10433HU** | 683HU (9.1px) |
//!
//! 그런데 행 높이 산정의 `raw_pad_v`(#6030) 와 높이 측정의 세 지점이 `apply_inner_margin`
//! 관문을 거치지 않고 `cell.padding` 을 **원시로** 더했다. `line_based` 5.3px 짜리 행이
//! `5.3 + 535.4 = 540.3px` 로 부풀어, 15행 표의 행 높이 합이 선언 366.4px 대비 **4400px**
//! 이 됐다. 표는 칸에 들어갈 수 없어 내용이 통째로 사라진다.
//!
//! ```text
//! 【뒷면/상시】 row_heights = [ 9.1,  36.0,  9.1,  24.6, 137.8, ...]   합 ≈ 366
//! 【뒷면/임시】 row_heights = [540.3, 543.0, 534.1, 461.7, 141.6, ...]  합 ≈ 4400
//! ```
//!
//! **수정.** `Cell::stored_vertical_padding_hu()` 를 단일 출처로 두고, aim=false
//! 이면서 (a) 값이 자기 셀 높이를 **엄격히 넘고** (b) 안 여백으로는 설명이 안 되는
//! 절대 크기(2cm↑)일 때만 0 으로 본다.
//!
//! **⚠ 값만 보고 전부 버리면 안 된다.** aim=false 저장값에 의존하는 조판 경로가
//! 여럿이라 전부 버리면 10건이 회귀했고, `>=` 로 두면 `total = height = 282`
//! (141+141 여백에 내용 높이 0)인 정상 셀 2587개까지 잡았다(#3637).
//!
//! 게다가 **같은 형상의 쓰레기값을 가진 정상 문서가 있다** — #1921 의 59043 은
//! `pad=(23812,10930)`, `height=282` 로 이 문서와 판박이인데 한글 실측 핀에 맞는
//! 상태다. 그래서 값이 아니라 **결과**로 가른다: 그 값이 표를 자기 선언 높이의
//! 2배 밖으로 밀어낸 **다행 표**에서만 억제하고 한 번 다시 잰다
//! (`unused_padding_overflows_declared_table`). 이 문서는 선언 366.4px 대비 행 합
//! 4225px = **11.5배**다.
//!
//! **오라클 — 한글 2022.** 문서 편집 버전은 `appVersion 11,0,0,4585` = 한글 2020
//! 으로 이 장비에 없어 가장 가까운 설치본을 썼다. 한글 PDF 쪽별 출현 수가
//! 수정 후 rhwp 와 **쪽 단위로 정확히 일치**한다.
//!
//! | 쪽 | 한글 2022 | 수정 전 | **수정 후** |
//! |---|---|---|---|
//! | 1 | 준수사항 4 · 발급기관장 0 | 4 · 0 | 4 · 0 |
//! | 2 | 준수사항 2 · 발급기관장 2 | **1 · 1** | **2 · 2** |
//! | 3 | 준수사항 2 · 발급기관장 2 | 2 · 2 | 2 · 2 |
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6442/access_pass_form.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6442-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 쪽별 render tree JSON (1-기반 쪽 번호).
fn render_tree(page1_based: usize) -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    std::fs::read_to_string(&paths[page1_based - 1]).unwrap()
}

/// 2쪽 두 장의 【뒷면】 카드가 **둘 다** 내용을 갖는다.
///
/// 종전엔 오른쪽 카드가 통째로 비어 각 문구가 한 번씩만 나왔다.
#[test]
fn both_back_side_cards_on_page2_carry_their_content() {
    let json = render_tree(2);
    let compliance = json.matches("준수사항").count();
    let issuer = json.matches("발급기관장").count();
    assert_eq!(
        (compliance, issuer),
        (2, 2),
        "2쪽 【뒷면/상시】·【뒷면/임시】 두 카드 모두 내용을 가져야 한다 \
         (한글 2022 실측 준수사항 2 · 발급기관장 2, 종전 rhwp 1 · 1)"
    );
}

/// 3쪽은 같은 구조의 통제군 — 종전에도 정상이었고 그대로여야 한다.
#[test]
fn page3_control_group_is_unchanged() {
    let json = render_tree(3);
    assert_eq!(
        (
            json.matches("준수사항").count(),
            json.matches("발급기관장").count()
        ),
        (2, 2),
        "3쪽(통제군)은 종전에도 정상이었다"
    );
}

/// "안 여백 지정"이 꺼진 셀의 저장 padding 은 높이 산정에 들어가지 않는다.
///
/// 근인을 직접 잠근다 — 위 두 테스트가 조판 경로 어디서든 상쇄로 통과하는 것을 막는다.
#[test]
fn unused_inner_margin_field_is_not_charged() {
    use rhwp::model::table::Cell;
    use rhwp::model::Padding;

    let pad = |top, bottom| Padding {
        left: 0,
        right: 0,
        top,
        bottom,
    };

    // 이 문서의 실제 값 — aim=false 인데 padding 이 셀 선언 높이(683HU)를 훨씬 넘는다.
    let mut garbage = Cell::default();
    garbage.set_apply_inner_margin(false);
    garbage.padding = pad(29693, 10433);
    garbage.height = 683;
    assert_eq!(
        garbage.stored_vertical_padding_hu(),
        0,
        "쓰이지 않는 필드에 담긴, 셀 높이를 넘는 값은 높이에 실리면 안 된다"
    );

    // aim=true 는 비정상 크기여도 종전대로 저장값을 쓴다 — 이 규칙은 **쓰이지 않는**
    // 필드만 겨냥한다. aim=true 의 방어는 `vertical_padding_is_abnormal` 소비처 몫이다.
    let mut applied = Cell::default();
    applied.set_apply_inner_margin(true);
    applied.padding = pad(29693, 10433);
    applied.height = 683;
    assert_eq!(
        applied.stored_vertical_padding_hu(),
        40126,
        "aim=true 셀의 안 여백은 그대로 실린다"
    );

    // aim=false 여도 **셀 높이 안에 들어가는** 저장 여백은 종전대로 살린다 —
    // rhwp 조판이 의존하는 경로가 여럿이라(#2195 중첩 비글자표 등) 전부 버리면 회귀한다.
    // 특히 `total == height` 는 141+141 여백에 내용 높이 0 인 정상 셀이다(#3637).
    let mut legacy = Cell::default();
    legacy.set_apply_inner_margin(false);
    legacy.padding = pad(510, 510);
    legacy.height = 2697;
    assert_eq!(
        legacy.stored_vertical_padding_hu(),
        1020,
        "셀 높이 안에 들어가는 aim=false 저장 여백은 그대로 유지된다"
    );

    // 선언 높이가 없으면 판정 근거가 없으니 종전대로 둔다.
    let mut unknown = Cell::default();
    unknown.set_apply_inner_margin(false);
    unknown.padding = pad(510, 510);
    unknown.height = 0;
    assert_eq!(
        unknown.stored_vertical_padding_hu(),
        1020,
        "선언 높이가 없는 셀은 판정하지 않는다"
    );

    // 같은 크기여도(엄격 초과 아님) 살린다 — #3637 의 2587개 셀이 이 형상이다.
    let mut equal = Cell::default();
    equal.set_apply_inner_margin(false);
    equal.padding = pad(3000, 3000);
    equal.height = 6000;
    assert_eq!(
        equal.stored_vertical_padding_hu(),
        6000,
        "total == height 는 엄격 초과가 아니라 살린다"
    );

    // 셀 높이를 넘어도 절대 크기가 안 여백 범위면 살린다.
    let mut plausible = Cell::default();
    plausible.set_apply_inner_margin(false);
    plausible.padding = pad(1417, 1417);
    plausible.height = 2000;
    assert_eq!(
        plausible.stored_vertical_padding_hu(),
        2834,
        "안 여백으로 설명되는 크기는 셀 높이를 넘어도 살린다"
    );
}
