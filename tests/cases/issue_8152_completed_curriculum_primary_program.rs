//! #8152 교육과정 완료 페이지: 실제 주 프로그램의 전진과 저장 어절 꼬리.
//! 같은 원본·독립 Print 전415쪽 Native/fresh WASM90% 및 p4/p27198%·직접
//! 판독 뒤 등록한다. 기준 좌표는 PDF 원점·벡터 선을96CSSpx로 환산한 값이다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
const INPUT: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";
struct Output(PathBuf);
impl Drop for Output {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn page(number: usize) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out = Output(
        root.join("output")
            .join(format!("curriculum-program-{}-{nonce}", std::process::id())),
    );
    fs::create_dir_all(&out.0).unwrap();
    let bin =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    // 실제 공급을 확인한 프로그램 선택의 조판 계약을 선언한다. 회귀 검사는
    // private TTF 설치나 글리프 래스터에 의존하지 않고 공통 Native 조판을 검사한다.
    let environment = r#"{"id":"audited-curriculum-primary-ttf","substitutions":{"휴먼명조":"휴먼명조"},"trueTypeFaces":["휴먼명조"],"preserveSourceMetrics":true}"#;
    let environment_path = out.0.join("font-environment.json");
    fs::write(&environment_path, environment).unwrap();
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(root.join(INPUT))
        .args(["--compat", "2022", "--font-environment"])
        .arg(&environment_path)
        .arg("--page")
        .arg((number - 1).to_string())
        .arg("-o")
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "실제 Native 출력: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&fs::read(out.0.join(format!("render_tree_{number:03}.json"))).unwrap())
        .unwrap()
}
fn children(n: &Value) -> impl Iterator<Item = &Value> {
    n.get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}
fn text(n: &Value) -> String {
    let mut out = if n["type"] == "TextRun" {
        n["text"].as_str().unwrap().to_string()
    } else {
        String::new()
    };
    for c in children(n) {
        out.push_str(&text(c));
    }
    out
}
fn runs<'a>(n: &'a Value, pi: u64, out: &mut Vec<&'a Value>) {
    if n["type"] == "TextRun" && n["pi"] == pi {
        out.push(n);
    }
    for c in children(n) {
        runs(c, pi, out);
    }
}
#[test]
fn selected_primary_program_keeps_completed_paragraph_origins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let doc = rhwp::parse_document(&fs::read(root.join(INPUT)).unwrap()).unwrap();
    let tree = page(4);
    for (prefix, x) in [
        ("범교과 학습 주제와", 139.28451538085938),
        ("에 관한 정책연구사업의", 171.9068400065104),
    ] {
        let (pi, source) = doc.sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .find(|(_, p)| p.text.starts_with(prefix))
            .unwrap();
        let mut found = vec![];
        runs(&tree, pi as u64, &mut found);
        assert_eq!(
            found
                .iter()
                .map(|n| n["text"].as_str().unwrap())
                .collect::<String>(),
            source.text,
            "독립 Print4쪽 문단의 순서·누락·중복"
        );
        let first = found.first().expect("같은 쪽 첫 source 글자");
        assert!(first["text"].as_str().unwrap().starts_with(prefix));
        let actual = first["bbox"]["x"].as_f64().unwrap();
        assert!(
            (actual - x).abs() <= 2.0,
            "TTF 전진에 HFT 굵게 추적을 더해 문단을 이동하지 않는다: {prefix} {actual} vs{x}"
        );
    }
}
fn cell(n: &Value, row: u64, col: u64) -> Option<&Value> {
    if n["type"] == "Cell" && n["row"] == row && n["col"] == col {
        return Some(n);
    }
    children(n).find_map(|c| cell(c, row, col))
}

// 독립 Print 48쪽 오른쪽 병합 칸은 현재 줄 소속을 유지하면서 가운데 정렬된다.
// 저장 줄 수가 바뀌었다고 Top으로 바꾸면 위·아래 빈 공간의 균형이 깨진다.
#[test]
fn current_program_keeps_the_declared_alignment_of_its_selected_span_lines() {
    fn ink_bounds(n: &Value, bounds: &mut Option<(f64, f64)>) {
        if n["type"] == "TextRun"
            && n["text"]
                .as_str()
                .is_some_and(|text| !text.trim().is_empty())
        {
            let top = n["bbox"]["y"].as_f64().unwrap();
            let bottom = top + n["bbox"]["h"].as_f64().unwrap();
            *bounds = Some(bounds.map_or((top, bottom), |(start, end)| {
                (start.min(top), end.max(bottom))
            }));
        }
        for child in children(n) {
            ink_bounds(child, bounds);
        }
    }
    let tree = page(48);
    let owner = cell(&tree, 8, 7).expect("Print 48쪽 오른쪽 걸침 칸");
    let content = text(owner);
    assert_eq!(content.matches("원주시 민주화운동").count(), 1);
    assert_eq!(content.matches("창원시 민주화운동").count(), 1);
    assert!(content.contains("업을 추진할 수 있다."));
    let mut bounds = None;
    ink_bounds(owner, &mut bounds);
    let (top, bottom) = bounds.expect("현재 조판이 선택한 실제 글줄");
    let cell_top = owner["bbox"]["y"].as_f64().unwrap();
    let cell_bottom = cell_top + owner["bbox"]["h"].as_f64().unwrap();
    assert!(
        top >= cell_top && bottom <= cell_bottom,
        "선택한 줄은 소유 칸 안에 남는다"
    );
    assert!(
        ((top - cell_top) - (cell_bottom - bottom)).abs() <= 2.0,
        "독립 Print의 Center 정렬: 위·아래 여유가 같아야 한다 ({top}..{bottom}, {cell_top}..{cell_bottom})"
    );
}
#[test]
fn projected_marker_tail_keeps_the_completed_continuation_owner() {
    let tree = page(271);
    assert!(
        cell(&tree, 24, 7).is_none(),
        "앞 쪽에서 수용한 어절 꼬리를 다음 쪽의 별도 행으로 중복 예약하지 않는다"
    );
    let next = cell(&tree, 25, 7).expect("독립 Print271쪽의 다음 원본 행");
    let content = text(next)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    assert!(content.contains("다양한갈등의양상과사회통합을위한구체적인방안제시하기"));
    assert!(content.contains("바람직한소통행위를담론윤리의관점에서설명하고토론실천하기"));
    assert_eq!(content.matches("다양한갈등").count(), 1);
    // PDF의 경계 선 중심146.6413167px. 셀 내용·행 소유로 대상을 찾은 뒤 대조한다.
    let y = next["bbox"]["y"].as_f64().unwrap();
    assert!(
        (y - 146.64131673177084).abs() <= 2.0,
        "어절 꼬리의 유령 행이 다음 행을 아래로 밀지 않는다:{y}"
    );
}

// 독립 Print 133쪽 마지막 줄과134쪽 첫 줄의 연결을 검사한다.
// 두 쪽 모두98% 이상은 아니므로 절대 px 좌표를 고정하지 않는다.
#[test]
fn current_line_closing_advance_preserves_both_page_owners() {
    let first_page = page(133);
    let next_page = page(134);
    let first = cell(&first_page, 144, 8).expect("Print 133쪽 마지막 원본 행의 첫 줄");
    let next = cell(&next_page, 144, 8).expect("Print 134쪽 같은 행의 이어지는 줄");
    let normalize = |value: String| {
        value
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let opening = "-지구주위를둘러싸고있는공기의역할을이해하고,도시";
    let continuation = "와시골에서공기의흐름이달라지는까닭을생각해보기";
    assert_eq!(
        normalize(text(first)),
        opening,
        "현재 줄은133쪽에서 수용한다"
    );
    assert_eq!(
        normalize(text(next)),
        continuation,
        "첫 줄을134쪽에 중복 이월하지 않는다"
    );
    for (page, owner) in [(&first_page, first), (&next_page, next)] {
        let content = normalize(text(page));
        assert_eq!(
            content.matches(&normalize(text(owner))).count(),
            1,
            "원본 줄의 누락·중복"
        );
        for line in children(owner).filter(|n| n["type"] == "TextLine") {
            let y = line["bbox"]["y"].as_f64().unwrap();
            let end = y + line["bbox"]["h"].as_f64().unwrap();
            let top = owner["bbox"]["y"].as_f64().unwrap();
            let bottom = top + owner["bbox"]["h"].as_f64().unwrap();
            assert!(
                y >= top && end <= bottom,
                "원본 줄이 실제 셀 조각 안에 포함된다"
            );
        }
    }
}

// 독립 Print 177→178쪽은 같은 원본 행의 마지막 줄을 이어받는다.
#[test]
fn unchanged_cell_keeps_its_local_continuation_frame_after_body_rebase() {
    let opening = page(177);
    let continuation = page(178);
    let first = cell(&opening, 52, 3).expect("독립 Print 177쪽 마지막 셀");
    let next = cell(&continuation, 52, 3).expect("독립 Print 178쪽 같은 셀의 나머지");
    let normalize = |s: String| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert_eq!(
        normalize(text(first)),
        "갈등해결과소통의윤리:사회의다양한갈등을극복하는데필요한소통의"
    );
    assert_eq!(normalize(text(next)), "윤리는무엇인가?");
    assert!(
        !normalize(text(&opening)).contains("윤리는무엇인가?"),
        "다음 쪽 줄의 선행 수용 금지"
    );
    assert_eq!(
        normalize(text(&continuation))
            .matches("윤리는무엇인가?")
            .count(),
        1
    );
    // 독립 Print 178쪽의 이 문장 원점: 240pt·110.9198837pt를96CSSpx로 환산.
    // Native/fresh WASM 각각99.96581%와 review·overlay 직접 판독 뒤 고정한다.
    let mut selected = vec![];
    runs(next, 1, &mut selected);
    let first_run = selected
        .iter()
        .find(|run| {
            run["text"]
                .as_str()
                .is_some_and(|text| text.starts_with("윤리는"))
        })
        .expect("이어받은 원본 문장의 첫 글자");
    for (axis, expected) in [("x", 320.0), ("y", 147.89317830403647)] {
        let actual = first_run["bbox"][axis].as_f64().unwrap();
        assert!(
            (actual - expected).abs() <= 2.0,
            "이어받기 줄의 {axis} 원점: {actual}"
        );
    }
    let following = cell(&continuation, 53, 3).expect("같은 쪽 후속 원본 행");
    let end = next["bbox"]["y"].as_f64().unwrap() + next["bbox"]["h"].as_f64().unwrap();
    assert!(
        end <= following["bbox"]["y"].as_f64().unwrap(),
        "이어받은 셀과 후속 행의 물리 순서"
    );
}
