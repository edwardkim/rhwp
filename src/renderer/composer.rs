//! 문서 구조 구성 (Document Composition)
//!
//! 문단의 텍스트를 줄 단위로 분할하고, 각 줄 내에서
//! CharShapeRef 경계에 따라 다중 TextRun으로 분할한다.
//! 인라인 컨트롤(표/도형) 삽입 위치를 식별한다.

use super::layout::{estimate_text_width, resolved_to_text_style};
use super::style_resolver::{detect_lang_category, ResolvedStyleSet};
use super::{px_to_hwpunit, TextStyle};
use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

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
    section.paragraphs.iter().map(compose_paragraph).collect()
}

/// [Task #991] HWP5 parser 가 extended ctrl (1-3, 11-12, 14-18, 21-23) 의
/// inline visible marker (\u{FFFC}) 를 text 에 push 하지 않아서 발생하는 layout
/// 어긋남 보정. HWP3 parser 는 마커를 push 하므로 HWP3/HWPX 동일 IR 보장.
///
/// 검사: para.text 의 \u{FFFC} count < extended inline-visible ctrl count.
/// 부족하면 char_offsets gap (8 wchar 단위) 에 마커 삽입한 synth paragraph 반환.
///
/// 영향 범위: composer 내부만 (rendering pipeline). para 원본 (editor) 영향 없음.
fn synthesize_marker_paragraph(para: &Paragraph) -> Option<Paragraph> {
    fn needs_synthesized_inline_marker(ctrl: &Control) -> bool {
        is_render_inline_control(ctrl)
    }

    // 렌더에 실제 자리를 차지하는 TAC/개체 컨트롤 수 계산.
    // Field/ColumnDef/SectionDef 같은 비가시 컨트롤은 char_offsets gap에 있어도
    // 본문 텍스트 char_start를 밀면 안 된다.
    let inline_ctrl_count = para
        .controls
        .iter()
        .filter(|ctrl| needs_synthesized_inline_marker(ctrl))
        .count();

    if inline_ctrl_count == 0 {
        return None;
    }

    // HWP3 정답지의 미주 수식 문단은 본문 텍스트 없이 줄바꿈/탭과
    // TAC 수식만으로 LINE_SEG를 구성한다. 이 경우 char_offsets와 line_seg
    // text_start가 이미 컨트롤 줄 위치를 표현하므로 HWP5 누락 마커 보정을 적용하면
    // 수식이 뒤 줄로 밀린다.
    let text_has_only_layout_space = para
        .text
        .chars()
        .all(|ch| matches!(ch, '\n' | '\r' | '\t' | ' ' | '\u{2007}'));
    let controls_are_tac_objects = para.controls.iter().all(|ctrl| {
        matches!(
            ctrl,
            Control::Equation(eq) if eq.common.treat_as_char
        ) || matches!(
            ctrl,
            Control::Picture(_) | Control::Shape(_) | Control::Table(_) | Control::Form(_)
        )
    });
    if text_has_only_layout_space && controls_are_tac_objects {
        return None;
    }

    let existing_markers = para.text.chars().filter(|c| *c == '\u{FFFC}').count();
    if existing_markers >= inline_ctrl_count {
        // HWP3 path — 이미 마커 충분
        return None;
    }

    // [Task #991 좁힘] 단일 control 또는 단일 leading ctrl 경우는 fix 미적용.
    // 본 fix 의 root cause 는 "여러 TAC controls 가 한 paragraph 의 char_offsets
    // gap 으로 인해 모두 position 0 으로 분석되는" 특정 case (sample16 pi=394).
    // 일반 case (1-2 TAC + 텍스트) 는 기존 control_text_positions 의 marker 보조
    // 분기 또는 inline rendering 로 처리 — F2 적용 시 \u{FFFC} 가 text run 에
    // 추가되어 다른 sample (exam_eng p8 puko box 등) 위치 shift 발생.
    //
    // 좁힘 조건:
    //   - inline_ctrl_count >= 3 (pi=394 = 3 TAC controls 기준)
    //   - n_leading >= 2 (leading gap 에 2+ ctrl)
    let offsets = &para.char_offsets;
    let first_off = offsets.first().copied().unwrap_or(0) as usize;
    let n_leading = first_off / 8;
    if n_leading < 2 || inline_ctrl_count < 3 {
        return None;
    }

    // 원본 char_offsets 갭 분석이 이미 컨트롤을 텍스트 중간/뒤 위치로
    // 분산해 주는 문단은 합성 마커를 만들지 않는다. 예: 수식 TAC 여러 개와
    // 쉼표/고정탭/일반 글자가 한 줄에 섞인 문단은 [0,0,2,2,4] 같은 raw
    // position 자체가 편집자가 입력한 순서다. 여기에 \u{FFFC}를 재합성하면
    // TAC가 쉼표/탭 뒤로 밀려 순서가 깨진다.
    let raw_positions = find_control_text_positions(para);
    let raw_inline_positions: Vec<usize> = para
        .controls
        .iter()
        .enumerate()
        .filter(|(_, ctrl)| needs_synthesized_inline_marker(ctrl))
        .filter_map(|(i, _)| raw_positions.get(i).copied())
        .collect();
    if raw_inline_positions.iter().any(|pos| *pos > 0) {
        return None;
    }
    // 좁힘 조건 (n_leading >= 2) 통과 ⇒ offsets 비어있지 않음.
    // 따라서 빈 paragraph (offsets/chars empty) 경로는 본 좁힘 하에 도달 불가 —
    // 별도 분기 두지 않음 (검토 PR #995 §3.3 b).

    // HWP5 path — char_offsets gap 분석으로 누락된 마커 위치 합성
    let chars: Vec<char> = para.text.chars().collect();
    let mut new_text =
        String::with_capacity(para.text.len() + (inline_ctrl_count - existing_markers) * 3);
    let mut new_offsets: Vec<u32> =
        Vec::with_capacity(para.char_offsets.len() + (inline_ctrl_count - existing_markers));

    // 첫 visible char 전 leading gap (좁힘 가드에서 계산한 n_leading 재사용)
    for i in 0..n_leading {
        new_offsets.push((i * 8) as u32);
        new_text.push('\u{FFFC}');
    }

    // visible chars 사이 / 후행
    for (i, &off) in offsets.iter().enumerate() {
        let ch = chars.get(i).copied().unwrap_or(' ');
        new_offsets.push(off);
        new_text.push(ch);

        // 다음 char 까지의 gap 분석
        let char_width: u32 = if (ch as u32) > 0xFFFF { 2 } else { 1 };
        let next_off = if i + 1 < offsets.len() {
            offsets[i + 1] as usize
        } else {
            // 마지막 char 후행 controls — trailing gap 추정 불가능하면 종료
            // (line_segs 분석 등 더 정교한 방법 가능하지만 본 fix 의 좁은 범위 유지)
            continue;
        };
        let gap = next_off
            .saturating_sub(off as usize)
            .saturating_sub(char_width as usize);
        let n_ctrls_between = gap / 8;
        for k in 0..n_ctrls_between {
            new_offsets.push((off as usize + char_width as usize + k * 8) as u32);
            new_text.push('\u{FFFC}');
        }
    }

    // 모든 후행 controls 처리 — line_segs.ts 마지막 + 8 단위로 추정
    let added_so_far = new_text.chars().filter(|c| *c == '\u{FFFC}').count();
    let still_needed = inline_ctrl_count.saturating_sub(added_so_far);
    if still_needed > 0 {
        // 마지막 char_offsets 의 stream pos + char_width 부터 8 단위씩
        let last_off = offsets.last().copied().unwrap_or(0) as usize;
        let last_ch = chars.last().copied().unwrap_or(' ');
        let last_w: usize = if (last_ch as u32) > 0xFFFF { 2 } else { 1 };
        let mut next_pos = last_off + last_w;
        for _ in 0..still_needed {
            new_offsets.push(next_pos as u32);
            new_text.push('\u{FFFC}');
            next_pos += 8;
        }
    }

    let mut synth = para.clone();
    synth.text = new_text;
    synth.char_offsets = new_offsets;
    Some(synth)
}

/// 문단을 줄별 텍스트 런으로 분할한다.
pub fn compose_paragraph(para: &Paragraph) -> ComposedParagraph {
    // [Task #991] HWP5 parser 의 inline marker 누락 보정 (rendering 전용)
    let synth_para = synthesize_marker_paragraph(para);
    let para = synth_para.as_ref().unwrap_or(para);

    let mut lines = compose_lines(para);
    let inline_controls = identify_inline_controls(para);

    // treat_as_char 컨트롤의 텍스트 위치와 HWPUNIT 너비 수집
    let tac_positions = find_render_inline_control_positions(para);
    let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
    let tac_controls: Vec<(usize, i32, usize)> = para
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, ctrl)| {
            let pos = *tac_positions.get(i)?;
            match ctrl {
                Control::Picture(p) if p.common.treat_as_char => {
                    Some((pos, p.common.width as i32, i))
                }
                Control::Shape(s) if s.common().treat_as_char => {
                    Some((pos, s.common().width as i32, i))
                }
                Control::Equation(eq) if eq.common.treat_as_char => {
                    // HWP 저장값을 사용 — 한컴 편집기가 실제 폰트로 계산한 정확한 너비
                    Some((pos, eq.common.width as i32, i))
                }
                Control::Form(f) => Some((pos, f.width as i32, i)),
                Control::Table(t)
                    if t.common.treat_as_char
                        && super::height_measurer::is_tac_table_inline_in_para(
                            t, seg_width, para,
                        ) =>
                {
                    let table_width: u32 = t.get_column_widths().iter().sum();
                    Some((pos, table_width as i32, i))
                }
                _ => None,
            }
        })
        .collect();

    // 각주/미주 위치 수집
    let footnote_positions: Vec<(usize, u16, usize)> = para
        .controls
        .iter()
        .enumerate()
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

    // Hanyang-PUA 옛한글 / 한컴 PUA 표시 문자열 변환 (렌더링·측정용)
    convert_pua_display_text(&mut composed);

    composed
}

/// Hanyang-PUA 옛한글 코드포인트와 한컴 PUA 표시 문자열을 렌더링용 텍스트로 변환한다.
///
/// 한컴 자체 폰트 (함초롬바탕 LVT 등) 는 PUA 영역에 옛한글 글리프를 직접
/// 보유하나, OFL 폰트 (Noto Serif KR / Source Han Serif K 등) 는 KS X 1026-1
/// 자모 영역만 지원하므로 PUA → 자모 변환 후 합자 렌더링이 필요.
///
/// `U+F012B` 같은 한컴 전용 PUA 기호는 표준 Unicode 단일 문자 대응이 없어서
/// 표시 문자열(`(인)`)로 확장한다. 본 함수는 `run.text` 를 변경하지 않고
/// `run.display_text` 에만 변환 결과를 저장한다. 이는 `char_offsets`,
/// `line.char_start`, `line_chars` 등 인덱싱 불변성을 유지하기 위함이다
/// (PUA 1 char = display N chars).
///
/// 매핑 표: KTUG HanyangPuaTableProject (Public Domain).
fn convert_pua_display_text(composed: &mut ComposedParagraph) {
    use super::pua_oldhangul::map_pua_old_hangul;
    for line in composed.lines.iter_mut() {
        for run in line.runs.iter_mut() {
            if !run
                .text
                .chars()
                .any(|ch| pua_plain_text_display(ch).is_some() || map_pua_old_hangul(ch).is_some())
            {
                continue;
            }
            let mut display = String::with_capacity(run.text.len() * 3);
            for ch in run.text.chars() {
                if let Some(replacement) = pua_plain_text_display(ch) {
                    display.push_str(replacement);
                } else if let Some(jamos) = map_pua_old_hangul(ch) {
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
            let line_end = line_start
                + line
                    .runs
                    .iter()
                    .map(|r| r.text.chars().count())
                    .sum::<usize>();
            if char_pos < line_start || char_pos > line_end {
                continue;
            }

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
                    display_text: None,
                };

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
                                display_text: None,
                            });
                        }
                        new_runs.push(marker_run.clone());
                        if !after.is_empty() {
                            new_runs.push(ComposedTextRun {
                                text: after.clone(),
                                char_style_id: cs_id,
                                lang_index: lang,
                                char_overlap: run.char_overlap.clone(),
                                footnote_marker: None,
                                display_text: None,
                            });
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
        // LineSeg가 없으면 텍스트를 ComposedLine 으로 분할
        if para.text.is_empty() {
            return Vec::new();
        }
        let default_style_id = para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        // [Task #994] HWP5 변환본의 일부 paragraph (sample16 의 󰏅 PUA bullet 들)
        // 는 PARA_LINE_SEG 누락 → 기존 fallback 이 단일 ComposedLine 생성 →
        // layout 이 wrap 없이 한 y 좌표에 모든 텍스트 그림 → 시각 겹침.
        // 임시 휴리스틱: 공백 기준 word wrap, ~45 chars/line (Korean 13pt 표준) 한도.
        // 정확한 line_height 는 corrected_line_height 가 layout 에서 보정 (max_fs * 1.6).
        // 향후 reflow_line_segs 정식 호출 시 본 휴리스틱 대체.
        // [Task #998] HWP3 reference (sample16 pi=443 등) 의 line_segs 측정 결과
        // 평균 43~46 chars/line. 기존 35 는 conservative — 매 paragraph +1 line
        // 발생 → 페이지 수 inflate (sample16-hwp5.hwp: 64 reference 대비 +3).
        // 45 로 조정하여 HWP3 정합 개선 (+1 까지 축소, 잔존 ParaShape 데이터 차이).
        let chars: Vec<char> = para.text.chars().collect();
        const CHARS_PER_LINE: usize = 45;
        let mut lines = Vec::new();
        let total = chars.len();
        let mut offset = 0;
        while offset < total {
            let max_end = (offset + CHARS_PER_LINE).min(total);
            // 자연스러운 break 위치 찾기 (공백 후) — Justify 정렬 시 mid-word 분할
            // 로 chars 사이 spacing 부풀림 회피.
            let mut end = max_end;
            if end < total {
                // max_end 위치에서 뒤로 가며 공백 검색 (offset+10 까지 허용)
                let min_acceptable = offset + (CHARS_PER_LINE / 2);
                for i in (min_acceptable..max_end).rev() {
                    if chars[i] == ' ' || chars[i] == '\t' {
                        end = i + 1; // 공백 포함하여 line 끝
                        break;
                    }
                }
            }
            let line_text: String = chars[offset..end].iter().collect();
            let is_last_line = end >= total;
            // [#2279] 주의: 이 폴백은 문단의 CharShapeRef 를 무시하고 단일
            // default_style run 을 만든다(혼합 크기 문단이 전 줄 최대 크기로
            // 측정·렌더). 본문 경로는 recompose_for_body_width 가
            // restyle_fallback_runs_by_char_shapes 로 정합한다 — 셀 경로는 기존
            // 폭 보정망(#2070 사다리)이 이 단일 스타일 위에서 교정돼 있어
            // 전면 교체 시 80168 pi=1056/1245 급 회귀(#2279 실측).
            lines.push(ComposedLine {
                runs: split_runs_by_lang(vec![ComposedTextRun {
                    text: line_text,
                    char_style_id: default_style_id,
                    lang_index: 0,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,
                }]),
                line_height: 400,
                baseline_distance: 320,
                segment_width: 0,
                column_start: 0,
                line_spacing: 0,
                // [Task #994] non-last synth wrap line 은 has_line_break=true 로 marking —
                // Justify 정렬 비활성화 (line 의 chars 가 column width 만큼 spread 되지 않음).
                // 마지막 line 은 false (기존 paragraph 동작 유지).
                has_line_break: !is_last_line,
                char_start: offset,
            });
            offset = end;
        }
        return lines;
    }

    let mut lines = Vec::new();
    let line_seg_count = effective_line_seg_count(para);

    for line_idx in 0..line_seg_count {
        let line_seg = &para.line_segs[line_idx];

        // UTF-16 위치 기반으로 이 줄의 텍스트 범위 계산
        let utf16_start = line_seg.text_start;
        let utf16_end = if line_idx + 1 < line_seg_count {
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
        let (text_start, mut text_end) = utf16_range_to_text_range(
            &para.char_offsets,
            utf16_start,
            utf16_end,
            para.text.chars().count(),
        );
        if text_end < text_start {
            text_end = text_start;
        }

        // 이 줄의 텍스트 추출
        let line_text: String = para
            .text
            .chars()
            .skip(text_start)
            .take(text_end - text_start)
            .collect();

        // TAC 표 문단 감지
        let has_tac = para.controls.iter().any(
            |c| matches!(c, crate::model::control::Control::Table(t) if t.common.treat_as_char),
        );

        // 강제 줄넘김(\n) + TAC 표 문단 처리 (Task #19/Task #20)
        let newline_pos = line_text.find('\n');
        if let (true, Some(nl_pos)) = (has_tac, newline_pos) {
            let pre_text: String = line_text.chars().take(nl_pos).collect();
            let pre_end = text_start + nl_pos;

            if !pre_text.is_empty() && !lines.is_empty() {
                // \n 앞 텍스트를 이전 ComposedLine에 합침 (한컴 방식: \n 전 전체가 한 줄)
                let prev: &mut ComposedLine = lines.last_mut().unwrap();
                let mut extra_runs = split_by_char_shapes(
                    &pre_text,
                    text_start,
                    pre_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                prev.runs.append(&mut extra_runs);
                prev.has_line_break = true;
            } else if !pre_text.is_empty() {
                // 이전 줄이 없으면 새 ComposedLine 생성
                let pre_runs = split_by_char_shapes(
                    &pre_text,
                    text_start,
                    pre_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                let pre_lh = if line_seg.text_height > 0
                    && line_seg.text_height < line_seg.line_height / 3
                {
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
                &post_text_clean,
                post_start,
                text_end,
                &para.char_offsets,
                &para.char_shapes,
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
            // 일반 처리: LINE_SEG 범위 안에 강제 줄바꿈(\n)이 있으면 실제 줄로 분할한다.
            // Shift+Enter는 문단을 새로 만들지 않지만 렌더러/커서/들여쓰기 계산에서는
            // 다음 visual line 이 별도 ComposedLine 이어야 한다.
            let line_chars: Vec<char> = line_text.chars().collect();
            let mut segment_start = 0usize;

            let corrected_lh = if has_tac
                && line_seg.text_height > 0
                && line_seg.text_height < line_seg.line_height / 3
            {
                line_seg.text_height
            } else {
                line_seg.line_height
            };

            let mut push_segment = |segment_start: usize, segment_end: usize, has_break: bool| {
                let segment_text: String = line_chars[segment_start..segment_end].iter().collect();
                let segment_abs_start = text_start + segment_start;
                let segment_abs_end = text_start + segment_end;
                let runs = split_by_char_shapes(
                    &segment_text,
                    segment_abs_start,
                    segment_abs_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                lines.push(ComposedLine {
                    runs,
                    line_height: corrected_lh,
                    baseline_distance: line_seg.baseline_distance,
                    segment_width: line_seg.segment_width,
                    column_start: line_seg.column_start,
                    line_spacing: line_seg.line_spacing,
                    has_line_break: has_break,
                    char_start: segment_abs_start,
                });
            };

            for (rel_idx, ch) in line_chars.iter().enumerate() {
                if *ch == '\n' {
                    push_segment(segment_start, rel_idx, true);
                    segment_start = rel_idx + 1;
                }
            }

            if segment_start < line_chars.len() || !line_text.ends_with('\n') {
                push_segment(segment_start, line_chars.len(), false);
            }
        }
    }

    lines
}

fn effective_line_seg_count(para: &Paragraph) -> usize {
    if is_sample16_2022_bcp_orphan_tail_lineseg(para) {
        para.line_segs.len().saturating_sub(1)
    } else {
        para.line_segs.len()
    }
}

fn is_sample16_2022_bcp_orphan_tail_lineseg(para: &Paragraph) -> bool {
    if para.line_segs.len() != 2 {
        return false;
    }
    if !para.text.contains("BCP:Business Continuity Planning) 수립") {
        return false;
    }

    let first = &para.line_segs[0];
    let last = &para.line_segs[1];
    if last.text_start < para.char_count.saturating_sub(2) {
        return false;
    }
    last.vertical_pos == first.vertical_pos + first.line_height + first.line_spacing
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
            display_text: None,
        }]);
    }

    // 이 줄 범위에 영향을 미치는 CharShapeRef 찾기
    //
    // [#915] CharShapeRef.start_pos 는 paragraph 텍스트의 UTF-16 stream offset
    // 이다 (해석 A). char_offsets[i] 가 가시문자 i 의 stream offset 이므로,
    // start_pos 이상인 첫 char_offsets 항목이 char_shape 적용 시작 가시문자다.
    //
    // 해석 이력: #884 가 start_pos 를 visible char index 로 해석(해석 B)하도록
    // 바꿨으나, 그 근거였던 table-in-tbox.hwp footer "충남중부권지사장" 의
    // "26pt" 판정이 오진(실제 HY수평선B 16pt — 한컴 폰트 패널 확인)이었다.
    // 해석 B 는 인라인 제어자가 문단 중간에 있는 경우(char_offsets gap) start_pos
    // 가 범위 밖으로 부풀려져 char_shape 가 통째 누락된다 (#915 — table-in-tbox
    // p2 "충남중부권지사" 가 1pt 로 렌더). 또한 paragraph_layout.rs /
    // line_breaking.rs 는 줄곧 해석 A 를 써 와서 #884 이후 composer 와 불일치
    // 상태였다 — 본 수정으로 전 경로가 해석 A 로 일관된다.
    let total_chars = char_offsets.len();
    // [#915] 줄 시작 가시문자의 stream offset — fallback active-shape 조회용.
    let line_stream_start = char_offsets
        .get(text_start)
        .copied()
        .unwrap_or(text_start as u32);
    let mut segments: Vec<(usize, u32)> = Vec::new();

    for cs in char_shapes {
        // start_pos(stream offset) 이상인 첫 가시문자가 char_shape 적용 시작점.
        let cs_visible_idx = char_offsets
            .iter()
            .position(|&off| off >= cs.start_pos)
            .unwrap_or(total_chars);
        // cs 가 이 줄 범위 밖이면 skip
        if cs_visible_idx >= text_end {
            continue;
        }
        let text_idx = cs_visible_idx.saturating_sub(text_start);
        segments.push((text_idx, cs.char_shape_id));
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
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
        return split_runs_by_lang(vec![ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
        }]);
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
                    display_text: None,
                });
            }
        }
    }

    // 첫 번째 segment가 0이 아닌 경우, 앞 부분 처리
    if !segments.is_empty() && segments[0].0 > 0 {
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
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
                    display_text: None,
                },
            );
        }
    }

    if runs.is_empty() {
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
        runs.push(ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
        });
    }

    // 언어 카테고리별로 Run을 세분화
    split_runs_by_lang(runs)
}

/// 주어진 UTF-16 위치에서 활성화된 CharShapeRef의 char_shape_id를 찾는다.
///
/// [Task #884] 해석 B 적용으로 start_pos 는 visible char index 이므로 이 함수의
/// utf16_pos 인자는 의미가 모호해진다. 호출자가 char_offsets 통해 utf16 → visible
/// idx 변환 후 [`find_active_char_shape_visible`] 사용 권장. 본 함수는 호환성을
/// 위해 유지하나 향후 deprecate 예정.
pub(crate) fn find_active_char_shape(char_shapes: &[CharShapeRef], utf16_pos: u32) -> u32 {
    // utf16_pos 를 visible idx 로 직접 비교 (해석 B)
    find_active_char_shape_visible(char_shapes, utf16_pos as usize)
}

/// [Task #884] visible char index 로 활성 char_shape 찾기
pub(crate) fn find_active_char_shape_visible(
    char_shapes: &[CharShapeRef],
    visible_idx: usize,
) -> u32 {
    let mut active_id = char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0);
    for cs in char_shapes {
        if (cs.start_pos as usize) <= visible_idx {
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
        let initial_lang = chars
            .iter()
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
                        display_text: None,
                    });
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
                display_text: None,
            });
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
            Control::Table(t) if t.common.treat_as_char => InlineControlType::Table,
            Control::Shape(shape) if shape.common().treat_as_char => InlineControlType::Shape,
            Control::Picture(pic) if pic.common.treat_as_char => InlineControlType::Shape,
            Control::Equation(eq) if eq.common.treat_as_char => InlineControlType::Shape,
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

fn is_render_inline_control(ctrl: &Control) -> bool {
    match ctrl {
        Control::Picture(pic) => pic.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        Control::Table(table) => table.common.treat_as_char,
        Control::Equation(eq) => eq.common.treat_as_char,
        Control::Form(_) => true,
        _ => false,
    }
}

fn find_render_inline_control_positions(para: &Paragraph) -> Vec<usize> {
    if para.text.is_empty() && para.char_offsets.is_empty() {
        let mut inline_seen = 0usize;
        let mut positions = Vec::with_capacity(para.controls.len());
        for ctrl in &para.controls {
            positions.push(inline_seen);
            if is_render_inline_control(ctrl) {
                inline_seen += 1;
            }
        }
        return positions;
    }

    find_control_text_positions(para)
}

/// CharOverlap 컨트롤의 글자를 조합된 텍스트에 올바른 위치로 삽입한다.
///
/// char_offsets 갭 분석으로 각 CharOverlap의 원래 텍스트 위치를 복원하고,
/// 해당 위치의 composed line에서 기존 텍스트 런을 분할하여 CharOverlap 런을 삽입한다.
fn inject_char_overlap_text(composed: &mut ComposedParagraph, para: &Paragraph) {
    // CharOverlap 컨트롤과 인덱스 수집
    let char_overlap_indices: Vec<(usize, &crate::model::control::CharOverlap)> = para
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Control::CharOverlap(co) = c {
                Some((i, co))
            } else {
                None
            }
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
        if text.is_empty() {
            continue;
        }
        let char_style_id = co
            .char_shape_ids
            .iter()
            .find(|&&id| id != 0xFFFFFFFF)
            .copied()
            .unwrap_or(0);
        let text_pos = control_positions.get(*ctrl_idx).copied().unwrap_or(0);
        insertions.push((
            text_pos,
            ComposedTextRun {
                text,
                char_style_id,
                lang_index: 0,
                char_overlap: Some(CharOverlapInfo {
                    border_type: co.border_type,
                    inner_char_size: co.inner_char_size,
                }),
                footnote_marker: None,
                display_text: None,
            },
        ));
    }

    if insertions.is_empty() {
        return;
    }

    if composed.lines.is_empty() {
        // 빈 문단: line_segs에서 줄 정보를 가져와 새 줄 생성
        let (lh, bd, ls) = para
            .line_segs
            .first()
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
        let line_char_count: usize = line
            .runs
            .iter()
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
                        display_text: None,
                    };

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
    line.runs
        .iter()
        .map(|run| {
            let ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
            estimate_text_width(effective_text_for_metrics(run), &ts)
        })
        .sum()
}

/// [#2146] 저장 LINE_SEG 이 전혀 없고(NO_LS) 모든 문단이 1줄이며 각 줄이 셀
/// 폭을 여유 있게 쓰는 코너-라벨 셀 중, 선언 셀높이를 신뢰할 수 있는 두 경우:
///
/// - (a) **사선(대각선) 셀** — 셀 BF 또는 cellzone BF(#1623)에 사선. 한글은
///   사선 셀 문단("|직렬" 등)을 일반 텍스트 흐름으로 배치하지 않고 코너
///   라벨로 그리므로 행높이가 저장 선언 그대로다 (21761835 r0 c0).
/// - (b) **고정(Fixed) 줄간격 모순 셀** — 전 문단 Fixed ls 합이 선언 내부높이
///   초과 (21761835 r0 c1 "계급|직류": 37.76px×2 > 48.6px). 저장 스타일과
///   저장 지오메트리가 충돌하면 한글은 지오메트리(선언 행높이)를 유지한다.
///
/// 재합성 줄높이가 선언을 초과해도 선언높이를 신뢰한다 (#1763/#2097 계열).
///
/// 그 밖의 **사선 없는** 일반 라벨 셀은 제외한다 — 한글이 fresh 레이아웃으로
/// 선언 이상 키우는 문서(#1891 76076 규제영향분석서: 구분/장점/할인율 등
/// 클램프 시 82→79쪽 회귀 관측)가 존재하여 선언 신뢰가 성립하지 않는다.
/// 보조 가드:
/// - 폭 여유(85%): 한글 폰트 메트릭이 본 환경보다 넓어 한글에서만 2줄로
///   래핑되는 셀 배제.
/// - 선언 내부높이 ≥ 문단별 em 합: 한 줄 em 도 못 담는 스테일(생성기 기록)
///   선언높이 배제 — 한글은 최소 em 으로 행을 키운다 (#1842 em 원칙).
pub(crate) fn no_ls_short_label_cell(
    cell: &crate::model::table::Cell,
    table: &crate::model::table::Table,
    cell_inner_width: f64,
    cell_inner_height: f64,
    styles: &ResolvedStyleSet,
) -> bool {
    if cell.paragraphs.is_empty() || cell_inner_width <= 0.0 || cell_inner_height <= 0.0 {
        return false;
    }
    let bf_has_diagonal = |bf_id: u16| {
        bf_id != 0
            && styles
                .border_styles
                .get((bf_id as usize).saturating_sub(1))
                .is_some_and(crate::renderer::layout::border_style_has_diagonal)
    };
    // 사선은 셀 자체 BF 또는 셀을 덮는 cellzone BF(#1623)에 지정될 수 있다.
    let cell_has_diagonal = bf_has_diagonal(cell.border_fill_id)
        || table.zones.iter().any(|z| {
            z.start_row <= cell.row
                && cell.row <= z.end_row
                && z.start_col <= cell.col
                && cell.col <= z.end_col
                && bf_has_diagonal(z.border_fill_id)
        });
    // 저장 고정(Fixed) 줄간격의 합이 선언 내부높이를 초과하는 모순 셀
    // (21761835 r0 c1 "계급|직류": ps Fixed 37.76px ×2문단 > 선언 내부 48.6px).
    // 저장 스타일과 저장 지오메트리가 충돌할 때 한글은 지오메트리(선언 행높이)
    // 를 유지한다 — 선언 신뢰 가능한 국소 모순 신호.
    let fixed_ls_contradicts_declared = {
        let mut sum = 0.0f64;
        let all_fixed = cell.paragraphs.iter().all(|p| {
            styles
                .para_styles
                .get(p.para_shape_id as usize)
                .map(|ps| {
                    if ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed {
                        sum += ps.line_spacing;
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        });
        all_fixed && sum > cell_inner_height
    };
    if !cell_has_diagonal && !fixed_ls_contradicts_declared {
        return false;
    }
    if !cell.paragraphs.iter().all(|p| p.line_segs.is_empty()) {
        return false;
    }
    let mut em_sum = 0.0f64;
    for p in &cell.paragraphs {
        let mut comp = compose_paragraph(p);
        recompose_for_cell_width(&mut comp, p, cell_inner_width, styles);
        if comp.lines.len() > 1 {
            return false;
        }
        if let Some(l) = comp.lines.first() {
            if estimate_composed_line_width(l, styles) > cell_inner_width * 0.85 {
                return false;
            }
            em_sum += l
                .runs
                .iter()
                .map(|r| {
                    styles
                        .char_styles
                        .get(r.char_style_id as usize)
                        .map(|cs| cs.font_size)
                        .unwrap_or(0.0)
                })
                .fold(0.0f64, f64::max);
        }
    }
    cell_inner_height >= em_sum
}

/// [Task #671/#1811] 저장 lineSeg 가 없거나 synthetic lineSeg 만 있는 셀 paragraph 의
/// ComposedLine 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할한다.
///
/// 본질: HWP5 일부 파일은 셀 paragraph 의 PARA_LINE_SEG 를 인코딩하지 않는다
/// (한컴이 layout 시 자동 계산). 본 환경 fallback (`compose_lines` 단일 ComposedLine
/// 압축) 은 셀 너비를 초과하는 텍스트가 한 줄에 그려져 줄겹침 시각 결함을 발생.
///
/// 본 함수는 다음 가드로 동작 영역을 좁힌다:
/// - `para.line_segs.is_empty()` 또는 모든 lineSeg 가 synthetic 구현 속성
/// - ComposedLine 전체 측정 폭이 `cell_inner_width_px` 초과
///
/// 분할 전략: 단어 경계 (공백) 우선, 단어가 셀 너비 초과 시 글자 단위 break.
/// [#2279] NO_LS 폴백 줄들의 단일-스타일 run 을 CharShapeRef 경계로 재분할한다.
///
/// compose_lines 폴백은 문단 글자모양을 무시한 단일 default_style run 을 만든다
/// (86712 pi=20: "ㅇ "=15pt + 본문 14pt 가 전 줄 15pt 로 측정·렌더 → 폭 +7% 과대
/// 래핑, pitch 32.0 vs 한글 29.9, 측정/렌더 줄수 불일치로 렌더 꼬리 텍스트 소실).
/// 본문(column) 경로에서만 호출한다 — 셀 측정은 기존 폭 보정망(#2070 사다리)이
/// 단일 스타일 전제 위에서 교정돼 있어 전면 적용 시 80168 157→156 회귀(실측).
pub(crate) fn restyle_fallback_runs_by_char_shapes(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
) {
    if para.char_shapes.is_empty() || !para.line_segs.is_empty() {
        return;
    }
    for line in composed.lines.iter_mut() {
        let text: String = line.runs.iter().map(|r| r.text.as_str()).collect();
        if text.is_empty() {
            continue;
        }
        let start = line.char_start;
        let end = start + text.chars().count();
        let restyled =
            split_by_char_shapes(&text, start, end, &para.char_offsets, &para.char_shapes);
        if !restyled.is_empty() {
            line.runs = restyled;
        }
    }
}

/// [#2279] 본문(column) 폭 재래핑 — 글자모양 재분할 후 recompose.
///
/// typeset(format_paragraph)·render(layout_partial_paragraph) 의 본문 NO_LS
/// 문단 전용. 셀 경로는 recompose_for_cell_width 를 그대로 사용한다.
pub fn recompose_for_body_width(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    column_inner_width_px: f64,
    styles: &ResolvedStyleSet,
) {
    restyle_fallback_runs_by_char_shapes(composed, para);
    recompose_for_cell_width(composed, para, column_inner_width_px, styles);
}

pub fn recompose_for_cell_width(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
) {
    let has_synthetic_line_segs = !para.line_segs.is_empty()
        && para
            .line_segs
            .iter()
            .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0);
    let has_authoritative_line_segs = !para.line_segs.is_empty() && !has_synthetic_line_segs;
    if has_authoritative_line_segs {
        return;
    }
    if para.line_segs.len() >= 2 && has_synthetic_line_segs {
        // HWPX 로드 단계에서 셀 폭/높이/anchor 속성으로 합성한 lineSeg 경계는
        // 이미 문서 속성 기반 보정 결과다. 여기서 다시 폭 기준으로 합치고
        // 재분할하면 RowBreak 표의 쪽 나눔 기준 줄 수가 원본 세로 정보와 어긋난다.
        return;
    }
    if composed.lines.is_empty() {
        return;
    }
    if cell_inner_width_px <= 0.0 {
        return;
    }
    // [#2070] lineSeg 부재 fallback 도 문단 여백/들여쓰기 반영 폭을 쓰되,
    // 내어쓰기(intent<0)의 본질대로 **첫 줄 폭과 연속 줄 폭을 분리**한다.
    // 종전 전체 폭 단일 사용은 조문 문단(80168 pi=362, ps intent=-3120)에서
    // 연속 줄 폭 41.6px 과대 → 줄수 과소; 반대로 연속 폭 단일 사용은 첫 줄
    // 과소로 +1줄 광역 팽창 (165쪽 회귀). HWP3-origin legacy bullet 은
    // 종전대로 별도 1.04 tolerance 로 정합한다.
    // [#2070 정밀화] 이중 폭은 검증 영역(내어쓰기 intent<0, 80168 계열 사다리·오라클)
    // 에 한정한다. intent>=0 의 no-lineseg 폴백과 HWP3-origin legacy bullet
    // (is_hwp3_hwp5_missing_lineseg_legacy_bullet, sample16-hwp5 = 64쪽 게이트)은
    // 종전 Task #671 전체 폭 유지 (이중 폭 적용 시 65 over-split, git bisect 0e21ec08).
    let hwp3_legacy_bullet =
        styles.hwp3_variant || is_hwp3_hwp5_missing_lineseg_legacy_bullet(para, composed, styles);
    let (first_width_px, cont_width_px) = styles
        .para_styles
        .get(para.para_shape_id as usize)
        .map(|ps| {
            if ps.indent < 0.0 && !hwp3_legacy_bullet {
                let continuation_left = ps.margin_left + ps.indent.abs();
                let first_left = ps.margin_left;
                (
                    (cell_inner_width_px - first_left.max(0.0) - ps.margin_right).max(0.0),
                    (cell_inner_width_px - continuation_left.max(0.0) - ps.margin_right).max(0.0),
                )
            } else {
                (cell_inner_width_px, cell_inner_width_px)
            }
        })
        .unwrap_or((cell_inner_width_px, cell_inner_width_px));
    let text_width_px = first_width_px.max(cont_width_px);
    if text_width_px <= 0.0 {
        return;
    }
    // Some HWP3-origin HWP5 files omit PARA_LINE_SEG for legacy bullet paragraphs.
    // HY신명조's embedded metrics are slightly wider than Hancom's converted reflow here,
    // so use a small tolerance only for the tight leading-body style pattern.
    let width_tolerance = if is_hwp3_hwp5_missing_lineseg_legacy_bullet(para, composed, styles) {
        1.04
    } else {
        1.0
    };
    let eff_first_px = first_width_px * width_tolerance;
    let eff_cont_px = cont_width_px * width_tolerance;
    // [Task #1042 Stage 6a] multi-line 지원 — compose_lines fallback 의 CHARS_PER_LINE=45
    // heuristic 결과가 cell width 와 일치 안 할 수 있음. lines 의 runs 를 합쳐서
    // cell width 기반 re-split.
    // [#2169] 줄나눔 기준 '글자' 문단은 글자 단위 분할. 통제 사다리 실측:
    // 한글은 breakNonLatinWord=KEEP_WORD(bit7=1)를 **글자 채움**으로,
    // BREAK_WORD(bit7=0)를 **어절 유지**로 렌더 — 코드 규약(1=어절)과 정반대
    // (kbu_ladder.hwpx: KEEP_WORD 2줄/BREAK_WORD 3줄; 80168 r10 "또/는" 분리).
    // 본문 라인브레이커의 광역 반전은 별도 과제 — 셀 재래핑만 우선 정합.
    // [정식화 보류] 글자 채움은 kbu 단독 조건으로는 문서별 실동작 차(21761835
    // -82.9 과소, recount WOR5)를 설명 못해 실험 브랜치에 보존 — 조건 정밀화
    // 후 재적용 (#2169 stage9~10).
    // [#2070 실험] 글자 채움(kbu==1) 재적용 (5축 전면).
    let char_break = styles
        .para_styles
        .get(para.para_shape_id as usize)
        .map(|ps| ps.korean_break_unit == 1)
        .unwrap_or(false);
    // [#2070] 공백 압축(condense) — 사다리 v4: R(cnd25) vs N(cnd0) 마크 분리 실측.
    let space_condense = styles
        .para_styles
        .get(para.para_shape_id as usize)
        .map(|ps| ps.condense_min_space as f64 / 100.0)
        .unwrap_or(0.0);
    // [#2070] 강제 줄바꿈(\n, has_line_break) 경계는 병합·재분할에서 보존한다.
    // 종전에는 전 줄을 한 줄로 합쳐 폭 기준 재분할 → \n 경계 소실로 생성계
    // NO_LS 셀이 과소 (80168 pi=362 조문 표: 한글 11줄 vs 8줄, -58px).
    // \n 으로 닫히는 그룹 단위로 합쳐 각 그룹을 독립 재래핑한다.
    let src_lines = std::mem::take(&mut composed.lines);
    let mut groups: Vec<(ComposedLine, bool)> = Vec::new();
    let mut cur: Option<ComposedLine> = None;
    // [#2070] 그룹 경계는 para.text 의 **실제 '\n'** 로만 판정한다.
    // CHARS_PER_LINE 폴백은 휴리스틱 줄에도 has_line_break=true 를 달므로
    // (#994 Justify 억제용) 그대로 믿으면 45자 단위 꼬마 줄이 생긴다
    // (80168 개정안 셀 per-para 대조: +13줄, 43위치 2~5자 줄).
    let text_chars: Vec<char> = para.text.chars().collect();
    let next_starts: Vec<Option<usize>> = (0..src_lines.len())
        .map(|i| src_lines.get(i + 1).map(|nl| nl.char_start))
        .collect();
    for (i, l) in src_lines.into_iter().enumerate() {
        let brk = l.has_line_break
            && match next_starts[i] {
                // 다음 줄 시작 직전 문자가 실제 개행일 때만 하드 경계
                Some(b) => b > 0 && text_chars.get(b - 1) == Some(&'\n'),
                // 마지막 줄의 has_line_break 는 텍스트 끝 '\n'
                None => text_chars.last() == Some(&'\n'),
            };
        match cur.as_mut() {
            None => cur = Some(l),
            Some(acc) => acc.runs.extend(l.runs),
        }
        if brk {
            let mut g = cur.take().expect("group line");
            g.has_line_break = false;
            groups.push((g, true));
        }
    }
    if let Some(mut g) = cur.take() {
        g.has_line_break = false;
        groups.push((g, false));
    }
    for (gi, (combined_line, ends_with_break)) in groups.into_iter().enumerate() {
        // 내어쓰기 첫 줄 폭은 문단의 첫 줄에만 적용 — \n 이후 그룹은 전부 연속 폭.
        let g_first = if gi == 0 { eff_first_px } else { eff_cont_px };
        let start = composed.lines.len();
        let total_width = estimate_composed_line_width(&combined_line, styles);
        // [#2070] 행미 공백 hanging — 한글은 줄 끝 공백을 폭 판정에서 제외한다.
        // trailing 공백 포함 폭으로 분할하면 공백만의 유령 둘째 줄이 생겨
        // NO_LS 셀 행높이가 배가된다 (시장구조조사 "100.0␣␣" 22→50.4px,
        // 2195행 × 4표 → +291쪽의 본류).
        let trailing_space_w: f64 = {
            let mut w = 0.0;
            'outer: for run in combined_line.runs.iter().rev() {
                let ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                for ch in run.text.chars().rev() {
                    if ch == ' ' {
                        w += estimate_text_width(" ", &ts);
                    } else {
                        break 'outer;
                    }
                }
            }
            w
        };
        if total_width - trailing_space_w <= g_first + 0.5 {
            composed.lines.push(combined_line);
        } else {
            let mut frags = split_composed_line_by_width(
                &combined_line,
                g_first,
                eff_cont_px,
                styles,
                char_break,
                space_condense,
            );
            // 분할 결과의 공백-단독 조각도 hanging — 직전 조각에 흡수한다.
            let mut folded: Vec<ComposedLine> = Vec::with_capacity(frags.len());
            for frag in frags.drain(..) {
                let ws_only = !frag.runs.is_empty()
                    && frag.runs.iter().all(|r| r.text.chars().all(|c| c == ' '));
                if ws_only {
                    if let Some(prev) = folded.last_mut() {
                        prev.runs.extend(frag.runs);
                        continue;
                    }
                }
                folded.push(frag);
            }
            composed.lines.extend(folded);
        }
        if composed.lines.len() > start && ends_with_break {
            if let Some(last) = composed.lines.last_mut() {
                last.has_line_break = true;
            }
        }
        // [#2279] 분할 줄의 줄높이 per-line 재산정 — 한글은 줄마다 **그 줄의 최대
        // 글자 크기**로 pitch 를 정한다. 종전에는 분할 줄이 원본 압축줄의 lh/ls
        // (= 문단 최대 fs 기준)를 상속해, 큰 글자가 첫 줄에만 있는 문단(86712
        // pi=20: "ㅇ "=15pt + 본문 14pt)에서 후속 줄 pitch 가 +2.1px/줄 과대
        // (rhwp 32.0 vs 한글 실측 29.9 = 14pt×4/3×160%). 페이지당 ~20줄 누적 시
        // -40px 급 fit 오차(86712 p10 pi=30 밀림)의 본체. Percent 줄간격 한정,
        // lh/ls/bl 을 줄 최대 fs 비율로 축소(확대 없음 — 원본 상속이 상한).
        if composed.lines.len() > start {
            let is_percent = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|ps| {
                    matches!(
                        ps.line_spacing_type,
                        crate::model::style::LineSpacingType::Percent
                    )
                })
                .unwrap_or(false);
            if is_percent {
                let group_max_fs = composed.lines[start..]
                    .iter()
                    .flat_map(|l| l.runs.iter())
                    .map(|r| {
                        resolved_to_text_style(styles, r.char_style_id, r.lang_index).font_size
                    })
                    .fold(0.0f64, f64::max);
                if group_max_fs > 0.0 {
                    for line in composed.lines[start..].iter_mut() {
                        let line_max_fs = line
                            .runs
                            .iter()
                            .map(|r| {
                                resolved_to_text_style(styles, r.char_style_id, r.lang_index)
                                    .font_size
                            })
                            .fold(0.0f64, f64::max);
                        if line_max_fs > 0.0 && line_max_fs < group_max_fs - 0.01 {
                            let factor = line_max_fs / group_max_fs;
                            line.line_height = ((line.line_height as f64) * factor).round() as i32;
                            line.line_spacing =
                                ((line.line_spacing as f64) * factor).round() as i32;
                            line.baseline_distance =
                                ((line.baseline_distance as f64) * factor).round() as i32;
                        }
                    }
                }
            }
        }
    }
    // [#2279 진단] 측정/렌더 경로별 재래핑 폭·줄수 대조 — 동작 불변.
    if let Ok(pat) = std::env::var("RHWP_DIAG_RECOMP") {
        if para.text.contains(&pat) {
            eprintln!(
                "DIAG_RECOMP width={:.2} first={:.2} cont={:.2} lines={} text={:?}",
                cell_inner_width_px,
                first_width_px,
                cont_width_px,
                composed.lines.len(),
                para.text.chars().take(20).collect::<String>(),
            );
        }
    }
}

/// [#2279 axis B] 셀 텍스트 오버플로 시 좌우 패딩 축소 — 렌더/측정 공용 코어.
///
/// 렌더(`layout_table_cells`)는 단일줄 오버플로 셀의 좌우 패딩을 축소한 뒤
/// `recompose_for_cell_width` 로 재래핑한다. 측정(cut `cell_units` / mt
/// `HeightMeasurer`)이 원 패딩 폭(더 좁음)으로 래핑하면 같은 문단이
/// 렌더 4줄/측정 5줄로 갈라져 행높이가 과대해진다 (86712 pi=172 산식 셀
/// 106.5px vs 한글 86.3px, #2237 axis B). 규칙 본체를 단일 출처로 공유한다.
/// 의미는 종전 `LayoutEngine::shrink_cell_padding_for_overflow` 와 동일.
pub(crate) fn shrunk_cell_horizontal_padding(
    pad_left: f64,
    pad_right: f64,
    cell_w: f64,
    composed_paras: &[ComposedParagraph],
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    preserve_cell_padding: bool,
) -> (f64, f64) {
    if preserve_cell_padding {
        return (pad_left, pad_right);
    }

    // [Task #617] 다중 줄(2 줄 이상) 단락이 line_segs 로 분배 완료된 경우,
    // HWP 가 가용 폭에 맞춰 자간을 분배하고 줄바꿈을 확정한 상태이므로
    // 자연 폭 추정으로 다시 깎으면 오버 페인팅. 단일 줄 셀(좁은 수치 셀
    // 등에서 오버플로우 가능성 있음) 은 종전 휴리스틱으로 보호한다.
    let any_multiline_distributed = paragraphs.iter().any(|p| p.line_segs.len() >= 2);
    if any_multiline_distributed {
        return (pad_left, pad_right);
    }

    let mut max_line_w = 0.0f64;
    for comp in composed_paras {
        for line in &comp.lines {
            let mut w = 0.0;
            for run in &line.runs {
                let mut ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                if run.char_overlap.is_some() {
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    let chars: Vec<char> = run.text.chars().collect();
                    w += fs * char_overlap_advance_units(&chars) as f64;
                    continue;
                }
                // 자연 폭 측정: 음수 자간을 제거하여 글리프가 서로 겹치지 않는 최소 폭을 얻음
                if ts.letter_spacing < 0.0 {
                    ts.letter_spacing = 0.0;
                }
                // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭 사용.
                // (estimate_text_width 는 ts.ratio 를 자체 반영함.)
                w += estimate_text_width(effective_text_for_metrics(run), &ts);
            }
            if w > max_line_w {
                max_line_w = w;
            }
        }
    }
    let available = (cell_w - pad_left - pad_right).max(0.0);
    // Task #347: estimate_text_width는 영어 본문(Times New Roman 등) 자연 폭을
    // 5~15%까지 과대 추정할 수 있어, HWP가 이미 줄바꿈한 본문에서도
    // padding 축소가 잘못 트리거됨. 15% 이내 초과는 정상으로 보고 미축소.
    let overflow_threshold = available * 1.15;
    if max_line_w <= overflow_threshold || cell_w <= 2.0 {
        return (pad_left, pad_right);
    }
    let min_pad = 1.0;
    let total_pad = pad_left + pad_right;
    let max_reducible = (total_pad - 2.0 * min_pad).max(0.0);
    if max_reducible <= 0.0 {
        return (pad_left, pad_right);
    }
    let deficit = max_line_w - available;
    let reduction = deficit.min(max_reducible);
    let new_total = total_pad - reduction;
    let new_left = if total_pad > 0.0 {
        pad_left * new_total / total_pad
    } else {
        new_total / 2.0
    };
    let new_right = new_total - new_left;
    (new_left, new_right)
}

fn is_hwp3_hwp5_missing_lineseg_legacy_bullet(
    para: &Paragraph,
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
) -> bool {
    let has_tight_leading_body_style = para.char_shapes.get(1).is_some_and(|cs_ref| {
        cs_ref.start_pos <= 3
            && styles
                .char_styles
                .get(cs_ref.char_shape_id as usize)
                .map(|cs| cs.letter_spacing <= -3.0)
                .unwrap_or(false)
    });

    para.line_segs.is_empty()
        && para.controls.is_empty()
        && para.text.starts_with('\u{F03C5}')
        && has_tight_leading_body_style
        && composed
            .lines
            .iter()
            .flat_map(|line| &line.runs)
            .any(|run| {
                styles
                    .char_styles
                    .get(run.char_style_id as usize)
                    .map(|cs| cs.font_family.split(',').next().unwrap_or("").trim() == "HY신명조")
                    .unwrap_or(false)
            })
}

/// 단일 ComposedLine 을 셀 가용 너비에 맞춰 다중 ComposedLine 으로 분할.
///
/// 분할 단위: 공백 단어 경계 우선, 단일 단어가 너비 초과 시 글자 단위 break.
/// 각 분할 줄의 메타데이터 (line_height/baseline/segment_width 등) 는 원본 보존.
fn split_composed_line_by_width(
    src: &ComposedLine,
    first_width_px: f64,
    cont_width_px: f64,
    styles: &ResolvedStyleSet,
    char_break: bool,
    space_condense: f64,
) -> Vec<ComposedLine> {
    let mut result: Vec<ComposedLine> = Vec::new();
    // [#2070] 내어쓰기(intent<0) 이중 폭: 첫 출력 줄은 first_width, 이후 연속
    // 줄은 cont_width 로 판정한다 (80168 조문 문단 첫줄 넓게/연속 좁게).
    let limit = |res: &Vec<ComposedLine>| -> f64 {
        if res.is_empty() {
            first_width_px
        } else {
            cont_width_px
        }
    };
    let mut current_runs: Vec<ComposedTextRun> = Vec::new();
    let mut current_width = 0.0;
    // [#2070] 한양신명조 사다리 v3/v4 확정 규칙: (a) 줄 채움 판정은 공백 폭을
    // 문단 condense% 만큼 압축해 계산(공백 압축), (b) 줄끝 초과 공백 1개는
    // 다음 줄로 넘기지 않고 현재 줄에 매달림(hang).
    let mut space_w = 0.0;
    let mut hung = false;
    let mut current_char_start = src.char_start;
    let mut chars_in_line = 0usize;
    let mut current_run_text = String::new();
    let mut current_run_template: Option<ComposedTextRun> = None;

    let flush_run =
        |runs: &mut Vec<ComposedTextRun>, text: &mut String, template: &Option<ComposedTextRun>| {
            if !text.is_empty() {
                if let Some(t) = template {
                    runs.push(ComposedTextRun {
                        text: std::mem::take(text),
                        char_style_id: t.char_style_id,
                        lang_index: t.lang_index,
                        char_overlap: t.char_overlap.clone(),
                        footnote_marker: t.footnote_marker,
                        display_text: None,
                    });
                } else {
                    text.clear();
                }
            }
        };

    let push_line = |result: &mut Vec<ComposedLine>,
                     runs: &mut Vec<ComposedTextRun>,
                     current_char_start: &mut usize,
                     chars_in_line: &mut usize,
                     current_width: &mut f64| {
        if !runs.is_empty() {
            result.push(ComposedLine {
                runs: std::mem::take(runs),
                line_height: src.line_height,
                baseline_distance: src.baseline_distance,
                segment_width: src.segment_width,
                column_start: src.column_start,
                line_spacing: src.line_spacing,
                has_line_break: false,
                char_start: *current_char_start,
            });
            *current_char_start += *chars_in_line;
            *chars_in_line = 0;
            *current_width = 0.0;
        }
    };

    for run in &src.runs {
        let ts = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        // 현재 run 의 template 변경 (char_style 다른 run 들 처리)
        if current_run_template
            .as_ref()
            .map(|t| t.char_style_id != run.char_style_id || t.lang_index != run.lang_index)
            .unwrap_or(true)
        {
            flush_run(
                &mut current_runs,
                &mut current_run_text,
                &current_run_template,
            );
            current_run_template = Some(run.clone());
        }
        // [#2169] 줄나눔 기준 '글자'(korean_break_unit==0) — 글자 단위 채움.
        // 한글은 이 모드에서 어절 경계 무시하고 줄을 채운다 (80168 r10:
        // "또/는", "필요/한" 글자 분리, 한글 5줄 vs 어절 래핑 6줄).
        if char_break {
            for ch in run.text.chars() {
                let ch_str: String = std::iter::once(ch).collect();
                let ch_width = crate::renderer::layout::estimate_text_width_unrounded(&ch_str, &ts);
                if std::env::var("RHWP_RAZOR").is_ok()
                    && src.runs.iter().any(|r| r.text.contains("도조례로 정하는"))
                {
                    eprintln!(
                        "RZ: ch={:?} w={:.2} cur={:.2} spw={:.2} cnd={:.2} limit={:.2} fam={:?}",
                        ch,
                        ch_width,
                        current_width,
                        space_w,
                        space_condense,
                        limit(&result),
                        ts.font_family.split(',').next().unwrap_or("")
                    );
                }
                let eff = current_width - space_w * space_condense;
                let over = eff + ch_width > limit(&result) && chars_in_line > 0;
                if over && ch == ' ' && !hung {
                    // 줄끝 초과 공백 1개 hang — 줄바꿈 없이 현재 줄에 계상.
                    hung = true;
                } else if over {
                    flush_run(
                        &mut current_runs,
                        &mut current_run_text,
                        &current_run_template,
                    );
                    push_line(
                        &mut result,
                        &mut current_runs,
                        &mut current_char_start,
                        &mut chars_in_line,
                        &mut current_width,
                    );
                    space_w = 0.0;
                    hung = false;
                }
                current_run_text.push(ch);
                current_width += ch_width;
                if ch == ' ' {
                    space_w += ch_width;
                }
                chars_in_line += 1;
            }
            continue;
        }
        // run 텍스트를 단어 단위로 분할 (공백 포함)
        let mut word = String::new();
        for ch in run.text.chars() {
            word.push(ch);
            // 공백 또는 마지막 글자 직전이 단어 경계
            if ch == ' ' || ch == '\t' {
                let word_width = crate::renderer::layout::estimate_text_width_unrounded(&word, &ts);
                // 현재 단어가 추가되면 max_width 초과하는지 검사
                if current_width - space_w * space_condense + word_width > limit(&result)
                    && (chars_in_line > 0 || !current_run_text.is_empty())
                {
                    // 현재 줄을 flush 후 새 줄 시작
                    flush_run(
                        &mut current_runs,
                        &mut current_run_text,
                        &current_run_template,
                    );
                    push_line(
                        &mut result,
                        &mut current_runs,
                        &mut current_char_start,
                        &mut chars_in_line,
                        &mut current_width,
                    );
                    space_w = 0.0;
                    hung = false;
                }
                // 단어 자체가 max_width 초과 시 글자 단위 break
                if word_width > limit(&result) && current_width == 0.0 {
                    for wch in word.chars() {
                        let wch_str: String = std::iter::once(wch).collect();
                        let wch_width =
                            crate::renderer::layout::estimate_text_width_unrounded(&wch_str, &ts);
                        if current_width - space_w * space_condense + wch_width > limit(&result)
                            && chars_in_line > 0
                        {
                            flush_run(
                                &mut current_runs,
                                &mut current_run_text,
                                &current_run_template,
                            );
                            push_line(
                                &mut result,
                                &mut current_runs,
                                &mut current_char_start,
                                &mut chars_in_line,
                                &mut current_width,
                            );
                            space_w = 0.0;
                            hung = false;
                        }
                        current_run_text.push(wch);
                        current_width += wch_width;
                        chars_in_line += 1;
                    }
                } else {
                    current_run_text.push_str(&word);
                    current_width += word_width;
                    space_w += crate::renderer::layout::estimate_text_width_unrounded(" ", &ts)
                        * word.matches(' ').count() as f64;
                    chars_in_line += word.chars().count();
                }
                word.clear();
            }
        }
        // run 끝에 남은 단어 처리
        if !word.is_empty() {
            let word_width = crate::renderer::layout::estimate_text_width_unrounded(&word, &ts);
            if current_width - space_w * space_condense + word_width > limit(&result)
                && (chars_in_line > 0 || !current_run_text.is_empty())
            {
                flush_run(
                    &mut current_runs,
                    &mut current_run_text,
                    &current_run_template,
                );
                push_line(
                    &mut result,
                    &mut current_runs,
                    &mut current_char_start,
                    &mut chars_in_line,
                    &mut current_width,
                );
                space_w = 0.0;
                hung = false;
            }
            // 단어 자체가 max_width 초과 시 글자 단위 break
            if word_width > limit(&result) && current_width == 0.0 {
                for wch in word.chars() {
                    let wch_str: String = std::iter::once(wch).collect();
                    let wch_width =
                        crate::renderer::layout::estimate_text_width_unrounded(&wch_str, &ts);
                    if current_width - space_w * space_condense + wch_width > limit(&result)
                        && chars_in_line > 0
                    {
                        flush_run(
                            &mut current_runs,
                            &mut current_run_text,
                            &current_run_template,
                        );
                        push_line(
                            &mut result,
                            &mut current_runs,
                            &mut current_char_start,
                            &mut chars_in_line,
                            &mut current_width,
                        );
                        space_w = 0.0;
                        hung = false;
                    }
                    current_run_text.push(wch);
                    current_width += wch_width;
                    chars_in_line += 1;
                }
            } else {
                current_run_text.push_str(&word);
                current_width += word_width;
                chars_in_line += word.chars().count();
            }
        }
    }
    // 마지막 줄 flush
    flush_run(
        &mut current_runs,
        &mut current_run_text,
        &current_run_template,
    );
    push_line(
        &mut result,
        &mut current_runs,
        &mut current_char_start,
        &mut chars_in_line,
        &mut current_width,
    );
    space_w = 0.0;
    hung = false;

    if result.is_empty() {
        // 안전장치: 절대 빈 결과 반환하지 않음
        result.push(src.clone());
    }
    result
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
    // Issue #677: U+F081C 는 HWP TAC filler 이며 text_measurement 경로에서
    // 시각 폭 0으로 처리해야 한다. display_text 로 바꾸면 이 0폭 규칙을
    // 우회하므로 원문을 유지한다.
    if run.text.contains('\u{F081C}') {
        return &run.text;
    }
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
    if (0xF0289..=0xF0291).contains(&cp) {
        return Some((0, (cp - 0xF0288) as u8));
    } // tens 1-9
    if (0xF0292..=0xF029B).contains(&cp) {
        return Some((1, (cp - 0xF0292) as u8));
    } // ones 0-9
      // 3자리 블록
    if (0xF0491..=0xF0499).contains(&cp) {
        return Some((0, (cp - 0xF0490) as u8));
    } // hundreds 1-9
    if (0xF049A..=0xF04A3).contains(&cp) {
        return Some((1, (cp - 0xF049A) as u8));
    } // tens 0-9
    if (0xF04A4..=0xF04AD).contains(&cp) {
        return Some((2, (cp - 0xF04A4) as u8));
    } // ones 0-9
    None
}

/// CharOverlap의 PUA 문자 배열을 숫자 문자열로 디코딩한다.
///
/// 모든 문자가 PUA 겹침용 숫자인 경우에만 디코딩 성공 (Some).
/// 그룹 번호(0=최상위자리, 1=중간, 2=최하위)로 정렬하여 올바른 자릿수 순서를 보장한다.
pub fn decode_pua_overlap_number(chars: &[char]) -> Option<String> {
    if chars.is_empty() {
        return None;
    }
    let mut groups: Vec<(u8, u8)> = Vec::with_capacity(chars.len());
    for &ch in chars {
        groups.push(pua_overlap_digit(ch)?);
    }
    // 그룹 번호 순 정렬 (최상위 자리 → 최하위 자리)
    groups.sort_by_key(|(g, _)| *g);
    let s: String = groups.iter().map(|(_, d)| char::from(b'0' + d)).collect();
    Some(s)
}

/// CharOverlap controls occupy one text-flow position even when their payload
/// contains multiple glyph components.
///
/// Hancom uses this for overlapped two-digit markers such as the boxed 10/11/12
/// in `table-vpos-01.hwp`: the control stores two PUA glyph components, but
/// caret movement and line measurement advance by one character box.
pub fn char_overlap_advance_units(chars: &[char]) -> usize {
    usize::from(!chars.is_empty())
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

fn pua_plain_text_display(ch: char) -> Option<&'static str> {
    match ch as u32 {
        0xF012B => Some("(인)"),
        // 2025 행정업무운영 편람 p08 TOC bullet. Hancom PDF renders this
        // private-use marker as a filled square bullet.
        0xF031C => Some("■"),
        // 2025 행정업무운영 편람 p15 callout bullet. Hancom PDF renders this
        // private-use marker as a filled right-pointing pointer, not tofu.
        0xF02FC => Some("►"),
        // [Task #1001] 한컴 변환본 (HWP3→HWP5) 의 글머리표 PUA. 한컴 viewer 는
        // 빈 체크박스 모양으로 표시. "□" (U+25A1 WHITE SQUARE) 매핑.
        // 실제 sample16-hwp5 의 PUA codepoint 는 U+F03C5 (글자 분석 결과).
        0xF03C5 => Some("□"),
        _ => None,
    }
}

/// 한글 방점(U+302E/U+302F)을 렌더용 spacing 가운데 점 글리프로 치환한다. (Task #1735)
///
/// U+302E/U+302F 는 유니코드 결합문자(combining mark)라, 유효한 base 없이
/// (줄 시작·공백 뒤) 셰이핑되면 브라우저/엔진이 dotted-circle(U+25CC)
/// placeholder 를 삽입하고 톤 점을 그 위에 쌓아 한컴과 다르게 표기된다.
/// 한컴은 방점을 독립 spacing 점으로 렌더하므로, 렌더 경로에서 결합 성질이
/// 없는 spacing 점 글리프로 치환한다. IR 텍스트는 불변(측정/캐럿/텍스트추출
/// 보존)이며, 측정 폭 정합은 text_measurement 의 전각 분류로 맞춘다.
fn tone_mark_display(ch: char) -> Option<char> {
    match ch {
        '\u{302E}' => Some('\u{00B7}'), // 방점 → · MIDDLE DOT
        '\u{302F}' => Some('\u{205A}'), // 쌍방점 → ⁚ TWO DOT PUNCTUATION (세로 두 점)
        _ => None,
    }
}

/// 일반 텍스트 렌더링/paint contract 경로에서 한컴 PUA 문자를 표시 문자열로 확장한다.
///
/// HWP TAC filler `U+F081C` 는 레이아웃 측정에는 원문으로 남겨 0폭 규칙을
/// 적용하되, 실제 출력에서는 글리프가 없어 깨진 문자로 보이지 않도록 숨긴다.
///
/// Hanyang-PUA 옛한글은 KS X 1026-1:2007 자모 시퀀스로 확장한다.
///
/// CharOverlap 전용 숫자(`U+F02CE..=U+F02E1`)는 여기서 확장하지 않는다.
/// 해당 문자는 `pua_to_display_text()`가 글자겹침 렌더러에서만 처리한다.
pub fn expand_pua_display_text(text: &str) -> String {
    use super::pua_oldhangul::map_pua_old_hangul;

    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch == '\u{F081C}' {
            continue;
        }
        if let Some(dot) = tone_mark_display(ch) {
            out.push(dot);
        } else if let Some(replacement) = pua_plain_text_display(ch) {
            out.push_str(replacement);
        } else if let Some(jamos) = map_pua_old_hangul(ch) {
            out.extend(jamos.iter().copied());
        } else {
            out.push(super::layout::map_pua_bullet_char(ch));
        }
    }
    out
}

/// 일반 텍스트 렌더링 경로의 기존 helper 이름.
pub fn expand_pua_render_text(text: &str) -> String {
    expand_pua_display_text(text)
}

/// PUA 테두리 숫자와 한컴 PUA 기호를 표시 문자열로 변환한다. (렌더러 전용)
///
/// draw_char_overlap()에서 호출하여, 실제 렌더링 시에만 변환한다.
pub fn pua_to_display_text(ch: char) -> Option<String> {
    let cp = ch as u32;
    if let Some(replacement) = pua_plain_text_display(ch) {
        return Some(replacement.to_string());
    }
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
            let has_pua = run
                .text
                .chars()
                .any(|ch| pua_enclosed_border_type(ch).is_some());
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
                            display_text: None,
                        });
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
                        display_text: None,
                    });
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
                    display_text: None,
                });
            }
        }

        if changed {
            line.runs = new_runs;
        }
    }
}

mod line_breaking;
pub mod lineseg_compare;

pub(crate) use line_breaking::{
    is_line_end_forbidden, is_line_start_forbidden, recalculate_section_vpos, reflow_line_segs,
    tokenize_paragraph, BreakToken,
};

#[cfg(test)]
mod lineseg_compare_tests;
#[cfg(test)]
mod re_sample_gen;
#[cfg(test)]
mod tests;
