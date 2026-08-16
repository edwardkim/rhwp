//! Planner HTTP 왕복 + 자격 증명 저장 — 부작용 있는 얇은 층.
//!
//! 판단 로직(요청 조립·파싱·분류)은 전부 `planner.rs` 의 순수 함수에 있고,
//! 여기는 (a) ureq 왕복 (b) Windows 자격 증명 관리자 접근만 담당한다.
//! API 키는 평문 설정 파일에 절대 쓰지 않는다.

use crate::planner;
use serde::Serialize;
use serde_json::Value;
use std::time::{Duration, Instant};

const PROBE_TIMEOUT_MS: u64 = 800;
const CHAT_TIMEOUT_MS: u64 = 180_000;

/// 자격 증명 관리자 서비스 이름 — 항목은 `프로필 id` 별로 나뉜다.
const KEYRING_SERVICE: &str = "rhwp-desk-planner";

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(PROBE_TIMEOUT_MS))
        .build()
}

/// GET → JSON. (status, body) 실패 시 분류된 메시지.
fn get_json(url: &str, timeout_ms: u64) -> Result<Value, String> {
    let resp = agent()
        .get(url)
        .timeout(Duration::from_millis(timeout_ms))
        .call()
        .map_err(map_ureq_err)?;
    resp.into_json::<Value>()
        .map_err(|e| format!("JSON 응답 파싱 실패: {e}"))
}

fn map_ureq_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            planner::classify_error(Some(code), &body)
        }
        ureq::Error::Transport(t) => planner::classify_error(None, &t.to_string()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServer {
    pub base_url: String,
    pub api: String,
    pub models: Vec<String>,
}

/// 로컬 LLM 서버 자동 탐지 — 후보 포트에 태그 목록/모델 목록 API 를 짧게 찔러 본다.
pub fn probe_ports(host: &str, ports: &[u16]) -> Vec<LocalServer> {
    let mut found = Vec::new();
    for &port in ports {
        let base = format!("http://{host}:{port}");
        if let Ok(v) = get_json(&planner::tags_url(&base), PROBE_TIMEOUT_MS) {
            let models = planner::parse_model_list(&v);
            found.push(LocalServer {
                base_url: base,
                api: "tags".into(),
                models,
            });
            continue;
        }
        if let Ok(v) = get_json(&planner::models_url(&base), PROBE_TIMEOUT_MS) {
            let models = planner::parse_model_list(&v);
            found.push(LocalServer {
                base_url: base,
                api: "openai".into(),
                models,
            });
        }
    }
    found
}

/// 모델 목록 조회 (수동 엔드포인트 등록 폼용).
pub fn list_models(base_url: &str, api_key: Option<&str>) -> Result<Vec<String>, String> {
    let mut req = agent()
        .get(&planner::models_url(base_url))
        .timeout(Duration::from_millis(5_000));
    if let Some(k) = api_key.filter(|k| !k.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {k}"));
    }
    let v: Value = req
        .call()
        .map_err(map_ureq_err)?
        .into_json()
        .map_err(|e| format!("JSON 응답 파싱 실패: {e}"))?;
    Ok(planner::parse_model_list(&v))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatOutcome {
    pub latency_ms: u64,
    pub parsed: planner::ParsedChat,
    pub response: Value,
}

/// chat/completions 1회 왕복.
pub fn chat(
    base_url: &str,
    model: &str,
    api_key: Option<&str>,
    messages: &Value,
    tools: Option<&Value>,
    timeout_ms: Option<u64>,
) -> Result<ChatOutcome, String> {
    let body = planner::build_chat_request(model, messages, tools);
    let mut req = agent()
        .post(&planner::chat_url(base_url))
        .timeout(Duration::from_millis(timeout_ms.unwrap_or(CHAT_TIMEOUT_MS)))
        .set("Content-Type", "application/json");
    if let Some(k) = api_key.filter(|k| !k.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {k}"));
    }
    let start = Instant::now();
    let resp: Value = req
        .send_json(body)
        .map_err(map_ureq_err)?
        .into_json()
        .map_err(|e| format!("JSON 응답 파싱 실패: {e}"))?;
    let latency_ms = start.elapsed().as_millis() as u64;
    let parsed = planner::parse_chat_response(&resp)?;
    Ok(ChatOutcome {
        latency_ms,
        parsed,
        response: resp,
    })
}

// ── 자격 증명 관리자 ────────────────────────────────────────────

fn keyring_entry(profile_id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, profile_id)
        .map_err(|e| format!("자격 증명 항목 열기 실패: {e}"))
}

/// API 키 저장 — 실패하면 Err. 호출측(UI)은 "이 세션에만 유지" 폴백을 안내한다.
pub fn secret_set(profile_id: &str, key: &str) -> Result<(), String> {
    keyring_entry(profile_id)?
        .set_password(key)
        .map_err(|e| format!("자격 증명 저장 실패: {e}"))
}

pub fn secret_exists(profile_id: &str) -> bool {
    keyring_entry(profile_id)
        .and_then(|e| e.get_password().map_err(|e| e.to_string()))
        .is_ok()
}

pub fn secret_get(profile_id: &str) -> Option<String> {
    keyring_entry(profile_id)
        .ok()
        .and_then(|e| e.get_password().ok())
}

pub fn secret_delete(profile_id: &str) -> Result<(), String> {
    match keyring_entry(profile_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("자격 증명 삭제 실패: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// 미리 준비한 HTTP 응답 1건을 돌려주는 가짜 서버.
    fn fake_server(status: u16, body: &str) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = body.to_string();
        let h = std::thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                // 요청 전문(헤더 + Content-Length 본문)을 다 읽은 뒤 응답한다 —
                // 안 읽고 닫으면 Windows 가 연결을 리셋한다(os error 10053).
                let mut req = Vec::new();
                let mut buf = [0u8; 4096];
                let (mut header_end, mut want) = (None, 0usize);
                while let Ok(n) = sock.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    req.extend_from_slice(&buf[..n]);
                    if header_end.is_none() {
                        if let Some(pos) = req.windows(4).position(|w| w == b"\r\n\r\n") {
                            header_end = Some(pos + 4);
                            let head = String::from_utf8_lossy(&req[..pos]).to_lowercase();
                            want = head
                                .lines()
                                .find_map(|l| l.strip_prefix("content-length:"))
                                .and_then(|v| v.trim().parse().ok())
                                .unwrap_or(0);
                        }
                    }
                    if let Some(he) = header_end {
                        if req.len() >= he + want {
                            break;
                        }
                    }
                }
                let resp = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes());
            }
        });
        (format!("http://127.0.0.1:{}", addr.port()), h)
    }

    #[test]
    fn 가짜_태그_서버를_탐지한다() {
        let (base, h) = fake_server(200, r#"{"models":[{"name":"로컬-모델-7b"}]}"#);
        let port: u16 = base.rsplit(':').next().unwrap().parse().unwrap();
        let found = probe_ports("127.0.0.1", &[port]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].api, "tags");
        assert_eq!(found[0].models, vec!["로컬-모델-7b"]);
        h.join().unwrap();
    }

    #[test]
    fn 죽은_포트는_조용히_건너뛴다() {
        // 방금 닫힌 포트를 얻는다
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        drop(l);
        assert!(probe_ports("127.0.0.1", &[port]).is_empty());
    }

    #[test]
    fn chat_왕복과_파싱() {
        let (base, h) = fake_server(
            200,
            r#"{"choices":[{"message":{"role":"assistant","content":"pong"}}]}"#,
        );
        let msgs = serde_json::json!([{"role":"user","content":"ping"}]);
        let out = chat(&base, "테스트-모델", None, &msgs, None, Some(3000)).unwrap();
        assert_eq!(out.parsed.kind, "text");
        assert_eq!(out.parsed.content.as_deref(), Some("pong"));
        h.join().unwrap();
    }

    #[test]
    fn 인증_실패는_분류된_메시지로() {
        let (base, h) = fake_server(401, r#"{"error":"unauthorized"}"#);
        let msgs = serde_json::json!([{"role":"user","content":"ping"}]);
        let err = chat(&base, "m", Some("wrong-key"), &msgs, None, Some(3000)).unwrap_err();
        assert!(err.contains("인증 실패"), "{err}");
        h.join().unwrap();
    }

    #[test]
    fn 자격_증명_저장_조회_삭제_왕복() {
        // 실제 Windows 자격 증명 관리자를 쓴다 — 테스트 전용 항목으로 격리.
        let id = format!("test-{}-{}", std::process::id(), crate::journal::now_ms());
        if secret_set(&id, "비밀키-123").is_err() {
            // 헤드리스/권한 제약 환경이면 저장 자체가 불가 — 폴백 경로가 UI 에 있으므로 통과.
            eprintln!("자격 증명 관리자 사용 불가 — 건너뜀");
            return;
        }
        assert!(secret_exists(&id));
        assert_eq!(secret_get(&id).as_deref(), Some("비밀키-123"));
        secret_delete(&id).unwrap();
        assert!(!secret_exists(&id));
        // 두 번 지워도 오류가 아니어야 한다
        secret_delete(&id).unwrap();
    }
}
