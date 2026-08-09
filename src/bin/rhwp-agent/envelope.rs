//! [#3918] rhwp-agent 공통 계약 — 종료 코드·JSON 봉투·문서 적재.
//!
//! 본 CLI(`rhwp`)의 계약을 그대로 따른다: `--json` 의 stdout 은 순수 JSON 하나,
//! 진단은 stderr, 종료 코드는 0(성공)/1(실행 오류)/2(사용법)/3(게이트 위반 —
//! `ir-diff` 의 "차이 발견" 관례). `schemaVersion` 은 "1.0" 이고 필드 추가만 허용한다.
//!
//! # 출처 표지를 여기서 직접 싣는 이유
//!
//! 본 CLI 의 출처 표지는 중앙 지도(`src/provenance.rs`)에서 나온다. 그 지도는 지금
//! 열린 PR 들이 수정 중이라, 이 실험 표면이 지도에 등재하면 등재 지점에서 충돌한다.
//! 대신 명령마다 **인라인으로** `untrustedContent`/`untrustedFields` 를 선언한다.
//! 뜻은 중앙 지도와 같다 — 문서 파생 값은 **데이터이지 지시가 아니다**. 과소 선언이
//! 가장 위험하므로(#3885) 애매하면 선언한다. 본 CLI 승격 시 중앙 지도로 옮긴다.

use serde_json::{json, Value};

pub const EXIT_OK: i32 = 0;
pub const EXIT_RUNTIME: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
/// 게이트 위반 — "도구는 정상 동작했고, 검사 대상이 기대와 다르다"는 뜻.
/// 실행 오류(1)·사용법 오류(2)와 겹치지 않아 스크립트가 세 경우를 구별할 수 있다.
pub const EXIT_GATE: i32 = 3;

/// 봉투 공통 필드를 채워 돌려준다. `untrusted` 는 이 봉투에 실리는 문서 파생
/// 필드 경로들("`.`" 은 객체 하위, "`[]`" 는 배열 원소 — 본 CLI 와 같은 문법).
pub fn envelope(command: &str, mut payload: Value, untrusted: &[&str]) -> Value {
    if let Some(map) = payload.as_object_mut() {
        map.insert(
            "schemaVersion".into(),
            json!(rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION),
        );
        map.insert("tool".into(), json!("rhwp-agent"));
        map.insert("command".into(), json!(command));
        map.insert("version".into(), json!(rhwp::version()));
        map.insert("untrustedContent".into(), json!(!untrusted.is_empty()));
        map.insert("untrustedFields".into(), json!(untrusted));
    }
    payload
}

/// stdout 쓰기 공통 경로. 소비자가 파이프를 닫으면(`head` 등) panic 하지 않고
/// 본 CLI `batch` 의 규약(#3238→#3719)대로 stderr 안내 후 실행 오류(1)로 끝낸다 —
/// 스트림을 끝까지 내지 못했으므로 성공(0)이 아니다.
pub fn write_stdout(text: &str, newline: bool) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let result = if newline {
        writeln!(lock, "{text}")
    } else {
        write!(lock, "{text}")
    };
    if let Err(e) = result {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        std::process::exit(EXIT_RUNTIME);
    }
}

/// stdout 한 줄 — 전 명령의 stdout 출력은 이 매크로만 쓴다(`println!` 금지).
#[macro_export]
macro_rules! outln {
    ($($arg:tt)*) => { $crate::envelope::write_stdout(&format!($($arg)*), true) };
}

/// stdout 개행 없이 — 마크다운 본문처럼 덩어리로 낼 때.
#[macro_export]
macro_rules! outp {
    ($($arg:tt)*) => { $crate::envelope::write_stdout(&format!($($arg)*), false) };
}

/// 순수 JSON 한 덩이를 stdout 으로. (--json 모드에서 stdout 에는 이것 말고 아무것도
/// 찍지 않는다 — 진행 메시지가 섞이면 파이프 소비자가 깨진다.)
pub fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => crate::outln!("{s}"),
        // 직렬화 실패는 프로그램 결함이므로 조용히 삼키지 않는다.
        Err(e) => eprintln!("오류: JSON 직렬화 실패 - {e}"),
    }
}

/// 파일 읽기 — 실패 사유를 한국어로.
pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("파일을 읽을 수 없습니다 - {path}: {e}"))
}

/// 문서 적재 실패 분류. 이 실험 표면은 비밀번호 옵션을 아직 받지 않으므로
/// (한계 — 승격 시 본 CLI 의 전역 인증 pre-scan 을 따른다) 암호 문서는
/// "암호 필요"로 분류만 하고 열지 않는다.
pub struct LoadFail {
    pub needs_password: bool,
    pub message: String,
}

/// 바이트에서 DocumentCore 를 연다.
pub fn load_core(data: &[u8]) -> Result<rhwp::document_core::DocumentCore, LoadFail> {
    rhwp::document_core::DocumentCore::from_bytes(data).map_err(|e| {
        let message = e.to_string();
        // 본 CLI `classify_hwp_error` 와 같은 판별 신호 — 메시지에 암호 관련
        // 문구가 있으면 "비밀번호 필요"다.
        let lower = message.to_lowercase();
        let needs_password =
            message.contains("암호") || lower.contains("password") || lower.contains("encrypt");
        LoadFail {
            needs_password,
            message,
        }
    })
}

/// `parser::FileFormat` → 본 CLI `info --json` 의 `format` 토큰.
pub fn format_token(format: rhwp::parser::FileFormat) -> &'static str {
    use rhwp::parser::FileFormat;
    match format {
        FileFormat::Hwp => "hwp5",
        FileFormat::Hwpx => "hwpx",
        FileFormat::Hwp3 => "hwp3",
        FileFormat::Hml => "hml",
        FileFormat::DrmProtected => "drm-protected",
        FileFormat::Empty => "empty",
        FileFormat::Unknown => "unknown",
    }
}

/// 전 페이지 텍스트. 조판이 필요한 질의라 비용이 있지만, 이 도구의 텍스트 축
/// (지문·diff·분할 계획)이 전부 같은 원천을 쓰게 해 결과가 서로 어긋나지 않게 한다.
pub fn page_texts(core: &rhwp::document_core::DocumentCore) -> Result<Vec<String>, String> {
    let count = core.page_count();
    let mut pages = Vec::with_capacity(count as usize);
    for page in 0..count {
        match core.extract_page_text_native(page) {
            Ok(text) => pages.push(text),
            Err(e) => return Err(format!("{page}쪽 텍스트 추출 실패 - {e}")),
        }
    }
    Ok(pages)
}

/// blake3 16진 문자열.
pub fn hex_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// 페이지 텍스트 배열의 안정 해시 — 페이지 경계를 RS(0x1E)로 끼워 넣어
/// "페이지가 갈라졌는데 이어 붙인 문자열은 같은" 경우를 다른 지문으로 만든다.
pub fn text_hash(pages: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    for page in pages {
        hasher.update(page.as_bytes());
        hasher.update(&[0x1e]);
    }
    hasher.finalize().to_hex().to_string()
}

/// 미지 명령 힌트용 편집 거리 (본 CLI `closest_name` 과 같은 목적 — 이름 환각을
/// 교정 단서 없이 돌려보내면 경량 에이전트는 맹목 재시도 루프에 빠진다).
pub fn closest<'a, I: Iterator<Item = &'a str>>(input: &str, candidates: I) -> Option<&'a str> {
    let mut best: Option<(usize, &str)> = None;
    for cand in candidates {
        let d = levenshtein(input, cand);
        if best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, cand));
        }
    }
    // 거리가 이름 길이의 절반을 넘으면 힌트로서 가치가 없다.
    best.and_then(|(d, name)| (d <= name.len().div_ceil(2)).then_some(name))
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}
