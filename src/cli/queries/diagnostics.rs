//! 문서 상태를 변경하지 않는 CLI 진단 조회 어댑터.
//!
//! Stage 2에서는 기존 binary-local load 및 단위 변환 seam을 보존한다. service 계층 이행은
//! Stage 3의 책임이다.

use std::fs;

use rhwp::model::footnote::FootnoteShape;

use crate::{hu_to_mm_i, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn dump_note_shape(args: &[String]) -> i32 {
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

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
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

fn note_shape_json(shape: &FootnoteShape) -> serde_json::Value {
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

pub(crate) fn dump_endnote_lines(args: &[String]) -> i32 {
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

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
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
        Control::IndexMark(_) => "IndexMark",
        Control::PageNumCtrl(_) => "PageNumCtrl",
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

/// 레이아웃 트리의 항목별 **실제 extent** 를 덤프한다.
///
/// `dump-pages` 는 쪽 나눔이 **의도한** 항목 목록과 저장 좌표를 보여준다. 그런데 쪽 밖
/// 배치를 조사할 때 필요한 것은 레이아웃이 **실제로 차지한** 영역이다. 둘이 어긋나는
/// 것이 결함의 실체이기 때문이다 (#3637).
///
/// 종전에는 SVG 의 `<text>`·`<rect>` y 좌표로 이를 역산했는데, **테두리 없는 표는
/// `<rect>` 를 만들지 않아** 그 자리를 "빈 공간" 으로 오판했다. 이 명령은 렌더 트리를
/// 직접 걸어 그 한계를 없앤다.
///
/// 사용법:
/// ```text
/// rhwp dump-extents <파일> [-p <쪽번호>] [--min-h <px>] [--outside] [--gaps]
/// ```
///
/// - `--outside` : 쪽 경계를 넘는 노드만 출력
/// - `--gaps`    : 콘텐츠 사이 세로 빈 구간만 출력 (무엇이 자리를 먹는지)
/// - `--min-h`   : 이 높이 미만 노드 생략 (기본 0)
pub(crate) fn dump_extents(args: &[String]) -> i32 {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    if args.is_empty() {
        eprintln!(
            "사용법: rhwp dump-extents <파일.hwp> [-p <쪽번호>] [--min-h <px>] [--outside] [--gaps]"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut target_page: Option<u32> = None;
    let mut min_h = 0.0f64;
    let mut only_outside = false;
    let mut show_gaps = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" | "-p" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("오류: {} 뒤에 쪽 번호가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                };
                match v.parse::<u32>() {
                    Ok(n) => target_page = Some(n),
                    Err(_) => {
                        eprintln!("오류: 쪽 번호가 올바르지 않습니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--min-h" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("오류: --min-h 뒤에 값이 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<f64>() {
                    Ok(n) => min_h = n,
                    Err(_) => {
                        eprintln!("오류: --min-h 값이 올바르지 않습니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--outside" => {
                only_outside = true;
                i += 1;
            }
            "--gaps" => {
                show_gaps = true;
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
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let page_count = doc.page_count();
    println!("문서 로드: {} ({}쪽)", file_path, page_count);

    // 노드 종류를 짧은 이름과 (문단/컨트롤) 요약으로 바꾼다.
    fn describe(n: &RenderNode) -> (&'static str, String) {
        match &n.node_type {
            RenderNodeType::Page(_) => ("Page", String::new()),
            RenderNodeType::PageBackground(_) => ("PageBg", String::new()),
            RenderNodeType::MasterPage => ("MasterPage", String::new()),
            RenderNodeType::Header => ("Header", String::new()),
            RenderNodeType::Footer => ("Footer", String::new()),
            RenderNodeType::Body { .. } => ("Body", String::new()),
            RenderNodeType::Column(c) => ("Column", format!("col={c}")),
            RenderNodeType::FootnoteArea => ("FootnoteArea", String::new()),
            RenderNodeType::TextLine(t) => (
                "TextLine",
                format!(
                    "pi={} line={} vpos={}",
                    t.para_index.map(|v| v as i64).unwrap_or(-1),
                    t.line_index.map(|v| v as i64).unwrap_or(-1),
                    t.vpos.unwrap_or(-1)
                ),
            ),
            RenderNodeType::TextRun(t) => (
                "TextRun",
                format!(
                    "pi={} {:?}",
                    t.para_index.map(|v| v as i64).unwrap_or(-1),
                    t.text.chars().take(14).collect::<String>()
                ),
            ),
            RenderNodeType::Table(t) => (
                "Table",
                format!(
                    "pi={} ci={} {}x{}",
                    t.para_index.map(|v| v as i64).unwrap_or(-1),
                    t.control_index.map(|v| v as i64).unwrap_or(-1),
                    t.row_count,
                    t.col_count
                ),
            ),
            RenderNodeType::TableCell(c) => ("TableCell", format!("r={} c={}", c.row, c.col)),
            _ => ("기타", String::new()),
        }
    }

    // 깊이 우선으로 걸으며 visit 를 호출한다.
    fn walk(n: &RenderNode, depth: usize, visit: &mut impl FnMut(&RenderNode, usize)) {
        visit(n, depth);
        for c in &n.children {
            walk(c, depth + 1, visit);
        }
    }

    // -p 는 다른 dump 명령과 같이 0-based 쪽 인덱스다. 범위를 벗어나면 렌더 트리 생성
    // 실패 메시지 대신 사용법 오류로 끊는다.
    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count.saturating_sub(1)
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    for p in pages {
        let tree = match doc.build_page_render_tree(p) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: {}쪽 렌더 트리 생성 실패 - {:?}", p + 1, e);
                return EXIT_RUNTIME;
            }
        };
        let page_h = tree.root.bbox.height;
        let page_w = tree.root.bbox.width;
        println!("\n=== {}쪽 (트리 {:.1}x{:.1}px) ===", p + 1, page_w, page_h);

        let mut outside: Vec<(f64, f64, &'static str, String)> = Vec::new();
        // [#4889] 쪽 **위쪽** 밖(음수 y) 노드. 아래쪽 넘침만 세던 탓에 이 방향은
        // 어떤 지표로도 안 잡혔다 — 쪽수는 정답지와 같고(3/3), 글자는 트리에 남아
        // 있어 텍스트 추출도 통과한다. 판별자는 `y < 0` 이 **아니라** `bottom <= 0`
        // 이다: 상단만 음수인 노드는 위가 잘릴 뿐 일부가 보이며(반복 머리말이 흔한
        // 예), 이를 소실로 세면 10k 기준 3배 과대집계된다.
        let mut above: Vec<(f64, f64, &'static str, String)> = Vec::new();
        let mut spans: Vec<(f64, f64, &'static str, String)> = Vec::new();

        walk(&tree.root, 0, &mut |n, depth| {
            let b = &n.bbox;
            if b.height < min_h {
                return;
            }
            let (kind, idx) = describe(n);
            let bottom = b.y + b.height;
            let is_outside = bottom > page_h + 0.5;
            if is_outside {
                outside.push((b.y, bottom, kind, idx.clone()));
            }
            if bottom < -0.5 {
                above.push((b.y, bottom, kind, idx.clone()));
            }
            // 빈 구간 계산에는 **잎 콘텐츠**만 쓴다.
            //
            // 컨테이너는 자기 안의 공백을 통째로 가린다. Body·Column 뿐 아니라 **표도**
            // 그렇다 — 본문 전체를 담은 1×1 표는 쪽 전체를 덮어 내부 201px 공백을
            // "구간 없음" 으로 만들었다(#3637 조사에서 실제로 겪은 오판이다).
            //
            // 그래서 TextLine 과, **자손에 TextLine 이 없는** 표(= 빈 표)만 센다.
            let has_text_descendant = {
                fn any_text(n: &RenderNode) -> bool {
                    if matches!(n.node_type, RenderNodeType::TextLine(_)) {
                        return true;
                    }
                    n.children.iter().any(any_text)
                }
                n.children.iter().any(any_text)
            };
            if matches!(n.node_type, RenderNodeType::TextLine(_))
                || (matches!(n.node_type, RenderNodeType::Table(_)) && !has_text_descendant)
            {
                spans.push((b.y, bottom, kind, idx.clone()));
            }
            if show_gaps || (only_outside && !is_outside) {
                return;
            }
            println!(
                "{:indent$}{kind:12} y={:8.1}..{:8.1} h={:7.1} x={:7.1} w={:7.1}  {idx}",
                "",
                b.y,
                bottom,
                b.height,
                b.x,
                b.width,
                indent = depth * 2,
            );
        });

        if show_gaps {
            spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            println!("  -- 콘텐츠 사이 세로 빈 구간 (30px 이상) --");
            let mut cursor = 0.0f64;
            let mut cursor_src = String::from("(쪽 시작)");
            for (y, bottom, kind, idx) in &spans {
                if *y - cursor > 30.0 {
                    println!(
                        "     빈 구간 {:8.1}..{:8.1} ({:6.1}px)  직전={cursor_src} → 다음={kind} {idx}",
                        cursor,
                        y,
                        y - cursor,
                    );
                }
                if *bottom > cursor {
                    cursor = *bottom;
                    cursor_src = format!("{kind} {idx}");
                }
            }
        }

        if outside.is_empty() {
            println!("  쪽 경계를 넘는 노드 없음");
        } else {
            let worst = outside
                .iter()
                .map(|(_, b, _, _)| *b - page_h)
                .fold(0.0f64, f64::max);
            println!(
                "  ** 쪽 경계를 넘는 노드 {}개 · 최대 초과 {:.1}px **",
                outside.len(),
                worst
            );
            for (y, bottom, kind, idx) in outside.iter().take(8) {
                println!(
                    "     {kind:12} y={y:8.1}..{bottom:8.1} 초과 {:7.1}px  {idx}",
                    bottom - page_h
                );
            }
        }

        // 위쪽 밖은 있을 때만 보고한다 — 없을 때도 한 줄 찍으면 기존 스냅샷이 전부
        // 흔들린다.
        if !above.is_empty() {
            let worst = above.iter().map(|(_, b, _, _)| -*b).fold(0.0f64, f64::max);
            println!(
                "  ** 쪽 위쪽 밖 노드 {}개 · 최대 {:.1}px 위 **",
                above.len(),
                worst
            );
            for (y, bottom, kind, idx) in above.iter().take(8) {
                println!(
                    "     {kind:12} y={y:8.1}..{bottom:8.1} 위 {:7.1}px  {idx}",
                    -*bottom
                );
            }
        }
    }
    EXIT_OK
}

pub(crate) fn diag_document(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp diag <파일.hwp>");
        return EXIT_USAGE;
    }

    // [#3884 G2] diag 는 추가 옵션이 없다 — 지금까지는 어떤 플래그를 붙여도(--json 포함)
    // 조용히 무시하고 exit 0 이라, 옵션이 먹혔다는 착각을 만들었다.
    if let Some(bad) = args.iter().find(|a| a.starts_with('-')) {
        eprintln!("오류: 알 수 없는 옵션입니다 - {bad}");
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

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
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
