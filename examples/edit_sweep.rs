//! 편집-스윕 상설 도구 (#2355) — 전 샘플 편집 전후 페이지 수 baseline 대조.
//!
//! PR #2314 검증에서 일회성 하네스로 수행한 편집-스윕(전 샘플에
//! `insert_text_native(0,0,0,"X")` 적용 → 전후 페이지 수 대조)을 상설화한다.
//! 편집 경로 PR(vpos 재계산·pagination·undo 계열)의 자가 검증용:
//! devel 과 작업 브랜치에서 각각 스윕을 뜨고 --compare 로 변동을
//! 공통/해소/신규로 분류하면 "가짜 페이지 변동" 증감이 정량으로 드러난다.
//!
//! 사용:
//!     # 스윕 → TSV (경로·전후 쪽수·변동·상태)
//!     cargo run --release --example edit_sweep -- samples -o out/sweep/devel.tsv
//!     # 편집 종류 선택(기본 insert1 = para0 앞 1자 삽입)
//!     cargo run --release --example edit_sweep -- samples -o out/sweep/a.tsv --edit insert1
//!     # 두 스윕 대조 → 공통/해소/신규 분류 md 리포트
//!     cargo run --release --example edit_sweep -- --compare out/sweep/devel.tsv out/sweep/branch.tsv -o out/sweep/report.md
//!
//! 대상: 지정 디렉터리 재귀의 .hwp/.hwpx/.hml, 기본 <2MB 캡(--max-kb, 0=무제한).
//! 파일 단위 panic 은 catch_unwind 로 격리해 스윕 전체를 죽이지 않고 status 에 기록한다.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use rhwp::wasm_api::HwpDocument;

const EXTS: [&str; 3] = ["hwp", "hwpx", "hml"];

/// 편집 종류 — 확장 시 이 목록과 apply_edit 에 추가.
const EDIT_KINDS: [&str; 1] = ["insert1"];

fn apply_edit(doc: &mut HwpDocument, kind: &str) -> Result<(), String> {
    match kind {
        // para0 앞 1자 삽입 — #2314 검증과 동일 (편집 문단 높이가 변하지 않는 최소 편집)
        "insert1" => doc
            .insert_text_native(0, 0, 0, "X")
            .map(|_| ())
            .map_err(|e| format!("{e:?}")),
        other => Err(format!("알 수 없는 편집 종류: {other}")),
    }
}

/// 한 파일 스윕: 파싱 → 편집 전 쪽수 → 편집 → 편집 후 쪽수. panic 격리.
fn sweep_one(path: &Path, edit: &str) -> (Option<u32>, Option<u32>, String) {
    let run = panic::catch_unwind(AssertUnwindSafe(|| {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => return (None, None, format!("read_error:{e}")),
        };
        let mut doc = match HwpDocument::from_bytes(&bytes) {
            Ok(d) => d,
            Err(e) => return (None, None, format!("parse_error:{e:?}")),
        };
        let before = doc.page_count();
        if let Err(e) = apply_edit(&mut doc, edit) {
            return (Some(before), None, format!("edit_error:{e}"));
        }
        let after = doc.page_count();
        (Some(before), Some(after), "ok".to_string())
    }));
    match run {
        Ok(r) => r,
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "?".to_string());
            (None, None, format!("panic:{}", sanitize(&msg)))
        }
    }
}

/// TSV 안전화 — 탭·개행을 공백으로.
fn sanitize(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn collect_files(root: &Path, max_kb: u64) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && !p.is_symlink() {
                stack.push(p);
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| EXTS.contains(&x.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
                && (max_kb == 0
                    || e.metadata()
                        .map(|m| m.len() <= max_kb * 1024)
                        .unwrap_or(false))
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn run_sweep(root: &Path, out: &Path, edit: &str, max_kb: u64) -> std::io::Result<i32> {
    let files = collect_files(root, max_kb);
    eprintln!(
        "[edit-sweep] {}개 파일 (<{}KB), 편집={edit}",
        files.len(),
        if max_kb == 0 { u64::MAX } else { max_kb }
    );
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // 파일 단위 panic 을 status 로 기록하므로 기본 panic 메시지 출력은 침묵시킨다.
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut f = std::fs::File::create(out)?;
    writeln!(f, "file\tbefore\tafter\tchanged\tstatus")?;
    let (mut changed, mut errs) = (0u32, 0u32);
    for (i, p) in files.iter().enumerate() {
        let rel = p.strip_prefix(root.parent().unwrap_or(root)).unwrap_or(p);
        let (before, after, status) = sweep_one(p, edit);
        let ch = matches!((before, after), (Some(b), Some(a)) if b != a);
        if ch {
            changed += 1;
        }
        if status != "ok" {
            errs += 1;
        }
        writeln!(
            f,
            "{}\t{}\t{}\t{}\t{}",
            rel.display(),
            before.map(|v| v.to_string()).unwrap_or_default(),
            after.map(|v| v.to_string()).unwrap_or_default(),
            u8::from(ch),
            status
        )?;
        if (i + 1) % 50 == 0 {
            eprintln!(
                "[edit-sweep] {}/{} (변동 {changed}, 오류 {errs})",
                i + 1,
                files.len()
            );
        }
    }
    panic::set_hook(prev_hook);
    eprintln!(
        "[edit-sweep] 완료: {}건 중 변동 {changed}건, 오류 {errs}건 → {}",
        files.len(),
        out.display()
    );
    Ok(0)
}

#[derive(Clone)]
struct Row {
    before: Option<u32>,
    after: Option<u32>,
    changed: bool,
    status: String,
}

fn load_tsv(path: &Path) -> std::io::Result<BTreeMap<String, Row>> {
    let text = std::fs::read_to_string(path)?;
    let mut map = BTreeMap::new();
    for line in text.lines().skip(1) {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 5 {
            continue;
        }
        map.insert(
            c[0].to_string(),
            Row {
                before: c[1].parse().ok(),
                after: c[2].parse().ok(),
                changed: c[3] == "1",
                status: c[4].to_string(),
            },
        );
    }
    Ok(map)
}

fn fmt_pages(r: &Row) -> String {
    match (r.before, r.after) {
        (Some(b), Some(a)) => format!("{b}→{a}"),
        _ => r.status.clone(),
    }
}

fn run_compare(a_path: &Path, b_path: &Path, out: Option<&Path>) -> std::io::Result<i32> {
    let (a, b) = (load_tsv(a_path)?, load_tsv(b_path)?);
    let mut common = Vec::new(); // 양쪽 공통 변동 (기존 동작)
    let mut resolved = Vec::new(); // A 에서만 변동 (B 가 해소)
    let mut new_ = Vec::new(); // B 에서만 변동 (신규)
    let mut status_diff = Vec::new(); // ok↔오류 전이 (커버리지 변화)
    for (file, ra) in &a {
        let Some(rb) = b.get(file) else {
            status_diff.push(format!("| {file} | {} | (B 에 없음) |", fmt_pages(ra)));
            continue;
        };
        if ra.status != rb.status {
            status_diff.push(format!(
                "| {file} | {} | {} |",
                fmt_pages(ra),
                fmt_pages(rb)
            ));
            continue;
        }
        match (ra.changed, rb.changed) {
            (true, true) => common.push(format!(
                "| {file} | {} | {} |",
                fmt_pages(ra),
                fmt_pages(rb)
            )),
            (true, false) => resolved.push(format!(
                "| {file} | {} | {} |",
                fmt_pages(ra),
                fmt_pages(rb)
            )),
            (false, true) => new_.push(format!(
                "| {file} | {} | {} |",
                fmt_pages(ra),
                fmt_pages(rb)
            )),
            (false, false) => {}
        }
    }
    for file in b.keys().filter(|k| !a.contains_key(*k)) {
        status_diff.push(format!(
            "| {file} | (A 에 없음) | {} |",
            fmt_pages(&b[file])
        ));
    }
    let (na, nb) = (
        a.values().filter(|r| r.changed).count(),
        b.values().filter(|r| r.changed).count(),
    );
    let mut md = vec![
        format!(
            "### 편집-스윕 대조 — A={} ({}건) vs B={} ({}건)",
            a_path.display(),
            a.len(),
            b_path.display(),
            b.len()
        ),
        String::new(),
        format!("| | A | B |\n|---|---|---|\n| 변동 | **{na}건** | **{nb}건** |"),
        format!(
            "| 분류 | 건수 |\n|---|---|\n| 양쪽 공통 변동 (기존 동작) | {} |\n| A 에서만 변동 (B 가 해소) | {} |\n| B 에서만 변동 (신규) | {} |\n| 상태 전이/커버리지 차이 | {} |",
            common.len(),
            resolved.len(),
            new_.len(),
            status_diff.len()
        ),
    ];
    for (title, rows) in [
        ("공통 변동", &common),
        ("A 에서만 변동 (B 가 해소)", &resolved),
        ("B 에서만 변동 (신규)", &new_),
        ("상태 전이/커버리지 차이", &status_diff),
    ] {
        if !rows.is_empty() {
            md.push(format!("\n#### {title}\n\n| 파일 | A | B |\n|---|---|---|"));
            md.extend(rows.iter().cloned());
        }
    }
    let text = md.join("\n");
    println!("{text}");
    if let Some(o) = out {
        if let Some(parent) = o.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(o, format!("{text}\n"))?;
        eprintln!("[edit-sweep] 리포트 → {}", o.display());
    }
    // 신규 변동 존재 = 회귀 후보 → exit 1 (게이트 사용)
    Ok(i32::from(!new_.is_empty()))
}

fn usage() -> ! {
    eprintln!(
        "사용:\n  edit_sweep <샘플디렉터리> -o <out.tsv> [--edit {}] [--max-kb 2048]\n  edit_sweep --compare <A.tsv> <B.tsv> [-o report.md]",
        EDIT_KINDS.join("|")
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut positional = Vec::new();
    let mut out = None;
    let mut edit = "insert1".to_string();
    let mut max_kb = 2048u64;
    let mut compare = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" | "--out" => out = it.next().map(PathBuf::from),
            "--edit" => edit = it.next().cloned().unwrap_or_else(|| usage()),
            "--max-kb" => {
                max_kb = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage())
            }
            "--compare" => compare = true,
            "-h" | "--help" => usage(),
            _ => positional.push(PathBuf::from(a)),
        }
    }
    let code = if compare {
        if positional.len() != 2 {
            usage();
        }
        run_compare(&positional[0], &positional[1], out.as_deref())
    } else {
        if positional.len() != 1 || out.is_none() {
            usage();
        }
        if !EDIT_KINDS.contains(&edit.as_str()) {
            usage();
        }
        run_sweep(&positional[0], out.as_ref().unwrap(), &edit, max_kb)
    };
    match code {
        Ok(c) => std::process::exit(c),
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(2)
        }
    }
}
