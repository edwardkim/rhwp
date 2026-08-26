//! [#4864] `context-cost` — 두 경로의 컨텍스트 비용과 복원율을 같은 문서에서 잰다.
//!
//! 이 저장소의 에이전트 표면은 "문서를 구조화해 주는 도구가 필요하다"는 전제 위에
//! 서 있는데, 그 전제를 뒷받침하는 숫자가 없었다. 원리로만 답하면("바이너리라서")
//! 반박도 검증도 할 수 없다. 이 명령은 그 자리를 **재현 가능한 수치**로 채운다.
//!
//! 두 경로를 잰다.
//!
//! - **그대로 싣기** — 파일 바이트를 텍스트로 디코딩해 모델에 넣는 경로. 범용 도구
//!   조합(파일 읽기 + 셸 텍스트 처리)이 바이너리 문서에 대해 할 수 있는 전부다.
//! - **문서-네이티브** — 파서를 거쳐 본문만 싣는 경로.
//!
//! # 정직 규율
//!
//! 1. **가장 유리한 대안도 같이 잰다.** UTF-8 만 재면 허수아비다. 인코딩을 바꿔 볼
//!    호출자를 상정해 UTF-16LE 복원율을 같은 봉투에 싣는다 — 한글 문서의 많은
//!    바이너리 포맷이 UTF-16LE 로 문자열을 담기 때문에 이쪽이 실제로 더 유리하다.
//! 2. **토큰이 아니라 문자를 센다.** 토크나이저는 모델마다 다르고 이 저장소는 모델을
//!    부르지 않는다. 문자 수는 결정론적이고 제3자가 손으로 검증할 수 있다. 봉투의
//!    `unit` 이 이 한계를 스스로 밝힌다.
//! 3. **봉투에 문서 본문이 한 글자도 실리지 않는다.** 숫자와 호출자가 지정한 경로뿐이라
//!    계측 결과를 그대로 로그·이슈에 붙여도 문서가 새지 않는다.

use crate::envelope::{
    envelope, load_core, page_texts, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

/// 복원율 계산에 쓸 최소 줄 길이. 짧은 줄("1.", "가.")은 바이너리 어디에나 우연히
/// 나타나 복원율을 부풀린다 — 우연 일치를 걸러야 숫자가 정직해진다.
const MIN_LINE_CHARS: usize = 4;

struct Measured {
    source: String,
    bytes: u64,
    utf8_chars: u64,
    utf16le_chars: u64,
    native_chars: u64,
    recovery_utf8: f64,
    recovery_utf16le: f64,
    sampled_chars: u64,
}

/// 파일 바이트를 UTF-16LE 로 손실 디코딩한다. 홀수 바이트 꼬리는 버린다 —
/// 디코딩 실패가 아니라 그 경로가 볼 수 있는 것의 한계다.
fn decode_utf16le(data: &[u8]) -> String {
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

/// 본문 줄 중 원문 그대로 디코딩 안에 있는 것의 문자 비율(0~100).
///
/// 문자 수가 아니라 **줄 단위**로 세는 이유: 한두 글자가 우연히 맞는 것은 복원이
/// 아니다. 사람이 읽을 수 있는 길이의 줄이 통째로 나와야 "읽혔다"고 할 수 있다.
fn recovery_percent(lines: &[&str], decoded: &str, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let hit: u64 = lines
        .iter()
        .filter(|line| decoded.contains(**line))
        .map(|line| line.chars().count() as u64)
        .sum();
    (hit as f64) * 100.0 / (total as f64)
}

fn measure(path: &str) -> Result<Measured, String> {
    let data = read_file(path)?;
    let core =
        load_core(&data).map_err(|fail| format!("문서를 열 수 없습니다: {}", fail.message))?;
    let pages = page_texts(&core)?;

    let utf8 = String::from_utf8_lossy(&data);
    let utf16 = decode_utf16le(&data);

    let native_chars: u64 = pages.iter().map(|p| p.chars().count() as u64).sum();
    let joined = pages.join("\n");
    let lines: Vec<&str> = joined
        .lines()
        .map(str::trim)
        .filter(|l| l.chars().count() >= MIN_LINE_CHARS)
        .collect();
    let sampled_chars: u64 = lines.iter().map(|l| l.chars().count() as u64).sum();

    Ok(Measured {
        source: path.to_string(),
        bytes: data.len() as u64,
        utf8_chars: utf8.chars().count() as u64,
        utf16le_chars: utf16.chars().count() as u64,
        native_chars,
        recovery_utf8: recovery_percent(&lines, &utf8, sampled_chars),
        recovery_utf16le: recovery_percent(&lines, &utf16, sampled_chars),
        sampled_chars,
    })
}

/// 배수는 소수 한 자리로 고정한다 — 부동소수 꼬리가 봉투마다 달라지면 결정론이 깨진다.
fn ratio(numerator: u64, denominator: u64) -> Value {
    if denominator == 0 {
        // 본문이 0자면 배수는 정의되지 않는다. 0 이나 무한대로 뭉개면 "쌌다"는
        // 정반대 해석이 나오므로 null 로 두고 소비자가 갈라 보게 한다.
        return Value::Null;
    }
    json!(((numerator as f64 / denominator as f64) * 10.0).round() / 10.0)
}

fn round1(v: f64) -> Value {
    json!((v * 10.0).round() / 10.0)
}

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent context-cost <파일...> [--json]";

    let mut json_mode = false;
    let mut files: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            positional => files.push(positional.to_string()),
        }
    }
    if files.is_empty() {
        eprintln!("오류: 대상 파일을 하나 이상 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let mut measured = Vec::with_capacity(files.len());
    for path in &files {
        match measure(path) {
            Ok(m) => measured.push(m),
            Err(message) => {
                // 한 파일 실패로 나머지 계측을 버리지 않는다. 다만 부분 성공을
                // 성공으로 보고하지도 않는다 — 종료 코드로 실행 오류를 남긴다.
                eprintln!("오류: {path}: {message}");
                return EXIT_RUNTIME;
            }
        }
    }

    let total_bytes: u64 = measured.iter().map(|m| m.bytes).sum();
    let total_utf8: u64 = measured.iter().map(|m| m.utf8_chars).sum();
    let total_native: u64 = measured.iter().map(|m| m.native_chars).sum();

    if json_mode {
        let items: Vec<Value> = measured
            .iter()
            .map(|m| {
                json!({
                    "source": m.source,
                    "bytes": m.bytes,
                    "rawChars": { "utf8": m.utf8_chars, "utf16le": m.utf16le_chars },
                    "nativeChars": m.native_chars,
                    "charRatio": ratio(m.utf8_chars, m.native_chars),
                    "recoveryPercent": {
                        "utf8": round1(m.recovery_utf8),
                        "utf16le": round1(m.recovery_utf16le),
                    },
                    "sampledChars": m.sampled_chars,
                })
            })
            .collect();
        let payload = json!({
            "unit": "chars",
            // 계측의 한계를 봉투가 스스로 밝힌다 — 읽는 쪽이 토큰 수로 오해하지
            // 않게 하는 것이 이 필드의 유일한 목적이다.
            "unitNote": "토크나이저는 모델마다 다르므로 토큰이 아니라 문자를 센다. 배수는 모델 간 이식 가능한 하한으로 읽으라.",
            "recoveryNote": format!("본문 줄({MIN_LINE_CHARS}자 이상)이 원문 그대로 디코딩 안에 있는 비율. 짧은 줄은 우연 일치를 만들어 제외한다."),
            "fileCount": measured.len(),
            "files": items,
            "summary": {
                "bytes": total_bytes,
                "rawChars": { "utf8": total_utf8 },
                "nativeChars": total_native,
                "charRatio": ratio(total_utf8, total_native),
            },
        });
        // 숫자와 호출자가 지정한 경로뿐 — 문서 본문이 실리지 않는 안전한 봉투다.
        print_json(&envelope("context-cost", payload, &[]));
    } else {
        crate::outln!("rhwp-agent context-cost — 파일 {}개", measured.len());
        crate::outln!("  단위는 문자(토큰 아님). 배수는 모델 간 이식 가능한 하한이다.");
        for m in &measured {
            crate::outln!("  {}", m.source);
            crate::outln!(
                "    바이트 {} · 그대로 싣기(UTF-8) {}자 · 문서 본문 {}자",
                m.bytes,
                m.utf8_chars,
                m.native_chars
            );
            let ratio_text = if m.native_chars == 0 {
                "정의 불가(본문 0자)".to_string()
            } else {
                format!("{:.1}배", m.utf8_chars as f64 / m.native_chars as f64)
            };
            crate::outln!(
                "    문자 배수 {ratio_text} · 복원율 UTF-8 {:.1}% · UTF-16LE {:.1}%",
                m.recovery_utf8,
                m.recovery_utf16le
            );
        }
    }
    EXIT_OK
}
