//! `bench` — HWP/HWPX 단계별 처리 성능 계측.
//!
//! 단계: parse(바이트→IR) · layout(=load−parse) · render(전 페이지 SVG) ·
//! serialize(serialize_hwpx). 워밍업 1회 후 N회 반복하여 median(ms) 으로 보고한다.
//!
//! 주의(정직성): 절대 수치는 측정 머신·빌드(release/debug)에 의존한다. 동일 머신·
//! 동일 빌드에서의 **상대 비교·재현용 지표**로 해석한다(한컴 등 외부 기준 아님).
//!
//! 사용법:
//!   rhwp bench <파일...> [-n 반복수] [--tsv 출력.tsv]
//!   rhwp bench --batch <폴더> [-n 반복수] [--tsv 출력.tsv]

use std::fs;
use std::time::Instant;

use crate::document_core::DocumentCore;
use crate::parser::parse_document;
use crate::serializer::serialize_hwpx;

struct Row {
    name: String,
    size_kb: f64,
    pages: u32,
    out_kb: f64,
    parse_ms: f64,
    layout_ms: f64,
    render_ms: f64,
    serialize_ms: f64,
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn ms_since(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

pub fn run(args: &[String]) -> i32 {
    // 종료 코드 계약(mydocs/manual/cli_commands.md): 파일 처리 실패는 이미 1 로 끝냈지만
    // 인자 누락만 0 이었다 — 오타로 대상이 비면 "0개 측정 성공" 이 되어 버린다.
    let mut files: Vec<String> = Vec::new();
    let mut batch: Option<String> = None;
    let mut iters = 3usize;
    let mut tsv: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --batch 뒤에 폴더 경로가 필요합니다.");
                    return super::EXIT_USAGE;
                };
                if value.starts_with('-') {
                    eprintln!("오류: --batch 뒤에 폴더 경로가 필요합니다 - {value}");
                    return super::EXIT_USAGE;
                }
                batch = Some(value.clone());
            }
            "-n" | "--iters" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --iters 뒤에 반복 횟수가 필요합니다.");
                    return super::EXIT_USAGE;
                };
                match value.parse::<usize>() {
                    Ok(parsed) => iters = parsed,
                    Err(_) => {
                        eprintln!("오류: --iters 뒤에는 0 이상의 정수가 필요합니다 - {value}");
                        return super::EXIT_USAGE;
                    }
                }
            }
            "--tsv" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --tsv 뒤에 출력 경로가 필요합니다.");
                    return super::EXIT_USAGE;
                };
                if value.starts_with('-') {
                    eprintln!("오류: --tsv 뒤에 출력 경로가 필요합니다 - {value}");
                    return super::EXIT_USAGE;
                }
                tsv = Some(value.clone());
            }
            // [#3884 G2] 미지 플래그를 파일 이름으로 접지 않는다 — `--json` 을 주면
            // "그런 파일이 없다"로 실패하던 것이 대표 증상이다. 침묵 무시는 더 나쁘다:
            // 오타 난 옵션이 측정 조건을 조용히 바꾼 채 성공을 보고한다.
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!(
                    "사용법: rhwp bench <파일...> | --batch <폴더> [-n 반복수] [--tsv 출력.tsv]"
                );
                return super::EXIT_USAGE;
            }
            other => files.push(other.to_string()),
        }
        i += 1;
    }

    if let Some(dir) = &batch {
        if let Err(error) = collect_samples(std::path::Path::new(dir), &mut files) {
            eprintln!("오류: --batch 폴더를 읽을 수 없습니다 - {dir}: {error}");
            return super::EXIT_RUNTIME;
        }
        files.sort();
    }
    if files.is_empty() {
        eprintln!("사용법: rhwp bench <파일...> | --batch <폴더> [-n 반복수] [--tsv 출력.tsv]");
        return super::EXIT_USAGE;
    }
    if iters == 0 {
        iters = 1;
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut failures = 0usize;
    for f in &files {
        match bench_one(f, iters) {
            Ok(r) => rows.push(r),
            Err(e) => {
                eprintln!("  {f}: 실패 — {e}");
                failures += 1;
            }
        }
    }

    let mut tsv_written: Option<String> = None;
    if let Some(path) = tsv {
        if rows.is_empty() {
            // 측정 성공 0건이면 산출물을 만들지 않는다(redact 의 "탐지 0건 = 파일
            // 미생성"과 같은 원칙) — 빈 표를 받은 쪽이 "측정했고 결과가 없다"로 오독한다.
            eprintln!("TSV 생략: 측정 성공이 0건입니다 - {path}");
        } else {
            match write_tsv(&path, &rows) {
                Ok(()) => tsv_written = Some(path),
                Err(e) => {
                    eprintln!("TSV 쓰기 실패: {e}");
                    failures += 1;
                }
            }
        }
    }

    // [#3884 G1] 실패가 하나라도 있으면 stdout 은 0바이트다
    // (`capabilities.jsonContract.failure`). 종전엔 성공 행이 하나 있으면 배너+표를
    // stdout 에 얹은 채 exit 1 을 냈고, `bench <문서> --json` 이 `--json` 을 파일
    // 이름으로 접던 시절의 518 B 누수가 그 혼합 실패였다. 사람용 전말은 stderr.
    if failures > 0 {
        eprintln!("\n{failures}개 파일 처리 실패 — 종료 코드 1");
        if let Some(path) = tsv_written {
            eprintln!("TSV: {path}");
        }
        return super::EXIT_RUNTIME;
    }

    if !rows.is_empty() {
        println!("=== bench: 단계별 처리 성능 (median of {iters}회, 워밍업 1회) ===");
        println!("주의: 절대 수치는 측정 머신·빌드 의존. 동일 환경 상대·재현 지표로 해석.");
        print_table(&rows);
    }
    if let Some(path) = tsv_written {
        println!("\nTSV: {path}");
    }
    super::EXIT_OK
}

fn collect_samples(dir: &std::path::Path, acc: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            collect_samples(&p, acc)?;
        } else if p.extension().is_some_and(|x| {
            let x = x.to_string_lossy().to_ascii_lowercase();
            x == "hwp" || x == "hwpx"
        }) {
            acc.push(p.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn bench_one(path: &str, iters: usize) -> Result<Row, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let size_kb = data.len() as f64 / 1024.0;

    // 워밍업 (페이지 캐시·할당자 정상화) — 측정에서 제외.
    let _ = DocumentCore::from_bytes(&data).map_err(|e| format!("{e:?}"))?;

    let mut parse_v = Vec::with_capacity(iters);
    let mut load_v = Vec::with_capacity(iters);
    let mut render_v = Vec::with_capacity(iters);
    let mut ser_v = Vec::with_capacity(iters);
    let mut pages = 0u32;
    let mut out_kb = 0.0;

    for _ in 0..iters {
        // parse: 바이트 → Document IR (격리)
        let t = Instant::now();
        let _doc = parse_document(&data).map_err(|e| format!("{e:?}"))?;
        parse_v.push(ms_since(t));

        // load: parse + layout (studio "열기" 비용). layout ≈ load − parse.
        let t = Instant::now();
        let core = DocumentCore::from_bytes(&data).map_err(|e| format!("{e:?}"))?;
        load_v.push(ms_since(t));
        pages = core.page_count();

        // render: 전 페이지 SVG 렌더 (페이지 렌더 실패는 파일 처리 실패로 전파)
        let t = Instant::now();
        for p in 0..pages {
            core.render_page_svg_native(p)
                .map_err(|e| format!("{e:?}"))?;
        }
        render_v.push(ms_since(t));

        // serialize: Document → HWPX 바이트 (studio "저장" 비용)
        let t = Instant::now();
        let bytes = serialize_hwpx(core.document()).map_err(|e| format!("{e:?}"))?;
        ser_v.push(ms_since(t));
        out_kb = bytes.len() as f64 / 1024.0;
    }

    let parse_ms = median(parse_v);
    let load_ms = median(load_v);
    let layout_ms = (load_ms - parse_ms).max(0.0);

    Ok(Row {
        name: path.to_string(),
        size_kb,
        pages,
        out_kb,
        parse_ms,
        layout_ms,
        render_ms: median(render_v),
        serialize_ms: median(ser_v),
    })
}

fn short_name(p: &str, max: usize) -> String {
    let base = std::path::Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string());
    if base.chars().count() <= max {
        base
    } else {
        let tail: String = base
            .chars()
            .rev()
            .take(max - 1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("…{tail}")
    }
}

fn print_table(rows: &[Row]) {
    println!(
        "\n{:<40} {:>8} {:>5} {:>9} {:>9} {:>10} {:>11} {:>10}",
        "파일", "크기KB", "쪽", "parse", "layout", "render", "serialize", "total"
    );
    println!("{}", "-".repeat(112));
    for r in rows {
        let total = r.parse_ms + r.layout_ms + r.render_ms + r.serialize_ms;
        println!(
            "{:<40} {:>8.1} {:>5} {:>7.1}ms {:>7.1}ms {:>8.1}ms {:>9.1}ms {:>8.1}ms",
            short_name(&r.name, 40),
            r.size_kb,
            r.pages,
            r.parse_ms,
            r.layout_ms,
            r.render_ms,
            r.serialize_ms,
            total
        );
    }
    if rows.len() > 1 {
        let sum_total: f64 = rows
            .iter()
            .map(|r| r.parse_ms + r.layout_ms + r.render_ms + r.serialize_ms)
            .sum();
        let pages: u32 = rows.iter().map(|r| r.pages).sum();
        println!("{}", "-".repeat(112));
        println!(
            "합계 {}개 파일, {}쪽, total {:.1}ms ({:.1}ms/쪽)",
            rows.len(),
            pages,
            sum_total,
            if pages > 0 {
                sum_total / pages as f64
            } else {
                0.0
            }
        );
    }
}

fn write_tsv(path: &str, rows: &[Row]) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut w = fs::File::create(path)?;
    writeln!(
        w,
        "file\tsize_kb\tpages\tout_kb\tparse_ms\tlayout_ms\trender_ms\tserialize_ms\ttotal_ms"
    )?;
    for r in rows {
        let total = r.parse_ms + r.layout_ms + r.render_ms + r.serialize_ms;
        writeln!(
            w,
            "{}\t{:.1}\t{}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            r.name,
            r.size_kb,
            r.pages,
            r.out_kb,
            r.parse_ms,
            r.layout_ms,
            r.render_ms,
            r.serialize_ms,
            total
        )?;
    }
    Ok(())
}
