//! 문서 구조 구성 (Document Composition)
//!
//! 문단의 텍스트를 줄 단위로 분할하고, 각 줄 내에서
//! CharShapeRef 경계에 따라 다중 TextRun으로 분할한다.
//! 인라인 컨트롤(표/도형) 삽입 위치를 식별한다.

use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use super::layout::{estimate_text_width, resolved_to_text_style};
use super::style_resolver::{ResolvedStyleSet, detect_lang_category};
use super::{TextStyle, px_to_hwpunit};

/// 글자겹침(CharOverlap) 렌더링 정보
#[derive(Debug, Clone, serde::Serialize)]
pub struct CharOverlapInfo {
    /// 테두리 타입 (0=없음, 1=원, 2=반전원, 3=사각형, 4=반전사각형)
    pub border_type: u8,
    /// 내부 글자 크기 (%, 기본 100)
    pub inner_char_size: i8,
}

/// 구성된 텍스트 런 (줄 내 동일 스타일 + 동일 언어 구간)
#[derive(Debug, Clone, Default)]
pub struct ComposedTextRun {
    /// 텍스트 조각
    pub text: String,
    /// 글자 스타일 ID (ResolvedStyleSet.char_styles 인덱스)
    pub char_style_id: u32,
    /// 언어 카테고리 (0=한국어, 1=영어, 2=한자, 3=일본어, 4=기타, 5=기호, 6=사용자)
    pub lang_index: usize,
    /// 글자겹침 정보 (CharOverlap 컨트롤에서 생성된 런인 경우)
    pub char_overlap: Option<CharOverlapInfo>,
    /// 각주/미주 마커 (Some이면 위첨자로 렌더링, 텍스트 흐름에 포함)
    pub footnote_marker: Option<u16>,
    /// PUA 옛한글 변환 후 표시 텍스트 (Some 이면 렌더러는 본 필드 사용).
    /// `text` 는 IR 와 동일하게 PUA char 1글자로 보존하여 char_offsets /
    /// char_start / line_chars 등 인덱싱 불변성을 유지한다 (Task #528).
    pub display_text: Option<String>,
}

/// 구성된 줄 (LineSeg 기반)
#[derive(Debug, Clone)]
pub struct ComposedLine {
    /// 스타일별 텍스트 런 목록
    pub runs: Vec<ComposedTextRun>,
    /// 원본 LineSeg (높이, 베이스라인 등)
    pub line_height: i32,
    /// 베이스라인 거리
    pub baseline_distance: i32,
    /// 세그먼트 폭
    pub segment_width: i32,
    /// 컬럼 시작 위치
    pub column_start: i32,
    /// 줄간격 (LineSeg.line_spacing)
    pub line_spacing: i32,
    /// 강제 줄 바꿈(\n, Shift+Enter)으로 끝나는 줄인지 여부
    pub has_line_break: bool,
    /// 이 줄의 첫 문자가 para.text 내에서 갖는 절대 char 인덱스
    pub char_start: usize,
}

/// 인라인 컨트롤 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InlineControlType {
    /// 표
    Table,
    /// 도형/그림
    Shape,
    /// 기타 (구역정의, 단정의 등)
    Other,
}

/// 인라인 컨트롤 위치 정보
#[derive(Debug, Clone)]
pub struct InlineControl {
    /// 삽입될 줄 인덱스
    pub line_index: usize,
    /// Paragraph.controls 내 인덱스
    pub control_index: usize,
    /// 컨트롤 종류
    pub control_type: InlineControlType,
}

/// 구성된 문단
#[derive(Debug, Clone)]
pub struct ComposedParagraph {
    /// 줄별 텍스트
    pub lines: Vec<ComposedLine>,
    /// 문단 스타일 ID
    pub para_style_id: u16,
    /// 인라인 컨트롤 위치 목록
    pub inline_controls: Vec<InlineControl>,
    /// 개요 번호/글머리표 등 문단 머리 텍스트 (렌더링 전용)
    /// 문서 좌표 char_offset에 포함되지 않으며 별도 TextRunNode로 렌더링된다.
    pub numbering_text: Option<String>,
    /// treat_as_char 컨트롤의 텍스트 위치와 HWPUNIT 너비 목록
    /// (para.text 내 절대 char 인덱스, 폭 HWPUNIT, para.controls 내 인덱스)
    pub tac_controls: Vec<(usize, i32, usize)>,
    /// 각주/미주 위치: (텍스트 내 char 인덱스, 번호, para.controls 내 인덱스)
    pub footnote_positions: Vec<(usize, u16, usize)>,
    /// 탭 확장 데이터 (HWP tab_extended / HWPX 인라인 탭)
    /// ext[0]=width, ext[1]=leader/fill_type, ext[2]=tab_type
    pub tab_extended: Vec<[u16; 7]>,
}

/// 구역의 문단 목록을 구성한다.
pub fn compose_section(section: &Section) -> Vec<ComposedParagraph> {
    section
        .paragraphs
        .iter()
        .map(compose_paragraph)
        .collect()
}

/// 문단을 줄별 텍스트 런으로 분할한다.
pub fn compose_paragraph(para: &Paragraph) -> ComposedParagraph {
    let mut lines = compose_lines(para);
    let inline_controls = identify_inline_controls(para);

    // treat_as_char 컨트롤의 텍스트 위치와 HWPUNIT 너비 수집
    let tac_positions = find_control_text_positions(para);
    let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
    let tac_controls: Vec<(usize, i32, usize)> = para.controls.iter().enumerate()
        .filter_map(|(i, ctrl)| {
            let pos = *tac_positions.get(i)?;
            match ctrl {
                Control::Picture(p) if p.common.treat_as_char => {
                    Some((pos, p.common.width as i32, i))
                }
                Control::Shape(s) if s.common().treat_as_char => {
                    Some((pos, s.common().width as i32, i))
                }
                Control::Equation(eq) => {
                    // HWP 저장값을 사용 — 한컴 편집기가 실제 폰트로 계산한 정확한 너비
                    Some((pos, eq.common.width as i32, i))
                }
                Control::Form(f) => {
                    Some((pos, f.width as i32, i))
                }
                Control::Table(t) if t.common.treat_as_char
                    && super::height_measurer::is_tac_table_inline(t, seg_width, &para.text, &para.controls) => {
                    let table_width: u32 = t.get_column_widths().iter().sum();
                    Some((pos, table_width as i32, i))
                }
                _ => None,
            }
        })
        .collect();

    // 각주/미주 위치 수집
    let footnote_positions: Vec<(usize, u16, usize)> = para.controls.iter().enumerate()
        .filter_map(|(i, ctrl)| {
            let pos = *tac_positions.get(i)?;
            match ctrl {
                Control::Footnote(fn_) => Some((pos, fn_.number, i)),
                Control::Endnote(en) => Some((pos, en.number, i)),
                _ => None,
            }
        })
        .collect();

    // 각주 마커는 paragraph_layout에서 FootnoteMarker 노드로 처리 (텍스트에 삽입하지 않음)

    let mut composed = ComposedParagraph {
        lines,
        para_style_id: para.para_shape_id,
        inline_controls,
        numbering_text: None,
        tac_controls,
        footnote_positions,
        tab_extended: para.tab_extended.clone(),
    };

    // CharOverlap 글자를 조합된 텍스트에 삽입
    inject_char_overlap_text(&mut composed, para);

    // PUA 테두리 숫자(사각형/원형 안의 숫자) → CharOverlap 런으로 변환
    convert_pua_enclosed_numbers(&mut composed);

    // Hanyang-PUA 옛한글 → KS X 1026-1:2007 자모 시퀀스 (Task #528)
    convert_pua_old_hangul(&mut composed);

    composed
}

/// Hanyang-PUA 옛한글 코드포인트를 KS X 1026-1:2007 자모 시퀀스로 변환한다.
///
/// 한컴 자체 폰트 (함초롬바탕 LVT 등) 는 PUA 영역에 옛한글 글리프를 직접
/// 보유하나, OFL 폰트 (Noto Serif KR / Source Han Serif K 등) 는 KS X 1026-1
/// 자모 영역만 지원하므로 PUA → 자모 변환 후 합자 렌더링이 필요.
///
/// 본 함수는 `run.text` 를 변경하지 않고 `run.display_text` 에만 변환 결과를
/// 저장한다. 이는 `char_offsets`, `line.char_start`, `line_chars` 등 인덱싱
/// 불변성을 유지하기 위함이다 (PUA 1 char = display N jamos).
///
/// 매핑 표: KTUG HanyangPuaTableProject (Public Domain).
fn convert_pua_old_hangul(composed: &mut ComposedParagraph) {
    use super::pua_oldhangul::map_pua_old_hangul;
    for line in composed.lines.iter_mut() {
        for run in line.runs.iter_mut() {
            if !run.text.chars().any(|ch| map_pua_old_hangul(ch).is_some()) {
                continue;
            }
            let mut display = String::with_capacity(run.text.len() * 3);
            for ch in run.text.chars() {
                if let Some(jamos) = map_pua_old_hangul(ch) {
                    display.extend(jamos.iter().copied());
                } else {
                    display.push(ch);
                }
            }
            run.display_text = Some(display);
        }
    }
}

/// 각주 마커를 해당 텍스트 위치의 런에 인라인 삽입
/// 각주 위치에서 기존 런을 분할하고 마커 런("1)" 등)을 사이에 삽입
fn inject_footnote_markers(lines: &mut [ComposedLine], positions: &[(usize, u16)]) {
    for &(char_pos, number) in positions {
        let marker_text = format!("{})", number);
        // char_pos에 해당하는 줄과 런 찾기
        for line in lines.iter_mut() {
            let line_start = line.char_start;
            let line_end = line_start + line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>();
            if char_pos < line_start || char_pos > line_end { continue; }

            // 이 줄 내에서 char_pos에 해당하는 런 찾기
            let mut run_char = line_start;
            let mut target_run_idx = None;
            let mut offset_in_run = 0;
            for (ri, run) in line.runs.iter().enumerate() {
                let run_len = run.text.chars().count();
                if char_pos >= run_char && char_pos <= run_char + run_len {
                    target_run_idx = Some(ri);
                    offset_in_run = char_pos - run_char;
                    break;
                }
                run_char += run_len;
            }

            if let Some(ri) = target_run_idx {
                let orig_run = &line.runs[ri];
                let cs_id = orig_run.char_style_id;
                let lang = orig_run.lang_index;

                // 런을 분할: [앞부분] [마커] [뒷부분]
                let orig_text: Vec<char> = orig_run.text.chars().collect();
                let before: String = orig_text[..offset_in_run].iter().collect();
                let after: String = orig_text[offset_in_run..].iter().collect();

                let marker_run = ComposedTextRun {
                    text: marker_text.clone(),
                    char_style_id: cs_id,
                    lang_index: lang,
                    char_overlap: None,
                    footnote_marker: Some(number),
                    display_text: None,                };

                let mut new_runs = Vec::new();
                // 앞부분에서 기존 런 교체
                for (i, run) in line.runs.iter().enumerate() {
                    if i == ri {
                        if !before.is_empty() {
                            new_runs.push(ComposedTextRun {
                                text: before.clone(),
                                char_style_id: cs_id,
                                lang_index: lang,
                                char_overlap: run.char_overlap.clone(),
                                footnote_marker: None,
                                display_text: None,                            });
                        }
                        new_runs.push(marker_run.clone());
                        if !after.is_empty() {
                            new_runs.push(ComposedTextRun {
                                text: after.clone(),
                                char_style_id: cs_id,
                                lang_index: lang,
                                char_overlap: run.char_overlap.clone(),
                                footnote_marker: None,
                                display_text: None,                            });
                        }
                    } else {
                        new_runs.push(run.clone());
                    }
                }
                line.runs = new_runs;
                break; // 이 각주 처리 완료
            }
        }
    }
}

/// 문단의 텍스트를 줄별로 분할하고, 각 줄 내에서 CharShapeRef 경계에 따라 분할한다.
fn compose_lines(para: &Paragraph) -> Vec<ComposedLine> {
    if para.line_segs.is_empty() {
        // LineSeg가 없으면 전체 텍스트를 하나의 줄로
        if para.text.is_empty() {
            return Vec::new();
        }
        let default_style_id = para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        return vec![ComposedLine {
            runs: split_runs_by_lang(vec![ComposedTextRun {
                text: para.text.clone(),
                char_style_id: default_style_id,
                lang_index: 0,
                char_overlap: None,
                    footnote_marker: None,
                    display_text: None,            }]),
            line_height: 400,
            baseline_distance: 320,
            segment_width: 0,
            column_start: 0,
            line_spacing: 0,
            has_line_break: false,
            char_start: 0,
        }];
    }

    let mut lines = Vec::new();

    for line_idx in 0..para.line_segs.len() {
        let line_seg = &para.line_segs[line_idx];

        // UTF-16 위치 기반으로 이 줄의 텍스트 범위 계산
        let utf16_start = line_seg.text_start;
        let utf16_end = if line_idx + 1 < para.line_segs.len() {
            para.line_segs[line_idx + 1].text_start
        } else {
            // 마지막 줄: char_count 또는 텍스트 끝까지
            if para.char_count > 0 {
                para.char_count
            } else {
                // char_count 미설정 시 텍스트 길이 기반 추정
                para.text.chars().count() as u32 + 1
            }
        };

        // UTF-16 위치 → 텍스트 문자 인덱스로 변환
        let (text_start, text_end) = utf16_range_to_text_range(
            &para.char_offsets,
            utf16_start,
            utf16_end,
            para.text.chars().count(),
        );

        // 이 줄의 텍스트 추출
        let line_text: String = para.text.chars().skip(text_start).take(text_end - text_start).collect();

        // TAC 표 문단 감지
        let has_tac = para.controls.iter().any(|c|
            matches!(c, crate::model::control::Control::Table(t) if t.common.treat_as_char));

        // 강제 줄넘김(\n) + TAC 표 문단 처리 (Task #19/Task #20)
        let newline_pos = line_text.find('\n');
        if let (true, Some(nl_pos)) = (has_tac, newline_pos) {
            let pre_text: String = line_text.chars().take(nl_pos).collect();
            let pre_end = text_start + nl_pos;

            if !pre_text.is_empty() && !lines.is_empty() {
                // \n 앞 텍스트를 이전 ComposedLine에 합침 (한컴 방식: \n 전 전체가 한 줄)
                let prev: &mut ComposedLine = lines.last_mut().unwrap();
                let mut extra_runs = split_by_char_shapes(
                    &pre_text, text_start, pre_end,
                    &para.char_offsets, &para.char_shapes,
                );
                prev.runs.append(&mut extra_runs);
                prev.has_line_break = true;
            } else if !pre_text.is_empty() {
                // 이전 줄이 없으면 새 ComposedLine 생성
                let pre_runs = split_by_char_shapes(
                    &pre_text, text_start, pre_end,
                    &para.char_offsets, &para.char_shapes,
                );
                let pre_lh = if line_seg.text_height > 0
                    && line_seg.text_height < line_seg.line_height / 3 {
                    line_seg.text_height
                } else {
                    line_seg.line_height
                };
                lines.push(ComposedLine {
                    runs: pre_runs,
                    line_height: pre_lh,
                    baseline_distance: line_seg.baseline_distance,
                    segment_width: line_seg.segment_width,
                    column_start: line_seg.column_start,
                    line_spacing: line_seg.line_spacing,
                    has_line_break: true,
                    char_start: text_start,
                });
            }

            // \n 이후: 표 줄 (빈 runs, 표는 layout에서 별도 처리)
            let post_start = text_start + nl_pos + 1;
            let post_text: String = line_text.chars().skip(nl_pos + 1).collect();
            let post_text_clean = post_text.trim_end_matches('\n').to_string();
            let post_runs = split_by_char_shapes(
                &post_text_clean, post_start, text_end,
                &para.char_offsets, &para.char_shapes,
            );
            lines.push(ComposedLine {
                runs: post_runs,
                line_height: line_seg.line_height,
                baseline_distance: line_seg.baseline_distance,
                segment_width: line_seg.segment_width,
                column_start: line_seg.column_start,
                line_spacing: line_seg.line_spacing,
                has_line_break: post_text.ends_with('\n'),
                char_start: post_start,
            });
        } else {
            // 일반 처리: 강제 줄넘김 없거나 끝에만 있는 경우
            let has_line_break = line_text.ends_with('\n');
            let line_text = if has_line_break {
                line_text.trim_end_matches('\n').to_string()
            } else {
                line_text
            };

            let runs = split_by_char_shapes(
                &line_text, text_start, text_end,
                &para.char_offsets, &para.char_shapes,
            );

            // TAC 표 문단: lh에 표 높이가 포함된 텍스트 줄은 th로 보정 (Task #19)
            let corrected_lh = if has_tac
                && line_seg.text_height > 0
                && line_seg.text_height < line_seg.line_height / 3
            {
                line_seg.text_height
            } else {
                line_seg.line_height
            };

            lines.push(ComposedLine {
                runs,
                line_height: corrected_lh,
                baseline_distance: line_seg.baseline_distance,
                segment_width: line_seg.segment_width,
                column_start: line_seg.column_start,
                line_spacing: line_seg.line_spacing,
                has_line_break,
                char_start: text_start,
            });
        }
    }

    lines
}

/// UTF-16 위치 범위를 텍스트 문자 인덱스 범위로 변환한다.
pub(crate) fn utf16_range_to_text_range(
    char_offsets: &[u32],
    utf16_start: u32,
    utf16_end: u32,
    text_len: usize,
) -> (usize, usize) {
    if char_offsets.is_empty() {
        // 오프셋 정보가 없으면 1:1 매핑 가정
        let start = (utf16_start as usize).min(text_len);
        let end = (utf16_end as usize).min(text_len);
        return (start, end);
    }

    // char_offsets[i] >= utf16_start인 첫 번째 i가 text_start
    let text_start = char_offsets
        .iter()
        .position(|&off| off >= utf16_start)
        .unwrap_or(text_len);

    // char_offsets[i] >= utf16_end인 첫 번째 i가 text_end
    let text_end = char_offsets
        .iter()
        .position(|&off| off >= utf16_end)
        .unwrap_or(text_len);

    (text_start, text_end)
}

/// 줄 내 텍스트를 CharShapeRef 경계에 따라 다중 TextRun으로 분할한다.
fn split_by_char_shapes(
    line_text: &str,
    text_start: usize,
    text_end: usize,
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
) -> Vec<ComposedTextRun> {
    if line_text.is_empty() {
        return Vec::new();
    }

    if char_shapes.is_empty() {
        return split_runs_by_lang(vec![ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: 0,
            lang_index: 0,
            char_overlap: None,
                    footnote_marker: None,
                    display_text: None,        }]);
    }

    // 이 줄 범위에 영향을 미치는 CharShapeRef 찾기
    // CharShapeRef.start_pos는 UTF-16 위치이므로 텍스트 인덱스로 변환해야 함
    let line_utf16_start = if text_start < char_offsets.len() {
        char_offsets[text_start]
    } else if !char_offsets.is_empty() {
        *char_offsets.last().unwrap() + 1
    } else {
        text_start as u32
    };

    let line_utf16_end = if text_end < char_offsets.len() {
        char_offsets[text_end]
    } else if !char_offsets.is_empty() {
        *char_offsets.last().unwrap() + 1
    } else {
        text_end as u32
    };

    // 이 줄에 적용되는 CharShapeRef 구간 수집
    // 각 구간: (텍스트 내 시작 인덱스, char_style_id)
    let mut segments: Vec<(usize, u32)> = Vec::new();

    for cs in char_shapes {
        if cs.start_pos < line_utf16_end {
            // 이 CharShapeRef의 시작 위치를 줄 내 텍스트 인덱스로 변환
            let text_idx = if cs.start_pos <= line_utf16_start {
                0 // 줄 시작 이전이면 0
            } else {
                // char_offsets에서 cs.start_pos에 해당하는 텍스트 인덱스 찾기
                let global_idx = char_offsets
                    .iter()
                    .position(|&off| off >= cs.start_pos)
                    .unwrap_or(text_end);
                global_idx.saturating_sub(text_start)
            };

            segments.push((text_idx, cs.char_shape_id));
        }
    }

    // 시작 인덱스로 정렬 (동일 인덱스 내에서는 원래 순서 유지)
    segments.sort_by_key(|&(idx, _)| idx);

    // 중복 시작 위치 제거: 동일 위치의 마지막 것(가장 최근 CharShapeRef)만 유지
    // 뒤에서부터 dedup하면 마지막 것이 유지됨
    segments.reverse();
    segments.dedup_by_key(|s| s.0);
    segments.reverse();

    // segments가 비어있으면 첫 번째 CharShapeRef 사용
    if segments.is_empty() {
        // 줄 시작 위치 이전의 마지막 CharShapeRef 찾기
        let style_id = find_active_char_shape(char_shapes, line_utf16_start);
        return split_runs_by_lang(vec![ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            char_overlap: None,
                    footnote_marker: None,
                    display_text: None,        }]);
    }

    // TextRun 생성
    let chars: Vec<char> = line_text.chars().collect();
    let mut runs = Vec::new();

    for i in 0..segments.len() {
        let (start_idx, style_id) = segments[i];
        let end_idx = if i + 1 < segments.len() {
            segments[i + 1].0
        } else {
            chars.len()
        };

        if start_idx < end_idx && start_idx < chars.len() {
            let actual_end = end_idx.min(chars.len());
            let run_text: String = chars[start_idx..actual_end].iter().collect();
            if !run_text.is_empty() {
                runs.push(ComposedTextRun {
                    text: run_text,
                    char_style_id: style_id,
                    lang_index: 0,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,                });
            }
        }
    }

    // 첫 번째 segment가 0이 아닌 경우, 앞 부분 처리
    if !segments.is_empty() && segments[0].0 > 0 {
        let style_id = find_active_char_shape(char_shapes, line_utf16_start);
        let end_idx = segments[0].0.min(chars.len());
        let prefix_text: String = chars[..end_idx].iter().collect();
        if !prefix_text.is_empty() {
            runs.insert(
                0,
                ComposedTextRun {
                    text: prefix_text,
                    char_style_id: style_id,
                    lang_index: 0,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,                },
            );
        }
    }

    if runs.is_empty() {
        let style_id = find_active_char_shape(char_shapes, line_utf16_start);
        runs.push(ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            char_overlap: None,
                    footnote_marker: None,
                    display_text: None,        });
    }

    // 언어 카테고리별로 Run을 세분화
    split_runs_by_lang(runs)
}

/// 주어진 UTF-16 위치에서 활성화된 CharShapeRef의 char_shape_id를 찾는다.
pub(crate) fn find_active_char_shape(char_shapes: &[CharShapeRef], utf16_pos: u32) -> u32 {
    let mut active_id = char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0);
    for cs in char_shapes {
        if cs.start_pos <= utf16_pos {
            active_id = cs.char_shape_id;
        } else {
            break;
        }
    }
    active_id
}

/// TextRun 목록을 언어 카테고리 경계에 따라 세분화한다.
///
/// 동일 CharShape 내에서도 한글→영문 전환 시 별도 Run으로 분리하여
/// 각 언어에 맞는 폰트를 적용할 수 있도록 한다.
///
/// 공백/구두점은 이전 문자의 언어를 따른다 (불필요한 Run 분할 방지).
pub(crate) fn split_runs_by_lang(runs: Vec<ComposedTextRun>) -> Vec<ComposedTextRun> {
    let mut result = Vec::new();

    for run in runs {
        let chars: Vec<char> = run.text.chars().collect();
        if chars.is_empty() {
            result.push(run);
            continue;
        }

        // 첫 번째 비중립 문자의 언어를 찾아 초기 언어로 설정
        let initial_lang = chars.iter()
            .map(|&c| detect_lang_category(c))
            .find(|&lang| lang != 0 || chars.iter().all(|&c| detect_lang_category(c) == 0))
            .unwrap_or(0);

        let mut current_lang = initial_lang;
        let mut current_start = 0;

        for (i, &ch) in chars.iter().enumerate() {
            let char_lang = detect_lang_category(ch);

            // 언어 중립 문자(공백/구두점 등 = 기본값 0)는 이전 언어를 따름
            // 단, detect_lang_category가 0을 반환하는 것은 한국어 또는 중립 두 가지 경우:
            //   - 한글 음절/자모: 명시적으로 0번 매치
            //   - 공백/구두점: _ => 0 폴백
            // 한글 음절은 확실한 한국어이므로 구분해야 함
            let is_neutral = is_lang_neutral(ch);

            if is_neutral {
                // 중립 문자: 현재 언어 유지
                continue;
            }

            if char_lang != current_lang {
                // 언어 전환: 이전 구간 확정
                if i > current_start {
                    let text: String = chars[current_start..i].iter().collect();
                    result.push(ComposedTextRun {
                        text,
                        char_style_id: run.char_style_id,
                        lang_index: current_lang,
                        char_overlap: run.char_overlap.clone(),
                    footnote_marker: None,
                    display_text: None,                    });
                }
                current_lang = char_lang;
                current_start = i;
            }
        }

        // 마지막 구간
        let text: String = chars[current_start..].iter().collect();
        if !text.is_empty() {
            result.push(ComposedTextRun {
                text,
                char_style_id: run.char_style_id,
                lang_index: current_lang,
                char_overlap: run.char_overlap.clone(),
                    footnote_marker: None,
                    display_text: None,            });
        }
    }

    result
}

/// 언어 중립 문자인지 판별한다 (공백, ASCII 구두점, 일반 기호 등).
/// 이 문자들은 Run 분할을 유발하지 않고 이전 문자의 언어를 따른다.
pub(crate) fn is_lang_neutral(ch: char) -> bool {
    let cp = ch as u32;
    matches!(cp,
        // 공백/제어문자
        0x0000..=0x0020 |
        // ASCII 구두점/기호 (영문자/숫자 제외)
        0x0021..=0x002F | 0x003A..=0x0040 | 0x005B..=0x0060 | 0x007B..=0x007F |
        // Latin-1 Supplement 구두점 (문자 제외)
        0x00A0..=0x00BF
    )
}

/// 문단 내 인라인 컨트롤(표/도형)의 위치를 식별한다.
fn identify_inline_controls(para: &Paragraph) -> Vec<InlineControl> {
    let mut result = Vec::new();

    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
        let control_type = match ctrl {
            Control::Table(_) => InlineControlType::Table,
            Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => InlineControlType::Shape,
            Control::SectionDef(_) | Control::ColumnDef(_) => InlineControlType::Other,
            _ => continue,
        };

        // 이 컨트롤이 어느 줄에 속하는지 결정
        // 컨트롤은 문단의 controls 배열에 순서대로 저장됨
        // 정확한 줄 위치는 텍스트 내 제어 문자 위치로 결정해야 하지만,
        // 현재는 첫 번째 줄에 배치 (향후 정확한 위치 계산 가능)
        let line_index = 0;

        result.push(InlineControl {
            line_index,
            control_index: ctrl_idx,
            control_type,
        });
    }

    result
}

/// char_offsets 갭을 분석하여 각 컨트롤의 텍스트 내 삽입 위치를 결정한다.
/// → document_core::helpers::find_control_text_positions 으로 위임
fn find_control_text_positions(para: &Paragraph) -> Vec<usize> {
    crate::document_core::find_control_text_positions(para)
}

/// CharOverlap 컨트롤의 글자를 조합된 텍스트에 올바른 위치로 삽입한다.
///
/// char_offsets 갭 분석으로 각 CharOverlap의 원래 텍스트 위치를 복원하고,
/// 해당 위치의 composed line에서 기존 텍스트 런을 분할하여 CharOverlap 런을 삽입한다.
fn inject_char_overlap_text(composed: &mut ComposedParagraph, para: &Paragraph) {
    // CharOverlap 컨트롤과 인덱스 수집
    let char_overlap_indices: Vec<(usize, &crate::model::control::CharOverlap)> = para.controls.iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Control::CharOverlap(co) = c { Some((i, co)) } else { None }
        })
        .collect();

    if char_overlap_indices.is_empty() {
        return;
    }

    // 모든 컨트롤의 텍스트 위치 결정
    let control_positions = find_control_text_positions(para);

    // CharOverlap별 (텍스트위치, 런) 수집
    let mut insertions: Vec<(usize, ComposedTextRun)> = Vec::new();
    for (ctrl_idx, co) in &char_overlap_indices {
        let text: String = co.chars.iter().collect();
        if text.is_empty() { continue; }
        let char_style_id = co.char_shape_ids.iter()
            .find(|&&id| id != 0xFFFFFFFF)
            .copied()
            .unwrap_or(0);
        let text_pos = control_positions.get(*ctrl_idx).copied().unwrap_or(0);
        insertions.push((text_pos, ComposedTextRun {
            text,
            char_style_id,
            lang_index: 0,
            char_overlap: Some(CharOverlapInfo {
                border_type: co.border_type,
                inner_char_size: co.inner_char_size,
            }),
            footnote_marker: None,
            display_text: None,        }));
    }

    if insertions.is_empty() {
        return;
    }

    if composed.lines.is_empty() {
        // 빈 문단: line_segs에서 줄 정보를 가져와 새 줄 생성
        let (lh, bd, ls) = para.line_segs.first()
            .map(|s| (s.line_height, s.baseline_distance, s.line_spacing))
            .unwrap_or((400, 340, 0));
        composed.lines.push(ComposedLine {
            runs: insertions.into_iter().map(|(_, run)| run).collect(),
            line_height: lh,
            baseline_distance: bd,
            segment_width: 0,
            column_start: 0,
            line_spacing: ls,
            has_line_break: false,
            char_start: 0,
        });
        return;
    }

    // 역순으로 삽입하여 이전 인덱스가 무효화되지 않도록
    insertions.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));

    for (text_pos, overlap_run) in insertions {
        insert_overlap_run(composed, text_pos, overlap_run);
    }
}

/// 조합된 라인들에서 text_pos 위치에 CharOverlap 런을 삽입한다.
/// 기존 텍스트 런을 필요시 분할한다.
fn insert_overlap_run(
    composed: &mut ComposedParagraph,
    text_pos: usize,
    overlap_run: ComposedTextRun,
) {
    let mut char_offset = 0usize;

    for line in composed.lines.iter_mut() {
        let line_char_count: usize = line.runs.iter()
            .filter(|r| r.char_overlap.is_none())
            .map(|r| r.text.chars().count())
            .sum();

        if text_pos < char_offset + line_char_count || text_pos == char_offset {
            // 이 라인에 삽입
            let local_pos = text_pos - char_offset;
            let mut run_offset = 0usize;

            for run_idx in 0..line.runs.len() {
                // CharOverlap 런은 건너뜀 (이미 삽입된 것)
                if line.runs[run_idx].char_overlap.is_some() {
                    continue;
                }

                let run_chars = line.runs[run_idx].text.chars().count();

                if local_pos == run_offset {
                    // 런 앞에 삽입
                    line.runs.insert(run_idx, overlap_run);
                    return;
                } else if local_pos > run_offset && local_pos < run_offset + run_chars {
                    // 런 중간에 삽입: 런을 분할
                    let split_at = local_pos - run_offset;
                    let original_text: String = line.runs[run_idx].text.chars().collect();
                    let before: String = original_text.chars().take(split_at).collect();
                    let after: String = original_text.chars().skip(split_at).collect();

                    let style_id = line.runs[run_idx].char_style_id;
                    let lang_idx = line.runs[run_idx].lang_index;

                    // 기존 런을 before로 교체
                    line.runs[run_idx].text = before;

                    // after 런 생성
                    let after_run = ComposedTextRun {
                        text: after,
                        char_style_id: style_id,
                        lang_index: lang_idx,
                        char_overlap: None,
                    footnote_marker: None,
                    display_text: None,                    };

                    // overlap_run과 after_run을 삽입
                    line.runs.insert(run_idx + 1, after_run);
                    line.runs.insert(run_idx + 1, overlap_run);
                    return;
                }

                run_offset += run_chars;
            }

            // 라인 끝에 삽입
            line.runs.push(overlap_run);
            return;
        }

        char_offset += line_char_count;
    }

    // 어느 라인에도 해당하지 않으면 마지막 라인에 추가
    if let Some(last_line) = composed.lines.last_mut() {
        last_line.runs.push(overlap_run);
    }
}

/// ComposedLine의 폭을 언어 인식 측정으로 계산한다.
///
/// 각 run별로 해당 언어의 폰트/자간/장평을 적용하여 측정한다.
/// 진단 API에서 저장된 segment_width와 비교하는 데 사용한다.
pub fn estimate_composed_line_width(line: &ComposedLine, styles: &ResolvedStyleSet) -> f64 {
    line.runs.iter().map(|run| {
        let ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        estimate_text_width(effective_text_for_metrics(run), &ts)
    }).sum()
}

/// [Task #555] 폰트 매트릭스 (글자폭/줄간격) 계산용 effective text 반환.
///
/// PUA 옛한글 변환 (Task #528) 후 `run.display_text` 가 자모 시퀀스를 보유하면
/// 본 함수는 그 자모 시퀀스를 반환한다. 그렇지 않으면 `run.text` (PUA char 1글자
/// 또는 일반 텍스트) 를 그대로 반환.
///
/// 사용처: `estimate_text_width` / `estimate_composed_line_width` 등 폰트 매트릭스
/// 측정 함수의 caller. visual 출력 (svg/web_canvas) 은 이미 `display_text` 사용.
///
/// 단일 룰 (분기/허용오차 없음): 비-PUA 텍스트는 fallback 으로 동일 동작.
pub fn effective_text_for_metrics(run: &ComposedTextRun) -> &str {
    run.display_text.as_deref().unwrap_or(&run.text)
}

/// PUA Supplementary 영역(U+F0000~) 문자가 테두리 숫자인지 판별한다.
///
/// HWP 특수문자표에서 표준 Unicode가 없는 테두리 숫자를 PUA로 인코딩한다.
/// - U+F02B1~U+F02C4: map_pua_bullet_char 에서 ①~⑳ 으로 매핑 (CharOverlap 제외)
/// - U+F02CE~U+F02E1: 반전 사각형 안의 숫자 1~20 (border_type=4)
///
/// 반환: Some(border_type) 또는 None
/// PUA 문자 자체는 변환하지 않고, 렌더러(draw_char_overlap)에서 표시 문자열로 변환한다.
/// 이렇게 하면 PUA 문자가 항상 1글자로 유지되어 font_size 기반 폭 계산이 정확하다.
/// PUA 글자겹침용 숫자 컴포넌트 디코딩
///
/// HWP tcps 컨트롤의 2~3자리 숫자는 자릿수별 PUA 코드포인트로 저장된다.
/// 각 PUA 문자를 (자릿수_그룹, 숫자값) 쌍으로 디코딩한다.
///
/// 2자리 블록 (U+F0288 base):
///   십의자리: F0289~F0291 (1-9)
///   일의자리: F0292~F029B (0-9)
///
/// 3자리 블록 (U+F0490 base):
///   백의자리: F0491~F0499 (1-9)
///   십의자리: F049A~F04A3 (0-9)
///   일의자리: F04A4~F04AD (0-9)
fn pua_overlap_digit(ch: char) -> Option<(u8, u8)> {
    let cp = ch as u32;
    // 2자리 블록
    if (0xF0289..=0xF0291).contains(&cp) { return Some((0, (cp - 0xF0288) as u8)); } // tens 1-9
    if (0xF0292..=0xF029B).contains(&cp) { return Some((1, (cp - 0xF0292) as u8)); } // ones 0-9
    // 3자리 블록
    if (0xF0491..=0xF0499).contains(&cp) { return Some((0, (cp - 0xF0490) as u8)); } // hundreds 1-9
    if (0xF049A..=0xF04A3).contains(&cp) { return Some((1, (cp - 0xF049A) as u8)); } // tens 0-9
    if (0xF04A4..=0xF04AD).contains(&cp) { return Some((2, (cp - 0xF04A4) as u8)); } // ones 0-9
    None
}

/// CharOverlap의 PUA 문자 배열을 숫자 문자열로 디코딩한다.
///
/// 모든 문자가 PUA 겹침용 숫자인 경우에만 디코딩 성공 (Some).
/// 그룹 번호(0=최상위자리, 1=중간, 2=최하위)로 정렬하여 올바른 자릿수 순서를 보장한다.
pub fn decode_pua_overlap_number(chars: &[char]) -> Option<String> {
    if chars.is_empty() { return None; }
    let mut groups: Vec<(u8, u8)> = Vec::with_capacity(chars.len());
    for &ch in chars {
        groups.push(pua_overlap_digit(ch)?);
    }
    // 그룹 번호 순 정렬 (최상위 자리 → 최하위 자리)
    groups.sort_by_key(|(g, _)| *g);
    let s: String = groups.iter().map(|(_, d)| char::from(b'0' + d)).collect();
    Some(s)
}

fn pua_enclosed_border_type(ch: char) -> Option<u8> {
    let cp = ch as u32;
    // U+F02B1~F02C4 (①~⑳): map_pua_bullet_char 에서 표준 원문자로 매핑 — CharOverlap 제외
    // 반전 사각형 안의 숫자: U+F02CE(1) ~ U+F02E1(20)
    if (0xF02CE..=0xF02E1).contains(&cp) {
        return Some(4); // border_type=4: 반전 사각형
    }
    None
}

/// PUA 테두리 숫자 문자를 표시 문자열로 변환한다. (렌더러 전용)
///
/// draw_char_overlap()에서 호출하여, 실제 렌더링 시에만 변환한다.
pub fn pua_to_display_text(ch: char) -> Option<String> {
    let cp = ch as u32;
    // U+F02B1~F02C4 는 map_pua_bullet_char 에서 ①~⑳ 으로 매핑 — 여기 도달 불가
    // 반전 사각형 안의 숫자: U+F02CE(1) ~ U+F02E1(20)
    if (0xF02CE..=0xF02E1).contains(&cp) {
        let num = cp - 0xF02CD;
        return Some(format!("{}", num));
    }
    None
}

/// 조합된 텍스트 런에서 PUA 테두리 숫자 문자를 찾아 CharOverlap 런으로 변환한다.
///
/// PUA 문자는 원본 그대로 유지하되 CharOverlapInfo만 부착한다.
/// 이렇게 하면 PUA 문자가 항상 1글자로 유지되어:
/// - reflow_line_segs()의 텍스트 측정과 레이아웃 폭 계산이 일치
/// - 두 자리 숫자(10~20)도 1글자 = 1박스 = font_size 폭
///   실제 표시 문자열(PUA → "1", "10" 등) 변환은 draw_char_overlap()에서 수행한다.
fn convert_pua_enclosed_numbers(composed: &mut ComposedParagraph) {
    for line in composed.lines.iter_mut() {
        let mut new_runs: Vec<ComposedTextRun> = Vec::new();
        let mut changed = false;

        for run in line.runs.iter() {
            // 이미 CharOverlap인 런은 그대로 유지
            if run.char_overlap.is_some() {
                new_runs.push(run.clone());
                continue;
            }

            // PUA 테두리 숫자 문자가 있는지 확인
            let has_pua = run.text.chars().any(|ch| pua_enclosed_border_type(ch).is_some());
            if !has_pua {
                new_runs.push(run.clone());
                continue;
            }

            changed = true;
            let mut buf = String::new();

            for ch in run.text.chars() {
                if let Some(border_type) = pua_enclosed_border_type(ch) {
                    // buf에 쌓인 일반 텍스트를 먼저 런으로 추가
                    if !buf.is_empty() {
                        new_runs.push(ComposedTextRun {
                            text: buf.clone(),
                            char_style_id: run.char_style_id,
                            lang_index: run.lang_index,
                            char_overlap: None,
                    footnote_marker: None,
                    display_text: None,                        });
                        buf.clear();
                    }
                    // PUA 문자 그대로 유지 + CharOverlapInfo 부착
                    new_runs.push(ComposedTextRun {
                        text: ch.to_string(),
                        char_style_id: run.char_style_id,
                        lang_index: run.lang_index,
                        char_overlap: Some(CharOverlapInfo {
                            border_type,
                            inner_char_size: 0,
                        }),
                        footnote_marker: None,
                        display_text: None,                    });
                } else {
                    buf.push(ch);
                }
            }

            // 남은 일반 텍스트
            if !buf.is_empty() {
                new_runs.push(ComposedTextRun {
                    text: buf,
                    char_style_id: run.char_style_id,
                    lang_index: run.lang_index,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,                });
            }
        }

        if changed {
            line.runs = new_runs;
        }
    }
}

mod line_breaking;
pub mod lineseg_compare;

pub(crate) use line_breaking::{reflow_line_segs, recalculate_section_vpos, is_line_start_forbidden, is_line_end_forbidden, tokenize_paragraph, BreakToken};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod lineseg_compare_tests;
#[cfg(test)]
mod re_sample_gen;
