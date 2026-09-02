//! rhwp-q-kit 공통 봉투·적재. 편집 API 는 여기서 부르지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use serde_json::{json, Map, Value};
use std::io::Write;

pub const EXIT_OK: i32 = 0;
pub const EXIT_RUNTIME: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const TOOL: &str = "rhwp-q-kit";

pub fn envelope(command: &str, mut payload: Value, untrusted: &[&str]) -> Value {
    if let Some(map) = payload.as_object_mut() {
        map.insert(
            "schemaVersion".into(),
            json!(rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION),
        );
        map.insert("tool".into(), json!(TOOL));
        map.insert("command".into(), json!(command));
        map.insert("version".into(), json!(rhwp::version()));
        map.insert("untrustedContent".into(), json!(!untrusted.is_empty()));
        map.insert("untrustedFields".into(), json!(untrusted));
    }
    payload
}

pub fn write_stdout(text: &str) -> i32 {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(e) = writeln!(lock, "{text}") {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        return EXIT_RUNTIME;
    }
    EXIT_OK
}

pub fn print_json(value: &Value) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(s) => write_stdout(&s),
        Err(e) => {
            eprintln!("오류: JSON 직렬화 실패 - {e}");
            EXIT_RUNTIME
        }
    }
}

pub fn read_file(path: &str) -> Result<Vec<u8>, i32> {
    std::fs::read(path).map_err(|e| {
        eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })
}

pub fn load_core(path: &str) -> Result<DocumentCore, i32> {
    let data = read_file(path)?;
    DocumentCore::from_bytes(&data).map_err(|e| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })
}

pub fn parse_u32(flag: &str, raw: &str) -> Result<u32, i32> {
    raw.parse::<u32>().map_err(|_| {
        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다 - {raw}");
        EXIT_USAGE
    })
}

pub fn parse_usize(flag: &str, raw: &str) -> Result<usize, i32> {
    raw.parse::<usize>().map_err(|_| {
        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다 - {raw}");
        EXIT_USAGE
    })
}

pub fn parse_f64(flag: &str, raw: &str) -> Result<f64, i32> {
    raw.parse::<f64>().map_err(|_| {
        eprintln!("오류: {flag} 뒤에 실수가 필요합니다 - {raw}");
        EXIT_USAGE
    })
}

pub fn parse_json_string(raw: &str) -> Result<Value, i32> {
    serde_json::from_str(raw).map_err(|e| {
        eprintln!("오류: 코어 JSON 파싱 실패 - {e}");
        EXIT_RUNTIME
    })
}

pub struct FileOpts {
    pub path: String,
    pub json: bool,
}

pub fn parse_one_file(args: &[String], usage: &str) -> Result<FileOpts, i32> {
    let mut path = None;
    let mut json = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return Err(EXIT_USAGE);
    };
    Ok(FileOpts { path, json })
}

pub fn emit_items(
    command: &str,
    source: &str,
    json_mode: bool,
    items: Vec<Value>,
    untrusted: &[&str],
) -> i32 {
    let count = items.len();
    let payload = json!({
        "source": source,
        "count": count,
        "items": items,
    });
    if json_mode {
        print_json(&envelope(command, payload, untrusted))
    } else {
        write_stdout(&format!("{command}={count}"))
    }
}

/// 컨트롤 위치. 중첩 컨테이너면 `paragraph`/`control` 은 그 컨테이너 문단 기준.
pub struct CtrlAt<'a> {
    pub section: usize,
    pub paragraph: usize,
    pub control: usize,
    pub container: &'static str,
    pub cell: Option<usize>,
    pub ctrl: &'a Control,
}

pub fn loc_json(at: &CtrlAt<'_>) -> Value {
    let mut map = Map::new();
    map.insert("section".into(), json!(at.section));
    map.insert("paragraph".into(), json!(at.paragraph));
    map.insert("control".into(), json!(at.control));
    map.insert("container".into(), json!(at.container));
    if let Some(cell) = at.cell {
        map.insert("cell".into(), json!(cell));
    }
    Value::Object(map)
}

const MAX_NEST_DEPTH: usize = 8;

pub fn for_each_control(doc: &Document, mut visit: impl FnMut(CtrlAt<'_>)) {
    for (section, sec) in doc.sections.iter().enumerate() {
        walk_paragraphs(&sec.paragraphs, section, "body", None, 0, &mut visit);
    }
}

fn walk_paragraphs<'a>(
    paragraphs: &'a [Paragraph],
    section: usize,
    container: &'static str,
    cell: Option<usize>,
    depth: usize,
    visit: &mut impl FnMut(CtrlAt<'a>),
) {
    if depth >= MAX_NEST_DEPTH {
        return;
    }
    for (paragraph, para) in paragraphs.iter().enumerate() {
        for (control, ctrl) in para.controls.iter().enumerate() {
            visit(CtrlAt {
                section,
                paragraph,
                control,
                container,
                cell,
                ctrl,
            });
            walk_nested(ctrl, section, depth, visit);
        }
    }
}

fn walk_nested<'a>(
    ctrl: &'a Control,
    section: usize,
    depth: usize,
    visit: &mut impl FnMut(CtrlAt<'a>),
) {
    if depth + 1 >= MAX_NEST_DEPTH {
        return;
    }
    match ctrl {
        Control::Table(table) => {
            for (cell, c) in table.cells.iter().enumerate() {
                walk_paragraphs(
                    &c.paragraphs,
                    section,
                    "tableCell",
                    Some(cell),
                    depth + 1,
                    visit,
                );
            }
            if let Some(caption) = &table.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
        }
        Control::Header(h) => {
            walk_paragraphs(&h.paragraphs, section, "header", None, depth + 1, visit);
        }
        Control::Footer(f) => {
            walk_paragraphs(&f.paragraphs, section, "footer", None, depth + 1, visit);
        }
        Control::Footnote(f) => {
            walk_paragraphs(&f.paragraphs, section, "footnote", None, depth + 1, visit);
        }
        Control::Endnote(e) => {
            walk_paragraphs(&e.paragraphs, section, "endnote", None, depth + 1, visit);
        }
        Control::HiddenComment(hc) => {
            walk_paragraphs(
                &hc.paragraphs,
                section,
                "hiddenComment",
                None,
                depth + 1,
                visit,
            );
        }
        Control::Picture(pic) => {
            if let Some(caption) = &pic.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
        }
        Control::Shape(shape) => walk_shape(shape, section, depth, visit),
        _ => {}
    }
}

fn walk_shape<'a>(
    shape: &'a ShapeObject,
    section: usize,
    depth: usize,
    visit: &mut impl FnMut(CtrlAt<'a>),
) {
    if depth + 1 >= MAX_NEST_DEPTH {
        return;
    }
    if let Some(drawing) = shape.drawing() {
        if let Some(tb) = drawing.text_box.as_ref() {
            walk_paragraphs(&tb.paragraphs, section, "textbox", None, depth + 1, visit);
        }
        if let Some(caption) = drawing.caption.as_ref() {
            walk_paragraphs(
                &caption.paragraphs,
                section,
                "caption",
                None,
                depth + 1,
                visit,
            );
        }
    }
    match shape {
        ShapeObject::Group(g) => {
            if let Some(caption) = &g.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
            for child in &g.children {
                walk_shape(child, section, depth + 1, visit);
            }
        }
        ShapeObject::Picture(p) => {
            if let Some(caption) = &p.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
        }
        ShapeObject::Chart(c) => {
            if let Some(caption) = &c.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
        }
        ShapeObject::Ole(o) => {
            if let Some(caption) = &o.caption {
                walk_paragraphs(
                    &caption.paragraphs,
                    section,
                    "caption",
                    None,
                    depth + 1,
                    visit,
                );
            }
        }
        _ => {}
    }
}
