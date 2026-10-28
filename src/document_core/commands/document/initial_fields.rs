//! 미기입 누름틀의 원본 안내문을 편집 IR의 빈 값으로 정규화한다.
//! 수정됨(bit15) 필드와 안내문이 아닌 실제 값은 보존한다.
use crate::model::control::{Control, FieldType};
use crate::model::document::Document;
use crate::model::identity::walk::{walk, Node};
use crate::model::paragraph::Paragraph;

pub(super) fn clear(document: &mut Document) {
    for section in &mut document.sections {
        // 본문·중첩 셀·도형 글상자 등 소유 문단을 같은 규칙으로 방문한다.
        walk(&mut section.paragraphs, |node| {
            if let Node::Paragraph(para) = node {
                process_para(para);
            }
            Ok(())
        })
        .expect("문단 정규화 방문자는 오류를 반환하지 않는다");
    }
}

fn process_para(para: &mut Paragraph) {
    // 삭제 대상 field_range 인덱스와 삭제할 문자 범위 수집
    let mut removals: Vec<(usize, usize, usize)> = Vec::new(); // (fr_idx, start, end)
    for (fri, fr) in para.field_ranges.iter().enumerate() {
        if fr.start_char_idx >= fr.end_char_idx {
            continue;
        }
        if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
            if f.field_type != FieldType::ClickHere {
                continue;
            }
            if f.properties & (1 << 15) != 0 {
                continue;
            } // 이미 수정된 상태
              // 필드 값이 안내문과 동일한지 확인
            if let Some(guide) = f.guide_text() {
                let chars: Vec<char> = para.text.chars().collect();
                if fr.end_char_idx <= chars.len() {
                    let field_val: String =
                        chars[fr.start_char_idx..fr.end_char_idx].iter().collect();
                    // trailing 공백 제거 후 비교 (한컴이 안내문 뒤에 공백을 추가하는 경우)
                    if field_val.trim_end() == guide || field_val == guide {
                        removals.push((fri, fr.start_char_idx, fr.end_char_idx));
                    }
                }
            }
        }
    }
    // [Task #1893] 삭제 수술의 IR 불변성 완성용 스냅샷 — 삭제 전 char_offsets 는
    // 원본 문자 인덱스→utf16 위치 매핑의 유일한 근거다. removal 좌표는 전부
    // 수집-시점(원본) 인덱스이므로, 원본 스냅샷으로 utf16 범위를 구해
    // char_shapes 경계를 함께 시프트해야 직렬화→재파스가 고정점이 된다.
    // (종전엔 text/field_ranges 만 고쳐 char_offsets/char_count/char_shapes 가
    // stale — 그 불일치 IR 을 저장하면 재파스 정준형과 조판이 갈라져
    // 라운드트립 렌더 752px 분기·빈 줄 추가가 발생했다.)
    let orig_offsets: Vec<u32> = para.char_offsets.clone();
    let orig_chars: Vec<char> = para.text.chars().collect();
    let offsets_valid = orig_offsets.len() == orig_chars.len();
    fn utf16_width(c: char) -> u32 {
        if c == '\t' {
            8
        } else if (c as u32) > 0xFFFF {
            2
        } else {
            1
        }
    }

    // 뒤에서부터 삭제 (인덱스 안정성 유지)
    for &(fri, start, end) in removals.iter().rev() {
        let chars: Vec<char> = para.text.chars().collect();
        // [Task #1620] 다중 removal 처리 중 앞선 removal 이 para.text 를 축소하면(특히
        // 같은 범위를 가리키는 중첩 field_range) 이후 removal 의 수집-시점 (start,end) 가
        // 현재 길이를 초과해 슬라이스 패닉(36396650). 현재 길이 기준 범위를 재검증해 skip.
        if start > end || end > chars.len() {
            continue;
        }
        let removed_len = end - start;
        let new_text: String = chars[..start].iter().chain(chars[end..].iter()).collect();
        para.text = new_text;
        para.field_ranges[fri].end_char_idx = start;
        // 이후 field_ranges의 char_idx 조정
        for i in 0..para.field_ranges.len() {
            if i == fri {
                continue;
            }
            let other = &mut para.field_ranges[i];
            if other.start_char_idx >= end {
                other.start_char_idx -= removed_len;
            }
            if other.end_char_idx >= end {
                other.end_char_idx -= removed_len;
            }
        }

        // [Task #1893] char_offsets/char_shapes/char_count 직접 수술 — 원본 utf16
        // 좌표 기준. 역순 처리라 오른쪽 removal 의 시프트가 왼쪽 utf16 좌표에 영향
        // 없고, 삭제 폭(u_end−u_start)은 원본 스냅샷 불변량이다. 컨트롤/필드 마커의
        // 8유닛 갭 구조는 기존 오프셋에 이미 올바르게 인코딩되어 있으므로 감산만으로
        // 보존된다.
        if offsets_valid && start < end && end <= orig_offsets.len() {
            let u_start = orig_offsets[start];
            // [#3545] 지워진 본문 run 을 HWPX 저장에서 되살리기 위한 잔재 기록.
            // 한컴 정준형은 미기입 누름틀의 안내문을 파일에 본문 run 으로 남기므로
            // (form-01.hwpx), 기록 없이 저장하면 파일에서 영구 소실된다. 서식까지
            // 되살리도록 그 텍스트를 담던 run 의 char shape 도 함께 남긴다 — 아래
            // 수술이 zero-width 로 접기 전의 원본 좌표에서 조회해야 정확하다.
            let residue_shape_id = para
                .char_shapes
                .iter()
                .rev()
                .find(|cs| cs.start_pos <= u_start)
                .map(|cs| cs.char_shape_id)
                .unwrap_or(0);
            let residue_text: String = orig_chars[start..end].iter().collect();
            let ctrl_idx = para.field_ranges[fri].control_idx;
            if let Some(Control::Field(f)) = para.controls.get_mut(ctrl_idx) {
                f.guide_residue = Some(crate::model::control::GuideResidue {
                    text: residue_text,
                    char_shape_id: residue_shape_id,
                });
            }
            // 삭제 폭 = 삭제 문자들의 utf16 폭만. orig_offsets[end] 는 필드 end
            // 마커의 8유닛 갭을 건너뛴 다음 문자 위치라 갭까지 폭에 포함되어
            // 후속 오프셋에서 마커 갭이 소실된다(슬롯 방출 위치 붕괴).
            let u_end = orig_offsets[end - 1] + utf16_width(orig_chars[end - 1]);
            let width = u_end.saturating_sub(u_start);
            // 삭제 구간의 오프셋 엔트리 제거 + 후속 엔트리 감산.
            para.char_offsets.drain(start..end);
            for off in para.char_offsets.iter_mut().skip(start) {
                *off = off.saturating_sub(width);
            }
            para.char_count = para.char_count.saturating_sub(width);
            for cs in &mut para.char_shapes {
                if cs.start_pos >= u_end {
                    cs.start_pos -= width;
                } else if cs.start_pos > u_start {
                    // 삭제 범위 내부 경계 → zero-width run 으로 시작점에 고정
                    // (한컴도 필드값 삭제 시 zero-width char run 을 남긴다 —
                    // 원본 서식의 자식 없는 <hp:run/> 33개와 동일 표현).
                    cs.start_pos = u_start;
                }
            }
        }
    }
}
