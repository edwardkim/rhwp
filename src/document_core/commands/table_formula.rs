//! 표 계산식 필드 — 계산식·쉬운 계산식·블록 계산식의 식 보존과 다시 계산
//!
//! 한/글 12.30은 계산 결과를 일반 글이 아니라 셀의 FORMULA 필드로 남긴다. 표의 아무 셀이나
//! 고치면 커서가 그 셀에 있는 채로 그 표의 계산식을 모두 다시 계산했다(어느 식도 참조하지 않는
//! 셀에 한 글자를 치자 모든 결과가 바뀌었다). 뒤 셀의 계산식을 앞 셀이 참조해도(A9 `=B10+0`,
//! B10 `=A10*2`) 바뀐 값을 썼다. 한/글 도움말은 손으로 고친 결과를 그 식이 쓰는 셀이 바뀔 때까지
//! 다시 계산하지 않는다고 한다(`table/calculation/calculation.htm` 계산식 다시 계산).

use std::collections::HashMap;

use super::super::DocumentCore;
use super::table_ops::parse_cell_number;
use crate::document_core::queries::field_query::{
    insert_field_in_para, remove_field_control_in_para,
};
use crate::document_core::table_calc::{
    evaluate_formula_in_table, format_formula_result, formula_command, formula_references,
    is_invalid_result_error, split_formula_command, tokenize, DirectionKind, TableContext, Token,
    FORMULA_FIELD_PROP, INVALID_RESULT_TEXT,
};
use crate::error::HwpError;
use crate::model::control::{Control, Field, FieldType, Parameter, ParameterList};
use crate::model::table::Table;
use crate::parser::tags;

/// 표 안의 계산식 필드 하나.
struct TableFormula {
    cell: usize,
    para: usize,
    /// 문단 `field_ranges` 번호
    range: usize,
    /// 문단 `controls` 번호
    control: usize,
    row: usize,
    col: usize,
    formula: String,
    result_format: String,
    last_result: String,
    /// 지금 셀에 보이는 결과 글
    text: String,
    /// 식이 참조하는 0-based (열, 행). 읽을 수 없는 식이면 `None`.
    refs: Option<Vec<(usize, usize)>>,
}

/// 표의 셀 값과 계산식 셀. 계산식 평가가 셀을 읽는 창구다.
struct TableCells<'a> {
    table: &'a Table,
    by_position: HashMap<(usize, usize), usize>,
    directions: HashMap<(usize, usize), Vec<DirectionKind>>,
}

impl<'a> TableCells<'a> {
    fn new(table: &'a Table, formulas: &[TableFormula]) -> Self {
        let by_position = table
            .cells
            .iter()
            .enumerate()
            .map(|(i, cell)| ((cell.row as usize, cell.col as usize), i))
            .collect();
        let mut directions: HashMap<(usize, usize), Vec<DirectionKind>> = HashMap::new();
        for formula in formulas {
            for token in tokenize(&formula.formula).unwrap_or_default() {
                if let Token::Direction(dir) = token {
                    directions
                        .entry((formula.row, formula.col))
                        .or_default()
                        .push(dir);
                }
            }
        }
        Self {
            table,
            by_position,
            directions,
        }
    }

    /// 셀 첫 문단의 숫자. 계산식 셀은 첫 계산식의 결과를 읽는다(한/글 12.30: 결과 8과 4.00이
    /// 나란히 든 셀을 8로 셌다). `results`는 다시 계산하는 중인 계산식 필드의 새 글이다.
    fn value(
        &self,
        col: usize,
        row: usize,
        formulas: &[TableFormula],
        results: &[String],
    ) -> Option<f64> {
        let cell = *self.by_position.get(&(row, col))?;
        let para = self.table.cells[cell].paragraphs.first()?;
        let first = formulas
            .iter()
            .zip(results)
            .filter(|(f, _)| f.cell == cell && f.para == 0)
            .filter_map(|(f, text)| Some((para.field_ranges.get(f.range)?.start_char_idx, text)))
            .min_by_key(|(start, _)| *start);
        match first {
            Some((_, text)) => parse_cell_number(text),
            None => parse_cell_number(&para.text),
        }
    }

    fn same_direction(&self, col: usize, row: usize, dir: DirectionKind) -> bool {
        self.directions
            .get(&(row, col))
            .is_some_and(|dirs| dirs.contains(&dir))
    }

    fn context(&self, row: usize, col: usize) -> TableContext {
        TableContext {
            row_count: self.table.row_count as usize,
            col_count: self.table.col_count as usize,
            current_row: row,
            current_col: col,
        }
    }
}

/// 표의 계산식 필드를 한/글이 다시 계산하는 차례(행, 열)로 모은다.
fn table_formulas(table: &Table) -> Vec<TableFormula> {
    let mut formulas = Vec::new();
    for (cell_idx, cell) in table.cells.iter().enumerate() {
        for (para_idx, para) in cell.paragraphs.iter().enumerate() {
            for (range_idx, range) in para.field_ranges.iter().enumerate() {
                let Some(Control::Field(field)) = para.controls.get(range.control_idx) else {
                    continue;
                };
                if field.field_type != FieldType::Formula {
                    continue;
                }
                let Some((formula, result_format, last_result)) =
                    split_formula_command(&field.command)
                else {
                    continue;
                };
                let ctx = TableContext {
                    row_count: table.row_count as usize,
                    col_count: table.col_count as usize,
                    current_row: cell.row as usize,
                    current_col: cell.col as usize,
                };
                formulas.push(TableFormula {
                    refs: formula_references(formula, &ctx).ok(),
                    cell: cell_idx,
                    para: para_idx,
                    range: range_idx,
                    control: range.control_idx,
                    row: cell.row as usize,
                    col: cell.col as usize,
                    formula: formula.to_string(),
                    result_format: result_format.to_string(),
                    last_result: last_result.to_string(),
                    text: para
                        .text
                        .chars()
                        .skip(range.start_char_idx)
                        .take(range.end_char_idx.saturating_sub(range.start_char_idx))
                        .collect(),
                });
            }
        }
    }
    formulas.sort_by_key(|f| (f.row, f.col, f.cell, f.para, f.range));
    formulas
}

/// 셀 자리(병합 범위 포함)가 참조 (열, 행)을 덮는가.
fn covers(table: &Table, cell: usize, (col, row): (usize, usize)) -> bool {
    table.cells.get(cell).is_some_and(|c| {
        let (r, c0) = (c.row as usize, c.col as usize);
        (r..r + c.row_span.max(1) as usize).contains(&row)
            && (c0..c0 + c.col_span.max(1) as usize).contains(&col)
    })
}

/// 서로 참조하는 계산식(순환)에 든 계산식 번호. 자기 셀만 참조하는 식은 따로 본다. 한/글 12.30은
/// 서로 참조하는 식(A1 `=B1+1`, B1 `=A1+1`)의 결과를 그대로 두었다.
// ponytail: 계산식마다 너비 우선 탐색이라 O(n·(n+e))다. 표 하나의 계산식 수에서는 충분하고, 커지면 SCC로 바꾼다.
fn cyclic_formulas(table: &Table, formulas: &[TableFormula]) -> Vec<bool> {
    let edges: Vec<Vec<usize>> = formulas
        .iter()
        .map(|f| {
            let refs = f.refs.as_deref().unwrap_or_default();
            (0..formulas.len())
                .filter(|&j| {
                    formulas[j].cell != f.cell
                        && refs.iter().any(|&r| covers(table, formulas[j].cell, r))
                })
                .collect()
        })
        .collect();
    (0..formulas.len())
        .map(|start| {
            let mut seen = vec![false; formulas.len()];
            let mut queue = edges[start].clone();
            while let Some(next) = queue.pop() {
                if next == start {
                    return true;
                }
                if !std::mem::replace(&mut seen[next], true) {
                    queue.extend(&edges[next]);
                }
            }
            false
        })
        .collect()
}

/// 계산 결과를 셀 글로. 값을 낼 수 없는 식은 한/글처럼 "잘못된 계산식"을 쓰고, 읽을 수 없는 식은
/// `Err`로 돌려준다.
fn result_text(
    evaluated: Result<(f64, Vec<(usize, usize)>), String>,
    result_format: &str,
) -> Result<(Option<f64>, String), String> {
    match evaluated {
        Ok((value, _)) => Ok((Some(value), format_formula_result(value, result_format))),
        Err(error) if is_invalid_result_error(&error) => {
            Ok((None, INVALID_RESULT_TEXT.to_string()))
        }
        Err(error) => Err(error),
    }
}

/// 한/글 저장본처럼 FORMULA 필드의 명령과 HWPX 매개변수(Prop·Command·Formula·ResultFormat·
/// LastResult)를 함께 쓴다.
fn set_formula_command(field: &mut Field, formula: &str, result_format: &str, last_result: &str) {
    field.command = formula_command(formula, result_format, last_result);
    let string = |name: &str, value: &str| Parameter::String {
        name: Some(name.to_string()),
        value: value.to_string(),
        preserve_space: false,
    };
    field.parameters = ParameterList {
        name: None,
        items: vec![
            Parameter::Integer {
                name: Some("Prop".to_string()),
                value: FORMULA_FIELD_PROP as i64,
            },
            string("Command", &field.command),
            string("Formula", formula),
            string("ResultFormat", result_format),
            string("LastResult", last_result),
        ],
    };
    field.raw_parameters_xml = None;
}

/// 한/글 계산식 창의 결과 형식인가(`%g`, `%.Nf`, 끝의 `,`).
fn valid_result_format(result_format: &str) -> bool {
    let spec = result_format.strip_suffix(',').unwrap_or(result_format);
    spec == "%g"
        || spec
            .strip_prefix("%.")
            .and_then(|rest| rest.strip_suffix('f'))
            .and_then(|n| n.parse::<usize>().ok())
            .is_some_and(|n| n <= 20)
}

impl DocumentCore {
    fn table_at(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&Table, HwpError> {
        match self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(parent_para_idx))
            .and_then(|p| p.controls.get(control_idx))
        {
            Some(Control::Table(table)) => Ok(table),
            _ => Err(HwpError::RenderError("표 컨트롤이 아님".into())),
        }
    }

    /// 계산식을 표의 셀 값과 계산식 셀로 평가한다(`evaluate_table_formula`와 같은 규칙).
    pub(super) fn evaluate_formula_at(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        row: usize,
        col: usize,
        formula: &str,
    ) -> Result<(f64, Vec<(usize, usize)>), String> {
        let table = self
            .table_at(section_idx, parent_para_idx, control_idx)
            .map_err(|e| e.to_string())?;
        let formulas = table_formulas(table);
        let cells = TableCells::new(table, &formulas);
        let results: Vec<String> = formulas.iter().map(|f| f.text.clone()).collect();
        evaluate_formula_in_table(
            formula,
            &cells.context(row, col),
            &|c, r| cells.value(c, r, &formulas, &results),
            &|c, r, dir| cells.same_direction(c, r, dir),
        )
    }

    /// 셀에 계산식 필드를 넣거나 그 셀 계산식을 바꾼다(한/글 표 > 계산식·쉬운 계산식·블록 계산식).
    ///
    /// 셀 첫 문단의 글은 계산 결과로 바뀐다. 결과 형식은 `%g`(기본)·`%.Nf`(소수 N자리)에 세 자리
    /// 쉼표면 `,`를 붙인다. 0으로 나누기처럼 값을 낼 수 없으면 "잘못된 계산식"을 쓴다. 자기 셀을
    /// 참조하거나 다른 계산식과 서로 참조하는 식은 받지 않는다. 넣은 뒤 표의 계산식을 다시 계산한다.
    pub fn set_table_cell_formula(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        formula: &str,
        result_format: &str,
    ) -> Result<String, HwpError> {
        let formula = formula.trim();
        if !valid_result_format(result_format) {
            return Err(HwpError::RenderError(format!(
                "계산식 결과 형식 오류: {result_format}"
            )));
        }
        let table = self.table_at(section_idx, parent_para_idx, control_idx)?;
        let cell = table
            .cells
            .get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {cell_idx} 범위 초과")))?;
        let (row, col) = (cell.row as usize, cell.col as usize);
        let para = cell
            .paragraphs
            .first()
            .ok_or_else(|| HwpError::RenderError("결과 셀 문단을 찾을 수 없음".into()))?;
        // 셀에 계산식 필드가 있으면 그 식을 바꾼다. 다른 개체·필드가 든 셀은 받지 않는다.
        let existing = para.field_ranges.iter().position(|range| {
            matches!(
                para.controls.get(range.control_idx),
                Some(Control::Field(f)) if f.field_type == FieldType::Formula
            )
        });
        let other_controls = para
            .controls
            .iter()
            .filter(|c| !matches!(c, Control::Field(f) if f.field_type == FieldType::Formula));
        if other_controls.count() > 0 || (existing.is_none() && !para.field_ranges.is_empty()) {
            return Err(HwpError::RenderError(
                "계산식 오류: 결과 셀에 개체나 다른 필드가 있습니다".into(),
            ));
        }
        let text_len = para.text.chars().count();

        let ctx = TableContext {
            row_count: table.row_count as usize,
            col_count: table.col_count as usize,
            current_row: row,
            current_col: col,
        };
        let refs = formula_references(formula, &ctx)
            .map_err(|e| HwpError::RenderError(format!("계산식 오류: {e}")))?;
        if refs.iter().any(|&r| covers(table, cell_idx, r)) {
            return Err(HwpError::RenderError(
                "계산식 오류: 결과 셀을 참조하는 계산식입니다".into(),
            ));
        }
        // 새 식을 이 셀의 식으로 두고 서로 참조하는 계산식이 생기는지 본다.
        let mut graph: Vec<TableFormula> = table_formulas(table)
            .into_iter()
            .filter(|f| f.cell != cell_idx)
            .collect();
        graph.push(TableFormula {
            cell: cell_idx,
            para: 0,
            range: 0,
            control: 0,
            row,
            col,
            formula: formula.to_string(),
            result_format: String::new(),
            last_result: String::new(),
            text: String::new(),
            refs: Some(refs.clone()),
        });
        if cyclic_formulas(table, &graph).last() == Some(&true) {
            return Err(HwpError::RenderError(
                "계산식 오류: 서로 참조하는 계산식입니다".into(),
            ));
        }

        let (value, text) = result_text(
            self.evaluate_formula_at(section_idx, parent_para_idx, control_idx, row, col, formula),
            result_format,
        )
        .map_err(|e| HwpError::RenderError(format!("계산식 오류: {e}")))?;

        let (control, start, old_len) = match existing {
            Some(range_idx) => {
                let para = self.get_cell_paragraph_mut(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    0,
                )?;
                let range = para.field_ranges[range_idx].clone();
                (
                    range.control_idx,
                    range.start_char_idx,
                    range.end_char_idx - range.start_char_idx,
                )
            }
            None => {
                let field_id = self.next_click_here_field_id();
                let para = self.get_cell_paragraph_mut(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    0,
                )?;
                if text_len > 0 {
                    para.replace_text_keeping_slots(0, text_len, "", None, false);
                }
                let field = Field {
                    field_type: FieldType::Formula,
                    extra_properties: FORMULA_FIELD_PROP,
                    field_id,
                    ctrl_id: tags::FIELD_FORMULA,
                    // 한/글은 FORMULA 필드의 fieldid를 ctrl id('%fmu')로 쓴다.
                    instance_id: Some(tags::FIELD_FORMULA),
                    ..Default::default()
                };
                let (start, control) = insert_field_in_para(para, 0, field)?;
                if let Some(range) = para
                    .field_ranges
                    .iter_mut()
                    .find(|range| range.control_idx == control)
                {
                    range.end_field_id = tags::FIELD_FORMULA;
                }
                (control, start, 0)
            }
        };
        {
            let para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                0,
            )?;
            if let Some(Control::Field(field)) = para.controls.get_mut(control) {
                set_formula_command(field, formula, result_format, &text);
            }
        }
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.replace_text_in_cell_field_impl(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            0,
            start,
            old_len,
            &text,
            false,
            Some(control),
        )?;
        // 이 셀을 쓰는 계산식은 손으로 고친 결과까지 다시 계산한다. 쪽은 끝에 한 번 나눈다.
        self.recalculate_table_formulas(
            section_idx,
            parent_para_idx,
            control_idx,
            Some(cell_idx),
            false,
        )?;
        self.flush_deferred_pagination();

        let cells: Vec<_> = refs
            .into_iter()
            .map(|(col, row)| serde_json::json!({"row": row, "col": col}))
            .collect();
        Ok(serde_json::json!({
            "ok": true,
            "result": value,
            "text": text,
            "formula": formula,
            "cells": cells,
        })
        .to_string())
    }

    /// 셀의 계산식 필드를 지우고 결과 글은 일반 글로 남긴다(한/글 12.30 계산식 지우기: 결과 6이
    /// 글로 남았다). 결과: `{"ok":true,"removed":지운 수}`.
    pub fn remove_table_cell_formulas(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<String, HwpError> {
        let para_count = self
            .table_at(section_idx, parent_para_idx, control_idx)?
            .cells
            .get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {cell_idx} 범위 초과")))?
            .paragraphs
            .len();
        let mut removed = 0;
        for para_idx in 0..para_count {
            loop {
                let para = self.get_cell_paragraph_mut(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    para_idx,
                )?;
                let Some(range) = para
                    .field_ranges
                    .iter()
                    .find(|range| {
                        matches!(
                            para.controls.get(range.control_idx),
                            Some(Control::Field(f)) if f.field_type == FieldType::Formula
                        )
                    })
                    .cloned()
                else {
                    break;
                };
                let text: String = para
                    .text
                    .chars()
                    .skip(range.start_char_idx)
                    .take(range.end_char_idx - range.start_char_idx)
                    .collect();
                remove_field_control_in_para(para, range.control_idx)?;
                if let Some(section) = self.document.sections.get_mut(section_idx) {
                    section.raw_stream = None;
                }
                // 지운 결과 글을 같은 자리에 일반 글로 다시 넣는다. 셀 글 교체 경로가 줄을 다시 짠다.
                self.replace_text_in_cell_native_impl(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    para_idx,
                    range.start_char_idx,
                    0,
                    &text,
                    false,
                )?;
                removed += 1;
            }
        }
        if removed > 0 {
            self.flush_deferred_pagination();
        }
        Ok(serde_json::json!({"ok": true, "removed": removed}).to_string())
    }

    /// 표의 계산식 필드를 다시 계산한다. 한/글처럼 표의 셀을 고칠 때마다 부른다.
    ///
    /// `edited_cell`은 고친 셀이다. 손으로 고친 결과(셀 글이 마지막 결과와 다름)는 그 식이 이
    /// 셀을 쓸 때만 다시 계산하고, `None`이면 모두 다시 계산한다. 읽을 수 없는 식과 서로 참조하는
    /// 식은 마지막 결과를 둔다. 셀 쪽 나눔은 미루며 `paginate`면 끝에 쪽을 나눈다. 결과:
    /// `{"ok":true,"changed":[셀 번호…],"cellFlowChanged":bool}`.
    pub fn recalculate_table_formulas(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        edited_cell: Option<usize>,
        paginate: bool,
    ) -> Result<String, HwpError> {
        let table = self.table_at(section_idx, parent_para_idx, control_idx)?;
        let formulas = table_formulas(table);
        if formulas.is_empty() {
            return Ok(r#"{"ok":true,"changed":[],"cellFlowChanged":false}"#.to_string());
        }
        let cyclic = cyclic_formulas(table, &formulas);
        // 한/글 12.30은 자기 셀을 참조하는 식(A2 `=A2+1`)에 "잘못된 계산식"을 썼다.
        let self_ref: Vec<bool> = formulas
            .iter()
            .map(|f| {
                f.refs
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .any(|&r| covers(table, f.cell, r))
            })
            .collect();
        let active: Vec<bool> = formulas
            .iter()
            .zip(&cyclic)
            .zip(&self_ref)
            .map(|((f, &cycle), &self_ref)| {
                let Some(refs) = f.refs.as_deref() else {
                    return false;
                };
                self_ref
                    || !cycle
                        && (f.text == f.last_result
                            || edited_cell.is_none_or(|edited| {
                                refs.iter().any(|&r| covers(table, edited, r))
                            }))
            })
            .collect();
        let cells = TableCells::new(table, &formulas);
        let mut results: Vec<String> = formulas.iter().map(|f| f.text.clone()).collect();
        // 앞 셀이 뒤 셀의 계산식을 참조해도 바뀐 값을 쓰도록 바뀌지 않을 때까지 되풀이한다. 순환은
        // 위에서 뺐으므로 참조 깊이만큼 돈다.
        for _ in 0..=formulas.len() {
            let mut changed = false;
            for (i, formula) in formulas.iter().enumerate() {
                if !active[i] {
                    continue;
                }
                if self_ref[i] {
                    results[i] = INVALID_RESULT_TEXT.to_string();
                    continue;
                }
                let evaluated = evaluate_formula_in_table(
                    &formula.formula,
                    &cells.context(formula.row, formula.col),
                    &|c, r| cells.value(c, r, &formulas, &results),
                    &|c, r, dir| cells.same_direction(c, r, dir),
                );
                let Ok((_, text)) = result_text(evaluated, &formula.result_format) else {
                    continue;
                };
                if results[i] != text {
                    results[i] = text;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut changed_cells = Vec::new();
        let mut flow_changed = false;
        for ((formula, text), _) in formulas
            .iter()
            .zip(&results)
            .zip(&active)
            .filter(|(_, &active)| active)
        {
            if *text == formula.text && *text == formula.last_result {
                continue;
            }
            let para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                formula.cell,
                formula.para,
            )?;
            let range = para.field_ranges[formula.range].clone();
            if let Some(Control::Field(field)) = para.controls.get_mut(formula.control) {
                set_formula_command(field, &formula.formula, &formula.result_format, text);
            }
            if let Some(section) = self.document.sections.get_mut(section_idx) {
                section.raw_stream = None;
            }
            if *text == formula.text {
                continue;
            }
            let edited = self.replace_text_in_cell_field_impl(
                section_idx,
                parent_para_idx,
                control_idx,
                formula.cell,
                formula.para,
                range.start_char_idx,
                range.end_char_idx - range.start_char_idx,
                text,
                false,
                Some(formula.control),
            )?;
            flow_changed |= serde_json::from_str::<serde_json::Value>(&edited)
                .ok()
                .and_then(|v| v.get("cellFlowChanged").and_then(|f| f.as_bool()))
                .unwrap_or(true);
            if !changed_cells.contains(&formula.cell) {
                changed_cells.push(formula.cell);
            }
        }
        if paginate && !changed_cells.is_empty() {
            self.flush_deferred_pagination();
        }
        Ok(serde_json::json!({
            "ok": true,
            "changed": changed_cells,
            "cellFlowChanged": flow_changed,
        })
        .to_string())
    }
}
