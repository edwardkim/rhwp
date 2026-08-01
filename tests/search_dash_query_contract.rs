//! [#3608] `-` 로 시작하는 검색어를 **표현할 방법**이 있어야 한다.
//!
//! `search <파일> <검색어>` 의 검색어는 위치 인자다. 파서가 `-` 로 시작하는 토큰을
//! 무조건 옵션으로 보면 `search doc.hwp "-회계"` 는 `알 수 없는 옵션: -회계` 로
//! exit 2 가 된다. 그런데 #2707 종료 코드 계약에서 exit 2 는 **"호출 조립 버그 —
//! 재시도하지 말고 인자를 고쳐라"** 다. 고칠 방법이 없는데 고치라고 하는 셈이다.
//!
//! 표면 간 비동형이기도 하다: 같은 검색어가 `batch search --query` 와 세션
//! `hwp_doc_search` 에서는 **통과한다**(둘 다 위치 인자가 아니다). 유독 `search`
//! 와 그것을 쓰는 MCP `hwp_search` 만 막힌다.
//!
//! POSIX 관례대로 `--` 를 옵션 종료 표시로 받는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn dash_leading_query_is_expressible_after_a_double_dash() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = p.to_string_lossy().to_string();

    // 구분자 없이는 여전히 사용법 오류다(옵션 오타를 삼키면 안 되므로 의도된 동작).
    // 다만 빠져나갈 길을 알려줘야 한다.
    let (code, _, err) = run(&["search", &path, "-회계", "--json"]);
    assert_eq!(
        code, 2,
        "옵션처럼 생긴 토큰은 여전히 exit 2 여야 한다: {err}"
    );
    assert!(
        err.contains("--"),
        "빠져나갈 방법(`--`)을 안내하지 않으면 에이전트는 고칠 수 없는 exit 2 앞에서 \
         멈춘다: {err}"
    );

    // `--` 뒤의 검색어는 그대로 엔진에 닿는다.
    let (code, out, err) = run(&["search", &path, "--json", "--", "-회계"]);
    assert_eq!(code, 0, "`--` 뒤 검색어가 거부됐다: {err}");
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("search --json 봉투");
    assert_eq!(v["query"], "-회계", "검색어가 그대로 전달되지 않았다: {v}");

    // 플래그와 **정확히 같은 철자**인 검색어도 검색어로 읽혀야 한다.
    let (code, out, _) = run(&["search", &path, "--json", "--", "-i"]);
    assert_eq!(code, 0, "`-i` 를 검색어로 받지 못했다");
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("봉투");
    assert_eq!(v["query"], "-i", "`-i` 가 플래그로 소비됐다: {v}");
}

#[test]
fn double_dash_does_not_break_normal_search() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = p.to_string_lossy().to_string();

    let (_, plain, _) = run(&["search", &path, "의", "--json"]);
    let plain: serde_json::Value = serde_json::from_str(plain.trim()).expect("봉투");
    let (_, dashed, _) = run(&["search", &path, "--json", "--", "의"]);
    let dashed: serde_json::Value = serde_json::from_str(dashed.trim()).expect("봉투");

    assert!(
        plain["matchCount"].as_u64().unwrap_or(0) > 0,
        "표본에 매치가 없으면 이 대조가 공허하다: {plain}"
    );
    assert_eq!(
        plain["matchCount"], dashed["matchCount"],
        "`--` 유무가 결과를 바꾸면 안 된다"
    );
}

/// MCP `hwp_search` 는 이 위치 인자 경로를 그대로 쓴다 — 배선에 `--` 가 없으면
/// 에이전트가 보낸 '-' 검색어가 서버에서 사용법 오류로 되돌아온다.
#[test]
fn mcp_search_wiring_closes_option_parsing_before_the_query() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("capabilities 실행 실패");
    let m: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities --mcp JSON");
    let t = m["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .find(|t| t["name"] == "hwp_search")
        .expect("hwp_search 선언이 없습니다");
    let args: Vec<&str> = t["cli"]["args"]
        .as_array()
        .expect("cli.args")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    let sep = args
        .iter()
        .position(|a| *a == "--")
        .unwrap_or_else(|| panic!("hwp_search 배선에 `--` 가 없다: {args:?}"));
    let q = args
        .iter()
        .position(|a| *a == "{query}")
        .unwrap_or_else(|| panic!("hwp_search 배선에 {{query}} 가 없다: {args:?}"));
    assert!(
        sep < q,
        "`--` 는 {{query}} 앞에 와야 옵션 파싱이 닫힌다: {args:?}"
    );
    assert_eq!(
        q,
        args.len() - 1,
        "{{query}} 뒤에 다른 인자가 있으면 그것도 위치 인자로 먹힌다: {args:?}"
    );
}
