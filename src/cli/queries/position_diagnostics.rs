//! Stream-anchor and caret-coordinate diagnostic query adapters.

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// 문단의 **스트림 좌표**를 찍는다 — 컨트롤 종류·`char_offsets`·컨트롤의 글자 위치.
///
/// 편집 액션이 개체 앵커를 옮기는지 볼 때 쓴다(계획서 §4.24 가 이걸로 나왔다). `ir-sweep`
/// 은 필드 나열이라 "컨트롤과 공백의 순서가 바뀌었다" 같은 **구조** 변화를 읽기 어렵다 —
/// 이 보기는 문단 하나를 스트림 순서 그대로 편다. 여태 임시 테스트 파일로 하던 일이다.
pub(crate) fn dump_anchors(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-anchors <파일…> [--all]");
        return EXIT_USAGE;
    }
    let all = args.iter().any(|a| a == "--all");
    for path in args.iter().filter(|a| !a.starts_with('-')) {
        let doc = match std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| rhwp::parser::parse_document(&b).map_err(|e| e.to_string()))
        {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{path}: {e}");
                return EXIT_RUNTIME;
            }
        };
        println!("== {path}");
        for (si, sec) in doc.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                if !all && para.controls.is_empty() {
                    continue;
                }
                let kinds: Vec<String> = para
                    .controls
                    .iter()
                    .map(|c| match c {
                        rhwp::model::control::Control::SectionDef(_) => "secd".to_string(),
                        rhwp::model::control::Control::ColumnDef(_) => "cold".to_string(),
                        rhwp::model::control::Control::Table(_) => "표".to_string(),
                        rhwp::model::control::Control::Picture(_) => "그림".to_string(),
                        rhwp::model::control::Control::Shape(s) => s.shape_name().to_string(),
                        other => format!("{other:?}")
                            .split(['(', ' '])
                            .next()
                            .unwrap_or("?")
                            .to_string(),
                    })
                    .collect();
                println!(
                    "s{si} p{pi}: chars={} text={:?}",
                    para.char_count, para.text
                );
                println!("   char_offsets={:?}", para.char_offsets);
                println!("   controls={kinds:?}");
                println!("   ctrl_positions={:?}", para.control_text_positions());
            }
        }
    }
    EXIT_OK
}

/// 문단 전 오프셋의 **캐럿 사각형**(x·y·height)을 찍는다 — studio 가 딛는 `getCursorRect`.
///
/// 줌·DPI 무관한 **문서 좌표**의 캐럿 기하다(한글의 화면 캐럿과 달리 안정적이다). 캐럿 높이는
/// 폰트에 달리므로 폰트별 표본으로 돌려 크기를 견준다. `--json` 은 한 줄 계약 봉투.
pub(crate) fn dump_carets(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-carets <파일> [--json] [-s <구역>] [-p <문단>]");
        return EXIT_USAGE;
    }
    let path = &args[0];
    let json_mode = args.iter().any(|a| a == "--json");
    let mut sec_filter: Option<usize> = None;
    let mut para_filter: Option<usize> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--section" if i + 1 < args.len() => {
                sec_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "-p" | "--para" if i + 1 < args.len() => {
                para_filter = args[i + 1].parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("읽기 실패: {path} — {e}");
            return EXIT_RUNTIME;
        }
    };
    let structure = match rhwp::parser::parse_document(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파싱 실패: {path} — {e}");
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => {
            let _ = &e;
            eprintln!("문서 로드 실패: {path}");
            return EXIT_RUNTIME;
        }
    };

    let mut rows: Vec<serde_json::Value> = Vec::new();
    for (si, sec) in structure.sections.iter().enumerate() {
        if sec_filter.is_some_and(|f| f != si) {
            continue;
        }
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            if para_filter.is_some_and(|f| f != pi) {
                continue;
            }
            // 문단 끝까지(포함) 캐럿을 둔다 — 마지막은 문단 부호 앞자리다.
            let last = para.char_count as usize;
            for off in 0..=last {
                let Ok(raw) = doc.get_cursor_rect_native(si, pi, off) else {
                    continue;
                };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
                    continue;
                };
                rows.push(serde_json::json!({
                    "section": si,
                    "para": pi,
                    "offset": off,
                    "pageIndex": v.get("pageIndex"),
                    "x": v.get("x"),
                    "y": v.get("y"),
                    "height": v.get("height"),
                }));
            }
        }
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "file": path,
            "count": rows.len(),
            "carets": rows,
        });
        println!("{}", provenance::marked(envelope, "dump-carets"));
        return EXIT_OK;
    }
    for r in &rows {
        println!(
            "s{}p{} off{:>3}: page {} x={:>7} y={:>7} h={}",
            r["section"], r["para"], r["offset"], r["pageIndex"], r["x"], r["y"], r["height"]
        );
    }
    println!("\n=== 캐럿 {} 개 ===", rows.len());
    EXIT_OK
}
