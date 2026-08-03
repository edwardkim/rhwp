//! [#3918] `diff-text` — 페이지 텍스트 줄 단위 비교.
//!
//! `ir-diff` 는 IR 구조, `render-diff` 는 픽셀을 본다. 사람 증빙(PR 전/후)과
//! 에이전트의 "무엇이 바뀌었나" 질문이 원하는 것은 **텍스트 수준**이다. 두 문서의
//! 전 쪽 텍스트를 줄로 펴서 LCS 로 비교하고, 유니파이드(사람)·JSON(기계)으로 낸다.
//!
//! # 규모 안전판
//!
//! LCS DP 는 O(N×M)이다. 공통 접두·접미를 먼저 걷어낸 뒤 남은 중간이 예산
//! (4,000,000 셀)을 넘으면 정밀 diff 대신 "중간 전체 교체" 한 덩이로 낸다 —
//! 침묵 상한이 아니라 `coarse: true` 로 표시한다(정밀 diff 가 아니라는 뜻).

use crate::envelope::{
    envelope, load_core, page_texts, print_json, read_file, EXIT_GATE, EXIT_OK, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::{json, Value};

/// 한 줄의 diff 연산.
#[derive(Clone, Copy, PartialEq)]
enum Op {
    Equal,
    Del,
    Ins,
}

pub struct Hunk {
    /// 1-기반 시작 줄 (각각 A·B 기준).
    pub a_start: usize,
    pub b_start: usize,
    /// (연산 문자 ' '·'-'·'+', 줄 텍스트)
    pub lines: Vec<(char, String)>,
}

pub struct DiffResult {
    pub lines_a: usize,
    pub lines_b: usize,
    pub added: usize,
    pub removed: usize,
    pub coarse: bool,
    pub hunks: Vec<Hunk>,
}

impl DiffResult {
    pub fn identical(&self) -> bool {
        self.added == 0 && self.removed == 0
    }
}

/// 쪽 텍스트 배열 → 줄 배열. 쪽 경계는 줄 경계로 취급한다.
pub fn lines_of(pages: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for page in pages {
        for line in page.lines() {
            out.push(line.to_string());
        }
    }
    out
}

/// LCS 기반 줄 diff. `context` 는 헝크 앞뒤에 붙일 동일 줄 수.
pub fn diff_lines(a: &[String], b: &[String], context: usize) -> DiffResult {
    // ① 공통 접두·접미 걷어내기 — 실무 문서 diff 의 대부분을 여기서 끝낸다.
    let mut prefix = 0usize;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut suffix = 0usize;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let mid_a = &a[prefix..a.len() - suffix];
    let mid_b = &b[prefix..b.len() - suffix];

    // ② 연산열 만들기.
    const BUDGET: usize = 4_000_000;
    let (ops, coarse): (Vec<Op>, bool) = if mid_a.is_empty() && mid_b.is_empty() {
        (Vec::new(), false)
    } else if mid_a.len().saturating_mul(mid_b.len()) <= BUDGET {
        (lcs_ops(mid_a, mid_b), false)
    } else {
        // 예산 초과 — 중간 전체 교체로 강등하고 그 사실을 표시한다.
        let mut ops = vec![Op::Del; mid_a.len()];
        ops.extend(std::iter::repeat_n(Op::Ins, mid_b.len()));
        (ops, true)
    };

    // ③ 전체 연산열 = 접두 Equal + 중간 + 접미 Equal.
    let mut full: Vec<Op> = Vec::with_capacity(prefix + ops.len() + suffix);
    full.extend(std::iter::repeat_n(Op::Equal, prefix));
    full.extend(ops);
    full.extend(std::iter::repeat_n(Op::Equal, suffix));

    let added = full.iter().filter(|op| **op == Op::Ins).count();
    let removed = full.iter().filter(|op| **op == Op::Del).count();

    // ④ 헝크로 묶기 — 변경 줄에서 `context` 이내의 동일 줄은 같은 헝크에 싣는다.
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut ai = 0usize; // A 의 현재 줄 (0-기반)
    let mut bi = 0usize;
    let mut idx = 0usize;
    while idx < full.len() {
        if full[idx] == Op::Equal {
            ai += 1;
            bi += 1;
            idx += 1;
            continue;
        }
        // 변경 시작 — 앞 문맥을 붙인 시작점을 잡는다.
        let lead = context.min(ai).min(bi);
        let mut hunk = Hunk {
            a_start: ai - lead + 1,
            b_start: bi - lead + 1,
            lines: Vec::new(),
        };
        for back in (1..=lead).rev() {
            hunk.lines.push((' ', a[ai - back].clone()));
        }
        // 변경 구간 + 사이에 낀 짧은 동일 구간(≤ context×2)을 이어 담는다.
        let mut trailing = 0usize;
        loop {
            if idx >= full.len() {
                break;
            }
            match full[idx] {
                Op::Del => {
                    hunk.lines.push(('-', a[ai].clone()));
                    ai += 1;
                    trailing = 0;
                }
                Op::Ins => {
                    hunk.lines.push(('+', b[bi].clone()));
                    bi += 1;
                    trailing = 0;
                }
                Op::Equal => {
                    // 뒤 문맥은 context 줄까지만 담고, 그 안에 다음 변경이 또
                    // 나오면 같은 헝크로 이어진다.
                    let more_changes_soon = full[idx..]
                        .iter()
                        .take(context * 2 + 1)
                        .any(|op| *op != Op::Equal);
                    if trailing >= context && !more_changes_soon {
                        break;
                    }
                    hunk.lines.push((' ', a[ai].clone()));
                    ai += 1;
                    bi += 1;
                    trailing += 1;
                }
            }
            idx += 1;
        }
        hunks.push(hunk);
    }

    DiffResult {
        lines_a: a.len(),
        lines_b: b.len(),
        added,
        removed,
        coarse,
        hunks,
    }
}

/// 고전 LCS DP + 역추적. 호출 전에 예산 검사를 마쳤다.
fn lcs_ops(a: &[String], b: &[String]) -> Vec<Op> {
    let n = a.len();
    let m = b.len();
    // dp[i][j] = a[i..]·b[j..] 의 LCS 길이. u32 로 충분(예산상 줄 수 < 2^32).
    let mut dp = vec![0u32; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[at(i, j)] = if a[i] == b[j] {
                dp[at(i + 1, j + 1)] + 1
            } else {
                dp[at(i + 1, j)].max(dp[at(i, j + 1)])
            };
        }
    }
    let mut ops = Vec::with_capacity(n + m);
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push(Op::Equal);
            i += 1;
            j += 1;
        } else if dp[at(i + 1, j)] >= dp[at(i, j + 1)] {
            ops.push(Op::Del);
            i += 1;
        } else {
            ops.push(Op::Ins);
            j += 1;
        }
    }
    ops.extend(std::iter::repeat_n(Op::Del, n - i));
    ops.extend(std::iter::repeat_n(Op::Ins, m - j));
    ops
}

/// 문서 경로 → 줄 배열 (적재 오류는 (코드, 메시지)).
pub fn document_lines(path: &str) -> Result<Vec<String>, (i32, String)> {
    let data = read_file(path).map_err(|m| (EXIT_RUNTIME, m))?;
    let core = load_core(&data).map_err(|fail| {
        (
            EXIT_RUNTIME,
            format!("문서를 열 수 없습니다 - {path}: {}", fail.message),
        )
    })?;
    let pages = page_texts(&core).map_err(|m| (EXIT_RUNTIME, format!("{path}: {m}")))?;
    Ok(lines_of(&pages))
}

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp-agent diff-text <파일A> <파일B> [--json] [--context <N>] [--max-hunks <N>]";

    let mut json_mode = false;
    let mut context = 2usize;
    let mut max_hunks = 50usize;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--context" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) => context = n,
                    None => {
                        eprintln!("오류: --context 뒤에 0 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--max-hunks" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => max_hunks = n,
                    _ => {
                        eprintln!("오류: --max-hunks 뒤에 1 이상의 정수가 필요합니다.");
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
                files.push(positional.to_string());
                i += 1;
            }
        }
    }

    if files.len() != 2 {
        eprintln!("오류: 파일 두 개(A·B)를 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let lines_a = match document_lines(&files[0]) {
        Ok(v) => v,
        Err((code, message)) => {
            eprintln!("오류: {message}");
            return code;
        }
    };
    let lines_b = match document_lines(&files[1]) {
        Ok(v) => v,
        Err((code, message)) => {
            eprintln!("오류: {message}");
            return code;
        }
    };

    let result = diff_lines(&lines_a, &lines_b, context);
    let truncated_hunks = result.hunks.len() > max_hunks;

    if json_mode {
        let hunks: Vec<Value> = result
            .hunks
            .iter()
            .take(max_hunks)
            .map(|h| {
                json!({
                    "aStart": h.a_start,
                    "bStart": h.b_start,
                    "lines": h
                        .lines
                        .iter()
                        .map(|(op, text)| json!({"op": op.to_string(), "text": text}))
                        .collect::<Vec<_>>(),
                })
            })
            .collect();
        let payload = json!({
            "sourceA": files[0],
            "sourceB": files[1],
            "linesA": result.lines_a,
            "linesB": result.lines_b,
            "added": result.added,
            "removed": result.removed,
            "identical": result.identical(),
            "coarse": result.coarse,
            "hunkCount": result.hunks.len(),
            "truncatedHunks": truncated_hunks,
            "hunks": hunks,
        });
        // 헝크 줄 텍스트는 문서 본문 그 자체다.
        let untrusted: &[&str] = if result.hunks.is_empty() {
            &[]
        } else {
            &["hunks[].lines[].text"]
        };
        print_json(&envelope("diff-text", payload, untrusted));
    } else {
        crate::outln!("--- {}", files[0]);
        crate::outln!("+++ {}", files[1]);
        if result.identical() {
            crate::outln!("두 문서의 텍스트가 같습니다 ({}줄).", result.lines_a);
        } else {
            if result.coarse {
                crate::outln!("(주의: 규모 예산 초과 — 정밀 diff 대신 중간 전체 교체로 표시)");
            }
            for hunk in result.hunks.iter().take(max_hunks) {
                let a_len = hunk.lines.iter().filter(|(op, _)| *op != '+').count();
                let b_len = hunk.lines.iter().filter(|(op, _)| *op != '-').count();
                crate::outln!(
                    "@@ -{},{} +{},{} @@",
                    hunk.a_start,
                    a_len,
                    hunk.b_start,
                    b_len
                );
                for (op, text) in &hunk.lines {
                    crate::outln!("{op}{text}");
                }
            }
            if truncated_hunks {
                crate::outln!(
                    "(헝크 {}개 중 {}개만 표시 — --max-hunks 로 조절)",
                    result.hunks.len(),
                    max_hunks
                );
            }
            crate::outln!(
                "요약: +{} -{} (A {}줄 → B {}줄)",
                result.added,
                result.removed,
                result.lines_a,
                result.lines_b
            );
        }
    }

    if result.identical() {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}
