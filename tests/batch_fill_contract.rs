//! [#3719 §6-6] `batch fill` — 서식 1 + 데이터 N행 → 산출 N개 (진짜 메일머지) 계약 테스트.
//!
//! 핵심 계약:
//! ① 행마다 NDJSON 레코드 하나, **실패한 행도 스트림에 남는다**(사라지면 처리 누락을
//!    셀 수 없다) ② 성공 레코드는 단건 `edit fill-fields --json` 봉투 + `row`
//! ③ 산출 이름이 겹쳐도 덮어쓰지 않는다 ④ 데이터는 stdin 이 아니라 `--data` **파일**이다
//! ⑤ 종료 코드: 전부 성공 0 / 한 행이라도 실패 1 / 인자 오류 2 / verify 불일치 3.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 누름틀을 가진 서식.
const FORM: &str = "samples/field-01.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        // fill 축은 stdin 을 읽지 않는다. null 로 두면 실수로 읽는 순간 즉시 EOF 라
        // "무한 대기"가 아니라 재현 가능한 실패로 드러난다.
        .stdin(Stdio::null())
        .output()
        .expect("rhwp 실행 실패")
}

/// 잘못 해석된 출력 경로가 저장소 루트에 파일을 만들지 않도록 실행 위치를 격리한다.
fn run_in_dir(args: &[&str], current_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .output()
        .expect("rhwp 실행 실패")
}

/// stdin 에 본문을 흘려 넣고 실행한다. fill 축이 stdin 을 **읽지 않음**을 증명할 때 쓴다.
fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    // 자식이 stdin 을 읽지 않고 끝내는 것이 정상 경로다 — BrokenPipe 는 무시한다.
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn ndjson(args: &[&str], output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}\n{}", describe(args, output)))
        })
        .collect()
}

fn json_of(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("JSON 아님 ({e}): {}", describe(args, output)))
}

/// 산출물을 쓰는 축이라 테스트마다 격리된 임시 폴더를 쓰고, 실패 assertion 뒤에도 지운다.
struct TmpDir(PathBuf);

impl TmpDir {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rhwp-batch-fill-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("임시 폴더 생성 실패");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// 산출 폴더. 미리 만들지 않는다 — `--dry-run` 이 폴더조차 만들지 않음을 볼 수 있게.
    fn out_dir(&self) -> PathBuf {
        self.0.join("out")
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 서식에 **한 번만** 나오는 누름틀 이름들. 테스트 데이터를 하드코딩하지 않고 실제 문서에서
/// 얻는다 — 샘플이 바뀌면 테스트가 조용히 무의미해지는 것을 막는다. 같은 이름이 여러 번 있는
/// 필드는 ambiguous 축(#3476)이라 여기의 관심사가 아니다.
fn unique_field_names() -> Vec<String> {
    let form = sample(FORM);
    let args = ["fields", form.to_str().expect("경로"), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = json_of(&args, &output);

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for f in v["fields"].as_array().expect("fields 배열") {
        let Some(name) = f["name"].as_str() else {
            continue;
        };
        if !counts.contains_key(name) {
            order.push(name.to_string());
        }
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    let names: Vec<String> = order.into_iter().filter(|n| counts[n] == 1).collect();
    assert!(
        names.len() >= 2,
        "이 테스트에는 한 번만 나오는 누름틀이 2개 이상 필요합니다: {names:?}"
    );
    names
}

/// `{이름: 값}` 한 행을 JSONL 한 줄로.
fn jsonl_line(pairs: &[(&str, &str)]) -> String {
    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(
            (*k).to_string(),
            serde_json::Value::String((*v).to_string()),
        );
    }
    format!("{}\n", serde_json::Value::Object(map))
}

fn written_files(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

// ── 기본 축: 행 N개 → 산출 N개 ─────────────────────────────────────────────

#[test]
fn jsonl_rows_become_one_document_each() {
    let names = unique_field_names();
    let tmp = TmpDir::new("jsonl");
    let data = tmp.join("rows.jsonl");
    let body: String = ["가나다 주식회사", "라마바 주식회사", "사아자 협동조합"]
        .iter()
        .enumerate()
        .map(|(i, v)| jsonl_line(&[(&names[0], v), (&names[1], &format!("문서 {i}"))]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 3, "{}", describe(&args, &output));
    for (i, v) in records.iter().enumerate() {
        assert_eq!(v["schemaVersion"], "1.0", "{v}");
        // row 는 성공·실패 어느 쪽에도 붙는다 — 없으면 어느 행이 빠졌는지 셀 수 없다.
        assert_eq!(v["row"], i, "행 번호가 입력 순서와 달라졌습니다: {v}");
        assert!(v.get("error").is_none(), "{v}");
        assert_eq!(v["filledCount"], 2, "{v}");
        assert!(
            v["notFound"].as_array().expect("notFound").is_empty(),
            "{v}"
        );
        let written = v["output"].as_str().expect("output");
        assert!(
            Path::new(written).is_file(),
            "레코드가 가리키는 산출물이 없습니다: {written}\n{}",
            describe(&args, &output)
        );
    }
    assert_eq!(
        written_files(&out).len(),
        3,
        "행 수와 산출물 수가 다릅니다: {:?}",
        written_files(&out)
    );
}

#[test]
fn record_is_isomorphic_to_single_command_envelope() {
    // 배치 레코드 = 단건 `edit fill-fields --json` 봉투 + row. 소비자가 단건과 배치를
    // 같은 코드로 읽는 것이 기존 batch 축 규약이다.
    let names = unique_field_names();
    let tmp = TmpDir::new("iso");
    let data = tmp.join("row.jsonl");
    std::fs::write(&data, jsonl_line(&[(&names[0], "값")])).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let batch_args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let batch_output = run(&batch_args);
    assert_eq!(
        batch_output.status.code(),
        Some(0),
        "{}",
        describe(&batch_args, &batch_output)
    );
    let record = ndjson(&batch_args, &batch_output)
        .into_iter()
        .next()
        .expect("레코드 1건");

    let single_out = tmp.join("single.hwp");
    let inline = {
        let mut m = serde_json::Map::new();
        m.insert(
            names[0].clone(),
            serde_json::Value::String("값".to_string()),
        );
        serde_json::Value::Object(m).to_string()
    };
    let single_args = [
        "edit",
        "fill-fields",
        form.to_str().expect("경로"),
        "--data",
        inline.as_str(),
        "-o",
        single_out.to_str().expect("경로"),
        "--json",
    ];
    let single_output = run(&single_args);
    assert_eq!(
        single_output.status.code(),
        Some(0),
        "{}",
        describe(&single_args, &single_output)
    );
    let single = json_of(&single_args, &single_output);

    let mut batch_keys: Vec<&String> = record.as_object().expect("객체").keys().collect();
    batch_keys.retain(|k| k.as_str() != "row");
    let single_keys: Vec<&String> = single.as_object().expect("객체").keys().collect();
    assert_eq!(
        batch_keys, single_keys,
        "배치 레코드는 단건 봉투와 같은 필드여야 합니다(row 제외)\n배치: {record}\n단건: {single}"
    );
    assert_eq!(record["filledCount"], single["filledCount"], "{record}");
}

// ── CSV 축 ────────────────────────────────────────────────────────────────

#[test]
fn csv_reads_bom_and_rfc4180_quoting() {
    // 엑셀 저장본을 그대로 받는다: UTF-8 BOM · CRLF · 따옴표 안의 쉼표/줄바꿈/이중 따옴표.
    // BOM 을 남기면 첫 헤더 이름이 통째로 어긋나 그 열이 조용히 notFound 가 된다.
    let names = unique_field_names();
    let tmp = TmpDir::new("csv");
    let data = tmp.join("rows.csv");
    let body = format!(
        "\u{feff}{},{}\r\n\"쉼표, 포함\",\"따옴표 \"\"안\"\" 값\"\r\n\"줄1\n줄2\",평범\r\n",
        names[0], names[1]
    );
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));

    let value_of = |record: &serde_json::Value, name: &str| -> String {
        record["filled"]
            .as_array()
            .expect("filled")
            .iter()
            .find(|f| f["name"] == name)
            .unwrap_or_else(|| panic!("{name} 이 채워지지 않았습니다: {record}"))["value"]
            .as_str()
            .expect("value")
            .to_string()
    };
    assert_eq!(value_of(&records[0], &names[0]), "쉼표, 포함");
    assert_eq!(value_of(&records[0], &names[1]), "따옴표 \"안\" 값");
    assert_eq!(value_of(&records[1], &names[0]), "줄1\n줄2");
    // BOM 이 첫 헤더에 붙어 있었다면 그 열은 notFound 가 된다.
    for v in &records {
        assert!(
            v["notFound"].as_array().expect("notFound").is_empty(),
            "BOM·인용 처리가 어긋나 매칭되지 않은 열이 있습니다: {v}"
        );
    }
}

#[test]
fn csv_column_count_mismatch_is_a_row_error_not_a_shifted_document() {
    // 칸 수가 다르면 값이 한 칸씩 밀려 **오류 없이 잘못된 문서**가 나온다. 행 단위로 거부하되
    // 스트림에는 남긴다.
    let names = unique_field_names();
    let tmp = TmpDir::new("csvlen");
    let data = tmp.join("rows.csv");
    let body = format!("{},{}\n정상A,정상B\n칸이,너무,많다\n", names[0], names[1]);
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "행 하나가 실패하면 exit 1 이어야 합니다\n{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    assert!(records[0].get("error").is_none(), "{}", records[0]);
    assert!(
        records[1]["error"].as_str().is_some(),
        "칸 수 불일치 행이 error 레코드가 아닙니다: {}",
        records[1]
    );
    assert_eq!(records[1]["row"], 1, "{}", records[1]);
    assert_eq!(written_files(&out).len(), 1, "{:?}", written_files(&out));
}

// ── 산출 이름 ─────────────────────────────────────────────────────────────

#[test]
fn duplicate_name_field_values_do_not_overwrite_each_other() {
    // 같은 이름이면 뒤 행이 앞 행을 덮어써 **조용히** 데이터가 사라진다 — 성공 레코드 N건과
    // 실제 파일 수가 어긋나면 안 된다.
    let names = unique_field_names();
    let tmp = TmpDir::new("dup");
    let data = tmp.join("rows.jsonl");
    let body: String = ["일차", "이차", "삼차"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], "홍길동"), (&names[1], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        names[0].as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    let outputs: Vec<&str> = records
        .iter()
        .map(|v| v["output"].as_str().expect("output"))
        .collect();
    let mut unique: Vec<String> = outputs.iter().map(|s| s.to_lowercase()).collect();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        3,
        "같은 이름의 행이 같은 경로를 가리켰습니다: {outputs:?}"
    );
    assert_eq!(
        written_files(&out).len(),
        3,
        "덮어쓰기로 산출물이 사라졌습니다: {:?}",
        written_files(&out)
    );
}

#[test]
fn name_field_cannot_escape_the_output_directory() {
    // 파일 이름은 데이터에서 온다. 경로 구분자를 그대로 두면 `--out-dir` 밖에 파일이 생긴다.
    let names = unique_field_names();
    let tmp = TmpDir::new("escape");
    let data = tmp.join("rows.jsonl");
    std::fs::write(
        &data,
        jsonl_line(&[(&names[0], "../../탈출"), (&names[1], "값")]),
    )
    .expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        names[0].as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let record = ndjson(&args, &output).into_iter().next().expect("레코드");
    let written = PathBuf::from(record["output"].as_str().expect("output"));
    assert_eq!(
        written.parent(),
        Some(out.as_path()),
        "산출물이 --out-dir 밖으로 나갔습니다: {}",
        written.display()
    );
    assert_eq!(written_files(&out).len(), 1, "{:?}", written_files(&out));
}

// ── 실패·선검증·입력 축 ────────────────────────────────────────────────────

#[test]
fn broken_rows_stay_in_the_stream_and_force_exit_1() {
    // 실패 행이 스트림에서 사라지면 소비자는 N행을 넣고 N-2건을 받고도 성공으로 읽는다.
    let names = unique_field_names();
    let tmp = TmpDir::new("broken");
    let data = tmp.join("rows.jsonl");
    let body = format!(
        "{}이건 JSON 이 아니다\n[\"배열은 객체가 아니다\"]\n{}",
        jsonl_line(&[(&names[0], "정상 앞")]),
        jsonl_line(&[(&names[0], "정상 뒤")]),
    );
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(
        records.len(),
        4,
        "실패 행이 스트림에서 사라졌습니다\n{}",
        describe(&args, &output)
    );
    for (i, v) in records.iter().enumerate() {
        assert_eq!(v["row"], i, "{v}");
    }
    assert!(records[0].get("error").is_none(), "{}", records[0]);
    for bad in [&records[1], &records[2]] {
        assert!(bad["error"].as_str().is_some(), "{bad}");
        assert_eq!(bad["exitClass"], "runtime", "{bad}");
        assert!(
            bad.get("output").is_none(),
            "실패 행에 산출 경로가 붙으면 안 됩니다: {bad}"
        );
    }
    assert!(records[3].get("error").is_none(), "{}", records[3]);
    assert_eq!(written_files(&out).len(), 2, "{:?}", written_files(&out));
}

#[test]
fn dry_run_writes_nothing_and_exits_zero() {
    let names = unique_field_names();
    let tmp = TmpDir::new("dryrun");
    let data = tmp.join("rows.jsonl");
    let body: String = ["하나", "둘"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    for v in &records {
        assert_eq!(v["dryRun"], true, "{v}");
        // 목적지는 밝히되 `dryRun: true` 가 함께 있어 "만들 예정" 임이 구분된다.
        let planned = v["output"].as_str().expect("output");
        assert!(
            !Path::new(planned).exists(),
            "--dry-run 이 파일을 만들었습니다: {planned}"
        );
        assert_eq!(v["filledCount"], 1, "{v}");
    }
    assert!(
        !out.exists(),
        "--dry-run 이 출력 폴더를 만들었습니다: {}",
        out.display()
    );
}

#[test]
fn data_comes_from_the_file_not_stdin() {
    // 다른 batch 축은 stdin 으로 경로 목록을 받는다. fill 은 읽지 않아야 한다 — 읽으면
    // MCP 서버의 프로토콜 stdin 을 자식이 삼키는 형태의 사고가 난다.
    let names = unique_field_names();
    let tmp = TmpDir::new("stdin");
    let data = tmp.join("rows.jsonl");
    std::fs::write(&data, jsonl_line(&[(&names[0], "파일에서 온 값")])).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run_with_stdin(&args, "이 줄은 경로도 데이터도 아니다\n또 한 줄\n");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdin 의 쓰레기 입력이 결과를 바꿨습니다\n{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 1, "{}", describe(&args, &output));
    assert_eq!(records[0]["filledCount"], 1, "{}", records[0]);
}

#[test]
fn verify_verdict_is_data_and_drives_exit_3() {
    let names = unique_field_names();
    let tmp = TmpDir::new("verify");
    let data = tmp.join("rows.jsonl");
    std::fs::write(&data, jsonl_line(&[(&names[0], "검증")])).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 1, "{}", describe(&args, &output));
    let identical = records[0]["verify"]["identical"]
        .as_bool()
        .unwrap_or_else(|| panic!("--verify 판정이 데이터로 오지 않았습니다: {}", records[0]));
    // 판정은 실패가 아니다 — 저장은 됐고 산출물도 있다. 3 은 "검토 대상", 1 은 "재실행 대상".
    let expected = if identical { 0 } else { 3 };
    assert_eq!(
        output.status.code(),
        Some(expected),
        "verify 판정과 종료 코드가 어긋났습니다\n{}",
        describe(&args, &output)
    );
    assert!(
        Path::new(records[0]["output"].as_str().expect("output")).is_file(),
        "검증 판정과 무관하게 산출물은 남아야 합니다: {}",
        records[0]
    );
}

// ── 부분 성공의 봉투 표현 ──────────────────────────────────────────────────

#[test]
fn unknown_field_names_are_partial_success_not_row_failure() {
    // 서식에 없는 이름은 **그 행의 실패가 아니다** — 나머지는 채워진 문서가 나온다.
    // 대신 notFound 에 남아 "완성본"으로 오해할 수 없게 한다. 이것을 error 로 접으면
    // 소비자가 재실행 대상(1)과 검토 대상을 구분할 수 없다.
    let names = unique_field_names();
    let tmp = TmpDir::new("partial");
    let data = tmp.join("rows.jsonl");
    std::fs::write(
        &data,
        jsonl_line(&[
            (&names[0], "채워짐"),
            ("이_문서에_없는_필드", "무시됨"),
            ("또_없는_필드", "무시됨"),
        ]),
    )
    .expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "없는 필드 이름은 행 실패가 아닙니다\n{}",
        describe(&args, &output)
    );

    let record = ndjson(&args, &output).into_iter().next().expect("레코드");
    assert!(record.get("error").is_none(), "{record}");
    assert_eq!(
        record["filledCount"], 1,
        "실제로 채운 수여야 합니다: {record}"
    );
    let not_found: Vec<&str> = record["notFound"]
        .as_array()
        .expect("notFound")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(not_found.len(), 2, "{record}");
    for missing in ["이_문서에_없는_필드", "또_없는_필드"] {
        assert!(
            not_found.contains(&missing),
            "{missing} 이 notFound 에 없습니다: {record}"
        );
    }
    assert!(
        Path::new(record["output"].as_str().expect("output")).is_file(),
        "부분 성공은 산출물을 남깁니다: {record}"
    );
}

#[test]
fn filled_values_survive_reparse_in_every_output() {
    // 레코드가 "채웠다"고 말하는 것과 문서에 실제로 들어간 것은 다른 축이다.
    // 산출물을 다시 열어 값이 살아 있는지 본다 — 여기서 끊기면 메일머지 전체가 무의미하다.
    let names = unique_field_names();
    let tmp = TmpDir::new("reparse");
    let data = tmp.join("rows.jsonl");
    let values = ["첫째 값", "둘째 값", "셋째 값"];
    let body: String = values
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), values.len(), "{}", describe(&args, &output));
    for (record, expected) in records.iter().zip(values.iter()) {
        let written = record["output"].as_str().expect("output");
        let probe = ["fields", written, "--json"];
        let probe_out = run(&probe);
        assert_eq!(
            probe_out.status.code(),
            Some(0),
            "산출물을 다시 열 수 없습니다\n{}",
            describe(&probe, &probe_out)
        );
        let reparsed = json_of(&probe, &probe_out);
        let value = reparsed["fields"]
            .as_array()
            .expect("fields")
            .iter()
            .find(|f| f["name"] == names[0].as_str())
            .unwrap_or_else(|| panic!("{} 필드가 산출물에 없습니다: {reparsed}", names[0]))
            ["value"]
            .as_str()
            .unwrap_or("");
        assert_eq!(
            value, *expected,
            "산출물의 값이 데이터 행과 다릅니다: {written}"
        );
    }
}

// ── 산출 이름 충돌·정규화 ──────────────────────────────────────────────────

#[test]
fn case_only_collisions_get_distinct_files() {
    // Windows·macOS 기본 파일시스템은 대소문자를 구분하지 않는다. 소문자 키로 판정하지
    // 않으면 그 OS 에서만 산출물이 사라져, Linux CI 는 통과하고 사용자만 데이터를 잃는다.
    let names = unique_field_names();
    let tmp = TmpDir::new("case");
    let data = tmp.join("rows.jsonl");
    let body: String = ["Alpha", "alpha", "ALPHA"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        names[0].as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert_eq!(
        written_files(&out).len(),
        3,
        "대소문자만 다른 이름이 서로를 덮어썼습니다: {:?}",
        written_files(&out)
    );
}

#[test]
fn windows_reserved_device_names_still_produce_a_file() {
    // CON·NUL 등은 Windows 에서 파일로 만들 수 없다. 그대로 쓰면 그 행만 조용히
    // 실패하거나(플랫폼별 분기) 장치에 쓰게 된다.
    let names = unique_field_names();
    let tmp = TmpDir::new("reserved");
    let data = tmp.join("rows.jsonl");
    let body: String = ["CON", "NUL", "com1"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        names[0].as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "예약 장치 이름에서 실패했습니다\n{}",
        describe(&args, &output)
    );
    let files = written_files(&out);
    assert_eq!(files.len(), 3, "{files:?}");
    for f in &files {
        let stem = f.split('.').next().unwrap_or("").to_ascii_uppercase();
        assert!(
            !matches!(stem.as_str(), "CON" | "NUL" | "COM1"),
            "예약 장치 이름이 그대로 쓰였습니다: {f}"
        );
    }
}

#[test]
fn very_long_name_field_value_is_truncated_to_a_writable_name() {
    // Windows 경로 한도(260) 안에 들어와야 한다. 자르지 않으면 그 행만 쓰기 실패한다.
    let names = unique_field_names();
    let tmp = TmpDir::new("long");
    let data = tmp.join("rows.jsonl");
    let long = "가".repeat(400);
    std::fs::write(&data, jsonl_line(&[(&names[0], long.as_str())])).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        names[0].as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "긴 이름에서 실패했습니다\n{}",
        describe(&args, &output)
    );
    let files = written_files(&out);
    assert_eq!(files.len(), 1, "{files:?}");
    let stem_chars = files[0].chars().count();
    assert!(
        stem_chars <= 100,
        "이름이 잘리지 않았습니다({stem_chars}자): {}",
        files[0]
    );
}

#[test]
fn unknown_name_field_falls_back_to_sequence_numbers() {
    // 이름 필드를 잘못 적었을 때 전부 같은 이름으로 몰아 덮어쓰면 안 된다.
    let names = unique_field_names();
    let tmp = TmpDir::new("noname");
    let data = tmp.join("rows.jsonl");
    let body: String = ["하나", "둘", "셋"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--name-field",
        "존재하지_않는_필드",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let files = written_files(&out);
    assert_eq!(files.len(), 3, "순번 폴백이 겹쳤습니다: {files:?}");
    for (i, f) in files.iter().enumerate() {
        assert!(
            f.starts_with(&format!("000{}", i + 1)),
            "순번 이름이 아닙니다: {files:?}"
        );
    }
}

// ── 실패의 전파 방식: 중단이 아니라 계속 ────────────────────────────────────

#[test]
fn many_broken_rows_do_not_abort_the_run() {
    // 계약의 요점: 실패는 **레코드**이고 프로세스는 끝까지 간다. 첫 실패에서 끊으면
    // 뒤 행의 산출물이 통째로 사라지는데, 스트림만 보면 그 사실을 알 수 없다.
    let names = unique_field_names();
    let tmp = TmpDir::new("manybroken");
    let data = tmp.join("rows.jsonl");
    let mut body = String::new();
    let mut expect_ok = Vec::new();
    for i in 0..8 {
        if i % 2 == 0 {
            body.push_str(&jsonl_line(&[(&names[0], &format!("정상 {i}"))]));
            expect_ok.push(i);
        } else {
            body.push_str(&format!("깨진 행 {i}\n"));
        }
    }
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        form.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(
        records.len(),
        8,
        "실패가 여러 건이어도 8행 전부가 스트림에 있어야 합니다\n{}",
        describe(&args, &output)
    );
    for (i, v) in records.iter().enumerate() {
        assert_eq!(v["row"], i, "행 번호가 밀렸습니다: {v}");
        let is_ok = expect_ok.contains(&i);
        assert_eq!(
            v.get("error").is_none(),
            is_ok,
            "행 {i} 의 성패가 뒤바뀌었습니다: {v}"
        );
    }
    assert_eq!(
        written_files(&out).len(),
        4,
        "실패 뒤의 정상 행이 처리되지 않았습니다: {:?}",
        written_files(&out)
    );
}

#[test]
fn unreadable_form_fails_before_any_row_and_writes_nothing() {
    // 서식은 행마다 열린다. 못 여는 서식이면 같은 실패를 N번 보고하게 되므로 —
    // 그건 진단이 아니다 — 한 행을 처리하기 전에 끝낸다.
    let names = unique_field_names();
    let tmp = TmpDir::new("noform");
    let data = tmp.join("rows.jsonl");
    let body: String = ["가", "나", "다"]
        .iter()
        .map(|v| jsonl_line(&[(&names[0], v)]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let missing = tmp.join("없는서식.hwp");
    let out = tmp.out_dir();
    let args = [
        "batch",
        "fill",
        "--form",
        missing.to_str().expect("경로"),
        "--data",
        data.to_str().expect("경로"),
        "--out-dir",
        out.to_str().expect("경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "한 행도 처리하지 못했는데 stdout 에 레코드가 있습니다\n{}",
        describe(&args, &output)
    );
    assert!(!out.exists(), "출력 폴더가 생겼습니다: {}", out.display());
}

#[test]
fn threads_do_not_change_row_order_or_output_names() {
    // 병렬로 돌려도 방출은 입력 순서다(기존 batch 규약). 산출 이름은 행 순서만으로
    // 정해지므로 스레드 수를 바꾼 재실행이 달라지면 안 된다.
    let names = unique_field_names();
    let tmp = TmpDir::new("threads");
    let data = tmp.join("rows.jsonl");
    let body: String = (0..12)
        .map(|i| jsonl_line(&[(&names[0], &format!("행 {i:02}"))]))
        .collect();
    std::fs::write(&data, body).expect("데이터 파일 쓰기");

    let form = sample(FORM);
    let mut signatures = Vec::new();
    for threads in ["1", "8"] {
        let out = tmp.join(&format!("out{threads}"));
        let args = [
            "batch",
            "fill",
            "--form",
            form.to_str().expect("경로"),
            "--data",
            data.to_str().expect("경로"),
            "--out-dir",
            out.to_str().expect("경로"),
            "--threads",
            threads,
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&args, &output)
        );
        let records = ndjson(&args, &output);
        assert_eq!(records.len(), 12, "{}", describe(&args, &output));
        let signature: Vec<(u64, String, String)> = records
            .iter()
            .map(|v| {
                (
                    v["row"].as_u64().expect("row"),
                    v["filled"][0]["value"].as_str().unwrap_or("").to_string(),
                    Path::new(v["output"].as_str().expect("output"))
                        .file_name()
                        .expect("파일명")
                        .to_string_lossy()
                        .to_string(),
                )
            })
            .collect();
        for (i, (row, _, _)) in signature.iter().enumerate() {
            assert_eq!(*row as usize, i, "threads={threads} 에서 순서가 깨졌습니다");
        }
        assert_eq!(written_files(&out).len(), 12, "{:?}", written_files(&out));
        signatures.push(signature);
    }
    assert_eq!(
        signatures[0], signatures[1],
        "스레드 수에 따라 결과가 달라졌습니다"
    );
}

// ── 인자 오류(2) ──────────────────────────────────────────────────────────

#[test]
fn missing_or_malformed_arguments_are_usage_errors() {
    let names = unique_field_names();
    let tmp = TmpDir::new("usage");
    let form = sample(FORM);
    let form_s = form.to_str().expect("경로").to_string();
    let out = tmp.out_dir();
    let out_s = out.to_str().expect("경로").to_string();

    let good = tmp.join("rows.jsonl");
    std::fs::write(&good, jsonl_line(&[(&names[0], "값")])).expect("데이터 파일 쓰기");
    let good_s = good.to_str().expect("경로").to_string();

    // 확장자로 형식을 정한다 — 모르는 확장자를 추측해서 읽지 않는다.
    let unknown_ext = tmp.join("rows.txt");
    std::fs::write(&unknown_ext, "아무거나\n").expect("데이터 파일 쓰기");
    // 0행을 성공(0)으로 끝내면 "전부 처리했다"와 구분되지 않는다.
    let empty = tmp.join("empty.jsonl");
    std::fs::write(&empty, "\n\n").expect("데이터 파일 쓰기");
    // 따옴표가 닫히지 않으면 뒤 행 전체가 한 칸으로 삼켜진다.
    let unclosed = tmp.join("unclosed.csv");
    std::fs::write(&unclosed, format!("{}\n\"열린 따옴표\n", names[0])).expect("데이터 파일 쓰기");
    // 같은 헤더가 둘이면 한 열이 통째로 무시된다.
    let dup_header = tmp.join("dup.csv");
    std::fs::write(&dup_header, format!("{0},{0}\n가,나\n", names[0])).expect("데이터 파일 쓰기");

    let cases: Vec<(&str, Vec<String>)> = vec![
        (
            "--form 없음",
            vec![
                "batch",
                "fill",
                "--data",
                &good_s,
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "--data 없음",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "--out-dir 없음",
            vec![
                "batch", "fill", "--form", &form_s, "--data", &good_s, "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "--json 없음",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                &good_s,
                "--out-dir",
                &out_s,
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "--out-dir 값 자리에 플래그",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                &good_s,
                "--out-dir",
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "모르는 확장자",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                unknown_ext.to_str().expect("경로"),
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "데이터 0행",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                empty.to_str().expect("경로"),
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "CSV 따옴표 미종료",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                unclosed.to_str().expect("경로"),
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
        (
            "CSV 헤더 중복",
            vec![
                "batch",
                "fill",
                "--form",
                &form_s,
                "--data",
                dup_header.to_str().expect("경로"),
                "--out-dir",
                &out_s,
                "--json",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
    ];

    for (label, argv) in cases {
        let args: Vec<&str> = argv.iter().map(String::as_str).collect();
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} 은 사용법 오류(2)여야 합니다\n{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{label}: 인자 오류에서 stdout 은 0바이트여야 합니다\n{}",
            describe(&args, &output)
        );
        assert!(
            !out.exists(),
            "{label}: 인자 오류인데 출력 폴더가 생겼습니다"
        );
    }
}

#[test]
fn fill_flags_are_axis_scoped() {
    // `--form`·`--name-field`·`--dry-run` 은 fill 축 전용이고, 다른 축의 전용 플래그는
    // fill 에서 거부된다 (`--query`·`--mode` 와 같은 규약).
    let tmp = TmpDir::new("scope");
    let form = sample(FORM);
    let form_s = form.to_str().expect("경로").to_string();

    for extra in [
        vec!["--form", &form_s],
        vec!["--name-field", "이름"],
        vec!["--dry-run"],
    ] {
        let mut args = vec!["batch", "info", "--json"];
        args.extend(extra.iter().map(|s| &**s));
        let output = run_in_dir(&args, tmp.path());
        assert_eq!(
            output.status.code(),
            Some(2),
            "fill 전용 플래그가 다른 축에서 통과했습니다\n{}",
            describe(&args, &output)
        );
    }

    for extra in [
        vec!["--mode", "auto"],
        vec!["--query", "가"],
        vec!["--verify-pages"],
    ] {
        let mut args = vec!["batch", "fill", "--json"];
        args.extend(extra.iter().map(|s| &**s));
        let output = run_in_dir(&args, tmp.path());
        assert_eq!(
            output.status.code(),
            Some(2),
            "다른 축의 플래그가 fill 에서 통과했습니다\n{}",
            describe(&args, &output)
        );
    }
}

// ── 자기서술 드리프트 가드 ─────────────────────────────────────────────────

fn capabilities() -> serde_json::Value {
    let args = ["capabilities"];
    json_of(&args, &run(&args))
}

fn capabilities_mcp() -> serde_json::Value {
    let args = ["capabilities", "--mcp"];
    json_of(&args, &run(&args))
}

#[test]
fn capabilities_declares_the_fill_axis() {
    let v = capabilities();
    let subs: Vec<&str> = v["batch"]["subcommands"]
        .as_array()
        .expect("batch.subcommands")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(subs.contains(&"fill"), "batch 축에 fill 누락: {subs:?}");

    let axis: Vec<&str> = v["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for expected in [
        "--form",
        "--name-field",
        "--dry-run",
        "--out-dir",
        "--verify",
    ] {
        assert!(
            axis.contains(&expected),
            "batch.flags 에 {expected} 누락: {axis:?}"
        );
    }

    // 축 선언과 명령 항목 선언이 어긋나면 소비자는 어느 쪽도 믿을 수 없다
    // (cli_json_contract::capabilities_declared_flags_are_real_cli_flags 와 같은 규약).
    let entry: Vec<&str> = v["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "batch")
        .expect("batch 항목")["flags"]
        .as_array()
        .expect("commands[batch].flags")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    let missing: Vec<&str> = axis
        .iter()
        .copied()
        .filter(|f| !entry.contains(f))
        .collect();
    assert!(
        missing.is_empty(),
        "batch.flags 에는 있는데 commands[batch].flags 에 없는 플래그: {missing:?}"
    );

    // fill 은 stdin 을 읽지 않는다는 사실이 자기서술에 있어야, 매니페스트만 읽은
    // 에이전트가 stdin 에 경로를 밀어 넣고 "왜 아무것도 안 되지" 를 겪지 않는다.
    let input = v["batch"]["input"].as_str().expect("batch.input");
    assert!(
        input.contains("fill"),
        "fill 축의 다른 입력 축이 batch.input 에 없습니다: {input}"
    );
}

#[test]
fn declared_batch_flags_are_accepted_by_some_axis() {
    // 드리프트 가드: 선언한 플래그는 **실제로 수용**돼야 한다. 매니페스트는 에이전트가
    // 도구 정의를 자동 생성하는 원천이라, 여기 있는데 CLI 가 모르는 플래그는 그 에이전트를
    // 영영 exit 2 에 가둔다.
    let v = capabilities();
    let flags: Vec<String> = v["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|s| s.as_str())
        .map(String::from)
        .collect();
    let subs: Vec<String> = v["batch"]["subcommands"]
        .as_array()
        .expect("batch.subcommands")
        .iter()
        .filter_map(|s| s.as_str())
        .map(String::from)
        .collect();
    assert!(!flags.is_empty() && !subs.is_empty(), "가드가 공허합니다");

    // 값이 필요한 플래그에는 아무 값이나 붙인다. 어차피 `--json` 을 주지 않으므로 어떤
    // 축도 실제 작업을 시작하지 않는다 — 인자 파서만 통과하는지 본다(부작용 없음).
    let needs_value =
        |flag: &str| !matches!(flag, "--json" | "--verify" | "--verify-pages" | "--dry-run");
    let tmp = TmpDir::new("flagprobe");

    for flag in &flags {
        let accepted = subs.iter().any(|sub| {
            let mut args = vec!["batch", sub.as_str(), flag.as_str()];
            if needs_value(flag) {
                args.push("1");
            }
            let output = run_in_dir(&args, tmp.path());
            !String::from_utf8_lossy(&output.stderr).contains(&format!("알 수 없는 옵션: {flag}"))
        });
        assert!(
            accepted,
            "capabilities 가 선언한 {flag} 를 받아 주는 batch 축이 하나도 없습니다"
        );
    }
}

#[test]
fn mcp_declares_batch_fill_and_wires_every_input() {
    let v = capabilities_mcp();
    let tools = v["tools"].as_array().expect("tools");
    let tool = tools
        .iter()
        .find(|t| t["name"] == "hwp_batch_fill")
        .unwrap_or_else(|| panic!("hwp_batch_fill 도구 누락: {v}"));

    let schema = &tool["inputSchema"];
    assert_eq!(schema["type"], "object", "{tool}");
    let props = schema["properties"].as_object().expect("properties");
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required 는 배열이어야 합니다")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for key in ["form", "data", "outDir"] {
        assert!(required.contains(&key), "required 에 {key} 누락: {tool}");
    }

    // 선언만 하고 배선하지 않으면 서버는 그 인자를 조용히 버린 채 성공을 보고한다.
    let mut wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .expect("cli.args")
        .iter()
        .filter_map(|a| a.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}') && s.len() > 2)
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();
    for o in tool["cli"]["optionalArgs"]
        .as_array()
        .expect("cli.optionalArgs")
    {
        wired.push(o["when"].as_str().expect("when").to_string());
    }
    for key in props.keys() {
        assert!(
            wired.contains(key),
            "hwp_batch_fill.{key} 가 CLI 인자에 배선되지 않았습니다: {tool}"
        );
    }
    assert_eq!(tool["cli"]["command"], "batch", "{tool}");

    // stdin 도구가 아니다 — 서버가 paths 를 요구하면 fill 은 영영 호출되지 않는다.
    let stdin_tools: Vec<&str> = v["invocation"]["stdinTools"]
        .as_array()
        .expect("invocation.stdinTools")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(
        !stdin_tools.contains(&"hwp_batch_fill"),
        "fill 은 stdin 을 읽지 않는데 stdinTools 에 있습니다: {stdin_tools:?}"
    );
}

#[test]
fn help_documents_the_fill_axis_and_its_different_input() {
    let args = ["--help"];
    let output = run(&args);
    let help = String::from_utf8_lossy(&output.stdout);
    for needle in ["batch fill", "--form", "--name-field", "--data"] {
        assert!(
            help.contains(needle),
            "--help 에 {needle} 가 없습니다 (capabilities 등재 명령은 help 에도 있어야 합니다)"
        );
    }
    assert!(
        help.contains("stdin 을 읽지 않는다"),
        "fill 이 stdin 을 읽지 않는다는 사실이 --help 에 없습니다"
    );
}
