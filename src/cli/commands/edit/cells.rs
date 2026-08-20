use std::{fs, path::Path, process};

use rhwp::{provenance, schema_registry::ENVELOPE_SCHEMA_VERSION};

use super::runtime::{
    edit_output_format, edit_serialize, edit_verify_report, finish_edit_write, EditOutputFormat,
};
use super::tables::{resolve_table_cell, CellResolveError};
use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn recolor_cell_text_black(
    document: &mut rhwp::model::document::Document,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
) -> bool {
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::CharShapeRef;

    // 대상 셀의 현재 글자모양을 기준으로 해야 한다. 문서 어딘가의 "검정" 모양을 재사용하면
    // 글꼴·크기까지 바뀔 수 있다.
    let source_id = {
        let Some(section) = document.sections.get(sec) else {
            return false;
        };
        let Some(parent) = section.paragraphs.get(para) else {
            return false;
        };
        let Some(Control::Table(table)) = parent.controls.get(ctrl) else {
            return false;
        };
        let Some(cell) = table.cells.get(cell_idx) else {
            return false;
        };
        let Some(paragraph) = cell.paragraphs.first() else {
            return false;
        };
        let Some(shape) = paragraph.char_shapes.first() else {
            return false;
        };
        shape.char_shape_id as usize
    };
    let Some(base) = document
        .doc_info
        .char_shapes
        .get(source_id)
        .or_else(|| document.doc_info.char_shapes.first())
        .cloned()
    else {
        return false;
    };
    let mut black = base;
    black.raw_data = None; // 원본 바이트를 버려 변경된 필드가 직렬화되게 한다.
    black.text_color = 0;
    black.italic = false;
    black.bold = false;
    black.strikethrough = false;
    black.underline_type = rhwp::model::style::UnderlineType::None;
    let black_id = document
        .doc_info
        .char_shapes
        .iter()
        .position(|candidate| candidate == &black)
        .map(|idx| idx as u32)
        .unwrap_or_else(|| {
            let new_id = document.doc_info.char_shapes.len() as u32;
            document.doc_info.char_shapes.push(black);
            new_id
        });

    let Some(section) = document.sections.get_mut(sec) else {
        return false;
    };
    let Some(parent) = section.paragraphs.get_mut(para) else {
        return false;
    };
    let Some(Control::Table(table)) = parent.controls.get_mut(ctrl) else {
        return false;
    };
    let Some(cell) = table.cells.get_mut(cell_idx) else {
        return false;
    };
    let Some(cell_para) = cell.paragraphs.get_mut(0) else {
        return false;
    };
    // 문단 전체를 하나의 검정 글자모양으로 덮는다.
    cell_para.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: black_id,
    }];
    true
}

/// [#3480] 셀에 넣을 텍스트가 칸 폭을 넘치는지 잰다.
///
/// 넘치면 `(칸 폭 px, 글자 폭 px, 예상 줄 수)` 를 돌려주고, 들어가면 `None`.
/// 폭은 조판 엔진의 글자 폭 추정(`estimate_text_width_px`)과 IR 의 `Cell.width` 를 쓴다.
/// **채우기를 막지는 않는다** — 여러 줄이 정상인 칸도 있으므로 신호만 준다.
pub(crate) fn measure_cell_overflow(
    doc: &rhwp::wasm_api::HwpDocument,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
    text: &str,
) -> Option<(f64, f64, usize)> {
    use rhwp::model::control::Control;
    use rhwp::renderer::hwpunit_to_px;

    if text.is_empty() {
        return None;
    }
    let cell = doc
        .document()
        .sections
        .get(sec)?
        .paragraphs
        .get(para)?
        .controls
        .get(ctrl)
        .and_then(|c| match c {
            Control::Table(t) => t.cells.get(cell_idx),
            _ => None,
        })?;

    // 셀 안여백을 뺀 실제 글자 영역 폭.
    let padding = (cell.padding.left + cell.padding.right) as f64;
    let usable = hwpunit_to_px(
        (cell.width as f64 - padding) as i32,
        rhwp::renderer::DEFAULT_DPI,
    );
    if usable <= 0.0 {
        return None;
    }

    let text_w = estimate_text_width_px(doc, sec, para, ctrl, cell_idx, text);
    if text_w <= usable {
        return None;
    }
    let lines = (text_w / usable).ceil() as usize;
    Some((usable, text_w, lines))
}

/// 셀의 첫 문단 글자 모양을 기준으로 텍스트 폭(px)을 추정한다.
///
/// 정밀 조판이 아니라 **넘침 여부 판정용 근사**다 — 한글은 전각, ASCII 는 반각으로 센다.
fn estimate_text_width_px(
    doc: &rhwp::wasm_api::HwpDocument,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
    text: &str,
) -> f64 {
    use rhwp::model::control::Control;
    use rhwp::renderer::hwpunit_to_px;

    // 셀 첫 문단의 글자 크기(HWPUNIT, 1pt = 100). 못 찾으면 10pt 로 본다.
    let size_hwpunit = doc
        .document()
        .sections
        .get(sec)
        .and_then(|s| s.paragraphs.get(para))
        .and_then(|p| p.controls.get(ctrl))
        .and_then(|c| match c {
            Control::Table(t) => t.cells.get(cell_idx),
            _ => None,
        })
        .and_then(|cell| cell.paragraphs.first())
        .and_then(|p| p.char_shapes.first())
        .and_then(|cs| {
            doc.document()
                .doc_info
                .char_shapes
                .get(cs.char_shape_id as usize)
        })
        .map(|cs| cs.base_size as f64)
        .unwrap_or(1000.0);

    let em = hwpunit_to_px(size_hwpunit as i32, rhwp::renderer::DEFAULT_DPI);
    text.chars()
        .map(|c| if c.is_ascii() { em * 0.5 } else { em })
        .sum()
}

/// [#3603] `set-cell` 계열이 셀 값으로 거부하는 제어문자 안내문.
///
/// CLI(`edit set-cell`)와 세션 도구(`hwp_doc_set_cell`)가 **같은 문장**으로 거부해야 한다 —
/// 두 경로가 서로 다른 문장(또는 한쪽만 검사)을 내면 에이전트는 같은 제약을 두 번 배워야
/// 하고, 무엇보다 세션 경로만 통과시키면 한 셀 문단 안에 raw 개행이 박힌 문서가 만들어진다.
/// v1 셀 기록 계약은 '한 줄 값'이다.
const SET_CELL_CONTROL_CHAR_MESSAGE: &str =
    "오류: --text 에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록).";

/// 셀 값에 제어문자가 있으면 공통 안내문을 돌려준다 (없으면 `None`).
///
/// 문장뿐 아니라 **판정식까지** 공유해야 '문장은 같은데 거부 조건이 다른' 어긋남이 안 생긴다.
pub(crate) fn set_cell_control_char_rejection(text: &str) -> Option<&'static str> {
    text.chars()
        .any(|ch| matches!(ch, '\r' | '\n' | '\t'))
        .then_some(SET_CELL_CONTROL_CHAR_MESSAGE)
}
pub(super) fn edit_set_cell(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut text_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    // [#3702] 저장 직후 자기검증 — 판정은 데이터, 차이 시 exit 3.
    let mut verify_mode = false;
    // [#3391] 실물 공고 양식의 기입 칸 안내문은 파란 이탤릭이 흔하다. set-cell 은
    // "안내문을 지우고 실값을 쓰는" 용도이므로 제출 요건(검정 글씨)에 맞춰 기본을
    // 검정·비이탤릭·비진하게로 기록한다. --keep-style 로 셀 스타일 상속을 유지한다.
    let mut keep_style = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--keep-style" => keep_style = true,
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {} 뒤에 0 이상의 정수가 필요합니다.", name);
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(value) => table_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(value) => row_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상 65535 이하의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(value) => col_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상 65535 이하의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!(
                            "오류: --text 뒤에 셀에 넣을 문자열이 필요합니다 (비우기는 \"\")."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
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

    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(new_text)) =
        (file_path, table_arg, row_arg, col_arg, text_arg)
    else {
        eprintln!(
            "사용법: rhwp edit set-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [-o <출력>] [--keep-style] [--dry-run] [--json]"
        );
        return EXIT_USAGE;
    };
    // 판정과 문장 모두 세션 도구(hwp_doc_set_cell)와 공유한다 — 문서를 읽기 전에 끊는다.
    if let Some(message) = set_cell_control_char_rejection(new_text) {
        eprintln!("{message}");
        return EXIT_USAGE;
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    // 격자 주소(export-tables 좌표) → 모델 좌표. 병합으로 덮인 칸은 앵커가 아니므로
    // 모델 셀 순회로 (row,col) 앵커를 직접 찾는다 (격자 배열 위치는 손상 방어 필터
    // 때문에 모델 인덱스와 어긋날 수 있어 쓰지 않는다).
    let (sec, para, ctrl, cell_idx, para_lens, old_text) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };

    // [#3480] 값이 그 칸에 들어가는지 재고 넘치면 알린다.
    // 에이전트는 렌더 결과를 보지 않으므로, 신호가 없으면 표 경계를 벗어난 문서를
    // 완성본으로 판단한다. 조판 엔진이 있어야 답할 수 있는 검사다.
    let overflow = measure_cell_overflow(&doc, sec, para, ctrl, cell_idx, &new_text).map(
        |(cell_w, text_w, lines)| {
            serde_json::json!({
                "target": format!("table{}[{},{}]", table_no, row, col),
                "text": new_text,
                "cellWidthPx": (cell_w * 100.0).round() / 100.0,
                "textWidthPx": (text_w * 100.0).round() / 100.0,
                "lines": lines,
            })
        },
    );

    if !dry_run {
        // 셀의 모든 문단 텍스트를 비운다 (다문단 셀 — 빈 문단 골격은 유지된다).
        for (pi, len) in para_lens.iter().enumerate() {
            if *len == 0 {
                continue;
            }
            if let Err(e) = doc.delete_text_in_cell(
                sec as u32,
                para as u32,
                ctrl as u32,
                cell_idx as u32,
                pi as u32,
                0,
                *len as u32,
            ) {
                eprintln!("오류: 셀 비우기 실패(문단 {}) - {:?}", pi, e);
                return EXIT_RUNTIME;
            }
        }
        if !new_text.is_empty() {
            if let Err(e) = doc.insert_text_in_cell(
                sec as u32,
                para as u32,
                ctrl as u32,
                cell_idx as u32,
                0,
                0,
                new_text,
            ) {
                eprintln!("오류: 셀 쓰기 실패 - {:?}", e);
                // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                return EXIT_RUNTIME;
            }
            // [#3391] 기본은 제출 요건(검정 글씨)에 맞춘다 — 셀 문단 0 의 글자모양을
            // 검정·비이탤릭·비진하게 글자모양 하나로 덮는다. --keep-style 이면 생략.
            if !keep_style
                && !recolor_cell_text_black(doc.document_mut(), sec, para, ctrl, cell_idx)
            {
                eprintln!("경고: 셀 글자색을 검정으로 바꾸지 못했습니다 (상속 스타일 유지).");
            }
        }
    }

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_cell.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        // [#3702] 저장 직후 자기검증 — 편집 후 IR ↔ 저장본 재파싱 IR.
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    // [#3712] 눈검증 대상 페이지 — 표 호스트 문단이 걸친 쪽 전부(분할 표 포함).
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "table": table_no,
            "row": row,
            "col": col,
            "oldText": old_text,
            "newText": new_text,
            "dryRun": dry_run,
            "changedPages": changed_pages,
            "keepStyle": keep_style,
            "overflow": overflow.clone().map(|o| vec![o]).unwrap_or_default(),
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "변경 예정: {} 표{} ({},{}) {:?} → {:?}",
            file_path, table_no, row, col, old_text, new_text
        );
    } else {
        println!(
            "셀 기록 완료: {} → {} — 표{} ({},{}) {:?} → {:?}",
            file_path, output_path, table_no, row, col, old_text, new_text
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}
/// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
/// `edit insert-text-in-cell` — 표 셀 문단에 텍스트 삽입. 코어 `insert_text_in_cell_native`.
pub(super) fn edit_insert_text_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-text-in-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [--offset N] [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut text_arg: Option<&str> = None;
    let mut offset_arg: usize = 0;
    let mut cell_para_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--offset" => offset_arg = n,
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => text_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다 (빈 문자열 불가).");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
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
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(text)) =
        (file_path, table_arg, row_arg, col_arg, text_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_text_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            text,
        ) {
            eprintln!("오류: 셀 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellins",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "cellPara": cell_para_arg,
            "offset": offset_arg,
            "text": text
        }),
        &[(sec, para)],
        &format!(
            "셀 텍스트 삽입 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg}"
        ),
        &format!("셀 텍스트 삽입 완료: {file_path}"),
    )
}

/// `edit delete-text-in-cell` — 표 셀 문단 텍스트 삭제. 코어 `delete_text_in_cell_native`.
pub(super) fn edit_delete_text_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-text-in-cell <파일> --table <번호> --row <행> --col <열> --count <글자수> [--offset N] [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut count_arg: Option<usize> = None;
    let mut offset_arg: usize = 0;
    let mut cell_para_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" | "--count" => {
                        match v.parse::<usize>() {
                            Ok(n) => match name.as_str() {
                                "--table" => table_arg = Some(n),
                                "--offset" => offset_arg = n,
                                "--count" => {
                                    if n == 0 {
                                        eprintln!("오류: --count 는 1 이상이어야 합니다.");
                                        return EXIT_USAGE;
                                    }
                                    count_arg = Some(n);
                                }
                                _ => cell_para_arg = n,
                            },
                            Err(_) => {
                                eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                                return EXIT_USAGE;
                            }
                        }
                    }
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
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
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(count)) =
        (file_path, table_arg, row_arg, col_arg, count_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.delete_text_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            count,
        ) {
            eprintln!("오류: 셀 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "celldel",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "cellPara": cell_para_arg,
            "offset": offset_arg,
            "count": count
        }),
        &[(sec, para)],
        &format!(
            "셀 텍스트 삭제 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg} 글자 {count}"
        ),
        &format!("셀 텍스트 삭제 완료: {file_path}"),
    )
}

/// [#4990 / #3608 M9] `edit insert-text` — 문단 좌표에 새 텍스트를 삽입한다.
///
/// 에이전트는 기존 문자열을 바꿀 수 있었지만(replace-text·fill-fields·set-cell)
/// **없는 자리에 글자를 넣는** 표면이 없었다. 새 편집 로직은 없다 —
/// 검증된 코어 `insert_text_native`(스튜디오·세션이 이미 쓰는 경로)만 배선한다.
/// 주소 어휘는 `search` 와 같다(구역·문단·문자 오프셋, 전부 0 기준).
/// `edit set-cell-props` — 표 셀 속성. 코어 `set_cell_properties_native`.
pub(super) fn edit_set_cell_props(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-cell-props <파일> --table <번호> --row <행> --col <열> --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
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
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(props)) =
        (file_path, table_arg, row_arg, col_arg, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if !matches!(
        serde_json::from_str::<serde_json::Value>(&props),
        Ok(serde_json::Value::Object(_))
    ) {
        eprintln!("오류: --props 는 JSON 객체여야 합니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, _para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if !dry_run {
        if let Err(e) = doc.set_cell_properties_native(sec, para, ctrl, cell_idx, &props) {
            eprintln!("오류: 셀 속성 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellprop",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row, "col": col }),
        &[(sec, para)],
        &format!("셀 속성 변경 예정: {file_path} 표 {table_no} ({row},{col})"),
        &format!("셀 속성 변경 완료: {file_path}"),
    )
}
