//! [#3787 S1] 봉투 출처 표지 드리프트 가드.
//!
//! 계약: 모든 `--json` 봉투는 `untrustedContent`(bool)와 `untrustedFields`(경로 배열)를
//! 싣고, 그 값은 `rhwp export-provenance-map --json` 지도와 일치한다.
//!
//! ## 이 파일이 지키는 것
//!
//! 출처 표지는 **선언**이다. 선언은 코드가 바뀌어도 조용히 그대로 남는다 — 새 명령이
//! 문서 텍스트를 실어 나르기 시작해도, 기존 필드에 문서 문자열이 하나 더 붙어도,
//! 지도는 아무 말 없이 옛 사실을 계속 광고한다. 6개월 뒤 "이 봉투는 안전하다"는
//! 표지가 거짓이 되는 경로가 그것이다.
//!
//! 그래서 여기서는 **선언을 믿지 않는다.** 실제 문서를 열어 그 문서에만 있는 문자열
//! 오라클을 만들고, 봉투 안에서 그 문자열이 나타나는 위치를 찾아 지도와 대조한다.
//! 지도에 없는 곳에서 문서 문자열이 나오면 실패다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

use serde_json::Value;

/// 본문·표·개요가 모두 있는 기본 샘플 (cli_json_contract 와 같은 문서).
const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 표 편집(`edit set-cell`·`run set_cell`)용 — 셀 텍스트가 봉투로 되돌아온다.
const TABLE_SAMPLE: &str = "samples/table-001.hwp";
/// 누름틀이 실제로 있는 문서 — `fields` 봉투를 비지 않게 한다.
const FIELD_SAMPLE: &str = "samples/field-01.hwp";
/// DocLang 내보내기가 지원하는 HWP5 문서 (HWP3 은 미지원).
const DOCLANG_SAMPLE: &str = "samples/para-001.hwp";
/// `export-hml` 은 HML 원본만 받는다.
const HML_SAMPLE: &str = "samples/hml/formatting_table.hml";
/// PrvImage 썸네일이 내장된 문서.
const THUMBNAIL_SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";

// ── 실행 도우미 ────────────────────────────────────────────────────────────

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tmp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp-provenance-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn run(args: &[String]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn run_with_stdin(args: &[String], body: &str) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    write_stdin_ignoring_early_exit(&mut child, body);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

/// stdin 에 본문을 쓰되, 자식이 stdin 을 읽기 전에 종료한 경우의 BrokenPipe 는
/// 무시한다. 인자 검증 거부 계열 테스트는 프로세스가 입력을 소비하기 전에
/// 종료하는 것이 정상 경로라, 쓰기 완료 여부는 검증 대상(종료 코드·출력)이
/// 아니다 (#3763 — batch_axes_contract.rs 와 같은 처리).
fn write_stdin_ignoring_early_exit(child: &mut std::process::Child, body: &str) {
    use std::io::ErrorKind;
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
}

fn describe(args: &[String], out: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn json_of(args: &[&str]) -> Value {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let out = run(&owned);
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 JSON 이 아닙니다 ({e}).\n{}",
            describe(&owned, &out)
        )
    })
}

fn capabilities() -> Value {
    json_of(&["capabilities"])
}

fn provenance_map() -> Value {
    json_of(&["export-provenance-map", "--json"])
}

/// capabilities 가 `--json` 계약을 선언한 명령 이름들.
fn json_commands(cap: &Value) -> Vec<String> {
    cap["commands"]
        .as_array()
        .expect("commands 배열")
        .iter()
        .filter(|c| c["json"] == true)
        .filter_map(|c| c["name"].as_str().map(String::from))
        .collect()
}

/// 지도가 선언한 경로들.
fn declared_paths(map: &Value, command: &str) -> Vec<String> {
    map["commands"][command]["untrusted"]
        .as_array()
        .unwrap_or_else(|| panic!("지도에 {command} 항목이 없습니다: {map}"))
        .iter()
        .filter_map(|p| p.as_str().map(String::from))
        .collect()
}

/// 경로의 최상위 키 — `matches[].context` → `matches`.
fn root_of(path: &str) -> &str {
    let end = path.find(['.', '[']).unwrap_or(path.len());
    &path[..end]
}

// ── 문서 문자열 오라클 ─────────────────────────────────────────────────────

/// "이 문자열이 봉투에 보이면 그 값은 문서에서 왔다" 는 판정 근거.
///
/// 지도(선언)를 참고하지 않고 **문서 자체**에서 만든다. 그래야 지도가 틀렸을 때
/// 가드가 지도 편을 들지 않는다.
struct DocOracle {
    /// 부분 문자열로 찾는 긴 토큰(6자 이상). 짧은 토큰은 엔진 라벨·고정 문구와
    /// 충돌할 수 있어 부분 일치 축에는 쓰지 않는다.
    tokens: Vec<String>,
    /// **통째로 같으면** 문서 파생인 짧은 문자열.
    ///
    /// 두 원천을 합친다.
    /// - 표 셀·캡션 전체 텍스트 — `edit set-cell` 의 `oldText`("구 분")를 잡는다.
    /// - 본문의 **한글이 든 2자 이상 낱말** — `fields[].name`("회사명")처럼 짧은
    ///   문서 값을 잡는다. 한글을 요구하는 이유는 이 저장소의 봉투 열거값이
    ///   ASCII(`hwp5`·`clean`·`page`…)이거나 공백이 든 한국어 문장이라, 한글
    ///   낱말 하나와 통째로 같아질 일이 없기 때문이다.
    exact: BTreeSet<String>,
}

impl DocOracle {
    fn hits(&self, s: &str) -> bool {
        if self.exact.contains(s.trim()) {
            return true;
        }
        self.tokens.iter().any(|t| s.contains(t.as_str()))
    }
}

fn collect_cell_text(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(o) => {
            for key in ["text", "caption"] {
                if let Some(s) = o.get(key).and_then(|t| t.as_str()) {
                    let t = s.trim();
                    if t.chars().count() >= 2 && t.chars().any(|c| c.is_alphanumeric()) {
                        out.insert(t.to_string());
                    }
                }
            }
            for val in o.values() {
                collect_cell_text(val, out);
            }
        }
        Value::Array(a) => {
            for e in a {
                collect_cell_text(e, out);
            }
        }
        _ => {}
    }
}

fn oracle(doc: &Path) -> DocOracle {
    let path = doc.to_str().expect("경로");
    let text_env = json_of(&["export-text", "--json", path]);
    let mut tokens: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut exact = BTreeSet::new();
    if let Some(pages) = text_env["pages"].as_array() {
        for page in pages {
            let Some(text) = page["text"].as_str() else {
                continue;
            };
            for raw in text.split_whitespace() {
                let t: String = raw.chars().filter(|c| c.is_alphanumeric()).collect();
                if t.chars().count() >= 6 && seen.insert(t.clone()) {
                    tokens.push(t);
                }
                let short = raw.trim();
                if short.chars().count() >= 2 && short.chars().any(|c| ('가'..='힣').contains(&c))
                {
                    exact.insert(short.to_string());
                }
            }
        }
    }
    tokens.truncate(600);

    let tables = run(&[
        "export-tables".into(),
        path.to_string(),
        "--json".to_string(),
    ]);
    if tables.status.success() {
        if let Ok(v) = serde_json::from_slice::<Value>(&tables.stdout) {
            collect_cell_text(&v["tables"], &mut exact);
        }
    }
    DocOracle { tokens, exact }
}

/// 호출자가 준 값이 그대로 되돌아오는 필드 — 문서 파생이 아니다.
///
/// 오라클은 문자열만 보므로, 입력 경로에 문서 본문과 같은 낱말이 들어 있으면
/// (예: `국립국어원` 이 파일명에도 본문에도 있다) 경로 반향을 문서 파생으로
/// 오판한다. 사유 없는 예외는 가드를 좀먹으므로 항목마다 근거를 단다.
const CALLER_ECHO: &[(&str, &str)] = &[
    ("source", "호출자가 준 입력 경로의 반향"),
    ("input", "run 계획서가 지정한 입력 경로"),
    ("output", "호출자가 지정한 산출 경로"),
    ("outputDir", "호출자가 지정한 산출 폴더"),
    ("assetsDir", "호출자가 지정한 자산 폴더"),
    // 전체 경로 항목 — 잎(leaf)으로 등재하면 같은 잎을 가진 문서 파생 경로
    // (예: fields[].name — 문서에서 읽은 누름틀 이름)까지 면제돼 가드가 약해진다.
    (
        "filled[].name",
        "fill-fields --data 의 키 반향 — 채움이 성공한 이름은 문서 누름틀 이름과 \
         같아질 수밖에 없지만, 봉투에 실리는 문자열 자체는 호출자가 준 것이다 \
         (src/provenance.rs edit 항목 note 의 기존 판정과 동일)",
    ),
    (
        "path",
        "매니페스트의 산출 파일 경로 — 입력 파일이름에서 조합된다",
    ),
    ("a", "ir-diff 비교 대상 A 경로"),
    ("b", "ir-diff 비교 대상 B 경로"),
    ("query", "search 검색어 — 호출자가 준 값"),
    ("find", "edit/run 의 찾을 문자열"),
    ("replace", "edit/run 의 바꿀 문자열"),
    ("newText", "set-cell 이 새로 넣는 값"),
];

fn is_caller_echo(path: &str) -> bool {
    // 전체 경로 일치 우선 — filled[].name 처럼 특정 자리만 반향인 경우를 잎 등재로
    // 넓히지 않기 위해서다.
    if CALLER_ECHO.iter().any(|(k, _)| *k == path) {
        return true;
    }
    let leaf = path.rsplit('.').next().unwrap_or(path);
    let leaf = leaf.trim_end_matches("[]");
    CALLER_ECHO.iter().any(|(k, _)| *k == leaf)
}

/// 봉투를 훑어 **문서 문자열이 실제로 나타난** 경로들을 모은다.
/// 경로 표기는 지도와 같다: `.` 은 객체 하위, `[]` 는 배열 전개.
fn scan(v: &Value, path: &str, or: &DocOracle, out: &mut BTreeSet<String>) {
    match v {
        Value::String(s) => {
            if !is_caller_echo(path) && or.hits(s) {
                out.insert(path.to_string());
            }
        }
        Value::Array(a) => {
            let p = format!("{path}[]");
            for e in a {
                scan(e, &p, or, out);
            }
        }
        Value::Object(o) => {
            for (k, val) in o {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                scan(val, &p, or, out);
            }
        }
        _ => {}
    }
}

// ── 호출 레시피 ────────────────────────────────────────────────────────────

/// 한 명령을 실제로 실행해 봉투를 얻는 방법.
struct Recipe {
    command: &'static str,
    /// 이 호출이 여는 문서 — 오라클의 원천. `None` 이면 문서를 열지 않는 명령이다.
    doc: Option<PathBuf>,
    args: Vec<String>,
    stdin: Option<String>,
    /// 성공으로 볼 종료 코드.
    exit: i32,
    /// stdout 이 NDJSON(줄당 봉투 하나)인가.
    ndjson: bool,
}

/// 레시피를 만들 수 없어 스윕에서 빼는 명령과 **그 사유**.
///
/// 여기 넣어도 되는 것은 "문서를 입력으로 받지 않아 문서 오라클을 만들 수 없는"
/// 명령뿐이다. 사유 없는 허용목록은 가드를 무력화하므로 사유를 강제한다.
const SWEEP_EXEMPT: &[(&str, &str)] = &[
    (
        "build-from-ingest",
        "입력이 문서가 아니라 호출자가 만든 ingest JSON 이라 '문서에서 온 문자열' 오라클을 \
         만들 수 없다. 봉투는 경로·바이트·문항/문단 개수뿐임을 지도가 선언하고, \
         tests/issue_3358_ingest_unknown_fields.rs 가 그 봉투를 따로 고정한다.",
    ),
    (
        "export-ir-schema",
        "문서를 입력으로 받지 않는 IR 타입 스키마다. --bare가 아닌 모드도 특정 문서가 아닌 \
         스키마 봉투를 낸다.",
    ),
    (
        "export-capabilities-schema",
        "문서를 입력으로 받지 않는 capabilities 타입 스키마다. --bare가 아닌 모드도 특정 \
         문서가 아닌 스키마 봉투를 낸다.",
    ),
    (
        "export-agent-manifest",
        "문서를 입력으로 받지 않는다 — 인자가 --json 과 --bare 뿐이고, 내는 것은 \
         capabilities·irSchema·provenanceMap·planSchema 를 조립한 rhwp 자신의 \
         매니페스트다. 구성 요소는 이미 각자의 계약으로 고정돼 있고(capabilities 는 \
         이 파일의 다른 가드, provenanceMap 은 export-provenance-map, planSchema 는 \
         tests/plan_schema_contract.rs), 여기서 다시 볼 문서 유래 문자열이 없다.",
    ),
    (
        "export-plan-schema",
        "문서를 입력으로 받지 않는 계획서 문법 스키마다. 인자가 --bare·-o·--json 뿐이고 \
         --bare가 아닌 모드도 특정 문서가 아닌 스키마 봉투를 낸다. \
         봉투 모양은 tests/plan_schema_contract.rs 가 따로 고정한다.",
    ),
    (
        "export-ontology",
        "문서를 입력으로 받지 않는다 — 자기서술(IR 스키마·capabilities·MCP 도구·출처 \
         지도)에서 기계 유도한 JSON-LD 온톨로지다. --bare가 아닌 모드도 특정 문서가 \
         아닌 온톨로지 봉투를 낸다. 봉투 모양은 tests/ontology_contract.rs 가 따로 \
         고정한다.",
    ),
];

fn s(v: &str) -> String {
    v.to_string()
}

fn recipes() -> Vec<Recipe> {
    let dir = tmp_dir();
    let main = sample(SAMPLE);
    let table = sample(TABLE_SAMPLE);
    let field = sample(FIELD_SAMPLE);
    let doclang = sample(DOCLANG_SAMPLE);
    let hml = sample(HML_SAMPLE);
    let thumb = sample(THUMBNAIL_SAMPLE);

    let p = |x: &Path| x.to_str().expect("경로").to_string();
    let out = |name: &str| p(&dir.join(name));

    // run 계획서 — set_cell 저널이 셀의 옛 텍스트(문서 값)를 되돌려 준다.
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p(&table),
        "output": out("run-plan.hwp"),
        "steps": [ { "action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "ZZ" } ],
    })
    .to_string();

    // [#4509] 서명 검증 픽스처 — 순수 fs 로 기형(malformed) 경로를 고정한다.
    // 암호학적 전 경로(valid·invalid·unknownKey·revoked)는 signing_contract 가
    // 실키로 덮고, 스윕의 관심은 봉투 표지뿐이라 기형 서명 + 빈 keyring 이면
    // 족하다 (exit 3 = 판정 데이터).
    let sig_capsule = dir.join("prov-sign.capsule.json");
    std::fs::write(&sig_capsule, br#"{"kind":"workCapsule"}"#).expect("서명 캡슐 픽스처");
    let sig_sidecar = dir.join("prov-sign.capsule.json.sig.json");
    std::fs::write(&sig_sidecar, br#"{"kind":"notASignature"}"#).expect("기형 서명 픽스처");
    let sig_keyring = dir.join("prov-keyring.json");
    std::fs::write(
        &sig_keyring,
        br#"{"schemaVersion":"1.0","kind":"keyring","keys":[]}"#,
    )
    .expect("빈 keyring 픽스처");
    // bundle 픽스처 — 뿌리 캡슐 1개(부모 없음)와 빈 keyring 도메인.
    let bundle_capsule = dir.join("prov-root.capsule.json");
    std::fs::write(
        &bundle_capsule,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "workCapsule", "parent": null,
            "plan": { "planVersion": "1.0", "input": "x", "output": "y", "steps": [] },
            "planText": "{}",
            "receipt": { "inputSha256": "00", "outputSha256": "00" },
        })
        .to_string(),
    )
    .expect("번들 캡슐 픽스처");
    let bundle_out = dir.join("prov.lineage-bundle");
    let bundle_domain = dir.join("prov-domain.json");
    std::fs::write(
        &bundle_domain,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "trustDomain", "domain": "prov",
            "keyring": { "schemaVersion": "1.0", "kind": "keyring", "keys": [] },
            "checkpoints": [],
        })
        .to_string(),
    )
    .expect("도메인 픽스처");
    // [#4551] disclose 픽스처 — 진짜 replay 캡슐이어야 restore 바이트 복원이 성립.
    let disclose_plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p(&table),
        "output": out("prov-disclose.out.hwp"),
        "steps": [ { "action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "DP" } ],
    })
    .to_string();
    let disclose_capsule = dir.join("prov-disclose.capsule.json");
    let disclose_redacted = dir.join("prov-disclose.redacted.json");
    let disclose_opening = dir.join("prov-disclose.opening.json");
    let disclose_restored = dir.join("prov-disclose.restored.json");

    // [#4553] settle 픽스처 — 명세서·allow 게이트 봉투 (캡슐은 disclose 픽스처 재사용).
    let settle_wo = dir.join("prov-settle.wo.json");
    std::fs::write(
        &settle_wo,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "workorder", "workorderId": "prov-wo-1",
            "acceptancePolicy": { "schemaVersion": "1.0", "kind": "admissionPolicy",
                                   "default": "deny", "rules": [] },
            "unitPrice": { "amount": "1", "currency": "KRW", "per": "capsule" },
        })
        .to_string(),
    )
    .expect("명세서 픽스처");
    let settle_gate = dir.join("prov-settle.gate.json");
    std::fs::write(
        &settle_gate,
        serde_json::json!({ "schemaVersion": "1.0", "verdict": "allow", "violations": [] })
            .to_string(),
    )
    .expect("게이트 봉투 픽스처");
    let settle_claim = dir.join("prov-settle.claim.json");
    let settle_ledger = dir.join("prov-settle.ledger.ndjson");

    // [#4558] y10 픽스처 — 감사 보고 산출 경로 (대상 폴더는 wrap 작업장 재사용).
    let y10_report = dir.join("prov-y10.report.json");
    // gate 픽스처 — 빈 규칙 + deny 기본 = 거부(exit 3) 순수 fs 경로.
    let gate_policy = dir.join("prov-gate-policy.json");
    std::fs::write(
        &gate_policy,
        br#"{"kind":"admissionPolicy","name":"prov","defaultVerdict":"deny","rules":[]}"#,
    )
    .expect("게이트 정책 픽스처");

    // anchor verify 픽스처 — 빈 로그 + 아무 캡슐 = 미등재(logged:false, exit 3).
    let anchor_log = dir.join("prov-anchor.ndjson");
    std::fs::write(&anchor_log, b"").expect("빈 앵커 로그");
    // entries·merkleRoot·upToSeq 는 checkpoint 봉투에만 있고, checkpoint 는 빈
    // 로그를 거부한다(exit 2). 그래서 전용 로그를 비운 뒤 add→checkpoint 를
    // 이 순서로 태운다 — 스윕은 선언 순서대로 순차 실행하므로 항목 수가 1로
    // 고정된다(허용목록 대신 레시피가 필드를 실제로 내게 하는 쪽).
    let anchor_seq_log = dir.join("prov-anchor-seq.ndjson");
    std::fs::write(&anchor_seq_log, b"").expect("연번 앵커 로그");

    // harness-status 픽스처 — 깨진 캡슐 폴더 규약(capsules/ 하위).
    let harness_dir = dir.join("prov-harness");
    std::fs::create_dir_all(harness_dir.join("capsules")).expect("하네스 작업장");
    std::fs::write(
        harness_dir
            .join("capsules")
            .join("0001_broken.capsule.json"),
        br#"{"kind":"notACapsule"}"#,
    )
    .expect("깨진 하네스 캡슐");
    // harness wrap 픽스처 — capsule·output·parent 는 실산출 경로에서만 나오므로
    // 허용목록 대신 레시피가 그 필드를 실제로 내게 한다(빈 작업장 = 첫 캡슐).
    let harness_wrap_dir = dir.join("prov-harness-wrap");
    std::fs::create_dir_all(harness_wrap_dir.join("capsules")).expect("wrap 작업장");
    let harness_plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p(&table),
        "output": p(&harness_wrap_dir.join("wrapped.hwp")),
        "steps": [ { "action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "ZZ" } ],
    })
    .to_string();

    let keygen_out = dir.join("prov-keygen.key.json");
    let _ = std::fs::remove_file(&keygen_out);

    // audit 캡슐 폴더 — 고의로 깨진 캡슐 1개. 실패 회계 봉투(failed[])가 문서
    // 문자열 없이 캡슐 이름·사유만 실음을 실측으로 고정한다 (실패 존재 → exit 3).
    let audit_dir = dir.join("prov-audit-capsules");
    std::fs::create_dir_all(&audit_dir).expect("audit 캡슐 폴더");
    std::fs::write(
        audit_dir.join("broken.capsule.json"),
        r#"{"kind":"notACapsule"}"#,
    )
    .expect("깨진 캡슐 픽스처");

    // search 질의어는 문서에서 뽑는다 — 매치가 0건이면 봉투가 비어 가드가 공허해진다.
    let main_oracle = oracle(&main);
    let query = main_oracle
        .tokens
        .first()
        .cloned()
        .expect("샘플에서 토큰을 얻지 못했습니다");

    // csv-to-table 은 호출자 CSV와 문서 표를 함께 받는다. 문서에서 뽑은 같은 CSV를
    // 다시 넣으면 변경 전 셀 텍스트가 봉투에 없을 수는 있지만, JSON 표지가 항상
    // 붙는지와 지도 경로가 이 명령을 덮는지는 실제 호출로 검증할 수 있다.
    let table_csv_path = PathBuf::from(out("provenance-table.csv"));
    let table_seed_args = vec![
        s("table-to-csv"),
        p(&table),
        s("--table"),
        s("0"),
        s("--json"),
    ];
    let table_seed = run(&table_seed_args);
    assert_eq!(
        table_seed.status.code(),
        Some(0),
        "CSV 기준선을 만들지 못했습니다:\n{}",
        describe(&table_seed_args, &table_seed)
    );
    let table_seed_json: Value =
        serde_json::from_slice(&table_seed.stdout).expect("table-to-csv 기준선 stdout JSON");
    let table_csv = table_seed_json["tables"][0]["csv"]
        .as_str()
        .expect("table-to-csv 기준선 CSV")
        .to_string();
    std::fs::write(&table_csv_path, table_csv).expect("CSV 기준선 쓰기");

    // [#3885] redact 스윕용 가짜 개인정보 문서 — 저장소에 PII 샘플을 두지 않는다
    // (tests/redact_sanitize_contract.rs 와 같은 fill-fields 주입 방식). 값은 전부
    // 가공이다: 검증 숫자(mod 11)를 통과하는 실재 인물 무관 주민번호, 하이픈 형태
    // 전화번호. 이 값들이 본문에 들어가야 오라클이 findings[].raw 를 잡는다.
    let pii = PathBuf::from(out("prov-pii.hwp"));
    let _ = std::fs::remove_file(&pii);
    let pii_args = vec![
        s("edit"),
        s("fill-fields"),
        p(&field),
        s("--data"),
        s(r#"{"작성자":"홍길동 900101-1234568","전화번호":"010-1234-5678"}"#),
        s("-o"),
        p(&pii),
        s("--json"),
    ];
    let pii_fill = run(&pii_args);
    assert_eq!(
        pii_fill.status.code(),
        Some(0),
        "PII 픽스처를 만들지 못했습니다:\n{}",
        describe(&pii_args, &pii_fill)
    );

    // insert-image 레시피용 그림 — 아무 실물 이미지면 된다.
    let stamp = sample("samples/s1.jpg");

    // [#3880 T1] recordFields 전수 대조용 픽스처 — 선언 필드가 "그 필드를 내는
    // 호출"에서만 나오는 경우, 그 호출을 스윕에 실제로 넣는다(허용목록 대신).
    let bad_plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p(&field),
        "output": out("prov-run-bad.hwp"),
        "steps": [ { "action": "fill_fields", "data": { "prov없는필드": "x" } } ],
    })
    .to_string();
    let fill_rows_path = PathBuf::from(out("prov-fill-rows.jsonl"));
    std::fs::write(&fill_rows_path, "{\"작성자\":\"홍길동 제안\"}\n").expect("fill rows 쓰기");
    let fill_out_dir = dir.join("prov-fill-out");
    let _ = std::fs::create_dir_all(&fill_out_dir);

    vec![
        Recipe {
            command: "info",
            doc: Some(main.clone()),
            args: vec![s("info"), s("--json"), p(&main)],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-text",
            doc: Some(main.clone()),
            args: vec![s("export-text"), s("--json"), p(&main)],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-structure",
            doc: Some(main.clone()),
            args: vec![s("export-structure"), s("--json"), p(&main)],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "digest",
            doc: Some(main.clone()),
            args: vec![s("digest"), s("--json"), p(&main)],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-tables",
            doc: Some(main.clone()),
            args: vec![s("export-tables"), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "table-to-csv",
            doc: Some(table.clone()),
            args: vec![
                s("table-to-csv"),
                p(&table),
                s("--table"),
                s("0"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "csv-to-table",
            doc: Some(table.clone()),
            args: vec![
                s("csv-to-table"),
                p(&table),
                s("--csv"),
                p(&table_csv_path),
                s("--table"),
                s("0"),
                s("--dry-run"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "search",
            doc: Some(main.clone()),
            args: vec![s("search"), p(&main), query, s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "extract-data",
            doc: Some(main.clone()),
            args: vec![s("extract-data"), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // inspect 는 하위 명령군이므로, 새 유니코드 축을 실제 문서에서 실행한다.
        // 정상 문서의 빈 findings 도 출처 표지가 유지되는지 확인할 수 있다.
        Recipe {
            command: "inspect",
            doc: Some(main.clone()),
            args: vec![s("inspect"), s("unicode"), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "dump-pages",
            doc: Some(main.clone()),
            args: vec![s("dump-pages"), p(&main), s("-p"), s("0"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "fields",
            doc: Some(field.clone()),
            args: vec![s("fields"), p(&field), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "explain",
            doc: Some(field.clone()),
            args: vec![s("explain"), p(&field), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "edit",
            doc: Some(table.clone()),
            args: vec![
                s("edit"),
                s("set-cell"),
                p(&table),
                s("--table"),
                s("0"),
                s("--row"),
                s("0"),
                s("--col"),
                s("0"),
                s("--text"),
                s("ZZ"),
                s("--dry-run"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // [#3885] edit 는 하위 명령마다 봉투가 다르다 — set-cell 하나로 "edit 를
        // 덮었다"고 치면 redact 의 findings[].raw(개인정보 원문) 같은 가장 민감한
        // 경로가 스윕 밖에 남는다. 문서 파생 값을 싣는 하위 명령을 각각 돌린다.
        Recipe {
            command: "edit",
            doc: Some(pii.clone()),
            args: vec![s("edit"), s("redact"), p(&pii), s("--dry-run"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "edit",
            doc: Some(pii.clone()),
            args: vec![
                s("edit"),
                s("sanitize"),
                p(&pii),
                s("-o"),
                out("prov-sanitized.hwp"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // 문서 유래 문자열이 없는 edit 봉투(insert-image)도 표지 존재 검사는 받는다.
        Recipe {
            command: "edit",
            doc: Some(field.clone()),
            args: vec![
                s("edit"),
                s("insert-image"),
                p(&field),
                s("--image"),
                p(&stamp),
                s("--dry-run"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // ── [#3880 T1] recordFields 전수 대조 보강 — 선언 필드를 실제로 내는 호출들 ──
        Recipe {
            // digest --sections: 선언 필드 sections(절 단위 청크)는 이 플래그에서만 나온다.
            command: "digest",
            doc: Some(main.clone()),
            args: vec![s("digest"), s("--json"), s("--sections"), p(&main)],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // inspect 는 축이 셋이다 — unicode 하나로 "덮었다" 치면 hidden-text 의
            // hiddenText·hiddenCharCount, injection 의 injectionSignals 등이 사각에 남는다.
            command: "inspect",
            doc: Some(main.clone()),
            args: vec![s("inspect"), s("hidden-text"), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "inspect",
            doc: Some(field.clone()),
            args: vec![s("inspect"), s("injection"), p(&field), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // run 무효 계획 — invalid[] 는 실패 봉투에만 실린다(exit 2). 실패 봉투도
            // 표지 존재 검사를 받게 하는 부수 효과가 있다.
            command: "run",
            doc: Some(field.clone()),
            args: vec![s("run"), s("--plan-json"), bad_plan, s("--json")],
            stdin: None,
            exit: 2,
            ndjson: false,
        },
        Recipe {
            // table-to-csv -o: output·outputFormat 은 파일 산출에서만 나온다.
            command: "table-to-csv",
            doc: Some(table.clone()),
            args: vec![
                s("table-to-csv"),
                p(&table),
                s("--table"),
                s("0"),
                s("-o"),
                out("prov-t2c.csv"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // csv-to-table 실적용 + --verify: output·outputFormat·verify.
            command: "csv-to-table",
            doc: Some(table.clone()),
            args: vec![
                s("csv-to-table"),
                p(&table),
                s("--csv"),
                p(&table_csv_path),
                s("--table"),
                s("0"),
                s("-o"),
                out("prov-c2t.hwp"),
                s("--verify"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // edit fill-fields --verify: filled·filledCount·notFound·verify.
            command: "edit",
            doc: Some(field.clone()),
            args: vec![
                s("edit"),
                s("fill-fields"),
                p(&field),
                s("--data"),
                s(r#"{"회사명":"티일 주식회사"}"#),
                s("-o"),
                out("prov-fill.hwp"),
                s("--verify"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // edit replace-text: replacedCount.
            command: "edit",
            doc: Some(pii.clone()),
            args: vec![
                s("edit"),
                s("replace-text"),
                p(&pii),
                s("--find"),
                s("홍길동"),
                s("--replace"),
                s("김샘플"),
                s("-o"),
                out("prov-repl.hwp"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // batch 오류 레코드 — error·exitClass 는 파일 단위 실패 레코드에만 실린다.
            // jsonContract: "batch 는 error 레코드 + 최종 exit 1".
            command: "batch",
            doc: None,
            args: vec![s("batch"), s("info"), s("--json")],
            stdin: Some(s("prov-no-such-file-3885.hwp\n")),
            exit: 1,
            ndjson: true,
        },
        Recipe {
            // batch fill — row·output·filledCount·notFound 는 메일머지 행 봉투에만.
            command: "batch",
            doc: Some(field.clone()),
            args: vec![
                s("batch"),
                s("fill"),
                s("--form"),
                p(&field),
                s("--data"),
                p(&fill_rows_path),
                s("--out-dir"),
                p(&fill_out_dir),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: true,
        },
        Recipe {
            // replay 영수증 — 봉투가 해시·카운트뿐(문서 문자열 없음)임을 실측으로 고정.
            command: "replay",
            doc: Some(table.clone()),
            args: vec![s("replay"), s("--plan-json"), plan.clone(), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // audit 실패 회계 — 캡슐(호출자 산출물)만 읽고 문서 오라클이 없다(doc: None).
            command: "audit",
            doc: None,
            args: vec![s("audit"), p(&audit_dir), s("--json")],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // keygen — 문서를 열지 않는 발급 명령. 봉투는 키 메타·경로 에코뿐.
            command: "keygen",
            doc: None,
            args: vec![
                s("keygen"),
                s("--key-id"),
                s("prov.example/sweep#1"),
                s("--out"),
                p(&keygen_out),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // verify-signature 기형 경로 — 판정(malformed)은 봉투 데이터(exit 3).
            command: "verify-signature",
            doc: None,
            args: vec![
                s("verify-signature"),
                p(&sig_capsule),
                s("--keyring"),
                p(&sig_keyring),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // harness-status — 깨진 캡슐 하나로 verdict:broken(exit 3) 경로 고정.
            command: "harness-status",
            doc: None,
            args: vec![s("harness-status"), p(&harness_dir), s("--json")],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // harness wrap — 실산출 경로. 봉투는 경로·해시·연번뿐이라 문서
            // 오라클이 없다(doc: None); steps 는 개수라 문서 문자열이 아니다.
            command: "harness",
            doc: None,
            args: vec![
                s("harness"),
                s("wrap"),
                s("--plan"),
                harness_plan,
                s("--dir"),
                p(&harness_wrap_dir),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // anchor 미등재 판정 — 판정은 봉투 데이터(exit 3), 문서 오라클 없음.
            command: "anchor",
            doc: None,
            args: vec![
                s("anchor"),
                s("verify"),
                p(&sig_capsule),
                s("--log"),
                p(&anchor_log),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // anchor add — 등재 성공 경로(seq·capsuleSha256). checkpoint 보다
            // 먼저 선언해야 아래 체크포인트가 항목 1을 본다.
            command: "anchor",
            doc: None,
            args: vec![
                s("anchor"),
                s("add"),
                p(&sig_capsule),
                s("--log"),
                p(&anchor_seq_log),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // gate deny 기본 — 빈 규칙은 통과가 아니다(exit 3), 문서 오라클 없음.
            command: "gate",
            doc: None,
            args: vec![
                s("gate"),
                p(&sig_capsule),
                s("--policy"),
                p(&gate_policy),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // [#4551] disclose 픽스처 발급 — 아래 3개 레시피가 이 캡슐을 쓴다.
            command: "replay",
            doc: Some(table.clone()),
            args: vec![
                s("replay"),
                s("--plan-json"),
                disclose_plan.clone(),
                s("--capsule"),
                p(&disclose_capsule),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // disclose redact — 커밋 치환·개봉 분리 (capsule/redacted/opening 실물).
            command: "disclose",
            doc: None,
            args: vec![
                s("disclose"),
                s("redact"),
                p(&disclose_capsule),
                s("-o"),
                p(&disclose_redacted),
                s("--opening-out"),
                p(&disclose_opening),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // disclose verify — 전체 개봉 대조 ok (verifiedFields/unopened 실물).
            command: "disclose",
            doc: None,
            args: vec![
                s("disclose"),
                s("verify"),
                p(&disclose_redacted),
                s("--opening"),
                p(&disclose_opening),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // disclose restore — 바이트 복원 (restoredSha256/byteIdentical 실물).
            command: "disclose",
            doc: None,
            args: vec![
                s("disclose"),
                s("restore"),
                p(&disclose_redacted),
                s("--opening"),
                p(&disclose_opening),
                s("-o"),
                p(&disclose_restored),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // [#4553] settle propose — 3해시 고정 (미서명: signed false 실측).
            command: "settle",
            doc: None,
            args: vec![
                s("settle"),
                s("propose"),
                s("--workorder"),
                p(&settle_wo),
                s("--capsule"),
                p(&disclose_capsule),
                s("--gate-envelope"),
                p(&settle_gate),
                s("-o"),
                p(&settle_claim),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // settle verify 전 축 — 미서명 청구라 signerOk false = exit 3 (판정 데이터).
            command: "settle",
            doc: None,
            args: vec![
                s("settle"),
                s("verify"),
                p(&settle_claim),
                s("--workorder"),
                p(&settle_wo),
                s("--capsule"),
                p(&disclose_capsule),
                s("--gate-envelope"),
                p(&settle_gate),
                s("--keyring"),
                p(&sig_keyring),
                s("--ledger"),
                p(&settle_ledger),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // settle record — 원장 기입 (seq 0, 동형 체인).
            command: "settle",
            doc: None,
            args: vec![
                s("settle"),
                s("record"),
                p(&settle_claim),
                s("--ledger"),
                p(&settle_ledger),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // settle record 재시도 — P3 이중 청구 (duplicate·existingSeq 실물).
            command: "settle",
            doc: None,
            args: vec![
                s("settle"),
                s("record"),
                p(&settle_claim),
                s("--ledger"),
                p(&settle_ledger),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // [#4558] audit-report — 픽스처 루트 전 캡슐 대상, opt-in 미지정 절은
            // null 로 실린다(전 필드 실측).
            command: "audit-report",
            doc: None,
            args: vec![
                s("audit-report"),
                s(dir.to_str().expect("경로")),
                s("-o"),
                p(&y10_report),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // recall-scope — disclose 픽스처 캡슐을 오염 지목(자기 자신 = 회수 1호).
            // settle 원장의 청구 대상이 바로 이 캡슐이라 claims 1건이 실측된다.
            command: "recall-scope",
            doc: None,
            args: vec![
                s("recall-scope"),
                s("--contaminated"),
                p(&disclose_capsule),
                s("--among"),
                s(dir.to_str().expect("경로")),
                s("--ledger"),
                p(&settle_ledger),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // conformance L1 — 픽스처 루트에는 영수증 없는 수제 캡슐이 섞여 있어
            // nonconformant(exit 3)가 결정론 — checks/achieved/verdict 전 필드 실측.
            command: "conformance",
            doc: None,
            args: vec![
                s("conformance"),
                s(dir.to_str().expect("경로")),
                s("--level"),
                s("L1"),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            // anchor checkpoint — entries·merkleRoot·upToSeq 는 여기서만 나온다.
            command: "anchor",
            doc: None,
            args: vec![
                s("anchor"),
                s("checkpoint"),
                s("--log"),
                p(&anchor_seq_log),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // bundle export — 뿌리 1개 폐쇄집합 (아래 verify 가 이 산출을 쓴다).
            command: "bundle",
            doc: None,
            args: vec![
                s("bundle"),
                s("export"),
                p(&bundle_capsule),
                s("-o"),
                p(&bundle_out),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // bundle verify — 미서명 뿌리 번들은 빈 keyring 도메인 기준 ok.
            command: "bundle",
            doc: None,
            args: vec![
                s("bundle"),
                s("verify"),
                p(&bundle_out),
                s("--trust-domain"),
                p(&bundle_domain),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            // lineage 깨진 머리 — kind 불일치 캡슐 하나로 valid:false 경로(exit 3)를
            // 고정한다. 봉투는 경로·해시·판정뿐, 문서 오라클이 없다(doc: None).
            command: "lineage",
            doc: None,
            args: vec![
                s("lineage"),
                p(&audit_dir.join("broken.capsule.json")),
                s("--json"),
            ],
            stdin: None,
            exit: 3,
            ndjson: false,
        },
        Recipe {
            command: "run",
            doc: Some(table.clone()),
            args: vec![s("run"), s("--plan-json"), plan, s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "batch",
            doc: Some(main.clone()),
            args: vec![s("batch"), s("export-text"), s("--json")],
            stdin: Some(format!("{}\n", p(&main))),
            exit: 0,
            ndjson: true,
        },
        Recipe {
            command: "thumbnail",
            doc: Some(thumb.clone()),
            args: vec![s("thumbnail"), p(&thumb), s("--base64"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-svg",
            doc: Some(main.clone()),
            args: vec![
                s("export-svg"),
                p(&main),
                s("-o"),
                out("svg"),
                s("-p"),
                s("0"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-pdf",
            doc: Some(main.clone()),
            args: vec![
                s("export-pdf"),
                p(&main),
                s("-o"),
                out("out.pdf"),
                s("-p"),
                s("0"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-markdown",
            doc: Some(main.clone()),
            args: vec![
                s("export-markdown"),
                p(&main),
                s("-o"),
                out("md"),
                s("-p"),
                s("0"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-hwpx",
            doc: Some(main.clone()),
            args: vec![s("export-hwpx"), p(&main), out("out.hwpx"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-hml",
            doc: Some(hml.clone()),
            args: vec![
                s("export-hml"),
                p(&hml),
                s("-o"),
                out("out.hml"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-doclang",
            doc: Some(doclang.clone()),
            args: vec![
                s("export-doclang"),
                p(&doclang),
                s("-o"),
                out("out.xml"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "extract-pages",
            doc: Some(main.clone()),
            args: vec![
                s("extract-pages"),
                p(&main),
                out("extract.hwp"),
                s("--from"),
                s("1"),
                s("--to"),
                s("1"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "convert",
            doc: Some(main.clone()),
            args: vec![s("convert"), p(&main), out("convert.hwp"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "ir-diff",
            doc: Some(main.clone()),
            args: vec![s("ir-diff"), p(&main), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // [#4113] 어떤 표본이든 항상 만족하는 기대(부조리 문자열의 부재)로 호출한다 —
        // 스윕의 관심은 판정 결과가 아니라 봉투의 표지(expectations[].actual)다.
        Recipe {
            command: "verify",
            doc: Some(main.clone()),
            args: vec![
                s("verify"),
                p(&main),
                s("--expect-not-contains"),
                s("존재할리없는-스윕-문자열-4113"),
                s("--json"),
            ],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "render-diff",
            doc: Some(main.clone()),
            args: vec![s("render-diff"), p(&main), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        // [#3918 승격 3호] scan — 파싱이 성공하는 표본이므로 probe.error 는 실리지
        // 않는다. 스윕의 관심은 판정 결과가 아니라 표지(untrustedContent·Fields)가
        // 항상 실리고 지도 밖 경로를 광고하지 않는 것이다.
        Recipe {
            command: "scan",
            doc: Some(main.clone()),
            args: vec![s("scan"), p(&main), s("--probe"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "capabilities",
            doc: None,
            args: vec![s("capabilities")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
        Recipe {
            command: "export-provenance-map",
            doc: None,
            args: vec![s("export-provenance-map"), s("--json")],
            stdin: None,
            exit: 0,
            ndjson: false,
        },
    ]
}

/// 스윕 1회분 — 레시피·봉투·오라클을 **프로세스당 한 번만** 만든다.
///
/// 가드 4종이 각자 전 명령을 다시 실행하면 같은 일을 네 번 하고, `export-pdf` 같은
/// 무거운 렌더가 스레드 병렬로 겹쳐 메모리 할당 실패로 죽는 것을 실측했다
/// (`memory allocation of 16273348 bytes failed`). 한 번만 돌려 공유한다.
struct Sweep {
    recipes: Vec<Recipe>,
    envelopes: BTreeMap<&'static str, Vec<Value>>,
    oracles: BTreeMap<PathBuf, DocOracle>,
}

static SWEEP: OnceLock<Sweep> = OnceLock::new();

fn sweep() -> &'static Sweep {
    SWEEP.get_or_init(|| {
        let recipes = recipes();
        let mut envelopes: BTreeMap<&'static str, Vec<Value>> = BTreeMap::new();
        let mut oracles: BTreeMap<PathBuf, DocOracle> = BTreeMap::new();
        for r in &recipes {
            // [#3885] insert 는 같은 명령의 앞 레시피 봉투를 덮어쓴다 — edit 처럼
            // 하위 명령이 여럿인 축은 레시피도 여럿이라, 누적하지 않으면 마지막
            // 하위 명령만 검사되고 나머지는 조용히 빠진다(redact 가 그렇게 빠졌다).
            envelopes
                .entry(r.command)
                .or_default()
                .extend(run_recipe(r));
            if let Some(doc) = &r.doc {
                if !oracles.contains_key(doc) {
                    oracles.insert(doc.clone(), oracle(doc));
                }
            }
        }
        Sweep {
            recipes,
            envelopes,
            oracles,
        }
    })
}

fn envelopes_of(command: &str) -> &'static [Value] {
    sweep()
        .envelopes
        .get(command)
        .unwrap_or_else(|| panic!("{command} 레시피 결과가 없습니다"))
}

/// 레시피를 실행해 봉투들을 얻는다(NDJSON 이면 여러 개).
fn run_recipe(r: &Recipe) -> Vec<Value> {
    let out = match &r.stdin {
        Some(body) => run_with_stdin(&r.args, body),
        None => run(&r.args),
    };
    assert_eq!(
        out.status.code(),
        Some(r.exit),
        "레시피가 실패했습니다 — 가드가 공허하게 통과하지 않도록 레시피를 고치세요.\n{}",
        describe(&r.args, &out)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    if r.ndjson {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str(l)
                    .unwrap_or_else(|e| panic!("NDJSON 줄이 JSON 이 아닙니다 ({e}): {l}"))
            })
            .collect()
    } else {
        vec![serde_json::from_str(text.trim()).unwrap_or_else(|e| {
            panic!(
                "stdout 이 JSON 이 아닙니다 ({e}).\n{}",
                describe(&r.args, &out)
            )
        })]
    }
}

// ── 가드 ① 지도가 `--json` 명령 전부를 덮는가 ───────────────────────────────

#[test]
fn provenance_map_covers_every_json_command() {
    let cap = capabilities();
    let map = provenance_map();
    let commands = map["commands"].as_object().expect("commands 객체");

    let declared = json_commands(&cap);
    assert!(
        declared.len() >= 20,
        "capabilities 파싱이 거의 0건이면 이 가드는 공허하게 통과한다: {declared:?}"
    );

    let missing: Vec<&String> = declared
        .iter()
        .filter(|n| !commands.contains_key(n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "--json 계약 명령인데 출처 지도에 없는 것: {missing:?}\n\
         src/provenance.rs 의 MAP 에 항목을 추가하세요. 문서 값을 담지 않는 명령이라도 \
         빈 목록과 사유(note)를 남겨야 소비자가 '판정했고 없음'을 알 수 있습니다."
    );

    // 반대 방향 — 지도에 남은 유령 항목(이름이 바뀌었거나 사라진 명령)도 드리프트다.
    let all_names: BTreeSet<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    let stale: Vec<&String> = commands
        .keys()
        .filter(|k| !all_names.contains(k.as_str()))
        .collect();
    assert!(
        stale.is_empty(),
        "capabilities 에 없는 명령이 출처 지도에 남아 있습니다: {stale:?}"
    );

    // 항목 모양: 근거 없는 선언은 검토할 수 없다.
    for (name, entry) in commands {
        let untrusted = entry["untrusted"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}.untrusted 배열 필요: {entry}"));
        let origins = entry["origins"]
            .as_object()
            .unwrap_or_else(|| panic!("{name}.origins 객체 필요: {entry}"));
        assert!(
            entry["note"].as_str().is_some_and(|n| !n.trim().is_empty()),
            "{name}.note 가 비었습니다 — 특히 빈 목록은 사유가 계약입니다: {entry}"
        );
        for path in untrusted {
            let path = path
                .as_str()
                .unwrap_or_else(|| panic!("{name}: 경로는 문자열"));
            let origin = origins
                .get(path)
                .and_then(|o| o.as_str())
                .unwrap_or_else(|| panic!("{name}.origins 에 {path} 근거가 없습니다: {entry}"));
            assert!(
                !origin.trim().is_empty(),
                "{name}.{path} 의 근거가 빈 문자열입니다"
            );
        }
        assert_eq!(
            origins.len(),
            untrusted.len(),
            "{name}: origins 와 untrusted 개수가 다릅니다(낡은 근거가 남았습니다): {entry}"
        );
    }
}

// ── 가드 ② 문서 텍스트를 내보내는 명령이 지도에 빠져 있으면 실패 ─────────────

#[test]
fn every_text_bearing_command_declares_untrusted_fields() {
    let cap = capabilities();
    let map = provenance_map();

    // 레시피가 `--json` 명령 전부를 덮는지 먼저 본다 — 새 명령이 스윕 밖으로
    // 조용히 빠져나가면 그 다음 검사는 아무 의미가 없다.
    let sweep = sweep();
    let covered: BTreeSet<&str> = sweep.recipes.iter().map(|r| r.command).collect();
    let uncovered: Vec<String> = json_commands(&cap)
        .into_iter()
        .filter(|n| !covered.contains(n.as_str()))
        .filter(|n| !SWEEP_EXEMPT.iter().any(|(c, _)| c == n))
        .collect();
    assert!(
        uncovered.is_empty(),
        "출처 스윕이 실행해 보지 않은 --json 명령: {uncovered:?}\n\
         tests/provenance_contract.rs 의 recipes() 에 호출 방법을 더하거나, \
         문서를 입력으로 받지 않는 명령이면 SWEEP_EXEMPT 에 사유와 함께 넣으세요."
    );
    for (name, why) in SWEEP_EXEMPT {
        assert!(!why.trim().is_empty(), "{name} 의 면제 사유가 비었습니다");
    }
    for (key, why) in CALLER_ECHO {
        assert!(!why.trim().is_empty(), "{key} 의 제외 사유가 비었습니다");
    }

    let mut text_bearing: BTreeSet<&str> = BTreeSet::new();
    let mut failures: Vec<String> = Vec::new();

    for r in &sweep.recipes {
        let Some(doc) = r.doc.clone() else {
            continue;
        };
        let or = &sweep.oracles[&doc];
        assert!(
            !or.tokens.is_empty() || !or.exact.is_empty(),
            "{} 에서 문서 문자열 오라클을 만들지 못했습니다 — 오라클이 비면 그 문서를 쓰는 \
             레시피는 아무것도 검사하지 못합니다. 본문이 있는 샘플로 바꾸세요.",
            doc.display()
        );

        let declared = declared_paths(&map, r.command);
        let declared_roots: BTreeSet<&str> = declared.iter().map(|p| root_of(p)).collect();

        for env in envelopes_of(r.command) {
            // [#3885] 표지 존재는 오라클 매치와 무관한 모든 봉투의 의무다. 종전에는
            // 문서 문자열이 "발견된" 봉투만 검사해서, 표지 키 자체가 없는 봉투는
            // 어느 단언에도 걸리지 않고 빠져나갔다 — redact 가 정확히 그 구멍이었다.
            if env.get("untrustedContent").is_none() || env.get("untrustedFields").is_none() {
                failures.push(format!(
                    "  - {}: 봉투에 출처 표지(untrustedContent/untrustedFields)가 없습니다",
                    r.command
                ));
            }
            let mut found = BTreeSet::new();
            scan(env, "", or, &mut found);
            if found.is_empty() {
                continue;
            }
            text_bearing.insert(r.command);

            if env["untrustedContent"] != Value::Bool(true) {
                failures.push(format!(
                    "  - {}: 문서 문자열이 {found:?} 에 실렸는데 untrustedContent 가 true 가 아닙니다",
                    r.command
                ));
            }
            for path in &found {
                if !declared_roots.contains(root_of(path)) {
                    failures.push(format!(
                        "  - {}: 봉투의 {path} 에 문서 문자열이 실렸는데 지도에 선언이 없습니다 \
                         (선언된 최상위 키: {declared_roots:?})",
                        r.command
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "선언되지 않은 문서 파생 필드 {}건:\n{}\n\n\
         src/provenance.rs 의 MAP 에 경로와 근거(origin)를 추가하세요. \
         봉투가 문서 값을 담는데 지도가 침묵하면, 그 값을 받은 에이전트는 문서에 적힌 \
         문장을 도구의 지시로 읽습니다.",
        failures.len(),
        failures.join("\n"),
    );

    // 탐지기 자체가 죽으면 이 테스트는 아무것도 안 하고 통과한다 — 그 상태를 막는다.
    assert!(
        text_bearing.len() >= 6,
        "문서 문자열을 실은 명령이 {}건뿐입니다 — 탐지기가 고장 났을 가능성이 큽니다: {text_bearing:?}",
        text_bearing.len()
    );
    for must in ["export-text", "search", "export-structure", "export-tables"] {
        assert!(
            text_bearing.contains(must),
            "{must} 는 정의상 문서 텍스트를 내보내는 명령인데 탐지되지 않았습니다: {text_bearing:?}"
        );
    }
}

// ── 가드 ③ 봉투의 표지가 지도와 일치하는가 ─────────────────────────────────

#[test]
fn untrusted_flag_matches_map() {
    let map = provenance_map();
    let mut checked = 0usize;

    for r in &sweep().recipes {
        let declared: BTreeSet<String> = declared_paths(&map, r.command).into_iter().collect();
        for env in envelopes_of(r.command) {
            checked += 1;
            let flag = env["untrustedContent"].as_bool().unwrap_or_else(|| {
                panic!(
                    "{}: untrustedContent(bool) 표지가 없습니다: {env}",
                    r.command
                )
            });
            let fields: Vec<&str> = env["untrustedFields"]
                .as_array()
                .unwrap_or_else(|| {
                    panic!(
                        "{}: untrustedFields(배열) 표지가 없습니다: {env}",
                        r.command
                    )
                })
                .iter()
                .map(|f| {
                    f.as_str()
                        .unwrap_or_else(|| panic!("{}: 경로는 문자열: {env}", r.command))
                })
                .collect();

            let unknown: Vec<&&str> = fields.iter().filter(|f| !declared.contains(**f)).collect();
            assert!(
                unknown.is_empty(),
                "{}: 봉투 표지가 지도에 없는 경로를 광고합니다 {unknown:?}\n지도: {declared:?}",
                r.command
            );
            assert_eq!(
                flag,
                !fields.is_empty(),
                "{}: untrustedContent 와 untrustedFields 가 서로 다른 말을 합니다: {env}",
                r.command
            );
        }
    }
    assert!(checked >= 20, "검사한 봉투가 {checked}건뿐입니다");
}

/// 표지는 **항상** 실린다 — 문서를 열지 않는 명령의 봉투도 `false` 를 명시한다.
/// 키가 없으면 소비자는 "문서 값 없음"과 "출처를 판정하지 않는 옛 바이너리"를
/// 구별할 수 없다(#3707 textSecurity 와 같은 규약).
#[test]
fn every_json_envelope_carries_the_flag() {
    for r in &sweep().recipes {
        for env in envelopes_of(r.command) {
            assert!(
                env.get("untrustedContent").is_some_and(Value::is_boolean),
                "{}: untrustedContent 표지 누락: {env}",
                r.command
            );
            assert!(
                env.get("untrustedFields").is_some_and(Value::is_array),
                "{}: untrustedFields 표지 누락: {env}",
                r.command
            );
        }
    }
}

// ── 가드 ④ 새 명령의 표면 배선(capabilities·help·MCP·실패 규약) ─────────────

#[test]
fn export_provenance_map_is_wired_across_every_surface() {
    // capabilities: --json 계약 명령으로 선언됐는가.
    let cap = capabilities();
    let entry = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "export-provenance-map")
        .expect("capabilities 에 export-provenance-map 이 없습니다");
    assert_eq!(entry["json"], true, "{entry}");
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(flags.contains(&"--json"), "{entry}");

    // 선언한 플래그가 실재하는가 — 선언만 있고 없는 플래그는 계약의 거짓말이다.
    for flag in &flags {
        let out = run(&[s("export-provenance-map"), s(flag)]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "선언한 플래그 {flag} 를 CLI 가 받지 않습니다"
        );
    }

    // --help: 사람이 보는 목록에도 있어야 한다.
    let help = run(&[s("--help")]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(
        help_text.contains("export-provenance-map"),
        "--help 에 export-provenance-map 이 없습니다"
    );

    // MCP: --json 명령은 MCP 도구로도 노출된다(+ 필수 3종 + required 배열).
    let mcp = json_of(&["capabilities", "--mcp"]);
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["cli"]["command"] == "export-provenance-map")
        .expect("MCP 도구가 없습니다");
    assert_eq!(tool["name"], "hwp_export_provenance_map", "{tool}");
    assert!(tool["description"].is_string(), "{tool}");
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert!(tool["inputSchema"]["properties"].is_object(), "{tool}");
    assert!(
        tool["inputSchema"]["required"].is_array(),
        "required 는 배열이어야 한다(빈 배열이라도): {tool}"
    );

    // 실패 시 stdout 0바이트 — 부분 출력을 성공으로 오인하지 않게 하는 규약.
    let bad = run(&[s("export-provenance-map"), s("--nope")]);
    assert_eq!(
        bad.status.code(),
        Some(2),
        "{}",
        describe(&[s("export-provenance-map"), s("--nope")], &bad)
    );
    assert!(
        bad.stdout.is_empty(),
        "실패인데 stdout 이 비지 않았습니다: {:?}",
        String::from_utf8_lossy(&bad.stdout)
    );
}

/// 지도는 `capabilities` 의 `jsonContract.provenance` 로도 광고된다 — 자기서술만
/// 읽는 에이전트가 표지의 의미와 지도의 위치를 알 수 있어야 한다.
#[test]
fn capabilities_advertises_the_provenance_contract() {
    let cap = capabilities();
    let p = &cap["jsonContract"]["provenance"];
    assert!(
        p.is_object(),
        "capabilities.jsonContract.provenance 가 없습니다: {cap}"
    );
    let fields: Vec<&str> = p["fields"]
        .as_array()
        .expect("provenance.fields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(fields.contains(&"untrustedContent"), "{p}");
    assert!(fields.contains(&"untrustedFields"), "{p}");
    assert!(
        p["map"]
            .as_str()
            .is_some_and(|m| m.contains("export-provenance-map")),
        "지도로 가는 길이 없습니다: {p}"
    );
    assert!(
        p["meaning"].as_str().is_some_and(|m| !m.trim().is_empty()),
        "표지의 의미 설명이 비었습니다: {p}"
    );
}

/// 기존 소비자 무해 — 표지는 **추가**일 뿐이라 `schemaVersion` 은 그대로다.
/// 올려야 할 변경(필드 변경·삭제)이 아님을 계약으로 고정한다.
#[test]
fn schema_version_stays_1_0_because_the_flag_is_additive() {
    let cap = capabilities();
    assert_eq!(
        cap["jsonContract"]["schemaPolicy"], "필드 추가 허용, 변경·삭제는 schemaVersion 범프",
        "추가 허용 정책이 바뀌었다면 이 판단(범프 없음)을 다시 해야 합니다"
    );
    for r in &sweep().recipes {
        for env in envelopes_of(r.command) {
            if let Some(v) = env.get("schemaVersion") {
                assert_eq!(
                    v, "1.0",
                    "{}: schemaVersion 이 바뀌었습니다: {env}",
                    r.command
                );
            }
        }
    }
    assert_eq!(provenance_map()["schemaVersion"], "1.0");
}

/// [#3885] 스윕 면제는 "문서 오라클을 만들 수 없다"는 뜻이지 "표지를 안 실어도
/// 된다"가 아니다 — 종전에는 면제 명령의 봉투를 아무 가드도 열어 보지 않아, 스키마
/// 봉투 2종(export-ir-schema·export-capabilities-schema)이 표지 없이 나가는 것을
/// 아무도 몰랐다. 면제 명령마다 실제 호출로 표지 존재를 고정하고, 호출표 완전성을
/// SWEEP_EXEMPT 와 기계로 대조한다 — 새 면제가 표지 검사까지 조용히 면제받는
/// 드리프트를 막는다.
#[test]
fn sweep_exempt_envelopes_still_carry_provenance_marks() {
    let dir = tmp_dir();
    let ingest = dir.join("prov-exempt-ingest.json");
    std::fs::write(&ingest, r#"{"version":"1","questions":[]}"#).expect("ingest 픽스처");
    let ingest_out = dir.join("prov-exempt-ingest.hwpx");

    let invocations: BTreeMap<&str, Vec<String>> = [
        ("export-ir-schema", vec![s("export-ir-schema"), s("--json")]),
        (
            "export-capabilities-schema",
            vec![s("export-capabilities-schema"), s("--json")],
        ),
        (
            "export-agent-manifest",
            vec![s("export-agent-manifest"), s("--json")],
        ),
        // [#3808 선등재] 아직 devel 에 없는 명령 — 표는 SWEEP_EXEMPT 기준으로만
        // 순회하므로 초과 항목은 미사용으로 잠들어 있다가, #3808 이 그 명령을
        // 면제 목록에 넣는 순간 자동으로 검사에 편입된다(어느 쪽이 먼저 머지돼도
        // 후속 수정 없음 — 3-PR 누적 머지에서 이 항목 유무로 실측 확인).
        (
            "export-plan-schema",
            vec![s("export-plan-schema"), s("--json")],
        ),
        ("export-ontology", vec![s("export-ontology"), s("--json")]),
        (
            "build-from-ingest",
            vec![
                s("build-from-ingest"),
                ingest.to_str().expect("경로").to_string(),
                s("-o"),
                ingest_out.to_str().expect("경로").to_string(),
                s("--json"),
            ],
        ),
    ]
    .into_iter()
    .collect();

    for (name, _why) in SWEEP_EXEMPT {
        let args = invocations.get(name).unwrap_or_else(|| {
            panic!(
                "SWEEP_EXEMPT 의 {name} 이 표지 존재 검사 호출표에 없습니다 — \
                 이 테스트의 invocations 에 호출 방법을 더하세요"
            )
        });
        let out = run(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{name} 호출 실패:\n{}",
            describe(args, &out)
        );
        let env: Value = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{name} stdout 이 JSON 이 아닙니다({e})"));
        assert!(
            env.get("untrustedContent").is_some() && env.get("untrustedFields").is_some(),
            "{name} 봉투에 출처 표지(untrustedContent/untrustedFields)가 없습니다"
        );
        // 면제 = 문서를 열지 않는 명령이므로 값도 false/빈 배열이어야 한다.
        assert_eq!(
            env["untrustedContent"],
            Value::Bool(false),
            "{name}: 문서를 열지 않는데 untrustedContent 가 false 가 아닙니다"
        );
    }
}

/// [#3880 T1] 선언한 `recordFields` 가 실물 봉투에 실제로 나타난다 — 스윕 전수.
///
/// `info` 의 `warnings` 부재(T1, #3882 로 수정)가 통과했던 구멍이 이것이다:
/// 자기서술이 필드를 광고하는데 **아무 가드도 실물과 대조하지 않았다.** 스윕이
/// 이미 전 `--json` 명령을 유효 인자로 실행하므로, 명령별 봉투 최상위 키 합집합과
/// 선언을 대조한다. 중첩 경로(`steps[].confusable` 류)는 최상위 대조 대상이
/// 아니다 — 이 테스트는 최상위 `recordFields`의 전수 계약만 다룬다.
///
/// 조건부 필드는 **사유와 함께** CONDITIONAL_RECORD_FIELDS 에 적는다 — 사유 없는
/// 허용목록은 가드를 무력화한다. 가능하면 허용 대신 레시피가 그 필드를 실제로
/// 나오게 하는 쪽을 택한다(예: sanitize 레시피가 `-o` 로 저장해 `output` 을 낸다).
const CONDITIONAL_RECORD_FIELDS: &[(&str, &str, &str)] = &[
    // (명령, 필드, 스윕 레시피가 그 필드를 못 내는 사유)
];

#[test]
fn declared_record_fields_actually_appear_in_envelopes() {
    let cap = capabilities();
    let sweep = sweep();

    let mut observed: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for r in &sweep.recipes {
        let keys = observed.entry(r.command).or_default();
        for env in envelopes_of(r.command) {
            if let Some(o) = env.as_object() {
                keys.extend(o.keys().cloned());
            }
        }
    }

    let empty = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    for c in cap["commands"].as_array().expect("commands") {
        if c["json"] != Value::Bool(true) {
            continue;
        }
        let name = c["name"].as_str().expect("name");
        // 스윕 밖(면제) 명령의 봉투는 sweep_exempt_envelopes_still_carry_provenance_marks
        // 가 따로 연다 — 여기서는 스윕이 실행해 본 명령만 대조한다.
        let Some(keys) = observed.get(name) else {
            continue;
        };
        for field in c["recordFields"].as_array().unwrap_or(&empty) {
            let Some(field) = field.as_str() else {
                continue;
            };
            if field.contains('[') || field.contains('.') {
                continue;
            }
            if keys.contains(field) {
                continue;
            }
            if let Some((_, _, why)) = CONDITIONAL_RECORD_FIELDS
                .iter()
                .find(|(cmd, f, _)| *cmd == name && *f == field)
            {
                assert!(
                    !why.trim().is_empty(),
                    "{name}.{field} 허용 사유가 비었습니다"
                );
                continue;
            }
            problems.push(format!(
                "  - {name}: 선언한 '{field}' 가 스윕 봉투 어디에도 없습니다 (실물 키: {keys:?})"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "자기서술이 광고하는 필드가 실물에 없는 것 {}건:\n{}\n\n\
         capabilities 선언을 실물에 맞추거나, 레시피가 그 필드를 실제로 내게 하거나, \
         조건부라면 CONDITIONAL_RECORD_FIELDS 에 사유와 함께 적으세요.",
        problems.len(),
        problems.join("\n"),
    );
}

/// [R10 조각] `json:true` 명령은 `recordFields` 를 비워 두고 가드를 지나갈 수 없다.
///
/// 위의 전수 대조는 **선언한** 필드가 실물에 나타나는지만 본다 — 선언이 아예
/// 없거나 빈 배열이면 대조할 것이 없어 공허하게 통과한다. 즉 "선언 회피가 가드
/// 회피의 가장 쉬운 길"(R12 가 map 축에서 봉쇄한 바로 그 부류)이 recordFields
/// 축에는 열려 있었다. 이 래칫은 선언의 존재·비어있지 않음 자체를 전수로
/// 요구해 그 길을 닫는다. 현재 `json:true` 전 명령이 비어있지 않은 선언을
/// 가지므로 면제 목록은 0건으로 시작하며, 봉투 필드를 약속할 수 없는 명령이
/// 생기면 **사유와 함께** EMPTY_RECORD_FIELDS_EXEMPT 에만 적는다.
const EMPTY_RECORD_FIELDS_EXEMPT: &[(&str, &str)] = &[
    // (명령, json 봉투의 고정 필드를 약속할 수 없는 사유)
];

#[test]
fn every_json_command_declares_nonempty_record_fields() {
    let cap = capabilities();
    let mut problems: Vec<String> = Vec::new();
    for c in cap["commands"].as_array().expect("commands") {
        if c["json"] != Value::Bool(true) {
            continue;
        }
        let name = c["name"].as_str().expect("name");
        if let Some((_, why)) = EMPTY_RECORD_FIELDS_EXEMPT.iter().find(|(n, _)| *n == name) {
            assert!(!why.trim().is_empty(), "{name} 면제 사유가 비었습니다");
            continue;
        }
        let declared = c["recordFields"].as_array().map(Vec::len).unwrap_or(0);
        if declared == 0 {
            problems.push(format!(
                "  - {name}: json:true 인데 recordFields 가 없거나 비어 있습니다 — \
                 declared_record_fields_actually_appear_in_envelopes 가 공허 통과합니다"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "recordFields 공선언 회피 {}건:\n{}\n\n\
         봉투가 약속하는 최상위 필드를 capabilities 에 선언하거나, 약속할 수 없다면 \
         EMPTY_RECORD_FIELDS_EXEMPT 에 사유와 함께 적으세요.",
        problems.len(),
        problems.join("\n"),
    );
}
