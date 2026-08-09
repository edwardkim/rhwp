//! [#4220 T1] MCP 스펙 개정 추종 대장 계약 — 대장이 낡으면 red.
//!
//! `mydocs/tech/agent_architecture/mcp_spec_ledger.md` §5 기계 대사 구역이 선언한
//! 값(구현 개정판·미지 리소스 오류 코드·프로토콜 메서드 전수)을 실물 세 곳과
//! 대사한다: 살아있는 서버의 응답, `src/mcp_serve.rs` 소스, 그 값을 고정한 계약
//! 테스트 파일들. 어느 한 곳만 바뀌고 대장이 안 따라오면 여기서 걸린다 —
//! 스펙 개정을 추종하는 날, 이 테스트가 "바꿀 지점 전부를 바꿨는가"의 체크리스트가
//! 된다. 문서가 코드를 고정하는 방향이 아니라, **서로가 서로를 고정**하는 방향이다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const LEDGER: &str = "mydocs/tech/agent_architecture/mcp_spec_ledger.md";
const SERVE_SRC: &str = "src/mcp_serve.rs";
/// 자기 자신은 -32002 문자열 전수 조사에서 제외한다 — 대장 값을 파싱해 쓰는
/// 이 파일이 조사 대상이 되면 대장 갱신만으로 집합이 어긋난다.
const SELF_FILE: &str = "mcp_spec_ledger_contract.rs";

fn root(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root(rel)).unwrap_or_else(|e| panic!("{rel} 읽기 실패: {e}"))
}

/// §5 기계 대사 구역(`MACHINE-LEDGER-BEGIN` ~ `-END`)의 `key: value` 행을 파싱한다.
fn ledger() -> BTreeMap<String, String> {
    let text = read(LEDGER);
    let begin = text
        .find("MACHINE-LEDGER-BEGIN")
        .unwrap_or_else(|| panic!("{LEDGER} 에 MACHINE-LEDGER-BEGIN 마커가 없습니다"));
    let end = text
        .find("MACHINE-LEDGER-END")
        .unwrap_or_else(|| panic!("{LEDGER} 에 MACHINE-LEDGER-END 마커가 없습니다"));
    assert!(
        begin < end,
        "{LEDGER} 의 기계 대사 마커 순서가 뒤집혔습니다"
    );
    let mut map = BTreeMap::new();
    for line in text[begin..end].lines() {
        let line = line.trim();
        if line.starts_with("```") || line.starts_with("<!--") || line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    map
}

fn ledger_value(map: &BTreeMap<String, String>, key: &str) -> String {
    map.get(key)
        .unwrap_or_else(|| panic!("{LEDGER} 기계 대사 구역에 `{key}` 항목이 없습니다"))
        .clone()
}

/// 살아있는 mcp-serve 프로세스와 그 stdio 파이프.
struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn start() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("rhwp mcp-serve 실행 실패");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        }
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.stdin, "{msg}").expect("요청 쓰기 실패");
        self.stdin.flush().expect("flush");
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line).expect("응답 읽기 실패");
            assert!(n > 0, "서버가 응답 없이 종료했습니다 (method={method})");
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line.trim())
                .unwrap_or_else(|e| panic!("stdout 이 순수 JSON-RPC 가 아닙니다 ({e}): {line}"));
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }

    fn initialize(&mut self, protocol_version: &str) -> serde_json::Value {
        self.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": protocol_version,
                "capabilities": {},
                "clientInfo": {"name": "spec-ledger-test", "version": "0"}
            }),
        )
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// ① 서버가 광고하는 protocolVersion == 대장의 "구현 개정판".
///
/// 대장 개정판으로 initialize 하면 같은 값이 돌아오고, 존재하지 않는 개정판을
/// 요구해도 서버의 역제안이 대장 값이어야 한다 — 지원 목록이 대장 밖의 개정판을
/// 몰래 품는 것까지 잡는다.
#[test]
fn server_advertises_ledger_revision() {
    let rev = ledger_value(&ledger(), "implemented-revision");

    let r = Server::start().initialize(&rev);
    assert_eq!(
        r["result"]["protocolVersion"].as_str(),
        Some(rev.as_str()),
        "대장의 구현 개정판({rev})으로 initialize 했는데 다른 값이 왔습니다: {r}"
    );

    let r = Server::start().initialize("1900-01-01");
    assert_eq!(
        r["result"]["protocolVersion"].as_str(),
        Some(rev.as_str()),
        "미지 개정판에 대한 서버 역제안이 대장의 구현 개정판({rev})이 아닙니다: {r}"
    );
}

/// ① 보강: 개정판 문자열을 고정한 소스·계약 테스트 상수가 대장과 한 값이다.
///
/// 스펙 개정을 추종하는 날 바꿔야 하는 세 지점(src 상수·계약 테스트 상수·대장)이
/// 따로 놀면 여기서 걸린다.
#[test]
fn ledger_revision_matches_pinned_constants() {
    let rev = ledger_value(&ledger(), "implemented-revision");

    let serve = read(SERVE_SRC);
    let expected = format!("const PROTOCOL_VERSION: &str = \"{rev}\";");
    assert!(
        serve.contains(&expected),
        "{SERVE_SRC} 의 PROTOCOL_VERSION 이 대장({rev})과 다릅니다 — \
         개정을 추종했다면 {LEDGER} §1·§5 를 함께 갱신하세요"
    );

    let contract = read("tests/mcp_server_contract.rs");
    let expected = format!("const SERVER_PROTOCOL_VERSION: &str = \"{rev}\";");
    assert!(
        contract.contains(&expected),
        "tests/mcp_server_contract.rs 의 SERVER_PROTOCOL_VERSION 이 대장({rev})과 다릅니다"
    );
}

/// ② 대장이 열거한 미지 리소스 오류 코드 표면이 실물과 일치한다.
///
/// 소스 상수, 살아있는 서버의 응답, 그리고 그 코드를 assertion 으로 싣는 계약
/// 테스트 파일 **집합**까지 대사한다 — 2026-07-28 개정이 이 코드를 -32602 로
/// 바꿨으므로, 추종하는 날 이 세 곳과 대장이 한 번에 움직여야 한다.
#[test]
fn ledger_resource_not_found_surface_matches_reality() {
    let map = ledger();
    let code_str = ledger_value(&map, "resource-not-found-code");
    let code: i64 = code_str
        .parse()
        .unwrap_or_else(|e| panic!("resource-not-found-code 가 정수가 아닙니다({e}): {code_str}"));

    // 소스 상수.
    let serve = read(SERVE_SRC);
    let expected = format!("const RESOURCE_NOT_FOUND: i64 = {code};");
    assert!(
        serve.contains(&expected),
        "{SERVE_SRC} 의 RESOURCE_NOT_FOUND 가 대장({code})과 다릅니다"
    );

    // 살아있는 서버.
    let mut s = Server::start();
    let rev = ledger_value(&map, "implemented-revision");
    s.initialize(&rev);
    let r = s.request(
        "resources/read",
        serde_json::json!({"uri": "rhwp://docs/스펙대장-없는-리소스"}),
    );
    assert_eq!(
        r["error"]["code"].as_i64(),
        Some(code),
        "미지 리소스 오류 코드가 대장({code})과 다릅니다: {r}"
    );

    // 계약 테스트 파일 집합 — 대장에 적힌 파일은 실제로 그 코드를 싣고,
    // 대장에 없는 파일이 그 코드를 새로 실으면 대장 갱신을 요구한다.
    let declared: BTreeSet<String> = ledger_value(&map, "resource-not-found-test-files")
        .split(',')
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect();
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(root("tests")).expect("tests/ 열람 실패") {
        let path = entry.expect("tests/ 항목").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") || name == SELF_FILE {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("tests/{name} 읽기 실패: {e}"));
        if text.contains(&code_str) {
            actual.insert(format!("tests/{name}"));
        }
    }
    assert_eq!(
        actual, declared,
        "오류 코드 {code} 를 싣는 계약 테스트 파일 집합이 대장과 다릅니다 — \
         {LEDGER} §5 의 resource-not-found-test-files 를 실물에 맞추세요"
    );
}

/// ② 보강: 대장의 프로토콜 메서드 전수가 디스패치의 match 팔과 집합으로 일치한다.
///
/// 메서드가 하나 생기거나 사라지는 것(예: 2026-07-28 의 `server/discover` 신설,
/// `ping`·`initialize` 제거)은 곧 표면 목록의 변화다 — 대장이 따라와야 한다.
#[test]
fn ledger_method_list_matches_dispatch_arms() {
    let declared: BTreeSet<String> = ledger_value(&ledger(), "serve-methods")
        .split(',')
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();

    // match 팔에서 문자열 리터럴을 걷는다. 프로토콜 메서드만 남기는 필터:
    // `/` 를 품거나(`tools/list` 류) 핸드셰이크·유틸 단독 메서드다. 도구 이름
    // (`hwp_*`)과 CLI 플래그(`--profile`)는 `/` 가 없어 자연히 걸러진다.
    let mut actual = BTreeSet::new();
    for line in read(SERVE_SRC).lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some(quote) = rest.find('"') else {
            continue;
        };
        let (literal, after) = rest.split_at(quote);
        if !after[1..].trim_start().starts_with("=>") {
            continue;
        }
        if literal == "initialize" || literal == "ping" || literal.contains('/') {
            actual.insert(literal.to_string());
        }
    }
    assert_eq!(
        actual, declared,
        "디스패치가 받는 프로토콜 메서드 전수가 대장과 다릅니다 — \
         {LEDGER} §1·§5 의 serve-methods 를 실물에 맞추세요"
    );
}
