//! 기존 조회 도구를 하나의 실행 파일과 하위 명령으로 연결한다.

#[path = "commands/rhwp-q-char-shape.rs"]
mod char_shape;
#[path = "commands/rhwp-q-control-layout.rs"]
mod control_layout;
#[path = "commands/rhwp-q-cursor-model.rs"]
mod cursor_model;
#[path = "commands/rhwp-q-cursor-rect.rs"]
mod cursor_rect;
#[path = "commands/rhwp-q-font-layout-evidence.rs"]
mod font_layout_evidence;
#[path = "commands/rhwp-q-font-trace.rs"]
mod font_trace;
#[path = "commands/rhwp-q-form-info.rs"]
mod form_info;
#[path = "commands/rhwp-q-hit-test.rs"]
mod hit_test;
#[path = "kit/main.rs"]
mod kit;
#[path = "commands/rhwp-q-markdown.rs"]
mod markdown;
#[path = "more/main.rs"]
mod more;
#[path = "commands/rhwp-q-object-cycle.rs"]
mod object_cycle;
#[path = "commands/rhwp-q-objects.rs"]
mod objects;
#[path = "pack/main.rs"]
mod pack;
#[path = "commands/rhwp-q-page-caret.rs"]
mod page_caret;
#[path = "commands/rhwp-q-page-def.rs"]
mod page_def;
#[path = "commands/rhwp-q-page-images.rs"]
mod page_images;
#[path = "commands/rhwp-q-page-items.rs"]
mod page_items;
#[path = "commands/rhwp-q-para-shape.rs"]
mod para_shape;
#[path = "commands/rhwp-q-scan-items.rs"]
mod scan_items;
#[path = "commands/rhwp-q-section-starts.rs"]
mod section_starts;
#[path = "commands/rhwp-q-text-file.rs"]
mod text_file;
#[path = "commands/rhwp-q-text-layout.rs"]
mod text_layout;

mod help;
mod help_catalog;

type Handler = fn(Vec<String>);

const COMMANDS: &[(&str, Handler)] = &[
    ("char-shape", char_shape::run_cli),
    ("control-layout", control_layout::run_cli),
    ("cursor-model", cursor_model::run_cli),
    ("cursor-rect", cursor_rect::run_cli),
    ("font-layout-evidence", font_layout_evidence::run_cli),
    ("font-trace", font_trace::run_cli),
    ("form-info", form_info::run_cli),
    ("hit-test", hit_test::run_cli),
    ("markdown", markdown::run_cli),
    ("object-cycle", object_cycle::run_cli),
    ("objects", objects::run_cli),
    ("page-caret", page_caret::run_cli),
    ("page-def", page_def::run_cli),
    ("page-images", page_images::run_cli),
    ("page-items", page_items::run_cli),
    ("para-shape", para_shape::run_cli),
    ("scan-items", scan_items::run_cli),
    ("section-starts", section_starts::run_cli),
    ("text-file", text_file::run_cli),
    ("text-layout", text_layout::run_cli),
    ("kit", kit::run_cli),
    ("more", more::run_cli),
    ("pack", pack::run_cli),
];

fn help(output: &mut impl std::io::Write) -> std::io::Result<()> {
    writeln!(output, "rhwp-q v{} — 문서 조회·진단", rhwp::version())?;
    writeln!(output, "사용법: rhwp-q <명령> [인자]\n")?;
    for (name, _) in COMMANDS {
        writeln!(output, "  {name}")?;
    }
    writeln!(output, "\n명령별 사용법: rhwp-q <명령> --help")
}

fn output_code(result: std::io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        // 파이프 소비자가 먼저 종료한 경우에는 도움말 출력도 정상 종료한다.
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => 0,
        Err(error) => {
            eprintln!("오류: stdout 쓰기 실패 - {error}");
            1
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(text) = help::render(&args) {
        use std::io::Write;
        std::process::exit(output_code(write!(std::io::stdout().lock(), "{text}")));
    }
    let mut args = args.into_iter();
    let code = match args.next().as_deref() {
        Some("--help" | "-h") => output_code(help(&mut std::io::stdout().lock())),
        Some("--version" | "-V") => {
            use std::io::Write;
            output_code(writeln!(
                std::io::stdout().lock(),
                "rhwp-q v{}",
                rhwp::version()
            ))
        }
        Some(name) => match COMMANDS.iter().find(|(command, _)| *command == name) {
            Some((_, handler)) => {
                handler(args.collect());
                0
            }
            None => {
                eprintln!("오류: 알 수 없는 조회 명령입니다 - {name}");
                2
            }
        },
        None => {
            eprintln!("사용법: rhwp-q <명령> [인자]  (rhwp-q --help 참고)");
            2
        }
    };
    std::process::exit(code);
}
