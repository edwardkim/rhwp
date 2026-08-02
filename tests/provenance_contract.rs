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
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body.as_bytes())
        .expect("stdin 쓰기 실패");
    child.wait_with_output().expect("rhwp 종료 대기 실패")
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
const SWEEP_EXEMPT: &[(&str, &str)] = &[(
    "build-from-ingest",
    "입력이 문서가 아니라 호출자가 만든 ingest JSON 이라 '문서에서 온 문자열' 오라클을 \
     만들 수 없다. 봉투는 경로·바이트·문항/문단 개수뿐임을 지도가 선언하고, \
     tests/issue_3358_ingest_unknown_fields.rs 가 그 봉투를 따로 고정한다.",
)];

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

    // search 질의어는 문서에서 뽑는다 — 매치가 0건이면 봉투가 비어 가드가 공허해진다.
    let main_oracle = oracle(&main);
    let query = main_oracle
        .tokens
        .first()
        .cloned()
        .expect("샘플에서 토큰을 얻지 못했습니다");

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
            command: "search",
            doc: Some(main.clone()),
            args: vec![s("search"), p(&main), query, s("--json")],
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
        let mut envelopes = BTreeMap::new();
        let mut oracles: BTreeMap<PathBuf, DocOracle> = BTreeMap::new();
        for r in &recipes {
            envelopes.insert(r.command, run_recipe(r));
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
