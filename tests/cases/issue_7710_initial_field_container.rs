//! #7710/#7715: 초기 안내문의 소유 컨테이너와 실제 입력값을 구분한다.
//! field-01 독립 Print의 2쪽은 제목과 01–05만 출력한다. 좌표/전체 SVG 해시는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn open() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!("../../samples/field-01.hwp")).expect("한컴 원본")
}

fn svg_text(svg: &str) -> String {
    let parsed = roxmltree::Document::parse(svg).expect("최종 SVG XML");
    parsed
        .descendants()
        .filter(|node| node.is_text() && node.ancestors().any(|parent| parent.has_tag_name("text")))
        .filter_map(|node| node.text())
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rhwp-7710-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path).expect("검사 폴더 생성");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn cli(args: &[&str]) -> std::process::Output {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let output = std::process::Command::new(binary)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .expect("실제 CLI");
    assert!(
        output.status.success(),
        "CLI 실패: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn exported_text(profile: &str) -> String {
    let output = TempDir::new();
    cli(&[
        "export-svg",
        "samples/field-01.hwp",
        "--compat",
        "2022",
        "--page",
        "1",
        "--profile",
        profile,
        "--output",
        output.0.to_str().unwrap(),
    ]);
    let svg = std::fs::read_dir(&output.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|extension| extension == "svg"))
        .expect("목차 쪽 SVG");
    svg_text(&std::fs::read_to_string(svg).unwrap())
}

#[test]
fn initial_textbox_guides_are_editor_only_and_print_keeps_numbered_content() {
    let output = cli(&["fields", "samples/field-01.hwp", "--json"]);
    let fields: serde_json::Value = serde_json::from_slice(&output.stdout).expect("필드 조회 봉투");
    let values = fields["fields"].as_array().expect("필드 목록");
    assert_eq!(values.len(), 11, "본문·글상자의 필드 누락/중복 없음");
    assert!(
        values.iter().all(|field| field["value"] == ""),
        "수정되지 않은 안내문은 값이 아님"
    );
    let screen = exported_text("screen");
    let printed = exported_text("print");
    assert_eq!(
        screen.matches("목차입력").count(),
        5,
        "각 글상자의 화면 안내문은 한 번씩 보존"
    );
    assert!(
        !printed.contains("목차입력"),
        "독립 Print에 없는 안내문 제외"
    );
    assert_eq!(printed, "목차0102030405", "번호의 소유·순서·내용 보존");
}

#[test]
fn edited_text_equal_to_guide_survives_hwp_save_and_reload() {
    let mut core = open();
    let id = core
        .collect_all_fields()
        .iter()
        .find(|field| field.field.field_id == 1629316075)
        .expect("원본03 글상자 필드")
        .field
        .field_id;
    core.set_field_value_by_id(id, "목차 입력")
        .expect("같은 문자열을 실제 값으로 입력");
    let bytes = core.export_hwp_native().expect("HWP 저장");
    let reopened = DocumentCore::from_bytes(&bytes).expect("다시 열기");
    let fields = reopened.collect_all_fields();
    let modified = fields
        .iter()
        .find(|field| field.field.field_id == id)
        .unwrap();
    assert_ne!(modified.field.properties & (1 << 15), 0, "원본 수정됨 계약");
    assert_eq!(modified.value, "목차 입력", "안내문과 같은 실제 값도 보존");
    assert!(
        fields
            .iter()
            .filter(|field| field.field.field_id != id)
            .all(|field| field.value.is_empty()),
        "다른 초기 필드와 혼동하지 않음"
    );
}

/// 완료된2쪽의 좌표 계약: 독립 Print의01–05 원점 x=65.004768pt →96dpi 86.673024px.
/// 양쪽 raw 실루엣98% 이상·직접 판독 증거는 PR #7715 review의 TSV/PNG에 연결한다.
#[test]
fn completed_contents_page_keeps_print_number_alignment_and_order() {
    fn numbers(node: &serde_json::Value, out: &mut Vec<(String, f64, f64)>) {
        if node["type"] == "TextRun" {
            if let Some(text) = node["text"].as_str() {
                if ["01", "02", "03", "04", "05"].contains(&text.trim()) {
                    out.push((
                        text.trim().to_string(),
                        node["bbox"]["x"].as_f64().unwrap(),
                        node["bbox"]["y"].as_f64().unwrap(),
                    ));
                }
            }
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                numbers(child, out);
            }
        }
    }
    let dir = TempDir::new();
    cli(&[
        "export-render-tree",
        "samples/field-01.hwp",
        "--page",
        "1",
        "--compat",
        "2022",
        "--output",
        dir.0.to_str().unwrap(),
    ]);
    let tree: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("render_tree_002.json")).unwrap())
            .unwrap();
    let mut positions = Vec::new();
    numbers(&tree, &mut positions);
    assert_eq!(
        positions
            .iter()
            .map(|(text, _, _)| text.as_str())
            .collect::<Vec<_>>(),
        ["01", "02", "03", "04", "05"],
        "누락·중복·소유 순서"
    );
    for (text, x, _) in &positions {
        assert!(
            (x - 86.673_024).abs() <= 2.0,
            "{text}: 독립 Print 왼쪽 정렬 ±2px, 실제 {x}"
        );
    }
    assert!(
        positions.windows(2).all(|pair| pair[0].2 < pair[1].2),
        "번호의 세로 순서 보존"
    );
}
