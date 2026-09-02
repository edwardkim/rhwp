//! 본문 통계. 문서를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, one_file, page_texts, print_json, read_file, EXIT_OK, EXIT_RUNTIME,
};
use serde_json::json;

fn pages_of(path: &str) -> Result<Vec<String>, i32> {
    let data = read_file(path).map_err(|m| {
        eprintln!("오류: {m}");
        EXIT_RUNTIME
    })?;
    let core = load_core(&data).map_err(|fail| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
        EXIT_RUNTIME
    })?;
    page_texts(&core).map_err(|m| {
        eprintln!("오류: {m}");
        EXIT_RUNTIME
    })
}

pub fn run_hangul_ratio(args: &[String]) -> i32 {
    let usage = "rhwp-agent hangul-ratio <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let mut hangul = 0u64;
    let mut total = 0u64;
    for t in &pages {
        for ch in t.chars() {
            if ch.is_whitespace() {
                continue;
            }
            total += 1;
            if ('가'..='힣').contains(&ch)
                || ('ㄱ'..='ㅎ').contains(&ch)
                || ('ㅏ'..='ㅣ').contains(&ch)
            {
                hangul += 1;
            }
        }
    }
    let ratio = if total == 0 {
        0.0
    } else {
        hangul as f64 / total as f64
    };
    let payload = json!({
        "source": opts.path,
        "hangulChars": hangul,
        "letterChars": total,
        "ratio": ratio,
    });
    if opts.json {
        print_json(&envelope("hangul-ratio", payload, &[]));
    } else {
        crate::outln!("{ratio:.4}");
    }
    EXIT_OK
}

pub fn run_ascii_ratio(args: &[String]) -> i32 {
    let usage = "rhwp-agent ascii-ratio <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let mut ascii = 0u64;
    let mut total = 0u64;
    for t in &pages {
        for ch in t.chars() {
            if ch.is_whitespace() {
                continue;
            }
            total += 1;
            if ch.is_ascii() {
                ascii += 1;
            }
        }
    }
    let ratio = if total == 0 {
        0.0
    } else {
        ascii as f64 / total as f64
    };
    let payload = json!({
        "source": opts.path,
        "asciiChars": ascii,
        "letterChars": total,
        "ratio": ratio,
    });
    if opts.json {
        print_json(&envelope("ascii-ratio", payload, &[]));
    } else {
        crate::outln!("{ratio:.4}");
    }
    EXIT_OK
}

pub fn run_line_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent line-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let n: u64 = pages.iter().map(|t| t.lines().count() as u64).sum();
    let payload = json!({ "source": opts.path, "lineCount": n, "pageCount": pages.len() });
    if opts.json {
        print_json(&envelope("line-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_unique_chars(args: &[String]) -> i32 {
    let usage = "rhwp-agent unique-chars <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let mut set = std::collections::BTreeSet::new();
    for t in &pages {
        for ch in t.chars() {
            set.insert(ch);
        }
    }
    let payload = json!({ "source": opts.path, "uniqueCount": set.len() });
    if opts.json {
        print_json(&envelope("unique-chars", payload, &[]));
    } else {
        crate::outln!("{}", set.len());
    }
    EXIT_OK
}

pub fn run_section_count(args: &[String]) -> i32 {
    let usage = "rhwp-agent section-count <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let data = match read_file(&opts.path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!(
                "오류: 문서를 열 수 없습니다 - {}: {}",
                opts.path, fail.message
            );
            return EXIT_RUNTIME;
        }
    };
    let n = core.document().sections.len();
    let payload = json!({ "source": opts.path, "sectionCount": n });
    if opts.json {
        print_json(&envelope("section-count", payload, &[]));
    } else {
        crate::outln!("{n}");
    }
    EXIT_OK
}

pub fn run_longest_page(args: &[String]) -> i32 {
    let usage = "rhwp-agent longest-page <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let (idx, chars) = pages
        .iter()
        .enumerate()
        .map(|(i, t)| (i, t.chars().count()))
        .max_by_key(|(_, n)| *n)
        .unwrap_or((0, 0));
    let payload =
        json!({ "source": opts.path, "page": idx, "chars": chars, "pageCount": pages.len() });
    if opts.json {
        print_json(&envelope("longest-page", payload, &[]));
    } else {
        crate::outln!("{idx}\t{chars}");
    }
    EXIT_OK
}

pub fn run_shortest_page(args: &[String]) -> i32 {
    let usage = "rhwp-agent shortest-page <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let (idx, chars) = pages
        .iter()
        .enumerate()
        .map(|(i, t)| (i, t.chars().count()))
        .min_by_key(|(_, n)| *n)
        .unwrap_or((0, 0));
    let payload =
        json!({ "source": opts.path, "page": idx, "chars": chars, "pageCount": pages.len() });
    if opts.json {
        print_json(&envelope("shortest-page", payload, &[]));
    } else {
        crate::outln!("{idx}\t{chars}");
    }
    EXIT_OK
}

pub fn run_text_hash(args: &[String]) -> i32 {
    let usage = "rhwp-agent text-hash <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let pages = match pages_of(&opts.path) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let hash = crate::envelope::text_hash(&pages);
    let payload = json!({ "source": opts.path, "textHash": hash, "pageCount": pages.len() });
    if opts.json {
        print_json(&envelope("text-hash", payload, &[]));
    } else {
        crate::outln!("{hash}");
    }
    EXIT_OK
}
