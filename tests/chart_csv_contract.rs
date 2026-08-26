//! [#4100 B1] `chart-to-csv` / `csv-to-chart` 계약 회귀 테스트.
//!
//! 이 축의 값은 세 가지다.
//!
//! 1. **값이 사라지지 않는다** — 행 수를 라벨 수로 잡으면 `c:cat` 이 없는 계열이 있는
//!    차트에서 값이 통째로 빠진 CSV 가 나온다. 실사용 보고서에서 실제로 그랬다
//!    (6계열 × 6값이 머리 행만 남았다). 오류 없이 틀린 산출이라 눈으로 알아채기 어렵다.
//! 2. **조용한 절삭 금지** — CSV 가 차트와 맞지 않으면 한 칸도 쓰지 않고 `invalid[]` 로
//!    보고하며 사용법 오류(2)로 끝난다. `csv-to-table` 과 같은 규약이다.
//! 3. **두 표현에 함께 쓴다** — 값 하나가 OOXML zip 파트와 중첩 CFB 에 중복 저장돼 있어
//!    한쪽만 쓰면 HWP 변환에서 편집이 사라진다. 어디에 썼는지는 `wrote[]` 로 드러난다.
//!
//! 종료 코드는 #2707 계약(0/1/2)을, 봉투는 `schemaVersion` 규약을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::DocumentCore;
use rhwp::serializer::ole_container::replace_ole_stream;

/// 차트 7종 중 가장 평범한 축 — 3계열 × 4값, 카테고리 라벨 보유.
const SAMPLE: &str = "samples/chart/세로막대형/묶은세로막대형.hwpx";
/// 실사용 보고서 — 차트 2개, 계열 6개, **한 계열에 `c:cat` 이 없다**.
const REPORT: &str = "samples/issue2006/1790387_prep_final_report.hwpx";
/// 분산형 — 첫 열이 X 값이다.
const SCATTER: &str = "samples/chart/분산형/직선이있는분산형.hwpx";
const OOXML_STREAM: &str = "OOXMLChartContents";

const BLANK_VALUE_CHART: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
    r#"<c:tx><c:v>계열</c:v></c:tx><c:cat><c:strLit><c:pt idx="0"><c:v>A</c:v></c:pt><c:pt idx="1"><c:v>B</c:v></c:pt></c:strLit></c:cat>"#,
    r#"<c:val><c:numLit><c:pt idx="0"><c:v>4.3</c:v></c:pt><c:pt idx="1"><c:v/></c:pt></c:numLit></c:val>"#,
    r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
);

const UNTITLED_CHART: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart><c:ser>"#,
    r#"<c:cat><c:strLit><c:pt idx="0"><c:v>A</c:v></c:pt></c:strLit></c:cat>"#,
    r#"<c:val><c:numLit><c:pt idx="0"><c:v>4.3</c:v></c:pt></c:numLit></c:val>"#,
    r#"</c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
);

const UNSHARED_CATEGORY_CHART: &str = concat!(
    r#"<c:chartSpace><c:chart><c:plotArea><c:barChart>"#,
    r#"<c:ser><c:tx><c:v>첫째</c:v></c:tx><c:cat><c:strLit><c:pt idx="0"><c:v>A</c:v></c:pt><c:pt idx="1"><c:v>B</c:v></c:pt></c:strLit></c:cat><c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser>"#,
    r#"<c:ser><c:tx><c:v>둘째</c:v></c:tx><c:cat><c:strLit><c:pt idx="0"><c:v>B</c:v></c:pt><c:pt idx="1"><c:v>A</c:v></c:pt></c:strLit></c:cat><c:val><c:numLit><c:pt idx="0"><c:v>3</c:v></c:pt><c:pt idx="1"><c:v>4</c:v></c:pt></c:numLit></c:val></c:ser>"#,
    r#"</c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
);

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// nextest archive 가 런타임에 주입하는 경로를 먼저 읽는다 (#3289 규약).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn tmp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp-chart-csv-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// 기존 HWPX의 차트 주소·CFB 골격을 재사용하되 ①/② XML을 함께 주입해 CLI 경계를
/// 실제 파일 입출력으로 검증한다.
fn synthetic_chart_input(name: &str, zip_xml: &[u8], nested_xml: &[u8]) -> PathBuf {
    let source = sample(SAMPLE);
    let bytes = std::fs::read(&source).expect("기준 HWPX 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("기준 HWPX 파싱");
    let chart = collect_charts(core.document())[0].clone();
    let zip_idx = chart.zip_part.expect("①");
    let nested_idx = chart.nested_copy.expect("②");
    let nested_original = core.document().bin_data_content[nested_idx].data.load();
    let nested_new =
        replace_ole_stream(&nested_original, OOXML_STREAM, nested_xml).expect("② 교체");
    core.document_mut().bin_data_content[zip_idx].data = zip_xml.to_vec().into();
    core.document_mut().bin_data_content[nested_idx].data = nested_new.into();

    let out = tmp_dir().join(name);
    std::fs::write(&out, core.export_hwpx_native().expect("합성 HWPX 저장"))
        .expect("합성 HWPX 쓰기");
    out
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout JSON 파싱 실패: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn path_str(p: &Path) -> String {
    p.to_str().expect("경로").to_string()
}

// ---------------------------------------------------------------------------
// chart-to-csv
// ---------------------------------------------------------------------------

/// 봉투 모양과 행·열 수. 모든 레코드의 필드 수가 `colCount + 1`(라벨 열) 이어야 한다.
#[test]
fn chart_to_csv_envelope_is_rectangular() {
    let file = path_str(&sample(SAMPLE));
    let out = run(&["chart-to-csv", &file, "--chart", "1", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["schemaVersion"], "1.0");
    assert_eq!(env["chartCount"], 1);
    let chart = &env["charts"][0];
    assert_eq!(chart["chart"], 1);

    let cols = chart["colCount"].as_u64().expect("colCount") as usize;
    let rows = chart["rowCount"].as_u64().expect("rowCount") as usize;
    let csv = chart["csv"].as_str().expect("csv");
    let records: Vec<&str> = csv.trim_end_matches("\r\n").split("\r\n").collect();
    assert_eq!(records.len(), rows + 1, "머리 행 + 데이터 행");
    for (i, record) in records.iter().enumerate() {
        assert_eq!(
            record.split(',').count(),
            cols + 1,
            "{i}행의 열 수가 다르다: {record}"
        );
    }
}

/// **회귀 가드 — 라벨이 없는 계열이 있어도 값이 사라지지 않는다.**
#[test]
fn charts_with_missing_category_labels_still_emit_every_value() {
    let file = path_str(&sample(REPORT));
    let out = run(&["chart-to-csv", &file, "--chart", "1", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let chart = &json_of(&out)["charts"][0];
    assert_eq!(chart["colCount"], 6, "계열 6개");
    assert_eq!(
        chart["rowCount"], 6,
        "값이 6개인데 행이 0개면 CSV 가 값을 잃은 것이다"
    );
    let csv = chart["csv"].as_str().expect("csv");
    assert!(
        csv.lines().count() >= 7,
        "머리 행만 남았다 — 값이 사라졌다:\n{csv}"
    );
}

/// 분산형은 머리 행 첫 칸이 `X` 다 — 첫 열이 라벨이 아니라 수치임을 알린다.
#[test]
fn scatter_marks_the_first_column_as_x() {
    let file = path_str(&sample(SCATTER));
    let out = run(&["chart-to-csv", &file, "--chart", "1", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let csv = json_of(&out)["charts"][0]["csv"]
        .as_str()
        .expect("csv")
        .to_string();
    assert!(csv.starts_with("X,"), "{csv}");
}

/// `-o` 없이 `--json` 도 없으면 CSV 본문을 stdout 으로 흘린다 — 파이프용.
#[test]
fn plain_stdout_is_the_csv_body() {
    let file = path_str(&sample(SAMPLE));
    let out = run(&["chart-to-csv", &file, "--chart", "1"]);
    assert_eq!(out.status.code(), Some(0));
    let body = String::from_utf8_lossy(&out.stdout);
    assert!(body.contains("\r\n"), "RFC 4180 CRLF");
    assert!(!body.trim_start().starts_with('{'), "봉투가 새어 나왔다");
}

/// `--bom` 은 **파일에만** 붙는다. 봉투의 `csv` 문자열에 붙으면 JSON 소비자가 첫 셀 앞의
/// U+FEFF 를 값으로 읽는다.
#[test]
fn bom_goes_to_the_file_not_the_envelope() {
    let file = path_str(&sample(SAMPLE));
    let dest = tmp_dir().join("bom.csv");
    let dest_s = path_str(&dest);
    let out = run(&[
        "chart-to-csv",
        &file,
        "--chart",
        "1",
        "-o",
        &dest_s,
        "--bom",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["bom"], true);
    let csv = env["charts"][0]["csv"].as_str().expect("csv");
    assert!(!csv.starts_with('\u{feff}'), "봉투에 BOM 이 붙었다");

    let bytes = std::fs::read(&dest).expect("산출 읽기");
    assert_eq!(&bytes[..3], b"\xef\xbb\xbf", "파일에 BOM 이 없다");
}

/// 없는 차트 번호는 실행 오류(1)다 — 사용법은 맞고 문서가 다른 경우다.
#[test]
fn missing_chart_number_is_a_runtime_error() {
    let file = path_str(&sample(SAMPLE));
    let out = run(&["chart-to-csv", &file, "--chart", "99", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{:?}", out);
}

// ---------------------------------------------------------------------------
// csv-to-chart
// ---------------------------------------------------------------------------

fn seed_csv(doc: &str, name: &str) -> (String, String) {
    let file = path_str(&sample(doc));
    let csv_path = tmp_dir().join(name);
    let csv_s = path_str(&csv_path);
    let out = run(&["chart-to-csv", &file, "--chart", "1", "-o", &csv_s]);
    assert_eq!(out.status.code(), Some(0), "기준선 CSV: {:?}", out);
    (file, csv_s)
}

/// **무편집 왕복은 한 칸도 바꾸지 않는다** (수용 기준 2 의 CLI 층).
#[test]
fn round_tripping_the_exported_csv_changes_nothing() {
    let (file, csv) = seed_csv(SAMPLE, "roundtrip.csv");
    let dest = path_str(&tmp_dir().join("roundtrip.hwpx"));
    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv,
        "--chart",
        "1",
        "-o",
        &dest,
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["changedCount"], 0, "{env}");
    assert_eq!(env["wrote"].as_array().expect("wrote").len(), 0);
    assert_eq!(env["invalid"].as_array().expect("invalid").len(), 0);
}

/// 실제 편집은 **두 표현에 함께** 쓴다 — `wrote[]` 가 그것을 드러낸다.
#[test]
fn an_edit_reports_both_representations_in_wrote() {
    let (file, csv) = seed_csv(SAMPLE, "edit.csv");
    let body = std::fs::read_to_string(&csv).expect("CSV 읽기");
    let edited = body.replacen("4.3", "91.7", 1);
    assert_ne!(edited, body, "센티널이 원본에 없다");
    std::fs::write(&csv, &edited).expect("CSV 쓰기");

    let dest = path_str(&tmp_dir().join("edited.hwpx"));
    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv,
        "--chart",
        "1",
        "-o",
        &dest,
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["changedCount"], 1, "{env}");
    let wrote: Vec<&str> = env["wrote"]
        .as_array()
        .expect("wrote")
        .iter()
        .map(|v| v.as_str().expect("문자열"))
        .collect();
    assert_eq!(wrote, ["zipPart", "nestedCopy"], "{env}");
    assert_eq!(env["changed"][0]["to"], "91.7");

    // 산출을 다시 읽으면 새 값이 있다.
    let back = run(&["chart-to-csv", &dest, "--chart", "1", "--json"]);
    assert_eq!(back.status.code(), Some(0));
    assert!(json_of(&back)["charts"][0]["csv"]
        .as_str()
        .expect("csv")
        .contains("91.7"));
}

/// 빈 `<c:v/>`는 CSV에서 빈 칸으로 유지된다. 다른 값만 바꿀 때 그 빈 칸 때문에 전체
/// 행렬이 거부되면 `chart-to-csv`/`csv-to-chart` 왕복 계약이 깨진다.
#[test]
fn csv_edit_keeps_an_unchanged_blank_value() {
    let input = synthetic_chart_input(
        "blank-value.hwpx",
        BLANK_VALUE_CHART.as_bytes(),
        BLANK_VALUE_CHART.as_bytes(),
    );
    let input_s = path_str(&input);
    let csv = tmp_dir().join("blank-value.csv");
    let csv_s = path_str(&csv);
    let export = run(&["chart-to-csv", &input_s, "--chart", "1", "-o", &csv_s]);
    assert_eq!(export.status.code(), Some(0), "{export:?}");

    let body = std::fs::read_to_string(&csv).expect("CSV 읽기");
    std::fs::write(&csv, body.replacen("4.3", "91.7", 1)).expect("CSV 편집");
    let output = tmp_dir().join("blank-value-edited.hwpx");
    let output_s = path_str(&output);
    let imported = run(&[
        "csv-to-chart",
        &input_s,
        "--csv",
        &csv_s,
        "--chart",
        "1",
        "-o",
        &output_s,
        "--json",
    ]);
    assert_eq!(imported.status.code(), Some(0), "{imported:?}");
    assert_eq!(json_of(&imported)["changedCount"], 1);
    assert!(output.exists(), "변환본이 없다");
}

/// `<c:tx>`가 없는 계열도 CSV의 빈 머리 칸을 다시 읽어 무편집 왕복할 수 있어야 한다.
#[test]
fn untitled_series_round_trips_through_csv() {
    let input = synthetic_chart_input(
        "untitled.hwpx",
        UNTITLED_CHART.as_bytes(),
        UNTITLED_CHART.as_bytes(),
    );
    let input_s = path_str(&input);
    let csv = tmp_dir().join("untitled.csv");
    let csv_s = path_str(&csv);
    let export = run(&["chart-to-csv", &input_s, "--chart", "1", "-o", &csv_s]);
    assert_eq!(export.status.code(), Some(0), "{export:?}");

    let output = tmp_dir().join("untitled-roundtrip.hwpx");
    let output_s = path_str(&output);
    let imported = run(&[
        "csv-to-chart",
        &input_s,
        "--csv",
        &csv_s,
        "--chart",
        "1",
        "-o",
        &output_s,
        "--json",
    ]);
    assert_eq!(imported.status.code(), Some(0), "{imported:?}");
    let env = json_of(&imported);
    assert_eq!(env["changedCount"], 0, "{env}");
    assert!(
        env["invalid"].as_array().expect("invalid").is_empty(),
        "{env}"
    );
}

/// 계열별 카테고리가 다르면 한 CSV 라벨 열이 어느 계열의 값인지 보장하지 못한다.
/// 내보내기와 되돌리기 모두 파일을 만들기 전에 fail-closed한다.
#[test]
fn nonshared_category_labels_are_refused_by_csv_commands() {
    let input = synthetic_chart_input(
        "unshared-category.hwpx",
        UNSHARED_CATEGORY_CHART.as_bytes(),
        UNSHARED_CATEGORY_CHART.as_bytes(),
    );
    let input_s = path_str(&input);
    let exported = tmp_dir().join("unshared-category.csv");
    let exported_s = path_str(&exported);
    let out = run(&[
        "chart-to-csv",
        &input_s,
        "--chart",
        "1",
        "-o",
        &exported_s,
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(!exported.exists(), "안전하지 않은 CSV가 만들어졌다");

    let csv = tmp_dir().join("unshared-category-input.csv");
    let csv_s = path_str(&csv);
    std::fs::write(&csv, ",첫째,둘째\r\nA,10,3\r\nB,2,4\r\n").expect("CSV 쓰기");
    let output = tmp_dir().join("unshared-category-output.hwpx");
    let output_s = path_str(&output);
    let out = run(&[
        "csv-to-chart",
        &input_s,
        "--csv",
        &csv_s,
        "--chart",
        "1",
        "-o",
        &output_s,
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(
        json_of(&out)["invalid"][0]["reason"],
        "sharedCategoryRequired"
    );
    assert!(!output.exists(), "안전하지 않은 HWPX가 만들어졌다");
}

/// `--dry-run` 은 파일을 만들지 않는다.
#[test]
fn dry_run_writes_no_file() {
    let (file, csv) = seed_csv(SAMPLE, "dry.csv");
    let body = std::fs::read_to_string(&csv).expect("CSV 읽기");
    std::fs::write(&csv, body.replacen("4.3", "91.7", 1)).expect("CSV 쓰기");

    let dest = tmp_dir().join("dry-out.hwpx");
    let _ = std::fs::remove_file(&dest);
    let dest_s = path_str(&dest);
    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv,
        "--chart",
        "1",
        "-o",
        &dest_s,
        "--dry-run",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["dryRun"], true);
    assert_eq!(env["changedCount"], 1);
    assert_eq!(env["wrote"].as_array().expect("wrote").len(), 0);
    assert!(!dest.exists(), "dry-run 이 파일을 만들었다");
}

/// **조용한 절삭 금지** — CSV 가 차트와 맞지 않으면 한 칸도 쓰지 않고 exit 2.
#[test]
fn mismatched_csv_writes_nothing_and_exits_two() {
    let (file, csv) = seed_csv(SAMPLE, "mismatch.csv");
    let body = std::fs::read_to_string(&csv).expect("CSV 읽기");

    // 데이터 행 하나를 지운다 — 값 개수가 달라진다.
    let mut records: Vec<&str> = body.trim_end_matches("\r\n").split("\r\n").collect();
    records.pop();
    let short = format!("{}\r\n", records.join("\r\n"));

    let dest = tmp_dir().join("mismatch.hwpx");
    let _ = std::fs::remove_file(&dest);
    let dest_s = path_str(&dest);
    std::fs::write(&csv, short).expect("CSV 쓰기");

    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv,
        "--chart",
        "1",
        "-o",
        &dest_s,
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2), "{:?}", out);

    let env = json_of(&out);
    assert_eq!(env["changedCount"], 0);
    assert_eq!(env["wrote"].as_array().expect("wrote").len(), 0);
    let reasons: Vec<&str> = env["invalid"]
        .as_array()
        .expect("invalid")
        .iter()
        .map(|v| v["reason"].as_str().expect("reason"))
        .collect();
    assert!(reasons.contains(&"valueCountMismatch"), "{reasons:?}");
    assert!(!dest.exists(), "거부했는데 파일이 생겼다");
}

/// 수치가 아닌 값도 같은 규약으로 거부된다.
#[test]
fn non_numeric_cell_is_refused() {
    let (file, csv) = seed_csv(SAMPLE, "nan.csv");
    let body = std::fs::read_to_string(&csv).expect("CSV 읽기");
    std::fs::write(&csv, body.replacen("4.3", "사십삼", 1)).expect("CSV 쓰기");

    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv,
        "--chart",
        "1",
        "--dry-run",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2), "{:?}", out);
    let env = json_of(&out);
    let reasons: Vec<&str> = env["invalid"]
        .as_array()
        .expect("invalid")
        .iter()
        .map(|v| v["reason"].as_str().expect("reason"))
        .collect();
    assert!(reasons.contains(&"notANumber"), "{reasons:?}");
}

/// 깨진 CSV 는 `csvParse` 로 거부된다 — 코어에 닿기 전이다.
#[test]
fn ragged_csv_is_refused_before_reaching_the_core() {
    let csv_path = tmp_dir().join("ragged.csv");
    std::fs::write(&csv_path, ",계열 1,계열 2\r\n항목 1,1,2\r\n항목 2,3\r\n").expect("CSV 쓰기");
    let file = path_str(&sample(SAMPLE));
    let csv_s = path_str(&csv_path);

    let out = run(&[
        "csv-to-chart",
        &file,
        "--csv",
        &csv_s,
        "--chart",
        "1",
        "--dry-run",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2), "{:?}", out);
    assert_eq!(json_of(&out)["invalid"][0]["reason"], "csvParse");
}

/// 필수 인자가 빠지면 사용법 오류(2)다.
#[test]
fn missing_required_args_is_usage_error() {
    let file = path_str(&sample(SAMPLE));
    assert_eq!(run(&["csv-to-chart", &file]).status.code(), Some(2));
    assert_eq!(
        run(&["csv-to-chart", &file, "--chart", "1"]).status.code(),
        Some(2)
    );
    assert_eq!(run(&["chart-to-csv"]).status.code(), Some(2));
}

// ---------------------------------------------------------------------------
// 4면 배선
// ---------------------------------------------------------------------------

/// capabilities·MCP·help 세 곳이 함께 갱신돼야 에이전트가 쓸 수 있다
/// (`table_csv_contract::capabilities_and_mcp_declare_both_commands` 의 짝).
#[test]
fn capabilities_and_mcp_declare_both_commands() {
    let cap = json_of(&run(&["capabilities"]));
    let names: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    for expected in ["chart-to-csv", "csv-to-chart"] {
        assert!(names.contains(&expected), "capabilities 누락: {expected}");
    }

    let mcp = json_of(&run(&["capabilities", "--mcp"]));
    let tools = mcp["tools"].as_array().expect("tools");
    for (tool_name, command) in [
        ("hwp_chart_to_csv", "chart-to-csv"),
        ("hwp_csv_to_chart", "csv-to-chart"),
    ] {
        let t = tools
            .iter()
            .find(|t| t["name"] == tool_name)
            .unwrap_or_else(|| panic!("MCP 도구 누락: {tool_name}"));
        assert_eq!(t["cli"]["command"], command, "{t}");
        assert_eq!(t["inputSchema"]["type"], "object", "{t}");
        assert!(t["inputSchema"]["properties"].is_object(), "{t}");
        assert!(t["inputSchema"]["required"].is_array(), "{t}");
    }

    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    for expected in ["  chart-to-csv ", "  csv-to-chart "] {
        assert!(
            help_text.contains(expected),
            "--help 에 {expected:?} 가 없습니다"
        );
    }
}
