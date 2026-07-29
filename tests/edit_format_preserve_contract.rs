//! [#3383] `edit` 3종의 **산출 형식 보존** 계약 회귀 테스트.
//!
//! 종전에는 `fill-fields`/`replace-text`/`set-cell` 이 모두 `export_hwp_native()` 로 HWP5 를
//! 강제 산출했다. 그래서 ① HWPX 입력이 조용히 `.hwp` 로 바뀌고 ② 어댑터 없는 native
//! 경로라 HWPX→HWP IR 매핑(#178)조차 타지 않아 실물 양식에서 차트·이미지가 유실됐다.
//!
//! 계약: ① HWPX 입력 → HWPX 산출이고 **다시 읽어도** HWPX 다 ② HWP 입력은 종전대로
//! HWP5 다 ③ 확장자가 어긋나는 `-o` 는 **경로를 그대로 존중**하고 stderr 로 경고한다.
//! 형식 판정은 확장자가 아니라 `info --json` 의 `format`(바이트 감지)으로 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀(이름 있는 CLICK_HERE)·표·본문 텍스트를 모두 가진 실물 HWPX 별지 서식.
/// 한 샘플로 세 하위 명령을 모두 돌릴 수 있다.
const HWPX_SAMPLE: &str = "samples/issue1893_clickhere_field_roundtrip.hwpx";
/// 비교군 HWP5 — 종전 동작(HWP5 산출)이 그대로인지 확인한다.
const HWP_SAMPLE: &str = "samples/field-01.hwp";
/// HWPX 본문에 실재하는 문자열(치환 대상).
const HWPX_NEEDLE: &str = "검찰청";
/// HWP5 본문에 실재하는 문자열(치환 대상) — `edit_replace_text_contract` 와 같은 낱말.
const HWP_NEEDLE: &str = "회사";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 기본 산출 경로 계약을 그대로 재보려면 **입력을 임시 폴더로 복사**해야 한다.
/// `fill-fields` 는 산출물을 입력 파일 옆에(#3469), 나머지 둘은 실행 디렉터리에 만들기
/// 때문에 저장소 안에서 그냥 돌리면 `samples/`·저장소 루트가 더러워진다.
struct Workdir {
    dir: PathBuf,
}

impl Workdir {
    fn new(tag: &str) -> Workdir {
        let dir = std::env::temp_dir().join(format!(
            "rhwp-fmt-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("임시 작업 폴더 생성");
        Workdir { dir }
    }

    /// 샘플을 고정된 이름으로 복사해 기본 산출 이름을 예측 가능하게 만든다.
    fn copy_in(&self, rel: &str, name: &str) -> PathBuf {
        let dst = self.dir.join(name);
        std::fs::copy(sample(rel), &dst).expect("샘플 복사");
        dst
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Workdir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// nextest archive 는 런타임에 `CARGO_BIN_EXE_rhwp` 를 주입한다(#3289) — 그 값을 먼저 읽고
/// 컴파일타임 값을 fallback 으로 쓴다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// 임시 작업 폴더를 현재 디렉터리로 삼아 실행한다 — 기본 산출물이 그 안에 떨어진다.
fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn run_ok(dir: &Path, args: &[&str]) -> serde_json::Value {
    let output = run_in(dir, args);
    assert_eq!(output.status.code(), Some(0), "{}", describe(args, &output));
    parse_json(args, &output)
}

/// 산출물을 **다시 읽어** 실제 형식을 얻는다 — 확장자가 아니라 파서의 판정이다.
fn format_of(dir: &Path, path: &Path) -> String {
    let args = ["info", path.to_str().unwrap(), "--json"];
    run_ok(dir, &args)["format"]
        .as_str()
        .expect("info 봉투의 format")
        .to_string()
}

/// 이름이 있는 첫 누름틀을 고른다 — 샘플의 필드 이름을 테스트에 박아 넣지 않는다.
fn first_field_name(dir: &Path, path: &Path) -> String {
    let args = ["fields", path.to_str().unwrap(), "--json"];
    run_ok(dir, &args)["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .filter_map(|f| f["name"].as_str())
        .find(|name| !name.is_empty())
        .expect("이름 있는 누름틀")
        .to_string()
}

/// 본문 최상위 표(containerPath 없음)의 첫 셀 좌표 — `edit_set_cell_contract` 와 같은 방식.
fn first_body_cell(dir: &Path, path: &Path) -> (u64, u64, u64) {
    let args = ["export-tables", path.to_str().unwrap(), "--json"];
    let v = run_ok(dir, &args);
    let table = v["tables"]
        .as_array()
        .expect("tables 배열")
        .iter()
        .find(|t| t.get("containerPath").is_none())
        .expect("본문 최상위 표");
    let cell = &table["cells"][0];
    (
        table["index"].as_u64().expect("index"),
        cell["row"].as_u64().expect("row"),
        cell["col"].as_u64().expect("col"),
    )
}

/// HWPX 입력 + `-o` 생략 → 기본 산출물이 `.hwpx` 이고 재독 형식도 HWPX 다.
#[test]
fn fill_fields_hwpx_input_defaults_to_hwpx_output() {
    let work = Workdir::new("fill");
    let input = work.copy_in(HWPX_SAMPLE, "input.hwpx");
    let field = first_field_name(&work.dir, &input);
    let mut data_map = serde_json::Map::new();
    data_map.insert(field, serde_json::Value::String("형식보존".to_string()));
    let data = serde_json::Value::Object(data_map).to_string();

    let args = [
        "edit",
        "fill-fields",
        input.to_str().unwrap(),
        "--data",
        &data,
        "--json",
    ];
    let v = run_ok(&work.dir, &args);
    assert_eq!(v["outputFormat"], "hwpx", "{v}");
    assert_eq!(v["filledCount"].as_u64(), Some(1), "{v}");

    let out = work.path("input_filled.hwpx");
    assert!(
        out.exists(),
        "HWPX 입력의 기본 산출은 .hwpx 여야 합니다: {v}"
    );
    assert!(
        !work.path("input_filled.hwp").exists(),
        "HWP5 산출물이 생기면 안 됩니다"
    );
    assert_eq!(format_of(&work.dir, &out), "hwpx", "재독 형식 불일치");
}

/// HWPX 입력 + `-o` 생략 → `_replaced.hwpx`, 재독 형식 HWPX.
#[test]
fn replace_text_hwpx_input_defaults_to_hwpx_output() {
    let work = Workdir::new("replace");
    let input = work.copy_in(HWPX_SAMPLE, "input.hwpx");

    let args = [
        "edit",
        "replace-text",
        input.to_str().unwrap(),
        "--find",
        HWPX_NEEDLE,
        "--replace",
        "지방청",
        "--json",
    ];
    let v = run_ok(&work.dir, &args);
    assert!(
        v["replacedCount"].as_u64().unwrap_or(0) >= 1,
        "샘플에 치환 대상이 있어야 합니다: {v}"
    );
    assert_eq!(v["outputFormat"], "hwpx", "{v}");

    let out = work.path("input_replaced.hwpx");
    assert!(
        out.exists(),
        "HWPX 입력의 기본 산출은 .hwpx 여야 합니다: {v}"
    );
    assert!(
        !work.path("input_replaced.hwp").exists(),
        "HWP5 산출물이 생기면 안 됩니다"
    );
    assert_eq!(format_of(&work.dir, &out), "hwpx", "재독 형식 불일치");

    // 형식만 보존하고 편집이 날아가면 의미가 없다 — 원문 0건을 재독으로 확인한다.
    let searched = run_ok(
        &work.dir,
        &["search", out.to_str().unwrap(), HWPX_NEEDLE, "--json"],
    );
    assert_eq!(searched["matchCount"].as_u64(), Some(0), "{searched}");
}

/// HWPX 입력 + `-o` 생략 → `_cell.hwpx`, 재독 형식 HWPX, 셀 값도 살아 있다.
#[test]
fn set_cell_hwpx_input_defaults_to_hwpx_output() {
    let work = Workdir::new("setcell");
    let input = work.copy_in(HWPX_SAMPLE, "input.hwpx");
    let (table, row, col) = first_body_cell(&work.dir, &input);
    let (ts, rs, cs) = (table.to_string(), row.to_string(), col.to_string());
    let new_value = "형식보존셀";

    let args = [
        "edit",
        "set-cell",
        input.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        new_value,
        "--json",
    ];
    let v = run_ok(&work.dir, &args);
    assert_eq!(v["outputFormat"], "hwpx", "{v}");

    let out = work.path("input_cell.hwpx");
    assert!(
        out.exists(),
        "HWPX 입력의 기본 산출은 .hwpx 여야 합니다: {v}"
    );
    assert!(
        !work.path("input_cell.hwp").exists(),
        "HWP5 산출물이 생기면 안 됩니다"
    );
    assert_eq!(format_of(&work.dir, &out), "hwpx", "재독 형식 불일치");

    // 같은 좌표를 재독해 기록이 HWPX 왕복에서 살아남았는지 대조한다.
    let after = run_ok(
        &work.dir,
        &["export-tables", out.to_str().unwrap(), "--json"],
    );
    let cell = after["tables"]
        .as_array()
        .expect("tables 배열")
        .iter()
        .find(|t| t["index"].as_u64() == Some(table))
        .expect("같은 index 표")["cells"]
        .as_array()
        .expect("cells 배열")
        .iter()
        .find(|c| c["row"].as_u64() == Some(row) && c["col"].as_u64() == Some(col))
        .expect("좌표 셀")
        .clone();
    assert_eq!(cell["text"], new_value, "재독 값 불일치: {cell}");
}

/// HWP5 입력은 종전 그대로다 — 기본 산출 `.hwp`, 재독 형식 HWP5, 경고 없음.
#[test]
fn hwp_input_keeps_hwp5_output() {
    let work = Workdir::new("hwp");
    let input = work.copy_in(HWP_SAMPLE, "input.hwp");

    let args = [
        "edit",
        "replace-text",
        input.to_str().unwrap(),
        "--find",
        HWP_NEEDLE,
        "--replace",
        "기관",
        "--json",
    ];
    let output = run_in(&work.dir, &args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["outputFormat"], "hwp5", "{v}");

    let out = work.path("input_replaced.hwp");
    assert!(out.exists(), "HWP 입력의 기본 산출은 .hwp 여야 합니다: {v}");
    assert!(
        !work.path("input_replaced.hwpx").exists(),
        "HWPX 산출물이 생기면 안 됩니다"
    );
    assert_eq!(format_of(&work.dir, &out), "hwp5", "재독 형식 불일치");
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("경고"),
        "형식이 그대로면 경고하지 않습니다.\n{}",
        describe(&args, &output)
    );
}

/// HWPX 입력에 `-o ….hwp` 를 명시하면 **경로를 그대로 존중**해 HWP5 로 저장하고,
/// 형식이 바뀐다는 사실을 stderr 로 경고한다 (이슈 제안 2의 과도기 규약).
#[test]
fn explicit_hwp_output_for_hwpx_input_is_honoured_with_warning() {
    let work = Workdir::new("explicit-hwp");
    let input = work.copy_in(HWPX_SAMPLE, "input.hwpx");

    let args = [
        "edit",
        "replace-text",
        input.to_str().unwrap(),
        "--find",
        HWPX_NEEDLE,
        "--replace",
        "지방청",
        "-o",
        "out.hwp",
        "--json",
    ];
    let output = run_in(&work.dir, &args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);

    // 경로를 몰래 바꾸지 않는다 — 지정한 문자열 그대로 보고하고 그 자리에 만든다.
    assert_eq!(v["output"], "out.hwp", "{v}");
    assert_eq!(v["outputFormat"], "hwp5", "{v}");
    assert!(work.path("out.hwp").exists(), "{v}");
    assert!(
        !work.path("out.hwpx").exists(),
        "확장자를 임의로 바꿔 쓰면 안 됩니다"
    );
    assert_eq!(
        format_of(&work.dir, &work.path("out.hwp")),
        "hwp5",
        "재독 형식 불일치"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("경고") && stderr.contains(".hwp"),
        "형식이 바뀌면 stderr 로 경고해야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// 반대 방향(HWP 입력 + `-o ….hwpx`)은 형식을 바꾸지 않는다 — 경고만 하고 HWP5 를 쓴다.
/// 형식 변환은 `edit` 이 아니라 `export-hwpx` 의 책임이다.
#[test]
fn explicit_hwpx_output_for_hwp_input_warns_and_keeps_hwp5() {
    let work = Workdir::new("explicit-hwpx");
    let input = work.copy_in(HWP_SAMPLE, "input.hwp");

    let args = [
        "edit",
        "replace-text",
        input.to_str().unwrap(),
        "--find",
        HWP_NEEDLE,
        "--replace",
        "기관",
        "-o",
        "out.hwpx",
        "--json",
    ];
    let output = run_in(&work.dir, &args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["output"], "out.hwpx", "{v}");
    assert_eq!(v["outputFormat"], "hwp5", "{v}");
    assert_eq!(
        format_of(&work.dir, &work.path("out.hwpx")),
        "hwp5",
        "확장자만 .hwpx 일 뿐 내용은 HWP5 여야 합니다"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("경고"),
        "확장자와 실제 형식이 다르면 경고해야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// `--dry-run` 은 형식이 무엇이든 파일을 만들지 않고 `output`/`outputFormat` 도 보고하지 않는다.
#[test]
fn dry_run_reports_no_output_format_and_writes_nothing() {
    let work = Workdir::new("dry");
    let input = work.copy_in(HWPX_SAMPLE, "input.hwpx");

    let args = [
        "edit",
        "replace-text",
        input.to_str().unwrap(),
        "--find",
        HWPX_NEEDLE,
        "--replace",
        "지방청",
        "--dry-run",
        "--json",
    ];
    let v = run_ok(&work.dir, &args);
    assert_eq!(v["dryRun"], true, "{v}");
    assert!(v.get("output").is_none(), "{v}");
    assert!(v.get("outputFormat").is_none(), "{v}");
    assert!(!work.path("input_replaced.hwpx").exists());
    assert!(!work.path("input_replaced.hwp").exists());
}
