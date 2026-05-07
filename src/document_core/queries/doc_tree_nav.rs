//! 문서 트리 DFS 기반 캐럿 네비게이션
//!
//! HWP 문서의 계층 구조(Section → Paragraph → Control → 내부 Paragraph → ...)를
//! DFS로 순회하여 다음/이전 편집 가능 위치를 찾는다.

use crate::document_core::DocumentCore;
use crate::document_core::helpers::{find_control_text_positions, get_textbox_from_shape, navigable_text_len};
use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::paragraph::Paragraph;
use crate::model::shape::ShapeObject;

/// 글상자 오버플로우 연결 정보 (소스 → 타겟)
#[derive(Debug, Clone)]
pub struct OverflowLink {
    /// 소스 글상자가 속한 문단 인덱스 (섹션 내)
    pub source_parent_para: usize,
    /// 소스 컨트롤 인덱스
    pub source_ctrl_idx: usize,
    /// 소스에서 오버플로우 시작 문단 인덱스
    pub overflow_start: usize,
    /// 타겟 글상자가 속한 문단 인덱스
    pub target_parent_para: usize,
    /// 타겟 컨트롤 인덱스
    pub target_ctrl_idx: usize,
}

/// 문서 트리 내 현재 위치를 나타내는 컨텍스트 스택의 한 단계
#[derive(Debug, Clone)]
pub struct NavContextEntry {
    /// 이 컨트롤이 속한 문단 인덱스 (부모 컨텍스트 기준)
    pub parent_para: usize,
    /// 부모 문단 내 컨트롤 인덱스
    pub ctrl_idx: usize,
    /// 부모 문단 텍스트에서 이 컨트롤의 charOffset 위치
    pub ctrl_text_pos: usize,
    /// Table: 셀 인덱스 / TextBox: 0
    pub cell_idx: usize,
    /// true=Shape+TextBox, false=Table
    pub is_textbox: bool,
}

/// DFS 순회 결과
#[derive(Debug)]
pub enum NavResult {
    /// 편집 가능한 텍스트 위치
    Text {
        sec: usize,
        para: usize,
        char_offset: usize,
        context: Vec<NavContextEntry>,
    },
    /// 문서 경계 (이동 불가)
    Boundary,
}

/// 컨트롤이 편집 가능(네비게이션 가능)한지 판별.
/// Some(true)=TextBox, Some(false)=Table/인라인 개체/각주 마커, None=건너뜀
///
/// 텍스트가 전혀 없는 빈 글상자(장식용 프레임 등)는 건너뛴다.
/// CharOverlap(글자겹침)은 1글자 단위로 건너뛴다 (표와 동일 취급).
fn classify_navigable(ctrl: &Control) -> Option<bool> {
    match ctrl {
        Control::Shape(s) => {
            if let Some(tb) = get_textbox_from_shape(s.as_ref()) {
                // TextBox가 있고 텍스트가 비어있지 않으면 글상자 진입
                if !tb.paragraphs.iter().all(|p| p.text.is_empty()) {
                    return Some(true);
                }
            }
            // TextBox 없거나 비어있는 도형 → 표처럼 1칸 건너뛰기
            Some(false)
        }
        Control::Table(_) => Some(false),
        Control::Picture(_) => Some(false),
        Control::Equation(_) => Some(false),
        Control::Footnote(_) => Some(false),
        Control::Endnote(_) => Some(false),
        // CharOverlap은 layout에서 char_count=1로 처리되므로
        // 별도의 건너뛰기 없이 일반 문자처럼 1칸 이동
        Control::CharOverlap(_) => None,
        _ => None,
    }
}

/// context 스택을 따라 현재 컨테이너의 paragraphs를 반환한다.
/// 오버플로우 타겟 글상자의 경우 소스의 오버플로우 문단 슬라이스를 반환한다.
fn resolve_paragraphs<'a>(
    sections: &'a [Section],
    sec: usize,
    context: &[NavContextEntry],
    overflow_links: &[OverflowLink],
) -> Option<&'a [Paragraph]> {
    let section_paras: &[Paragraph] = &sections.get(sec)?.paragraphs;
    let mut paragraphs: &[Paragraph] = section_paras;

    for (depth, entry) in context.iter().enumerate() {
        let para = paragraphs.get(entry.parent_para)?;
        let ctrl = para.controls.get(entry.ctrl_idx)?;
        if entry.is_textbox {
            if let Control::Shape(s) = ctrl {
                let tb = get_textbox_from_shape(s.as_ref())?;

                // 마지막 컨텍스트 entry가 빈 글상자이면 오버플로우 타겟인지 확인
                if depth == context.len() - 1
                    && tb.paragraphs.iter().all(|p| p.text.is_empty())
                {
                    if let Some(link) = overflow_links.iter().find(|l|
                        l.target_parent_para == entry.parent_para
                        && l.target_ctrl_idx == entry.ctrl_idx
                    ) {
                        // 소스 글상자의 오버플로우 문단 반환
                        let src_para = section_paras.get(link.source_parent_para)?;
                        let src_ctrl = src_para.controls.get(link.source_ctrl_idx)?;
                        if let Control::Shape(src_s) = src_ctrl {
                            let src_tb = get_textbox_from_shape(src_s.as_ref())?;
                            if link.overflow_start < src_tb.paragraphs.len() {
                                return Some(&src_tb.paragraphs[link.overflow_start..]);
                            }
                        }
                    }
                }

                paragraphs = &tb.paragraphs;
            } else if let Control::Picture(pic) = ctrl {
                // Picture 캡션 내부 텍스트 편집
                let cap = pic.caption.as_ref()?;
                paragraphs = &cap.paragraphs;
            } else {
                return None;
            }
        } else {
            if let Control::Table(t) = ctrl {
                let cell = t.cells.get(entry.cell_idx)?;
                paragraphs = &cell.paragraphs;
            } else {
                return None;
            }
        }
    }

    Some(paragraphs)
}

/// Table의 reading order로 다음 셀 인덱스를 반환한다.
/// 행 우선 순서 (row 0 col 0, row 0 col 1, ..., row 1 col 0, ...)
fn next_cell_index(table: &crate::model::table::Table, current_cell_idx: usize, forward: bool) -> Option<usize> {
    if table.cells.is_empty() {
        return None;
    }
    if forward {
        let next = current_cell_idx + 1;
        if next < table.cells.len() { Some(next) } else { None }
    } else {
        if current_cell_idx > 0 { Some(current_cell_idx - 1) } else { None }
    }
}

impl DocumentCore {
    /// 외부에서 전달받은 context의 ctrl_text_pos를 실제 문서 데이터로 복원한다.
    /// TypeScript 측에서 ctrl_text_pos를 모르는 경우 0으로 전달하므로,
    /// Rust가 find_control_text_positions()로 올바른 값을 채운다.
    pub(crate) fn fix_context_text_positions(
        sections: &[Section],
        sec: usize,
        context: &[NavContextEntry],
    ) -> Vec<NavContextEntry> {
        if context.is_empty() {
            return Vec::new();
        }
        let mut fixed = Vec::with_capacity(context.len());
        let mut paragraphs: &[Paragraph] = match sections.get(sec) {
            Some(s) => &s.paragraphs,
            None => return context.to_vec(),
        };

        for entry in context {
            let ctrl_text_pos = if let Some(para) = paragraphs.get(entry.parent_para) {
                let positions = find_control_text_positions(para);
                positions.get(entry.ctrl_idx).copied().unwrap_or(entry.ctrl_text_pos)
            } else {
                entry.ctrl_text_pos
            };

            fixed.push(NavContextEntry {
                parent_para: entry.parent_para,
                ctrl_idx: entry.ctrl_idx,
                ctrl_text_pos,
                cell_idx: entry.cell_idx,
                is_textbox: entry.is_textbox,
            });

            // 다음 depth를 위해 현재 컨트롤 내부 paragraphs로 이동
            if let Some(para) = paragraphs.get(entry.parent_para) {
                if let Some(ctrl) = para.controls.get(entry.ctrl_idx) {
                    if entry.is_textbox {
                        if let Control::Shape(s) = ctrl {
                            if let Some(tb) = get_textbox_from_shape(s.as_ref()) {
                                paragraphs = &tb.paragraphs;
                            }
                        }
                    } else {
                        if let Control::Table(t) = ctrl {
                            if let Some(cell) = t.cells.get(entry.cell_idx) {
                                paragraphs = &cell.paragraphs;
                            }
                        }
                    }
                }
            }
        }

        fixed
    }

    /// 문서 트리 DFS 기반 다음/이전 편집 가능 위치를 반환한다.
    ///
    /// - `sec`: 현재 섹션 인덱스
    /// - `para`: 현재 컨텍스트 내 문단 인덱스
    /// - `char_offset`: 현재 문자 오프셋
    /// - `delta`: +1(forward) 또는 -1(backward)
    /// - `context`: 현재 컨텍스트 스택 (빈 배열 = body)
    /// - `max_para`: 현재 컨테이너에서 렌더링된 마지막 문단 인덱스 (None이면 제한 없음)
    /// - `overflow_links`: 글상자 오버플로우 연결 정보
    pub(crate) fn navigate_next_editable(
        &self,
        sec: usize,
        para: usize,
        char_offset: usize,
        delta: i32,
        context: &[NavContextEntry],
        max_para: Option<usize>,
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let sections = &self.document.sections;
        let forward = delta > 0;

        // 현재 컨테이너의 paragraphs 해석
        let paragraphs = match resolve_paragraphs(sections, sec, context, overflow_links) {
            Some(p) => p,
            None => {
                return NavResult::Boundary;
            }
        };

        // Step 1: 현재 문단 내 탐색
        if let Some(current_para) = paragraphs.get(para) {
            let text_len = navigable_text_len(current_para);
            let ctrl_positions = find_control_text_positions(current_para);

            if forward {
                // Forward: char_offset 이후의 컨트롤 또는 텍스트 끝 확인
                let next_offset = char_offset + 1;

                // char_offset 위치 또는 그 이후에 있는 편집 가능 컨트롤 탐색
                for (ci, ctrl) in current_para.controls.iter().enumerate() {
                    let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                    if cpos < next_offset {
                        continue; // 이미 지나친 컨트롤
                    }
                    if cpos == char_offset {
                        // 현재 위치에 컨트롤이 있음
                        if let Some(is_tb) = classify_navigable(ctrl) {
                            if is_tb {
                                // 글상자: 진입
                                return self.enter_control_forward(
                                    sec, para, ci, cpos, is_tb, context, overflow_links,
                                );
                            }
                            // 도형/표/그림/수식: 컨트롤 다음 위치로 이동 (글자처럼 1칸)
                            let next = cpos + 1;
                            if next <= text_len {
                                return NavResult::Text {
                                    sec, para, char_offset: next,
                                    context: context.to_vec(),
                                };
                            }
                            // 문단 끝 넘어감 → Step 2로
                            break;
                        }
                        // 편집 불가 컨트롤 → 건너뜀 (다음 offset으로 이동)
                        // 계속 다음 컨트롤 탐색
                    }
                    if cpos > char_offset && cpos < next_offset.min(text_len) {
                        // char_offset과 next_offset 사이에 컨트롤이 있음
                        if let Some(is_tb) = classify_navigable(ctrl) {
                            if is_tb {
                                return self.enter_control_forward(
                                    sec, para, ci, cpos, is_tb, context, overflow_links,
                                );
                            }
                            // 표: 건너뛰기
                            let skip = cpos + 1;
                            if skip <= text_len {
                                return NavResult::Text {
                                    sec, para, char_offset: skip,
                                    context: context.to_vec(),
                                };
                            }
                            break;
                        }
                    }
                }

                // 다음 offset에 컨트롤이 있는지도 확인
                for (ci, ctrl) in current_para.controls.iter().enumerate() {
                    let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                    if cpos == next_offset {
                        if let Some(is_tb) = classify_navigable(ctrl) {
                            if is_tb {
                                return self.enter_control_forward(
                                    sec, para, ci, cpos, is_tb, context, overflow_links,
                                );
                            }
                            // 도형/표: next_offset을 반환 (getCursorRect에서 적절히 처리)
                            return NavResult::Text {
                                sec, para, char_offset: next_offset,
                                context: context.to_vec(),
                            };
                        }
                    }
                }

                // 텍스트가 남아있으면 다음 charOffset 반환
                if next_offset <= text_len {
                    return NavResult::Text {
                        sec,
                        para,
                        char_offset: next_offset,
                        context: context.to_vec(),
                    };
                }

                // 문단 끝 도달 → Step 2
            } else {
                // Backward: char_offset 이전의 컨트롤 또는 텍스트 시작 확인
                if char_offset == 0 {
                    // 문단 시작 도달 → Step 2 (이전 문단/컨테이너)
                } else {
                    let prev_offset = char_offset - 1;

                    // prev_offset 위치에 컨트롤이 있는지 역순 탐색
                    for (ci, ctrl) in current_para.controls.iter().enumerate().rev() {
                        let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                        if cpos == char_offset {
                            // 현재 위치에 컨트롤 → 건너뛰기 (이미 이 위치에 있으므로)
                            if let Some(is_tb) = classify_navigable(ctrl) {
                                if is_tb {
                                    return self.enter_control_backward(
                                        sec, para, ci, cpos, is_tb, context, overflow_links,
                                    );
                                }
                                // 도형/표/그림: 컨트롤 앞으로 건너뛰기
                                if cpos > 0 {
                                    return NavResult::Text {
                                        sec, para, char_offset: cpos - 1,
                                        context: context.to_vec(),
                                    };
                                }
                                break;
                            }
                        }
                        if cpos == prev_offset {
                            // 이전 위치에 컨트롤 → 이 위치에서 멈춤
                            if let Some(is_tb) = classify_navigable(ctrl) {
                                if is_tb {
                                    return self.enter_control_backward(
                                        sec, para, ci, cpos, is_tb, context, overflow_links,
                                    );
                                }
                                return NavResult::Text {
                                    sec, para, char_offset: cpos,
                                    context: context.to_vec(),
                                };
                            }
                        }
                    }

                    // prev_offset 위치 또는 그 이전 컨트롤 탐색 (사이에 있는 경우)
                    for (ci, ctrl) in current_para.controls.iter().enumerate().rev() {
                        let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                        if cpos > prev_offset && cpos < char_offset {
                            if let Some(is_tb) = classify_navigable(ctrl) {
                                if is_tb {
                                    return self.enter_control_backward(
                                        sec, para, ci, cpos, is_tb, context, overflow_links,
                                    );
                                }
                                // 표: 건너뛰기
                                if cpos > 0 {
                                    return NavResult::Text {
                                        sec, para, char_offset: cpos - 1,
                                        context: context.to_vec(),
                                    };
                                }
                                break;
                            }
                        }
                    }

                    // 텍스트 이동
                    return NavResult::Text {
                        sec,
                        para,
                        char_offset: prev_offset,
                        context: context.to_vec(),
                    };
                }
            }
        }

        // Step 2: 같은 컨테이너의 다음/이전 문단
        // max_para가 설정된 경우, 렌더링된 범위를 넘어서는 문단으로 이동하지 않는다.
        let para_count = paragraphs.len();
        // 렌더 범위 상한: max_para가 있으면 그 이후 문단은 진입하지 않음
        let effective_para_limit = max_para.map_or(para_count, |mp| (mp + 1).min(para_count));

        if forward {
            if para + 1 < effective_para_limit {
                return self.navigate_to_para_start(sec, para + 1, context, overflow_links);
            }

            // 렌더 범위 끝 도달 → 오버플로우 링크 확인
            // 현재 소스 글상자의 오버플로우 타겟이 있으면 타겟으로 이동
            if !context.is_empty() && max_para.is_some() {
                let last = &context[context.len() - 1];
                if last.is_textbox {
                    if let Some(link) = overflow_links.iter().find(|l|
                        l.source_parent_para == last.parent_para
                        && l.source_ctrl_idx == last.ctrl_idx
                    ) {
                        // 타겟 글상자 컨텍스트로 전환 (para 0부터 시작)
                        let mut target_ctx = context[..context.len() - 1].to_vec();
                        let ctrl_text_pos = last.ctrl_text_pos; // 같은 텍스트 위치 유지
                        target_ctx.push(NavContextEntry {
                            parent_para: link.target_parent_para,
                            ctrl_idx: link.target_ctrl_idx,
                            ctrl_text_pos,
                            cell_idx: 0,
                            is_textbox: true,
                        });
                        return self.navigate_to_para_start(sec, 0, &target_ctx, overflow_links);
                    }
                }
            }

            // 렌더 범위 끝 → Step 3 (컨테이너 탈출)으로 진행
        } else {
            if para > 0 {
                return self.navigate_to_para_end(sec, para - 1, context, overflow_links);
            }

            // 문단 시작 → 오버플로우 타겟에서 backward 시 소스로 복귀
            if !context.is_empty() {
                let last = &context[context.len() - 1];
                if last.is_textbox {
                    if let Some(link) = overflow_links.iter().find(|l|
                        l.target_parent_para == last.parent_para
                        && l.target_ctrl_idx == last.ctrl_idx
                    ) {
                        // 소스 글상자의 마지막 렌더 문단 끝으로 복귀
                        let mut source_ctx = context[..context.len() - 1].to_vec();
                        let ctrl_text_pos = last.ctrl_text_pos;
                        source_ctx.push(NavContextEntry {
                            parent_para: link.source_parent_para,
                            ctrl_idx: link.source_ctrl_idx,
                            ctrl_text_pos,
                            cell_idx: 0,
                            is_textbox: true,
                        });
                        // overflow_start - 1 = 소스에서 마지막으로 렌더링된 문단
                        let last_rendered = link.overflow_start.saturating_sub(1);
                        return self.navigate_to_para_end(sec, last_rendered, &source_ctx, overflow_links);
                    }
                }
            }

            // 문단 시작 도달 → Step 3 (컨테이너 탈출)으로 진행
        }

        // Step 3: 컨테이너 탈출
        if !context.is_empty() {
            let mut parent_ctx = context.to_vec();
            let last = parent_ctx.pop().unwrap();

            if last.is_textbox {
                // TextBox 탈출: 컨트롤 위치를 기준으로 바로 다음/이전 위치 반환
                // (navigate_next_editable 재귀 호출하면 같은 컨트롤 재진입 위험)
                return self.exit_control(sec, &last, forward, &parent_ctx, overflow_links);
            } else {
                // Table 탈출: 다음/이전 셀 확인
                let parent_paras = match resolve_paragraphs(sections, sec, &parent_ctx, overflow_links) {
                    Some(p) => p,
                    None => return NavResult::Boundary,
                };
                let parent_para = match parent_paras.get(last.parent_para) {
                    Some(p) => p,
                    None => return NavResult::Boundary,
                };
                if let Some(Control::Table(table)) = parent_para.controls.get(last.ctrl_idx) {
                    if let Some(next_cell) = next_cell_index(table, last.cell_idx, forward) {
                        let mut new_ctx = parent_ctx.clone();
                        new_ctx.push(NavContextEntry {
                            parent_para: last.parent_para,
                            ctrl_idx: last.ctrl_idx,
                            ctrl_text_pos: last.ctrl_text_pos,
                            cell_idx: next_cell,
                            is_textbox: false,
                        });
                        if forward {
                            return self.navigate_to_para_start(sec, 0, &new_ctx, overflow_links);
                        } else {
                            let cell_para_count = table.cells.get(next_cell)
                                .map(|c| c.paragraphs.len()).unwrap_or(1);
                            return self.navigate_to_para_end(sec, cell_para_count.saturating_sub(1), &new_ctx, overflow_links);
                        }
                    }
                    // 모든 셀 소진 → 표 탈출 (컨트롤 재진입 방지)
                    return self.exit_control(sec, &last, forward, &parent_ctx, overflow_links);
                }
                return NavResult::Boundary;
            }
        }

        // Step 4: Body 수준 — 다음/이전 섹션
        if forward {
            if sec + 1 < sections.len() {
                return self.navigate_to_para_start(sec + 1, 0, &[], overflow_links);
            }
        } else {
            if sec > 0 {
                let prev_sec = sec - 1;
                let prev_para_count = sections[prev_sec].paragraphs.len();
                return self.navigate_to_para_end(prev_sec, prev_para_count.saturating_sub(1), &[], overflow_links);
            }
        }

        NavResult::Boundary
    }

    /// 문단 시작으로 이동 (forward 방향으로 진입 시 사용)
    /// 문단 시작(offset=0)에 컨트롤이 있으면 자동 진입
    fn navigate_to_para_start(
        &self,
        sec: usize,
        para: usize,
        context: &[NavContextEntry],
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let sections = &self.document.sections;
        let paragraphs = match resolve_paragraphs(sections, sec, context, overflow_links) {
            Some(p) => p,
            None => return NavResult::Boundary,
        };

        if let Some(current_para) = paragraphs.get(para) {
            let text_len = navigable_text_len(current_para);
            let ctrl_positions = find_control_text_positions(current_para);

            // offset 0에 컨트롤이 있는지 확인
            for (ci, ctrl) in current_para.controls.iter().enumerate() {
                let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                if cpos == 0 {
                    if let Some(is_tb) = classify_navigable(ctrl) {
                        if is_tb {
                            // 글상자: 진입
                            return self.enter_control_forward(sec, para, ci, 0, is_tb, context, overflow_links);
                        }
                        // 표: 건너뛰기 → 다음 위치로
                        let skip = 1;
                        if skip <= text_len {
                            return NavResult::Text {
                                sec, para, char_offset: skip,
                                context: context.to_vec(),
                            };
                        }
                        // 표만 있는 문단 → 다음 문단으로
                        return self.navigate_next_editable(sec, para, 0, 1, context, None, overflow_links);
                    }
                }
                if cpos > 0 {
                    break; // 위치 0 이후 → 중단
                }
            }
        }

        NavResult::Text {
            sec,
            para,
            char_offset: 0,
            context: context.to_vec(),
        }
    }

    /// 문단 끝으로 이동 (backward 방향으로 진입 시 사용)
    /// 문단 끝에 컨트롤이 있으면 자동 진입
    fn navigate_to_para_end(
        &self,
        sec: usize,
        para: usize,
        context: &[NavContextEntry],
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let sections = &self.document.sections;
        let paragraphs = match resolve_paragraphs(sections, sec, context, overflow_links) {
            Some(p) => p,
            None => return NavResult::Boundary,
        };

        if let Some(current_para) = paragraphs.get(para) {
            let text_len = navigable_text_len(current_para);
            let ctrl_positions = find_control_text_positions(current_para);

            // 문단 끝에 컨트롤이 있는지 역순 확인
            for (ci, ctrl) in current_para.controls.iter().enumerate().rev() {
                let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                if cpos == text_len {
                    if let Some(is_tb) = classify_navigable(ctrl) {
                        if is_tb {
                            // 글상자: 진입
                            return self.enter_control_backward(sec, para, ci, cpos, is_tb, context, overflow_links);
                        }
                        // 표: 건너뛰기 → 표 앞 위치로
                        // (text_len 위치에 표가 있으므로 text_len은 표 FFFC 뒤 = 실제 텍스트 끝)
                        // backward 진입이므로 text_len 위치에서 멈춤 (표 뒤)
                    }
                }
                if cpos < text_len {
                    break;
                }
            }

            return NavResult::Text {
                sec,
                para,
                char_offset: text_len,
                context: context.to_vec(),
            };
        }

        NavResult::Text {
            sec,
            para,
            char_offset: 0,
            context: context.to_vec(),
        }
    }

    /// 컨트롤에 forward 방향으로 진입 (내부 첫 위치)
    fn enter_control_forward(
        &self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        ctrl_text_pos: usize,
        is_textbox: bool,
        context: &[NavContextEntry],
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let mut new_ctx = context.to_vec();

        if is_textbox {
            new_ctx.push(NavContextEntry {
                parent_para: para,
                ctrl_idx,
                ctrl_text_pos,
                cell_idx: 0,
                is_textbox: true,
            });
            self.navigate_to_para_start(sec, 0, &new_ctx, overflow_links)
        } else {
            // Table: 첫 셀로 진입
            new_ctx.push(NavContextEntry {
                parent_para: para,
                ctrl_idx,
                ctrl_text_pos,
                cell_idx: 0,
                is_textbox: false,
            });
            self.navigate_to_para_start(sec, 0, &new_ctx, overflow_links)
        }
    }

    /// 컨트롤에 backward 방향으로 진입 (내부 마지막 위치)
    fn enter_control_backward(
        &self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        ctrl_text_pos: usize,
        is_textbox: bool,
        context: &[NavContextEntry],
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let sections = &self.document.sections;

        if is_textbox {
            let mut new_ctx = context.to_vec();
            new_ctx.push(NavContextEntry {
                parent_para: para,
                ctrl_idx,
                ctrl_text_pos,
                cell_idx: 0,
                is_textbox: true,
            });

            // 이 글상자가 오버플로우 소스이면, 타겟의 마지막 문단 끝으로 진입
            if let Some(link) = overflow_links.iter().find(|l|
                l.source_parent_para == para && l.source_ctrl_idx == ctrl_idx
            ) {
                let mut target_ctx = context.to_vec();
                target_ctx.push(NavContextEntry {
                    parent_para: link.target_parent_para,
                    ctrl_idx: link.target_ctrl_idx,
                    ctrl_text_pos,
                    cell_idx: 0,
                    is_textbox: true,
                });
                let target_paras = match resolve_paragraphs(sections, sec, &target_ctx, overflow_links) {
                    Some(p) => p,
                    None => return NavResult::Boundary,
                };
                let last_para = target_paras.len().saturating_sub(1);
                return self.navigate_to_para_end(sec, last_para, &target_ctx, overflow_links);
            }

            // TextBox 내부의 마지막 문단 끝으로
            let inner_paras = match resolve_paragraphs(sections, sec, &new_ctx, overflow_links) {
                Some(p) => p,
                None => return NavResult::Boundary,
            };
            let last_para = inner_paras.len().saturating_sub(1);
            self.navigate_to_para_end(sec, last_para, &new_ctx, overflow_links)
        } else {
            // Table: 마지막 셀로 진입
            let parent_paras = match resolve_paragraphs(sections, sec, context, overflow_links) {
                Some(p) => p,
                None => return NavResult::Boundary,
            };
            let parent_para = match parent_paras.get(para) {
                Some(p) => p,
                None => return NavResult::Boundary,
            };
            if let Some(Control::Table(table)) = parent_para.controls.get(ctrl_idx) {
                let last_cell = table.cells.len().saturating_sub(1);
                let mut new_ctx = context.to_vec();
                new_ctx.push(NavContextEntry {
                    parent_para: para,
                    ctrl_idx,
                    ctrl_text_pos,
                    cell_idx: last_cell,
                    is_textbox: false,
                });
                let cell_para_count = table.cells.get(last_cell)
                    .map(|c| c.paragraphs.len()).unwrap_or(1);
                return self.navigate_to_para_end(sec, cell_para_count.saturating_sub(1), &new_ctx, overflow_links);
            }
            NavResult::Boundary
        }
    }

    /// 컨트롤 탈출 시 부모 문단에서 다음/이전 위치를 직접 반환한다.
    /// navigate_next_editable 재귀 호출을 피하여 같은 컨트롤 재진입을 방지한다.
    ///
    /// - Forward 탈출: ctrl_text_pos 위치 반환 (컨트롤 직후의 텍스트 문자)
    /// - Backward 탈출: ctrl_text_pos - 1 위치 반환 (컨트롤 직전의 텍스트 문자)
    ///
    /// 오버플로우 타겟 탈출 시:
    /// - Forward: 소스 컨트롤을 건너뛰고 다음 편집 가능 컨트롤 진입
    /// - Backward: (Step 2에서 처리 — 여기에 도달하지 않음)
    fn exit_control(
        &self,
        sec: usize,
        exited: &NavContextEntry,
        forward: bool,
        parent_ctx: &[NavContextEntry],
        overflow_links: &[OverflowLink],
    ) -> NavResult {
        let sections = &self.document.sections;

        // 오버플로우 타겟 탈출인지 확인
        let target_link = overflow_links.iter().find(|l|
            l.target_parent_para == exited.parent_para
            && l.target_ctrl_idx == exited.ctrl_idx
        );

        if forward {
            // 부모 문단의 텍스트 길이 확인
            let parent_paras = match resolve_paragraphs(sections, sec, parent_ctx, overflow_links) {
                Some(p) => p,
                None => return NavResult::Boundary,
            };
            let text_len = parent_paras.get(exited.parent_para)
                .map(|p| navigable_text_len(p))
                .unwrap_or(0);

            if exited.ctrl_text_pos <= text_len {
                // 컨트롤 다음 위치에 다른 편집 가능 컨트롤이 있는지 확인
                if let Some(parent_p) = parent_paras.get(exited.parent_para) {
                    let ctrl_positions = find_control_text_positions(parent_p);
                    // 오버플로우 타겟 탈출 시 소스 컨트롤 건너뛰기
                    let skip_ctrl_idx = target_link.map(|l| l.source_ctrl_idx);

                    for (ci, ctrl) in parent_p.controls.iter().enumerate() {
                        if ci <= exited.ctrl_idx {
                            continue; // 같은 컨트롤이거나 이미 지나친 것
                        }
                        // 소스 컨트롤 건너뛰기
                        if Some(ci) == skip_ctrl_idx {
                            continue;
                        }
                        let cpos = ctrl_positions.get(ci).copied().unwrap_or(text_len);
                        if cpos == exited.ctrl_text_pos {
                            // 같은 위치에 다른 편집 가능 컨트롤 → 진입
                            if let Some(is_tb) = classify_navigable(ctrl) {
                                return self.enter_control_forward(
                                    sec, exited.parent_para, ci, cpos, is_tb, parent_ctx, overflow_links,
                                );
                            }
                        }
                        if cpos > exited.ctrl_text_pos {
                            break; // 더 뒤 위치는 나중에 도달
                        }
                    }
                }

                // ctrl_text_pos가 문단 끝 미만이면 해당 위치로 이동
                // (문단 끝이면 다음 문단/섹션으로 진행)
                if exited.ctrl_text_pos < text_len {
                    return NavResult::Text {
                        sec,
                        para: exited.parent_para,
                        char_offset: exited.ctrl_text_pos,
                        context: parent_ctx.to_vec(),
                    };
                }
            }

            // ctrl_text_pos >= text_len → 다음 문단으로
            let para_count = parent_paras.len();
            if exited.parent_para + 1 < para_count {
                return self.navigate_to_para_start(sec, exited.parent_para + 1, parent_ctx, overflow_links);
            }
            // 부모 컨테이너도 끝 → 더 상위로 탈출
            if !parent_ctx.is_empty() {
                let mut grandparent_ctx = parent_ctx.to_vec();
                let grandparent = grandparent_ctx.pop().unwrap();
                return self.exit_control(sec, &grandparent, true, &grandparent_ctx, overflow_links);
            }
            // Body 수준 → 다음 섹션
            if sec + 1 < sections.len() {
                return self.navigate_to_para_start(sec + 1, 0, &[], overflow_links);
            }
            NavResult::Boundary
        } else {
            // Backward: 부모 문단 정보 조회
            let parent_paras = match resolve_paragraphs(sections, sec, parent_ctx, overflow_links) {
                Some(p) => p,
                None => return NavResult::Boundary,
            };

            // 같은 텍스트 위치에 이전 편집 가능 컨트롤이 있으면 backward 진입
            // (모든 컨트롤이 ctrl_text_pos=0인 문단에서 역방향 탐색에 필요)
            if let Some(parent_p) = parent_paras.get(exited.parent_para) {
                let ctrl_positions = find_control_text_positions(parent_p);
                for (ci, ctrl) in parent_p.controls.iter().enumerate().rev() {
                    if ci >= exited.ctrl_idx {
                        continue;
                    }
                    let cpos = ctrl_positions.get(ci).copied()
                        .unwrap_or(navigable_text_len(parent_p));
                    if cpos == exited.ctrl_text_pos {
                        if let Some(is_tb) = classify_navigable(ctrl) {
                            return self.enter_control_backward(
                                sec, exited.parent_para, ci, cpos, is_tb, parent_ctx, overflow_links,
                            );
                        }
                    }
                    if cpos < exited.ctrl_text_pos {
                        break;
                    }
                }
            }

            // ctrl_text_pos > 0: 컨트롤 직전 텍스트 위치 (ctrl_text_pos - 1)
            if exited.ctrl_text_pos > 0 {
                let prev_pos = exited.ctrl_text_pos - 1;
                // prev_pos에 다른 편집 가능 컨트롤이 있는지 확인
                if let Some(parent_p) = parent_paras.get(exited.parent_para) {
                    let ctrl_positions = find_control_text_positions(parent_p);
                    for (ci, ctrl) in parent_p.controls.iter().enumerate().rev() {
                        if ci >= exited.ctrl_idx {
                            continue;
                        }
                        let cpos = ctrl_positions.get(ci).copied()
                            .unwrap_or(navigable_text_len(parent_p));
                        if cpos == prev_pos {
                            if let Some(is_tb) = classify_navigable(ctrl) {
                                return self.enter_control_backward(
                                    sec, exited.parent_para, ci, cpos, is_tb, parent_ctx, overflow_links,
                                );
                            }
                        }
                        if cpos < prev_pos {
                            break;
                        }
                    }
                }
                return NavResult::Text {
                    sec,
                    para: exited.parent_para,
                    char_offset: prev_pos,
                    context: parent_ctx.to_vec(),
                };
            }

            // ctrl_text_pos == 0: 문단 시작 → 이전 문단
            if exited.parent_para > 0 {
                return self.navigate_to_para_end(sec, exited.parent_para - 1, parent_ctx, overflow_links);
            }
            // 부모 컨테이너도 시작 → 더 상위로 탈출
            if !parent_ctx.is_empty() {
                let mut grandparent_ctx = parent_ctx.to_vec();
                let grandparent = grandparent_ctx.pop().unwrap();
                return self.exit_control(sec, &grandparent, false, &grandparent_ctx, overflow_links);
            }
            // Body 수준 → 이전 섹션
            if sec > 0 {
                let prev_sec = sec - 1;
                let prev_para_count = sections[prev_sec].paragraphs.len();
                return self.navigate_to_para_end(prev_sec, prev_para_count.saturating_sub(1), &[], overflow_links);
            }
            NavResult::Boundary
        }
    }

    /// NavResult를 JSON 문자열로 직렬화한다.
    pub(crate) fn nav_result_to_json(result: &NavResult) -> String {
        match result {
            NavResult::Text { sec, para, char_offset, context } => {
                let ctx_json: Vec<String> = context.iter().map(|e| {
                    format!(
                        "{{\"parentPara\":{},\"ctrlIdx\":{},\"ctrlTextPos\":{},\"cellIdx\":{},\"isTextBox\":{}}}",
                        e.parent_para, e.ctrl_idx, e.ctrl_text_pos, e.cell_idx, e.is_textbox
                    )
                }).collect();
                format!(
                    "{{\"type\":\"text\",\"sec\":{},\"para\":{},\"charOffset\":{},\"context\":[{}]}}",
                    sec, para, char_offset, ctx_json.join(",")
                )
            }
            NavResult::Boundary => {
                "{\"type\":\"boundary\"}".to_string()
            }
        }
    }

    /// JSON 문자열에서 NavContextEntry 배열을 파싱한다.
    pub(crate) fn parse_nav_context(json: &str) -> Vec<NavContextEntry> {
        let json = json.trim();
        if json == "[]" || json.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        // 간단한 JSON 배열 파서
        let inner = json.trim_start_matches('[').trim_end_matches(']');
        if inner.trim().is_empty() {
            return result;
        }

        // {...},{...} 형태를 분리
        let mut depth = 0;
        let mut start = 0;
        let bytes = inner.as_bytes();
        for i in 0..bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let obj = &inner[start..=i].trim();
                        if let Some(entry) = Self::parse_nav_entry(obj) {
                            result.push(entry);
                        }
                        start = i + 1;
                    }
                }
                b',' if depth == 0 => {
                    start = i + 1;
                }
                _ => {}
            }
        }

        result
    }

    fn parse_nav_entry(json: &str) -> Option<NavContextEntry> {
        use crate::document_core::helpers::{json_i32, json_bool};
        let parent_para = json_i32(json, "parentPara")? as usize;
        let ctrl_idx = json_i32(json, "ctrlIdx")? as usize;
        let ctrl_text_pos = json_i32(json, "ctrlTextPos").unwrap_or(0) as usize;
        let cell_idx = json_i32(json, "cellIdx").unwrap_or(0) as usize;
        let is_textbox = json_bool(json, "isTextBox").unwrap_or(true);
        Some(NavContextEntry {
            parent_para,
            ctrl_idx,
            ctrl_text_pos,
            cell_idx,
            is_textbox,
        })
    }

    /// 섹션의 글상자 오버플로우 연결 정보를 계산(캐시)하여 반환한다.
    pub(crate) fn get_overflow_links(&self, sec_idx: usize) -> Vec<OverflowLink> {
        {
            let cache = self.overflow_links_cache.borrow();
            if let Some(links) = cache.get(&sec_idx) {
                return links.clone();
            }
        }
        let links = self.compute_overflow_links(sec_idx);
        self.overflow_links_cache.borrow_mut().insert(sec_idx, links.clone());
        links
    }

    /// 섹션 내 글상자 오버플로우 연결 정보를 계산한다.
    ///
    /// scan_textbox_overflow()와 동일한 알고리즘:
    /// 1. Rectangle Shape + TextBox가 있는 컨트롤 수집
    /// 2. 소스 감지: segment_width 변화 + vpos 리셋 → overflow_start 결정
    /// 3. 타겟 감지: 모든 문단의 텍스트가 비어있는 글상자
    /// 4. segment_width 매칭으로 소스→타겟 매핑
    fn compute_overflow_links(&self, sec_idx: usize) -> Vec<OverflowLink> {
        let section = match self.document.sections.get(sec_idx) {
            Some(s) => s,
            None => return Vec::new(),
        };

        // 오버플로우 소스 수집: (target_sw, para_idx, ctrl_idx, overflow_start)
        let mut overflow_sources: Vec<(i32, usize, usize, usize)> = Vec::new();
        // 빈 타겟 글상자 수집: (para_idx, ctrl_idx, inner_sw)
        let mut empty_targets: Vec<(usize, usize, i32)> = Vec::new();

        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let drawing = match ctrl {
                    Control::Shape(s) => match s.as_ref() {
                        ShapeObject::Rectangle(r) => &r.drawing,
                        _ => continue,
                    },
                    _ => continue,
                };
                let tb = match &drawing.text_box {
                    Some(tb) => tb,
                    None => continue,
                };

                let has_text = tb.paragraphs.iter().any(|p| !p.text.is_empty());
                if !has_text {
                    // 빈 글상자 → 타겟 후보
                    let inner_sw = tb.paragraphs.first()
                        .and_then(|p| p.line_segs.first())
                        .map(|ls| ls.segment_width)
                        .unwrap_or(0);
                    empty_targets.push((pi, ci, inner_sw));
                    continue;
                }

                // 오버플로우 감지 (scan_textbox_overflow와 동일)
                let first_sw = tb.paragraphs.first()
                    .and_then(|p| p.line_segs.first())
                    .map(|ls| ls.segment_width)
                    .unwrap_or(0);
                let mut max_vpos_end: i32 = 0;
                let mut overflow_idx: Option<usize> = None;
                for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                    if let Some(first_ls) = tp.line_segs.first() {
                        if tpi > 0 && first_ls.segment_width != first_sw
                            && first_ls.vertical_pos < max_vpos_end
                        {
                            overflow_idx = Some(tpi);
                            break;
                        }
                        if let Some(last_ls) = tp.line_segs.last() {
                            let end = last_ls.vertical_pos + last_ls.line_height;
                            if end > max_vpos_end {
                                max_vpos_end = end;
                            }
                        }
                    }
                }

                if let Some(oi) = overflow_idx {
                    let target_sw = tb.paragraphs[oi].line_segs.first()
                        .map(|ls| ls.segment_width)
                        .unwrap_or(0);
                    overflow_sources.push((target_sw, pi, ci, oi));
                }
            }
        }

        // 소스→타겟 매핑 (segment_width 가장 가까운 빈 글상자)
        let mut links = Vec::new();
        for (target_sw, src_pi, src_ci, oi) in overflow_sources {
            let best = empty_targets.iter()
                .enumerate()
                .min_by_key(|(_, (_, _, esw))| (target_sw - *esw).abs());
            if let Some((idx, &(tgt_pi, tgt_ci, _))) = best {
                links.push(OverflowLink {
                    source_parent_para: src_pi,
                    source_ctrl_idx: src_ci,
                    overflow_start: oi,
                    target_parent_para: tgt_pi,
                    target_ctrl_idx: tgt_ci,
                });
                empty_targets.remove(idx);
            }
        }

        links
    }
}
