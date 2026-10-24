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
