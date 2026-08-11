//! [#4543] 앵커 로그 — 5년 축(투명성 로그) 코어.
//!
//! ## 무엇을 잡나 — T7 (역사 전체 재작성)
//!
//! 해시 체인+서명까지도, 키 소유자 본인이 뿌리부터 전부 재발급·재서명하면
//! 못 잡는다(합류 설계서 T7, git 강제 푸시와 같은 한계). 방어는 **외부 시점
//! 고정**뿐이다: append-only 로그에 캡슐 해시를 등재하고, 로그의 머클
//! 체크포인트를 외부에 공표한다. 공표 이후의 재작성은 체크포인트와 충돌한다.
//!
//! ## 형식 — 줄 단위 해시 체인 (ndjson)
//!
//! ```json
//! {"seq":0,"kind":"anchorLog","capsuleSha256":"…","prevEntryHash":null,"loggedAt":"…"}
//! {"seq":1,"kind":"anchorLog","capsuleSha256":"…","prevEntryHash":"<직전 줄 바이트 SHA-256>","loggedAt":"…"}
//! ```
//!
//! `prevEntryHash` 는 직전 **줄 원문 바이트**의 SHA-256 — 캡슐 체인(파일
//! 바이트)·서명(파일 바이트)과 같은 대상 규약이라 세 체계가 어긋나지 않는다.
//! 중간 줄이 1바이트라도 바뀌면 다음 줄의 기록 해시가 폭로한다.
//!
//! ## 도구의 경계 (설계서 §2.2 그대로)
//!
//! 이 모듈은 등재·자기 무결·머클 루트·경로 증명까지다. 체크포인트를 **어디에
//! 공표하는가**(사내 게시·git 커밋·타임스탬프 기관)는 운영 절차이며, 봉투의
//! 어떤 필드도 공표를 증명하는 척하지 않는다.

use sha2::{Digest, Sha256};

/// 로그 항목의 `kind`.
pub const LOG_KIND: &str = "anchorLog";
/// 체크포인트의 `kind`.
pub const CHECKPOINT_KIND: &str = "anchorCheckpoint";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// 파싱된 로그 — 항목과 줄 바이트 해시(머클 잎).
pub struct AnchorLog {
    pub entries: Vec<serde_json::Value>,
    /// 각 줄 원문 바이트의 SHA-256 — 체인 검증과 머클 잎에 함께 쓴다.
    pub line_hashes: Vec<String>,
}

/// 로그를 읽고 **자기 무결**(줄 해시 체인·seq 연번·kind)을 판정한다.
///
/// 빈 파일(또는 부재)은 항목 0개의 유효한 로그다 — 첫 등재 전 상태.
pub fn load(path: &str) -> Result<AnchorLog, String> {
    load_kind(path, LOG_KIND)
}

/// [#4553] kind 매개변수판 — 9년 정산 원장이 같은 체인 검증 경로를 동형
/// 재사용한다(신규 형식 발명 0, 설계서 §2.3).
pub fn load_kind(path: &str, kind: &str) -> Result<AnchorLog, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("로그를 읽을 수 없습니다 - {path}: {e}")),
    };
    let mut entries = Vec::new();
    let mut line_hashes: Vec<String> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("{path}:{}: 로그 줄 파싱 실패 - {e}", i + 1))?;
        if entry["kind"] != kind {
            return Err(format!("{path}:{}: kind 가 {kind} 가 아님", i + 1));
        }
        if entry["seq"].as_u64() != Some(entries.len() as u64) {
            return Err(format!(
                "{path}:{}: seq 연번 위반 (기대 {}, 실제 {})",
                i + 1,
                entries.len(),
                entry["seq"]
            ));
        }
        let expected_prev = match line_hashes.last() {
            Some(h) => serde_json::json!(h),
            None => serde_json::Value::Null,
        };
        if entry["prevEntryHash"] != expected_prev {
            return Err(format!(
                "{path}:{}: prevEntryHash 불일치 — 이전 줄이 사후에 바뀌었다 (append-only 위반)",
                i + 1
            ));
        }
        line_hashes.push(sha256_hex(line.as_bytes()));
        entries.push(entry);
    }
    Ok(AnchorLog {
        entries,
        line_hashes,
    })
}

/// 새 등재 줄(JSON 한 줄)을 만든다 — 호출자가 파일 끝에 append 한다.
pub fn make_entry_line(log: &AnchorLog, capsule_sha256: &str, logged_at: &str) -> String {
    let entry = serde_json::json!({
        "seq": log.entries.len(),
        "kind": LOG_KIND,
        "capsuleSha256": capsule_sha256,
        "prevEntryHash": log.line_hashes.last().cloned(),
        // 주장 필드 — 시점 증명은 체크포인트의 외부 공표가 만든다.
        "loggedAt": logged_at,
    });
    entry.to_string()
}

/// 머클 루트 — 잎은 줄 바이트 해시, 홀수 층은 마지막을 복제한다.
pub fn merkle_root(leaves: &[String]) -> Option<String> {
    if leaves.is_empty() {
        return None;
    }
    let mut level: Vec<String> = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = pair.get(1).unwrap_or(&pair[0]);
            next.push(sha256_hex(format!("{}{}", pair[0], right).as_bytes()));
        }
        level = next;
    }
    level.pop()
}

/// seq 잎의 머클 경로 — (형제 해시, 형제가 왼쪽인가) 목록. 검증자는 잎에서
/// 루트까지 재계산한다.
pub fn merkle_path(leaves: &[String], seq: usize) -> Vec<(String, bool)> {
    let mut path = Vec::new();
    let mut level: Vec<String> = leaves.to_vec();
    let mut idx = seq;
    while level.len() > 1 {
        let left_child = idx.is_multiple_of(2);
        let sibling = if left_child {
            level.get(idx + 1).unwrap_or(&level[idx]).clone()
        } else {
            level[idx - 1].clone()
        };
        path.push((sibling, !left_child));
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = pair.get(1).unwrap_or(&pair[0]);
            next.push(sha256_hex(format!("{}{}", pair[0], right).as_bytes()));
        }
        level = next;
        idx /= 2;
    }
    path
}

/// 머클 경로로 잎→루트 재계산.
pub fn merkle_verify(leaf: &str, path: &[(String, bool)], root: &str) -> bool {
    let mut acc = leaf.to_string();
    for (sibling, sibling_is_left) in path {
        acc = if *sibling_is_left {
            sha256_hex(format!("{sibling}{acc}").as_bytes())
        } else {
            sha256_hex(format!("{acc}{sibling}").as_bytes())
        };
    }
    acc == root
}
