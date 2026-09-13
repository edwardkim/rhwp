//! [#6266] 양식 개체(PushButton)가 원본 배치대로 놓인다.
//!
//! `samples/issue6266/seizure_list_form_button.hwp`(HWP3, 1쪽)의 서식 일련번호
//! `- 581-13 -` 은 **용지 기준 · 가로 가운데 · 세로 아래** 로 배치된 양식 개체다.
//!
//! 종전에는 `FormObject` 에 배치 필드가 하나도 없었고 HWP3 파서가 원본 개체의
//! `common`(기준·정렬·오프셋·바깥여백)에서 width/height 만 옮겼다. 그래서 렌더러가
//! 이 개체를 **인라인 말고는 놓을 수 없었고**, 쪽 하단에 있어야 할 개체가 1쪽 제목
//! `압 류 목 록` 오른쪽에 그려졌다. 개체가 줄 폭을 먹는 바람에 가운데 정렬 제목까지
//! 왼쪽으로 밀렸다.
//!
//! 한글 2024 실측(COM SaveAs PDF, producer=Hancom PDF):
//! - `- 581-13 -` y = 787.06..798.99pt, x 중심 297.47pt
//! - `압 류 목 록` x = 264.58..330.54pt (중심 297.56pt)
//!
//! 배치 산식의 마지막 조각은 **바깥 여백**이다 — 이 개체의 `margin.bottom` 은
//! 4252HWPUNIT(42.5pt)이고, 한글은 용지 하단에서 정확히 그만큼 위에 둔다.
//!
//! ## [#6874] 개체 종류가 Form -> Table 로 바뀌었다 — 수치 계약은 그대로다
//!
//! HWP3 `obj_type=3` 은 캡션을 담은 1x1 표 구조로 저장되고 한글도 그것을 표로 만든다
//! (정본 HWP3 -> HWPX 대조: `hp:tbl 2 / hp:btn 0`). 종전 파서는 그 표를 버리고
//! `FormObject{PushButton}` 만 남겨 `- 581-13 -` 이 본문 글자가 아니라 개체 속성이
//! 됐고, 저장본에서 그 12자가 사라졌다. 이제 표로 보존하므로 이 테스트도 **개체 종류가
//! 아니라 배치**를 본다 — 기대 좌표(1052.5px / 396.7px)는 한글 2024 COM PDF 실측
//! 그대로이고 바꾸지 않았다.
//!
//! 표로 두면 세로 기준이 `VertRelTo::Paper` 경로로 가는데, 그 기준 높이가
//! `col_area.y * 2 + col_area.height`(상·하 여백이 같다는 가정)이면 이 문서
//! (위 30mm·아래 20mm)에서 +37.8px 아래로 밀린다. 같은 커밋이 그 기준을 실제 용지
//! 높이로 바로잡아 두 계약이 함께 산다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6266/seizure_list_form_button.hwp")
        .to_string_lossy()
        .into_owned()
}

/// `dump-extents` 의 첫 쪽 트리에서 (종류, x, y, w, h) 를 훑는다.
fn extents() -> String {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `압 류` 를 담은 첫 본문 줄의 TextRun x 시작점(px).
fn title_run_x() -> f64 {
    let text = extents();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.starts_with("TextRun") || !line.contains("\"압 류\"") {
            continue;
        }
        // 표 안의 `압 류` 셀도 같은 문자열이라, 본문 줄(y < 200px)만 취한다.
        let y = line
            .split("y=")
            .nth(1)
            .and_then(|r| r.split("..").next())
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(f64::MAX);
        if y > 200.0 {
            continue;
        }
        if let Some(x) = line
            .split(" x=")
            .nth(1)
            .and_then(|r| r.split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok())
        {
            return x;
        }
    }
    panic!("본문 제목 TextRun 을 찾지 못했다:\n{text}");
}

#[test]
fn form_object_is_not_inlined_into_the_title_line() {
    // 제목은 본문 가운데(297.5pt = 396.7px)에 온다. 양식 개체가 같은 줄에 인라인으로
    // 들어가면 제목 + 개체 묶음의 가운데를 잡아 제목이 왼쪽으로 밀린다
    // (종전 실측 x=228.6pt=304.9px, 한글 대비 35.9pt 이탈).
    let x = title_run_x();
    // 한글 실측 264.58pt = 352.8px.
    assert!(
        (x - 352.8).abs() <= 4.0,
        "제목이 한글(352.8px)에서 벗어났다 — 양식 개체가 줄 폭을 먹고 있다: {x:.1}"
    );
}

#[test]
fn form_object_lands_at_paper_bottom_center() {
    // 개체는 1x1 표로 보존된다(#6874). `dump-extents` 의 표 줄은
    //   `Table  y=A..B h=H x=X w=W  pi=P ci=C RxC`
    // 꼴이라, 이 문서에서 유일한 `1x1` 표가 그 개체다(본문 표는 16x8).
    let text = extents();
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("Table") && l.ends_with("1x1"))
        .expect("1x1 표 노드가 없다 — 개체가 소실됐다");
    // `y=A..B` 의 범위 표기를 공백으로 풀어 앞 값만 읽는다.
    let flat = line.replace("..", " ");
    let num = |key: &str| -> f64 {
        flat.split(&format!("{key}="))
            .nth(1)
            .and_then(|r| {
                r.trim_start()
                    .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .find(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or_else(|| panic!("{key} 를 읽지 못했다: {flat}"))
    };
    let (x, y, w) = (num("x"), num("y"), num("w"));

    // 세로: 용지 하단에서 바깥 여백(4252HU = 56.7px)만큼 위 -> 1052.5px.
    // 인라인이던 종전에는 제목 줄(161px)에 있었다.
    assert!(
        (y - 1052.5).abs() <= 6.0,
        "개체가 쪽 하단에 놓이지 않았다: y={y:.1} (기대 1052.5)"
    );
    // 가로: 용지 가운데 -> 중심 396.7px (한글 297.47pt).
    let center = x + w / 2.0;
    assert!(
        (center - 396.7).abs() <= 4.0,
        "개체가 용지 가운데가 아니다: center={center:.1} (기대 396.7)"
    );

    // [#6874] 캡션이 **본문 글자**로 남는다 — 종전에는 개체 속성이라 저장본에서 사라졌다.
    assert!(
        text.lines()
            .any(|l| l.contains("TextRun") && l.contains("- 581-13 -")),
        "일련번호가 본문 글자로 남지 않았다"
    );
}
