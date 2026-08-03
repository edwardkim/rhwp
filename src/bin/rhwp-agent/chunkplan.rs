//! [#3918] `chunk-plan` — LLM 컨텍스트 예산 분할 계획.
//!
//! 큰 문서를 한 번에 못 싣는 에이전트는 지금 "일단 digest 를 불러 보고, 넘치면
//! 쪼개서 다시" 식으로 시행착오한다. 이 명령은 쪽별 문자 수만으로 **실행 전에**
//! 연속 쪽 구간 계획을 준다. 실행 수단은 기존 명령이다: `rhwp digest <파일>
//! --pages a..b`. 봉투에는 문서 텍스트가 한 글자도 실리지 않는다(숫자만) —
//! 계획 단계는 안전한 봉투로 두는 것이 요점이다.

use crate::envelope::{
    envelope, load_core, page_texts, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent chunk-plan <파일> --max-chars <N> [--json]";

    let mut json_mode = false;
    let mut max_chars: Option<u64> = None;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--max-chars" => {
                match args.get(i + 1).and_then(|v| v.parse::<u64>().ok()) {
                    Some(n) if n >= 1 => max_chars = Some(n),
                    _ => {
                        eprintln!("오류: --max-chars 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            positional => {
                if file.is_some() {
                    eprintln!("오류: 파일은 하나만 지정할 수 있습니다 - {positional}");
                    return EXIT_USAGE;
                }
                file = Some(positional.to_string());
                i += 1;
            }
        }
    }

    let Some(path) = file else {
        eprintln!("오류: 대상 파일을 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let Some(budget) = max_chars else {
        eprintln!("오류: --max-chars <N> 은 필수입니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };

    let data = match read_file(&path) {
        Ok(d) => d,
        Err(message) => {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    };
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
            return EXIT_RUNTIME;
        }
    };
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(message) => {
            eprintln!("오류: {path}: {message}");
            return EXIT_RUNTIME;
        }
    };
    let page_chars: Vec<u64> = pages.iter().map(|p| p.chars().count() as u64).collect();
    let total_chars: u64 = page_chars.iter().sum();

    // 탐욕 묶기: 연속 쪽을 예산까지 채운다. 예산보다 큰 단일 쪽은 제 구간이 되고
    // oversize 로 표시한다 — 침묵 상한 금지, 소비자가 그 쪽만 다른 전략을 쓰게 한다.
    struct Chunk {
        from: usize, // 1-기반 (digest --pages 관례)
        to: usize,
        chars: u64,
        oversize: bool,
    }
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut current: Option<Chunk> = None;
    for (idx, chars) in page_chars.iter().enumerate() {
        let page = idx + 1;
        if *chars > budget {
            if let Some(chunk) = current.take() {
                chunks.push(chunk);
            }
            chunks.push(Chunk {
                from: page,
                to: page,
                chars: *chars,
                oversize: true,
            });
            continue;
        }
        match current.as_mut() {
            Some(chunk) if chunk.chars + *chars <= budget => {
                chunk.to = page;
                chunk.chars += *chars;
            }
            _ => {
                if let Some(chunk) = current.take() {
                    chunks.push(chunk);
                }
                current = Some(Chunk {
                    from: page,
                    to: page,
                    chars: *chars,
                    oversize: false,
                });
            }
        }
    }
    if let Some(chunk) = current.take() {
        chunks.push(chunk);
    }

    let oversize_count = chunks.iter().filter(|c| c.oversize).count();

    if json_mode {
        let items: Vec<Value> = chunks
            .iter()
            .enumerate()
            .map(|(index, c)| {
                json!({
                    "index": index,
                    "pageFrom": c.from,
                    "pageTo": c.to,
                    "chars": c.chars,
                    "oversize": c.oversize,
                    "run": format!("rhwp digest {path} --pages {}..{} --json", c.from, c.to),
                })
            })
            .collect();
        let payload = json!({
            "source": path,
            "pageCount": page_chars.len(),
            "totalChars": total_chars,
            "maxChars": budget,
            "chunkCount": chunks.len(),
            "oversizeCount": oversize_count,
            "chunks": items,
        });
        // 숫자·경로·실행 힌트뿐 — 문서 본문이 실리지 않는 안전한 봉투다.
        print_json(&envelope("chunk-plan", payload, &[]));
    } else {
        crate::outln!("rhwp-agent chunk-plan — {path}");
        crate::outln!(
            "  {}쪽, 총 {}자, 구간당 {}자 예산",
            page_chars.len(),
            total_chars,
            budget
        );
        for (index, c) in chunks.iter().enumerate() {
            crate::outln!(
                "  구간 {index}: {}..{}쪽, {}자{}",
                c.from,
                c.to,
                c.chars,
                if c.oversize {
                    " (예산 초과 단일 쪽)"
                } else {
                    ""
                }
            );
        }
        if oversize_count > 0 {
            crate::outln!("주의: 예산보다 큰 쪽 {oversize_count}개 — 그 쪽은 --max-chars 를 키우거나 본문 추출로 따로 다루세요.");
        }
    }
    EXIT_OK
}
