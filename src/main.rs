use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod atomic_file;

/// [#2707] CLI 종료 코드 계약 — 성공.
const EXIT_OK: i32 = 0;
/// [#2707] CLI 종료 코드 계약 — 런타임 실패(읽기·파싱·렌더·쓰기).
const EXIT_RUNTIME: i32 = 1;
/// [#2707] CLI 종료 코드 계약 — 사용법 오류(인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과).
///
/// 3(`--verify` IR 차이)·4(`--verify-pages` 페이지 수 불일치)는
/// `mydocs/manual/cli_commands.md` 에 이미 문서화된 계약이므로 상수화 대상에서 제외하고
/// 기존 `process::exit(3)`/`process::exit(4)` 호출부를 그대로 둔다.
const EXIT_USAGE: i32 = 2;

/// [#2707] 명령 함수가 돌려준 종료 코드를 프로세스 종료 코드로 전파한다.
///
/// 0이면 아무것도 하지 않아 `main` 이 정상 종료하고, 그 외에는 즉시 그 코드로 종료한다.
fn exit_with(exit_code: i32) {
    if exit_code != EXIT_OK {
        process::exit(exit_code);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => print_help(),
        Some("--version") | Some("-V") => println!("rhwp v{}", rhwp::version()),
        Some("export-svg") => exit_with(export_svg(&args[2..])),
        Some("export-render-tree") => exit_with(export_render_tree(&args[2..])),
        Some("export-structure") => exit_with(export_structure(&args[2..])),
        Some("export-png") => exit_with(export_png(&args[2..])),
        Some("export-pdf") => exit_with(export_pdf(&args[2..])),
        Some("export-text") => exit_with(export_text(&args[2..])),
        Some("export-markdown") => exit_with(export_markdown(&args[2..])),
        Some("export-tables") => exit_with(export_tables(&args[2..])),
        Some("export-hwpx") => exit_with(export_hwpx(&args[2..])),
        Some("export-hml") => export_hml(&args[2..]),
        Some("export-doclang") => exit_with(export_doclang(&args[2..])),
        Some("capabilities") => exit_with(show_capabilities(&args[2..])),
        Some("batch") => exit_with(run_batch(&args[2..])),
        Some("info") => exit_with(show_info(&args[2..])),
        Some("dump") => exit_with(dump_controls(&args[2..])),
        Some("dump-note-shape") => exit_with(dump_note_shape(&args[2..])),
        Some("dump-endnote-lines") => exit_with(dump_endnote_lines(&args[2..])),
        Some("dump-pages") => exit_with(dump_pages(&args[2..])),
        Some("diag") => exit_with(diag_document(&args[2..])),
        Some("search") => exit_with(search_document(&args[2..])),
        Some("convert") => exit_with(convert_hwp(&args[2..])),
        Some("build-from-ingest") => exit_with(build_from_ingest(&args[2..])),
        Some("hwp5-inventory") => rhwp::diagnostics::hwp5_inventory::run(&args[2..]),
        Some("hwp5-inventory-diff") => rhwp::diagnostics::hwp5_inventory_diff::run(&args[2..]),
        Some("hwp5-contract-analyze") => rhwp::diagnostics::hwp5_contract_analyze::run(&args[2..]),
        Some("hwp5-ctrl-data-trace") => rhwp::diagnostics::hwp5_ctrl_data_trace::run(&args[2..]),
        Some("hwp5-contract-probe") => rhwp::diagnostics::hwp5_contract_probe::run(&args[2..]),
        Some("hwp5-table-probe") => rhwp::diagnostics::hwp5_table_probe::run(&args[2..]),
        Some("hwp5-mel-personnel-probe") => {
            rhwp::diagnostics::hwp5_mel_personnel_probe::run(&args[2..])
        }
        Some("hwp5-borderfill-diagonal-probe") => {
            rhwp::diagnostics::hwp5_borderfill_diagonal_probe::run(&args[2..])
        }
        Some("hwp5-first-para-control-probe") => {
            rhwp::diagnostics::hwp5_first_para_control_probe::run(&args[2..])
        }
        Some("hwp5-anchor-trace") => rhwp::diagnostics::hwp5_anchor_trace::run(&args[2..]),
        Some("hwp5-cell-header-probe") => {
            rhwp::diagnostics::hwp5_cell_header_probe::run(&args[2..])
        }
        Some("dump-records") => exit_with(dump_raw_records(&args[2..])),
        Some("test-shape") => test_shape_roundtrip(&args[2..]),
        Some("test-caption") => test_caption(&args[2..]),
        Some("gen-table") => gen_table(&args[2..]),
        Some("gen-pua") => gen_pua_test(&args[2..]),
        Some("test-field") => test_field_roundtrip(&args[2..]),
        Some("ir-diff") => exit_with(ir_diff(&args[2..])),
        Some("hwpx-roundtrip") => rhwp::diagnostics::hwpx_roundtrip_batch::run(&args[2..]),
        Some("hwp5-roundtrip") => rhwp::diagnostics::hwp5_roundtrip_batch::run(&args[2..]),
        Some("render-diff") => rhwp::diagnostics::render_geom_diff::run(&args[2..]),
        Some("measure-width") => rhwp::diagnostics::text_width_probe::run(&args[2..]),
        Some("core-pages") => rhwp::diagnostics::core_pages_probe::run(&args[2..]),
        Some("bench") => rhwp::diagnostics::bench::run(&args[2..]),
        Some("thumbnail") => extract_thumbnail(&args[2..]),
        Some("fields") => exit_with(show_fields(&args[2..])),
        // [#2707] 알 수 없는 명령·명령 누락은 사용법 오류다. 표준 CLI 관례대로 stderr 로 안내하고
        // 종료 코드 2로 끝낸다(기존에는 stdout + 0이라 오타 낸 명령이 스크립트에서 성공으로 보였다).
        other => {
            match other {
                Some(command) => eprintln!("오류: 알 수 없는 명령입니다 - {}", command),
                None => eprintln!("오류: 명령을 지정해주세요."),
            }
            eprintln!("rhwp v{}", rhwp::version());
            eprintln!("사용법: rhwp <명령> [옵션]");
            eprintln!("'rhwp --help'로 자세한 사용법을 확인하세요.");
            process::exit(EXIT_USAGE);
        }
    }
}

/// [#3263] `capabilities --mcp` — MCP 도구 정의 생성.
///
/// MCP 서버 저자(및 함수 호출 클라이언트)가 도구 이름·설명·입력 JSON Schema·실행 배선을
/// 손으로 옮겨 적지 않게 한다. `--json` 계약을 가진 명령이 늘면
/// `capabilities_mcp_covers_every_json_command` 가 누락을 잡는다.
fn show_mcp_tools() -> i32 {
    /// 문서 경로 하나를 받는 도구의 표준 입력 스키마.
    fn path_schema(extra: serde_json::Value) -> serde_json::Value {
        let mut props = serde_json::json!({
            "path": { "type": "string", "description": "HWP/HWPX/HML 문서 경로" }
        });
        if let (Some(p), Some(e)) = (props.as_object_mut(), extra.as_object()) {
            for (k, v) in e {
                p.insert(k.clone(), v.clone());
            }
        }
        serde_json::json!({
            "type": "object",
            "properties": props,
            "required": ["path"],
        })
    }

    fn tool(
        name: &str,
        description: &str,
        input_schema: serde_json::Value,
        command: &str,
        args_template: serde_json::Value,
        output_fields: &[&str],
    ) -> serde_json::Value {
        serde_json::json!({
            "name": name,
            "description": description,
            "inputSchema": input_schema,
            "cli": { "command": command, "args": args_template },
            "outputFields": output_fields,
        })
    }

    let tools = vec![
        tool(
            "hwp_info",
            "HWP/HWPX/HML 문서의 메타데이터(포맷·구역/페이지/문단 수·폰트)를 조회한다. 문서를 열기 전에 규모와 형식을 파악할 때 쓴다.",
            path_schema(serde_json::json!({})),
            "info",
            serde_json::json!(["info", "--json", "{path}"]),
            &["format", "sizeBytes", "sections", "pageCount", "paraCount", "fonts"],
        ),
        tool(
            "hwp_export_text",
            "문서의 페이지별 본문 텍스트를 추출한다. 특정 페이지만 필요하면 page 를 준다.",
            path_schema(serde_json::json!({
                "page": { "type": "integer", "minimum": 0, "description": "0부터 시작하는 페이지 번호. 생략하면 전체" }
            })),
            "export-text",
            serde_json::json!(["export-text", "--json", "{path}"]),
            &["pageCount", "pages"],
        ),
        tool(
            "hwp_export_structure",
            "문서의 개요/조문 계층을 트리로 추출한다. 법령·규정의 '제N조' 구조를 얻어 조문 단위로 인용하거나 청킹할 때 쓴다.",
            path_schema(serde_json::json!({
                "mode": {
                    "type": "string",
                    "enum": ["auto", "outline", "clause"],
                    "description": "분류 방식. 기본 auto"
                }
            })),
            "export-structure",
            serde_json::json!(["export-structure", "--json", "{path}"]),
            &["mode", "nodeCount", "structure"],
        ),
        tool(
            "hwp_ir_diff",
            "두 문서의 내부 표현(IR) 차이를 비교한다. 변환 전후의 내용 보존을 검증할 때 쓴다. 차이가 있으면 CLI 종료 코드 3.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "string", "description": "비교 대상 A 경로" },
                    "b": { "type": "string", "description": "비교 대상 B 경로" }
                },
                "required": ["a", "b"],
            }),
            "ir-diff",
            serde_json::json!(["ir-diff", "{a}", "{b}", "--json"]),
            &["identical", "diffCount", "categories"],
        ),
        tool(
            "hwp_export_svg",
            "문서를 SVG로 렌더하고 생성된 페이지별 파일 경로를 JSON 매니페스트로 돌려준다.",
            path_schema(serde_json::json!({})),
            "export-svg",
            serde_json::json!(["export-svg", "{path}", "--json"]),
            &["format", "outputDir", "pageCount", "renderedCount", "pages"],
        ),
        tool(
            "hwp_export_tables",
            "문서의 표를 병합 정보와 중첩 구조를 보존한 격자 JSON으로 추출한다.",
            path_schema(serde_json::json!({})),
            "export-tables",
            serde_json::json!(["export-tables", "{path}", "--json"]),
            &["source", "tableCount", "tables"],
        ),
        tool(
            "hwp_search",
            "문서에서 검색어를 찾아 구역·문단·페이지·문자 오프셋 주소와 문맥을 돌려준다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "HWP/HWPX 문서 경로" },
                    "query": { "type": "string", "minLength": 1, "description": "검색어" }
                },
                "required": ["path", "query"],
            }),
            "search",
            serde_json::json!(["search", "{path}", "{query}", "--json"]),
            &["source", "query", "caseSensitive", "matchCount", "matches"],
        ),
        tool(
            "hwp_fields",
            "문서의 누름틀·필드를 이름·안내문·현재값·위치와 함께 조사한다.",
            path_schema(serde_json::json!({})),
            "fields",
            serde_json::json!(["fields", "{path}", "--json"]),
            &["source", "fieldCount", "fields"],
        ),
        tool(
            "hwp_batch",
            "여러 문서를 한 프로세스에서 병렬 처리해 NDJSON 스트림으로 받는다. 파일 목록은 stdin 으로 한 줄에 하나씩 넣는다. 아카이브 전체를 스윕할 때 쓴다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "subcommand": {
                        "type": "string",
                        "enum": ["export-text", "info", "export-structure"],
                        "description": "각 파일에 적용할 처리"
                    },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "처리할 문서 경로 목록 (stdin 으로 전달된다)"
                    },
                    "threads": { "type": "integer", "minimum": 1, "description": "병렬 스레드 수. 기본은 CPU 코어 수" }
                },
                "required": ["subcommand", "paths"],
            }),
            "batch",
            serde_json::json!(["batch", "{subcommand}", "--json"]),
            &["schemaVersion", "source", "error", "exitClass"],
        ),
    ];

    let manifest = serde_json::json!({
        "schemaVersion": "1.0",
        "protocol": "mcp",
        "server": {
            "suggestedName": "rhwp",
            "version": rhwp::version(),
            "description": "HWP/HWPX 한국어 문서를 읽는 도구 모음 (읽기 전용)",
        },
        "invocation": {
            "transport": "cli",
            "note": "각 도구의 cli.args 에서 {name} 자리표시자를 inputSchema 의 같은 이름 값으로 치환해 실행한다. stdout 은 순수 JSON, 진단은 stderr, 종료 코드는 0/1/2(+ir-diff 차이 3).",
            "stdinTools": ["hwp_batch"],
        },
        "tools": tools,
    });
    println!("{manifest}");
    EXIT_OK
}

/// [#3263] 도구 자기서술 — 에이전트가 첫 호출 1회로 명령·계약·스키마를 파악하는 입구.
///
/// `--help`(사람용)와 본 목록(기계용)은 함께 현행화한다 — help 에만 추가된 명령은
/// `tests/cli_json_contract.rs::capabilities_covers_every_help_command` 가 잡는다.
fn show_capabilities(args: &[String]) -> i32 {
    // [#3263] --mcp: MCP 서버가 그대로 등록할 수 있는 도구 정의.
    // 로드맵상 MCP 서버 자체는 별도 저장소(#227)지만, 그 서버가 도구 목록·입력 스키마를
    // 손으로 베껴 쓰면 rhwp 가 바뀔 때마다 조용히 낡는다. 원천을 여기서 낸다.
    let mut mcp_mode = false;
    for a in args {
        match a.as_str() {
            "--mcp" => mcp_mode = true,
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
    }
    if mcp_mode {
        return show_mcp_tools();
    }

    fn cmd(name: &str, category: &str, summary: &str) -> serde_json::Value {
        serde_json::json!({ "name": name, "category": category, "summary": summary })
    }
    fn cmd_json(
        name: &str,
        category: &str,
        summary: &str,
        batch: bool,
        flags: &[&str],
        record_fields: &[&str],
    ) -> serde_json::Value {
        serde_json::json!({
            "name": name, "category": category, "summary": summary,
            "json": true, "batch": batch, "flags": flags, "recordFields": record_fields,
        })
    }

    let commands = vec![
        // ── 기계 계약(--json) 명령 ──
        cmd_json(
            "info",
            "query",
            "문서 메타(포맷·버전·페이지/문단 수·폰트) 표시",
            true,
            &["--json"],
            &[
                "schemaVersion",
                "source",
                "format",
                "sizeBytes",
                "version",
                "sections",
                "pageCount",
                "paraCount",
                "fonts",
            ],
        ),
        cmd_json(
            "export-text",
            "export",
            "페이지별 텍스트 추출 (TXT 파일 또는 --json stdout)",
            true,
            &["-o", "-p", "--json"],
            &["schemaVersion", "source", "pageCount", "pages"],
        ),
        cmd_json(
            "export-structure",
            "export",
            "문서 개요/조문 계층을 JSON 트리로 추출",
            true,
            &["--mode", "-o", "--json"],
            &["schemaVersion", "source", "mode", "nodeCount", "structure"],
        ),
        cmd_json(
            "capabilities",
            "query",
            "본 자기서술 JSON 출력",
            false,
            &[],
            &[
                "schemaVersion",
                "tool",
                "version",
                "exitCodes",
                "commands",
                "batch",
            ],
        ),
        // ── 내보내기/변환 ──
        cmd_json(
            "export-svg",
            "export",
            "문서를 페이지별 SVG로 렌더하고 --json 매니페스트 출력",
            false,
            &["-o", "-p", "--json"],
            &[
                "schemaVersion",
                "source",
                "format",
                "outputDir",
                "pageCount",
                "renderedCount",
                "pages",
            ],
        ),
        cmd(
            "export-png",
            "export",
            "문서를 페이지별 PNG로 렌더 (native-skia)",
        ),
        cmd(
            "export-pdf",
            "export",
            "문서를 PDF로 렌더 (svg|direct backend)",
        ),
        cmd(
            "export-markdown",
            "export",
            "페이지별 텍스트를 Markdown으로 추출",
        ),
        cmd(
            "export-hwpx",
            "export",
            "HWP→HWPX 변환 저장 (--verify 게이트, exit 3/4)",
        ),
        cmd("export-hml", "export", "HML 원본을 HWPML 2.91 XML로 저장"),
        cmd(
            "export-doclang",
            "export",
            "문서를 DocLang v0.6 XML로 내보내기",
        ),
        cmd_json(
            "export-tables",
            "export",
            "표를 병합·중첩 구조를 보존한 격자 JSON으로 추출",
            false,
            &["-o", "--json"],
            &["schemaVersion", "source", "tableCount", "tables"],
        ),
        cmd_json(
            "search",
            "query",
            "문서 검색 결과를 구역·문단·페이지·문자 오프셋 주소와 함께 출력",
            false,
            &["--json", "--ignore-case", "--limit"],
            &[
                "schemaVersion",
                "source",
                "query",
                "caseSensitive",
                "matchCount",
                "matches",
            ],
        ),
        cmd_json(
            "fields",
            "query",
            "누름틀/필드를 이름·안내문·현재값·위치와 함께 조사",
            false,
            &["--json"],
            &["schemaVersion", "source", "fieldCount", "fields"],
        ),
        cmd(
            "export-render-tree",
            "export",
            "페이지별 render tree bbox JSON 덤프",
        ),
        cmd(
            "convert",
            "export",
            "HWP↔HWPX 변환 (--verify/--verify-pages 게이트)",
        ),
        cmd("build-from-ingest", "export", "ingest JSON에서 HWPX 생성"),
        cmd("thumbnail", "export", "내장 썸네일(PrvImage) 추출"),
        // ── 배치 ──
        cmd_json(
            "batch",
            "batch",
            "stdin 파일 목록을 한 프로세스에서 파일 간 병렬 처리, NDJSON 스트림 출력",
            true,
            &["--json", "--threads", "--mode"],
            &["schemaVersion", "source", "error", "exitClass"],
        ),
        // ── 진단 ──
        cmd("dump", "diagnostic", "문서 조판부호 구조 덤프"),
        cmd("dump-pages", "diagnostic", "페이지네이션 항목 덤프"),
        cmd("dump-note-shape", "diagnostic", "각주/미주 모양 덤프"),
        cmd("dump-endnote-lines", "diagnostic", "미주 줄 배치 덤프"),
        cmd("dump-records", "diagnostic", "저수준 레코드 스트림 덤프"),
        cmd("diag", "diagnostic", "문서 구조 진단(번호/글머리표/개요)"),
        cmd_json(
            "ir-diff",
            "diagnostic",
            "두 문서의 IR 차이를 JSON으로 비교",
            false,
            &["-s", "-p", "--json"],
            &[
                "schemaVersion",
                "sourceA",
                "sourceB",
                "identical",
                "diffCount",
                "categories",
            ],
        ),
        cmd(
            "render-diff",
            "diagnostic",
            "왕복/두 파일 렌더 기하 차이 검증",
        ),
        cmd("hwpx-roundtrip", "diagnostic", "HWPX 왕복 무손실 게이트"),
        cmd("hwp5-roundtrip", "diagnostic", "HWP5 왕복 무손실 게이트"),
        cmd("measure-width", "diagnostic", "텍스트 폭 측정 프로브"),
        cmd("core-pages", "diagnostic", "코어 페이지 수 프로브"),
        cmd("bench", "diagnostic", "성능 벤치마크"),
        cmd("hwp5-inventory", "diagnostic", "HWP5 레코드 인벤토리"),
        cmd("hwp5-inventory-diff", "diagnostic", "HWP5 인벤토리 비교"),
        cmd(
            "hwp5-contract-analyze",
            "diagnostic",
            "HWPX→HWP5 저장 계약 분석",
        ),
        cmd("hwp5-contract-probe", "diagnostic", "HWP5 저장 계약 프로브"),
        cmd("hwp5-ctrl-data-trace", "diagnostic", "CTRL_DATA 추적"),
        cmd("hwp5-table-probe", "diagnostic", "표 저장 프로브"),
        cmd(
            "hwp5-mel-personnel-probe",
            "diagnostic",
            "특정 샘플 재현 프로브",
        ),
        cmd(
            "hwp5-borderfill-diagonal-probe",
            "diagnostic",
            "테두리 대각선 프로브",
        ),
        cmd(
            "hwp5-first-para-control-probe",
            "diagnostic",
            "첫 문단 컨트롤 프로브",
        ),
        cmd("hwp5-anchor-trace", "diagnostic", "앵커 추적"),
        cmd("hwp5-cell-header-probe", "diagnostic", "셀 헤더 프로브"),
        // ── 내부 개발용 ──
        cmd("test-shape", "internal", "도형 왕복 테스트"),
        cmd("test-caption", "internal", "캡션 테스트"),
        cmd("test-field", "internal", "누름틀 왕복 테스트"),
        cmd("gen-table", "internal", "표 샘플 생성"),
        cmd("gen-pua", "internal", "PUA 샘플 생성"),
    ];

    let caps = serde_json::json!({
        "schemaVersion": "1.0",
        "tool": "rhwp",
        "version": rhwp::version(),
        "formats": { "read": ["hwp5", "hwpx", "hwp3", "hml"], "write": ["hwpx", "hml", "pdf", "svg", "png", "txt", "md", "doclang"] },
        "exitCodes": {
            "0": "성공",
            "1": "런타임 실패 (읽기·파싱·렌더·쓰기)",
            "2": "사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과)",
            "3": "--verify IR 차이 (convert/export-hwpx)",
            "4": "--verify-pages 페이지 수 불일치 (convert/export-hwpx)",
        },
        "jsonContract": {
            "stdout": "데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr",
            "schemaPolicy": "필드 추가 허용, 변경·삭제는 schemaVersion 범프",
            "failure": "단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1",
        },
        "batch": {
            "subcommands": ["export-text", "info", "export-structure"],
            "flags": ["--json", "--threads", "--mode"],
            "ordering": "stdin 입력 순서 보존",
            "input": "stdin, 한 줄당 파일 경로 하나",
        },
        "commands": commands,
    });
    println!("{caps}");
    EXIT_OK
}

fn print_help() {
    println!("rhwp v{} - HWP 파일 뷰어", rhwp::version());
    println!();
    println!("사용법: rhwp <명령> [옵션]");
    println!();
    println!("명령:");
    println!("  export-svg <파일.hwp|파일.hwpx|파일.hml> [옵션]");
    println!("      HWP/HWPX/HML 문서를 SVG로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!(
        "      --profile <프로필>      layer 출력 프로필: screen|print|high-quality|fast-preview"
    );
    println!("      --show-para-marks       문단부호(↵/↓) 표시");
    println!("      --show-control-codes    조판부호 보이기 (문단부호 + 개체 마커 등)");
    println!("      --debug-overlay         디버그 오버레이 (문단/표 경계 + 인덱스 라벨)");
    println!("      --respect-vpos-reset    LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리");
    println!("      --show-grid[=Nmm]       격자 오버레이 (기본: 1mm, 예: --show-grid=3mm)");
    println!("      --grid-origin=X,Y|auto  격자 종이 기준 위치 (예: --grid-origin=15mm,20mm)");
    println!("      --font-style            @font-face local() 참조 삽입 (폰트 데이터 미포함)");
    println!("      --embed-fonts           폰트 서브셋 임베딩 (사용 글자만 base64)");
    println!("      --embed-fonts=full      폰트 전체 임베딩 (base64)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!("      --json                  산출물 매니페스트를 JSON으로 stdout에 출력");
    println!();
    println!("  export-render-tree <파일.hwp> [옵션]");
    println!("      페이지별 render tree bbox JSON을 내보내기 (레이아웃 시각 분석용)");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --show-para-marks       문단부호(↵/↓) 표시 상태의 트리 생성");
    println!("      --show-control-codes    조판부호 보이기 상태의 트리 생성");
    println!("      --respect-vpos-reset    LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리");
    println!();
    println!("  export-structure <파일> [--mode auto|outline|clause] [-o out.json] [--json]");
    println!("      문서 개요/조문(편·장·절·관·조·항·호·목) 계층을 중첩 JSON 트리로 추출");
    println!();
    println!("      --mode <방식>           분류 방식 auto|outline|clause (기본: auto)");
    println!("      -o, --out <파일>        출력 JSON 파일 경로 (생략 시 stdout)");
    println!();
    println!("  export-png <파일.hwp> [옵션]   (native-skia feature 필요)");
    println!("      HWP 파일을 PNG로 내보내기 (Skia raster backend, AI 파이프라인 + VLM 연동)");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!(
        "      --profile <프로필>      출력 프로필: screen|print|high-quality|fast-preview (기본: high-quality)"
    );
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!("                              한컴 전용 폰트 (HY견명조 등) 가 시스템에 없을 때 ttfs 디렉토리 지정");
    println!("      --scale <배율>          렌더링 배율 (기본: 1.0)");
    println!("      --max-dimension <픽셀>  한 변 최대 픽셀 (longest edge). VLM 입력 한도용.");
    println!(
        "                              명시 --scale 이 없으면 자동 scale 계산 (페이지 → 한도 안)"
    );
    println!("      --dpi <값>              DPI 메타데이터 (PNG pHYs chunk). 실제 픽셀 수 무관.");
    println!("                              --scale 미지정 시 scale = dpi/96 자동 계산");
    println!("      --vlm-target <프리셋>   VLM 입력 프리셋 (하이픈/밑줄 모두 허용):");
    println!("                              claude:     1568 px / 1.15 MP (Claude Vision)");
    println!("                              gpt4v-low:  512 px (GPT-4V low detail)");
    println!(
        "                              gpt4v-high: 2000 px / 1.54 MP (GPT-4V high, 별칭: gpt4v)"
    );
    println!("                              gemini:     3072 px (Google Gemini)");
    println!("                              qwen-vl:    2240 px (Qwen-VL, 별칭: qwen)");
    println!("                              llava:      672 px (LLaVA / OSS CLIP)");
    println!();
    println!("  export-text <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 TXT로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --json                  결과를 JSON으로 stdout에 출력 (파일 저장 안 함)");
    println!();
    println!("  batch <export-text|info|export-structure> --json [--threads <N>]");
    println!(
        "      stdin의 파일 목록(한 줄당 하나)을 한 프로세스로 전건 처리해 NDJSON 스트림 출력"
    );
    println!("      --threads <N>           파일 간 병렬 스레드 수 (기본: CPU 코어 수)");
    println!("      --mode <m>              export-structure 전용: auto|outline|clause");
    println!();
    println!("  export-markdown <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 Markdown(.md)으로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!();
    println!("  export-tables <파일.hwp|파일.hwpx> [--json] [-o <출력.json>]");
    println!("      표를 격자 JSON으로 추출 (병합 rowSpan/colSpan·중첩 표 보존)");
    println!();
    println!("      --json                  계약 봉투 JSON을 stdout에 출력");
    println!("      -o, --output <파일>     JSON을 파일로 저장");
    println!();
    println!("  export-pdf <파일.hwp|파일.hwpx|파일.hml> [옵션]");
    println!("      HWP/HWPX/HML 문서를 PDF로 내보내기 (기본: SVG 호환 backend)");
    println!();
    println!("      -o, --output <파일>      출력 PDF 파일 (기본: output/<입력명>.pdf)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --backend <svg|direct>  PDF backend (기본값: svg)");
    println!(
        "      --profile <프로필>      layer 출력 프로필: screen|print|high-quality|fast-preview"
    );
    println!("      --raster-dpi <DPI>      direct backend fallback raster DPI (기본값: 144)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!("      --fallback-serif <명>   PDF serif generic fallback family");
    println!("      --fallback-sans <명>    PDF sans-serif generic fallback family");
    println!("      --fallback-mono <명>    PDF monospace generic fallback family");
    println!("      --equation-font <명>    PDF 수식 SVG 우선 font-family");
    println!("      --text-as-paths         텍스트를 폰트 임베드 대신 path로 변환");
    println!("                              (메모리 대폭 절감, 텍스트 선택·검색 불가)");
    println!(
        "                              <...>는 자리표시자이며, 실제 입력에는 꺾쇠괄호를 쓰지 않음"
    );
    println!(
        "                              경로/폰트명에 공백이 있으면 큰따옴표 권장: --font-path \"./My Fonts\""
    );
    println!("                              예: --fallback-sans \"Apple SD Gothic Neo\"");
    println!();
    println!("  export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx] [--verify] [--verify-pages]");
    println!("      HWP 문서를 HWPX(ZIP+XML)로 변환 저장. 출력 생략 시 <입력 stem>.hwpx");
    println!(
        "      --verify              변환 후 산출물을 재파싱해 IR 차이를 검출 (차이 시 exit 3)"
    );
    println!("      --verify-pages        변환 전/후 렌더 페이지 수를 비교 (불일치 시 exit 4)");
    println!();
    println!("  export-hml <입력.hml> -o <출력.hml>");
    println!("      HML 원본 문서를 의미 보존 HWPML 2.91 XML로 저장");
    println!("      -o, --output <파일>    출력 HML 파일 (필수, 원본 덮어쓰기 금지)");
    println!();
    println!("  export-doclang <파일.hwp|파일.hwpx> [-o <출력.xml>] [--assets-dir <디렉터리>]");
    println!("      HWP/HWPX 문서를 DocLang v0.6 XML로 내보내기");
    println!();
    println!("      -o, --output <파일>     출력 XML 파일 (기본: <입력 stem>.dclg.xml)");
    println!("      --assets-dir <디렉터리> 그림 등 이진 자원을 이 디렉터리에 파일로 기록");
    println!("                              (생략 시 base64 data URI로 XML에 인라인)");
    println!();
    println!("  info <파일.hwp|파일.hwpx|파일.hml> [--json]");
    println!("      HWP/HWPX/HML 문서 정보 표시");
    println!();
    println!("      --json                  문서 정보를 JSON으로 stdout에 출력");
    println!();
    println!("  capabilities [--mcp]");
    println!("      도구 자기서술 JSON 출력 (명령·플래그·JSON 계약·종료 코드) — 에이전트용");
    println!();
    println!("      --mcp                   MCP 도구 정의(name/description/inputSchema) 출력");
    println!();
    println!("  dump <파일.hwp|파일.hwpx|파일.hml> [--section <번호>] [--para <번호>]");
    println!("      문서 조판부호 구조 덤프 (디버깅용)");
    println!();
    println!("  dump-note-shape <파일.hwp|파일.hwpx>");
    println!("      구역별 각주/미주 모양 raw 값과 한컴 UI 의미값을 JSON으로 덤프");
    println!();
    println!("  dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]");
    println!("      특정 미주 원본 문단의 line_seg, TextRun, TAC 수식 위치를 함께 덤프");
    println!();
    println!("  dump-pages <파일.hwp> [-p <번호>] [--respect-vpos-reset]");
    println!("      페이지네이션 결과 덤프 (페이지별 문단/표 배치 목록)");
    println!();
    println!("  dump-records <파일.hwp>");
    println!("      HWP5 raw record 덤프 (DocInfo/BodyText 레코드 트리)");
    println!();
    println!("  diag <파일.hwp>");
    println!("      문서 구조 진단 (번호/글머리표/개요 분석)");
    println!();
    println!("  search <파일.hwp|파일.hwpx> <검색어> [옵션]");
    println!("      문서 검색 — 매치마다 구역·문단·페이지·문자 오프셋을 함께 반환");
    println!();
    println!("      --json                    계약 봉투 JSON을 stdout에 출력");
    println!("      --ignore-case             대소문자 무시");
    println!("      --limit <N>               최대 매치 수 (컨텍스트 절약용)");
    println!();
    println!("  hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]");
    println!("      HWP5 DocInfo/BodyText record inventory 생성 (HWPX→HWP contract 분석용)");
    println!();
    println!("  hwp5-inventory-diff <oracle.hwp> <generated.hwp> [--align index|lcs] [--report diff|hints|bundles|table-fields|table-probe-plan] [--focus all|table|shape|ctrl|missing|docinfo] [--window N] [--format jsonl|md] [--section N] [--out <path>]");
    println!("      HWP5 inventory 비교 결과, contract 후보 힌트, 후보 주변 bundle 생성");
    println!();
    println!("  hwp5-contract-analyze <source.hwpx> <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      HWPX/HWP oracle/generated record-control contract graph 분석 보고서 생성");
    println!();
    println!("  hwp5-ctrl-data-trace <oracle.hwp> <generated.hwp> --out <path> [--section N] [--record-index N]");
    println!("      oracle/generated CTRL_DATA ParameterSet 구조 추적 보고서 생성");
    println!();
    println!("  hwp5-contract-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      DocInfo MEMO_SHAPE/ID_MAPPINGS와 누락 CTRL_DATA 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-table-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      TABLE/CTRL_HEADER(Table) field 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-mel-personnel-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      mel-001 인원현황 표 TABLE/LIST_HEADER/PARA_HEADER 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-borderfill-diagonal-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      DocInfo BORDER_FILL 대각선 attr/payload 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-first-para-control-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      첫 문단 control/PARA_TEXT/PARA_CHAR_SHAPE 계약 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-anchor-trace <파일.hwp> --needle <텍스트> [--section N] [--window N] [--out <path>]");
    println!("      특정 텍스트를 포함한 PARA_TEXT 주변의 raw HWP5 record를 추적");
    println!();
    println!("  hwp5-cell-header-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      표 셀 LIST_HEADER/PARA_HEADER 계약 축별 판정용 HWP probe 생성");
    println!();
    println!("  convert <입력.hwp|입력.hwpx> <출력.hwp> [--verify] [--verify-pages]");
    println!("      배포용(읽기전용) HWP를 편집 가능한 HWP로 변환");
    println!("      --verify              저장 후 재파싱 IR 차이를 검출 (차이 시 exit 3)");
    println!("      --verify-pages        저장 전/후 렌더 페이지 수를 비교 (불일치 시 exit 4)");
    println!();
    println!("  build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
    println!("      ingest JSON(시험문제 등)을 HWPX로 생성 (rhwp-exam-ingest 파이프라인)");
    println!();
    println!("  ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>] [--json]");
    println!("      두 파일의 IR(중간표현) 비교 (HWPX↔HWP 불일치 검출)");
    println!("      --json                  판정 봉투 JSON 한 줄 출력, 차이 발견 시 exit 3");
    println!("      비교 항목: text, char_count, char_offsets, char_shapes, line_segs,");
    println!("                 controls(타입+속성), tab_extended, ParaShape, TabDef");
    println!("      표: page_break, outer_margin, treat_as_char, wrap, size, v_offset/h_offset");
    println!("      그림/도형: treat_as_char, wrap, size, v_offset/h_offset, vert_rel/horz_rel");
    println!();
    println!("  hwpx-roundtrip <파일.hwpx | --batch 폴더> [-o <출력폴더>] [--lineseg-report]");
    println!("      HWPX → IR → HWPX roundtrip 검증 (Task #1315 baseline)");
    println!("      재조립 .hwpx와 inventory.tsv를 출력 폴더(기본 output/poc/task1315)에 생성");
    println!("      --lineseg-report: 문단별 lineseg diff를 lineseg_diff.tsv로 산출 (#1380 측정)");
    println!("  hwp5-roundtrip <파일.hwp | --batch 폴더> [-o <출력폴더>]");
    println!("      HWP5 → IR → HWP5 roundtrip 무손실 검증 (Task #1552)");
    println!("      재조립 .rt.hwp와 inventory.tsv를 출력 폴더(기본 output/poc/task1552)에 생성");
    println!("  render-diff <파일> [--via hwpx|hwp] [-p <페이지>] [--max-disp <px>]");
    println!("  render-diff <파일A> <파일B> [-p <페이지>] [--max-disp <px>]");
    println!("  render-diff --batch <폴더> [--via hwpx] [-o <출력폴더>] [--max-disp <px>]");
    println!("      라운드트립 시각 정합성 게이트 — 페이지별 RenderNode bbox 변위(px) 정량화");
    println!("      자기 라운드트립(원본 IR vs 직렬화→재로드 IR) 또는 두 파일 직접 비교");
    println!("      배치: geom_inventory.tsv 산출(기본 output/poc/render_diff)");
    println!("  bench <파일...> | --batch <폴더> [-n <반복수>] [--tsv <출력.tsv>]");
    println!("      단계별 처리 성능 계측 — parse/layout/render/serialize median(ms)");
    println!("      워밍업 1회 후 N회(기본 3) 반복. 파일별 크기/쪽수 + total 표 + TSV");
    println!("      주의: 절대 수치는 머신·빌드 의존, 동일 환경 상대·재현 지표로 해석");
    println!();
    println!("  thumbnail <파일.hwp> [옵션]");
    println!("      HWP 파일에서 썸네일(PrvImage) 추출");
    println!();
    println!("      -o, --output <파일>       출력 파일 경로 (기본: 입력명_thumb.png)");
    println!("      --base64                  base64 문자열을 stdout에 출력");
    println!("      --data-uri                data:image/... URI 형식으로 stdout에 출력");
    println!();
    println!("  fields <파일.hwp|파일.hwpx> [--json]");
    println!("      누름틀/필드 조사 (읽기 전용) — 이름·안내문·지시문·현재값·위치");
    println!();
    println!("      --json                    계약 봉투 JSON을 stdout에 출력");
    println!();
    println!("내부 개발·회귀 도구 (일반 사용자 대상 아님):");
    println!("  test-caption <파일.hwp>             캡션 라운드트립 검증");
    println!("  test-field <파일.hwp>               필드 라운드트립 검증");
    println!("  test-shape <입력.hwp> <출력.hwp>    도형 라운드트립 검증");
    println!("  gen-table                           표 테스트 HWP 생성");
    println!("  gen-pua                             PUA 문자 테스트 HWP 생성");
    println!();
    println!("옵션:");
    println!("  -h, --help      도움말 표시");
    println!("  -V, --version   버전 표시");
}

fn allows_implicit_sibling_resources(format: rhwp::parser::FileFormat) -> bool {
    // HML sibling paths are untrusted input and require an explicit resolver policy.
    !matches!(format, rhwp::parser::FileFormat::Hml)
}

fn export_svg(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp export-svg <파일.hwp|파일.hwpx|파일.hml> [옵션] (rhwp --help 참조)"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut debug_overlay = false;
    let mut grid_mm: Option<f64> = None;
    let mut grid_origin = GridOriginOption::Fixed((0.0_f64, 0.0_f64));
    let mut respect_vpos_reset = false;
    let mut font_embed_mode = rhwp::renderer::svg::FontEmbedMode::None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut render_profile: Option<rhwp::paint::RenderProfile> = None;
    let mut json_mode = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--profile" => {
                if i + 1 < args.len() {
                    render_profile = rhwp::paint::RenderProfile::parse(&args[i + 1]);
                    if render_profile.is_none() {
                        eprintln!(
                            "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                        );
                        return EXIT_USAGE;
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --profile 뒤에 프로필 이름이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--debug-overlay" => {
                debug_overlay = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            arg if arg == "--show-grid" || arg.starts_with("--show-grid=") => {
                grid_mm = if let Some(value) = arg.strip_prefix("--show-grid=") {
                    match parse_grid_mm(value) {
                        Some(v) => Some(v),
                        None => {
                            eprintln!(
                                "오류: --show-grid 값이 올바르지 않습니다. 예: --show-grid=3mm"
                            );
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    Some(1.0)
                };
                i += 1;
            }
            arg if arg == "--grid-origin" || arg == "--grid-paper-origin" => {
                if i + 1 < args.len() {
                    match parse_grid_origin_option(&args[i + 1]) {
                        Some(v) => grid_origin = v,
                        None => {
                            eprintln!(
                                "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                            );
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --grid-origin 뒤에 가로,세로 값이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            arg if arg.starts_with("--grid-origin=") || arg.starts_with("--grid-paper-origin=") => {
                let value = arg
                    .strip_prefix("--grid-origin=")
                    .or_else(|| arg.strip_prefix("--grid-paper-origin="))
                    .unwrap_or_default();
                match parse_grid_origin_option(value) {
                    Some(v) => grid_origin = v,
                    None => {
                        eprintln!(
                            "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                        );
                        return EXIT_USAGE;
                    }
                }
                i += 1;
            }
            "--font-style" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Style;
                i += 1;
            }
            "--embed-fonts" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Subset;
                i += 1;
            }
            "--embed-fonts=full" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Full;
                i += 1;
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--json" => {
                // [#3286] 산출물 매니페스트를 stdout 에 JSON 으로 — 에이전트가
                // 어떤 파일이 생겼는지 파싱 없이 알 수 있게 한다.
                json_mode = true;
                i += 1;
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    if render_profile.is_some() && font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
        eprintln!("오류: --profile은 --font-style/--embed-fonts와 함께 사용할 수 없습니다.");
        return EXIT_USAGE;
    }

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let source_format = rhwp::parser::detect_format(&data);

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 문서 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    // [Task #741 후속] 외부 file path 그림 영역 영역 HWP file 영역 영역 같은 dir 영역
    // 영역 image 영역 영역 자동 load (basename 매칭).
    if allows_implicit_sibling_resources(source_format) {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            let _loaded = doc.populate_external_images_from_dir(parent);
        }
    }

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if debug_overlay {
        doc.set_debug_overlay(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();
    if !json_mode {
        // stdout 순수성: --json 모드에서는 데이터(JSON)만 나간다.
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    }

    // 출력 폴더 생성
    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    // 페이지 범위 결정
    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // SVG 내보내기
    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut manifest: Vec<serde_json::Value> = Vec::new();
    let mut written = 0usize;

    for page_num in &pages {
        let svg_result = if let Some(profile) = render_profile {
            doc.render_page_svg_layer_with_profile_native(*page_num, profile)
        } else if font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
            doc.render_page_svg_with_fonts(*page_num, font_embed_mode, &font_paths)
        } else {
            doc.render_page_svg_native(*page_num)
        };
        match svg_result {
            Ok(mut svg) => {
                // 격자 오버레이 삽입
                if let Some(mm) = grid_mm {
                    let origin_mm = match grid_origin {
                        GridOriginOption::Fixed(origin) => origin,
                        GridOriginOption::AutoPaper => {
                            match grid_paper_origin_mm(&doc, *page_num) {
                                Some(origin) => origin,
                                None => {
                                    eprintln!(
                                        "오류: 페이지 {}의 격자 기준 위치를 계산할 수 없습니다.",
                                        page_num
                                    );
                                    continue;
                                }
                            }
                        }
                    };
                    svg = insert_grid_overlay(&svg, mm, origin_mm);
                }
                let svg_filename = if page_count == 1 {
                    format!("{}.svg", file_stem)
                } else {
                    format!("{}_{:03}.svg", file_stem, page_num + 1)
                };
                let svg_path = output_path.join(&svg_filename);

                match fs::write(&svg_path, &svg) {
                    Ok(_) => {
                        if json_mode {
                            manifest.push(serde_json::json!({
                                "page": page_num,
                                "path": svg_path.display().to_string(),
                                "bytes": svg.len(),
                            }));
                        } else {
                            println!("  → {}", svg_path.display());
                        }
                        written += 1;
                    }
                    Err(e) => eprintln!("오류: SVG 저장 실패 - {}: {}", svg_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num, e);
            }
        }
    }

    // 단건 JSON 명령의 실패는 stdout 을 비워야 한다. 부분 매니페스트를 출력하면
    // 소비자가 성공 결과로 오인하거나 stdout JSON을 파싱한 뒤 실패를 놓친다.
    if written != pages.len() {
        if !json_mode {
            println!("내보내기 완료: {}개 SVG 파일 → {}/", written, output_dir);
        }
        return EXIT_RUNTIME;
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": "1.0",
            "source": file_path,
            "format": "svg",
            "outputDir": output_dir,
            "pageCount": page_count,
            "renderedCount": written,
            "pages": manifest,
        });
        println!("{envelope}");
    } else {
        println!("내보내기 완료: {}개 SVG 파일 → {}/", written, output_dir);
    }

    EXIT_OK
}

fn export_render_tree(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-render-tree <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut respect_vpos_reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let source_format = rhwp::parser::detect_format(&data);

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if allows_implicit_sibling_resources(source_format) {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            let _loaded = doc.populate_external_images_from_dir(parent);
        }
    }

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut written = 0usize;

    for page_num in &pages {
        match doc.build_page_render_tree(*page_num) {
            Ok(tree) => {
                let json_path = output_path.join(format!("render_tree_{:03}.json", page_num + 1));
                let json = tree.root.to_json();
                match fs::write(&json_path, json) {
                    Ok(_) => {
                        println!("  → {}", json_path.display());
                        written += 1;
                    }
                    Err(e) => {
                        eprintln!(
                            "오류: render tree 저장 실패 - {}: {}",
                            json_path.display(),
                            e
                        )
                    }
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} render tree 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 render tree JSON 파일 → {}/",
        written, output_dir
    );

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다.
    if written == pages.len() {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

/// `export-structure` — 문서 개요/조문 계층을 중첩 JSON 트리로 추출 (조문 DB화용).
fn export_structure(args: &[String]) -> i32 {
    use rhwp::document_core::queries::structure::{build_structure, StructureMode};

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut mode = StructureMode::Auto;
    // [#3261] --json: 계약 봉투(schemaVersion·source)를 씌운 한 줄 JSON.
    // 기본 출력(무봉투 pretty JSON·-o 파일 저장)은 기존 소비자 계약이라 건드리지 않는다.
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--mode" => {
                i += 1;
                match args.get(i).and_then(|s| StructureMode::parse(s)) {
                    Some(m) => mode = m,
                    None => {
                        eprintln!("오류: --mode 는 auto|outline|clause");
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp export-structure <파일> [--mode auto|outline|clause] [-o out.json]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let st = build_structure(doc.document(), mode);

    if json_mode {
        // [#3261] 봉투는 한 줄 — NDJSON(batch)과 같은 스키마로 단건/배치 동일 소비.
        let envelope = structure_json_value(file_path, &st);
        println!("{envelope}");
        return EXIT_OK;
    }

    let json = match serde_json::to_string_pretty(&st) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("오류: JSON 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match out_path {
        Some(p) => match fs::write(&p, &json) {
            Ok(_) => {
                println!(
                    "구조 추출 완료: mode={} 노드={} → {}",
                    st.mode, st.node_count, p
                );
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", p, e);
                // [#2707] 출력 파일을 못 쓴 실행은 실패다.
                EXIT_RUNTIME
            }
        },
        None => {
            println!("{json}");
            EXIT_OK
        }
    }
}

fn parse_grid_mm(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let number = trimmed
        .strip_suffix("mm")
        .or_else(|| trimmed.strip_suffix("MM"))
        .unwrap_or(trimmed)
        .trim();
    let mm = number.parse::<f64>().ok()?;
    if mm.is_finite() && mm > 0.0 {
        Some(mm)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum GridOriginOption {
    Fixed((f64, f64)),
    AutoPaper,
}

fn parse_grid_origin_option(value: &str) -> Option<GridOriginOption> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridOriginOption::AutoPaper);
    }
    parse_grid_origin_mm(value).map(GridOriginOption::Fixed)
}

fn parse_grid_origin_mm(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((parse_grid_mm(x)?, parse_grid_mm(y)?))
}

fn grid_paper_origin_mm(doc: &rhwp::wasm_api::HwpDocument, page_num: u32) -> Option<(f64, f64)> {
    let page_info = doc.get_page_info_native(page_num).ok()?;
    let page_info: serde_json::Value = serde_json::from_str(&page_info).ok()?;
    let section_idx = page_info.get("sectionIndex")?.as_u64()? as usize;
    let page_def = &doc
        .document()
        .sections
        .get(section_idx)?
        .section_def
        .page_def;
    Some((
        hu_to_mm(page_def.margin_left),
        hu_to_mm(page_def.margin_top + page_def.margin_header),
    ))
}

/// SVG에 mm 단위 점 격자 오버레이를 삽입한다.
/// export-svg 디버그용 격자는 한컴오피스의 "종이 기준 위치"를 옵션으로 맞출 수 있다.
fn insert_grid_overlay(svg: &str, grid_mm: f64, origin_mm: (f64, f64)) -> String {
    // SVG viewBox에서 크기 추출
    let (width, height) = extract_svg_dimensions(svg);
    // 96dpi: 1inch = 25.4mm, 1px = 25.4/96 = 0.2646mm.
    let grid_size = 96.0 / 25.4 * grid_mm;
    let origin_x = 96.0 / 25.4 * origin_mm.0;
    let origin_y = 96.0 / 25.4 * origin_mm.1;

    let g = format!("{:.4}", grid_size);
    let ox = format!("{:.4}", origin_x);
    let oy = format!("{:.4}", origin_y);
    let w = format!("{:.2}", width);
    let h = format!("{:.2}", height);
    let defs_part = format!(
        "<defs><pattern id=\"rhwp-grid\" x=\"{ox}\" y=\"{oy}\" width=\"{g}\" height=\"{g}\" patternUnits=\"userSpaceOnUse\"><rect x=\"0\" y=\"0\" width=\"1\" height=\"1\" fill=\"#002096\" fill-opacity=\"0.9\"/></pattern></defs>"
    );
    let grid_rect = format!("\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>");
    let grid_defs =
        format!("{defs_part}\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>\n");

    // 페이지 배경(fill="#ffffff") rect 직후에 격자를 삽입
    // 이렇게 해야 흰색 배경 위에, 본문 컨텐츠 아래에 격자가 표시됨
    let bg_pattern = "fill=\"#ffffff\"/>";
    if let Some(pos) = svg.find(bg_pattern) {
        let insert_pos = pos + bg_pattern.len();
        // defs는 SVG 시작 부분에, 격자 rect는 배경 뒤에
        // defs를 <svg> 태그 직후에 삽입
        let mut result = svg.to_string();
        // 배경 rect 뒤에 격자 rect 삽입
        result.insert_str(insert_pos, &grid_rect);
        // <svg ...>\n 직후에 defs 삽입
        if let Some(svg_end) = result.find(">\n") {
            result.insert_str(svg_end + 2, &format!("{}\n", defs_part));
        }
        result
    } else {
        // 배경 rect가 없으면 기존 방식
        if let Some(pos) = svg.find(">\n") {
            let insert_pos = pos + 2;
            format!("{}{}{}", &svg[..insert_pos], grid_defs, &svg[insert_pos..])
        } else {
            svg.to_string()
        }
    }
}

/// SVG의 width/height 속성 또는 viewBox에서 크기를 추출한다.
fn extract_svg_dimensions(svg: &str) -> (f64, f64) {
    // viewBox="0 0 W H" 패턴에서 추출
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let vb = &svg[vb_start + 9..];
        if let Some(vb_end) = vb.find('"') {
            let parts: Vec<&str> = vb[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                let w: f64 = parts[2].parse().unwrap_or(800.0);
                let h: f64 = parts[3].parse().unwrap_or(1100.0);
                return (w, h);
            }
        }
    }
    // width/height 속성에서 추출
    let w = extract_attr_f64(svg, "width").unwrap_or(800.0);
    let h = extract_attr_f64(svg, "height").unwrap_or(1100.0);
    (w, h)
}

fn extract_attr_f64(svg: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = svg.find(&pattern) {
        let val = &svg[start + pattern.len()..];
        if let Some(end) = val.find('"') {
            return val[..end].trim_end_matches("px").parse().ok();
        }
    }
    None
}

#[cfg(not(feature = "native-skia"))]
fn export_png(_args: &[String]) -> i32 {
    eprintln!("오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features native-skia");
    // [#2707] 기능이 아예 빌드되지 않은 바이너리다. 0으로 끝내면 스크립트가 성공으로 읽는다.
    EXIT_USAGE
}

#[cfg(feature = "native-skia")]
fn export_png(args: &[String]) -> i32 {
    use rhwp::document_core::queries::rendering::{PngExportOptions, VlmTarget};

    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-png <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut scale: Option<f64> = None;
    let mut max_dimension: Option<i32> = None;
    let mut vlm_target: Option<VlmTarget> = None;
    let mut dpi: Option<f64> = None;
    // PNG export is print-equivalent output. Editor visuals require an explicit screen profile.
    let mut render_profile = rhwp::paint::RenderProfile::HighQuality;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--profile" => {
                if i + 1 < args.len() {
                    let Some(profile) = rhwp::paint::RenderProfile::parse(&args[i + 1]) else {
                        eprintln!(
                            "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                        );
                        return EXIT_USAGE;
                    };
                    render_profile = profile;
                    i += 2;
                } else {
                    eprintln!("오류: --profile 뒤에 프로필 이름이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(s) if s.is_finite() && s > 0.0 => scale = Some(s),
                        _ => {
                            eprintln!("오류: --scale 값이 올바르지 않습니다 (양수 실수 필요).");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --scale 뒤에 배율 값이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--max-dimension" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<i32>() {
                        Ok(n) if n > 0 => max_dimension = Some(n),
                        _ => {
                            eprintln!(
                                "오류: --max-dimension 값이 올바르지 않습니다 (양수 정수 필요)."
                            );
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --max-dimension 뒤에 픽셀 값이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--dpi" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(d) if d.is_finite() && d > 0.0 => dpi = Some(d),
                        _ => {
                            eprintln!("오류: --dpi 값이 올바르지 않습니다 (양수 실수 필요).");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --dpi 뒤에 DPI 값이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--vlm-target" => {
                if i + 1 < args.len() {
                    match VlmTarget::from_str(&args[i + 1]) {
                        Some(t) => vlm_target = Some(t),
                        None => {
                            eprintln!(
                                "오류: --vlm-target 값이 올바르지 않습니다 (지원: {}).",
                                VlmTarget::all_names()
                            );
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --vlm-target 뒤에 프리셋 이름이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let png_options = PngExportOptions {
        scale,
        max_dimension,
        vlm_target,
        dpi,
        font_paths: font_paths.clone(),
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let mut core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {:?}", e);
            return EXIT_RUNTIME;
        }
    };

    // [#3302] 외부 연결 그림(HWP3 pic_type=0 등)의 같은 디렉터리 자동 적재 — export-svg
    // 의 #741 규칙과 동일. 누락 시 skia 렌더가 회색 placeholder 를 그린다 (SO-SUEOP 1쪽 실측).
    if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
        if let Some(parent) = Path::new(file_path).parent() {
            let _loaded = core.populate_external_images_from_dir(parent);
        }
    }

    let page_count = core.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count as u32 {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count as u32).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let total_pages = pages.len();
    let mut success = 0;
    let mut total_bytes = 0usize;

    for page_num in &pages {
        let has_options = png_options.scale.is_some()
            || png_options.max_dimension.is_some()
            || png_options.vlm_target.is_some()
            || png_options.dpi.is_some()
            || render_profile != rhwp::paint::RenderProfile::Screen;
        let result = if has_options {
            core.render_page_png_native_with_profile_and_export_options(
                *page_num,
                render_profile,
                &png_options,
            )
        } else if !font_paths.is_empty() {
            core.render_page_png_native_with_fonts(*page_num, &font_paths)
        } else {
            core.render_page_png_native(*page_num)
        };
        match result {
            Ok(png_bytes) => {
                let png_filename = if total_pages == 1 {
                    format!("{}.png", file_stem)
                } else {
                    format!("{}_{:03}.png", file_stem, page_num + 1)
                };
                let png_path = output_path.join(&png_filename);
                if let Err(e) = fs::write(&png_path, &png_bytes) {
                    eprintln!("오류: 페이지 {} PNG 저장 실패 - {}", page_num + 1, e);
                    continue;
                }
                println!("  → {} ({} bytes)", png_path.display(), png_bytes.len());
                total_bytes += png_bytes.len();
                success += 1;
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num + 1, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 PNG 파일 → {}/ ({:.1} MB)",
        success,
        output_dir,
        total_bytes as f64 / 1024.0 / 1024.0
    );

    // [#2707] 성공 수 집계는 이미 정확했지만 종료 코드가 항상 0이었다.
    if success == total_pages {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

fn export_pdf(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        print_export_pdf_usage();
        return 2;
    }
    if args[0] == "--help" || args[0] == "-h" {
        print_export_pdf_usage();
        return 0;
    }

    #[cfg(target_arch = "wasm32")]
    {
        eprintln!("오류: PDF 내보내기는 native 빌드에서만 지원됩니다.");
        return 1;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file_path = &args[0];
        let mut output_file = String::new();
        let mut target_page: Option<u32> = None;
        let mut pdf_backend = rhwp::renderer::pdf::PdfBackend::default();
        let mut pdf_options = rhwp::renderer::pdf::PdfExportOptions::default();
        let mut direct_pdf_options = rhwp::renderer::pdf::DirectPdfExportOptions::default();
        let mut render_profile: Option<rhwp::paint::RenderProfile> = None;
        let mut compatibility_only_options = Vec::new();
        let mut direct_raster_dpi_was_set = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        output_file = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
                        return 2;
                    }
                }
                "--page" | "-p" => {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<u32>() {
                            Ok(n) => target_page = Some(n),
                            Err(_) => {
                                eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                                return 2;
                            }
                        }
                        i += 2;
                    } else {
                        eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                        return 2;
                    }
                }
                "--profile" => {
                    if i + 1 < args.len() {
                        render_profile = rhwp::paint::RenderProfile::parse(&args[i + 1]);
                        if render_profile.is_none() {
                            eprintln!(
                                "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                            );
                            return 2;
                        }
                        i += 2;
                    } else {
                        eprintln!("오류: --profile 뒤에 프로필 이름이 필요합니다.");
                        return 2;
                    }
                }
                "--backend" => {
                    if i + 1 < args.len() {
                        let Some(backend) = rhwp::renderer::pdf::PdfBackend::parse(&args[i + 1])
                        else {
                            eprintln!("오류: --backend 값이 올바르지 않습니다 (svg|direct).");
                            return 2;
                        };
                        pdf_backend = backend;
                        i += 2;
                    } else {
                        eprintln!("오류: --backend 뒤에 backend 이름이 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--backend=") => {
                    let Some(backend) = rhwp::renderer::pdf::PdfBackend::parse(
                        arg.trim_start_matches("--backend="),
                    ) else {
                        eprintln!("오류: --backend 값이 올바르지 않습니다 (svg|direct).");
                        return 2;
                    };
                    pdf_backend = backend;
                    i += 1;
                }
                "--raster-dpi" => {
                    if i + 1 < args.len() {
                        let Ok(raster_dpi) = args[i + 1].parse::<f32>() else {
                            eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                            return 2;
                        };
                        if !raster_dpi.is_finite() || raster_dpi <= 0.0 {
                            eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                            return 2;
                        }
                        direct_pdf_options.raster_dpi = raster_dpi;
                        direct_raster_dpi_was_set = true;
                        i += 2;
                    } else {
                        eprintln!("오류: --raster-dpi 뒤에 DPI 값이 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--raster-dpi=") => {
                    let Ok(raster_dpi) = arg.trim_start_matches("--raster-dpi=").parse::<f32>()
                    else {
                        eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                        return 2;
                    };
                    if !raster_dpi.is_finite() || raster_dpi <= 0.0 {
                        eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                        return 2;
                    }
                    direct_pdf_options.raster_dpi = raster_dpi;
                    direct_raster_dpi_was_set = true;
                    i += 1;
                }
                "--font-path" => {
                    if i + 1 < args.len() {
                        pdf_options
                            .font_paths
                            .push(std::path::PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                        return 2;
                    }
                }
                "--fallback-serif" => {
                    compatibility_only_options.push("--fallback-serif");
                    if i + 1 < args.len() {
                        pdf_options.fallback_serif = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("오류: --fallback-serif 뒤에 폰트 family가 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--fallback-serif=") => {
                    compatibility_only_options.push("--fallback-serif");
                    pdf_options.fallback_serif =
                        arg.trim_start_matches("--fallback-serif=").to_string();
                    i += 1;
                }
                "--fallback-sans" | "--fallback-sans-serif" => {
                    compatibility_only_options.push("--fallback-sans");
                    if i + 1 < args.len() {
                        pdf_options.fallback_sans = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("오류: --fallback-sans 뒤에 폰트 family가 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--fallback-sans=")
                    || arg.starts_with("--fallback-sans-serif=") =>
                {
                    compatibility_only_options.push("--fallback-sans");
                    pdf_options.fallback_sans = arg
                        .strip_prefix("--fallback-sans=")
                        .or_else(|| arg.strip_prefix("--fallback-sans-serif="))
                        .unwrap_or_default()
                        .to_string();
                    i += 1;
                }
                "--fallback-mono" | "--fallback-monospace" => {
                    compatibility_only_options.push("--fallback-mono");
                    if i + 1 < args.len() {
                        pdf_options.fallback_mono = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("오류: --fallback-mono 뒤에 폰트 family가 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--fallback-mono=")
                    || arg.starts_with("--fallback-monospace=") =>
                {
                    compatibility_only_options.push("--fallback-mono");
                    pdf_options.fallback_mono = arg
                        .strip_prefix("--fallback-mono=")
                        .or_else(|| arg.strip_prefix("--fallback-monospace="))
                        .unwrap_or_default()
                        .to_string();
                    i += 1;
                }
                // [Task #2264] 텍스트를 PDF 폰트로 임베드하지 않고 path 로 변환한다.
                // 폰트 서브셋 경로를 건너뛰어 메모리를 크게 줄이는 대신,
                // PDF 의 텍스트 선택·검색 기능을 잃는다 (시각적 출력은 동일).
                "--text-as-paths" => {
                    compatibility_only_options.push("--text-as-paths");
                    pdf_options.embed_text = false;
                    i += 1;
                }
                "--equation-font" | "--equation-font-family" => {
                    compatibility_only_options.push("--equation-font");
                    if i + 1 < args.len() {
                        pdf_options.equation_font = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        eprintln!("오류: --equation-font 뒤에 폰트 family가 필요합니다.");
                        return 2;
                    }
                }
                arg if arg.starts_with("--equation-font=")
                    || arg.starts_with("--equation-font-family=") =>
                {
                    compatibility_only_options.push("--equation-font");
                    pdf_options.equation_font = Some(
                        arg.strip_prefix("--equation-font=")
                            .or_else(|| arg.strip_prefix("--equation-font-family="))
                            .unwrap_or_default()
                            .to_string(),
                    );
                    i += 1;
                }
                _ => {
                    eprintln!("알 수 없는 옵션: {}", args[i]);
                    print_export_pdf_usage();
                    return 2;
                }
            }
        }

        compatibility_only_options.sort_unstable();
        compatibility_only_options.dedup();
        if pdf_backend == rhwp::renderer::pdf::PdfBackend::DirectLayer
            && !compatibility_only_options.is_empty()
        {
            eprintln!(
                "오류: direct PDF backend는 다음 SVG 호환 옵션을 지원하지 않습니다: {}",
                compatibility_only_options.join(", ")
            );
            return 2;
        }
        if pdf_backend == rhwp::renderer::pdf::PdfBackend::CompatibilitySvg
            && direct_raster_dpi_was_set
        {
            eprintln!("오류: --raster-dpi는 direct PDF backend에서만 사용할 수 있습니다.");
            return 2;
        }

        // 기본 출력 파일명
        if output_file.is_empty() {
            let stem = Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            output_file = format!("output/{}.pdf", stem);
        }

        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
                return 1;
            }
        };

        let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 문서 파싱 실패 - {}", e);
                return 1;
            }
        };

        // [#3302] 외부 연결 그림 같은 디렉터리 자동 적재 — export-svg/export-png 와 동일 규칙.
        if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
            if let Some(parent) = Path::new(file_path).parent() {
                let _loaded = doc.populate_external_images_from_dir(parent);
            }
        }

        let page_count = doc.page_count();
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
        if page_count == 0 {
            eprintln!("오류: PDF로 내보낼 페이지가 없습니다.");
            return 1;
        }

        // 출력 디렉토리 생성
        if let Some(parent) = Path::new(&output_file).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("오류: 출력 디렉토리를 만들 수 없습니다 - {}", e);
                    return 1;
                }
            }
        }

        // 페이지 범위 결정
        let pages: Vec<u32> = match target_page {
            Some(p) => {
                if p >= page_count {
                    eprintln!(
                        "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                        page_count - 1
                    );
                    return 2;
                }
                vec![p]
            }
            None => (0..page_count).collect(),
        };

        let pdf_result = match pdf_backend {
            rhwp::renderer::pdf::PdfBackend::CompatibilitySvg => match render_profile {
                Some(profile) => doc.render_pages_pdf_native_with_profile_and_options(
                    &pages,
                    profile,
                    &pdf_options,
                ),
                None => doc.render_pages_pdf_native_with_options(&pages, &pdf_options),
            },
            rhwp::renderer::pdf::PdfBackend::DirectLayer => {
                #[cfg(feature = "native-skia")]
                {
                    direct_pdf_options.font_paths = pdf_options.font_paths.clone();
                    doc.render_pages_pdf_direct_native_with_profile_and_options(
                        &pages,
                        render_profile.unwrap_or(rhwp::paint::RenderProfile::Print),
                        &direct_pdf_options,
                    )
                }
                #[cfg(not(feature = "native-skia"))]
                {
                    Err(rhwp::error::HwpError::RenderError(
                        "direct PDF backend requires a build with the native-skia feature"
                            .to_string(),
                    ))
                }
            }
        };
        let pdf_bytes = match pdf_result {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("오류: PDF 변환 실패 - {}", e);
                return 1;
            }
        };
        if let Err(e) = fs::write(&output_file, &pdf_bytes) {
            eprintln!("오류: PDF 저장 실패 - {}", e);
            return 1;
        }
        println!(
            "  → {} ({}KB, {}페이지)",
            output_file,
            pdf_bytes.len() / 1024,
            pages.len()
        );
        if pdf_backend == rhwp::renderer::pdf::PdfBackend::DirectLayer {
            println!("PDF backend: direct");
        }
        println!("PDF 내보내기 완료");
        0
    }
}

fn print_export_pdf_usage() {
    eprintln!("사용법: rhwp export-pdf <파일.hwp|파일.hwpx|파일.hml> [옵션]");
    eprintln!("  -o, --output <파일>       출력 PDF 파일");
    eprintln!("  -p, --page <번호>        특정 페이지만 내보내기 (0부터 시작)");
    eprintln!("      --backend <svg|direct> PDF backend (기본값: svg)");
    eprintln!(
        "      --profile <프로필>   layer 출력 프로필 (screen|print|high-quality|fast-preview)"
    );
    eprintln!("      --raster-dpi <DPI>    direct backend fallback raster DPI (기본값: 144)");
    eprintln!("      --font-path <경로>   폰트 파일 탐색 경로 (여러 번 지정 가능)");
    eprintln!("      --fallback-serif <명>");
    eprintln!("      --fallback-sans <명>");
    eprintln!("      --fallback-mono <명>");
    eprintln!("      --equation-font <명>");
    eprintln!("  direct backend는 native-skia feature로 빌드한 native CLI가 필요합니다.");
    eprintln!("  참고: <...>는 자리표시자이며, 실제 입력에는 꺾쇠괄호를 쓰지 않습니다.");
    eprintln!("        공백 없는 값: --font-path ./ttfs");
    eprintln!(
        "        공백 포함 값은 큰따옴표 권장: --font-path \"./My Fonts\", --fallback-sans \"Apple SD Gothic Neo\""
    );
    eprintln!("        작은따옴표는 zsh/bash/PowerShell에서 literal 값이 필요할 때만 사용합니다.");
}

fn export_text(args: &[String]) -> i32 {
    // [#3237] --json: 결과를 파일 대신 stdout JSON 으로 낸다. stdout 은 순수 JSON 이어야
    // 하므로 이 모드에서는 진행 메시지를 찍지 않는다. 위치 무관 플래그다 (info 와 동일 규약).
    let json_mode = args.iter().any(|a| a == "--json");
    let args: Vec<String> = args
        .iter()
        .filter(|a| a.as_str() != "--json")
        .cloned()
        .collect();
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-text <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count = doc.page_count();
    if !json_mode {
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    }
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return EXIT_RUNTIME;
    }

    let output_path = Path::new(&output_dir);
    if !json_mode && !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // [#3237] JSON 모드: 파일을 쓰지 않고 요청 페이지 전체를 stdout JSON 하나로 낸다.
    if json_mode {
        let mut page_objs = Vec::with_capacity(pages.len());
        for page_num in &pages {
            match doc.extract_page_text_native(*page_num) {
                Ok(text) => {
                    page_objs.push(serde_json::json!({ "page": page_num, "text": text }));
                }
                Err(e) => {
                    eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {}", page_num, e);
                    return EXIT_RUNTIME;
                }
            }
        }
        let result = serde_json::json!({
            "schemaVersion": "1.0",
            "source": file_path,
            "pageCount": page_objs.len(),
            "pages": page_objs,
        });
        println!("{result}");
        return EXIT_OK;
    }

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut written = 0usize;

    for page_num in &pages {
        match doc.extract_page_text_native(*page_num) {
            Ok(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }

                let txt_filename = if page_count == 1 {
                    format!("{}.txt", file_stem)
                } else {
                    format!("{}_{:03}.txt", file_stem, page_num + 1)
                };
                let txt_path = output_path.join(&txt_filename);

                match fs::write(&txt_path, text.as_bytes()) {
                    Ok(_) => {
                        println!("  → {}", txt_path.display());
                        written += 1;
                    }
                    Err(e) => eprintln!("오류: TXT 저장 실패 - {}: {}", txt_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "텍스트 내보내기 완료: {}개 TXT 파일 → {}/",
        written, output_dir
    );

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다.
    if written == pages.len() {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

/// `export-tables` — 표를 격자 JSON 으로 추출 (병합·중첩 보존).
///
/// 평문·Markdown 추출은 병합(rowSpan/colSpan)을 잃어 소비자가 덮인 칸을 별개 열로
/// 오독한다. 본 명령은 `Table.cells`(앵커 셀 + span)를 그대로 직역해 격자를 보존한다.
fn export_tables(args: &[String]) -> i32 {
    use rhwp::document_core::queries::table_extract::extract_tables;

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" | "--out" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다.");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp export-tables <파일.hwp|파일.hwpx> [--json] [-o <출력.json>]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let tables = extract_tables(doc.document());
    let envelope = serde_json::json!({
        "schemaVersion": "1.0",
        "source": file_path,
        "tableCount": tables.len(),
        "tables": tables,
    });

    if let Some(p) = out_path {
        let json = match serde_json::to_string_pretty(&envelope) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("오류: JSON 직렬화 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        };
        return match fs::write(&p, &json) {
            Ok(_) => {
                println!("표 추출 완료: {}개 → {}", tables.len(), p);
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", p, e);
                EXIT_RUNTIME
            }
        };
    }

    if json_mode {
        println!("{envelope}");
        return EXIT_OK;
    }

    // 기본 출력은 사람용 요약 — 기계 소비는 --json 이 담당한다.
    println!("문서 로드: {} (표 {}개)", file_path, tables.len());
    for t in &tables {
        let merged = t
            .cells
            .iter()
            .filter(|c| c.row_span > 1 || c.col_span > 1)
            .count();
        let nested = t.cells.iter().filter(|c| !c.nested.is_empty()).count();
        println!(
            "  표{} [구역{}:문단{}]: {}행×{}열, 셀 {}개 (병합 {}개, 중첩 {}개)",
            t.index, t.section, t.paragraph, t.rows, t.cols, t.cell_count, merged, nested
        );
    }
    EXIT_OK
}

fn export_markdown(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-markdown <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return EXIT_RUNTIME;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let assets_dir_name = format!("{}_assets", file_stem);
    let assets_dir_path = output_path.join(&assets_dir_name);
    let mut written_image_count: usize = 0;
    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 MD 페이지 수를 센다.
    // 이미지 실패는 경고로 남기고 MD 자체는 저장되므로 페이지 실패로 세지 않는다.
    let mut written_page_count = 0usize;

    let mime_to_ext = |mime: &str| -> &'static str {
        match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/webp" => "webp",
            _ => "bin",
        }
    };

    for page_num in &pages {
        match doc.extract_page_markdown_with_images_native(*page_num) {
            Ok((mut markdown, image_refs)) => {
                for (img_idx, (sec_idx, para_idx, control_idx, bin_data_id)) in
                    image_refs.iter().enumerate()
                {
                    let token = format!("[[RHWP_IMAGE:{}]]", img_idx + 1);

                    let try_control = match (sec_idx, para_idx, control_idx) {
                        (Some(si), Some(pi), Some(ci)) => Some((*si, *pi, *ci)),
                        _ => None,
                    };

                    let (mime, image_data) = if let Some((si, pi, ci)) = try_control {
                        match (
                            doc.get_control_image_mime_native(si, pi, &[], ci),
                            doc.get_control_image_data_native(si, pi, &[], ci),
                        ) {
                            (Ok(m), Ok(d)) => (m, d),
                            _ => {
                                if *bin_data_id == 0 {
                                    eprintln!(
                                        "경고: 페이지 {} 이미지 추출 실패 (s{} p{} c{}), fallback bin_data_id 없음",
                                        page_num, si, pi, ci
                                    );
                                    markdown = markdown.replace(&token, "");
                                    continue;
                                }
                                let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id)
                                {
                                    Ok(m) => m,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id)
                                {
                                    Ok(d) => d,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                (fb_mime, fb_data)
                            }
                        }
                    } else {
                        if *bin_data_id == 0 {
                            eprintln!(
                                "경고: 페이지 {} 이미지 추출 실패 (문서 좌표 없음, bin_data_id=0)",
                                page_num
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                        let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id) {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        (fb_mime, fb_data)
                    };

                    if !assets_dir_path.exists() {
                        if let Err(e) = fs::create_dir_all(&assets_dir_path) {
                            eprintln!(
                                "오류: 이미지 출력 폴더 생성 실패 - {}: {}",
                                assets_dir_path.display(),
                                e
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                    }

                    let ext = mime_to_ext(&mime);
                    let image_filename = format!(
                        "{}_p{:03}_img{:03}.{}",
                        file_stem,
                        page_num + 1,
                        img_idx + 1,
                        ext
                    );
                    let image_path = assets_dir_path.join(&image_filename);

                    if let Err(e) = fs::write(&image_path, &image_data) {
                        eprintln!("경고: 이미지 저장 실패 - {}: {}", image_path.display(), e);
                        markdown = markdown.replace(&token, "");
                        continue;
                    }

                    let image_link = format!(
                        "![image {}]({}/{})",
                        img_idx + 1,
                        assets_dir_name,
                        image_filename
                    );
                    markdown = markdown.replace(&token, &image_link);
                    written_image_count += 1;
                }

                if !markdown.ends_with('\n') {
                    markdown.push('\n');
                }

                let md_filename = if page_count == 1 {
                    format!("{}.md", file_stem)
                } else {
                    format!("{}_{:03}.md", file_stem, page_num + 1)
                };
                let md_path = output_path.join(&md_filename);

                match fs::write(&md_path, markdown.as_bytes()) {
                    Ok(_) => {
                        println!("  → {}", md_path.display());
                        written_page_count += 1;
                    }
                    Err(e) => eprintln!("오류: Markdown 저장 실패 - {}: {}", md_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} Markdown 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    if written_image_count > 0 {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일, {}개 이미지 → {}/",
            written_page_count, written_image_count, output_dir
        );
    } else {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일 → {}/",
            written_page_count, output_dir
        );
    }

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다.
    if written_page_count == pages.len() {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

/// [#3238] batch — 파일 목록을 stdin(한 줄당 하나)으로 받아 한 프로세스에서 전건 처리하고
/// NDJSON 스트림을 stdout 으로 낸다. 건별 실패는 `error` 레코드로 스트림을 계속하되,
/// 하나라도 실패하면 [#2707] 계약대로 종료 코드 1 로 끝난다.
fn run_batch(args: &[String]) -> i32 {
    use std::io::{BufRead, Write};

    const USAGE: &str = "사용법: <파일 목록> | rhwp batch <export-text|info|export-structure> --json [--mode auto|outline|clause] [--threads <N>]  (stdin: 한 줄당 파일 경로 하나)";

    let subcommand = args.first().map(String::as_str);
    let is_structure = subcommand == Some("export-structure");
    if !matches!(
        subcommand,
        Some("export-text") | Some("info") | Some("export-structure")
    ) {
        match subcommand {
            Some(unknown) => eprintln!(
                "오류: batch 는 현재 export-text·info·export-structure 만 지원합니다 - {}",
                unknown
            ),
            None => eprintln!("오류: batch 서브커맨드를 지정해주세요."),
        }
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let mut json_mode = false;
    let mut threads_opt: Option<usize> = None;
    let mut structure_mode = rhwp::document_core::queries::structure::StructureMode::Auto;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--mode" => {
                // [#3261] --mode 는 export-structure 축 전용이다.
                if !is_structure {
                    eprintln!("오류: --mode 는 export-structure 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --mode 뒤에 auto|outline|clause 가 필요합니다.");
                    return EXIT_USAGE;
                };
                match rhwp::document_core::queries::structure::StructureMode::parse(value) {
                    Some(m) => structure_mode = m,
                    None => {
                        eprintln!("오류: --mode 는 auto|outline|clause - {}", value);
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--threads" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --threads 뒤에 스레드 수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match value.parse::<usize>() {
                    Ok(n) if n >= 1 => threads_opt = Some(n),
                    _ => {
                        eprintln!("오류: 스레드 수가 올바르지 않습니다 - {}", value);
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            other => {
                eprintln!("알 수 없는 옵션: {}", other);
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
        }
    }
    if !json_mode {
        eprintln!("오류: batch 는 현재 --json 출력만 지원합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let mode = match subcommand {
        Some("export-text") => BatchMode::ExportText,
        Some("info") => BatchMode::Info,
        _ => BatchMode::Structure(structure_mode),
    };

    let stdin = std::io::stdin();
    let mut paths: Vec<String> = Vec::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let path = l.trim().to_string();
                if !path.is_empty() {
                    paths.push(path);
                }
            }
            Err(e) => {
                eprintln!("오류: stdin 읽기 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        }
    }

    let threads = threads_opt
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
        .max(1);

    let started = std::time::Instant::now();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    // 파일 간 병렬 처리 + 한계 재정렬 버퍼(bounded reorder buffer) 스트리밍.
    //
    // 배리어 없이 완전 병렬로 돌리되, 완료 레코드는 stdin 입력 순서대로 즉시 방출한다.
    // 완료-미방출 레코드가 cap 을 넘으면 워커가 대기(역압)해 메모리를 상한한다.
    // 단, 방출 차례(next_emit) 레코드는 cap 과 무관하게 넣을 수 있어야 교착이 없다 —
    // 느린 파일 하나가 버퍼를 채워도, 그 파일이 곧 방출 차례이므로 항상 전진한다.
    let n = paths.len();
    let cap = threads.saturating_mul(8).max(1);
    let next_claim = std::sync::atomic::AtomicUsize::new(0);
    let abort = std::sync::atomic::AtomicBool::new(false);
    let buf: std::sync::Mutex<std::collections::HashMap<usize, serde_json::Value>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
    let next_emit = std::sync::atomic::AtomicUsize::new(0);
    let space = std::sync::Condvar::new(); // 버퍼에 자리가 났다
    let ready = std::sync::Condvar::new(); // 방출 차례 레코드가 도착했다

    let (failed, emitted) = std::thread::scope(|scope| {
        for _ in 0..threads.min(n) {
            scope.spawn(|| loop {
                if abort.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let idx = next_claim.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if idx >= n {
                    break;
                }
                let record = batch_record(mode, &paths[idx]);
                let mut guard = buf.lock().expect("batch buf lock");
                while guard.len() >= cap
                    && idx != next_emit.load(std::sync::atomic::Ordering::Relaxed)
                    && !abort.load(std::sync::atomic::Ordering::Relaxed)
                {
                    guard = space.wait(guard).expect("batch buf lock");
                }
                if abort.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                guard.insert(idx, record);
                // 방출자는 하나뿐이므로 notify_one 으로 충분하다.
                ready.notify_one();
            });
        }

        // 방출자(현재 스레드): 입력 순서대로 도착 즉시 방출한다. 도착해 있는 연속
        // 레코드는 한 번의 락으로 일괄 드레인하고 notify 도 배치당 1회만 보낸다 —
        // 레코드당 notify_all 은 대기 워커 전원을 헛깨우는 thundering herd 가 된다
        // (271건 실측에서 방출 버스트 구간 수 초 손실).
        let mut failed = 0usize;
        let mut emitted = 0usize;
        let mut drained: Vec<serde_json::Value> = Vec::new();
        'emit: while emitted < n {
            drained.clear();
            {
                let mut guard = buf.lock().expect("batch buf lock");
                while guard.get(&emitted).is_none() {
                    guard = ready.wait(guard).expect("batch buf lock");
                }
                while let Some(record) = guard.remove(&emitted) {
                    emitted += 1;
                    drained.push(record);
                }
                next_emit.store(emitted, std::sync::atomic::Ordering::Relaxed);
            }
            space.notify_all();
            for record in &drained {
                if record.get("error").is_some() {
                    failed += 1;
                }
                if let Err(e) = writeln!(out, "{record}") {
                    // 파이프 소비자가 끊은 경우(broken pipe 등): 새 작업 수주를 멈추고
                    // 대기 중인 워커를 전부 깨워 정리한다.
                    eprintln!("오류: stdout 쓰기 실패 - {}", e);
                    abort.store(true, std::sync::atomic::Ordering::Relaxed);
                    space.notify_all();
                    break 'emit;
                }
            }
        }
        (failed, emitted)
    });

    if abort.load(std::sync::atomic::Ordering::Relaxed) {
        return EXIT_RUNTIME;
    }
    if let Err(e) = out.flush() {
        eprintln!("오류: stdout 쓰기 실패 - {}", e);
        return EXIT_RUNTIME;
    }

    eprintln!(
        "batch: {}건 중 {} 성공, {} 실패 ({}ms, threads={})",
        emitted,
        emitted - failed,
        failed,
        started.elapsed().as_millis(),
        threads
    );
    if failed > 0 {
        EXIT_RUNTIME
    } else {
        EXIT_OK
    }
}

/// [#3238] batch 가 처리하는 서브커맨드 축.
#[derive(Clone, Copy)]
enum BatchMode {
    ExportText,
    Info,
    /// [#3261] 문서 개요/조문 구조 — `export-structure --json` 과 스키마 공유.
    Structure(rhwp::document_core::queries::structure::StructureMode),
}

/// [#3238] 파일 하나를 처리해 NDJSON 레코드 하나를 만든다. 실패는 레코드로 보고하고
/// 스트림은 계속된다 — 프로세스 중단 없이 부분 실패를 종료 코드로 신호하기 위함.
///
/// 배치는 신뢰할 수 없는 대량 코퍼스를 훑는 용도라, 한 건의 파서 panic 이 배치 전체를
/// 죽여서는 안 된다. panic 도 해당 파일의 `error` 레코드로 격리한다.
fn batch_record(mode: BatchMode, path: &str) -> serde_json::Value {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match mode {
        BatchMode::ExportText => batch_export_text_record_inner(path),
        BatchMode::Info => batch_info_record_inner(path),
        BatchMode::Structure(structure_mode) => batch_structure_record_inner(path, structure_mode),
    })) {
        Ok(record) => record,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "원인 불명".to_string());
            batch_fail_record(path, format!("내부 오류(panic): {}", message))
        }
    }
}

fn batch_fail_record(path: &str, message: String) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "1.0",
        "source": path,
        "error": message,
        "exitClass": "runtime",
    })
}

fn batch_export_text_record_inner(path: &str) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파싱 실패: {}", e)),
    };

    let page_count = doc.page_count();
    let mut text = String::new();
    for page_num in 0..page_count {
        match doc.extract_page_text_native(page_num) {
            Ok(t) => {
                text.push_str(&t);
                if !t.ends_with('\n') {
                    text.push('\n');
                }
            }
            Err(e) => {
                return batch_fail_record(
                    path,
                    format!("페이지 {} 텍스트 추출 실패: {}", page_num, e),
                )
            }
        }
    }

    serde_json::json!({
        "schemaVersion": "1.0",
        "source": path,
        "pageCount": page_count,
        "text": text,
    })
}

/// [#3261] `batch export-structure --json` 의 파일당 레코드 — `export-structure --json`
/// 봉투(`structure_json_value` 공유)와 같은 스키마다.
fn batch_structure_record_inner(
    path: &str,
    mode: rhwp::document_core::queries::structure::StructureMode,
) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파싱 실패: {}", e)),
    };
    let st = rhwp::document_core::queries::structure::build_structure(doc.document(), mode);
    structure_json_value(path, &st)
}

/// [#3238] `batch info --json` 의 파일당 레코드 — `info --json` 과 같은 스키마
/// (`info_json_value` 공유)라 소비자가 단건/배치를 같은 코드로 읽는다.
fn batch_info_record_inner(path: &str) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let file_size = data.len();
    let detected_format = rhwp::parser::detect_format(&data);
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return batch_fail_record(path, format!("파싱 실패: {}", e)),
    };
    info_json_value(path, file_size, detected_format, &doc)
}

/// [#3261] `export-structure --json`·`batch export-structure --json` 이 공유하는
/// 구조 봉투 레코드. `mode`/`nodeCount` 를 톱레벨로 올려 스윕 선별(jq select)이 싸다.
fn structure_json_value(
    file_path: &str,
    st: &rhwp::document_core::queries::structure::StructureDoc,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "1.0",
        "source": file_path,
        "mode": st.mode,
        "nodeCount": st.node_count,
        "structure": st,
    })
}

/// [#3237] `info --json`·`batch info --json` 이 공유하는 문서 메타 JSON 레코드.
/// `schemaVersion` 이 계약이며 필드 추가는 허용, 변경·삭제는 계약 테스트가 잡는다.
fn info_json_value(
    file_path: &str,
    file_size: usize,
    detected_format: rhwp::parser::FileFormat,
    doc: &rhwp::wasm_api::HwpDocument,
) -> serde_json::Value {
    let document = doc.document();
    let format_str = match detected_format {
        rhwp::parser::FileFormat::Hwp => "hwp5",
        rhwp::parser::FileFormat::Hwpx => "hwpx",
        rhwp::parser::FileFormat::Hwp3 => "hwp3",
        rhwp::parser::FileFormat::Hml => "hml",
        // 파싱이 성공한 뒤에는 도달하지 않지만, 계약상 문자열은 고정해 둔다.
        rhwp::parser::FileFormat::DrmProtected => "drm-protected",
        rhwp::parser::FileFormat::Empty => "empty",
        rhwp::parser::FileFormat::Unknown => "unknown",
    };
    let version = if detected_format == rhwp::parser::FileFormat::Hml {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(format!(
            "{}.{}.{}.{}",
            document.header.version.major,
            document.header.version.minor,
            document.header.version.build,
            document.header.version.revision,
        ))
    };
    let fonts: Vec<String> = document
        .doc_info
        .font_faces
        .first()
        .map(|faces| faces.iter().map(|f| f.name.clone()).collect())
        .unwrap_or_default();
    let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    serde_json::json!({
        "schemaVersion": "1.0",
        "source": file_path,
        "format": format_str,
        "sizeBytes": file_size,
        "version": version,
        "sections": document.sections.len(),
        "pageCount": doc.page_count(),
        "paraCount": para_count,
        "fonts": fonts,
    })
}

fn show_info(args: &[String]) -> i32 {
    // [#3237] --json은 위치와 무관하다. 단일 입력 명령이므로 추가 경로를 무시하지 않는다.
    let mut json_mode = false;
    let mut file_path: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
    }
    let Some(file_path) = file_path else {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        return EXIT_USAGE;
    };

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let file_size = data.len();
    let detected_format = rhwp::parser::detect_format(&data);

    // 문서 파싱
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 문서 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let document = doc.document();

    // [#3237] JSON 모드: 핵심 메타를 stdout JSON 하나로 낸다. `schemaVersion` 이 계약이며
    // 필드 추가는 허용, 기존 필드 변경·삭제는 `tests/cli_json_contract.rs` 가 잡는다.
    if json_mode {
        let info = info_json_value(file_path, file_size, detected_format, &doc);
        println!("{info}");
        return EXIT_OK;
    }

    if detected_format == rhwp::parser::FileFormat::Hml {
        println!("format: HML");
        println!(
            "hwpml_version: {}",
            document
                .doc_info
                .hwpml_version
                .as_deref()
                .unwrap_or("unknown")
        );
        println!("sections: {}", document.sections.len());
        println!("pages: {}", doc.page_count());
        if let Some(metadata) = doc.hml_metadata() {
            let encoding = match metadata.encoding {
                rhwp::parser::hml::HmlEncoding::Utf8 => "UTF-8",
                rhwp::parser::hml::HmlEncoding::Utf16Le => "UTF-16LE",
                rhwp::parser::hml::HmlEncoding::Utf16Be => "UTF-16BE",
            };
            println!("encoding: {encoding}");
            println!("resources: {}", metadata.resource_count);
            println!("warnings: {}", metadata.warnings.len());
            for warning in &metadata.warnings {
                eprintln!(
                    "warning [{:?}] {}: {}",
                    warning.code, warning.xml_path, warning.message
                );
            }
        }
    }

    println!("파일: {}", file_path);
    println!("크기: {} bytes", file_size);
    if detected_format != rhwp::parser::FileFormat::Hml {
        println!(
            "버전: {}.{}.{}.{}",
            document.header.version.major,
            document.header.version.minor,
            document.header.version.build,
            document.header.version.revision,
        );
        println!(
            "압축: {}",
            if document.header.compressed {
                "예"
            } else {
                "아니오"
            }
        );
        println!(
            "암호화: {}",
            if document.header.encrypted {
                "예"
            } else {
                "아니오"
            }
        );
        println!(
            "배포용: {}",
            if document.header.distribution {
                "예"
            } else {
                "아니오"
            }
        );
    }
    println!("구역 수: {}", document.sections.len());
    println!("페이지 수: {}", doc.page_count());

    // 용지 정보
    for (sec_idx, section) in document.sections.iter().enumerate() {
        let page_def = &section.section_def.page_def;
        let orientation = if page_def.landscape {
            "가로"
        } else {
            "세로"
        };
        println!(
            "구역{} 용지: {}×{} HWPUNIT, 방향={} (여백: 좌{} 우{} 상{} 하{})",
            sec_idx,
            page_def.width,
            page_def.height,
            orientation,
            page_def.margin_left,
            page_def.margin_right,
            page_def.margin_top,
            page_def.margin_bottom,
        );
        println!(
            "  머리말여백={} 꼬리말여백={} 제본여백={}",
            page_def.margin_header, page_def.margin_footer, page_def.margin_gutter
        );
        if section.section_def.hide_empty_line {
            println!("  빈 줄 감추기: 활성");
        }
    }

    // 폰트 목록
    let lang_names = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];
    for (i, fonts) in document.doc_info.font_faces.iter().enumerate() {
        if !fonts.is_empty() {
            let name = if i < lang_names.len() {
                lang_names[i]
            } else {
                "기타"
            };
            let font_names: Vec<String> = fonts
                .iter()
                .enumerate()
                .map(|(idx, f)| format!("[{}]{}", idx, f.name))
                .collect();
            println!("폰트({}): {}", name, font_names.join(", "));
        }
    }

    // 스타일 목록
    if !document.doc_info.styles.is_empty() {
        let style_names: Vec<&str> = document
            .doc_info
            .styles
            .iter()
            .map(|s| s.local_name.as_str())
            .collect();
        println!("스타일: {}", style_names.join(", "));
    }

    // 문단 통계
    let total_paras: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    println!("총 문단 수: {}", total_paras);

    // [Task #554] HWP3 → HWP5 변환본 식별 휴리스틱 정보
    // 한컴이 HWP3 → HWP5 변환 시 ParaShape/CharShape 를 거의 재사용하지 않고 매우 적은
    // 수만 생성한다. 직접 작성본은 작성자가 다양한 스타일을 사용하므로 비율이 paragraph
    // 와 비슷하거나 더 높다. 임계값 < 0.05 / < 0.15 로 27 fixture 100% 분류 (Stage 1).
    let ps_count = document.doc_info.para_shapes.len();
    let cs_count = document.doc_info.char_shapes.len();
    if total_paras > 0 {
        let ps_ratio = ps_count as f64 / total_paras as f64;
        let cs_ratio = cs_count as f64 / total_paras as f64;
        let origin = if total_paras > 50 && ps_ratio < 0.05 && cs_ratio < 0.15 {
            "HWP3 변환본 추정 (margin_bottom -1600 HU 보정 적용)"
        } else if total_paras <= 50 {
            "판정 불가 (문단 수 ≤ 50, 비율 왜곡 회피)"
        } else {
            "한컴 한글 직접 작성 추정"
        };
        println!("ParaShape: {} (PS/문단 = {:.3})", ps_count, ps_ratio);
        println!("CharShape: {} (CS/문단 = {:.3})", cs_count, cs_ratio);
        println!("Origin 추정: {}", origin);
    }

    // BinData 정보
    if !document.doc_info.bin_data_list.is_empty() {
        println!("BinData:");
        for (idx, bd) in document.doc_info.bin_data_list.iter().enumerate() {
            let type_str = match bd.data_type {
                rhwp::model::bin_data::BinDataType::Link => "Link",
                rhwp::model::bin_data::BinDataType::Embedding => "Embedding",
                rhwp::model::bin_data::BinDataType::Storage => "Storage",
            };
            let ext = bd.extension.as_deref().unwrap_or("?");
            // 로드된 데이터 크기 확인
            let loaded_size = document
                .bin_data_content
                .iter()
                .find(|c| c.id == bd.storage_id)
                .map(|c| c.data.len())
                .unwrap_or(0);
            println!(
                "  [{}] {} (ID: {}, ext: {}, loaded: {} bytes)",
                idx, type_str, bd.storage_id, ext, loaded_size
            );
        }
    }

    // 테이블 및 그림 정보
    use rhwp::model::control::Control;
    let mut table_idx = 0;
    let mut picture_idx = 0;

    fn count_pictures(ctrl: &Control, picture_idx: &mut usize, location: &str) {
        match ctrl {
            Control::Picture(pic) => {
                *picture_idx += 1;
                println!(
                    "그림{} [{}]: bin_data_id={}, size={}×{}",
                    *picture_idx,
                    location,
                    pic.image_attr.bin_data_id,
                    pic.common.width,
                    pic.common.height,
                );
            }
            Control::Table(table) => {
                // 표 내부 셀의 문단에서도 그림 검색
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                        for cc in &cp.controls {
                            let loc = format!("{}→셀{}:문단{}", location, cell_idx, cp_idx);
                            count_pictures(cc, picture_idx, &loc);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                let location = format!("구역{}:문단{}", sec_idx, para_idx);
                match ctrl {
                    Control::Table(table) => {
                        table_idx += 1;
                        let page_break_str = match table.page_break {
                            rhwp::model::table::TablePageBreak::None => "나누지 않음",
                            rhwp::model::table::TablePageBreak::CellBreak => "셀 단위 나눔",
                            rhwp::model::table::TablePageBreak::RowBreak => "나눔(행 단위)",
                        };
                        println!(
                            "표{} [{}]: {}행×{}열, 셀 {}개, 쪽나눔={} (attr=0x{:08x}), 제목반복={}",
                            table_idx,
                            location,
                            table.row_count,
                            table.col_count,
                            table.cells.len(),
                            page_break_str,
                            table.raw_table_record_attr,
                            table.repeat_header,
                        );
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Picture(_) => {
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Shape(shape) => {
                        use rhwp::model::shape::ShapeObject;
                        let s = shape.as_ref();
                        let shape_type = s.shape_name();
                        let common = s.common();
                        let border_info = match shape.as_ref() {
                            ShapeObject::Rectangle(r) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                r.drawing.border_line.color,
                                r.drawing.border_line.width,
                                r.drawing.border_line.attr,
                            ),
                            ShapeObject::Line(l) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                l.drawing.border_line.color,
                                l.drawing.border_line.width,
                                l.drawing.border_line.attr,
                            ),
                            _ => String::new(),
                        };
                        println!(
                            "도형 [{}]: {}, size={}×{}, treat_as_char={}{}",
                            location,
                            shape_type,
                            common.width,
                            common.height,
                            common.treat_as_char,
                            border_info,
                        );
                        // 그룹 자식 상세 정보
                        if let ShapeObject::Group(g) = shape.as_ref() {
                            for (i, child) in g.children.iter().enumerate() {
                                let ctype = child.shape_name();
                                let cattr = child.shape_attr();
                                let eff_w = (cattr.current_width as f64 * cattr.render_sx) as i32;
                                let eff_h = (cattr.current_height as f64 * cattr.render_sy) as i32;
                                println!("  자식[{}]: {}, orig={}×{}, scale=({:.3},{:.3}), eff={}×{} at ({:.0},{:.0})",
                                    i, ctype,
                                    cattr.current_width, cattr.current_height,
                                    cattr.render_sx, cattr.render_sy,
                                    eff_w, eff_h,
                                    cattr.render_tx, cattr.render_ty);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    EXIT_OK
}

/// HWPUNIT(u32)을 mm로 변환
fn hu_to_mm(hu: u32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// HWPUNIT(i32)을 mm로 변환
fn hu_to_mm_i(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

fn dump_note_shape(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-note-shape <파일.hwp|파일.hwpx>");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let sections: Vec<serde_json::Value> = doc
        .document()
        .sections
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            serde_json::json!({
                "section": idx,
                "footnoteShape": note_shape_json(&section.section_def.footnote_shape),
                "endnoteShape": note_shape_json(&section.section_def.endnote_shape),
            })
        })
        .collect();

    let value = serde_json::json!({
        "file": file_path,
        "sections": sections,
    });
    match serde_json::to_string_pretty(&value) {
        Ok(text) => {
            println!("{}", text);
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: JSON 생성 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

fn note_shape_json(shape: &rhwp::model::footnote::FootnoteShape) -> serde_json::Value {
    serde_json::json!({
        "raw": {
            "attr": shape.attr,
            "numberFormat": format!("{:?}", shape.number_format),
            "userChar": shape.user_char.to_string(),
            "prefixChar": shape.prefix_char.to_string(),
            "suffixChar": shape.suffix_char.to_string(),
            "startNumber": shape.start_number,
            "separatorLength": hu_json(shape.separator_length as i32),
            "separatorMarginTop": hu_json(shape.separator_margin_top as i32),
            "separatorMarginBottom": hu_json(shape.separator_margin_bottom as i32),
            "noteSpacing": hu_json(shape.note_spacing as i32),
            "separatorLineType": shape.separator_line_type,
            "separatorLineWidth": shape.separator_line_width,
            "separatorColor": format!("0x{:08x}", shape.separator_color),
            "numbering": format!("{:?}", shape.numbering),
            "placement": format!("{:?}", shape.placement),
            "numberCodeSuperscript": shape.number_code_superscript,
            "printInlineAfterText": shape.print_inline_after_text,
            "rawUnknown": hu_json(shape.raw_unknown as i32),
        },
        "ui": {
            "separatorAbove": hu_json(shape.separator_above_margin_hu() as i32),
            "separatorBelow": hu_json(shape.separator_below_margin_hu() as i32),
            "betweenNotes": hu_json(shape.between_notes_margin_hu() as i32),
        },
    })
}

fn hu_json(hu: i32) -> serde_json::Value {
    serde_json::json!({
        "hu": hu,
        "mm": rounded_mm(hu),
    })
}

fn rounded_mm(hu: i32) -> f64 {
    (hu_to_mm_i(hu) * 1000.0).round() / 1000.0
}

fn dump_pages(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-pages <파일.hwp> [-p <페이지번호>]");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut target_page: Option<u32> = None;
    let mut respect_vpos_reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    // 형제 명령(export_svg/export_png/export_text)과 동일하게 파싱 실패를
                    // 오류로 처리한다. 종전 `.parse().ok()` 는 잘못된 인자를 조용히 삼켜
                    // 한 쪽만 요청했는데 문서 전체를 덤프했다.
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다: {}", args[i + 1]);
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: {} 뒤에 페이지 번호가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                }
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();

    // 형제 명령(export_svg)과 동일한 범위 검사. 종전엔 검사가 없어 -p 999 가
    // 아무것도 매칭하지 않은 빈 출력을 내, 잘못된 인자가 아니라 "쪽이 없는 문서"
    // 처럼 보였다.
    if let Some(p) = target_page {
        if p >= page_count {
            eprintln!(
                "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                page_count.saturating_sub(1)
            );
            return EXIT_USAGE;
        }
    }

    println!("문서 로드: {} ({}페이지)", file_path, page_count);
    print!("{}", doc.dump_page_items(target_page));
    EXIT_OK
}

fn dump_endnote_lines(args: &[String]) -> i32 {
    if args.len() < 4 {
        eprintln!(
            "사용법: rhwp dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let section_idx = match args[1].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: section 인덱스 파싱 실패 - {}", e);
            return EXIT_USAGE;
        }
    };
    let para_idx = match args[2].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: para 인덱스 파싱 실패 - {}", e);
            return EXIT_USAGE;
        }
    };
    let control_idx = match args[3].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: control 인덱스 파싱 실패 - {}", e);
            return EXIT_USAGE;
        }
    };
    let target_note_para = if args.len() >= 5 {
        match args[4].parse::<usize>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("오류: note-para 인덱스 파싱 실패 - {}", e);
                return EXIT_USAGE;
            }
        }
    } else {
        None
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let document = doc.document();
    let Some(section) = document.sections.get(section_idx) else {
        eprintln!("오류: section {} 범위 초과", section_idx);
        return EXIT_USAGE;
    };
    let Some(source_para) = section.paragraphs.get(para_idx) else {
        eprintln!("오류: para {} 범위 초과", para_idx);
        return EXIT_USAGE;
    };
    let Some(ctrl) = source_para.controls.get(control_idx) else {
        eprintln!("오류: control {} 범위 초과", control_idx);
        return EXIT_USAGE;
    };

    let rhwp::model::control::Control::Endnote(endnote) = ctrl else {
        eprintln!(
            "오류: s{}:p{}:ci{} 는 미주가 아닙니다 ({})",
            section_idx,
            para_idx,
            control_idx,
            control_kind(ctrl)
        );
        return EXIT_USAGE;
    };

    println!(
        "문서: {} source=s{}:p{}:ci{} endnote_no={} note_paras={}",
        file_path,
        section_idx,
        para_idx,
        control_idx,
        endnote.number,
        endnote.paragraphs.len()
    );
    println!("source_text={}", brief_text(&source_para.text, 120));
    println!(
        "source_control_positions={}",
        format_control_positions(source_para)
    );

    for (note_para_idx, para) in endnote.paragraphs.iter().enumerate() {
        if target_note_para.is_some_and(|target| target != note_para_idx) {
            continue;
        }
        println!(
            "\n-- note_para={} source=s{}:p{}:ci{}:note{} --",
            note_para_idx, section_idx, para_idx, control_idx, note_para_idx
        );
        dump_paragraph_line_trace(para);
    }
    EXIT_OK
}

fn dump_paragraph_line_trace(para: &rhwp::model::paragraph::Paragraph) {
    use rhwp::model::control::Control;

    let composed = rhwp::renderer::composer::compose_paragraph(para);
    let control_positions = para.control_text_positions();

    println!(
        "para text_len={} char_count={} controls={} line_segs={} char_offsets={} text={}",
        para.text.chars().count(),
        para.char_count,
        para.controls.len(),
        para.line_segs.len(),
        format_u32_list(&para.char_offsets),
        brief_text(&para.text, 160)
    );
    for (i, seg) in para.line_segs.iter().enumerate() {
        println!(
            "  line_seg[{i}] ts={} char={} vpos={} lh={} th={} bl={} gap={} cs={} sw={} tag=0x{:08x}",
            seg.text_start,
            para.utf16_pos_to_char_idx(seg.text_start),
            seg.vertical_pos,
            seg.line_height,
            seg.text_height,
            seg.baseline_distance,
            seg.line_spacing,
            seg.column_start,
            seg.segment_width,
            seg.tag
        );
    }

    if para.controls.is_empty() {
        println!("  controls=[]");
    } else {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            let pos = control_positions.get(ci).copied().unwrap_or(usize::MAX);
            match ctrl {
                Control::Equation(eq) => println!(
                    "  control[{ci}] kind=Equation pos={} tac=true size={}x{} font={} baseline={} script={}",
                    pos,
                    eq.common.width,
                    eq.common.height,
                    eq.font_size,
                    eq.baseline,
                    brief_text(&eq.script, 100)
                ),
                Control::Picture(pic) => println!(
                    "  control[{ci}] kind=Picture pos={} tac={} size={}x{}",
                    pos, pic.common.treat_as_char, pic.common.width, pic.common.height
                ),
                Control::Shape(shape) => {
                    let common = shape.common();
                    println!(
                        "  control[{ci}] kind=Shape pos={} tac={} size={}x{}",
                        pos, common.treat_as_char, common.width, common.height
                    );
                }
                Control::Table(table) => println!(
                    "  control[{ci}] kind=Table pos={} tac={} rows={} cols={}",
                    pos,
                    table.common.treat_as_char,
                    table.row_count,
                    table.col_count
                ),
                other => println!(
                    "  control[{ci}] kind={} pos={} tac=false",
                    control_kind(other),
                    pos
                ),
            }
        }
    }

    println!("  composed_lines={}", composed.lines.len());
    for (li, line) in composed.lines.iter().enumerate() {
        let next_start = composed
            .lines
            .get(li + 1)
            .map(|next| next.char_start)
            .unwrap_or_else(|| {
                line.char_start
                    + line
                        .runs
                        .iter()
                        .map(|run| run.text.chars().count())
                        .sum::<usize>()
                    + usize::from(line.has_line_break)
            });
        println!(
            "    line[{li}] char={}..{} runs={} break={} lh={} bl={} gap={} cs={} sw={} layout_tacs={}",
            line.char_start,
            next_start,
            format_runs(&line.runs),
            line.has_line_break,
            line.line_height,
            line.baseline_distance,
            line.line_spacing,
            line.column_start,
            line.segment_width,
            format_layout_tac_hits(&composed, li)
        );
    }

    if composed.tac_controls.is_empty() {
        println!("  tac_controls=[]");
    } else {
        println!("  tac_controls:");
        for (pos, width_hu, ci) in &composed.tac_controls {
            let line_hits = composed
                .lines
                .iter()
                .enumerate()
                .filter_map(|(li, line)| {
                    let start = line.char_start;
                    let end = composed
                        .lines
                        .get(li + 1)
                        .map(|next| next.char_start)
                        .unwrap_or_else(|| {
                            line.char_start
                                + line
                                    .runs
                                    .iter()
                                    .map(|run| run.text.chars().count())
                                    .sum::<usize>()
                                + usize::from(line.has_line_break)
                        });
                    if if end > start {
                        *pos >= start && *pos < end
                    } else {
                        *pos == start
                    } {
                        Some(li.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "    tac ci={} pos={} width={} strict_line_candidates=[{}]",
                ci, pos, width_hu, line_hits
            );
        }
    }
}

fn format_layout_tac_hits(
    composed: &rhwp::renderer::composer::ComposedParagraph,
    line_idx: usize,
) -> String {
    let Some(line) = composed.lines.get(line_idx) else {
        return "[]".to_string();
    };
    if composed.tac_controls.is_empty() {
        return "[]".to_string();
    }

    let mut hits = Vec::new();
    if line.runs.is_empty() {
        let start = line.char_start;
        let end = composed
            .lines
            .get(line_idx + 1)
            .map(|next| next.char_start)
            .unwrap_or(usize::MAX);
        for (pos, _, ci) in &composed.tac_controls {
            if *pos >= start && *pos < end {
                hits.push(format!("ci{}@{}:empty", ci, pos));
            }
        }
    } else {
        let mut run_start = line.char_start;
        for (run_idx, run) in line.runs.iter().enumerate() {
            let run_len = run.text.chars().count();
            let run_end = run_start + run_len;
            let next_line_starts_at_run_end = composed
                .lines
                .get(line_idx + 1)
                .is_some_and(|next| next.char_start == run_end);
            let allow_end = run_idx == line.runs.len() - 1 && !next_line_starts_at_run_end;
            for (pos, _, ci) in &composed.tac_controls {
                if *pos >= run_start && (*pos < run_end || (allow_end && *pos == run_end)) {
                    hits.push(format!(
                        "ci{}@{}:run{}+{}",
                        ci,
                        pos,
                        run_idx,
                        pos.saturating_sub(run_start)
                    ));
                }
            }
            run_start = run_end;
        }
    }

    if hits.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", hits.join(","))
    }
}

fn control_kind(ctrl: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match ctrl {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::AutoNumber(_) => "AutoNumber",
        Control::NewNumber(_) => "NewNumber",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::Bookmark(_) => "Bookmark",
        Control::Hyperlink(_) => "Hyperlink",
        Control::Ruby(_) => "Ruby",
        Control::CharOverlap(_) => "CharOverlap",
        Control::PageHide(_) => "PageHide",
        Control::HiddenComment(_) => "HiddenComment",
        Control::Equation(_) => "Equation",
        Control::Field(_) => "Field",
        Control::Form(_) => "Form",
        Control::Unknown(_) => "Unknown",
    }
}

fn format_control_positions(para: &rhwp::model::paragraph::Paragraph) -> String {
    let positions = para.control_text_positions();
    if positions.is_empty() {
        return "[]".to_string();
    }
    positions
        .iter()
        .enumerate()
        .map(|(ci, pos)| {
            let kind = para.controls.get(ci).map(control_kind).unwrap_or("?");
            format!("{ci}:{kind}@{pos}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_runs(runs: &[rhwp::renderer::composer::ComposedTextRun]) -> String {
    if runs.is_empty() {
        return "[]".to_string();
    }
    let parts = runs
        .iter()
        .map(|run| {
            format!(
                "cs{}:l{}:'{}'",
                run.char_style_id,
                run.lang_index,
                brief_text(&run.text, 40)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join("|"))
}

fn format_u32_list(values: &[u32]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    if values.len() <= 16 {
        return format!("{:?}", values);
    }
    let head = values
        .iter()
        .take(8)
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let tail = values
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}...{};len={}]", head, tail, values.len())
}

fn brief_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (count, ch) in text.chars().enumerate() {
        if count >= max_chars {
            out.push('…');
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{FFFC}' => out.push('□'),
            c if c.is_control() => out.push_str(&format!("\\u{{{:04X}}}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn dump_controls(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp dump <파일.hwp|파일.hwpx|파일.hml> [--section <번호>] [--para <번호>]"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut filter_section: Option<usize> = None;
    let mut filter_para: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "-s" => {
                if i + 1 < args.len() {
                    filter_section = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--para" | "-p" => {
                if i + 1 < args.len() {
                    filter_para = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 문서 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let document = doc.document();

    // border_fill 상세 덤프 (필터 없을 때 전체, 필터 있을 때 관련 bf만)
    if filter_section.is_none() && filter_para.is_none() {
        for (i, bf) in document.doc_info.border_fills.iter().enumerate() {
            let fill = &bf.fill;
            let solid_info = fill
                .solid
                .as_ref()
                .map(|s| {
                    format!(
                        "bg=#{:06X} pat_type={} pat_color=#{:06X}",
                        s.background_color, s.pattern_type, s.pattern_color
                    )
                })
                .unwrap_or_default();
            let grad_info = if fill.gradient.is_some() {
                " gradient"
            } else {
                ""
            };
            let img_info = fill
                .image
                .as_ref()
                .map(|img| {
                    format!(
                        " image(bin_id={}, mode={:?}, brightness={}, contrast={}, effect={})",
                        img.bin_data_id, img.fill_mode, img.brightness, img.contrast, img.effect
                    )
                })
                .unwrap_or_default();
            println!(
                "  border_fill[{}] fill_type={:?} {}{}{}",
                i, fill.fill_type, solid_info, grad_info, img_info
            );
        }
    }

    use rhwp::model::control::Control;
    use rhwp::model::paragraph::ColumnBreakType;
    use rhwp::model::shape::{HorzRelTo, ShapeObject, TextWrap, VertRelTo};

    let vert_str = |v: &VertRelTo| -> &str {
        match v {
            VertRelTo::Paper => "용지",
            VertRelTo::Page => "쪽",
            VertRelTo::Para => "문단",
        }
    };
    let horz_str = |h: &HorzRelTo| -> &str {
        match h {
            HorzRelTo::Paper => "용지",
            HorzRelTo::Page => "쪽",
            HorzRelTo::Column => "단",
            HorzRelTo::Para => "문단",
        }
    };
    let wrap_str = |w: &TextWrap| -> &str {
        match w {
            TextWrap::Square => "어울림",
            TextWrap::Tight => "빈 공간 채움",
            TextWrap::Through => "통과",
            TextWrap::TopAndBottom => "자리차지",
            TextWrap::BehindText => "글뒤로",
            TextWrap::InFrontOfText => "글앞으로",
        }
    };
    let break_str = |b: &ColumnBreakType| -> &str {
        match b {
            ColumnBreakType::None => "",
            ColumnBreakType::Section => "[구역나누기]",
            ColumnBreakType::MultiColumn => "[다단나누기]",
            ColumnBreakType::Page => "[쪽나누기]",
            ColumnBreakType::Column => "[단나누기]",
        }
    };

    // 도형 공통 속성 출력 헬퍼
    let dump_common = |c: &rhwp::model::shape::CommonObjAttr, indent: &str| {
        println!(
            "{}  크기: {:.1}mm × {:.1}mm ({}×{} HU)",
            indent,
            hu_to_mm(c.width),
            hu_to_mm(c.height),
            c.width,
            c.height
        );
        println!(
            "{}  위치: 가로={} 오프셋={:.1}mm({}) 정렬={:?}, 세로={} 오프셋={:.1}mm({}) 정렬={:?}",
            indent,
            horz_str(&c.horz_rel_to),
            hu_to_mm(c.horizontal_offset),
            c.horizontal_offset,
            c.horz_align,
            vert_str(&c.vert_rel_to),
            hu_to_mm(c.vertical_offset),
            c.vertical_offset,
            c.vert_align
        );
        println!(
            "{}  배치: {}, 글자처럼={}, z={}",
            indent,
            wrap_str(&c.text_wrap),
            c.treat_as_char,
            c.z_order
        );
        println!(
            "{}  바깥 여백: left={:.2}mm({}) right={:.2}mm({}) top={:.2}mm({}) bottom={:.2}mm({})",
            indent,
            hu_to_mm_i(c.margin.left as i32),
            c.margin.left,
            hu_to_mm_i(c.margin.right as i32),
            c.margin.right,
            hu_to_mm_i(c.margin.top as i32),
            c.margin.top,
            hu_to_mm_i(c.margin.bottom as i32),
            c.margin.bottom
        );
    };

    // 도형 요소 속성 출력 헬퍼
    let dump_shape_attr = |sa: &rhwp::model::shape::ShapeComponentAttr, indent: &str| {
        let eff_w = (sa.current_width as f64 * sa.render_sx) as u32;
        let eff_h = (sa.current_height as f64 * sa.render_sy) as u32;
        println!("{}  요소: orig={}×{}, curr={}×{}, M=[{:.3},{:.3},{:.0}; {:.3},{:.3},{:.0}], offset=({},{}), eff={:.1}mm×{:.1}mm",
            indent, sa.original_width, sa.original_height,
            sa.current_width, sa.current_height,
            sa.render_sx, sa.render_b, sa.render_tx,
            sa.render_c, sa.render_sy, sa.render_ty,
            sa.offset_x, sa.offset_y,
            hu_to_mm(eff_w), hu_to_mm(eff_h));
        if sa.horz_flip || sa.vert_flip || sa.rotation_angle != 0 {
            println!(
                "{}  변환: 뒤집기=({},{}), 회전={}",
                indent, sa.horz_flip, sa.vert_flip, sa.rotation_angle
            );
        }
    };

    // 재귀적 도형 덤프
    fn dump_shape(
        shape: &ShapeObject,
        indent: &str,
        dump_common_fn: &dyn Fn(&rhwp::model::shape::CommonObjAttr, &str),
        dump_sa_fn: &dyn Fn(&rhwp::model::shape::ShapeComponentAttr, &str),
    ) {
        match shape {
            ShapeObject::Line(s) => {
                println!(
                    "{}[직선] start=({},{}) end=({},{})",
                    indent, s.start.x, s.start.y, s.end.x, s.end.y
                );
                println!(
                    "{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent,
                    s.drawing.border_line.color,
                    s.drawing.border_line.width,
                    s.drawing.border_line.attr
                );
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Rectangle(s) => {
                println!("{}[사각형] round={}%", indent, s.round_rate);
                println!(
                    "{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent,
                    s.drawing.border_line.color,
                    s.drawing.border_line.width,
                    s.drawing.border_line.attr
                );
                println!(
                    "{}  채우기: {:?}{}",
                    indent,
                    s.drawing.fill.fill_type,
                    if let Some(ref img) = s.drawing.fill.image {
                        format!(
                            ", image=bin_data_id={}, mode={:?}",
                            img.bin_data_id, img.fill_mode
                        )
                    } else {
                        String::new()
                    }
                );
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                if let Some(tb) = &s.drawing.text_box {
                    println!("{}  글상자: list_attr={:#010x}, margins=({},{},{},{}), max_width={}, paras={}",
                        indent, tb.list_attr, tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom,
                        tb.max_width, tb.paragraphs.len());
                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                        let text_preview = if tp.text.is_empty() {
                            "(빈)".to_string()
                        } else if tp.text.chars().count() > 60 {
                            let end = tp
                                .text
                                .char_indices()
                                .nth(60)
                                .map(|(i, _)| i)
                                .unwrap_or(tp.text.len());
                            format!("\"{}...\"", &tp.text[..end])
                        } else {
                            format!("\"{}\"", tp.text)
                        };
                        println!(
                            "{}    p[{}]: ps_id={}, cc={}, text={}, ls_count={}, ctrls={}",
                            indent,
                            tpi,
                            tp.para_shape_id,
                            tp.char_count,
                            text_preview,
                            tp.line_segs.len(),
                            tp.controls.len()
                        );
                        for (li, ls) in tp.line_segs.iter().enumerate() {
                            println!(
                                "{}      ls[{}]: vpos={}, lh={}, th={}, bl={}, cs={}, sw={}",
                                indent,
                                li,
                                ls.vertical_pos,
                                ls.line_height,
                                ls.text_height,
                                ls.baseline_distance,
                                ls.column_start,
                                ls.segment_width
                            );
                        }
                    }
                }
            }
            ShapeObject::Ellipse(s) => {
                println!("{}[타원]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Arc(s) => {
                println!("{}[호]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Polygon(s) => {
                println!("{}[다각형] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                // 좌표 범위 출력
                if !s.points.is_empty() {
                    let min_x = s.points.iter().map(|p| p.x).min().unwrap();
                    let max_x = s.points.iter().map(|p| p.x).max().unwrap();
                    let min_y = s.points.iter().map(|p| p.y).min().unwrap();
                    let max_y = s.points.iter().map(|p| p.y).max().unwrap();
                    println!(
                        "{}  좌표범위: x=[{},{}], y=[{},{}]",
                        indent, min_x, max_x, min_y, max_y
                    );
                }
            }
            ShapeObject::Curve(s) => {
                println!("{}[곡선] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Group(g) => {
                println!("{}[묶음] children={}", indent, g.children.len());
                dump_common_fn(&g.common, indent);
                dump_sa_fn(&g.shape_attr, indent);
                let child_indent = format!("{}  ", indent);
                for (ci, child) in g.children.iter().enumerate() {
                    print!("{}child[{}] ", child_indent, ci);
                    dump_shape(child, &child_indent, dump_common_fn, dump_sa_fn);
                }
            }
            ShapeObject::Picture(p) => {
                println!("{}[그림] bin_data_id={}", indent, p.image_attr.bin_data_id);
                dump_common_fn(&p.common, indent);
                dump_sa_fn(&p.shape_attr, indent);
            }
            ShapeObject::Chart(c) => {
                println!(
                    "{}[차트] type={:?} series={} raw_chart_data={}B",
                    indent,
                    c.chart_type,
                    c.series.len(),
                    c.raw_chart_data.len()
                );
                dump_common_fn(&c.common, indent);
                dump_sa_fn(&c.drawing.shape_attr, indent);
            }
            ShapeObject::Ole(o) => {
                println!(
                    "{}[OLE] bin_data_id={} extent={}x{} flags=0x{:02X} raw={}B",
                    indent,
                    o.bin_data_id,
                    o.extent_x,
                    o.extent_y,
                    o.flags,
                    o.raw_tag_data.len()
                );
                dump_common_fn(&o.common, indent);
                dump_sa_fn(&o.drawing.shape_attr, indent);
            }
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        if let Some(fs) = filter_section {
            if sec_idx != fs {
                continue;
            }
        }

        let pd = &section.section_def.page_def;
        println!("=== 구역 {} ===", sec_idx);
        println!(
            "  용지: {:.1}mm × {:.1}mm ({}×{} HU), {}",
            hu_to_mm(pd.width),
            hu_to_mm(pd.height),
            pd.width,
            pd.height,
            if pd.landscape { "가로" } else { "세로" }
        );
        println!(
            "  여백: 좌={:.1} 우={:.1} 상={:.1} 하={:.1} 머리말={:.1} 꼬리말={:.1} mm",
            hu_to_mm(pd.margin_left),
            hu_to_mm(pd.margin_right),
            hu_to_mm(pd.margin_top),
            hu_to_mm(pd.margin_bottom),
            hu_to_mm(pd.margin_header),
            hu_to_mm(pd.margin_footer)
        );

        // 바탕쪽 정보
        if !section.section_def.master_pages.is_empty() {
            println!("  바탕쪽: {}개", section.section_def.master_pages.len());
            for (mi, mp) in section.section_def.master_pages.iter().enumerate() {
                println!("    [{}] {:?}, 문단 {}개, 영역 {}×{} HU, is_ext={}, overlap={}, ext_flags=0x{:04X}, text_ref={}, num_ref={}",
                    mi, mp.apply_to, mp.paragraphs.len(), mp.text_width, mp.text_height,
                    mp.is_extension, mp.overlap, mp.ext_flags, mp.text_ref, mp.num_ref);
                for (pi, para) in mp.paragraphs.iter().enumerate() {
                    println!(
                        "      p[{}]: cc={}, text=\"{}\"",
                        pi,
                        para.controls.len(),
                        if para.text.is_empty() {
                            "(빈 문단)".to_string()
                        } else {
                            para.text.chars().take(30).collect::<String>()
                        }
                    );
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        let ctrl_name = match ctrl {
                            Control::Table(t) => {
                                let cell_texts: Vec<String> = t
                                    .cells
                                    .iter()
                                    .take(3)
                                    .map(|c| {
                                        c.paragraphs
                                            .iter()
                                            .map(|p| p.text.chars().take(20).collect::<String>())
                                            .collect::<Vec<_>>()
                                            .join("|")
                                    })
                                    .collect();
                                format!("표({}x{}, tac={}, wrap={:?}, vert={:?}/{}, horz={:?}/{}, size={}x{}, cells=[{}])",
                                    t.row_count, t.col_count, t.common.treat_as_char,
                                    t.common.text_wrap, t.common.vert_rel_to, t.common.vertical_offset,
                                    t.common.horz_rel_to, t.common.horizontal_offset,
                                    t.common.width, t.common.height,
                                    cell_texts.join("; "))
                            }
                            Control::Shape(s) => {
                                let mut desc = format!("도형(ctrl_id=0x{:08X}, w={}, h={}, attr=0x{:08X}, wc={:?}, hc={:?})",
                                    s.common().ctrl_id, s.common().width, s.common().height,
                                    s.common().attr, s.common().width_criterion, s.common().height_criterion);
                                // TextBox 내용 출력
                                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                                    desc += &format!(" 글상자({}문단)", tb.paragraphs.len());
                                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                        let tp_text: String = tp.text.chars().take(20).collect();
                                        desc += &format!(
                                            "\n          tb_p[{}]: cc={} text=\"{}\"",
                                            tpi,
                                            tp.controls.len(),
                                            tp_text
                                        );
                                        for (tci, tc) in tp.controls.iter().enumerate() {
                                            let tc_name = match tc {
                                                Control::AutoNumber(an) => {
                                                    format!("자동번호({:?})", an.number_type)
                                                }
                                                _ => format!("{:?}", std::mem::discriminant(tc)),
                                            };
                                            desc += &format!(
                                                "\n            tb_ctrl[{}]: {}",
                                                tci, tc_name
                                            );
                                        }
                                    }
                                }
                                desc
                            }
                            Control::Picture(p) => {
                                let wm = p
                                    .image_attr
                                    .watermark_preset()
                                    .map(|s| format!(", watermark={}", s))
                                    .unwrap_or_default();
                                format!(
                                    "그림(bin_id={}, w={}, h={}, tac={}{})",
                                    p.image_attr.bin_data_id,
                                    p.common.width,
                                    p.common.height,
                                    p.common.treat_as_char,
                                    wm
                                )
                            }
                            Control::Header(_) => "머리말".to_string(),
                            Control::Footer(_) => "꼬리말".to_string(),
                            _ => format!("{:?}", std::mem::discriminant(ctrl)),
                        };
                        println!("        ctrl[{}]: {}", ci, ctrl_name);
                    }
                }
            }
        }
        if section.section_def.hide_master_page {
            println!("  바탕쪽 감추기: true");
        }

        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(fp) = filter_para {
                if para_idx != fp {
                    continue;
                }
            }

            let text_preview = if para.text.is_empty() {
                "(빈 문단)".to_string()
            } else {
                let preview = if para.text.chars().count() > 50 {
                    let end = para
                        .text
                        .char_indices()
                        .nth(50)
                        .map(|(i, _)| i)
                        .unwrap_or(para.text.len());
                    format!("\"{}...\"", &para.text[..end])
                } else {
                    format!("\"{}\"", para.text)
                };
                preview
            };

            let break_info = break_str(&para.column_type);
            println!(
                "\n--- 문단 {}.{} --- cc={}, text_len={}, controls={} {}",
                sec_idx,
                para_idx,
                para.char_count,
                para.text.chars().count(),
                para.controls.len(),
                break_info
            );
            println!("  텍스트: {}", text_preview);
            // char_shapes 출력
            if !para.char_shapes.is_empty() {
                let text_chars: Vec<char> = para.text.chars().collect();
                for (ci, cs) in para.char_shapes.iter().enumerate() {
                    let next_pos = para
                        .char_shapes
                        .get(ci + 1)
                        .map(|n| n.start_pos)
                        .unwrap_or(u32::MAX);
                    let char_at = text_chars
                        .iter()
                        .enumerate()
                        .find(|(i, _)| {
                            if *i < para.char_offsets.len() {
                                para.char_offsets[*i] >= cs.start_pos
                                    && para.char_offsets[*i] < next_pos
                            } else {
                                false
                            }
                        })
                        .map(|(_, c)| *c);
                    if let Some(chs) = document.doc_info.char_shapes.get(cs.char_shape_id as usize)
                    {
                        let bold = (chs.attr & 0x02) != 0;
                        let spacing = chs.spacings[0]; // 한국어 자간
                        let ratio = chs.ratios[0]; // 한국어 장평
                        println!(
                            "  [CS] pos={} id={} bold={} spacing={}% ratio={}% base={} attr=0x{:08X} text=#{:06X} shade=#{:06X} shadow=#{:06X} border_fill_id={} shadow_type={} shadow_off=({}, {}) char={:?}",
                            cs.start_pos,
                            cs.char_shape_id,
                            bold,
                            spacing,
                            ratio,
                            chs.base_size,
                            chs.attr,
                            chs.text_color,
                            chs.shade_color,
                            chs.shadow_color,
                            chs.border_fill_id,
                            chs.shadow_type,
                            chs.shadow_offset_x,
                            chs.shadow_offset_y,
                            char_at.map(|c| c.to_string()).unwrap_or_default()
                        );
                    }
                }
            }
            if let Some(ps) = document
                .doc_info
                .para_shapes
                .get(para.para_shape_id as usize)
            {
                // 문단 모양 기본 정보 (항상 출력)
                println!(
                    "  [PS] ps_id={} align={:?} spacing: before={} after={} line={}/{:?}",
                    para.para_shape_id,
                    ps.alignment,
                    ps.spacing_before,
                    ps.spacing_after,
                    ps.line_spacing,
                    ps.line_spacing_type
                );
                println!(
                    "       margins: left={} right={} indent={} border_fill_id={}",
                    ps.margin_left, ps.margin_right, ps.indent, ps.border_fill_id
                );
                println!(
                    "       keep: with_next={} keep_lines={} widow_orphan={} pbreak_before={} (attr1=0x{:08X} attr2=0x{:08X})",
                    (ps.attr1 >> 17) & 1 != 0 || (ps.attr2 >> 6) & 1 != 0,
                    (ps.attr1 >> 18) & 1 != 0 || (ps.attr2 >> 7) & 1 != 0,
                    (ps.attr1 >> 16) & 1 != 0 || (ps.attr2 >> 5) & 1 != 0,
                    (ps.attr1 >> 19) & 1 != 0 || (ps.attr2 >> 8) & 1 != 0,
                    ps.attr1, ps.attr2
                );
                if ps.border_fill_id > 0 {
                    println!(
                        "       border_spacing: left={} right={} top={} bottom={}",
                        ps.border_spacing[0],
                        ps.border_spacing[1],
                        ps.border_spacing[2],
                        ps.border_spacing[3]
                    );
                }
                if ps.head_type != rhwp::model::style::HeadType::None {
                    println!("       head={:?} level={} num_id={} attr1=0x{:08X} attr2=0x{:08X} raw_extra={:?}",
                        ps.head_type, ps.para_level, ps.numbering_id, ps.attr1, ps.attr2,
                        &para.raw_header_extra);
                }
                {
                    let td_id = ps.tab_def_id;
                    if let Some(td) = document.doc_info.tab_defs.get(td_id as usize) {
                        let tabs_str: Vec<String> = td
                            .tabs
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "tab[{}] pos={} ({:.1}mm) type={} fill={}",
                                    i,
                                    t.position,
                                    hu_to_mm(t.position),
                                    t.tab_type,
                                    t.fill_type
                                )
                            })
                            .collect();
                        println!(
                            "       tab_def_id={} auto_left={} auto_right={} tabs=[{}]",
                            td_id,
                            td.auto_tab_left,
                            td.auto_tab_right,
                            if tabs_str.is_empty() {
                                "(없음)".to_string()
                            } else {
                                tabs_str.join(", ")
                            }
                        );
                    } else {
                        println!("       tab_def_id={} (정의 없음)", td_id);
                    }
                }
            }
            // line_segs 출력
            if !para.line_segs.is_empty() {
                for (li, ls) in para.line_segs.iter().enumerate() {
                    println!("  ls[{}]: ts={}, vpos={}, lh={}, th={}, bl={}, ls={}, cs={}, sw={}, tag=0x{:08X}",
                        li, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                        ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
                }
            }

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                let prefix = format!("  [{}] ", ctrl_idx);
                match ctrl {
                    Control::ColumnDef(cd) => {
                        let ct = match cd.column_type {
                            rhwp::model::page::ColumnType::Normal => "일반",
                            rhwp::model::page::ColumnType::Distribute => "배분",
                            rhwp::model::page::ColumnType::Parallel => "병행",
                        };
                        println!(
                            "{}단정의: {}단, 유형={}, 간격={:.1}mm({}), 같은너비={}",
                            prefix,
                            cd.column_count,
                            ct,
                            hu_to_mm_i(cd.spacing as i32),
                            cd.spacing,
                            cd.same_width
                        );
                        if !cd.widths.is_empty() {
                            // 비례값일 경우 body_width 기준으로 실제 mm 변환
                            let body_width_hu = {
                                let spd = &section.section_def.page_def;
                                let (pw, _) = if spd.landscape {
                                    (spd.height, spd.width)
                                } else {
                                    (spd.width, spd.height)
                                };
                                (pw - spd.margin_left - spd.margin_right - spd.margin_gutter) as f64
                            };
                            let total: f64 = if cd.proportional_widths {
                                cd.widths
                                    .iter()
                                    .chain(cd.gaps.iter())
                                    .map(|&v| (v as u16) as f64)
                                    .sum()
                            } else {
                                1.0
                            };
                            let cols_info: Vec<String> = cd
                                .widths
                                .iter()
                                .enumerate()
                                .map(|(i, w)| {
                                    let gap = cd.gaps.get(i).copied().unwrap_or(0);
                                    if cd.proportional_widths && total > 0.0 {
                                        let w_hu = (*w as u16) as f64 / total * body_width_hu;
                                        let g_hu = (gap as u16) as f64 / total * body_width_hu;
                                        format!(
                                            "너비={:.1}mm 간격={:.1}mm",
                                            w_hu * 25.4 / 7200.0,
                                            g_hu * 25.4 / 7200.0
                                        )
                                    } else {
                                        format!(
                                            "너비={:.1}mm 간격={:.1}mm",
                                            hu_to_mm_i(*w as i32),
                                            hu_to_mm_i(gap as i32)
                                        )
                                    }
                                })
                                .collect();
                            println!("{}  단별: [{}]", prefix, cols_info.join(", "));
                        }
                        if cd.separator_type > 0 {
                            println!(
                                "{}  구분선: type={}, width={}, color={:#010x}",
                                prefix, cd.separator_type, cd.separator_width, cd.separator_color
                            );
                        }
                    }
                    Control::SectionDef(sd) => {
                        let spd = &sd.page_def;
                        println!(
                            "{}구역정의: 용지 {:.1}×{:.1}mm, {}, flags=0x{:08X}",
                            prefix,
                            hu_to_mm(spd.width),
                            hu_to_mm(spd.height),
                            if spd.landscape { "가로" } else { "세로" },
                            sd.flags
                        );
                        if sd.hide_header || sd.hide_footer || sd.hide_master_page {
                            println!(
                                "{}  감추기: 머리말={} 꼬리말={} 바탕쪽={}",
                                prefix, sd.hide_header, sd.hide_footer, sd.hide_master_page
                            );
                        }
                    }
                    Control::Table(table) => {
                        println!("{}표: {}행×{}열, 셀={}, 쪽나눔={:?} (attr=0x{:08x}), padding=({},{},{},{}), cs={}",
                            prefix, table.row_count, table.col_count,
                            table.cells.len(), table.page_break, table.raw_table_record_attr,
                            table.padding.left, table.padding.right, table.padding.top, table.padding.bottom,
                            table.cell_spacing);
                        if !table.zones.is_empty() {
                            for (zi, z) in table.zones.iter().enumerate() {
                                println!(
                                    "{}  zone[{}] row={}..{} col={}..{} bf={}",
                                    prefix,
                                    zi,
                                    z.start_row,
                                    z.end_row,
                                    z.start_col,
                                    z.end_col,
                                    z.border_fill_id
                                );
                            }
                        }
                        {
                            let c = &table.common;
                            println!("{}  [common] treat_as_char={}, wrap={}, vert={}({}={:.1}mm), horz={}({}={:.1}mm)",
                                prefix, c.treat_as_char, wrap_str(&c.text_wrap),
                                vert_str(&c.vert_rel_to), c.vertical_offset, hu_to_mm(c.vertical_offset),
                                horz_str(&c.horz_rel_to), c.horizontal_offset, hu_to_mm(c.horizontal_offset));
                            println!(
                                "{}  [common] size={}×{}({:.1}×{:.1}mm), valign={:?}, halign={:?}",
                                prefix,
                                c.width,
                                c.height,
                                hu_to_mm(c.width),
                                hu_to_mm(c.height),
                                c.vert_align,
                                c.horz_align
                            );
                            println!("{}  [outer_margin] left={:.1}mm({}) right={:.1}mm({}) top={:.1}mm({}) bottom={:.1}mm({})",
                                prefix,
                                hu_to_mm_i(table.outer_margin_left as i32), table.outer_margin_left,
                                hu_to_mm_i(table.outer_margin_right as i32), table.outer_margin_right,
                                hu_to_mm_i(table.outer_margin_top as i32), table.outer_margin_top,
                                hu_to_mm_i(table.outer_margin_bottom as i32), table.outer_margin_bottom);
                            if table.raw_ctrl_data.len() >= 20 {
                                println!(
                                    "{}  [raw] {:02X?}",
                                    prefix,
                                    &table.raw_ctrl_data[..20.min(table.raw_ctrl_data.len())]
                                );
                            }
                        }
                        // 셀 상세 출력
                        fn dump_table_deep(
                            table: &rhwp::model::table::Table,
                            indent: &str,
                            depth: usize,
                        ) {
                            for (ci, cell) in table.cells.iter().enumerate() {
                                let text_preview: String = cell
                                    .paragraphs
                                    .iter()
                                    .map(|p| p.text.chars().take(30).collect::<String>())
                                    .collect::<Vec<_>>()
                                    .join("|");
                                println!("{}셀[{}] r={},c={} rs={},cs={} h={} w={} pad=({},{},{},{}) valign={:?} aim={} hdr={} bf={} paras={} text=\"{}\"",
                                    indent, ci, cell.row, cell.col, cell.row_span, cell.col_span,
                                    cell.height, cell.width,
                                    cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
                                    cell.vertical_align,
                                    cell.apply_inner_margin,
                                    cell.is_header,
                                    cell.border_fill_id, cell.paragraphs.len(), text_preview);
                                if let Some(ref fname) = cell.field_name {
                                    println!("{}  field=\"{}\"", indent, fname);
                                }
                                // 셀 내 LINE_SEG 상세
                                for (pi, cp) in cell.paragraphs.iter().enumerate() {
                                    if !cp.line_segs.is_empty() || !cp.controls.is_empty() {
                                        let ls_info: Vec<String> = cp
                                            .line_segs
                                            .iter()
                                            .enumerate()
                                            .map(|(li, ls)| {
                                                format!(
                                                    "ls[{}] vpos={} lh={} ls={}",
                                                    li,
                                                    ls.vertical_pos,
                                                    ls.line_height,
                                                    ls.line_spacing
                                                )
                                            })
                                            .collect();
                                        println!(
                                            "{}  p[{}] ps_id={} ctrls={} text_len={} {}",
                                            indent,
                                            pi,
                                            cp.para_shape_id,
                                            cp.controls.len(),
                                            cp.text.len(),
                                            ls_info.join(", ")
                                        );
                                    }
                                    // 셀 내부 컨트롤 상세
                                    for (ci, ctrl) in cp.controls.iter().enumerate() {
                                        match ctrl {
                                            Control::Picture(p) => {
                                                println!("{}    ctrl[{}] 그림: bin_id={}, w={} h={} ({:.1}×{:.1}mm), tac={}, wrap={:?}, vert={:?}(off={}), horz={:?}(off={}), orig={}×{}, cur={}×{}, crop=({},{},{},{})",
                                                    indent, ci, p.image_attr.bin_data_id,
                                                    p.common.width, p.common.height,
                                                    p.common.width as f64 / 7200.0 * 25.4,
                                                    p.common.height as f64 / 7200.0 * 25.4,
                                                    p.common.treat_as_char,
                                                    p.common.text_wrap, p.common.vert_rel_to, p.common.vertical_offset,
                                                    p.common.horz_rel_to, p.common.horizontal_offset,
                                                    p.shape_attr.original_width, p.shape_attr.original_height,
                                                    p.shape_attr.current_width, p.shape_attr.current_height,
                                                    p.crop.left, p.crop.top, p.crop.right, p.crop.bottom);
                                                println!("{}      [image_attr] effect={:?} brightness={} contrast={} watermark={}",
                                                    indent, p.image_attr.effect, p.image_attr.brightness, p.image_attr.contrast,
                                                    p.image_attr.watermark_preset().unwrap_or("none"));
                                            }
                                            Control::Shape(s) => {
                                                println!(
                                                    "{}    ctrl[{}] {}: tac={}, wrap={:?}",
                                                    indent,
                                                    ci,
                                                    s.shape_name(),
                                                    s.common().treat_as_char,
                                                    s.common().text_wrap
                                                );
                                            }
                                            Control::PageHide(ph) => {
                                                println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
                                                    indent, ci,
                                                    ph.hide_header, ph.hide_footer, ph.hide_master_page,
                                                    ph.hide_border, ph.hide_fill, ph.hide_page_num);
                                            }
                                            _ => {}
                                        }
                                    }
                                    // 내부 표 재귀
                                    if depth < 3 {
                                        for ctrl in &cp.controls {
                                            if let Control::Table(inner) = ctrl {
                                                println!("{}  p[{}] 내부표: {}행×{}열, 셀={}, cs={}, pad=({},{},{},{})",
                                                    indent, pi, inner.row_count, inner.col_count,
                                                    inner.cells.len(), inner.cell_spacing,
                                                    inner.padding.left, inner.padding.right, inner.padding.top, inner.padding.bottom);
                                                let next_indent = format!("{}    ", indent);
                                                dump_table_deep(inner, &next_indent, depth + 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        dump_table_deep(table, &format!("{}  ", prefix), 0);
                    }
                    Control::Shape(shape) => {
                        print!("{}", prefix);
                        dump_shape(shape, "  ", &dump_common, &dump_shape_attr);
                    }
                    Control::Picture(pic) => {
                        let sa = &pic.shape_attr;
                        println!("{}그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}",
                            prefix, pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                            sa.original_width, sa.original_height,
                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                            sa.current_width, sa.current_height,
                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                            pic.common.treat_as_char);
                        println!(
                            "{}  [placement] wrap={:?} vert={:?}(off={}) horz={:?}(off={}) vert_align={:?}",
                            prefix, pic.common.text_wrap, pic.common.vert_rel_to, pic.common.vertical_offset,
                            pic.common.horz_rel_to, pic.common.horizontal_offset, pic.common.vert_align);
                        println!(
                            "{}  [image_attr] effect={:?} brightness={} contrast={} watermark={}{}",
                            prefix,
                            pic.image_attr.effect,
                            pic.image_attr.brightness,
                            pic.image_attr.contrast,
                            pic.image_attr.watermark_preset().unwrap_or("none"),
                            pic.image_attr
                                .external_path
                                .as_ref()
                                .map(|p| format!(" external_path=\"{}\"", p))
                                .unwrap_or_default()
                        );
                        println!("{}  border_x={:?} border_y={:?} border_color=#{:06X} border_width={} ({:.2}mm) border_attr={:?}",
                            prefix, pic.border_x, pic.border_y,
                            pic.border_color, pic.border_width, pic.border_width as f64 / 7200.0 * 25.4,
                            pic.border_attr);
                        println!(
                            "{}  crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                            prefix,
                            pic.crop.left,
                            pic.crop.top,
                            pic.crop.right,
                            pic.crop.bottom,
                            pic.crop.left as f64 / 7200.0 * 25.4,
                            pic.crop.top as f64 / 7200.0 * 25.4,
                            pic.crop.right as f64 / 7200.0 * 25.4,
                            pic.crop.bottom as f64 / 7200.0 * 25.4
                        );
                        if let Some(ref cap) = pic.caption {
                            let cap_text: String = cap
                                .paragraphs
                                .iter()
                                .map(|p| p.text.clone())
                                .collect::<Vec<_>>()
                                .join("|");
                            println!(
                                "{}  caption: dir={:?} width={} paras={} text={:?}",
                                prefix,
                                cap.direction,
                                cap.width,
                                cap.paragraphs.len(),
                                cap_text
                            );
                        }
                        let shape_indent = format!("{}  ", prefix);
                        dump_shape_attr(sa, &shape_indent);
                        dump_common(&pic.common, "  ");
                    }
                    Control::Header(h) => {
                        let text: String = h
                            .paragraphs
                            .iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!(
                            "{}머리말({:?}): paras={} \"{}\"",
                            prefix,
                            h.apply_to,
                            h.paragraphs.len(),
                            text
                        );
                        for (hpi, hp) in h.paragraphs.iter().enumerate() {
                            if !hp.controls.is_empty() {
                                for (hci, hc) in hp.controls.iter().enumerate() {
                                    let cn = match hc {
                                        Control::AutoNumber(an) => {
                                            format!("자동번호({:?})", an.number_type)
                                        }
                                        Control::Shape(s) => {
                                            let c = s.common();
                                            let mut desc = format!(
                                                "Shape horz={:?}/{} halign={:?} w={} h={}",
                                                c.horz_rel_to,
                                                c.horizontal_offset,
                                                c.horz_align,
                                                c.width,
                                                c.height
                                            );
                                            if let Some(tb) =
                                                s.drawing().and_then(|d| d.text_box.as_ref())
                                            {
                                                let text: String = tb
                                                    .paragraphs
                                                    .iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!(" text={:?}", text);
                                            }
                                            desc
                                        }
                                        Control::Table(t) => {
                                            let mut desc = format!(
                                                "표 {}행×{}열 셀={}",
                                                t.row_count,
                                                t.col_count,
                                                t.cells.len()
                                            );
                                            for (si, cell) in t.cells.iter().enumerate() {
                                                let cell_text: String = cell
                                                    .paragraphs
                                                    .iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!(
                                                    "\n{}    셀[{}] text={:?}",
                                                    prefix, si, cell_text
                                                );
                                                for (cpi, cp) in cell.paragraphs.iter().enumerate()
                                                {
                                                    for (cci, cc) in cp.controls.iter().enumerate()
                                                    {
                                                        let ccn = match cc {
                                                            Control::AutoNumber(an) => format!(
                                                                "자동번호({:?})",
                                                                an.number_type
                                                            ),
                                                            Control::Shape(s) => {
                                                                let c = s.common();
                                                                let mut d = format!("Shape vert={:?}/{} valign={:?} horz={:?}/{} halign={:?} w={} h={}",
                                                c.vert_rel_to, c.vertical_offset, c.vert_align,
                                                c.horz_rel_to, c.horizontal_offset, c.horz_align, c.width, c.height);
                                                                if let Some(tb) =
                                                                    s.drawing().and_then(|dd| {
                                                                        dd.text_box.as_ref()
                                                                    })
                                                                {
                                                                    for (tpi, tp) in tb
                                                                        .paragraphs
                                                                        .iter()
                                                                        .enumerate()
                                                                    {
                                                                        let t: String = tp
                                                                            .text
                                                                            .chars()
                                                                            .take(30)
                                                                            .collect();
                                                                        d += &format!(" tb_p[{}] ps_id={} text={:?}", tpi, tp.para_shape_id, t);
                                                                    }
                                                                }
                                                                d
                                                            }
                                                            _ => format!(
                                                                "{:?}",
                                                                std::mem::discriminant(cc)
                                                            ),
                                                        };
                                                        desc += &format!(
                                                            "\n{}      p[{}]c[{}]: {}",
                                                            prefix, cpi, cci, ccn
                                                        );
                                                    }
                                                }
                                            }
                                            desc
                                        }
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        }
                                        _ => format!("{:?}", std::mem::discriminant(hc)),
                                    };
                                    let display = if cn.chars().count() > 30 {
                                        format!(
                                            "{}...(truncated)",
                                            cn.chars().take(30).collect::<String>()
                                        )
                                    } else {
                                        cn
                                    };
                                    println!("{}  hp[{}] ctrl[{}]: {}", prefix, hpi, hci, display);
                                }
                            }
                        }
                    }
                    Control::Footer(f) => {
                        let text: String = f
                            .paragraphs
                            .iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!(
                            "{}꼬리말({:?}): paras={} \"{}\"",
                            prefix,
                            f.apply_to,
                            f.paragraphs.len(),
                            text
                        );
                        for (fpi, fp) in f.paragraphs.iter().enumerate() {
                            if !fp.controls.is_empty() {
                                for (fci, fc) in fp.controls.iter().enumerate() {
                                    let cn = match fc {
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        }
                                        _ => format!("{:?}", std::mem::discriminant(fc)),
                                    };
                                    println!("{}  fp[{}] ctrl[{}]: {}", prefix, fpi, fci, cn);
                                }
                            }
                        }
                    }
                    Control::Footnote(fn_) => {
                        println!("{}각주: paragraphs={}", prefix, fn_.paragraphs.len());
                    }
                    Control::Endnote(en) => {
                        println!("{}미주: paragraphs={}", prefix, en.paragraphs.len());
                    }
                    Control::AutoNumber(an) => {
                        println!(
                            "{}자동번호: type={:?}, number={}",
                            prefix, an.number_type, an.number
                        );
                    }
                    Control::NewNumber(nn) => {
                        println!(
                            "{}새번호: type={:?}, number={}",
                            prefix, nn.number_type, nn.number
                        );
                    }
                    Control::PageNumberPos(pn) => {
                        println!(
                            "{}쪽번호위치: format={}, pos={}",
                            prefix, pn.format, pn.position
                        );
                    }
                    Control::Bookmark(bm) => {
                        println!("{}책갈피: \"{}\"", prefix, bm.name);
                    }
                    Control::Hyperlink(hl) => {
                        println!("{}하이퍼링크: \"{}\"", prefix, hl.url);
                    }
                    Control::Ruby(r) => {
                        println!("{}덧말: \"{}\"", prefix, r.ruby_text);
                    }
                    Control::PageHide(ph) => {
                        println!("{}감추기: header={}, footer={}, master={}, border={}, fill={}, page_num={}",
                            prefix, ph.hide_header, ph.hide_footer, ph.hide_master_page, ph.hide_border, ph.hide_fill, ph.hide_page_num);
                    }
                    Control::HiddenComment(_) => {
                        println!("{}숨은설명", prefix);
                    }
                    Control::Field(f) => {
                        let name = f.field_name().unwrap_or("(이름없음)");
                        println!(
                            "{}필드: {:?} name=\"{}\" cmd=\"{}\"",
                            prefix, f.field_type, name, f.command
                        );
                    }
                    Control::CharOverlap(co) => {
                        println!("{}글자겹침: {:?}", prefix, co.chars);
                    }
                    Control::Equation(eq) => {
                        println!(
                            "{}수식: script=\"{}\" font_size={} font=\"{}\" size={}x{} tac={}",
                            prefix,
                            eq.script,
                            eq.font_size,
                            eq.font_name,
                            eq.common.width,
                            eq.common.height,
                            eq.common.treat_as_char
                        );
                    }
                    Control::Form(f) => {
                        println!(
                            "{}양식개체: {:?} name=\"{}\" caption=\"{}\" {}x{}",
                            prefix, f.form_type, f.name, f.caption, f.width, f.height
                        );
                    }
                    Control::Unknown(u) => {
                        println!("{}알수없음: ctrl_id={:#010x}", prefix, u.ctrl_id);
                    }
                }
            }
        }
    }

    println!(
        "\n=== 완료: {} 구역, {} 문단 ===",
        document.sections.len(),
        document
            .sections
            .iter()
            .map(|s| s.paragraphs.len())
            .sum::<usize>()
    );

    EXIT_OK
}

/// `search` — 주소(구역·문단·**페이지**)를 가진 문서 검색.
///
/// 평문을 뽑아 외부에서 찾으면 주소가 소멸해 근거 제시가 불가능하다. rhwp 는 조판 엔진이
/// 있어 "몇 쪽"에 답할 수 있는 유일한 도구인데, 그 출구가 없었다.
fn search_document(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut query: Option<&str> = None;
    let mut json_mode = false;
    let mut ignore_case = false;
    let mut limit: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--ignore-case" | "-i" => ignore_case = true,
            "--limit" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => limit = Some(n),
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.is_none() {
                    file_path = Some(other);
                } else if query.is_none() {
                    query = Some(other);
                } else {
                    eprintln!("오류: 인자가 너무 많습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(query)) = (file_path, query) else {
        eprintln!(
            "사용법: rhwp search <파일.hwp|파일.hwpx> <검색어> [--json] [--ignore-case] [--limit <N>]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let matches = doc.grep(query, !ignore_case, limit);

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": "1.0",
            "source": file_path,
            "query": query,
            "caseSensitive": !ignore_case,
            "matchCount": matches.len(),
            "matches": matches,
        });
        println!("{envelope}");
        // 매치 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707).
        return EXIT_OK;
    }

    println!("검색: {:?} in {} — {}건", query, file_path, matches.len());
    for m in &matches {
        let page = m
            .page
            .map(|p| format!("{}쪽", p + 1))
            .unwrap_or_else(|| "쪽 미배치".to_string());
        println!(
            "  [{}] 구역{}:문단{} +{}  {}",
            page, m.section, m.paragraph, m.char_offset, m.context
        );
    }
    EXIT_OK
}

fn diag_document(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp diag <파일.hwp>");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let document = doc.document();
    use rhwp::model::style::HeadType;

    // === DocInfo 요약 ===
    println!("=== DocInfo 요약 ===");
    println!("  Numbering: {}개", document.doc_info.numberings.len());
    for (i, num) in document.doc_info.numberings.iter().enumerate() {
        let formats: Vec<String> = num
            .level_formats
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.is_empty())
            .map(|(lv, f)| format!("L{}=\"{}\"", lv + 1, f))
            .collect();
        println!(
            "    [{}] start={}, formats: {}",
            i,
            num.start_number,
            formats.join(", ")
        );
    }

    println!("  Bullet: {}개", document.doc_info.bullets.len());
    for (i, bullet) in document.doc_info.bullets.iter().enumerate() {
        println!(
            "    [{}] char='{}' (U+{:04X})",
            i, bullet.bullet_char, bullet.bullet_char as u32
        );
    }

    // === ParaShape head_type 분포 ===
    println!("\n=== ParaShape head_type 분포 ===");
    let mut count_none = 0u32;
    let mut count_outline = 0u32;
    let mut count_number = 0u32;
    let mut count_bullet = 0u32;
    for ps in &document.doc_info.para_shapes {
        match ps.head_type {
            HeadType::None => count_none += 1,
            HeadType::Outline => count_outline += 1,
            HeadType::Number => count_number += 1,
            HeadType::Bullet => count_bullet += 1,
        }
    }
    println!(
        "  None: {}개, Outline: {}개, Number: {}개, Bullet: {}개",
        count_none, count_outline, count_number, count_bullet
    );

    // === SectionDef 개요번호 ===
    println!("\n=== SectionDef 개요번호 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        // SectionDef의 raw_ctrl_extra에서 바이트 14-15 추출 (outline_numbering_id)
        // 현재 outline_numbering_id 필드가 없으므로 파싱 전 상태에서는 raw_ctrl_extra 참조
        // 6단계에서 필드 추가 후 직접 참조로 변경 예정
        let sd = &section.section_def;
        let num_ref = if sd.outline_numbering_id > 0 {
            format!(" → Numbering[{}]", sd.outline_numbering_id - 1)
        } else {
            " (없음)".to_string()
        };
        println!(
            "  구역{}: outline_numbering_id={}{}, flags={:#010x}",
            sec_idx, sd.outline_numbering_id, num_ref, sd.flags
        );
    }

    // === 비None head_type 문단 ===
    println!("\n=== 비None head_type 문단 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(ps) = document
                .doc_info
                .para_shapes
                .get(para.para_shape_id as usize)
            {
                if ps.head_type != HeadType::None {
                    let text_preview: String = para.text.chars().take(40).collect();
                    let text_display = if para.text.chars().count() > 40 {
                        format!("\"{}...\"", text_preview)
                    } else {
                        format!("\"{}\"", text_preview)
                    };
                    println!(
                        "  구역{}:문단{} head={:?} level={} num_id={} text={}",
                        sec_idx,
                        para_idx,
                        ps.head_type,
                        ps.para_level,
                        ps.numbering_id,
                        text_display
                    );
                }
            }
        }
    }

    EXIT_OK
}

#[derive(Debug, Default, Clone, Copy)]
struct ConversionVerifyOptions {
    verify: bool,
    verify_pages: bool,
}

impl ConversionVerifyOptions {
    fn enabled(self) -> bool {
        self.verify || self.verify_pages
    }
}

fn parse_conversion_verify_args(
    args: &[String],
    usage: &str,
    min_positionals: usize,
    max_positionals: usize,
) -> Result<(Vec<String>, ConversionVerifyOptions), String> {
    let mut positionals = Vec::new();
    let mut options = ConversionVerifyOptions::default();

    for arg in args {
        match arg.as_str() {
            "--verify" => options.verify = true,
            "--verify-pages" => options.verify_pages = true,
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {}\n사용법: {}", value, usage));
            }
            value => positionals.push(value.to_string()),
        }
    }

    if positionals.len() < min_positionals || positionals.len() > max_positionals {
        return Err(format!("사용법: {}", usage));
    }

    Ok((positionals, options))
}

fn print_ir_verify_failure(diff: &rhwp::serializer::hwpx::roundtrip::IrDiff, converted: &str) {
    eprintln!(
        "검증 실패(--verify): {} 재파싱 후 IR 차이 {}건",
        converted,
        diff.differences.len()
    );
    for difference in diff.differences.iter().take(20) {
        eprintln!("  [차이] {}", difference);
    }
    if diff.differences.len() > 20 {
        eprintln!(
            "  ... 이하 생략 (총 {}건, 상세 비교는 ir-diff 사용)",
            diff.differences.len()
        );
    }
}

fn verify_reparse_failed_exit_code(options: ConversionVerifyOptions) -> i32 {
    if options.verify {
        3
    } else {
        4
    }
}

fn convert_hwp(args: &[String]) -> i32 {
    let (positionals, verify_options) = match parse_conversion_verify_args(
        args,
        "rhwp convert <입력.hwp|입력.hwpx> <출력.hwp> [--verify] [--verify-pages]",
        2,
        2,
    ) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{}", message);
            return EXIT_USAGE;
        }
    };

    let input_path = &positionals[0];
    let output_path = &positionals[1];

    // 입력 파일 읽기
    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", input_path, e);
            return EXIT_RUNTIME;
        }
    };

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count_before = if verify_options.verify_pages {
        Some(doc.page_count())
    } else {
        None
    };
    let was_distribution = doc.document().header.distribution;
    if !was_distribution {
        println!("{}: 이미 편집 가능한 문서입니다.", input_path);
    }

    // 변환
    match doc.convert_to_editable_native() {
        Ok(_) => {
            if was_distribution {
                println!("배포용 → 편집 가능 변환 완료");
            }
        }
        Err(e) => {
            eprintln!("오류: 변환 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    }

    // 직렬화
    match doc.export_hwp_with_adapter() {
        Ok(bytes) => match fs::write(output_path, &bytes) {
            Ok(_) => {
                println!("저장 완료: {} ({}KB)", output_path, bytes.len() / 1024);
                if verify_options.enabled() {
                    let reloaded = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("검증 실패: 저장된 HWP 재파싱 실패 - {}", e);
                            process::exit(verify_reparse_failed_exit_code(verify_options));
                        }
                    };

                    if let Some(before) = page_count_before {
                        let after = reloaded.page_count();
                        if before != after {
                            eprintln!(
                                "검증 실패(--verify-pages): 변환 전 {}쪽, 재파싱 후 {}쪽",
                                before, after
                            );
                            process::exit(4);
                        }
                        println!("검증 통과(--verify-pages): {}쪽", before);
                    }

                    if verify_options.verify {
                        let diff = rhwp::serializer::hwpx::roundtrip::diff_documents(
                            doc.document(),
                            reloaded.document(),
                        );
                        if !diff.is_empty() {
                            print_ir_verify_failure(&diff, output_path);
                            process::exit(3);
                        }
                        println!("검증 통과(--verify): IR 차이 없음");
                    }
                }
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 파일 저장 실패 - {}: {}", output_path, e);
                // [#2707] 출력 파일이 아예 안 만들어졌는데 0으로 끝나던 경로.
                EXIT_RUNTIME
            }
        },
        Err(e) => {
            eprintln!("오류: 직렬화 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

/// `rhwp export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx]` — HWP→HWPX 직접 변환 (#1868).
///
/// 파서가 포맷을 자동 감지(HWP5/HWP3/HWPX)해 `Document` IR 로 읽고
/// `export_hwpx_native()` 로 HWPX(ZIP) 직렬화한다. `convert`(배포용 해제 → .hwp 출력)와
/// 별개의 포맷 변환 명령. 출력 생략 시 입력과 같은 폴더에 `<stem>.hwpx`.
fn export_doclang(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp export-doclang <파일.hwp|파일.hwpx> [-o <출력.xml>] [--assets-dir <디렉터리>] (rhwp --help 참조)"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut output_override: Option<std::path::PathBuf> = None;
    let mut assets_dir: Option<std::path::PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_override = Some(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--assets-dir" => {
                if i + 1 < args.len() {
                    assets_dir = Some(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --assets-dir 뒤에 디렉터리 경로가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    // 기본 출력 경로: 입력 stem + `.dclg.xml` (입력 파일 옆).
    let input_path = std::path::Path::new(file_path);
    let output_path = output_override.unwrap_or_else(|| input_path.with_extension("dclg.xml"));
    if paths_refer_to_same_file(input_path, &output_path) {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        return EXIT_USAGE;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                input_path.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    };

    // 자원 정책: --assets-dir 지정 시 AssetDir(디렉터리 경로를 URI 접두어로), 아니면 인라인.
    let mut opts = rhwp::doclang::ConvertOptions::default();
    if let Some(dir) = &assets_dir {
        opts.resource_policy =
            rhwp::doclang::ResourcePolicy::asset_dir(dir.to_string_lossy().into_owned());
    }

    let outcome = match rhwp::doclang::convert(&data, &opts) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: DocLang 변환 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    // 이진 자원을 먼저 기록한다(있을 때만) — XML 저장 전에 실패를 드러내기 위함.
    if let Some(dir) = &assets_dir {
        if !outcome.assets.is_empty() {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!(
                    "오류: 에셋 디렉터리를 만들 수 없습니다 - {}: {}",
                    dir.display(),
                    e
                );
                return EXIT_RUNTIME;
            }
            for asset in &outcome.assets {
                let asset_path = dir.join(&asset.path);
                if let Err(e) = fs::write(&asset_path, &asset.data) {
                    eprintln!("오류: 에셋 저장 실패 - {}: {}", asset_path.display(), e);
                    return EXIT_RUNTIME;
                }
            }
        }
    }

    match fs::write(&output_path, outcome.xml.as_bytes()) {
        Ok(_) => {
            println!(
                "저장 완료: {} ({}KB)",
                output_path.display(),
                outcome.xml.len() / 1024
            );
            if let Some(dir) = &assets_dir {
                if !outcome.assets.is_empty() {
                    println!("에셋 {}개 저장: {}", outcome.assets.len(), dir.display());
                }
            }
            let loss_count = outcome.loss.len();
            if loss_count > 0 {
                println!(
                    "손실 보고: {}건 (DocLang v0.6 으로 표현할 수 없는 정보)",
                    loss_count
                );
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: 파일 저장 실패 - {}: {}", output_path.display(), e);
            EXIT_RUNTIME
        }
    }
}

fn export_hwpx(args: &[String]) -> i32 {
    let (positionals, verify_options) = match parse_conversion_verify_args(
        args,
        "rhwp export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx] [--verify] [--verify-pages]",
        1,
        2,
    ) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{}", message);
            return EXIT_USAGE;
        }
    };

    let input_path = std::path::Path::new(&positionals[0]);
    let output_path = match positionals.get(1) {
        Some(p) => std::path::PathBuf::from(p),
        None => input_path.with_extension("hwpx"),
    };
    if output_path
        .extension()
        .map(|e| !e.eq_ignore_ascii_case("hwpx"))
        .unwrap_or(true)
    {
        eprintln!(
            "경고: 출력 확장자가 .hwpx 가 아닙니다: {}",
            output_path.display()
        );
    }
    if output_path == input_path {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        return EXIT_USAGE;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                input_path.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 문서 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count_before = if verify_options.verify_pages {
        Some(doc.page_count())
    } else {
        None
    };

    match doc.export_hwpx_native() {
        Ok(bytes) => match fs::write(&output_path, &bytes) {
            Ok(_) => {
                println!(
                    "저장 완료: {} ({}KB)",
                    output_path.display(),
                    bytes.len() / 1024
                );
                if verify_options.enabled() {
                    let reloaded = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("검증 실패: 저장된 HWPX 재파싱 실패 - {}", e);
                            process::exit(verify_reparse_failed_exit_code(verify_options));
                        }
                    };

                    if let Some(before) = page_count_before {
                        let after = reloaded.page_count();
                        if before != after {
                            eprintln!(
                                "검증 실패(--verify-pages): 변환 전 {}쪽, 재파싱 후 {}쪽",
                                before, after
                            );
                            process::exit(4);
                        }
                        println!("검증 통과(--verify-pages): {}쪽", before);
                    }

                    if verify_options.verify {
                        let diff = rhwp::serializer::hwpx::roundtrip::diff_documents(
                            doc.document(),
                            reloaded.document(),
                        );
                        if !diff.is_empty() {
                            print_ir_verify_failure(&diff, &output_path.display().to_string());
                            process::exit(3);
                        }
                        println!("검증 통과(--verify): IR 차이 없음");
                    }
                }
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 파일 저장 실패 - {}: {}", output_path.display(), e);
                // [#2707] 출력 파일이 아예 안 만들어졌는데 0으로 끝나던 경로.
                EXIT_RUNTIME
            }
        },
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

struct HmlExportArgs {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
}

fn parse_hml_export_args(args: &[String]) -> Result<HmlExportArgs, String> {
    let usage = "rhwp export-hml <입력.hml> -o <출력.hml>";
    let mut input = None;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--output" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("출력 경로가 필요합니다\n사용법: {usage}"))?;
                if value.starts_with('-') {
                    return Err(format!("출력 경로가 필요합니다\n사용법: {usage}"));
                }
                if output.replace(std::path::PathBuf::from(value)).is_some() {
                    return Err(format!("출력 경로를 한 번만 지정하세요\n사용법: {usage}"));
                }
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {value}\n사용법: {usage}"));
            }
            value => {
                if input.replace(std::path::PathBuf::from(value)).is_some() {
                    return Err(format!("입력 파일을 하나만 지정하세요\n사용법: {usage}"));
                }
                index += 1;
            }
        }
    }
    Ok(HmlExportArgs {
        input: input.ok_or_else(|| format!("입력 파일이 필요합니다\n사용법: {usage}"))?,
        output: output.ok_or_else(|| format!("출력 경로가 필요합니다\n사용법: {usage}"))?,
    })
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> bool {
    input == output
        || paths_have_same_file_identity(input, output)
        || match (input.canonicalize(), output.canonicalize()) {
            (Ok(input), Ok(output)) => input == output,
            _ => false,
        }
}

#[cfg(unix)]
fn paths_have_same_file_identity(input: &Path, output: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    match (input.metadata(), output.metadata()) {
        (Ok(input), Ok(output)) => input.dev() == output.dev() && input.ino() == output.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn paths_have_same_file_identity(_input: &Path, _output: &Path) -> bool {
    false
}

fn print_hml_export_error(error: &rhwp::serializer::hml::HmlExportError) {
    eprintln!("오류: {error}");
    for blocker in error.blockers() {
        eprintln!(
            "  [{}] {}: {}",
            blocker.code, blocker.xml_path, blocker.message
        );
    }
}

fn export_hml(args: &[String]) {
    let paths = parse_hml_export_args(args).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(2);
    });
    if paths_refer_to_same_file(&paths.input, &paths.output) {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        process::exit(2);
    }
    let data = fs::read(&paths.input).unwrap_or_else(|error| {
        eprintln!(
            "오류: 파일을 읽을 수 없습니다 - {}: {error}",
            paths.input.display()
        );
        process::exit(1);
    });
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).unwrap_or_else(|error| {
        eprintln!("오류: 문서 파싱 실패 - {error}");
        process::exit(1);
    });
    let bytes = core.export_hml_native().unwrap_or_else(|error| {
        print_hml_export_error(&error);
        process::exit(1);
    });
    atomic_file::write_atomically(&paths.output, &bytes).unwrap_or_else(|error| {
        eprintln!("오류: 파일 저장 실패 - {}: {error}", paths.output.display());
        process::exit(1);
    });
    println!(
        "저장 완료: {} ({}KB)",
        paths.output.display(),
        bytes.len() / 1024
    );
}

/// `rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>`
///
/// Claude Code Skill (`rhwp-exam-ingest`)이 생성한 JSON 중간 표현을 HWPX로 변환한다.
/// Task #660 (Neumann 본 작업 1단계).
fn build_from_ingest(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
        return EXIT_USAGE;
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut media_dir: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: -o 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                output_path = Some(&args[i + 1]);
                i += 2;
            }
            "--media-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --media-dir 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                media_dir = Some(&args[i + 1]);
                i += 2;
            }
            other => {
                if input_path.is_none() {
                    input_path = Some(other);
                } else {
                    eprintln!("경고: 알 수 없는 인자 '{}' 무시", other);
                }
                i += 1;
            }
        }
    }

    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("오류: 입력 ingest JSON 경로가 누락되었습니다");
            return EXIT_USAGE;
        }
    };
    let output = match output_path {
        Some(p) => p,
        None => {
            eprintln!("오류: -o <출력 경로> 가 누락되었습니다");
            return EXIT_USAGE;
        }
    };

    let bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력 파일 읽기 실패 - {}: {}", input, e);
            return EXIT_RUNTIME;
        }
    };

    let ingest = match rhwp::parser::ingest::parse_ingest_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: ingest JSON 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(md) = media_dir {
        let p = Path::new(md);
        if !p.exists() {
            eprintln!(
                "경고: 미디어 디렉토리가 존재하지 않습니다 ({}). 본 단계는 이미지 placeholder로 처리됩니다.",
                md
            );
        }
    }

    let doc = rhwp::document_core::builders::exam_paper::build_exam_paper(&ingest);

    let hwpx_bytes = match rhwp::serializer::serialize_hwpx(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match fs::write(output, &hwpx_bytes) {
        Ok(_) => {
            println!(
                "저장 완료: {} ({}바이트, 문제 {}개, 문단 {}개)",
                output,
                hwpx_bytes.len(),
                ingest.questions.len(),
                doc.sections
                    .iter()
                    .map(|s| s.paragraphs.len())
                    .sum::<usize>()
            );
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
            EXIT_RUNTIME
        }
    }
}

fn dump_raw_records(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-records <파일.hwp>");
        return EXIT_USAGE;
    }
    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {}", e);
            return EXIT_RUNTIME;
        }
    };
    use rhwp::parser::cfb_reader::CfbReader;
    use rhwp::parser::record::Record;
    let mut cfb = match CfbReader::open(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return EXIT_RUNTIME;
        }
    };
    // FileHeader에서 압축 여부 확인
    let header = cfb.read_stream_raw("FileHeader").unwrap_or_default();
    let compressed = header.len() >= 40 && (header[36] & 0x01) != 0;
    let section = match cfb.read_body_text_section(0, compressed, false) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return EXIT_RUNTIME;
        }
    };
    let records = match Record::read_all(&section) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return EXIT_RUNTIME;
        }
    };
    let tag_name = |id: u16| -> &str {
        match id {
            66 => "PARA_HEADER",
            67 => "PARA_TEXT",
            68 => "PARA_CHAR_SHAPE",
            69 => "PARA_LINE_SEG",
            70 => "PARA_RANGE_TAG",
            71 => "CTRL_HEADER",
            72 => "LIST_HEADER",
            73 => "PAGE_DEF",
            74 => "FOOTNOTE_SHAPE",
            75 => "PAGE_BORDER_FILL",
            76 => "SHAPE_COMPONENT",
            77 => "TABLE",
            78 => "SC_LINE",
            79 => "SC_RECT",
            80 => "SC_ELLIPSE",
            81 => "SC_ARC",
            82 => "SC_POLYGON",
            83 => "SC_CURVE",
            85 => "SC_PICTURE",
            86 => "SC_CONTAINER",
            89 => "CTRL_DATA",
            _ => "?",
        }
    };
    for (i, rec) in records.iter().enumerate() {
        let indent = "  ".repeat(rec.level as usize);
        println!(
            "[{:3}] {}tag={:<3} {:16} lv={} sz={}",
            i,
            indent,
            rec.tag_id,
            tag_name(rec.tag_id),
            rec.level,
            rec.data.len()
        );
        // shape 관련 레코드만 hex 덤프
        if matches!(rec.tag_id, 71 | 72 | 76 | 79 | 85 | 89) {
            // 16바이트씩 나눠서 hex 출력
            for chunk in rec.data.chunks(16) {
                let hex: String = chunk
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("       {}  {}", indent, hex);
            }
        }
    }
    EXIT_OK
}

fn test_shape_roundtrip(args: &[String]) {
    let input = if args.is_empty() {
        "saved/g555-s.hwp"
    } else {
        &args[0]
    };
    let output = if args.len() > 1 {
        &args[1]
    } else {
        "/tmp/test-shape-out.hwp"
    };

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("입력 파일 읽기 오류: {}", e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("HWP 파싱 오류: {:?}", e);
            return;
        }
    };

    let _ = doc.convert_to_editable_native();

    // 글상자 생성 (9000 x 6750 HWPUNIT)
    let result = doc.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6750,
        0,
        0,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    );
    match &result {
        Ok(r) => eprintln!("글상자 생성 성공: {}", r),
        Err(e) => {
            eprintln!("글상자 생성 실패: {:?}", e);
            return;
        }
    }

    match doc.export_hwp_native() {
        Ok(bytes) => {
            if let Err(e) = fs::write(output, &bytes) {
                eprintln!("파일 저장 오류: {}", e);
            } else {
                eprintln!("저장 완료: {} ({}KB)", output, bytes.len() / 1024);
            }
        }
        Err(e) => eprintln!("직렬화 오류: {:?}", e),
    }
}

/// 캡션 방향별 테스트: 4개 이미지에 각각 Bottom/Top/Left/Right 캡션을 설정하고 SVG 출력
fn test_caption(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp test-caption <파일.hwp>");
        return;
    }

    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파일 읽기 오류: {}", e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파싱 오류: {}", e);
            return;
        }
    };

    // 문단 0: 컨트롤 2,3 / 문단 1: 컨트롤 0,1
    let pic_refs: [(usize, usize); 4] = [(0, 2), (0, 3), (1, 0), (1, 1)];

    // 4개 이미지에 각각 다른 캡션 방향 설정
    let directions = [
        ("Bottom", "Top"),
        ("Top", "Top"),
        ("Left", "Center"),
        ("Right", "Center"),
    ];

    for (i, ((para, ci), (dir, va))) in pic_refs.iter().zip(directions.iter()).enumerate() {
        let json = format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":8504,"captionSpacing":850}}"#,
            dir, va
        );
        println!("[{}] para={}, ci={}, dir={}, va={}", i, para, ci, dir, va);
        match doc.set_picture_properties_native(0, *para, *ci, &json) {
            Ok(r) => println!("  결과: {}", r),
            Err(e) => println!("  오류: {:?}", e),
        }
    }

    // 캡션 상태 확인
    for (i, (para, ci)) in pic_refs.iter().enumerate() {
        let section = &doc.document().sections[0];
        let p = &section.paragraphs[*para];
        if let rhwp::model::control::Control::Picture(pic) = &p.controls[*ci] {
            println!(
                "[{}] caption={:?}",
                i,
                pic.caption.as_ref().map(|c| {
                    format!(
                        "dir={:?}, paras={}, text={:?}",
                        c.direction,
                        c.paragraphs.len(),
                        c.paragraphs.first().map(|p| &p.text)
                    )
                })
            );
        }
    }

    // SVG 출력
    let output_dir = "output/caption-test";
    let _ = fs::create_dir_all(output_dir);
    let page_count = doc.page_count();
    println!("페이지 수: {}", page_count);
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더링 오류");
        let path = format!("{}/caption-test-p{}.svg", output_dir, p);
        fs::write(&path, &svg).unwrap();
        println!("  → {}", path);
    }
    println!("완료");
}

fn gen_table(args: &[String]) {
    let rows: u16 = args.first().and_then(|s| s.parse().ok()).unwrap_or(1000);
    let cols: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let output = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("output/gen_table.hwp");

    println!("{}행 × {}열 표 생성 중...", rows, cols);

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // 표 생성
    let result = core
        .create_table_native(0, 0, 0, rows, cols)
        .expect("표 생성 실패");
    println!("  표 생성: {}", result);

    // 결과에서 paraIdx 파싱
    let table_para_idx: usize = result
        .split("\"paraIdx\":")
        .nth(1)
        .and_then(|s| s.split(&[',', '}'][..]).next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    println!("  표 문단 인덱스: {}", table_para_idx);

    // 배치 모드로 셀 내용 채우기
    core.begin_batch_native().expect("배치 시작 실패");

    let headers = ["번호", "이름", "부서", "직급", "연락처", "비고"];
    // 헤더 행
    for (ci, header) in headers.iter().enumerate().take(cols as usize) {
        let _ = core.insert_text_in_cell_native(0, table_para_idx, 0, ci, 0, 0, header);
    }

    // 데이터 행
    let departments = ["개발팀", "기획팀", "디자인팀", "영업팀", "인사팀", "재무팀"];
    let positions = ["사원", "대리", "과장", "차장", "부장"];
    for row in 1..rows as usize {
        for col in 0..cols as usize {
            let cell_idx = row * cols as usize + col;
            let text = match col {
                0 => format!("{}", row),
                1 => format!("홍길동{}", row),
                2 => departments[row % departments.len()].to_string(),
                3 => positions[row % positions.len()].to_string(),
                4 => format!(
                    "010-{:04}-{:04}",
                    1000 + row % 9000,
                    1000 + (row * 7) % 9000
                ),
                5 => {
                    if row % 3 == 0 {
                        "특이사항 없음".to_string()
                    } else {
                        String::new()
                    }
                }
                _ => format!("R{}C{}", row, col),
            };
            if !text.is_empty() {
                let _ =
                    core.insert_text_in_cell_native(0, table_para_idx, 0, cell_idx, 0, 0, &text);
            }
        }
        if row % 100 == 0 {
            println!("  {} / {} 행 완료", row, rows);
        }
    }

    core.end_batch_native().expect("배치 종료 실패");
    println!("  셀 내용 입력 완료");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({}행 × {}열)", output, rows, cols);
}

/// PUA (Private Use Area) 문자 셋트를 입력한 HWP 테스트 문서 생성.
///
/// Task #509 (PUA 회귀 정정) 의 한컴 정답지 확보용. 본 라이브러리가 발견한
/// 14 샘플 광범위 PUA 코드포인트 18 종을 한 문서에 입력 → 한컴 편집기로 PDF
/// 출력 + rhwp SVG 출력 시각 비교.
///
/// 사용:
///   rhwp gen-pua [output_path]
///   기본 출력: output/pua-test.hwp
fn gen_pua_test(args: &[String]) {
    let output = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("output/pua-test.hwp");

    println!("PUA 문자 셋트 입력 HWP 문서 생성 중...");

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // PUA 코드포인트 셋트 (Task #509 Stage 1 의 14 샘플 광범위 통계 정합)
    // (codepoint, 영역 분류, 사용 샘플, 본 라이브러리 현재 매핑)
    let pua_set: &[(u32, &str, &str, &str)] = &[
        // ── Basic PUA (0xF020~0xF0FF) — 매핑 표 적용 영역 ──
        (0x0F076, "Basic", "mel-001", "❖ U+2756"),
        (0x0F09F, "Basic", "biz_plan", "• U+2022"),
        (0x0F0A0, "Basic", "synam-001", "▪ U+25AA"),
        (0x0F0A7, "Basic", "kps-ai", "▪ U+25AA"),
        (0x0F0E8, "Basic", "kps-ai", "(미정의)"),
        (0x0F0F2, "Basic", "KTX", "⇩ U+21E9 (의도 정정 후보)"),
        (0x0F0FE, "Basic", "k-water-rfp", "☑ U+2611"),
        // ── Basic PUA — 매핑 표 외 영역 ──
        (0x0F53A, "Basic-out", "hwpspec", "(매핑 표 외)"),
        // ── Supplementary PUA-A (0xF0000~0xFFFFD) — 매핑 표 미지원 영역 ──
        (0xF02B1, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B2, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B3, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B4, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B5, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B6, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B7, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B8, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B9, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02EF, "Suppl-A", "KTX (회귀)", "(매핑 표 외) ★"),
    ];

    println!("  PUA 코드포인트 {} 종 입력", pua_set.len());

    core.begin_batch_native().expect("배치 시작 실패");

    // 첫 paragraph (0번) 에 제목 입력
    let title = "[PUA 회귀 검증 — Task #509]";
    core.insert_text_native(0, 0, 0, title)
        .expect("제목 입력 실패");

    // 각 PUA 글자별로 paragraph 추가:
    // "U+0F0F2 (Basic, KTX): {char}    ← 한컴 정답지 / rhwp 비교"
    // 빈 paragraph 추가 + 텍스트 입력 패턴
    for (i, &(cp, area, sample, mapping)) in pua_set.iter().enumerate() {
        let pi = i + 1; // 0번은 제목, 1번부터 PUA paragraphs

        // 새 paragraph 추가 (pi 위치에 새 문단 삽입)
        core.insert_paragraph_native(0, pi)
            .unwrap_or_else(|e| panic!("paragraph 추가 실패 (pi={}): {:?}", pi, e));

        // PUA 글자 char 변환 (i32 unsafe 회피)
        let pua_char =
            char::from_u32(cp).unwrap_or_else(|| panic!("invalid codepoint U+{:05X}", cp));

        // 텍스트: "U+0F0F2 (Basic, KTX, ⇩ U+21E9 매핑): " + PUA + "  ← 한컴 PDF 글리프 정답지"
        let text = format!(
            "U+{:05X} ({}, {}, {}): {}  ← 한컴 PDF 정답지",
            cp, area, sample, mapping, pua_char
        );

        core.insert_text_native(0, pi, 0, &text)
            .unwrap_or_else(|e| panic!("텍스트 입력 실패 (pi={}): {:?}", pi, e));
    }

    core.end_batch_native().expect("배치 종료 실패");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({} 종 PUA)", output, pua_set.len());
    println!();
    println!("다음 단계:");
    println!("  1. 한컴 2022 편집기에서 본 파일 열기 → PDF 출력 (정답지)");
    println!("  2. rhwp export-svg {} → SVG 출력 비교", output);
    println!("  3. 시각 비교로 매핑 정합 확정");
}

fn test_field_roundtrip(args: &[String]) {
    let input = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("hwp_webctl/bsbc01_10_000.hwp");
    let output = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("output/field_test.hwp");

    let data = std::fs::read(input).expect("파일 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("문서 파싱 실패");

    // 1. 필드 목록 출력
    let fields = core.collect_all_fields();
    println!("=== 필드 목록 ({}개) ===", fields.len());
    for fi in &fields {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 2. 필드에 값 설정
    let test_data = [
        ("mbizNm", "청소년 자립지원사업"),
        ("newCtnuTxt", "계속"),
        ("chargerNm", "홍길동"),
        ("telno", "02-1234-5678"),
        ("sFisYear", "2026"),
        // 셀 필드
        ("bizPurps", "청소년 자립 역량 강화"),
        ("bizPrdTxt", "2026.01 ~ 2026.12"),
        ("insttNm", "시청 복지과"),
    ];

    println!("\n=== 필드 값 설정 ===");
    for (name, value) in &test_data {
        match core.set_field_value_by_name(name, value) {
            Ok(r) => println!("  ✓ {} = \"{}\" → {}", name, value, r),
            Err(e) => println!("  ✗ {} = \"{}\" → {}", name, value, e),
        }
    }

    // 3. 설정 후 확인
    println!("\n=== 설정 후 확인 ===");
    let fields2 = core.collect_all_fields();
    for fi in &fields2 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 3.5 pi=0 문단 텍스트 직접 확인
    let para0 = &core.document().sections[0].paragraphs[0];

    // 4. 직렬화 → 저장
    let saved = core.export_hwp_native().expect("직렬화 실패");
    std::fs::write(output, &saved).expect("저장 실패");
    println!("\n저장: {} ({}바이트)", output, saved.len());

    // 5. 재로딩 → 필드 확인
    let mut core2 = rhwp::document_core::DocumentCore::from_bytes(&saved).expect("재로딩 실패");
    let fields3 = core2.collect_all_fields();
    println!("\n=== 재로딩 후 확인 ===");
    for fi in &fields3 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }
}

fn control_tag(c: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match c {
        Control::SectionDef(_) => "secd",
        Control::ColumnDef(_) => "cold",
        Control::Table(_) => "tbl",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "pic",
        Control::Header(_) => "head",
        Control::Footer(_) => "foot",
        Control::Footnote(_) => "fn",
        Control::Endnote(_) => "en",
        Control::AutoNumber(_) => "atno",
        Control::NewNumber(_) => "nwno",
        Control::PageNumberPos(_) => "pgnp",
        Control::Bookmark(_) => "bokm",
        Control::Hyperlink(_) => "hlk",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "tcps",
        Control::PageHide(_) => "pghd",
        Control::HiddenComment(_) => "tcmt",
        Control::Equation(_) => "eqed",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

fn diff_table(
    diffs: &mut Vec<String>,
    ci: usize,
    a: &rhwp::model::table::Table,
    b: &rhwp::model::table::Table,
) {
    if a.row_count != b.row_count {
        diffs.push(format!(
            "ctrl[{}] tbl rows: A={} vs B={}",
            ci, a.row_count, b.row_count
        ));
    }
    if a.col_count != b.col_count {
        diffs.push(format!(
            "ctrl[{}] tbl cols: A={} vs B={}",
            ci, a.col_count, b.col_count
        ));
    }
    if a.page_break != b.page_break {
        diffs.push(format!(
            "ctrl[{}] tbl page_break: A={:?} vs B={:?}",
            ci, a.page_break, b.page_break
        ));
    }
    if a.repeat_header != b.repeat_header {
        diffs.push(format!(
            "ctrl[{}] tbl repeat_header: A={} vs B={}",
            ci, a.repeat_header, b.repeat_header
        ));
    }
    if a.cell_spacing != b.cell_spacing {
        diffs.push(format!(
            "ctrl[{}] tbl cell_spacing: A={} vs B={}",
            ci, a.cell_spacing, b.cell_spacing
        ));
    }
    if a.border_fill_id != b.border_fill_id {
        diffs.push(format!(
            "ctrl[{}] tbl border_fill_id: A={} vs B={}",
            ci, a.border_fill_id, b.border_fill_id
        ));
    }
    if a.outer_margin_left != b.outer_margin_left
        || a.outer_margin_right != b.outer_margin_right
        || a.outer_margin_top != b.outer_margin_top
        || a.outer_margin_bottom != b.outer_margin_bottom
    {
        diffs.push(format!(
            "ctrl[{}] tbl outer_margin: A=({},{},{},{}) vs B=({},{},{},{})",
            ci,
            a.outer_margin_left,
            a.outer_margin_top,
            a.outer_margin_right,
            a.outer_margin_bottom,
            b.outer_margin_left,
            b.outer_margin_top,
            b.outer_margin_right,
            b.outer_margin_bottom,
        ));
    }
    diff_common_obj(diffs, ci, "tbl", &a.common, &b.common);
}

fn diff_common_obj(
    diffs: &mut Vec<String>,
    ci: usize,
    tag: &str,
    a: &rhwp::model::shape::CommonObjAttr,
    b: &rhwp::model::shape::CommonObjAttr,
) {
    if a.treat_as_char != b.treat_as_char {
        diffs.push(format!(
            "ctrl[{}] {} tac: A={} vs B={}",
            ci, tag, a.treat_as_char, b.treat_as_char
        ));
    }
    if a.text_wrap != b.text_wrap {
        diffs.push(format!(
            "ctrl[{}] {} wrap: A={:?} vs B={:?}",
            ci, tag, a.text_wrap, b.text_wrap
        ));
    }
    if a.width != b.width || a.height != b.height {
        diffs.push(format!(
            "ctrl[{}] {} size: A={}x{} vs B={}x{}",
            ci, tag, a.width, a.height, b.width, b.height
        ));
    }
    if a.vertical_offset != b.vertical_offset {
        diffs.push(format!(
            "ctrl[{}] {} v_offset: A={} vs B={}",
            ci, tag, a.vertical_offset, b.vertical_offset
        ));
    }
    if a.horizontal_offset != b.horizontal_offset {
        diffs.push(format!(
            "ctrl[{}] {} h_offset: A={} vs B={}",
            ci, tag, a.horizontal_offset, b.horizontal_offset
        ));
    }
    if a.vert_rel_to != b.vert_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} vert_rel: A={:?} vs B={:?}",
            ci, tag, a.vert_rel_to, b.vert_rel_to
        ));
    }
    if a.horz_rel_to != b.horz_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} horz_rel: A={:?} vs B={:?}",
            ci, tag, a.horz_rel_to, b.horz_rel_to
        ));
    }
}

/// [#1807] 글상자 문단 한 쌍의 핵심 필드 비교 — 본문 문단 비교의 축약판.
/// 직렬화 결함(#1795: FIELD_END 갭 선점 → char_offsets 시프트)이 글상자 안에서
/// 발생해도 ir-diff 가 검출하도록 text/cc/char_offsets/char_shapes/line_segs/
/// field_ranges 를 비교한다.
fn diff_textbox_paragraph_fields(
    diffs: &mut Vec<String>,
    prefix: &str,
    pa: &rhwp::model::paragraph::Paragraph,
    pb: &rhwp::model::paragraph::Paragraph,
) {
    if pa.text != pb.text {
        diffs.push(format!(
            "{} text: A={:?} vs B={:?}",
            prefix,
            pa.text.chars().take(30).collect::<String>(),
            pb.text.chars().take(30).collect::<String>()
        ));
    }
    if pa.char_count != pb.char_count {
        diffs.push(format!(
            "{} cc: A={} vs B={}",
            prefix, pa.char_count, pb.char_count
        ));
    }
    if pa.char_offsets != pb.char_offsets {
        if pa.char_offsets.len() != pb.char_offsets.len() {
            diffs.push(format!(
                "{} char_offsets len: A={} vs B={}",
                prefix,
                pa.char_offsets.len(),
                pb.char_offsets.len()
            ));
        } else if let Some((idx, (a, b))) = pa
            .char_offsets
            .iter()
            .zip(pb.char_offsets.iter())
            .enumerate()
            .find(|(_, (a, b))| a != b)
        {
            diffs.push(format!(
                "{} char_offsets[{}]: A={} vs B={}",
                prefix, idx, a, b
            ));
        }
    }
    if pa.char_shapes.len() != pb.char_shapes.len() {
        diffs.push(format!(
            "{} char_shapes count: A={} vs B={}",
            prefix,
            pa.char_shapes.len(),
            pb.char_shapes.len()
        ));
    } else if let Some((idx, (ca, cb))) = pa
        .char_shapes
        .iter()
        .zip(pb.char_shapes.iter())
        .enumerate()
        .find(|(_, (ca, cb))| ca.start_pos != cb.start_pos || ca.char_shape_id != cb.char_shape_id)
    {
        diffs.push(format!(
            "{} cs[{}]: A=({},{}) vs B=({},{})",
            prefix, idx, ca.start_pos, ca.char_shape_id, cb.start_pos, cb.char_shape_id
        ));
    }
    if pa.line_segs.len() != pb.line_segs.len() {
        diffs.push(format!(
            "{} line_segs count: A={} vs B={}",
            prefix,
            pa.line_segs.len(),
            pb.line_segs.len()
        ));
    } else if let Some((idx, (la, lb))) = pa
        .line_segs
        .iter()
        .zip(pb.line_segs.iter())
        .enumerate()
        .find(|(_, (la, lb))| la.text_start != lb.text_start || la.vertical_pos != lb.vertical_pos)
    {
        diffs.push(format!(
            "{} ls[{}]: A=(ts={},vpos={}) vs B=(ts={},vpos={})",
            prefix, idx, la.text_start, la.vertical_pos, lb.text_start, lb.vertical_pos
        ));
    }
    if pa.field_ranges.len() != pb.field_ranges.len() {
        diffs.push(format!(
            "{} field_ranges count: A={} vs B={}",
            prefix,
            pa.field_ranges.len(),
            pb.field_ranges.len()
        ));
    } else if let Some((idx, (fa, fb))) = pa
        .field_ranges
        .iter()
        .zip(pb.field_ranges.iter())
        .enumerate()
        .find(|(_, (fa, fb))| {
            fa.start_char_idx != fb.start_char_idx
                || fa.end_char_idx != fb.end_char_idx
                || fa.control_idx != fb.control_idx
        })
    {
        diffs.push(format!(
            "{} field_ranges[{}]: A=({}..{},c{}) vs B=({}..{},c{})",
            prefix,
            idx,
            fa.start_char_idx,
            fa.end_char_idx,
            fa.control_idx,
            fb.start_char_idx,
            fb.end_char_idx,
            fb.control_idx
        ));
    }
}

/// [#1807] 글상자 문단 목록 재귀 비교. 중첩 글상자(Shape in Shape)도 재귀한다.
fn diff_textbox_paragraph_lists(
    diffs: &mut Vec<String>,
    prefix: &str,
    pas: &[rhwp::model::paragraph::Paragraph],
    pbs: &[rhwp::model::paragraph::Paragraph],
) {
    use rhwp::model::control::Control;
    if pas.len() != pbs.len() {
        diffs.push(format!(
            "{} tb 문단 수: A={} vs B={}",
            prefix,
            pas.len(),
            pbs.len()
        ));
    }
    for (k, (pa, pb)) in pas.iter().zip(pbs.iter()).enumerate() {
        let p = format!("{} tb_p[{}]", prefix, k);
        diff_textbox_paragraph_fields(diffs, &p, pa, pb);
        for (cj, (ca, cb)) in pa.controls.iter().zip(pb.controls.iter()).enumerate() {
            if let (Control::Shape(sa), Control::Shape(sb)) = (ca, cb) {
                diff_shape_textbox(diffs, &format!("{}.ctrl[{}]", p, cj), sa, sb);
            }
        }
    }
}

/// [#1807] Shape 글상자 유무 + 내부 문단 재귀 비교 진입점.
fn diff_shape_textbox(
    diffs: &mut Vec<String>,
    prefix: &str,
    sa: &rhwp::model::shape::ShapeObject,
    sb: &rhwp::model::shape::ShapeObject,
) {
    let ta = sa.drawing().and_then(|d| d.text_box.as_ref());
    let tb = sb.drawing().and_then(|d| d.text_box.as_ref());
    match (ta, tb) {
        (Some(ta), Some(tb)) => {
            diff_textbox_paragraph_lists(diffs, prefix, &ta.paragraphs, &tb.paragraphs);
        }
        (Some(_), None) | (None, Some(_)) => {
            diffs.push(format!(
                "{} text_box 유무: A={} vs B={}",
                prefix,
                ta.is_some(),
                tb.is_some()
            ));
        }
        (None, None) => {}
    }
}

/// `tab_extended`(`[u16; 7]`) 두 인라인 탭 레코드가 **의미 있는** 필드에서 다른지 판정.
///
/// HWPX 파서(`parse_tab_extension`)는 인라인 탭을 `ext[0]`=width,
/// `ext[2]`=`type<<8 | leader`(leader 는 low byte), `ext[6]`=0x0009 마커로만 채우고
/// `ext[1]`·`ext[3]`·`ext[4]`·`ext[5]`는 0 으로 둔다. HWPX 직렬화(`render_hp_t_content`)도
/// width/leader/type 를 오직 `ext[0]`·`ext[2]`에서만 읽는다. 반면 HWP5 인라인 탭(8 WCHAR
/// 블록)은 `ext[1]`을 leader/fill 슬롯으로, `ext[3]`·`ext[4]`·`ext[5]`를 WCHAR 4~6 원본
/// 바이트(보통 0x20)로 채운다 — 이들은 HWPX `<hp:tab>`에 대응 속성이 없어 HWPX 쪽이 항상
/// 0 이라, HWPX↔HWP5 parity 비교에서 거의 모든 탭에 거짓 차이(0 vs leader, 0 vs 32)를 만들어
/// 실제 차이(width/type/leader)를 가린다. 따라서 두 포맷이 공통으로 쓰는 필드
/// [0]=width, [2]=type/leader 팩, [6]=마커만 비교하고 [1],[3],[4],[5]는 제외한다.
/// (HWP5 직렬화는 [1],[3..6]을 그대로 보존하므로 self-roundtrip 충실도에는 영향 없음 —
/// 도구 비교에서만 제외.)
fn tab_ext_semantic_differs(a: &[u16; 7], b: &[u16; 7]) -> bool {
    // 두 포맷 공통 필드만: [0]=width, [2]=type<<8|leader, [6]=0x0009 마커.
    // [1](HWP5 leader/fill 슬롯, HWPX=0)·[3]·[4]·[5](HWP5 예약 바이트, HWPX=0)는 제외.
    const SEMANTIC: [usize; 3] = [0, 2, 6];
    SEMANTIC.iter().any(|&k| a[k] != b[k])
}

/// [Task #2122] ir-diff 출력 상태 — 종전 fn-지역 macro(emit_header/emit_diff) 본문을
/// 메서드로 이관 (동작·출력 불변, macro 확장 인라인 제거).
struct IrDiffEmitter {
    summary_mode: bool,
    max_lines: Option<usize>,
    printed_lines: usize,
    truncated: bool,
    summary_buckets: std::collections::BTreeMap<String, u32>,
}

impl IrDiffEmitter {
    fn println_guarded(&mut self, line: String) {
        match self.max_lines {
            Some(limit) if self.printed_lines >= limit => {
                if !self.truncated {
                    println!("... 이하 생략 (--max-lines {} 도달)", limit);
                    self.truncated = true;
                }
            }
            _ => {
                println!("{}", line);
                self.printed_lines += 1;
            }
        }
    }
    /// paragraph/섹션 헤더. summary 모드에서는 출력 안 함, max_lines 초과 시 truncate.
    fn header(&mut self, line: String) {
        if !self.summary_mode {
            self.println_guarded(line);
        }
    }
    /// 차이 라인. summary 모드에서는 카테고리별 카운트, 일반 모드에서는 "  [차이] {}" 형식.
    /// 카테고리 추출: ":" 앞쪽 첫 토큰. controls[N].xxx 는 ".xxx" 만 추출.
    fn diff(&mut self, body: String) {
        if self.summary_mode {
            let prefix = body.split(':').next().unwrap_or(&body);
            let cat = if let Some(pos) = prefix.rfind(']') {
                prefix[pos + 1..].trim_start_matches('.').trim().to_string()
            } else {
                prefix.trim().to_string()
            };
            let key = if cat.is_empty() { body.clone() } else { cat };
            *self.summary_buckets.entry(key).or_insert(0) += 1;
        } else {
            self.println_guarded(format!("  [차이] {}", body));
        }
    }
}

/// [Task #2122] ir-diff 문단 단위 필드 비교 — 차이 문자열 목록 생산 (원본 무변경 이동).
fn ir_diff_paragraph_fields(
    pa: &rhwp::model::paragraph::Paragraph,
    pb: &rhwp::model::paragraph::Paragraph,
    doc_a: &rhwp::model::document::Document,
    doc_b: &rhwp::model::document::Document,
) -> Vec<String> {
    let mut diffs: Vec<String> = Vec::new();

    // 텍스트 비교
    if pa.text != pb.text {
        diffs.push(format!(
            "text: A={:?} vs B={:?}",
            pa.text.chars().take(30).collect::<String>(),
            pb.text.chars().take(30).collect::<String>()
        ));
    }

    // char_count 비교
    if pa.char_count != pb.char_count {
        diffs.push(format!("cc: A={} vs B={}", pa.char_count, pb.char_count));
    }

    // char_offsets 비교
    if pa.char_offsets != pb.char_offsets {
        let len_a = pa.char_offsets.len();
        let len_b = pb.char_offsets.len();
        if len_a != len_b {
            diffs.push(format!("char_offsets len: A={} vs B={}", len_a, len_b));
        } else {
            let first_diff = pa
                .char_offsets
                .iter()
                .zip(pb.char_offsets.iter())
                .enumerate()
                .find(|(_, (a, b))| a != b);
            if let Some((idx, (a, b))) = first_diff {
                diffs.push(format!("char_offsets[{}]: A={} vs B={}", idx, a, b));
            }
        }
    }

    // para_shape_id 비교
    if pa.para_shape_id != pb.para_shape_id {
        diffs.push(format!(
            "ps_id: A={} vs B={}",
            pa.para_shape_id, pb.para_shape_id
        ));
    }

    // tab_extended 비교
    if pa.tab_extended.len() != pb.tab_extended.len() {
        diffs.push(format!(
            "tab_ext count: A={} vs B={}",
            pa.tab_extended.len(),
            pb.tab_extended.len()
        ));
    } else {
        for (ti, (ta, tb)) in pa
            .tab_extended
            .iter()
            .zip(pb.tab_extended.iter())
            .enumerate()
        {
            if tab_ext_semantic_differs(ta, tb) {
                diffs.push(format!("tab_ext[{}]: A={:?} vs B={:?}", ti, ta, tb));
                break;
            }
        }
    }

    // LINE_SEG 비교
    if pa.line_segs.len() != pb.line_segs.len() {
        diffs.push(format!(
            "line_segs count: A={} vs B={}",
            pa.line_segs.len(),
            pb.line_segs.len()
        ));
    } else {
        for (li, (la, lb)) in pa.line_segs.iter().zip(pb.line_segs.iter()).enumerate() {
            if la.text_start != lb.text_start {
                diffs.push(format!(
                    "ls[{}].ts: A={} vs B={}",
                    li, la.text_start, lb.text_start
                ));
            }
            if la.vertical_pos != lb.vertical_pos {
                diffs.push(format!(
                    "ls[{}].vpos: A={} vs B={}",
                    li, la.vertical_pos, lb.vertical_pos
                ));
            }
            if la.line_height != lb.line_height {
                diffs.push(format!(
                    "ls[{}].lh: A={} vs B={}",
                    li, la.line_height, lb.line_height
                ));
            }
            if la.text_height != lb.text_height {
                diffs.push(format!(
                    "ls[{}].th: A={} vs B={}",
                    li, la.text_height, lb.text_height
                ));
            }
            if la.baseline_distance != lb.baseline_distance {
                diffs.push(format!(
                    "ls[{}].bl: A={} vs B={}",
                    li, la.baseline_distance, lb.baseline_distance
                ));
            }
            if la.line_spacing != lb.line_spacing {
                diffs.push(format!(
                    "ls[{}].ls: A={} vs B={}",
                    li, la.line_spacing, lb.line_spacing
                ));
            }
            if la.column_start != lb.column_start {
                diffs.push(format!(
                    "ls[{}].cs: A={} vs B={}",
                    li, la.column_start, lb.column_start
                ));
            }
            if la.segment_width != lb.segment_width {
                diffs.push(format!(
                    "ls[{}].sw: A={} vs B={}",
                    li, la.segment_width, lb.segment_width
                ));
            }
        }
    }

    // 컨트롤 식별 비교
    if pa.controls.len() != pb.controls.len() {
        diffs.push(format!(
            "controls count: A={} vs B={}",
            pa.controls.len(),
            pb.controls.len()
        ));
    }
    {
        use rhwp::model::control::Control;
        let ctrl_count = pa.controls.len().min(pb.controls.len());
        for ci in 0..ctrl_count {
            let ca = &pa.controls[ci];
            let cb = &pb.controls[ci];
            match (ca, cb) {
                (Control::Table(ta), Control::Table(tb)) => {
                    diff_table(&mut diffs, ci, ta, tb);
                }
                (Control::Picture(pic_a), Control::Picture(pic_b)) => {
                    diff_common_obj(&mut diffs, ci, "pic", &pic_a.common, &pic_b.common);
                }
                (Control::Shape(sa), Control::Shape(sb)) => {
                    diff_common_obj(&mut diffs, ci, "shape", sa.common(), sb.common());
                    // [#1807] 글상자 내부 문단 재귀 비교 — 직렬화 결함이
                    // 글상자 안에서 발생해도 검출되도록 (#1795 소거망 구멍)
                    diff_shape_textbox(&mut diffs, &format!("ctrl[{}] shape", ci), sa, sb);
                }
                _ if control_tag(ca) != control_tag(cb) => {
                    diffs.push(format!(
                        "ctrl[{}] type: A={} vs B={}",
                        ci,
                        control_tag(ca),
                        control_tag(cb)
                    ));
                }
                _ => {}
            }
        }
    }

    // char_shapes 비교
    if pa.char_shapes.len() != pb.char_shapes.len() {
        diffs.push(format!(
            "char_shapes count: A={} vs B={}",
            pa.char_shapes.len(),
            pb.char_shapes.len()
        ));
    } else {
        for (ci, (ca, cb)) in pa.char_shapes.iter().zip(pb.char_shapes.iter()).enumerate() {
            if ca.start_pos != cb.start_pos {
                diffs.push(format!(
                    "cs[{}].pos: A={} vs B={}",
                    ci, ca.start_pos, cb.start_pos
                ));
                break;
            }
            if ca.char_shape_id != cb.char_shape_id {
                diffs.push(format!(
                    "cs[{}].id: A={} vs B={}",
                    ci, ca.char_shape_id, cb.char_shape_id
                ));
                break;
            }
        }
    }
    diffs
}

fn ir_diff(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("사용법: rhwp ir-diff <파일A> <파일B> [-s <구역>] [-p <문단>] [--summary] [--max-lines <N>] [--json]");
        // [#3274] 인자 부족은 사용법 오류다 — 종전엔 0 으로 끝나 스크립트가 감지 못했다.
        return EXIT_USAGE;
    }

    let file_a = &args[0];
    let file_b = &args[1];
    let mut section_filter: Option<usize> = None;
    let mut para_filter: Option<usize> = None;
    // [Task #653 보강] 출력 가드 옵션
    let mut summary_mode = false;
    let mut max_lines: Option<usize> = None;
    // [#3274] --json: 계약 봉투 한 줄(카테고리 요약 포함), 차이 발견 시 exit 3.
    let mut json_mode = false;

    // [#3274] 값을 받는 옵션은 다음 토큰이 플래그(`-` 시작)면 값으로 삼키지 않는다.
    // 종전엔 `--max-lines --json` 처럼 값을 빠뜨리면 "--json" 이 값으로 소비돼
    // json 모드가 조용히 꺼지고, 게이트를 기대한 스크립트가 차이를 통과로 오판했다.
    // (-s/-p/--max-lines 는 모두 비음수만 받으므로 `-` 로 시작하는 값은 없다.)
    let is_value = |idx: usize| idx < args.len() && !args[idx].starts_with('-');
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--section" if is_value(i + 1) => {
                section_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "-p" | "--para" if is_value(i + 1) => {
                para_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "--summary" => {
                summary_mode = true;
                i += 1;
            }
            "--max-lines" if is_value(i + 1) => {
                max_lines = args[i + 1].parse().ok();
                i += 2;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // [#3274] 읽기·파싱 실패는 exit 1 (#2707 정렬) — 종전엔 0 으로 끝나
    // "비교했고 차이 없음"과 "비교 자체를 못 함"을 구별할 수 없었다.
    let data_a = match fs::read(file_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_a, e);
            return EXIT_RUNTIME;
        }
    };
    let data_b = match fs::read(file_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_b, e);
            return EXIT_RUNTIME;
        }
    };

    let doc_a = match rhwp::parser::parse_document(&data_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패: {:?}", file_a, e);
            return EXIT_RUNTIME;
        }
    };
    let doc_b = match rhwp::parser::parse_document(&data_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패: {:?}", file_b, e);
            return EXIT_RUNTIME;
        }
    };

    let name_a = Path::new(file_a)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let name_b = Path::new(file_b)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    if !summary_mode && !json_mode {
        println!("=== IR 비교: {} vs {} ===", name_a, name_b);
    }

    // [Task #653 보강] 출력 가드 상태 — IrDiffEmitter 로 통합 (#2122)
    // [#3274] json 모드는 summary 와 같은 수집 전용 경로(버킷만 쌓고 무출력)를 탄다 —
    // stdout 순수성을 위해 텍스트 라인을 한 줄도 내면 안 된다.
    let mut em = IrDiffEmitter {
        summary_mode: summary_mode || json_mode,
        max_lines,
        printed_lines: 0,
        truncated: false,
        summary_buckets: std::collections::BTreeMap::new(),
    };

    let mut total_diffs = 0u32;

    // 구역 수 비교
    // [#3274] 종전엔 total_diffs 선언이 이 블록 뒤에 있어 구역 수 차이가 집계되지
    // 않았다. 텍스트 모드에선 차이 라인이 화면에 보여 무해했으나, --json 게이트에서는
    // 구역 하나가 덧붙은 변환본이 diffCount=0·identical:true·exit 0 으로 통과하는
    // 치명적 누락이었다(봉투 자기모순). 선언을 앞으로 올리고 여기서도 집계한다.
    if doc_a.sections.len() != doc_b.sections.len() {
        em.diff(format!(
            "구역 수: A={} vs B={}",
            doc_a.sections.len(),
            doc_b.sections.len()
        ));
        total_diffs += 1;
    }

    let sec_count = doc_a.sections.len().min(doc_b.sections.len());

    for sec_idx in 0..sec_count {
        if let Some(sf) = section_filter {
            if sec_idx != sf {
                continue;
            }
        }

        let sec_a = &doc_a.sections[sec_idx];
        let sec_b = &doc_b.sections[sec_idx];

        if sec_a.paragraphs.len() != sec_b.paragraphs.len() {
            em.diff(format!(
                "구역 {}: 문단 수 A={} vs B={}",
                sec_idx,
                sec_a.paragraphs.len(),
                sec_b.paragraphs.len()
            ));
            total_diffs += 1;
        }

        let para_count = sec_a.paragraphs.len().min(sec_b.paragraphs.len());
        for pi in 0..para_count {
            if let Some(pf) = para_filter {
                if pi != pf {
                    continue;
                }
            }

            let pa = &sec_a.paragraphs[pi];
            let pb = &sec_b.paragraphs[pi];
            let diffs = ir_diff_paragraph_fields(pa, pb, &doc_a, &doc_b);

            if !diffs.is_empty() {
                let text_preview: String = pa.text.chars().take(30).collect();
                em.header(format!(
                    "\n--- 문단 {}.{} --- \"{}\"",
                    sec_idx, pi, text_preview
                ));
                for d in &diffs {
                    em.diff(format!("{}", d));
                }
                total_diffs += diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: ParaShape
    {
        let ps_a = &doc_a.doc_info.para_shapes;
        let ps_b = &doc_b.doc_info.para_shapes;
        if ps_a.len() != ps_b.len() {
            em.diff(format!(
                "ParaShape 수: A={} vs B={}",
                ps_a.len(),
                ps_b.len()
            ));
            total_diffs += 1;
        }
        let ps_count = ps_a.len().min(ps_b.len());
        for i in 0..ps_count {
            let a = &ps_a[i];
            let b = &ps_b[i];
            let mut ps_diffs: Vec<String> = Vec::new();
            if a.margin_left != b.margin_left {
                ps_diffs.push(format!("ml: {}vs{}", a.margin_left, b.margin_left));
            }
            if a.margin_right != b.margin_right {
                ps_diffs.push(format!("mr: {}vs{}", a.margin_right, b.margin_right));
            }
            if a.indent != b.indent {
                ps_diffs.push(format!("indent: {}vs{}", a.indent, b.indent));
            }
            if a.tab_def_id != b.tab_def_id {
                ps_diffs.push(format!("tab_def: {}vs{}", a.tab_def_id, b.tab_def_id));
            }
            if a.spacing_before != b.spacing_before {
                ps_diffs.push(format!("sb: {}vs{}", a.spacing_before, b.spacing_before));
            }
            if a.spacing_after != b.spacing_after {
                ps_diffs.push(format!("sa: {}vs{}", a.spacing_after, b.spacing_after));
            }
            if a.line_spacing != b.line_spacing {
                ps_diffs.push(format!("ls: {}vs{}", a.line_spacing, b.line_spacing));
            }
            if !ps_diffs.is_empty() {
                em.diff(format!("PS[{}] {}", i, ps_diffs.join(", ")));
                total_diffs += ps_diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: TabDef
    {
        let td_a = &doc_a.doc_info.tab_defs;
        let td_b = &doc_b.doc_info.tab_defs;
        if td_a.len() != td_b.len() {
            em.diff(format!("TabDef 수: A={} vs B={}", td_a.len(), td_b.len()));
            total_diffs += 1;
        }
        let td_count = td_a.len().min(td_b.len());
        for i in 0..td_count {
            let a = &td_a[i];
            let b = &td_b[i];
            if a.tabs.len() != b.tabs.len() {
                em.diff(format!(
                    "TD[{}] 탭 수: A={} vs B={}",
                    i,
                    a.tabs.len(),
                    b.tabs.len()
                ));
                total_diffs += 1;
            } else {
                for (ti, (ta, tb)) in a.tabs.iter().zip(b.tabs.iter()).enumerate() {
                    if ta.position != tb.position
                        || ta.tab_type != tb.tab_type
                        || ta.fill_type != tb.fill_type
                    {
                        em.diff(format!(
                            "TD[{}][{}] pos: {}vs{}, type: {}vs{}, fill: {}vs{}",
                            i,
                            ti,
                            ta.position,
                            tb.position,
                            ta.tab_type,
                            tb.tab_type,
                            ta.fill_type,
                            tb.fill_type
                        ));
                        total_diffs += 1;
                    }
                }
            }
        }
    }

    // [Task #653 보강] 요약 모드 출력 — 카테고리별 카운트 (내림차순 → 알파벳)
    // [#3274] --summary --json 병용 시 JSON 이 이긴다 — stdout 순수성 우선.
    if summary_mode && !json_mode {
        println!("=== 카테고리별 차이 요약 ===");
        let mut entries: Vec<(String, u32)> = em.summary_buckets.clone().into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (cat, count) in &entries {
            println!("  {:>5}건  {}", count, cat);
        }
    }

    if json_mode {
        // [#3274] 계약 봉투 한 줄 — 카테고리 버킷(BTreeMap)은 키 정렬이 결정적이다.
        let envelope = serde_json::json!({
            "schemaVersion": "1.0",
            "a": file_a,
            "b": file_b,
            "identical": total_diffs == 0,
            "diffCount": total_diffs,
            "categories": em.summary_buckets,
        });
        println!("{envelope}");
        // 차이 발견 = 3: #2707 의 "--verify IR 차이" 코드와 같은 의미의 게이트 신호.
        return if total_diffs == 0 { EXIT_OK } else { 3 };
    }

    println!("\n=== 비교 완료: 차이 {} 건 ===", total_diffs);
    EXIT_OK
}

/// `fields` — 누름틀/필드 조사 (읽기 전용).
///
/// rhwp 는 이미 필드에 값을 **쓸 수** 있는데(`set_field_value_by_name`) 조회 API 는
/// WASM/스튜디오 경로에만 있어, 브라우저 밖 에이전트는 "이 서식이 무엇을 요구하는지"
/// 알 방법이 없었다. 기존 `collect_all_fields()` 를 그대로 노출한다(라이브러리 무변경).
fn show_fields(args: &[String]) -> i32 {
    use rhwp::document_core::queries::field_query::NestedEntry;

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => file_path = Some(other),
        }
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp fields <파일.hwp|파일.hwpx> [--json]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let infos = doc.collect_all_fields();
    let fields: Vec<serde_json::Value> = infos
        .iter()
        .map(|fi| {
            // 중첩 경로: 표 셀·글상자 안의 필드가 어디에 있는지 — 후속 편집의 좌표다.
            let nested: Vec<serde_json::Value> = fi
                .location
                .nested_path
                .iter()
                .map(|e| match e {
                    NestedEntry::TableCell {
                        control_index,
                        cell_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "tableCell",
                        "control": control_index,
                        "cell": cell_index,
                        "paragraph": para_index,
                    }),
                    NestedEntry::TextBox {
                        control_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "textBox",
                        "control": control_index,
                        "paragraph": para_index,
                    }),
                })
                .collect();

            serde_json::json!({
                "fieldId": fi.field.field_id,
                "fieldType": format!("{:?}", fi.field.field_type),
                "name": fi.field.field_name().unwrap_or(""),
                "guide": fi.field.guide_text().unwrap_or(""),
                "memo": fi.field.memo_text().unwrap_or_default(),
                "command": fi.field.command,
                "value": fi.value,
                "editableInForm": fi.field.is_editable_in_form(),
                "location": {
                    "section": fi.location.section_index,
                    "paragraph": fi.location.para_index,
                    "nested": nested,
                },
            })
        })
        .collect();

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": "1.0",
            "source": file_path,
            "fieldCount": fields.len(),
            "fields": fields,
        });
        println!("{envelope}");
        return EXIT_OK;
    }

    println!("문서 로드: {} (필드 {}개)", file_path, fields.len());
    for f in &fields {
        let name = f["name"].as_str().unwrap_or("");
        let label = if name.is_empty() {
            "(이름 없음)"
        } else {
            name
        };
        println!(
            "  [{}] {} = {:?}{}",
            f["fieldType"].as_str().unwrap_or("?"),
            label,
            f["value"].as_str().unwrap_or(""),
            if f["editableInForm"] == true {
                ""
            } else {
                " (서식 편집 불가)"
            }
        );
    }
    EXIT_OK
}

fn extract_thumbnail(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp thumbnail <파일.hwp> [옵션]");
        eprintln!("  -o, --output <파일>   출력 파일 경로");
        eprintln!("  --base64              base64 문자열 출력");
        eprintln!("  --data-uri            data:image/... URI 출력");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path: Option<String> = None;
    let mut mode = "file"; // "file", "base64", "data-uri"

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                }
            }
            "--base64" => mode = "base64",
            "--data-uri" => mode = "data-uri",
            _ => {}
        }
        i += 1;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다: {} ({})", input_path, e);
            std::process::exit(1);
        }
    };

    let result = match rhwp::parser::extract_thumbnail_only(&data) {
        Some(r) => r,
        None => {
            eprintln!("오류: PrvImage 썸네일이 없습니다: {}", input_path);
            std::process::exit(1);
        }
    };

    let mime = match result.format.as_str() {
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };

    match mode {
        "base64" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("{}", b64);
        }
        "data-uri" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("data:{};base64,{}", mime, b64);
        }
        _ => {
            // 파일 출력
            let out = output_path.unwrap_or_else(|| {
                let stem = Path::new(input_path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let ext = &result.format;
                format!("output/{}_thumb.{}", stem, ext)
            });

            // 출력 디렉토리 생성
            if let Some(parent) = Path::new(&out).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).ok();
                }
            }

            match fs::write(&out, &result.data) {
                Ok(_) => {
                    println!(
                        "썸네일 추출 완료: {} ({}x{}, {} bytes, {})",
                        out,
                        result.width,
                        result.height,
                        result.data.len(),
                        result.format
                    );
                }
                Err(e) => {
                    eprintln!("오류: 파일 저장 실패: {} ({})", out, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{allows_implicit_sibling_resources, tab_ext_semantic_differs};
    use rhwp::parser::FileFormat;

    #[test]
    fn hml_does_not_implicitly_load_sibling_resources() {
        assert!(!allows_implicit_sibling_resources(FileFormat::Hml));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwp));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwpx));
    }

    #[test]
    fn tab_ext_reserved_fields_ignored() {
        // 같은 문서의 HWPX(파서가 [1],[3..6]=0) vs HWP5([1]=leader/fill 슬롯, [3..6]=원본 바이트).
        // 이 포맷 비대칭 슬롯들은 모두 무시 → 의미 차이 없음.
        let hwpx = [1640, 0, 256, 0, 0, 0, 9];
        let hwp5 = [1640, 5, 256, 32, 32, 32, 9];
        assert!(!tab_ext_semantic_differs(&hwpx, &hwp5));
    }

    #[test]
    fn tab_ext_semantic_fields_detected() {
        let base = [1640, 0, 256, 0, 0, 0, 9];
        assert!(!tab_ext_semantic_differs(&base, &base));
        // width([0]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1641, 0, 256, 0, 0, 0, 9]));
        // type([2] high byte) 차이 검출 — 256(0x0100)→512(0x0200)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 512, 0, 0, 0, 9]));
        // leader([2] low byte, 두 포맷 공통) 차이 검출 — 256(0x0100)→257(0x0101)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 257, 0, 0, 0, 9]));
        // HWP5 leader/fill 슬롯([1], HWPX는 항상 0)은 포맷 비대칭이라 무시 — 차이로 치지 않음
        assert!(!tab_ext_semantic_differs(
            &base,
            &[1640, 1, 256, 0, 0, 0, 9]
        ));
        // marker([6]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 256, 0, 0, 0, 0]));
    }
}
