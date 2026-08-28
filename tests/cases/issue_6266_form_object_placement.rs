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
    let out = Command::new(rhwp_bin())
        .args(["export-render-tree", &sample(), "-p", "0", "--stdout"])
        .output()
        .unwrap();
    let json = if out.status.success() && !out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stdout).into_owned()
    } else {
        // `--stdout` 미지원 빌드 대비 — 임시 폴더로 내보내 읽는다.
        let dir = std::env::temp_dir().join(format!("rhwp-6266-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = Command::new(rhwp_bin())
            .args([
                "export-render-tree",
                &sample(),
                "-p",
                "0",
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
            .expect("render tree JSON 이 없다");
        std::fs::read_to_string(path).unwrap()
    };

    // `"Form"` 노드의 bbox 를 찾는다.
    let idx = json
        .find("\"Form\"")
        .expect("Form 노드가 없다 — 개체가 소실됐다");
    let tail = &json[idx..];
    let bbox = tail.find("\"bbox\"").expect("Form bbox 가 없다");
    let seg = &tail[bbox..(bbox + 200).min(tail.len())];
    let num = |key: &str| -> f64 {
        seg.split(&format!("\"{key}\""))
            .nth(1)
            .and_then(|r| r.trim_start().strip_prefix(':'))
            .map(|r| {
                r.trim_start()
                    .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            })
            .and_then(|r| {
                r.split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .find(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or_else(|| panic!("bbox.{key} 를 읽지 못했다: {seg}"))
    };
    let (x, y, w) = (num("x"), num("y"), num("w"));

    // 세로: 용지 하단에서 바깥 여백(4252HU = 56.7px)만큼 위 → 1052.5px.
    // 인라인이던 종전에는 제목 줄(161px)에 있었다.
    assert!(
        (y - 1052.5).abs() <= 6.0,
        "양식 개체가 쪽 하단에 놓이지 않았다: y={y:.1} (기대 1052.5)"
    );
    // 가로: 용지 가운데 → 중심 396.7px (한글 297.47pt).
    let center = x + w / 2.0;
    assert!(
        (center - 396.7).abs() <= 4.0,
        "양식 개체가 용지 가운데가 아니다: center={center:.1} (기대 396.7)"
    );
}
