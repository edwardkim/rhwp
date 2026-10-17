//! 편집 후 저장 사다리의 컷과 재작성한 블록 호스트 줄의 출처.

use std::collections::BTreeSet;

use crate::model::{control::Control, paragraph::LineSeg, table::Cell};
use crate::renderer::layout::table_layout::CellUnitLineAnchor;

pub(super) struct CellFrameWriteback {
    pub(super) section_idx: usize,
    pub(super) para_idx: usize,
    pub(super) control_idx: usize,
    pub(super) cell_idx: usize,
    pub(super) resets: Vec<(usize, usize)>,
    pub(super) uncharged_hosts: BTreeSet<usize>,
    pub(super) generated_host_lines: Vec<(usize, usize)>,
    pub(super) preserved_frames: Vec<((usize, usize), (usize, usize))>,
}

/// 여러 물리 유닛이 공유하는 빈 블록 표 호스트 줄은 부모 LINE_SEG 한 개로
/// 내부 컷을 표현할 수 없다. 원본 사다리를 다른 컷에 이어 붙인 뒤에도 이 줄을
/// 한컴 저장 프레임으로 신뢰하면, 존재하지 않는 닫힌 프레임의 뒤 간격을 복원한다.
/// 실제 컷과 같은 cell_units의 줄 소유를 사용해 그런 재작성 줄만 구분한다.
/// 글자 줄·인라인 표·단일 유닛은 원래 저장 줄의 조성 정보를 보존한다.
pub(super) fn generated_block_host_lines(
    cell: &Cell,
    anchors: &[CellUnitLineAnchor],
) -> Vec<(usize, usize)> {
    let mut opened = None;
    let mut generated = BTreeSet::new();
    for anchor in anchors {
        match *anchor {
            CellUnitLineAnchor::Opens(para, line) => opened = Some((para, line)),
            CellUnitLineAnchor::InsideAnOpenLine => {
                if let Some((para, line)) = opened {
                    if cell.paragraphs.get(para).is_some_and(|host| {
                        host.text.is_empty()
                            && host.controls.iter().any(|control| {
                                matches!(control, Control::Table(table)
                                    if !table.common.treat_as_char)
                            })
                    }) {
                        generated.insert((para, line));
                    }
                }
            }
            CellUnitLineAnchor::NoLineOfItsOwn => {}
        }
    }
    generated.into_iter().collect()
}

pub(super) fn mark_generated_host_lines(
    cell: &mut Cell,
    original: &Cell,
    lines: &[(usize, usize)],
) {
    for &(para, line) in lines {
        if let Some(segment) = cell
            .paragraphs
            .get_mut(para)
            .and_then(|host| host.line_segs.get_mut(line))
        {
            // 다른 줄의 컷을 되썼다는 이유로 그대로 재사용한 원본 호스트의
            // 계보까지 바꾸지 않는다. 실제 줄 메트릭이 바뀐 호스트만 생성 줄이다.
            if original
                .paragraphs
                .get(para)
                .and_then(|host| host.line_segs.get(line))
                == Some(&*segment)
            {
                continue;
            }
            // 저장 텍스트·줄 메트릭·기존 tag는 보존한다. 독립 한컴 프레임이 아니라
            // 재작성된 부모 호스트라는 출처만 싣는다. reader의 기존 합성 줄 계약을 쓴다.
            segment.tag |= LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        }
    }
}
