//! [Issue #6101] 문단의 블록(비인라인) TAC 표와 본문 텍스트가 저장 lineseg
//! **한 줄**에 함께 담긴 경우, 합성이 줄을 분리하지 않아 ① 레이아웃 TAC 폴백이
//! line0 전체 폭(=동거 텍스트)을 leading 으로 오산해 표가 텍스트 폭만큼
//! 우측으로 밀려 쪽 밖으로 잘리고(36361137 7쪽 x 626 vs 한글 69.7, 36501883
//! 1쪽 x 495.6 vs 한글 76.8) ② 표→텍스트 순서 문단(36361137)에서는 조판이
//! 텍스트 줄을 계상·발행하지 않아 본문("ㅇ 직무요건 …")이 통째로 소실됐다.
//!
//! 수정: 합성(compose_paragraph)이 블록 TAC 표 줄과 텍스트 줄을 분리한다 —
//! 한글 2020 오라클은 두 문서 모두 표를 줄 머리에, 텍스트를 표 아래 줄에 둔다.
//!
//! 결함 상태에서는 표 좌단 x 어서션(두 문서)과 텍스트 존재 어서션(36361137)이
//! 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const FIREFIGHTER: &str = "samples/issue6101/36361137_firefighter_training_plan.hwpx";
const APPROVAL: &str = "samples/issue6101/36501883_approval_doc_body.hwpx";

// 이전 봉인 출력기로도 같은 기존 검사를 실행해 결함 검출을 입증한다.
fn load(sample: &str, pages: &[u32]) -> (usize, Vec<String>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(sample);
    if let Some(bin) = std::env::var_os("RHWP_6101_RENDER_BIN") {
        let info = std::process::Command::new(&bin)
            .args(["info", "--json"])
            .arg(&path)
            .output()
            .expect("이전 출력기 정보 실행");
        assert!(info.status.success(), "이전 출력기 정보 실패");
        let info: serde_json::Value = serde_json::from_slice(&info.stdout).expect("정보 JSON");
        let count = info["pageCount"].as_u64().expect("쪽수") as usize;
        let dir = root.join("output/pr-review/regression-temp").join(format!(
            "issue6101-{}-{}",
            std::process::id(),
            path.file_stem().expect("원문 이름").to_string_lossy()
        ));
        std::fs::create_dir_all(&dir).expect("진단 출력 경로");
        let mut svgs = Vec::new();
        for page in pages {
            let run = std::process::Command::new(&bin)
                .arg("export-svg")
                .arg(&path)
                .args(["-p", &page.to_string(), "-o"])
                .arg(&dir)
                .output()
                .expect("이전 출력기 SVG 실행");
            assert!(run.status.success(), "이전 출력기 SVG 실패");
            let svg = dir.join(format!(
                "{}_{:03}.svg",
                path.file_stem().expect("원문 이름").to_string_lossy(),
                page + 1
            ));
            svgs.push(std::fs::read_to_string(svg).expect("이전 SVG 읽기"));
        }
        std::fs::remove_dir_all(&dir).expect("해당 진단 출력만 정리");
        return (count, svgs);
    }
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원문 읽기")).expect("문서 열기");
    let svgs = pages
        .iter()
        .map(|page| core.render_page_svg_native(*page).expect("쪽 SVG"))
        .collect();
    (core.page_count() as usize, svgs)
}

#[test]
fn issue_6101_table_after_control_text_renders_and_table_stays_left() {
    let (count, svgs) = load(FIREFIGHTER, &[2, 6]);
    // 3쪽 비활성 셀 여백 표의 실제 외곽은 정상 PDF y=700.03–850.27px다.
    // 셀의 마지막 가시 줄 간격이나 보존 여백을 되살려 높이면 안 된다.
    let mut top = f64::INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for chunk in svgs[0].split("<line ").skip(1) {
        let head = chunk.split('>').next().expect("괘선 태그");
        if let (Some(y1), Some(y2)) = (attr(head, "y1"), attr(head, "y2")) {
            // 잘못 높아진 이전 표의 877px 하단도 수집한다. 마지막 괘선은
            // 병합 셀 때문에 여러 조각이므로 전폭 한 조각만 고르지 않는다.
            if (y1 - y2).abs() < 0.01 && (690.0..900.0).contains(&y1) {
                top = top.min(y1);
                bottom = bottom.max(y1);
            }
        }
    }
    assert!(
        (top - 700.03).abs() < 2.0 && (bottom - 850.27).abs() < 2.0,
        "3쪽 표 외곽은 독립 PDF와 일치해야 한다: {top:.2}–{bottom:.2}"
    );
    // 같은 원문의 정상 Windows 한컴 PDF는 11쪽이다. 전체 Native/fresh WASM
    // 90% 검증 뒤 잠정 12쪽 핀을 독립 기준으로 교체한다.
    assert_eq!(count, 11, "정상 한컴 2020 PDF의 11쪽과 일치해야 한다");
    let svg = &svgs[1];

    // 표(입학학년/연령요건, 폭 633px)는 좌측 여백(한글 x=69.7)에서 시작해야
    // 한다. 결함 시 x≈626 에서 시작해 본문 우단을 534px 초과.
    let table_left = wide_rule_min_x(svg, 600.0, 400.0, 520.0).expect("표 전폭 괘선");
    assert!(
        (table_left - 69.72).abs() < 2.0,
        "블록 TAC 표는 좌측 여백에서 시작해야 한다 (한글 69.7, 결함 시 626): {table_left:.1}"
    );

    // 정상 PDF 직무요건 기준선은 401.52pt×96/72=535.36px다.
    // 문자열 존재만으로 표 아래의 실제 위치를 대신하지 않는다.
    let below_table = text_glyphs_in_band(svg, 533.36, 537.36);
    assert!(
        ['직', '무', '요', '건'].iter().all(|ch| below_table.contains(*ch)),
        "표 뒤 본문 텍스트가 소실되면 안 된다 (결함 시 render-tree·export-text 모두 부재): {below_table:?}"
    );
}

#[test]
fn issue_6101_text_before_control_table_stays_left() {
    let (count, svgs) = load(APPROVAL, &[0]);
    assert_eq!(count, 2, "한글 2020 정본은 2쪽이다");
    let svg = &svgs[0];

    // 텍스트→표 순서(별표 53자 + 표 11×9)도 같은 서명 — 표는 좌측 여백(한글
    // x=76.8)에서 시작해야 한다. 결함 시 x≈495.6 으로 본문 우단을 422px 초과.
    let table_left = wide_rule_min_x(svg, 600.0, 430.0, 770.0).expect("표 전폭 괘선");
    assert!(
        (table_left - 76.76).abs() < 2.0,
        "블록 TAC 표는 좌측 여백에서 시작해야 한다 (한글 76.8, 결함 시 495.6): {table_left:.1}"
    );
    let mut table_top = f64::INFINITY;
    for chunk in svg.split("<line ").skip(1) {
        let head = chunk.split('>').next().expect("괘선 태그");
        if let (Some(x1), Some(x2), Some(y1), Some(y2)) = (
            attr(head, "x1"),
            attr(head, "x2"),
            attr(head, "y1"),
            attr(head, "y2"),
        ) {
            if (y1 - y2).abs() < 0.01 && (x2 - x1).abs() > 600.0 && (430.0..455.0).contains(&y1) {
                table_top = table_top.min(y1);
            }
        }
    }
    assert!(
        (table_top - 443.035).abs() < 2.0,
        "1쪽 표 상단은 정상 PDF 443.035px와 일치해야 한다: {table_top:.3}"
    );
}

fn text_glyphs_in_band(svg: &str, y_min: f64, y_max: f64) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(y) = attr(&chunk[..tag_end], "y") else {
            continue;
        };
        if y < y_min || y > y_max {
            continue;
        }
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
}

/// y 대역 [y_min, y_max] 안의 전폭(>min_w) 가로 괘선들 중 최소 x1.
fn wide_rule_min_x(svg: &str, min_w: f64, y_min: f64, y_max: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end];
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            attr(head, "x1"),
            attr(head, "y1"),
            attr(head, "x2"),
            attr(head, "y2"),
        ) else {
            continue;
        };
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < min_w || y1 < y_min || y1 > y_max {
            continue;
        }
        let left = x1.min(x2);
        best = Some(best.map_or(left, |b: f64| b.min(left)));
    }
    best
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    if let Some(start) = head.find(&needle) {
        let rest = &head[start + needle.len()..];
        let end = rest.find('"')?;
        return rest[..end].parse().ok();
    }
    // 상대 크기/장평 적용 글리프의 translate 원점도 같은 기준선을 검사한다.
    let index = match name {
        "x" => 0,
        "y" => 1,
        _ => return None,
    };
    let transform = head
        .split("transform=\"translate(")
        .nth(1)?
        .split(')')
        .next()?;
    transform
        .split([',', ' '])
        .filter(|value| !value.is_empty())
        .nth(index)?
        .parse()
        .ok()
}
