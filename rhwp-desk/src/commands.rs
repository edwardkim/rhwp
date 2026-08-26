//! Tauri 커맨드 — UI 가 부르는 백엔드 표면.
//!
//! 원칙(설계 문서 §4): 문서를 읽고 바꾸는 모든 일은 `rhwp.exe` 서브프로세스
//! (계약 경로)를 거친다. 이 파일은 프로세스 실행·저널 기록·결과 중계만 하고,
//! 문서 판단은 하지 않는다. M0 에서는 빠른 경로(코어 crate 직링크)를 생략하고
//! 계약 경로만으로 시작한다.

use crate::{caps, engine, journal, planner, planner_net};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// stdout/stderr 보존 상한 — 저널이 무한히 크지 않게.
const TAIL_LIMIT: usize = 200_000;
const STDERR_LIMIT: usize = 8_000;

pub struct AppState {
    pub data_dir: PathBuf,
}

impl AppState {
    fn journal_dir(&self) -> PathBuf {
        self.data_dir.join("journal")
    }
    fn render_dir(&self) -> PathBuf {
        self.data_dir.join("render")
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub path: String,
    pub source: String,
    pub version: Option<String>,
}

struct ExecOut {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u64,
}

/// 엔진 서브프로세스를 실행한다. Windows 에서는 콘솔 창이 뜨지 않게 한다.
fn exec_engine(engine_path: &str, args: &[String]) -> Result<ExecOut, String> {
    let mut cmd = Command::new(engine_path);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let start = Instant::now();
    let out = cmd
        .output()
        .map_err(|e| format!("엔진 실행 실패 ({engine_path}): {e}"))?;
    Ok(ExecOut {
        exit_code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn truncate_tail(s: &str, limit: usize) -> String {
    if s.len() <= limit {
        return s.to_string();
    }
    // UTF-8 경계 보정
    let mut cut = s.len() - limit;
    while !s.is_char_boundary(cut) {
        cut += 1;
    }
    format!("…(앞 {}바이트 생략)…{}", cut, &s[cut..])
}

/// 실행 결과를 저널 항목으로 빚는다. stdout 이 JSON 이면 봉투로 승격.
fn make_entry(
    engine_path: &str,
    args: &[String],
    origin: &str,
    out: &ExecOut,
) -> journal::JournalEntry {
    let ts = journal::now_ms();
    let trimmed = out.stdout.trim();
    let envelope = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        serde_json::from_str::<serde_json::Value>(trimmed).ok()
    } else {
        None
    };
    let stdout_tail = if envelope.is_none() && !trimmed.is_empty() {
        Some(truncate_tail(trimmed, TAIL_LIMIT))
    } else {
        None
    };
    let stderr_trim = out.stderr.trim();
    let stderr_tail = if stderr_trim.is_empty() {
        None
    } else {
        Some(truncate_tail(stderr_trim, STDERR_LIMIT))
    };
    journal::JournalEntry {
        id: journal::new_id(ts),
        ts_ms: ts,
        engine: engine_path.to_string(),
        command: args.first().cloned().unwrap_or_default(),
        args: args.to_vec(),
        exit_code: out.exit_code,
        duration_ms: out.duration_ms,
        envelope,
        stdout_tail,
        stderr_tail,
        origin: origin.to_string(),
    }
}

/// 엔진 탐색 + `--version` 확인.
#[tauri::command]
pub fn detect_engine(configured: Option<String>) -> Result<EngineInfo, String> {
    let hit = engine::discover(configured.as_deref())
        .ok_or_else(|| "rhwp.exe 를 찾지 못했습니다".to_string())?;
    let path = hit.path.to_string_lossy().into_owned();
    let version = exec_engine(&path, &["--version".to_string()])
        .ok()
        .filter(|o| o.exit_code == Some(0))
        .map(|o| o.stdout.trim().to_string());
    Ok(EngineInfo {
        path,
        source: hit.source,
        version,
    })
}

/// `capabilities` 를 실행해 원문 JSON 을 돌려준다. 파싱 검증은 하되
/// UI 에는 원문을 넘긴다(단일 출처 계약 — 여기서 가공하지 않는다).
#[tauri::command]
pub fn load_capabilities(engine_path: String) -> Result<serde_json::Value, String> {
    let out = exec_engine(&engine_path, &["capabilities".to_string()])?;
    if out.exit_code != Some(0) {
        return Err(format!(
            "capabilities 실행 실패 (exit {:?}): {}",
            out.exit_code,
            truncate_tail(out.stderr.trim(), 500)
        ));
    }
    caps::parse(&out.stdout)?; // 계약 검증
    serde_json::from_str(&out.stdout).map_err(|e| format!("JSON 파싱 실패: {e}"))
}

/// 도구 호출 1건 — journal-first: 저널에 적힌 뒤에야 카드가 된다.
#[tauri::command]
pub fn run_tool(
    state: tauri::State<'_, AppState>,
    engine_path: String,
    args: Vec<String>,
    origin: Option<String>,
) -> Result<journal::JournalEntry, String> {
    if args.is_empty() {
        return Err("실행할 명령 인자가 비어 있습니다".into());
    }
    let out = exec_engine(&engine_path, &args)?;
    let entry = make_entry(
        &engine_path,
        &args,
        origin.as_deref().unwrap_or("palette"),
        &out,
    );
    journal::append(&state.journal_dir(), &entry).map_err(|e| format!("저널 기록 실패: {e}"))?;
    Ok(entry)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    pub svg: String,
    pub page: u32,
    pub page_count: u32,
    pub entry: journal::JournalEntry,
}

/// 보조 문서 뷰용 페이지 렌더 — `export-svg <file> -p <page-1> --json` 계약 경로.
/// UI 페이지 번호는 1-기준, CLI 는 0-기준이다.
#[tauri::command]
pub fn render_page(
    state: tauri::State<'_, AppState>,
    engine_path: String,
    file: String,
    page: u32,
) -> Result<RenderResult, String> {
    let page0 = page.saturating_sub(1);
    let out_dir = state.render_dir();
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("렌더 폴더 생성 실패: {e}"))?;
    let args: Vec<String> = vec![
        "export-svg".into(),
        file.clone(),
        "-p".into(),
        page0.to_string(),
        "-o".into(),
        out_dir.to_string_lossy().into_owned(),
        "--json".into(),
    ];
    let out = exec_engine(&engine_path, &args)?;
    let entry = make_entry(&engine_path, &args, "viewer", &out);
    journal::append(&state.journal_dir(), &entry).map_err(|e| format!("저널 기록 실패: {e}"))?;

    if out.exit_code != Some(0) {
        return Err(format!(
            "렌더 실패 (exit {:?}): {}",
            out.exit_code,
            truncate_tail(out.stderr.trim(), 500)
        ));
    }
    let manifest = entry
        .envelope
        .as_ref()
        .ok_or("export-svg 매니페스트가 비어 있습니다")?;
    let page_count = manifest
        .get("pageCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let svg_path = manifest
        .get("pages")
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("path"))
        .and_then(|p| p.as_str())
        .ok_or("매니페스트에 페이지 경로가 없습니다")?;
    let svg = std::fs::read_to_string(svg_path)
        .map_err(|e| format!("SVG 읽기 실패 ({svg_path}): {e}"))?;
    Ok(RenderResult {
        svg,
        page,
        page_count,
        entry,
    })
}

/// 저널 꼬리 읽기 — 카드 스트림은 이 결과의 1:1 렌더다.
#[tauri::command]
pub fn read_journal(
    state: tauri::State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<journal::JournalEntry>, String> {
    journal::read_tail(&state.journal_dir(), limit.unwrap_or(200))
        .map_err(|e| format!("저널 읽기 실패: {e}"))
}

/// 파일 연결/명령행으로 넘어온 시작 인자 (문서 경로 등).
#[tauri::command]
pub fn startup_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

/// 경로 존재 확인 — 최근 문서 목록 정리용.
#[tauri::command]
pub fn path_kind(path: String) -> String {
    let p = std::path::Path::new(&path);
    if p.is_file() {
        "file".into()
    } else if p.is_dir() {
        "dir".into()
    } else {
        "missing".into()
    }
}

// ── Planner (LLM 어댑터) ────────────────────────────────────────
//
// 원칙(설계 §5): LLM 은 계획 수립만, 실행·검증·회계는 결정론 자산이 한다.
// 모든 Planner 왕복도 저널에 먼저 기록된다 — 무엇이 나갔는지(프롬프트)와
// 무엇이 돌아왔는지를 사용자가 카드에서 그대로 열어볼 수 있다.

/// 로컬 LLM 서버 자동 탐지.
#[tauri::command]
pub fn probe_local_llm() -> Vec<planner_net::LocalServer> {
    planner_net::probe_ports("127.0.0.1", planner::LOCAL_PORT_CANDIDATES)
}

fn resolve_key(session_key: Option<String>, profile_id: Option<String>) -> Option<String> {
    session_key
        .filter(|k| !k.is_empty())
        .or_else(|| profile_id.as_deref().and_then(planner_net::secret_get))
}

/// 모델 목록 조회 (엔드포인트 등록 폼).
#[tauri::command]
pub fn planner_list_models(
    base_url: String,
    profile_id: Option<String>,
    session_key: Option<String>,
) -> Result<Vec<String>, String> {
    let key = resolve_key(session_key, profile_id);
    planner_net::list_models(&base_url, key.as_deref())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
}

/// 연결 테스트 — 실제 1회 왕복. 실패 원인은 분류된 문장으로.
#[tauri::command]
pub fn planner_test(
    base_url: String,
    model: String,
    profile_id: Option<String>,
    session_key: Option<String>,
) -> PlannerTestResult {
    let key = resolve_key(session_key, profile_id);
    let msgs =
        json!([{ "role": "user", "content": "연결 테스트입니다. '확인' 한 단어로만 답하세요." }]);
    match planner_net::chat(&base_url, &model, key.as_deref(), &msgs, None, Some(20_000)) {
        Ok(out) => PlannerTestResult {
            ok: true,
            latency_ms: out.latency_ms,
            message: format!(
                "응답 수신 ({}ms): {}",
                out.latency_ms,
                out.parsed
                    .content
                    .unwrap_or_default()
                    .chars()
                    .take(60)
                    .collect::<String>()
            ),
        },
        Err(e) => PlannerTestResult {
            ok: false,
            latency_ms: 0,
            message: e,
        },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerChatResult {
    pub parsed: planner::ParsedChat,
    pub entry: journal::JournalEntry,
}

/// Planner 1 라운드 — journal-first: 요청/응답이 저널에 적힌 뒤 카드가 된다.
/// `messages` 는 프런트가 프라이버시 정책(본문 차단/허용)을 이미 적용한 것이다.
#[tauri::command]
pub fn planner_chat(
    state: tauri::State<'_, AppState>,
    base_url: String,
    model: String,
    profile_id: Option<String>,
    session_key: Option<String>,
    messages: serde_json::Value,
    tools: Option<serde_json::Value>,
) -> Result<PlannerChatResult, String> {
    let key = resolve_key(session_key, profile_id);
    let tools_count = tools
        .as_ref()
        .and_then(|t| t.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let result = planner_net::chat(
        &base_url,
        &model,
        key.as_deref(),
        &messages,
        tools.as_ref(),
        None,
    );

    let ts = journal::now_ms();
    let (envelope, exit_code, duration) = match &result {
        Ok(out) => (
            json!({
                "model": model,
                "local": planner::is_local_url(&base_url),
                "latencyMs": out.latency_ms,
                "toolsCount": tools_count,
                "request": { "messages": messages },
                "response": out.response,
                "untrustedContent": true,
                "untrustedFields": ["response"],
            }),
            Some(0),
            out.latency_ms,
        ),
        Err(e) => (
            json!({
                "model": model,
                "toolsCount": tools_count,
                "request": { "messages": messages },
                "error": e,
            }),
            Some(1),
            0,
        ),
    };
    let entry = journal::JournalEntry {
        id: journal::new_id(ts),
        ts_ms: ts,
        engine: base_url.clone(),
        command: "planner/chat".into(),
        args: vec![model.clone()],
        exit_code,
        duration_ms: duration,
        envelope: Some(envelope),
        stdout_tail: None,
        stderr_tail: None,
        origin: "planner".into(),
    };
    journal::append(&state.journal_dir(), &entry).map_err(|e| format!("저널 기록 실패: {e}"))?;

    let out = result?;
    Ok(PlannerChatResult {
        parsed: out.parsed,
        entry,
    })
}

/// `capabilities --mcp` 원문 — 도구 정의의 단일 출처.
#[tauri::command]
pub fn load_mcp_tools(engine_path: String) -> Result<serde_json::Value, String> {
    let out = exec_engine(&engine_path, &["capabilities".into(), "--mcp".into()])?;
    if out.exit_code != Some(0) {
        return Err(format!(
            "capabilities --mcp 실패 (exit {:?}): {}",
            out.exit_code,
            truncate_tail(out.stderr.trim(), 400)
        ));
    }
    serde_json::from_str(&out.stdout).map_err(|e| format!("JSON 파싱 실패: {e}"))
}

/// MCP 도구 정의 → OpenAI tools 배열 변환.
#[tauri::command]
pub fn mcp_to_openai_tools(
    mcp: serde_json::Value,
    allowlist: Option<Vec<String>>,
) -> serde_json::Value {
    planner::mcp_tools_to_openai(&mcp, allowlist.as_deref())
}

/// 도구 호출 인자 → CLI argv (mcp-serve 와 같은 템플릿 치환 규칙).
#[tauri::command]
pub fn map_tool_call(
    cli: serde_json::Value,
    arguments: serde_json::Value,
) -> Result<Vec<String>, String> {
    planner::substitute_cli_args(&cli, &arguments)
}

// ── 자격 증명 (Windows 자격 증명 관리자) ─────────────────────────

#[tauri::command]
pub fn secret_set(profile_id: String, key: String) -> Result<(), String> {
    planner_net::secret_set(&profile_id, &key)
}

#[tauri::command]
pub fn secret_exists(profile_id: String) -> bool {
    planner_net::secret_exists(&profile_id)
}

#[tauri::command]
pub fn secret_delete(profile_id: String) -> Result<(), String> {
    planner_net::secret_delete(&profile_id)
}

/// 폴더에서 HWP/HWPX 문서를 나열한다 — batch 러너의 입력.
#[tauri::command]
pub fn list_documents(dir: String) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(&dir).map_err(|e| format!("폴더 읽기 실패 ({dir}): {e}"))?;
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_file() {
            continue;
        }
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        if matches!(ext.as_deref(), Some("hwp") | Some("hwpx")) {
            out.push(p.to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}
