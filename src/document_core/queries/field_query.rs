//! 필드 조회/설정 API (Task 230)
//!
//! 문서 전체에서 필드를 재귀 탐색하여 조회·설정하는 기능을 제공한다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{Control, Field, FieldType};
use crate::model::event::DocumentEvent;
use crate::model::paragraph::{FieldRange, Paragraph};
use crate::parser::tags;

/// 필드 위치 정보
#[derive(Debug, Clone)]
pub struct FieldLocation {
    pub section_index: usize,
    pub para_index: usize,
    /// 표/글상자 내 필드인 경우 중첩 경로
    pub nested_path: Vec<NestedEntry>,
}

/// 중첩 경로 항목 (표 셀 또는 글상자 내부)
#[derive(Debug, Clone)]
pub enum NestedEntry {
    /// 표 셀: (control_index, cell_index, para_index)
    TableCell {
        control_index: usize,
        cell_index: usize,
        para_index: usize,
    },
    /// 글상자: (control_index, para_index)
    TextBox {
        control_index: usize,
        para_index: usize,
    },
}

/// 필드 검색 결과
#[derive(Debug)]
pub struct FieldInfo {
    pub field: Field,
    pub location: FieldLocation,
    /// 필드 범위 내 텍스트 (빈 필드이면 빈 문자열)
    pub value: String,
    /// field_ranges에서의 인덱스
    pub field_range_index: usize,
    /// 필드가 속한 리스트 아이디 (한글 커서 좌표계 — [`ListEntry`] 참고)
    pub list_id: u32,
    /// 그 리스트 안에서의 문단 번호
    pub para_in_list: usize,
    /// 문단 안에서의 시작 위치 (코드 유닛). 셀 필드는 0.
    pub start_pos: usize,
    /// 문단 안에서의 끝 위치 (코드 유닛). 셀 필드는 0.
    pub end_pos: usize,
}

/// 본문(루트) 리스트의 아이디.
pub const ROOT_LIST_ID: u32 = 0;

/// 서브리스트(표 셀·글상자)가 받는 첫 아이디.
///
/// 1 번은 문서마다 하나 비어 있다 — 한글2022 실측으로 세 문서에서 모두 그랬고, 그 자리에
/// 무엇이 있는지는 아직 모른다(바탕쪽으로 짐작만 한다). 규명 전까지 상수로 둔다.
pub const FIRST_SUB_LIST_ID: u32 = 2;

/// 서브리스트 한 칸 — 한글 `GetPos`/`SetPos` 의 `list` 좌표계.
///
/// 번호는 **문서 순서 깊이 우선**으로 붙는다: 셀에 번호를 준 **뒤** 그 셀 안으로 내려간다
/// (한글2022 실측 — 3중 중첩 표에서 부모 셀 28 → 자식 표 → 부모 셀 29 순서를 확인).
#[derive(Debug, Clone)]
pub struct ListEntry {
    pub list_id: u32,
    /// 셀이면 true, 글상자면 false.
    pub is_cell: bool,
    /// 이 리스트를 담은 리스트의 아이디.
    pub host_list_id: u32,
    pub section_index: usize,
    /// 호스트 리스트 안에서 이 리스트를 담은 문단 번호.
    pub host_para_index: usize,
    /// 그 문단 안에서 컨트롤 번호 — 상위로 올라갈 때 위치는 `8 × control_index` 다.
    pub control_index: usize,
    pub cell_index: usize,
    pub para_count: usize,
    /// 표 안에서의 격자 자리 — 셀이 아니면 `None`. 표 셀 이동(`TableRightCell` 따위)이 딛는다.
    pub grid: Option<CellGrid>,
}

/// 표 셀 하나의 격자 자리와 병합 크기.
#[derive(Debug, Clone, Copy)]
pub struct CellGrid {
    pub row: u16,
    pub col: u16,
    pub row_span: u16,
    pub col_span: u16,
    /// 그 표가 담은 셀 수 — 표의 끝을 알아야 이동이 멈출 자리를 안다.
    pub cell_count: usize,
}

/// 리스트 번호를 매기며 걷는 상태.
struct ListWalk {
    next_id: u32,
    lists: Vec<ListEntry>,
}

impl ListWalk {
    fn new() -> Self {
        Self {
            next_id: FIRST_SUB_LIST_ID,
            lists: Vec::new(),
        }
    }

    fn alloc(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl DocumentCore {
    /// 문서 전체에서 모든 필드를 검색하여 목록으로 반환한다.
    pub fn collect_all_fields(&self) -> Vec<FieldInfo> {
        self.collect_fields_and_lists().0
    }

    /// 필드와 리스트 목록을 **한 번의 순회로** 함께 모은다.
    ///
    /// 둘을 따로 걸으면 리스트 번호가 서로 어긋난다 — 필드가 말하는 `listId` 와 리스트 표의
    /// 아이디가 같은 순회에서 나와야 커서 좌표가 성립한다.
    pub fn collect_fields_and_lists(&self) -> (Vec<FieldInfo>, Vec<ListEntry>) {
        let mut result = Vec::new();
        let mut walk = ListWalk::new();
        for (si, sec) in self.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                let loc = FieldLocation {
                    section_index: si,
                    para_index: pi,
                    nested_path: Vec::new(),
                };
                collect_fields_from_paragraph(para, &loc, ROOT_LIST_ID, &mut walk, &mut result);
            }
        }
        (result, walk.lists)
    }

    /// 본문 문단의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let section = self
                .document
                .sections
                .get_mut(section_idx)
                .ok_or_else(|| HwpError::InvalidField("구역 인덱스 초과".into()))?;
            section.raw_stream = None;
            let para = section
                .paragraphs
                .get_mut(para_idx)
                .ok_or_else(|| HwpError::InvalidField("문단 인덱스 초과".into()))?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            &self.document.sections[section_idx].paragraphs[para_idx],
        );
        self.reflow_paragraph(section_idx, para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: para_idx,
            offset: inserted_offset,
            len: 0,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// 셀/글상자 내 문단의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        _is_textbox: bool,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// path 기반 중첩 표 셀의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::InvalidField("cellPath가 비어 있음".into()));
        }
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// getFieldList: 모든 필드를 JSON 배열로 반환
    pub fn get_field_list_json(&self) -> String {
        let fields = self.collect_all_fields();
        let entries: Vec<String> = fields
            .iter()
            .map(|fi| {
                let name = fi.field.field_name().unwrap_or("");
                let guide = fi.field.guide_text().unwrap_or("");
                let location_json = field_location_json(&fi.location);
                let (start_char_idx, end_char_idx) = field_range_bounds(self, fi)
                    .unwrap_or((0, fi.value.chars().count()));
                format!(
                    "{{\"fieldId\":{},\"fieldType\":\"{}\",\"cellField\":{},\"name\":{},\"guide\":{},\"command\":{},\"value\":{},\"location\":{},\"startCharIdx\":{},\"endCharIdx\":{},\"editableInForm\":{},\"listId\":{},\"paraInList\":{},\"startPos\":{},\"endPos\":{}}}",
                    fi.field.field_id,
                    fi.field.field_type_str(),
                    // 셀 구역 이름(가상 필드)과 문단 안 누름틀은 다른 것이다. `fieldType` 은 둘 다
                    // ClickHere 라 가르지 못한다 — 소비자가 종류를 물을 유일한 자리다.
                    fi.field.ctrl_id == 0,
                    json_escape(name),
                    json_escape(guide),
                    json_escape(&fi.field.command),
                    json_escape(&fi.value),
                    location_json,
                    start_char_idx,
                    end_char_idx,
                    fi.field.is_editable_in_form(),
                    // 한글 커서 좌표(list/para/pos) — 웹한글컨트롤 GetPos·MoveToField 가 쓴다.
                    fi.list_id,
                    fi.para_in_list,
                    fi.start_pos,
                    fi.end_pos,
                )
            })
            .collect();
        format!("[{}]", entries.join(","))
    }

    /// 아무 내용도 없는 빈 문서인가 — 웹한글컨트롤 `IsEmpty`(§8.2.7).
    ///
    /// 구역·단 정의는 새 문서에도 늘 있으므로 "내용"으로 세지 않는다. 글자 하나, 표 하나라도
    /// 있으면 빈 문서가 아니다(오라클 실측: 새 문서 true, 영수증 서식 false).
    pub fn is_empty_document(&self) -> bool {
        self.document.sections.iter().all(|section| {
            section.paragraphs.iter().all(|para| {
                para.text.is_empty()
                    && para
                        .controls
                        .iter()
                        .all(|ctrl| matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)))
            })
        })
    }

    /// 한글 커서 좌표계(`GetPos`/`SetPos`/`MovePos`)를 쓰는 데 필요한 문서 사실을 모아 준다.
    ///
    /// ```json
    /// {"listCount":325,
    ///  "root":{"paraCount":3,"topPos":72,"endPara":2,"endPos":0},
    ///  "lists":[{"listId":2,"isCell":true,"hostListId":0,"hostPara":0,
    ///            "controlIndex":2,"cellIndex":0,"paraCount":1}, …]}
    /// ```
    ///
    /// `root.topPos` 는 "문서의 시작"(`MovePos(2)`)이 떨어지는 자리다. 한글은 문단 앞머리의
    /// **자리차지 컨트롤을 건너뛴다** — 영수증 서식(자리차지 표 7개)은 72, 보도자료 서식
    /// (인라인 표)은 16 이었다. 컨트롤 하나가 8 코드 유닛이다.
    pub fn get_cursor_model_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let entries: Vec<String> = lists
            .iter()
            .map(|l| {
                let grid = match l.grid {
                    Some(g) => format!(
                        ",\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"cellCount\":{}",
                        g.row, g.col, g.row_span, g.col_span, g.cell_count,
                    ),
                    None => String::new(),
                };
                // `hostPara` 는 **구역 안 번호**다. 구역이 여럿이면 다른 구역의 같은 번호와
                // 겹치므로 `sectionIndex` 를 함께 줘야 표를 갈라 볼 수 있다.
                format!(
                    "{{\"listId\":{},\"isCell\":{},\"hostListId\":{},\"sectionIndex\":{},\"hostPara\":{},\"controlIndex\":{},\"cellIndex\":{},\"paraCount\":{}{}}}",
                    l.list_id,
                    l.is_cell,
                    l.host_list_id,
                    l.section_index,
                    l.host_para_index,
                    l.control_index,
                    l.cell_index,
                    l.para_count,
                    grid,
                )
            })
            .collect();

        // 본문은 **구역을 가로질러** 하나의 리스트다(한글 실측). 첫 구역만 세면 다구역 문서에서
        // 문단 번호가 통째로 어긋난다 — `root_para_location` 의 주석 참고.
        let para_count = root_para_count(self);
        let top_pos = self
            .document
            .sections
            .first()
            .and_then(|s| s.paragraphs.first())
            .map(leading_anchor_pos)
            .unwrap_or(0);
        let end_para = para_count.saturating_sub(1);
        let end_pos = root_para_location(self, end_para)
            .and_then(|(si, pi)| self.document.sections.get(si)?.paragraphs.get(pi))
            .map(|p| (p.char_count as usize).saturating_sub(1))
            .unwrap_or(0);

        format!(
            "{{\"listCount\":{},\"root\":{{\"paraCount\":{},\"topPos\":{},\"endPara\":{},\"endPos\":{}}},\"lists\":[{}]}}",
            FIRST_SUB_LIST_ID as usize + lists.len(),
            para_count,
            top_pos,
            end_para,
            end_pos,
            entries.join(","),
        )
    }

    /// getFieldValue: field_id로 필드 값 조회
    pub fn get_field_value_by_id(&self, field_id: u32) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        for fi in &fields {
            if fi.field.field_id == field_id {
                return Ok(format!(
                    "{{\"ok\":true,\"value\":{}}}",
                    json_escape(&fi.value)
                ));
            }
        }
        Err(HwpError::InvalidField(format!("필드 ID {} 없음", field_id)))
    }

    /// getFieldValueByName: 필드 이름으로 값 조회
    pub fn get_field_value_by_name(&self, name: &str) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        for fi in &fields {
            if let Some(field_name) = fi.field.field_name() {
                if field_name == name {
                    return Ok(format!(
                        "{{\"ok\":true,\"fieldId\":{},\"value\":{}}}",
                        fi.field.field_id,
                        json_escape(&fi.value),
                    ));
                }
            }
        }
        Err(HwpError::InvalidField(format!("필드 이름 '{}' 없음", name)))
    }

    /// setFieldValue: field_id로 필드 값 설정
    pub fn set_field_value_by_id(
        &mut self,
        field_id: u32,
        value: &str,
    ) -> Result<String, HwpError> {
        // 먼저 필드 위치 찾기
        let fields = self.collect_all_fields();
        let fi = fields
            .iter()
            .find(|f| f.field.field_id == field_id)
            .ok_or_else(|| HwpError::InvalidField(format!("필드 ID {} 없음", field_id)))?;

        let location = fi.location.clone();
        let fri = fi.field_range_index;
        let old_value = fi.value.clone();

        let section_index = location.section_index;
        self.set_field_text_at(&location, fri, value)?;
        self.recompose_section(section_index);

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"oldValue\":{},\"newValue\":{}}}",
            field_id,
            json_escape(&old_value),
            json_escape(value),
        ))
    }

    /// setFieldValueByName: 필드 이름으로 값 설정
    pub fn set_field_value_by_name(&mut self, name: &str, value: &str) -> Result<String, HwpError> {
        self.set_field_value_by_name_at(name, 0, value)
    }

    /// [#3476] 같은 이름이 여러 번 나오는 서식에서 **N 번째**(0 기준) 필드에 값을 넣는다.
    ///
    /// 규제영향분석서 같은 실제 제출 서식은 같은 항목 묶음을 여러 번 요구한다
    /// (`피규제집단명` ×14 등). 이름만으로 찾으면 첫 매치만 바뀌어 나머지를 채울 수 없다.
    /// 순서는 `collect_all_fields()` 가 주는 문서 순서와 같으므로, 소비자는
    /// `fields --json` 목록의 순번을 그대로 쓰면 된다.
    pub fn set_field_value_by_name_at(
        &mut self,
        name: &str,
        occurrence: usize,
        value: &str,
    ) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        let fi = fields
            .iter()
            .filter(|f| f.field.field_name().map(|n| n == name).unwrap_or(false))
            .nth(occurrence)
            .ok_or_else(|| {
                HwpError::InvalidField(format!("필드 이름 '{}'[{}] 없음", name, occurrence))
            })?;

        let field_id = fi.field.field_id;
        let location = fi.location.clone();
        let fri = fi.field_range_index;
        let old_value = fi.value.clone();
        let is_cell_field = fi.field.ctrl_id == 0; // 가상 셀 필드

        let section_index = location.section_index;

        if is_cell_field {
            // 셀 필드: 셀의 첫 문단 텍스트를 직접 교체
            self.set_cell_field_text(&location, value)?;
        } else {
            // ClickHere 필드: field_ranges 기반 교체
            self.set_field_text_at(&location, fri, value)?;
        }

        // raw_stream 무효화
        if let Some(sec) = self.document.sections.get_mut(section_index) {
            sec.raw_stream = None;
        }
        self.recompose_section(section_index);

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"oldValue\":{},\"newValue\":{}}}",
            field_id,
            json_escape(&old_value),
            json_escape(value),
        ))
    }

    /// 한글 커서 좌표(list/para/pos)에 누름틀을 넣는다 — 웹한글컨트롤 `CreateField` 용.
    ///
    /// 좌표계가 두 개다. 한글의 `pos` 는 **코드 유닛**(확장 컨트롤 하나가 8칸)이고 rhwp 의
    /// 삽입 API 는 **글자 번호**를 받는다. 그 사이를 여기서 옮긴다 — 호출 측이 옮기면
    /// `char_offsets` 가 없는 곳에서 추측하게 된다.
    pub fn insert_click_here_field_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        if list_id == ROOT_LIST_ID {
            let char_offset = self
                .document
                .sections
                .first()
                .and_then(|s| s.paragraphs.get(para_in_list))
                .map(|p| char_idx_at_code_unit(p, pos))
                .unwrap_or(0);
            return self.insert_click_here_field_at(
                0,
                para_in_list,
                char_offset,
                guide,
                memo,
                name,
                editable,
            );
        }

        let (_, lists) = self.collect_fields_and_lists();
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        let char_offset = self
            .cell_paragraph_at_path(section_index, host_para, &path)
            .map(|p| char_idx_at_code_unit(p, pos))
            .unwrap_or(0);
        self.insert_click_here_field_at_by_path(
            section_index,
            host_para,
            &path,
            char_offset,
            guide,
            memo,
            name,
            editable,
        )
    }

    /// 경로가 가리키는 셀 문단(읽기 전용). 좌표 변환에만 쓴다.
    fn cell_paragraph_at_path(
        &self,
        section_index: usize,
        host_para: usize,
        path: &[(usize, usize, usize)],
    ) -> Option<&Paragraph> {
        let mut para = self
            .document
            .sections
            .get(section_index)?
            .paragraphs
            .get(host_para)?;
        for (control_index, cell_index, para_index) in path {
            let ctrl = para.controls.get(*control_index)?;
            para = match ctrl {
                Control::Table(table) => {
                    table.cells.get(*cell_index)?.paragraphs.get(*para_index)?
                }
                // 그리기 개체는 글상자와 캡션을 연다 — 둘째 칸이 그중 몇 번째인지다.
                // 여기가 표만 알던 탓에 글상자·캡션 리스트는 **자리가 있어도 못 닿았다**
                // (한글은 그 리스트 끝을 18 로 답하는데 rhwp 는 0 이었다).
                Control::Shape(shape) => shape_lists(shape)
                    .into_iter()
                    .find(|(node, _)| node == cell_index)
                    .and_then(|(_, paragraphs)| paragraphs.get(*para_index))?,
                _ => return None,
            };
        }
        Some(para)
    }

    /// RenameField(웹한글컨트롤 §8.3.36) — 이름이 `oldname` 인 필드를 모두 `newname` 으로 바꾼다.
    ///
    /// 필드는 저장 경로가 두 갈래다. 누름틀은 문단의 CTRL_DATA 에, 셀 필드는 셀
    /// LIST_HEADER 의 추가 바이트에 이름을 싣는다. 누름틀 경로만 고치면 셀 필드는 **조용히
    /// 안 바뀐다** — `updateClickHereProps` 가 셀 필드에서 `{"ok":false}` 를 돌려주던 이유다.
    ///
    /// 같은 이름이 여러 번 나오면 **전부** 바꾼다(한글2022 실측: `pt_no` ×2 문서에서 한 번의
    /// 호출 뒤 `FieldExist("pt_no")` 가 false).
    ///
    /// 반환: `{"ok":true,"renamed":N}`. 없는 이름이면 `{"ok":false,"renamed":0}` —
    /// 오라클도 아무 일도 하지 않는다.
    pub fn rename_field_by_name(
        &mut self,
        oldname: &str,
        newname: &str,
    ) -> Result<String, HwpError> {
        if oldname.is_empty() {
            return Err(HwpError::InvalidField("바꿀 필드 이름이 비어 있음".into()));
        }
        // (위치, field_range 인덱스, 셀 필드 여부). ctrl_id == 0 은 collect_all_fields 가
        // 셀 이름으로 만들어 낸 가상 필드다.
        let targets: Vec<(FieldLocation, usize, bool)> = self
            .collect_all_fields()
            .iter()
            .filter(|fi| fi.field.field_name() == Some(oldname))
            .map(|fi| {
                (
                    fi.location.clone(),
                    fi.field_range_index,
                    fi.field.ctrl_id == 0,
                )
            })
            .collect();
        if targets.is_empty() {
            return Ok(r#"{"ok":false,"renamed":0}"#.to_string());
        }

        let mut renamed = 0usize;
        let mut touched_sections: Vec<usize> = Vec::new();
        for (location, field_range_index, is_cell) in targets {
            if is_cell {
                let cell = self.cell_at_location_mut(&location)?;
                cell.field_name = if newname.is_empty() {
                    None
                } else {
                    Some(newname.to_string())
                };
            } else {
                let para = self.get_para_mut_at_location(&location)?;
                let control_idx = para
                    .field_ranges
                    .get(field_range_index)
                    .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?
                    .control_idx;
                if let Some(Control::Field(field)) = para.controls.get_mut(control_idx) {
                    field.ctrl_data_name = if newname.is_empty() {
                        None
                    } else {
                        Some(newname.to_string())
                    };
                }
                write_ctrl_data_name(&mut para.ctrl_data_records, control_idx, newname);
            }
            renamed += 1;
            if !touched_sections.contains(&location.section_index) {
                touched_sections.push(location.section_index);
            }
        }

        // raw_stream 무효화: 없으면 저장이 원본 바이트를 재방출해 옛 이름이 되살아난다.
        // 이름은 조판에 영향이 없으므로 재조판은 하지 않는다.
        for section_index in touched_sections {
            if let Some(sec) = self.document.sections.get_mut(section_index) {
                sec.raw_stream = None;
            }
        }

        Ok(format!("{{\"ok\":true,\"renamed\":{}}}", renamed))
    }

    /// 필드 위치가 가리키는 **셀 자체**의 가변 참조. 경로의 마지막 항목이 셀이어야 한다.
    fn cell_at_location_mut(
        &mut self,
        location: &FieldLocation,
    ) -> Result<&mut crate::model::table::Cell, HwpError> {
        let (control_index, cell_index) = match location.nested_path.last() {
            Some(NestedEntry::TableCell {
                control_index,
                cell_index,
                ..
            }) => (*control_index, *cell_index),
            _ => return Err(HwpError::InvalidField("셀 필드가 아닌 위치".into())),
        };
        // 마지막 항목을 뗀 위치가 그 셀을 담은 표가 있는 문단이다.
        let mut host = location.clone();
        host.nested_path.pop();
        let para = self.get_para_mut_at_location(&host)?;
        let ctrl = para
            .controls
            .get_mut(control_index)
            .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
        match ctrl {
            Control::Table(table) => table
                .cells
                .get_mut(cell_index)
                .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into())),
            _ => Err(HwpError::InvalidField(
                "셀을 담은 컨트롤이 표가 아님".into(),
            )),
        }
    }

    /// 셀 필드의 텍스트를 교체한다 (셀의 첫 문단 텍스트를 value로 대체).
    /// 중첩 표를 재귀적으로 탐색하여 임의 깊이를 지원한다.
    fn set_cell_field_text(
        &mut self,
        location: &FieldLocation,
        value: &str,
    ) -> Result<(), HwpError> {
        if location.nested_path.is_empty() {
            return Err(HwpError::InvalidField(
                "셀 필드 위치에 중첩 경로 없음".into(),
            ));
        }
        let sec = self
            .document
            .sections
            .get_mut(location.section_index)
            .ok_or_else(|| HwpError::InvalidField("구역 초과".into()))?;
        let mut para: &mut Paragraph = sec
            .paragraphs
            .get_mut(location.para_index)
            .ok_or_else(|| HwpError::InvalidField("문단 초과".into()))?;

        // 마지막 항목 직전까지 중첩 탐색
        for (i, entry) in location.nested_path[..location.nested_path.len() - 1]
            .iter()
            .enumerate()
        {
            para = match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Table(ref mut table) = ctrl {
                        let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 인덱스 {} 초과",
                                i, cell_index
                            ))
                        })?;
                        cell.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Table이 아님",
                            i, control_index
                        )));
                    }
                }
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Shape(ref mut shape) = ctrl {
                        let drawing = shape.drawing_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: Shape에 DrawingObjAttr 없음",
                                i
                            ))
                        })?;
                        let tb = drawing.text_box.as_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!("경로[{}]: Shape에 TextBox 없음", i))
                        })?;
                        tb.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 글상자 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Shape가 아님",
                            i, control_index
                        )));
                    }
                }
            };
        }

        // 마지막 항목: 셀의 첫 문단 텍스트를 교체
        let last_idx = location.nested_path.len() - 1;
        let last = location.nested_path.last().unwrap();
        match last {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                ..
            } => {
                let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                    HwpError::InvalidField(format!(
                        "경로[{}]: 컨트롤 인덱스 {} 초과",
                        last_idx, control_index
                    ))
                })?;
                if let Control::Table(ref mut table) = ctrl {
                    let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 셀 인덱스 {} 초과",
                            last_idx, cell_index
                        ))
                    })?;
                    if let Some(cell_para) = cell.paragraphs.first_mut() {
                        let old_len = cell_para.text.chars().count();
                        if old_len > 0 {
                            cell_para.delete_text_at(0, old_len);
                        }
                        if !value.is_empty() {
                            cell_para.insert_text_at(0, value);
                        }
                        rebuild_char_offsets(cell_para);
                    }
                    Ok(())
                } else {
                    Err(HwpError::InvalidField(format!(
                        "경로[{}]: controls[{}]가 Table이 아님",
                        last_idx, control_index
                    )))
                }
            }
            _ => Err(HwpError::InvalidField("셀 필드가 아닌 위치".into())),
        }
    }

    /// 필드 위치에서 텍스트를 교체한다.
    ///
    /// delete_text_at + insert_text_at를 사용하여 char_shapes, line_segs,
    /// range_tags, char_count 등 모든 메타데이터를 올바르게 시프트한다.
    /// (직접 para.text 조작 시 메타데이터 불일치로 한컴 "파일 손상" 발생 — #838)
    fn set_field_text_at(
        &mut self,
        location: &FieldLocation,
        field_range_index: usize,
        value: &str,
    ) -> Result<(), HwpError> {
        // raw_stream 무효화: 직렬화 시 수정된 모델을 사용하도록 강제
        if let Some(sec) = self.document.sections.get_mut(location.section_index) {
            sec.raw_stream = None;
        }
        let para = self.get_para_mut_at_location(location)?;
        let fr = para
            .field_ranges
            .get(field_range_index)
            .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?
            .clone();

        let start_idx = fr.start_char_idx;
        let count = fr.end_char_idx.saturating_sub(start_idx);

        // 기존 텍스트 삭제 (char_shapes, line_segs, range_tags 등 자동 시프트)
        if count > 0 {
            para.delete_text_at(start_idx, count);
        }

        // 새 값 삽입
        if !value.is_empty() {
            para.insert_text_at(start_idx, value);
        }

        // field_ranges 갱신: start와 end를 명시적으로 재설정
        let new_end = start_idx + value.chars().count();
        let current_fr = para
            .field_ranges
            .get_mut(field_range_index)
            .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?;
        current_fr.start_char_idx = start_idx;
        current_fr.end_char_idx = new_end;
        let control_idx = current_fr.control_idx;

        // [#3380] 값을 채운 필드는 더 이상 "초기 상태"가 아니다 — properties 비트 15를 세운다.
        //
        // 적재 시 `clear_initial_field_texts` 는 비트 15가 0 인 ClickHere 필드의 텍스트가
        // 안내문과 같으면 "한컴이 남긴 안내문 잔재"로 보고 지운다. 그래서 채운 값이 하필
        // 안내문과 같으면(행정 서식의 "주무관"·"공개"·"해당없음" 등 흔한 실값) 저장·재적재
        // 후 그 칸만 소리 없이 비었다. 쓰기 시점에 상태를 표시해 두면 정규화가 값을 잔재로
        // 오인하지 않는다. 비트 15 는 이 정규화와 Memo 직렬화에서만 쓰여 렌더에 영향이 없다.
        if !value.is_empty() {
            if let Some(Control::Field(field)) = para.controls.get_mut(control_idx) {
                field.properties |= 1 << 15;
            }
        }

        // char_offsets 재생성: FIELD_BEGIN/END 갭, 탭 폭, UTF-16 code unit 크기 반영
        rebuild_char_offsets(para);

        Ok(())
    }

    /// FieldLocation에 해당하는 Paragraph의 가변 참조를 반환한다.
    ///
    /// 중첩 표/글상자를 재귀적으로 탐색하여 임의 깊이를 지원한다.
    fn get_para_mut_at_location(
        &mut self,
        location: &FieldLocation,
    ) -> Result<&mut Paragraph, HwpError> {
        let sec = self
            .document
            .sections
            .get_mut(location.section_index)
            .ok_or_else(|| HwpError::InvalidField("구역 인덱스 초과".into()))?;
        let mut para = sec
            .paragraphs
            .get_mut(location.para_index)
            .ok_or_else(|| HwpError::InvalidField("문단 인덱스 초과".into()))?;

        for (i, entry) in location.nested_path.iter().enumerate() {
            para = match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Table(ref mut table) = ctrl {
                        let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 인덱스 {} 초과",
                                i, cell_index
                            ))
                        })?;
                        cell.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Table이 아님",
                            i, control_index
                        )));
                    }
                }
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Shape(ref mut shape) = ctrl {
                        let drawing = shape.drawing_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: Shape에 DrawingObjAttr 없음",
                                i
                            ))
                        })?;
                        let tb = drawing.text_box.as_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!("경로[{}]: Shape에 TextBox 없음", i))
                        })?;
                        tb.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 글상자 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Shape가 아님",
                            i, control_index
                        )));
                    }
                }
            };
        }

        Ok(para)
    }

    /// 본문 문단의 커서 위치에서 필드를 제거한다 (필드 내용과 컨트롤 삭제).
    ///
    /// 성공 시 `{"ok":true}`, 필드가 없으면 에러를 반환한다.
    pub fn remove_field_at(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get_mut(section_idx)
            .and_then(|s| s.paragraphs.get_mut(para_idx))
            .ok_or_else(|| HwpError::InvalidField("문단 위치 초과".into()))?;
        remove_field_in_para(para, char_offset)?;
        // 필드 제거는 섹션 본문을 바꾸므로 raw_stream 을 무효화해야 저장에 반영된다
        // (삽입 짝 insert_click_here_field_at 과 동형). 누락 시 recompose 로 화면만
        // 갱신되고 저장은 원본 바이트를 재방출해 지운 필드가 되살아난다.
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.recompose_section(section_idx);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 셀/글상자 내 문단의 커서 위치에서 필드를 제거한다.
    pub fn remove_field_at_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> Result<String, HwpError> {
        let para = {
            let host = self
                .document
                .sections
                .get_mut(section_idx)
                .and_then(|s| s.paragraphs.get_mut(parent_para_idx))
                .ok_or_else(|| HwpError::InvalidField("호스트 문단 위치 초과".into()))?;
            let ctrl = host
                .controls
                .get_mut(control_idx)
                .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
            if is_textbox {
                if let Control::Shape(shape) = ctrl {
                    let drawing = shape.drawing_mut().ok_or_else(|| {
                        HwpError::InvalidField("Shape에 DrawingObjAttr 없음".into())
                    })?;
                    let tb = drawing
                        .text_box
                        .as_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 TextBox 없음".into()))?;
                    tb.paragraphs
                        .get_mut(cell_para_idx)
                        .ok_or_else(|| HwpError::InvalidField("글상자 문단 인덱스 초과".into()))?
                } else {
                    return Err(HwpError::InvalidField("예상된 Shape 컨트롤이 아님".into()));
                }
            } else {
                if let Control::Table(table) = ctrl {
                    let cell = table
                        .cells
                        .get_mut(cell_idx)
                        .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into()))?;
                    cell.paragraphs
                        .get_mut(cell_para_idx)
                        .ok_or_else(|| HwpError::InvalidField("셀 문단 인덱스 초과".into()))?
                } else {
                    return Err(HwpError::InvalidField("예상된 Table 컨트롤이 아님".into()));
                }
            }
        };
        remove_field_in_para(para, char_offset)?;
        // 셀/글상자 내 필드 제거도 섹션 본문 스트림을 바꾸므로 raw_stream 무효화 필요
        // (삽입 짝 insert_click_here_field_at_in_cell 과 동형).
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.recompose_section(section_idx);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 커서가 진입한 활성 필드를 설정한다 (안내문 렌더링 스킵용).
    ///
    /// 본문 문단: `set_active_field(sec, para, char_offset)`
    /// 설정 후 해당 페이지의 렌더 트리 캐시를 무효화한다.
    /// 활성 필드를 설정한다. 변경이 발생하면 true를 반환한다.
    pub fn set_active_field(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let ctrl_idx = self.find_field_control_idx(section_idx, para_idx, char_offset, None);
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx,
                control_idx: ci,
                cell_path: None,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }

    /// 셀/글상자 내 활성 필드를 설정한다. 변경이 발생하면 true를 반환한다.
    pub fn set_active_field_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let cell_path = Some(vec![(control_idx, cell_idx, cell_para_idx)]);
        let ctrl_idx = self.find_field_control_idx_in_cell(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            is_textbox,
        );
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx: cell_para_idx,
                control_idx: ci,
                cell_path,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }

    /// 활성 필드를 해제한다.
    pub fn clear_active_field(&mut self) {
        if self.active_field.is_some() {
            self.active_field = None;
            self.invalidate_page_tree_cache();
        }
    }

    /// 본문 문단의 커서 위치에서 필드 범위 정보를 조회한다.
    ///
    /// 커서가 필드 범위 내에 있으면 필드 정보를 JSON으로 반환하고,
    /// 필드 밖이면 `{"inField":false}`를 반환한다.
    pub fn get_field_info_at(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let para = match self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(para_idx))
        {
            Some(p) => p,
            None => return r#"{"inField":false}"#.to_string(),
        };
        field_info_at_in_para(para, char_offset)
    }

    /// 셀/글상자 내 문단의 커서 위치에서 필드 범위 정보를 조회한다.
    pub fn get_field_info_at_in_cell(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> String {
        let para = (|| {
            let host = self
                .document
                .sections
                .get(section_idx)?
                .paragraphs
                .get(parent_para_idx)?;
            let ctrl = host.controls.get(control_idx)?;
            if is_textbox {
                if let Control::Shape(shape) = ctrl {
                    let tb = shape.drawing()?.text_box.as_ref()?;
                    return tb.paragraphs.get(cell_para_idx);
                }
            } else {
                if let Control::Table(table) = ctrl {
                    let cell = table.cells.get(cell_idx)?;
                    return cell.paragraphs.get(cell_para_idx);
                }
            }
            None
        })();
        match para {
            Some(p) => field_info_at_in_para(p, char_offset),
            None => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀의 필드 범위 정보를 조회한다.
    pub fn get_field_info_at_by_path(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> String {
        match self.resolve_paragraph_by_path(section_idx, parent_para_idx, path) {
            Ok(para) => field_info_at_in_para(para, char_offset),
            Err(_) => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
    pub fn set_active_field_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let para = match self.resolve_paragraph_by_path(section_idx, parent_para_idx, path) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let ctrl_idx = find_field_ctrl_idx_in_para(para, char_offset);
        if let Some(ci) = ctrl_idx {
            let last = path.last().unwrap();
            let cell_para_idx = last.2;
            // cell_path: 전체 path를 저장 (중첩 표 구분용)
            let cell_path = Some(path.to_vec());
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx: cell_para_idx,
                control_idx: ci,
                cell_path,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }
}

/// 문단 내 커서 위치의 필드 범위 정보를 JSON으로 반환한다.
fn field_info_at_in_para(para: &Paragraph, char_offset: usize) -> String {
    for fr in &para.field_ranges {
        if fr.start_char_idx != fr.end_char_idx || char_offset != fr.start_char_idx {
            continue;
        }
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            let guide = field.guide_text().unwrap_or("");
            return format!(
                "{{\"inField\":true,\"fieldId\":{},\"fieldType\":\"{}\",\"startCharIdx\":{},\"endCharIdx\":{},\"isGuide\":true,\"guideName\":{},\"editableInForm\":{}}}",
                field.field_id,
                field.field_type_str(),
                fr.start_char_idx,
                fr.end_char_idx,
                json_escape(guide),
                field.is_editable_in_form(),
            );
        }
    }

    for fr in &para.field_ranges {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            // 커서가 필드 범위 내에 있는지 확인 (start 이상, end 이하)
            // end가 exclusive이므로 커서가 end 위치에 있으면 필드 "끝"에 있는 것
            if char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx {
                let is_guide = fr.start_char_idx == fr.end_char_idx;
                let guide = field.guide_text().unwrap_or("");
                return format!(
                    "{{\"inField\":true,\"fieldId\":{},\"fieldType\":\"{}\",\"startCharIdx\":{},\"endCharIdx\":{},\"isGuide\":{},\"guideName\":{},\"editableInForm\":{}}}",
                    field.field_id,
                    field.field_type_str(),
                    fr.start_char_idx,
                    fr.end_char_idx,
                    is_guide,
                    json_escape(guide),
                    field.is_editable_in_form(),
                );
            }
        }
    }
    r#"{"inField":false}"#.to_string()
}

/// 문단에서 필드를 수집한다 (재귀: 표 셀, 글상자 내부 포함).
fn collect_fields_from_paragraph(
    para: &Paragraph,
    base_location: &FieldLocation,
    list_id: u32,
    walk: &mut ListWalk,
    result: &mut Vec<FieldInfo>,
) {
    let para_in_list = match base_location.nested_path.last() {
        Some(NestedEntry::TableCell { para_index, .. }) => *para_index,
        Some(NestedEntry::TextBox { para_index, .. }) => *para_index,
        None => base_location.para_index,
    };

    // 현재 문단의 field_ranges에서 필드 수집
    for (fri, fr) in para.field_ranges.iter().enumerate() {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            let value = if fr.start_char_idx < fr.end_char_idx {
                let chars: Vec<char> = para.text.chars().collect();
                if fr.end_char_idx <= chars.len() {
                    chars[fr.start_char_idx..fr.end_char_idx].iter().collect()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            result.push(FieldInfo {
                field: field.clone(),
                location: base_location.clone(),
                value,
                field_range_index: fri,
                list_id,
                para_in_list,
                start_pos: field_content_start(para, fri),
                // 끝은 시작 + **내용 길이**다. 빈 누름틀의 내용은 한글이 채워 넣는 안내문이다.
                end_pos: field_content_start(para, fri)
                    + if fr.start_char_idx == fr.end_char_idx {
                        guide_units(para, fr.control_idx)
                    } else {
                        fr.end_char_idx - fr.start_char_idx
                    },
            });
        }
    }

    // 컨트롤 내부 재귀 탐색 (표 셀, 글상자)
    for (ci, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            Control::Table(table) => {
                for (cell_i, cell) in table.cells.iter().enumerate() {
                    // 셀에 먼저 번호를 주고 **그 다음** 셀 안으로 내려간다(깊이 우선).
                    let cell_list_id = walk.alloc();
                    walk.lists.push(ListEntry {
                        list_id: cell_list_id,
                        is_cell: true,
                        host_list_id: list_id,
                        section_index: base_location.section_index,
                        host_para_index: para_in_list,
                        control_index: ci,
                        cell_index: cell_i,
                        para_count: cell.paragraphs.len(),
                        grid: Some(CellGrid {
                            row: cell.row,
                            col: cell.col,
                            row_span: cell.row_span,
                            col_span: cell.col_span,
                            cell_count: table.cells.len(),
                        }),
                    });
                    // 셀 자체의 field_name이 있으면 가상 필드로 추가
                    if let Some(ref fname) = cell.field_name {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TableCell {
                            control_index: ci,
                            cell_index: cell_i,
                            para_index: 0,
                        });
                        // 셀의 첫 문단 텍스트를 값으로 사용
                        let value = cell
                            .paragraphs
                            .first()
                            .map(|p| p.text.clone())
                            .unwrap_or_default();
                        result.push(FieldInfo {
                            field: Field {
                                ctrl_id: 0,
                                field_id: (ci as u32) << 16 | cell_i as u32,
                                field_type: FieldType::ClickHere,
                                command: String::new(),
                                properties: if cell.editable_in_form() { 1 } else { 0 },
                                extra_properties: 0,
                                ctrl_data_name: Some(fname.clone()),
                                instance_id: None,
                                memo_index: 0,
                                memo_paragraphs: Vec::new(),
                                memo_text_direction: None,
                                raw_parameters_xml: None,
                                parameters: Default::default(),
                                guide_residue: None,
                            },
                            location: loc,
                            value,
                            field_range_index: 0,
                            // 셀 필드는 셀 리스트 그 자체를 가리킨다 — 커서는 첫 문단 0 위치다.
                            list_id: cell_list_id,
                            para_in_list: 0,
                            start_pos: 0,
                            end_pos: 0,
                        });
                    }
                    for (pi, cell_para) in cell.paragraphs.iter().enumerate() {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TableCell {
                            control_index: ci,
                            cell_index: cell_i,
                            para_index: pi,
                        });
                        collect_fields_from_paragraph(cell_para, &loc, cell_list_id, walk, result);
                    }
                }
            }
            Control::Shape(shape) => {
                // 글상자도 리스트 하나를 차지한다(한글2022 실측: 표 12칸 다음 글상자가
                // 그 자리에서 아이디를 받고, 그 뒤 표가 이어받는다). **캡션도 마찬가지다** —
                // 안 세면 그 뒤 리스트 번호가 전부 밀린다.
                for (node, paragraphs) in shape_lists(shape) {
                    let box_list_id = walk.alloc();
                    walk.lists.push(ListEntry {
                        list_id: box_list_id,
                        is_cell: false,
                        host_list_id: list_id,
                        section_index: base_location.section_index,
                        host_para_index: para_in_list,
                        control_index: ci,
                        // 묶음 안에서 몇 번째 마디인지 — 사슬 쪽과 같은 번호를 쓴다.
                        cell_index: node,
                        para_count: paragraphs.len(),
                        grid: None,
                    });
                    for (pi, tb_para) in paragraphs.iter().enumerate() {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TextBox {
                            control_index: ci,
                            para_index: pi,
                        });
                        collect_fields_from_paragraph(tb_para, &loc, box_list_id, walk, result);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 리스트까지 내려가는 셀 경로 `(control, cell, para)` 를 리스트 표에서 되짚는다.
///
/// 마지막 칸의 문단 번호만 호출 측이 정한다(그 리스트 안에서 캐럿이 있는 문단).
pub(crate) fn cell_path_to_list(
    lists: &[ListEntry],
    list_id: u32,
    para_in_list: usize,
) -> Option<Vec<(usize, usize, usize)>> {
    let mut path = Vec::new();
    let mut current = lists.iter().find(|l| l.list_id == list_id)?;
    let mut para_index = para_in_list;
    loop {
        // 셀만 풀던 자리다. 글상자·캡션도 같은 꼴의 경로를 쓴다 — 둘째 칸이 셀 번호가 아니라
        // 그 개체가 연 리스트 중 몇 번째냐일 뿐이고, 푸는 쪽(`cell_paragraph_at_path`)이
        // 컨트롤 갈래를 보고 가른다. 여기서 막고 있어서 캡션 리스트는 끝이 0 으로 나왔다.
        path.push((current.control_index, current.cell_index, para_index));
        if current.host_list_id == ROOT_LIST_ID {
            break;
        }
        para_index = current.host_para_index;
        current = lists.iter().find(|l| l.list_id == current.host_list_id)?;
    }
    path.reverse();
    Some(path)
}

/// 리스트가 딛고 선 **본문 문단** 번호.
pub(crate) fn root_para_of(lists: &[ListEntry], entry: &ListEntry) -> usize {
    let mut current = entry;
    while current.host_list_id != ROOT_LIST_ID {
        match lists.iter().find(|l| l.list_id == current.host_list_id) {
            Some(host) => current = host,
            None => break,
        }
    }
    current.host_para_index
}

/// 커서 좌표(리스트 아이디 + 그 안의 문단 번호)가 가리키는 문단.
///
/// 리스트 표를 이미 만들어 둔 쪽에서 쓴다 — 표를 다시 만드는 비용을 부르는 쪽이 정한다.
pub(crate) fn cursor_paragraph<'a>(
    core: &'a DocumentCore,
    lists: &[ListEntry],
    list_id: u32,
    para_in_list: usize,
) -> Option<&'a Paragraph> {
    if list_id == ROOT_LIST_ID {
        let (si, pi) = root_para_location(core, para_in_list)?;
        return core.document.sections.get(si)?.paragraphs.get(pi);
    }
    let entry = lists.iter().find(|l| l.list_id == list_id)?;
    let path = cell_path_to_list(lists, list_id, para_in_list)?;
    core.cell_paragraph_at_path(entry.section_index, root_para_of(lists, entry), &path)
}

/// 본문 문단 번호를 `(구역, 그 구역 안 문단)` 으로 푼다.
///
/// **한글의 본문 리스트는 구역을 가로질러 하나로 이어진다.** rhwp 는 구역별로 나눠 들고 있어서
/// 첫 구역만 보면 다구역 문서에서 문단 번호가 통째로 어긋난다(실측: `2026_oss_rst.hwp` 는
/// 두 구역이고 한글은 본문 문단을 0‥14 로 세는데 첫 구역만 보면 0‥7 이다).
pub(crate) fn root_para_location(
    core: &DocumentCore,
    para_in_list: usize,
) -> Option<(usize, usize)> {
    let mut rest = para_in_list;
    for (si, sec) in core.document.sections.iter().enumerate() {
        if rest < sec.paragraphs.len() {
            return Some((si, rest));
        }
        rest -= sec.paragraphs.len();
    }
    None
}

/// 본문이 담은 문단 수 — 구역을 모두 합친다.
pub(crate) fn root_para_count(core: &DocumentCore) -> usize {
    core.document
        .sections
        .iter()
        .map(|s| s.paragraphs.len())
        .sum()
}

/// 코드 유닛 위치를 글자 번호로 되돌린다 — `code_unit_pos` 의 짝.
pub(crate) fn char_idx_at_code_unit(para: &Paragraph, pos: usize) -> usize {
    match para
        .char_offsets
        .iter()
        .position(|offset| *offset as usize >= pos)
    {
        Some(idx) => idx,
        None => para.char_offsets.len(),
    }
}

/// 문단 앞머리의 **자리차지(비 인라인) 컨트롤**이 차지하는 코드 유닛 수.
///
/// 한글의 "문서의 시작"은 이 자리다 — 자리차지 개체의 컨트롤 문자는 건너뛰고, 처음 만나는
/// 인라인 컨트롤이나 글자 앞에서 멈춘다. 컨트롤 하나는 8 코드 유닛이다.
pub(crate) fn leading_anchor_pos(para: &Paragraph) -> usize {
    let mut skipped = 0usize;
    let mut anchored_count = 0usize;
    for ctrl in &para.controls {
        let anchored = match ctrl {
            Control::Table(t) => !t.common.treat_as_char,
            Control::Shape(s) => !s.common().treat_as_char,
            // 누름틀은 **건너뛰면 안 된다** — 그 시작 자리가 캐럿이 서는 곳이다.
            Control::Field(_) => false,
            // 자동 번호는 글자 사이에 놓이는 **인라인**이다. 문단 앞머리에 오더라도 캐럿을
            // 밀지 않는다(실측: 쪽 번호를 셋 끼운 문단에서 `SetPos(…, 3)` 이 3 그대로다).
            Control::AutoNumber(_) | Control::NewNumber(_) => false,
            // 구역·단 정의 같은 표식은 자리를 차지하되 캐럿이 설 곳이 아니다.
            _ => true,
        };
        if !anchored {
            break;
        }
        skipped += 8;
        anchored_count += 1;
    }
    // 컨트롤 개수 × 8 은 **개체들이 스트림에서 맞붙어 있을 때만** 맞는다. 자리차지 개체가
    // 여럿인 문단은 개체 사이에 **공백 글자가 한 칸씩** 있어(한글 실측: 앵커가 16·25·34 로
    // 8 이 아니라 9 씩 벌어진다) 개수 셈이 그만큼 앞질러 간다. `mix-shape-01` 에서 이 함수는
    // 40 을 줬고 한글은 24 였다 — 캐럿이 문단 끝으로 밀려 `SelectCtrlFront` 가 아무 개체도
    // 못 골랐다.
    //
    // 스트림에 글자가 있으면 그 **첫 글자 자리**가 답이다. 개수 셈이 그보다 앞서 나갈 때만
    // 갈아끼운다 — 맞붙어 있는 문단에서는 두 값이 같아 아무것도 안 바뀐다.
    //
    // **0 은 안 믿는다.** 앞머리에 컨트롤이 있는데 첫 글자가 스트림 0 이라는 것은 대응표가
    // 컨트롤을 안 담았다는 뜻이다 — 구역 나누기로 **새로 생긴 문단**이 그렇다(`char_offsets`
    // 가 0 부터 다시 매겨진다). 그 문단에서는 개수 셈이 맞는 답이라 그대로 둔다.
    if anchored_count > 0 {
        if let Some(first_char) = para.char_offsets.first() {
            let first_char = *first_char as usize;
            if first_char > 0 && first_char < skipped {
                skipped = first_char;
            }
        }
    }
    skipped.min((para.char_count as usize).saturating_sub(1))
}

/// 확장 컨트롤 하나가 스트림에서 차지하는 코드 유닛 수.
pub(crate) const EXTENDED_CTRL_UNITS: usize = 8;

/// **블록**이 시작할 수 있는 첫 자리. 캐럿이 설 수 있는 자리([`leading_anchor_pos`])와 다르다.
///
/// 캐럿은 앞머리 자리차지 개체를 전부 건너뛰지만 블록은 그 개체들을 **담을 수 있어야** 한다 —
/// 표를 통째로 잡는 것이 그런 경우다. 한글이 건너뛰는 것은 구역·단 정의처럼 자리만 차지하는
/// 표식뿐이다(본문 첫 문단 `SelectAll` 실측: 캐럿 시작은 72, 블록 시작은 16 = 표식 둘).
pub(crate) fn select_start_pos(para: &Paragraph) -> usize {
    para.controls
        .iter()
        .take_while(|ctrl| matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)))
        .count()
        * EXTENDED_CTRL_UNITS
}

/// rhwp 의 글자 번호를 **한글이 보는 스트림 위치**(코드 유닛)로 옮긴다.
///
/// `char_offsets` 를 쓰지 않는다. 그 배열은 적재 때 안내문을 지우면서 추정으로 다시 쓰여
/// 문단마다 다르게 세고(계획서 §4.6.1), 무엇보다 **rhwp 모델과 한글 스트림이 다른 문서**가
/// 됐다는 사실을 담지 못한다.
///
/// 원본 바이트로 확인한 세 규칙(영수증 서식 6개 필드 전수 일치):
///
/// 1. 글자 하나는 1칸.
/// 2. 확장 컨트롤(표·개체·누름틀 시작/끝)은 8칸. 누름틀 하나가 시작·끝 **두 개**를 낸다.
/// 3. **빈 누름틀은 안내문 글자를 스트림에 담는다.** rhwp 는 적재 때 그것을 지우지만
///    한글은 파일을 열며 다시 채운다 — 그래서 그 뒤 위치가 안내문 길이만큼 밀린다.
pub(crate) fn stream_pos(para: &Paragraph, char_idx: usize) -> usize {
    let control_positions = para.control_text_positions();
    let mut units = char_idx + tab_padding(para, char_idx);

    // 필드가 아닌 확장 컨트롤 — 자기 자리에서 8칸.
    //
    // (컨트롤의 **앵커 자리**를 구할 때는 이 셈을 쓰면 안 된다. 문단마다 자리표 글자를
    // 남기는 것과 안 남기는 것이 섞여 있어서 여기서 갈라 주지 못한다 — 앵커는
    // `controls_json` 이 컨트롤을 차례로 걸으며 따로 센다.)
    for (ci, ctrl) in para.controls.iter().enumerate() {
        if matches!(ctrl, Control::Field(_)) {
            continue;
        }
        if control_positions
            .get(ci)
            .is_some_and(|pos| *pos <= char_idx)
        {
            units += ctrl_stream_units(ctrl);
        }
    }

    // 누름틀 — 시작 코드, 끝 코드, 그리고 빈 필드면 안내문 글자.
    for fr in &para.field_ranges {
        if fr.start_char_idx <= char_idx {
            units += EXTENDED_CTRL_UNITS;
        }
        if fr.end_char_idx <= char_idx {
            units += EXTENDED_CTRL_UNITS;
            if fr.start_char_idx == fr.end_char_idx {
                units += guide_units(para, fr.control_idx);
            }
        }
    }
    units
}

/// 그리기 개체 하나가 여는 **글 리스트들** — 글상자와 **캡션**이다.
///
/// 첫 값은 그 개체 안에서 몇 번째 리스트인지다. 리스트 표(`cell_index` 자리)와 컨트롤 사슬이
/// 이 번호를 같이 써서 둘이 **같은 마디를 같은 이름으로** 가리킨다.
///
/// 캡션을 안 세던 것이 리스트 번호가 밀리던 뿌리였다(실측: `samples/draw-group.hwp` 의 묶음은
/// 글상자가 없는데도 한글이 list 2 를 주고, 그 부모가 바로 그 묶음이며, 안에 `atno` 가 든
/// 가운데 정렬 한 문단이다 — 캡션이다).
///
/// 글상자와 캡션을 **둘 다** 가진 개체의 앞뒤 순서는 아직 못 쟀다. 그런 표본을 만나면 그때
/// 실측해서 고칠 것 — 지금 순서는 기존에 검증된 글상자 자리를 지키려고 글상자를 앞에 둔 것뿐이다.
pub(crate) fn shape_lists(shape: &crate::model::shape::ShapeObject) -> Vec<(usize, &[Paragraph])> {
    let mut out: Vec<(usize, &[Paragraph])> = Vec::new();
    if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
        out.push((0, tb.paragraphs.as_slice()));
    }
    let caption = match shape {
        crate::model::shape::ShapeObject::Group(g) => g.caption.as_ref(),
        other => other.drawing().and_then(|d| d.caption.as_ref()),
    };
    if let Some(cap) = caption {
        if !cap.paragraphs.is_empty() {
            out.push((1, cap.paragraphs.as_slice()));
        }
    }
    out
}

/// 컨트롤 하나가 **글자 몫을 빼고** 스트림에서 더 차지하는 칸 수.
///
/// 확장 컨트롤은 8칸인데, 그중 자동 번호 계열은 파서가 **자리표 글자 한 칸**을 텍스트에 남긴다
/// (`parse_para_text` 의 `0x0012` 가지가 공백을 넣는다). 그 한 칸은 이미 글자로 세어졌으므로
/// 여기서 8을 더하면 아홉이 된다 — 오라클은 여덟이다(실측: `InsertPageNum` 뒤 문단 끝 7 → 15).
fn ctrl_stream_units(ctrl: &Control) -> usize {
    match ctrl {
        Control::AutoNumber(_) | Control::NewNumber(_) => EXTENDED_CTRL_UNITS - 1,
        _ => EXTENDED_CTRL_UNITS,
    }
}

/// 탭이 스트림에서 더 차지하는 칸 수.
///
/// 파서는 탭(`0x0009`)을 **글자 하나** `'\t'` 로 담는데 한글 스트림에서 탭은 **8칸**이다
/// (`parse_para_text` 의 탭 가지가 `pos += 16` 으로 넘어간다 — 16바이트 = 8 코드 유닛).
/// 그래서 탭이 든 문단은 그 뒤 자리가 탭 하나당 7칸씩 앞당겨져 보였다. 누름틀 좌표가
/// 어긋나는 결함이라, 오라클 실측(`InsertTab` 뒤 캐럿 3 → 11)으로 확인하고 여기서 메운다.
fn tab_padding(para: &Paragraph, char_idx: usize) -> usize {
    para.text
        .chars()
        .take(char_idx)
        .filter(|c| *c == '\t')
        .count()
        * (EXTENDED_CTRL_UNITS - 1)
}

/// 빈 누름틀이 스트림에 담는 안내문 길이(글자 수). 안내문이 없으면 0.
fn guide_units(para: &Paragraph, control_idx: usize) -> usize {
    match para.controls.get(control_idx) {
        Some(Control::Field(field)) => field.guide_text().map(|g| g.chars().count()).unwrap_or(0),
        _ => 0,
    }
}

/// 문단이 스트림에서 차지하는 코드 유닛 수 — 끝 문단 부호는 빼고 센다.
///
/// 앞머리 자리차지 자리를 **바닥으로 깐다.** 컨트롤 중에는 문단 텍스트에 자리표 글자를 남기지
/// 않는 것이 있어서([`Paragraph::control_text_positions`] 가 자리를 못 주는 것들 — 구역·단
/// 정의 같은 표식) 텍스트만 세면 문단이 실제보다 짧게 나온다. 본문 첫 문단이 그런 꼴인데,
/// 그러면 시작(72)이 끝(24)보다 커지는 있을 수 없는 상태가 된다. 한글은 그 문단의 시작도 끝도
/// 72 로 답한다(`MoveDocBegin`·`MoveSelParaEnd` 실측).
pub(crate) fn stream_len(para: &Paragraph) -> usize {
    stream_pos(para, para.text.chars().count()).max(leading_anchor_pos(para))
}

/// 누름틀 **내용이 시작하는** 스트림 위치.
///
/// [`stream_pos`] 를 그대로 쓰면 안 된다 — 그 함수는 "그 자리의 글자"를 기준으로 세기 때문에
/// 빈 누름틀에서는 **자기 FIELD_END 와 자기 안내문까지** 앞선 것으로 친다. 내용의 시작은
/// 자기 FIELD_BEGIN **바로 뒤**다.
pub(crate) fn field_content_start(para: &Paragraph, field_range_index: usize) -> usize {
    let Some(own) = para.field_ranges.get(field_range_index) else {
        return 0;
    };
    let control_positions = para.control_text_positions();
    let mut units = own.start_char_idx + tab_padding(para, own.start_char_idx);

    for (ci, ctrl) in para.controls.iter().enumerate() {
        if matches!(ctrl, Control::Field(_)) {
            continue;
        }
        if control_positions
            .get(ci)
            .is_some_and(|pos| *pos <= own.start_char_idx)
        {
            units += ctrl_stream_units(ctrl);
        }
    }

    for (i, fr) in para.field_ranges.iter().enumerate() {
        if i == field_range_index {
            continue; // 자기 것은 아래에서 시작 코드만 더한다
        }
        if fr.start_char_idx <= own.start_char_idx {
            units += EXTENDED_CTRL_UNITS;
        }
        if fr.end_char_idx <= own.start_char_idx {
            units += EXTENDED_CTRL_UNITS;
            if fr.start_char_idx == fr.end_char_idx {
                units += guide_units(para, fr.control_idx);
            }
        }
    }

    units + EXTENDED_CTRL_UNITS // 자기 FIELD_BEGIN
}

/// 캐럿이 설 수 있는 자리들 — 한 글자 이동(`MoveNextChar` 류)이 딛는 눈금.
///
/// 한글 실측(영수증 서식 list 9): `0 · 8 · 22 · 23 · 24 · 25 · 33 · 47 · 48 · 49`.
/// 규칙은 세 가지다.
///
/// - 글자마다 한 자리.
/// - 누름틀은 **시작 코드 앞**과 **내용 시작**에 자리가 있다. 끝 코드 앞에는 없다 —
///   빈 누름틀에서 한 칸 가면 안내문과 끝 코드를 **한꺼번에 건너뛴다**(8 → 22).
/// - 문단 끝에 한 자리.
pub(crate) fn caret_stops(para: &Paragraph) -> Vec<usize> {
    let mut stops = Vec::new();
    for (fri, _) in para.field_ranges.iter().enumerate() {
        let content = field_content_start(para, fri);
        stops.push(content.saturating_sub(EXTENDED_CTRL_UNITS)); // 시작 코드 앞
        stops.push(content);
    }
    let len = para.text.chars().count();
    for i in 0..len {
        stops.push(stream_pos(para, i));
    }
    stops.push(stream_len(para));
    stops.sort_unstable();
    stops.dedup();
    stops
}

/// 단어가 시작하는 자리들 — `MoveNextWord`·`MovePrevWord` 가 딛는 눈금.
///
/// 한글2022 실측(두 셀 · 두 방향 자기정합). 단어는 **공백으로 나뉜 덩어리**이고, 누름틀은
/// 그 자체가 경계를 만든다.
///
/// | 자리 | 왜 |
/// |---|---|
/// | 문단 시작·끝 | 언제나 |
/// | 앞이 공백인 글자 | 평문 규칙 — 문장부호는 단어를 안 가른다(`!@` 는 한 덩어리) |
/// | 누름틀 시작 코드 자리 | 필드가 시작하면 새 단어 |
/// | 누름틀 내용 시작 | 안내문/내용이 새 단어 |
///
/// 끝 코드(FIELD_END)는 경계가 **아니다** — 필드 바로 뒤 글자는 앞 단어에 붙는다
/// (list 9 에서 22 번 '부' 가 단어 시작이 아니다).
pub(crate) fn word_starts(para: &Paragraph) -> Vec<usize> {
    let mut starts = vec![leading_anchor_pos(para)];

    for (fri, _) in para.field_ranges.iter().enumerate() {
        let content = field_content_start(para, fri);
        starts.push(content.saturating_sub(EXTENDED_CTRL_UNITS));
        starts.push(content);
    }

    // 공백 다음 글자가 단어 시작이다. 단 **스트림에서 바로 이웃**이어야 한다 — 사이에 누름틀이
    // 끼면(공백 … 필드 … 글자) 그 글자는 앞 단어에 붙는다(list 9 의 47 번 '까').
    let chars: Vec<char> = para.text.chars().collect();
    for i in 1..chars.len() {
        if chars[i - 1].is_whitespace()
            && !chars[i].is_whitespace()
            && stream_pos(para, i) == stream_pos(para, i - 1) + 1
        {
            starts.push(stream_pos(para, i));
        }
    }

    starts.push(stream_len(para));
    starts.sort_unstable();
    starts.dedup();
    starts
}

/// 지금 단어의 끝 — `MoveWordEnd` 가 가는 자리.
///
/// **다음 공백 글자의 자리**다(실측: `가나 다라마` 에서 4 → 6, 1 → 2). 마지막 단어면 문단 끝.
pub(crate) fn word_end_from(para: &Paragraph, pos: usize) -> usize {
    let chars: Vec<char> = para.text.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let at = stream_pos(para, i);
        if at >= pos && ch.is_whitespace() {
            return at;
        }
    }
    stream_len(para)
}

/// [`stream_pos`] 의 역 — 한글이 준 스트림 위치를 rhwp 의 글자 번호로 되돌린다.
///
/// 문단은 짧아서 훑어도 된다. 대응표를 따로 두면 편집 때마다 둘이 어긋난다.
pub(crate) fn char_idx_at_stream_pos(para: &Paragraph, pos: usize) -> usize {
    let len = para.text.chars().count();
    (0..=len)
        .find(|&i| stream_pos(para, i) >= pos)
        .unwrap_or(len)
}

/// 글자 번호를 문단 안의 **코드 유닛 위치**로 옮긴다 (옛 경로).
///
/// 한글의 `pos` 는 글자 수가 아니라 코드 유닛이다 — 확장 컨트롤(표·누름틀 시작/끝)이 8칸을
/// 차지한다. `char_offsets` 가 그 대응표다.
#[allow(dead_code)]
fn code_unit_pos(para: &Paragraph, char_idx: usize, control_idx: usize) -> usize {
    if let Some(offset) = para.char_offsets.get(char_idx) {
        return *offset as usize;
    }
    if para.char_offsets.is_empty() {
        // 글자가 없는 문단(개체만 늘어선 본문 문단 등)에는 대응표가 없다. 앞선 컨트롤을
        // 세어 자리를 짚는다 — 확장 컨트롤은 8칸, 필드는 시작·끝 두 개라 16칸이다.
        let before: usize = para
            .controls
            .iter()
            .take(control_idx)
            .map(|c| {
                if matches!(c, Control::Field(_)) {
                    16
                } else {
                    8
                }
            })
            .sum();
        return before + 8; // 자기 FIELD_BEGIN 뒤가 텍스트 자리다
    }
    // 글자 끝을 넘어선 자리 — 문단 길이(끝 문단 부호 제외)로 잡는다.
    para.char_offsets
        .last()
        .map(|last| *last as usize + 1)
        .unwrap_or_else(|| (para.char_count as usize).saturating_sub(1))
}

/// 문단 CTRL_DATA 레코드의 필드 이름 부분을 새 이름으로 다시 쓴다.
///
/// 레이아웃은 파서 대칭이다(`parser::body_text::parse_ctrl_data_field_name`):
/// 헤더 10바이트 + 이름 길이(u16 LE, 글자 수) + UTF-16LE 이름.
/// 인코더가 둘이 되면 한쪽만 고쳐져 조용히 어긋나므로 여기 하나만 둔다.
pub(crate) fn write_ctrl_data_name(
    records: &mut Vec<Option<Vec<u8>>>,
    control_idx: usize,
    new_name: &str,
) {
    while records.len() <= control_idx {
        records.push(None);
    }
    let name_chars: Vec<u16> = new_name.encode_utf16().collect();
    let mut data = match &records[control_idx] {
        // 원본이 있으면 헤더(10바이트)를 보존한다 — 한컴이 남긴 paramset 값이 들어 있다.
        Some(existing) if existing.len() >= 12 => existing[..10].to_vec(),
        Some(_) => return, // 12바이트 미만은 이름 자리가 없는 레코드다. 건드리지 않는다.
        None => vec![0x1Bu8, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00],
    };
    data.extend_from_slice(&(name_chars.len() as u16).to_le_bytes());
    for ch in &name_chars {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    records[control_idx] = Some(data);
}

/// FieldLocation을 JSON으로 변환
fn field_location_json(loc: &FieldLocation) -> String {
    if loc.nested_path.is_empty() {
        format!(
            "{{\"sectionIndex\":{},\"paraIndex\":{}}}",
            loc.section_index, loc.para_index,
        )
    } else {
        let path_entries: Vec<String> = loc.nested_path.iter().map(|e| match e {
            NestedEntry::TableCell { control_index, cell_index, para_index } => {
                format!("{{\"type\":\"cell\",\"controlIndex\":{},\"cellIndex\":{},\"paraIndex\":{}}}",
                    control_index, cell_index, para_index)
            }
            NestedEntry::TextBox { control_index, para_index } => {
                format!("{{\"type\":\"textbox\",\"controlIndex\":{},\"paraIndex\":{}}}",
                    control_index, para_index)
            }
        }).collect();
        format!(
            "{{\"sectionIndex\":{},\"paraIndex\":{},\"path\":[{}]}}",
            loc.section_index,
            loc.para_index,
            path_entries.join(","),
        )
    }
}

fn para_at_location<'a>(core: &'a DocumentCore, location: &FieldLocation) -> Option<&'a Paragraph> {
    let mut para = core
        .document
        .sections
        .get(location.section_index)?
        .paragraphs
        .get(location.para_index)?;

    for entry in &location.nested_path {
        para = match entry {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                para_index,
            } => {
                let ctrl = para.controls.get(*control_index)?;
                if let Control::Table(table) = ctrl {
                    table.cells.get(*cell_index)?.paragraphs.get(*para_index)?
                } else {
                    return None;
                }
            }
            NestedEntry::TextBox {
                control_index,
                para_index,
            } => {
                let ctrl = para.controls.get(*control_index)?;
                if let Control::Shape(shape) = ctrl {
                    shape
                        .drawing()?
                        .text_box
                        .as_ref()?
                        .paragraphs
                        .get(*para_index)?
                } else {
                    return None;
                }
            }
        };
    }

    Some(para)
}

fn field_range_bounds(core: &DocumentCore, fi: &FieldInfo) -> Option<(usize, usize)> {
    let para = para_at_location(core, &fi.location)?;
    let range = para.field_ranges.get(fi.field_range_index)?;
    Some((range.start_char_idx, range.end_char_idx))
}

impl DocumentCore {
    /// 본문 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        _cell_path: Option<(usize, usize, usize)>,
    ) -> Option<usize> {
        let para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(para_idx)?;
        find_field_ctrl_idx_in_para(para, char_offset)
    }

    /// 셀/글상자 내 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx_in_cell(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> Option<usize> {
        let host = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        let ctrl = host.controls.get(control_idx)?;
        let para = if is_textbox {
            if let Control::Shape(shape) = ctrl {
                let tb = shape.drawing()?.text_box.as_ref()?;
                tb.paragraphs.get(cell_para_idx)?
            } else {
                return None;
            }
        } else {
            if let Control::Table(table) = ctrl {
                table.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)?
            } else {
                return None;
            }
        };
        find_field_ctrl_idx_in_para(para, char_offset)
    }

    fn next_click_here_field_id(&self) -> u32 {
        let mut max_id = 0u32;
        for section in &self.document.sections {
            for para in &section.paragraphs {
                collect_max_field_id(para, &mut max_id);
            }
        }
        max_id.saturating_add(1).max(1)
    }
}

fn collect_max_field_id(para: &Paragraph, max_id: &mut u32) {
    for ctrl in &para.controls {
        match ctrl {
            Control::Field(field) if field.field_id > *max_id => {
                *max_id = field.field_id;
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    for cell_para in &cell.paragraphs {
                        collect_max_field_id(cell_para, max_id);
                    }
                }
                if let Some(caption) = &table.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_field_id(cap_para, max_id);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(text_box) = &drawing.text_box {
                        for tb_para in &text_box.paragraphs {
                            collect_max_field_id(tb_para, max_id);
                        }
                    }
                }
            }
            Control::Picture(pic) => {
                if let Some(caption) = &pic.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_field_id(cap_para, max_id);
                    }
                }
            }
            _ => {}
        }
    }
}

fn insert_click_here_field_in_para(
    para: &mut Paragraph,
    char_offset: usize,
    field_id: u32,
    guide: &str,
    memo: &str,
    name: &str,
    editable: bool,
) -> Result<usize, HwpError> {
    let text_len = para.text.chars().count();
    let start = char_offset.min(text_len);
    let positions = para.control_text_positions();
    let insert_idx = positions
        .iter()
        .position(|&pos| pos > start)
        .unwrap_or(para.controls.len());

    for range in &mut para.field_ranges {
        if range.control_idx >= insert_idx {
            range.control_idx += 1;
        }
    }

    let field = Field {
        field_type: FieldType::ClickHere,
        // [#1434] 이름은 ctrl_data_name(CTRL_DATA 0x57)으로 별도 저장하므로 command 에
        // 넣지 않는다 (Name 키가 끼면 한컴이 안내문 바인딩 실패).
        command: Field::build_clickhere_command(guide, memo),
        properties: if editable { 1 } else { 0 },
        extra_properties: 0x09,
        field_id,
        ctrl_id: tags::FIELD_CLICKHERE,
        instance_id: None,
        ctrl_data_name: if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        },
        memo_index: 0,
        memo_paragraphs: Vec::new(),
        memo_text_direction: None,
        raw_parameters_xml: None,
        parameters: Default::default(),
        guide_residue: None,
    };

    para.controls.insert(insert_idx, Control::Field(field));
    if para.ctrl_data_records.len() < insert_idx {
        para.ctrl_data_records.resize(insert_idx, None);
    }
    para.ctrl_data_records.insert(insert_idx, None);

    let new_range = FieldRange {
        start_char_idx: start,
        end_char_idx: start,
        control_idx: insert_idx,
        ..Default::default()
    };
    let range_idx = para
        .field_ranges
        .iter()
        .position(|range| {
            range.start_char_idx > start
                || (range.start_char_idx == start && range.control_idx > insert_idx)
        })
        .unwrap_or(para.field_ranges.len());
    para.field_ranges.insert(range_idx, new_range);
    rebuild_char_offsets(para);

    Ok(start)
}

/// 문단에서 커서 위치의 ClickHere 필드 컨트롤 인덱스를 반환한다.
fn find_field_ctrl_idx_in_para(para: &Paragraph, char_offset: usize) -> Option<usize> {
    // 인접 누름틀 경계에서는 앞 누름틀의 끝과 다음 빈 누름틀의 시작이
    // 같은 charOffset을 공유한다. 새 빈 누름틀을 먼저 잡아야 첫 입력이
    // 앞 누름틀 값으로 붙지 않는다.
    for fr in &para.field_ranges {
        if fr.start_char_idx == fr.end_char_idx && char_offset == fr.start_char_idx {
            if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
                if field.field_type == FieldType::ClickHere {
                    return Some(fr.control_idx);
                }
            }
        }
    }

    for fr in &para.field_ranges {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            if char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx {
                return Some(fr.control_idx);
            }
        }
    }
    None
}

/// 문단 내 커서 위치의 누름틀 필드를 제거한다.
fn remove_field_in_para(para: &mut Paragraph, char_offset: usize) -> Result<(), HwpError> {
    let idx = para.field_ranges.iter().position(|fr| {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                return false;
            }
            char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx
        } else {
            false
        }
    });
    match idx {
        Some(i) => {
            let start = para.field_ranges[i].start_char_idx;
            let end = para.field_ranges[i].end_char_idx;
            let removed_control_idx = para.field_ranges[i].control_idx;
            para.field_ranges.remove(i);
            if end > start {
                para.delete_text_at(start, end - start);
            }
            if removed_control_idx < para.controls.len() {
                para.controls.remove(removed_control_idx);
            }
            if removed_control_idx < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(removed_control_idx);
            }
            for range in &mut para.field_ranges {
                if range.control_idx > removed_control_idx {
                    range.control_idx -= 1;
                }
            }
            rebuild_char_offsets(para);
            Ok(())
        }
        None => Err(HwpError::InvalidField(
            "커서 위치에 누름틀 필드 없음".into(),
        )),
    }
}

/// 문자열을 JSON 이스케이프한다.
/// 문단의 char_offsets를 컨트롤/필드/텍스트 배치 순서에 맞게 재생성한다.
///
/// 원본 char_offsets에서 컨트롤 배치 패턴을 보존하면서,
/// 텍스트 길이 변경(필드 값 삽입)에 맞게 오프셋을 재계산한다.
pub(crate) fn rebuild_char_offsets(para: &mut Paragraph) {
    // [#4149] 호출부는 방금 text/controls 수술을 마친 상태다 (필드 제거·필드값
    // 기입·클립보드 트림 등, 셀 문단 포함) — 단일줄 과밀 memo 무효화의 수렴점.
    para.invalidate_single_line_overflow_memo();
    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();

    // 원본 char_offsets에서 첫 문자 이전 컨트롤 수 추정
    // (원본 gap / 8 = 컨트롤 수)
    let ctrls_before_text = if !para.char_offsets.is_empty() {
        para.char_offsets[0] as usize / 8
    } else {
        para.controls.len()
    }
    .min(para.controls.len());

    // FIELD_BEGIN: 이미 char_offsets의 첫 갭에 포함된 선행 컨트롤은 보존하고,
    // 새로 삽입된 시작 위치 필드는 첫 문자 앞에도 갭을 추가해야 한다.
    let mut field_begin_at: Vec<usize> = vec![0; text_len + 1];
    for fr in &para.field_ranges {
        if fr.control_idx >= ctrls_before_text {
            let idx = fr.start_char_idx.min(text_len);
            field_begin_at[idx] += 1;
        }
    }

    // FIELD_END 수: field_ranges에서 end가 텍스트 범위 내인 것
    let mut field_end_at: Vec<usize> = vec![0; text_len + 1];
    for fr in &para.field_ranges {
        let idx = fr.end_char_idx.min(text_len);
        field_end_at[idx] += 1;
    }

    if text_len == 0 {
        para.char_offsets = Vec::new();
        para.char_count =
            ((ctrls_before_text + field_begin_at[0] + field_end_at[0]) * 8 + 1) as u32;
        return;
    }

    let mut offset: u32 = ctrls_before_text as u32 * 8;
    let mut new_offsets = Vec::with_capacity(text_len);

    for (i, ch) in text_chars.iter().enumerate() {
        // 이 문자 앞에 FIELD_BEGIN 컨트롤 갭 삽입
        offset += field_begin_at[i] as u32 * 8;
        // 이 문자 앞에 FIELD_END 마커 갭 삽입
        offset += field_end_at[i] as u32 * 8;

        new_offsets.push(offset);

        let char_size = match *ch {
            '\t' => 8,
            '\n' | '\u{00A0}' => 1,
            c => {
                let mut buf = [0u16; 2];
                c.encode_utf16(&mut buf).len() as u32
            }
        };
        offset += char_size;
    }

    // 텍스트 뒤에 위치한 빈 필드/필드 끝 마커와 문단 끝 마커를 char_count에 반영한다.
    offset += field_begin_at[text_len] as u32 * 8;
    offset += field_end_at[text_len] as u32 * 8;
    para.char_count = offset + 1;
    para.char_offsets = new_offsets;
}

pub(crate) fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::{Control, Field, FieldType};
    use crate::model::document::Section;
    use crate::model::paragraph::{FieldRange, Paragraph};
    use crate::model::table::{Cell, Table};

    fn make_field_control(ctrl_id: u32) -> Control {
        Control::Field(Field {
            field_type: FieldType::ClickHere,
            command: String::new(),
            properties: 0,
            extra_properties: 0,
            field_id: ctrl_id,
            ctrl_id,
            instance_id: None,
            ctrl_data_name: None,
            memo_index: 0,
            memo_paragraphs: Vec::new(),
            memo_text_direction: None,
            raw_parameters_xml: None,
            parameters: Default::default(),
            guide_residue: None,
        })
    }

    fn para_with_click_here_field() -> Paragraph {
        // 스트림: [ColumnDef 8B] A B C [FIELD_BEGIN] X Y [FIELD_END], 필드는 [3,5]
        Paragraph {
            text: "ABCXY".into(),
            controls: vec![
                Control::ColumnDef(Default::default()),
                make_field_control(100),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 3,
                end_char_idx: 5,
                control_idx: 1,
                ..Default::default()
            }],
            char_count: 21,
            char_offsets: vec![8, 9, 10, 19, 20],
            ..Default::default()
        }
    }

    #[test]
    fn tab_occupies_eight_stream_units() {
        // 한글 스트림에서 탭은 8칸인데 파서는 글자 하나로 담는다. 그 차이를 `stream_pos` 가
        // 메우지 않으면 탭 뒤의 모든 자리가 탭 하나당 7칸씩 앞당겨진다 — 누름틀 좌표가
        // 어긋나는 결함이다(오라클 실측: `InsertTab` 뒤 캐럿 3 → 11).
        let para = Paragraph {
            text: "AB\tCD".into(),
            char_count: 13, // 글자 4 + 탭 8 + 문단끝 1
            ..Default::default()
        };
        assert_eq!(stream_pos(&para, 0), 0);
        assert_eq!(stream_pos(&para, 2), 2, "탭 앞은 그대로다");
        assert_eq!(stream_pos(&para, 3), 10, "탭 하나를 지나면 8칸");
        assert_eq!(stream_pos(&para, 5), 12);
        assert_eq!(stream_len(&para), 12);
        // 되짚기도 같은 자를 써야 한다.
        assert_eq!(char_idx_at_stream_pos(&para, 10), 3);
        assert_eq!(char_idx_at_stream_pos(&para, 12), 5);
    }

    #[test]
    fn remove_field_at_invalidates_raw_stream() {
        // 본문 필드 제거는 섹션 본문을 바꾸므로 raw_stream 이 무효화돼야 저장에 반영된다.
        // 무효화 라인을 제거하면 이 테스트가 실패한다(RED): 저장이 원본 바이트를 재방출해
        // 지운 필드가 되살아난다.
        let mut core = DocumentCore::new_empty();
        core.document.sections.push(Section {
            paragraphs: vec![para_with_click_here_field()],
            raw_stream: Some(vec![0xAB; 64]),
            ..Default::default()
        });
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];

        core.remove_field_at(0, 0, 4).unwrap();

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "remove_field_at 후 raw_stream 이 무효화돼야 한다"
        );
        let bytes = crate::serializer::body_text::serialize_section(&core.document.sections[0]);
        assert_ne!(
            bytes,
            vec![0xAB; 64],
            "serialize_section 이 여전히 원본 바이트를 반환"
        );
        // 필드 컨트롤이 실제로 제거돼 ColumnDef 만 남는다
        assert_eq!(core.document.sections[0].paragraphs[0].controls.len(), 1);
    }

    #[test]
    fn remove_field_at_in_cell_invalidates_raw_stream() {
        // 표 셀 내 필드 제거도 섹션 본문 스트림을 바꾸므로 raw_stream 무효화가 필요하다.
        let table = Table {
            cells: vec![Cell {
                paragraphs: vec![para_with_click_here_field()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let parent_para = Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        };
        let mut core = DocumentCore::new_empty();
        core.document.sections.push(Section {
            paragraphs: vec![parent_para],
            raw_stream: Some(vec![0xAB; 64]),
            ..Default::default()
        });
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];

        core.remove_field_at_in_cell(0, 0, 0, 0, 0, 4, false)
            .unwrap();

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "remove_field_at_in_cell 후 raw_stream 이 무효화돼야 한다"
        );
        let bytes = crate::serializer::body_text::serialize_section(&core.document.sections[0]);
        assert_ne!(bytes, vec![0xAB; 64]);
    }

    /// 이름이 `name` 인 셀 하나를 담은 표 문단.
    fn para_with_named_cells(names: &[&str]) -> Paragraph {
        let table = Table {
            cells: names
                .iter()
                .map(|name| Cell {
                    field_name: Some((*name).to_string()),
                    paragraphs: vec![Paragraph::default()],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        }
    }

    fn core_with(paragraphs: Vec<Paragraph>) -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.document.sections.push(Section {
            paragraphs,
            raw_stream: Some(vec![0xAB; 64]),
            ..Default::default()
        });
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];
        core
    }

    fn field_names(core: &DocumentCore) -> Vec<String> {
        core.collect_all_fields()
            .iter()
            .filter_map(|fi| fi.field.field_name().map(str::to_string))
            .collect()
    }

    /// 셀 하나짜리 표를 담은 문단. 셀 안에 다른 컨트롤을 넣을 수 있다.
    fn para_with_cells(cells: Vec<Cell>) -> Paragraph {
        Paragraph {
            controls: vec![Control::Table(Box::new(Table {
                cells,
                ..Default::default()
            }))],
            ..Default::default()
        }
    }

    fn named_cell(name: &str, paragraphs: Vec<Paragraph>) -> Cell {
        Cell {
            field_name: Some(name.to_string()),
            paragraphs,
            ..Default::default()
        }
    }

    #[test]
    fn list_ids_run_depth_first_from_two() {
        // 한글2022 실측 규칙: 셀에 번호를 준 **뒤** 그 셀 안으로 내려간다. 바깥 셀을 모두
        // 세고 나서 안으로 들어가면 중첩 표에서 번호가 어긋난다(3중 중첩 문서로 확인).
        let inner = para_with_cells(vec![
            named_cell("inner0", vec![Paragraph::default()]),
            named_cell("inner1", vec![Paragraph::default()]),
        ]);
        let outer = para_with_cells(vec![
            named_cell("outer0", vec![Paragraph::default()]),
            named_cell("outer1", vec![inner]),
            named_cell("outer2", vec![Paragraph::default()]),
        ]);
        let core = core_with(vec![outer]);

        let (fields, lists) = core.collect_fields_and_lists();
        let ids: Vec<(String, u32)> = fields
            .iter()
            .map(|fi| (fi.field.field_name().unwrap_or("").to_string(), fi.list_id))
            .collect();

        assert_eq!(
            ids,
            vec![
                ("outer0".to_string(), 2),
                ("outer1".to_string(), 3),
                ("inner0".to_string(), 4),
                ("inner1".to_string(), 5),
                ("outer2".to_string(), 6),
            ],
            "셀 3 다음에 그 안의 표가 오고, 그 뒤에 셀 6 이 이어져야 한다"
        );
        assert_eq!(lists.len(), 5);
        // 안쪽 리스트는 바깥 셀(리스트 3)에 매달린다 — 상위 이동이 이 값을 딛는다.
        assert_eq!(lists[2].host_list_id, 3);
        assert_eq!(lists[0].host_list_id, ROOT_LIST_ID);
    }

    #[test]
    fn cursor_model_top_pos_skips_anchored_controls() {
        // "문서의 시작"은 자리차지 개체를 건너뛴 자리다. 인라인 표를 만나면 그 앞에서 멈춘다.
        let mut anchored = para_with_cells(vec![named_cell("a", vec![Paragraph::default()])]);
        anchored.char_count = 9;
        let mut core = core_with(vec![anchored]);
        let model = core.get_cursor_model_json();
        assert!(
            model.contains("\"topPos\":8"),
            "자리차지 표 하나를 건너뛰어 8이어야 한다: {model}"
        );

        if let Control::Table(t) = &mut core.document.sections[0].paragraphs[0].controls[0] {
            t.common.treat_as_char = true;
        }
        let model = core.get_cursor_model_json();
        assert!(
            model.contains("\"topPos\":0"),
            "인라인 표 앞에서 멈춰 0이어야 한다: {model}"
        );
    }

    #[test]
    fn para_bounds_never_start_past_end() {
        // 자리표 글자를 안 남기는 컨트롤이 있으면 텍스트만 세는 길이가 앞머리 자리차지보다
        // 짧아져 시작 > 끝이라는 있을 수 없는 상태가 된다. 한글은 그런 문단의 시작도 끝도
        // 같은 자리로 답한다.
        let mut anchored = para_with_cells(vec![named_cell("a", vec![Paragraph::default()])]);
        anchored.char_count = 9;
        let core = core_with(vec![anchored]);

        let bounds = core.para_bounds_json(0, 0);

        assert_eq!(
            bounds, r#"{"start":8,"end":8,"selectStart":0}"#,
            "시작이 끝을 넘으면 안 된다: {bounds}"
        );
    }

    #[test]
    fn rename_field_renames_cell_field() {
        // 셀 필드는 누름틀과 저장 경로가 다르다. `updateClickHereProps` 로는 못 바꾼 자리다.
        let mut core = core_with(vec![para_with_named_cells(&["pt_nm"])]);

        let json = core.rename_field_by_name("pt_nm", "환자명").unwrap();

        assert_eq!(json, r#"{"ok":true,"renamed":1}"#);
        assert_eq!(field_names(&core), vec!["환자명".to_string()]);
        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "raw_stream 이 남으면 저장이 옛 이름을 재방출한다"
        );
    }

    #[test]
    fn rename_field_renames_every_occurrence() {
        // 오라클 실측: 같은 이름이 두 번 나오는 문서에서 한 번의 호출로 둘 다 바뀐다.
        let mut core = core_with(vec![
            para_with_named_cells(&["pt_no", "other"]),
            para_with_named_cells(&["pt_no"]),
        ]);

        let json = core.rename_field_by_name("pt_no", "접수번호").unwrap();

        assert_eq!(json, r#"{"ok":true,"renamed":2}"#);
        assert_eq!(
            field_names(&core),
            vec![
                "접수번호".to_string(),
                "other".to_string(),
                "접수번호".to_string()
            ]
        );
    }

    #[test]
    fn rename_field_rewrites_click_here_ctrl_data() {
        // 누름틀 이름은 CTRL_DATA 바이트에도 실려 있다. 모델만 바꾸면 저장에서 되돌아간다.
        let mut para = para_with_click_here_field();
        if let Control::Field(f) = &mut para.controls[1] {
            f.ctrl_data_name = Some("old_name".into());
        }
        let mut core = core_with(vec![para]);

        let json = core.rename_field_by_name("old_name", "새이름").unwrap();

        assert_eq!(json, r#"{"ok":true,"renamed":1}"#);
        assert_eq!(field_names(&core), vec!["새이름".to_string()]);
        let record = core.document.sections[0].paragraphs[0].ctrl_data_records[1]
            .as_ref()
            .expect("CTRL_DATA 레코드가 만들어져야 한다");
        let name_len = u16::from_le_bytes([record[10], record[11]]) as usize;
        let chars: Vec<u16> = record[12..12 + name_len * 2]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(String::from_utf16_lossy(&chars), "새이름");
    }

    #[test]
    fn rename_field_missing_name_changes_nothing() {
        // 없는 이름이면 오라클도 아무 일도 하지 않는다(FieldExist 가 계속 false).
        let mut core = core_with(vec![para_with_named_cells(&["pt_nm"])]);

        let json = core.rename_field_by_name("없는필드", "아무개").unwrap();

        assert_eq!(json, r#"{"ok":false,"renamed":0}"#);
        assert_eq!(field_names(&core), vec!["pt_nm".to_string()]);
        assert!(
            core.document.sections[0].raw_stream.is_some(),
            "바꾼 게 없으면 원본 스트림을 버리지 않는다"
        );
    }

    #[test]
    fn rebuild_preserves_mid_text_field_begin_gap() {
        // Stream: [ColumnDef 8B] A(1) B(1) C(1) [FIELD_BEGIN 8B] X(1) Y(1) [FIELD_END 8B]
        let mut para = Paragraph {
            text: "ABCXY".into(),
            controls: vec![
                Control::ColumnDef(Default::default()),
                make_field_control(100),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 3,
                end_char_idx: 5,
                control_idx: 1,
                ..Default::default()
            }],
            char_offsets: vec![8, 9, 10, 19, 20],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        // A=8(+1) B=9(+1) C=10(+1) → gap 8 for FIELD_BEGIN → X=19(+1) Y=20
        assert_eq!(para.char_offsets, vec![8, 9, 10, 19, 20]);
    }

    #[test]
    fn rebuild_field_at_start_no_double_count() {
        // FIELD_BEGIN is pre-text control (control_idx=0 < ctrls_before_text=1)
        let mut para = Paragraph {
            text: "XY".into(),
            controls: vec![make_field_control(100)],
            field_ranges: vec![FieldRange {
                start_char_idx: 0,
                end_char_idx: 2,
                control_idx: 0,
                ..Default::default()
            }],
            char_offsets: vec![8, 9],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        assert_eq!(para.char_offsets, vec![8, 9]);
    }

    #[test]
    fn rebuild_after_set_field_creates_serializable_gap() {
        // After set_field: "라벨: " [FIELD_BEGIN] "NEW" [FIELD_END]
        let mut para = Paragraph {
            text: "라벨: NEW".into(), // 7 chars: 라 벨 : ' ' N E W
            controls: vec![
                Control::ColumnDef(Default::default()),
                make_field_control(200),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 4,
                end_char_idx: 7,
                control_idx: 1,
                ..Default::default()
            }],
            // 원본 offsets (stale after text change, but char_offsets[0] still valid for ctrls_before_text)
            char_offsets: vec![8, 9, 10, 11, 20, 21, 22],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        // ctrls_before_text = 8/8 = 1
        // 라=8(+1) 벨=9(+1) :=10(+1) ' '=11(+1) → field_begin gap +8 → N=20(+1) E=21(+1) W=22
        assert_eq!(para.char_offsets[0], 8); // 라
        assert_eq!(para.char_offsets[3], 11); // ' '
        assert_eq!(para.char_offsets[4], 20); // N — 8-byte gap after ' ' for FIELD_BEGIN
        let gap = para.char_offsets[4] as i64 - (para.char_offsets[3] as i64 + 1);
        assert_eq!(gap, 8); // serializer needs exactly 8 code units for FIELD_BEGIN
    }

    #[test]
    fn set_cell_field_text_updates_text_metadata() {
        let cell_para = Paragraph {
            text: "기존값".into(),
            char_count: 4,
            char_offsets: vec![0, 1, 2],
            ..Default::default()
        };
        let table = Table {
            cells: vec![Cell {
                field_name: Some("셀필드".into()),
                paragraphs: vec![cell_para],
                ..Default::default()
            }],
            ..Default::default()
        };
        let parent_para = Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        };

        let mut core = DocumentCore::new_empty();
        core.document.sections.push(Section {
            paragraphs: vec![parent_para],
            ..Default::default()
        });

        let location = FieldLocation {
            section_index: 0,
            para_index: 0,
            nested_path: vec![NestedEntry::TableCell {
                control_index: 0,
                cell_index: 0,
                para_index: 0,
            }],
        };

        core.set_cell_field_text(&location, "새값").unwrap();

        let Control::Table(table) = &core.document.sections[0].paragraphs[0].controls[0] else {
            panic!("expected table control");
        };
        let updated = &table.cells[0].paragraphs[0];
        assert_eq!(updated.text, "새값");
        assert_eq!(updated.char_count, 3);
        assert_eq!(updated.char_offsets, vec![0, 1]);
    }
}
