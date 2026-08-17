//! 작업 저널 — journal-first NDJSON (설계 문서 §7).
//!
//! 모든 도구 호출은 카드로 그려지기 **전에** 여기 한 줄로 적힌다.
//! UI 의 작업 카드 스트림은 이 파일의 1:1 렌더일 뿐이며, 저널에 없는 것을
//! 지어내 보여주지 않는다. 형식은 한 줄 = JSON 객체 하나(NDJSON)다.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

/// 도구 호출 1건의 기록 — 카드 1장과 1:1 대응한다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: String,
    /// UNIX epoch 밀리초. 표시 포맷은 UI 몫이다.
    pub ts_ms: u64,
    /// 실행한 엔진 바이너리 경로.
    pub engine: String,
    /// 첫 위치 인자 (예: "info", "export-svg").
    pub command: String,
    /// 엔진에 넘긴 전체 인자열.
    pub args: Vec<String>,
    /// 프로세스 종료 코드. 시그널 종료 등으로 없을 수 있다.
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    /// stdout 이 JSON 봉투로 파싱되면 그 값.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<serde_json::Value>,
    /// JSON 이 아닐 때의 stdout 꼬리.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_tail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_tail: Option<String>,
    /// 호출을 일으킨 UI 표면 (palette / verify / viewer / export / info ...).
    pub origin: String,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 프로세스 내에서 유일한 항목 id 를 만든다.
pub fn new_id(ts_ms: u64) -> String {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{ts_ms:x}-{n:04x}")
}

pub fn journal_path(dir: &Path) -> PathBuf {
    dir.join("journal.ndjson")
}

/// 항목 1건을 NDJSON 으로 덧붙인다. 디렉토리가 없으면 만든다.
pub fn append(dir: &Path, entry: &JournalEntry) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let line = serde_json::to_string(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path(dir))?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    f.flush()
}

/// 마지막 `limit` 건을 오래된 것부터 순서대로 돌려준다.
/// 저널이 아직 없으면 빈 목록, 깨진 줄은 건너뛴다(부분 손상 내성).
pub fn read_tail(dir: &Path, limit: usize) -> std::io::Result<Vec<JournalEntry>> {
    let path = journal_path(dir);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut entries: Vec<JournalEntry> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    if entries.len() > limit {
        entries.drain(..entries.len() - limit);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("rhwp-desk-test").join(format!(
            "{tag}-{}-{}",
            std::process::id(),
            now_ms()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn entry(command: &str, exit: i32) -> JournalEntry {
        let ts = now_ms();
        JournalEntry {
            id: new_id(ts),
            ts_ms: ts,
            engine: "rhwp.exe".into(),
            command: command.into(),
            args: vec![command.into(), "--json".into()],
            exit_code: Some(exit),
            duration_ms: 42,
            envelope: Some(serde_json::json!({"schemaVersion":"1.0"})),
            stdout_tail: None,
            stderr_tail: None,
            origin: "palette".into(),
        }
    }

    #[test]
    fn 기록_후_그대로_읽힌다() {
        let dir = temp_dir("roundtrip");
        let a = entry("info", 0);
        let b = entry("inspect", 3);
        append(&dir, &a).unwrap();
        append(&dir, &b).unwrap();
        let got = read_tail(&dir, 10).unwrap();
        assert_eq!(got, vec![a, b]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn limit_은_최신_쪽을_남긴다() {
        let dir = temp_dir("limit");
        for i in 0..5 {
            append(&dir, &entry(&format!("cmd{i}"), 0)).unwrap();
        }
        let got = read_tail(&dir, 2).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].command, "cmd3");
        assert_eq!(got[1].command, "cmd4");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 저널_없음은_빈_목록이고_깨진_줄은_건너뛴다() {
        let dir = temp_dir("missing");
        assert!(read_tail(&dir, 10).unwrap().is_empty());

        append(&dir, &entry("ok", 0)).unwrap();
        // 손상 줄을 중간에 끼워도 나머지는 살아 있어야 한다.
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(journal_path(&dir))
                .unwrap();
            writeln!(f, "{{broken json").unwrap();
        }
        append(&dir, &entry("after", 1)).unwrap();
        let got = read_tail(&dir, 10).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[1].command, "after");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
