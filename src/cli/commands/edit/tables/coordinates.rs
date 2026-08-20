/// 본문 최상위 표 번호 → (section, paragraph, control).
pub(super) fn resolve_top_table(
    document: &rhwp::model::document::Document,
    table_no: usize,
) -> Result<(usize, usize, usize), String> {
    use rhwp::document_core::queries::table_extract::extract_tables;
    let grids = extract_tables(document);
    match grids
        .iter()
        .find(|g| g.index == table_no && g.container_path.is_empty())
    {
        Some(g) => Ok((g.section, g.paragraph, g.control)),
        None => {
            let n = grids.iter().filter(|g| g.container_path.is_empty()).count();
            Err(format!(
                "오류: 본문 최상위 표 {table_no} 번이 없습니다 (최상위 표 {n}개)."
            ))
        }
    }
}
/// [#3603] 격자 주소(export-tables 좌표) → 모델 좌표 해석.
/// CLI(edit set-cell)와 세션 도구(hwp_doc_set_cell)가 공유한다 — 병합으로 덮인 칸은
/// 앵커 좌표를 안내하며 실패한다(보호 동작). 반환: (sec, para, ctrl, cell_idx,
/// 문단별 글자 수, 기존 텍스트).
pub(crate) enum CellResolveError {
    Usage(String),
    Runtime(String),
}
#[allow(clippy::type_complexity)]
pub(crate) fn resolve_table_cell(
    document: &rhwp::model::document::Document,
    table_no: usize,
    row: u16,
    col: u16,
) -> Result<(usize, usize, usize, usize, Vec<usize>, String), CellResolveError> {
    use rhwp::document_core::queries::table_extract::extract_tables;
    use rhwp::model::control::Control;
    let grids = extract_tables(document);
    let Some(grid) = grids
        .iter()
        .find(|g| g.index == table_no && g.container_path.is_empty())
    else {
        let top_level = grids.iter().filter(|g| g.container_path.is_empty()).count();
        return Err(CellResolveError::Runtime(format!(
            "오류: 본문 최상위 표 {} 번이 없습니다 (최상위 표 {}개; 중첩 표는 v1 범위 밖).",
            table_no, top_level
        )));
    };
    let Some(Control::Table(table)) = document.sections[grid.section].paragraphs[grid.paragraph]
        .controls
        .get(grid.control)
    else {
        return Err(CellResolveError::Runtime(
            "오류: 표 컨트롤 좌표 해석 실패 (내부 불일치).".into(),
        ));
    };
    if row >= table.row_count || col >= table.col_count {
        return Err(CellResolveError::Usage(format!(
            "오류: 좌표가 격자를 벗어났습니다 — 표 {} 는 {}x{} 입니다.",
            table_no, table.row_count, table.col_count
        )));
    }
    match table
        .cells
        .iter()
        .enumerate()
        .find(|(_, c)| c.row == row && c.col == col)
    {
        Some((cell_idx, c)) => {
            let para_lens: Vec<usize> = c
                .paragraphs
                .iter()
                .map(|p| p.text.chars().count())
                .collect();
            let old_text = c
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join(
                    "
",
                )
                .trim()
                .to_string();
            Ok((
                grid.section,
                grid.paragraph,
                grid.control,
                cell_idx,
                para_lens,
                old_text,
            ))
        }
        None => {
            let anchor = table.cells.iter().find(|c| {
                c.row <= row && row < c.row + c.row_span && c.col <= col && col < c.col + c.col_span
            });
            Err(CellResolveError::Usage(match anchor {
                Some(a) => format!(
                    "오류: ({},{}) 는 병합으로 덮인 칸입니다 — 앵커 ({},{}) 를 지정하세요.",
                    row, col, a.row, a.col
                ),
                None => format!("오류: ({},{}) 위치에 셀이 없습니다.", row, col),
            }))
        }
    }
}
