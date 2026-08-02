//! [#3719 §6-10] `extract-data` 출력 계약 회귀 테스트 — 값과 **주소**가 한 몸이다.
//!
//! 이 명령의 존재 이유는 값 옆에 붙는 주소(구역·문단·**쪽**·문자 오프셋)다. 평문을 뽑아
//! 밖에서 정규식을 돌려도 값 자체는 얻지만 근거 제시가 안 된다. 주소가 사라지면 기능
//! 전체가 무의미해지므로 그 정합성을 계약으로 고정한다.
//!
//! 두 번째 축은 **모름의 보존**이다. 두 자리 연도·한글 수사 금액은 정규화할 수 없으므로
//! `normalized: null` 이어야 한다 — 여기서 그럴듯한 값을 지어내면 소비자는 틀린 값을
//! 옳은 값과 구별할 방법이 없다.
//!
//! 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 실제 정부 문서 — 점 구분 날짜(`1949. 7. 15.`)·단위 배수 금액(`3,180백만원`)·
/// 접두 금액(`금113,560원`)·백분율이 모두 실재한다.
const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// 실제 정부 양식 — 날짜 표기가 `2026. 1.`(연·월)뿐이라 부분 날짜 규약의 실증 대상이다.
const SAMPLE_FORM: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";
/// 실제 예산 문서 — `3,180백만원`·`12.03억원` 처럼 단위 배수와 소수가 섞인 금액이 실재한다.
const SAMPLE_BUDGET: &str = "samples/2022년 국립국어원 업무계획.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

fn run_ok(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(output.status.code(), Some(0), "{}", describe(args, &output));
    parse_json(args, &output)
}

/// `info` 의 사람용 출력에서 쪽 수를 읽는다 — 쪽 주소 유효 범위 검증용.
fn page_count(path: &Path) -> u64 {
    let output = run(&["info", path.to_str().unwrap()]);
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find_map(|l| l.strip_prefix("페이지 수:"))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| panic!("info 에서 페이지 수를 찾지 못했습니다:\n{text}"))
}

#[test]
fn extract_data_json_envelope_and_addresses() {
    let p = sample(SAMPLE);
    let args = ["extract-data", p.to_str().unwrap(), "--json"];
    let v = run_ok(&args);

    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["kind"], "all", "{v}");
    let count = v["itemCount"].as_u64().expect("itemCount");
    let items = v["items"].as_array().expect("items 배열");
    assert_eq!(items.len() as u64, count, "{v}");
    assert!(
        count >= 1,
        "실물 정부 문서인데 0건이면 인식 규칙이 죽은 것입니다: {v}"
    );
    assert_eq!(v["totalItemCount"], v["itemCount"], "절단 없이 전량: {v}");
    assert_eq!(v["truncated"], false, "{v}");

    // counts 는 요청한 종류 전부를 담고, 합계가 총량과 같아야 한다.
    let counts = v["counts"].as_object().expect("counts 객체");
    for kind in ["date", "amount", "number"] {
        assert!(counts.contains_key(kind), "counts.{kind} 누락: {v}");
    }
    let sum: u64 = counts.values().filter_map(serde_json::Value::as_u64).sum();
    assert_eq!(sum, count, "counts 합이 itemCount 와 다릅니다: {v}");

    for item in items {
        assert!(item["section"].as_u64().is_some(), "{item}");
        assert!(item["paragraph"].as_u64().is_some(), "{item}");
        assert!(item["charOffset"].as_u64().is_some(), "{item}");
        assert!(item["length"].as_u64().unwrap() >= 1, "{item}");
        assert!(!item["raw"].as_str().unwrap_or("").is_empty(), "{item}");
        // 정규화 실패도 계약이다 — 키 자체가 사라지면 소비자가 구별할 수 없다.
        assert!(
            item.get("normalized").is_some(),
            "normalized 키는 null 이어도 있어야 합니다: {item}"
        );
    }
}

#[test]
fn extract_data_reports_page_within_document_range() {
    // 이 기능의 존재 이유 — 쪽 주소가 실제로, 유효 범위 안에서 나와야 한다.
    let p = sample(SAMPLE);
    let pages = page_count(&p);
    let v = run_ok(&["extract-data", p.to_str().unwrap(), "--json"]);

    let paged: Vec<u64> = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["page"].as_u64())
        .collect();
    assert!(
        !paged.is_empty(),
        "쪽 주소가 하나도 없으면 기능이 무의미합니다: {v}"
    );
    for page in paged {
        assert!(
            page < pages,
            "쪽 {page} 가 문서 쪽 수({pages}) 범위를 벗어납니다"
        );
    }
}

#[test]
fn dates_are_normalized_to_iso_8601() {
    let p = sample(SAMPLE);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "date",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    assert!(
        items.len() >= 100,
        "점 구분 날짜가 다수 실재하는 문서입니다: {}",
        items.len()
    );

    let mut normalized_seen = 0;
    for item in items {
        assert_eq!(item["kind"], "date", "{item}");
        let Some(iso) = item["normalized"].as_str() else {
            // 두 자리 연도는 정규화 불가가 정답이다.
            assert!(item["normalized"].is_null(), "{item}");
            continue;
        };
        normalized_seen += 1;
        // YYYY-MM-DD 또는 부분 날짜 YYYY-MM.
        let parts: Vec<&str> = iso.split('-').collect();
        assert!(
            matches!(parts.len(), 2 | 3),
            "ISO-8601 날짜가 아닙니다: {iso}"
        );
        assert_eq!(parts[0].len(), 4, "{iso}");
        assert_eq!(parts[1].len(), 2, "{iso}");
        let month: u32 = parts[1].parse().unwrap_or_else(|_| panic!("{iso}"));
        assert!((1..=12).contains(&month), "{iso}");
        if let Some(day) = parts.get(2) {
            assert_eq!(day.len(), 2, "{iso}");
            let day: u32 = day.parse().unwrap_or_else(|_| panic!("{iso}"));
            assert!((1..=31).contains(&day), "{iso}");
        }
    }
    assert!(
        normalized_seen >= 100,
        "정규화된 날짜가 너무 적습니다: {normalized_seen}"
    );
}

#[test]
fn year_month_only_notation_is_a_partial_date_not_a_guess() {
    // 이 양식의 유일한 날짜는 `2026. 1.` 이다. 없는 날(日)을 1일로 채우면 조용히 틀린
    // 값이 되므로 부분 날짜 `2026-01` 로 둔다.
    let p = sample(SAMPLE_FORM);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "date",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    let found = items
        .iter()
        .find(|m| m["normalized"] == "2026-01")
        .unwrap_or_else(|| panic!("`2026. 1.` 을 부분 날짜로 뽑지 못했습니다: {v}"));
    assert_eq!(found["raw"], "2026. 1.", "{found}");
    assert!(found["page"].as_u64().is_some(), "쪽 주소 누락: {found}");
}

#[test]
fn amounts_carry_currency_and_exact_integer_value() {
    let p = sample(SAMPLE);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "amount",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    assert!(!items.is_empty(), "금액이 실재하는 문서입니다: {v}");

    for item in items {
        assert_eq!(item["kind"], "amount", "{item}");
        assert_eq!(item["currency"], "KRW", "{item}");
        if !item["normalized"].is_null() {
            assert!(
                item["normalized"].is_i64() || item["normalized"].is_u64(),
                "금액 정규화는 정수여야 합니다: {item}"
            );
        }
    }

    // 실물 표기 검증 — 접두 `금` 은 공백 없이 붙어 나오고, 단위는 배수로 반영된다.
    let prefixed = items
        .iter()
        .find(|m| m["raw"] == "금113,560원")
        .unwrap_or_else(|| panic!("`금113,560원` 을 뽑지 못했습니다: {v}"));
    assert_eq!(prefixed["normalized"], 113_560, "{prefixed}");

    let scaled = items
        .iter()
        .find(|m| m["raw"] == "21,345천원")
        .unwrap_or_else(|| panic!("`21,345천원` 을 뽑지 못했습니다: {v}"));
    assert_eq!(scaled["normalized"], 21_345_000i64, "{scaled}");

    // 이 문서는 같은 문단에서 아라비아 숫자 금액과 한글 수사 금액을 나란히 쓴다
    // (`금113,560원(금일십일만삼천오백육십원)`). 한글 수사는 v1 범위 밖이므로
    // 값을 지어내지 않고 null 로 두되, raw 는 남겨 사람이 판단할 수 있게 한다.
    let korean = items
        .iter()
        .find(|m| m["raw"] == "금일십일만삼천오백육십원")
        .unwrap_or_else(|| panic!("한글 수사 금액을 뽑지 못했습니다: {v}"));
    assert!(
        korean["normalized"].is_null(),
        "한글 수사 금액은 정규화하지 않는다: {korean}"
    );
    assert_eq!(korean["currency"], "KRW", "{korean}");
}

#[test]
fn decimal_with_a_unit_multiplier_stays_exact() {
    // `12.03억원` = 1,203,000,000. 부동소수 곱셈이면 끝자리가 흔들린다 — 정수 연산이라야
    // 예산 문서의 합계가 맞는다.
    let p = sample(SAMPLE_BUDGET);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "amount",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    for (raw, expected) in [
        ("3,180백만원", 3_180_000_000i64),
        ("12.03억원", 1_203_000_000),
        ("2.15백만원", 2_150_000),
        ("57억원", 5_700_000_000),
    ] {
        let item = items
            .iter()
            .find(|m| m["raw"] == raw)
            .unwrap_or_else(|| panic!("`{raw}` 을 뽑지 못했습니다: {v}"));
        assert_eq!(
            item["normalized"].as_i64(),
            Some(expected),
            "{raw} 정규화 오류: {item}"
        );
    }
}

#[test]
fn quantities_separate_the_unit() {
    let p = sample(SAMPLE);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "number",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    assert!(!items.is_empty(), "수량이 실재하는 문서입니다: {v}");
    for item in items {
        assert_eq!(item["kind"], "number", "{item}");
        assert!(
            item["unit"].as_str().is_some_and(|u| !u.is_empty()),
            "단위 없는 수는 수량이 아닙니다: {item}"
        );
        assert!(
            item["currency"].is_null(),
            "수량에 통화가 붙었습니다: {item}"
        );
    }
    // 백분율은 단위로 분리된다.
    assert!(
        items.iter().any(|m| m["unit"] == "%"),
        "백분율(`62.9%`)이 실재하는 문서인데 단위 %가 없습니다: {v}"
    );
}

#[test]
fn kind_filter_selects_only_that_kind_and_counts_follow() {
    let p = sample(SAMPLE);
    let all = run_ok(&["extract-data", p.to_str().unwrap(), "--json"]);
    for kind in ["date", "amount", "number"] {
        let v = run_ok(&[
            "extract-data",
            p.to_str().unwrap(),
            "--kind",
            kind,
            "--json",
        ]);
        assert_eq!(v["kind"], kind, "{v}");
        let counts = v["counts"].as_object().expect("counts");
        // 요청하지 않은 종류의 키는 넣지 않는다 — `"amount": 0` 은 "금액 없음"으로 오독된다.
        assert_eq!(counts.len(), 1, "{v}");
        assert!(counts.contains_key(kind), "{v}");
        // 필터는 경계 판정을 바꾸지 않는다 — 전체 실행에서 그 종류만 센 값과 같아야 한다.
        assert_eq!(all["counts"][kind], v["counts"][kind], "{v}");
        for item in v["items"].as_array().unwrap() {
            assert_eq!(item["kind"], kind, "{item}");
        }
    }
}

#[test]
fn limit_caps_items_but_totals_stay_honest() {
    let p = sample(SAMPLE);
    let full = run_ok(&["extract-data", p.to_str().unwrap(), "--json"]);
    let total = full["totalItemCount"].as_u64().expect("totalItemCount");
    assert!(total > 3, "표본이 너무 작습니다: {total}");

    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--json",
        "--limit",
        "3",
    ]);
    assert_eq!(v["itemCount"], 3, "{v}");
    assert_eq!(v["items"].as_array().unwrap().len(), 3, "{v}");
    // 절단을 숨기면 에이전트가 "정확히 3건"과 "3건만 표시"를 구별할 수 없다.
    assert_eq!(v["totalItemCount"].as_u64(), Some(total), "{v}");
    assert_eq!(v["truncated"], true, "{v}");
    // counts 는 절단 전 총량이다.
    let sum: u64 = v["counts"]
        .as_object()
        .unwrap()
        .values()
        .filter_map(serde_json::Value::as_u64)
        .sum();
    assert_eq!(
        sum, total,
        "counts 는 --limit 절단 전 총량이어야 합니다: {v}"
    );
}

#[test]
fn items_never_overlap_within_one_text_run() {
    // 겹치는 항목은 같은 문자를 두 번 세는 것이다 — 합계·집계가 조용히 틀어진다.
    // 본문·표 셀·글상자는 각각 별개의 문자열이므로 오프셋 축도 따로 본다.
    let p = sample(SAMPLE);
    let v = run_ok(&["extract-data", p.to_str().unwrap(), "--json"]);
    let mut spans: std::collections::HashMap<String, Vec<(u64, u64)>> =
        std::collections::HashMap::new();
    for item in v["items"].as_array().unwrap() {
        let key = format!(
            "{}/{}/{}/{}",
            item["section"], item["paragraph"], item["cell"], item["textbox"]
        );
        spans.entry(key).or_default().push((
            item["charOffset"].as_u64().expect("charOffset"),
            item["length"].as_u64().expect("length"),
        ));
    }
    for (key, mut run) in spans {
        run.sort_unstable();
        for pair in run.windows(2) {
            assert!(
                pair[0].0 + pair[0].1 <= pair[1].0,
                "{key} 에서 항목이 겹칩니다: {:?} vs {:?}",
                pair[0],
                pair[1]
            );
        }
    }
}

#[test]
fn item_count_zero_or_not_the_exit_code_is_success() {
    // 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707). 값이 거의 없는 문서로도
    // 파이프라인이 멈추지 않아야 한다.
    let p = sample("samples/hwp3-sample.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["extract-data", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(
        v["itemCount"].as_u64().unwrap(),
        v["items"].as_array().unwrap().len() as u64,
        "{v}"
    );
}

#[test]
fn default_output_is_human_summary_not_json() {
    let p = sample(SAMPLE_FORM);
    let args = ["extract-data", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "기본 출력은 JSON 이 아니어야 합니다(--json 전용).\n{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_file_exit_runtime_and_silent_stdout() {
    let args = ["extract-data", "없는파일-extract-data.hwp", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_path_exit_usage_and_silent_stdout() {
    let args = ["extract-data", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn invalid_kind_exit_usage_and_silent_stdout() {
    let p = sample(SAMPLE_FORM);
    let args = [
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "치즈",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// 실물 편람은 같은 문단에서 `금113,560원(금일십일만삼천오백육십원)` 처럼 아라비아
/// 숫자와 한글 수사를 나란히 쓴다. 수사 쪽은 값을 지어내지 않고 `null` 로 남는지,
/// 그리고 그 `null` 이 문서 안에서 실제로 관측되는지 확인한다.
#[test]
fn unnormalizable_values_survive_as_raw_with_null() {
    let p = sample(SAMPLE);
    let v = run_ok(&["extract-data", p.to_str().unwrap(), "--json"]);
    let nulls: Vec<&serde_json::Value> = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["normalized"].is_null())
        .collect();
    assert!(
        !nulls.is_empty(),
        "정규화 불가 표기가 실재하는 문서인데 null 항목이 없습니다 — 값을 \
         지어내고 있을 수 있습니다: {v}"
    );
    for item in nulls {
        assert!(
            !item["raw"].as_str().unwrap_or("").is_empty(),
            "normalized 가 null 이면 raw 만이 유일한 근거다: {item}"
        );
        assert!(
            item["page"].as_u64().is_some() || item["section"].as_u64().is_some(),
            "주소 없는 null 항목은 확인할 방법이 없다: {item}"
        );
    }
    // 두 자리 연도(`’26. 1.`)는 세기를 추정하지 않는다.
    let two_digit_year = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "date" && m["normalized"].is_null());
    if let Some(item) = two_digit_year {
        let raw = item["raw"].as_str().unwrap_or("");
        assert!(
            raw.starts_with('\'') || raw.starts_with('\u{2018}') || raw.starts_with('\u{2019}'),
            "정규화 불가 날짜는 두 자리 연도뿐이어야 합니다: {item}"
        );
    }
}

#[test]
fn declared_flags_are_accepted_by_the_cli() {
    // 드리프트 가드: capabilities 가 광고하는 플래그는 실제로 받아야 한다. 선언만 있고
    // 받지 않으면 매니페스트로 도구를 자동 생성한 에이전트가 영영 못 쓰는 기능이 된다.
    let cap = run_ok(&["capabilities"]);
    let entry = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "extract-data")
        .unwrap_or_else(|| panic!("capabilities 에 extract-data 누락: {cap}"));
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(flags.contains(&"--json") && flags.contains(&"--kind") && flags.contains(&"--limit"));

    let p = sample(SAMPLE_FORM);
    let path = p.to_str().unwrap();
    for args in [
        vec!["extract-data", path, "--json"],
        vec!["extract-data", path, "--json", "--kind", "amount"],
        vec!["extract-data", path, "--json", "--limit", "1"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "선언된 플래그를 CLI 가 거부했습니다.\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn mcp_tool_declares_and_wires_every_input() {
    // 드리프트 가드: 스키마에 쓴 인자는 반드시 자식 CLI 에 닿아야 한다. 닿지 않으면
    // 서버는 그 인자를 조용히 버리고 성공을 보고한다.
    let mcp = run_ok(&["capabilities", "--mcp"]);
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_extract_data")
        .unwrap_or_else(|| panic!("hwp_extract_data 도구 누락: {mcp}"));

    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert!(tool["inputSchema"]["properties"].is_object(), "{tool}");
    let required = tool["inputSchema"]["required"]
        .as_array()
        .unwrap_or_else(|| panic!("required 배열 누락: {tool}"));
    assert!(required.iter().any(|r| r == "path"), "{tool}");
    assert_eq!(tool["cli"]["command"], "extract-data", "{tool}");

    let wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .chain(
            tool["cli"]["optionalArgs"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|o| o["when"].as_str().map(str::to_string)),
        )
        .collect();
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .unwrap()
        .keys()
    {
        // password 는 argv 가 아니라 stdin 축이다(cli.passwordStdin 계약).
        if key == "password" {
            continue;
        }
        assert!(
            wired.contains(key),
            "inputSchema 에만 있고 배선되지 않은 인자: {key} — {tool}"
        );
    }
}
