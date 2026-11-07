//! 줄 경계에서 소비하는 구분 공백과 후행 개체 앞의 작성된 공백을 구분한다.

use std::collections::BTreeSet;

use super::{Atom, InlineFlowBox, InlineFlowContent};

pub(super) fn has_flow_content(row: &[InlineFlowBox], chars: &[char]) -> bool {
    row.iter().any(|item| match &item.content {
        InlineFlowContent::Text { range, .. } => {
            chars[range.clone()].iter().any(|ch| !ch.is_whitespace())
        }
        InlineFlowContent::Table { .. } | InlineFlowContent::Picture { .. } => true,
        InlineFlowContent::FloatingTable { .. } | InlineFlowContent::FloatingShape { .. } => false,
    })
}

pub(super) struct SeparatorSpaces {
    pub wrap: BTreeSet<usize>,
    /// 문단 끝 공백 — 뒤에 글자·개체·개행·부동 앵커가 없다. 한/글은 말미 공백에
    /// 줄상자를 주지 않으므로(#7500·#7160 `absorb_whitespace_only_rows`) 넘쳐도
    /// 앞 줄 끝에 걸린다.
    pub trailing: BTreeSet<usize>,
    pub object: BTreeSet<usize>,
    pub first_object: BTreeSet<usize>,
}

pub(super) fn classify_with_storage<S: crate::storage::Storage>(
    atoms: &[Atom],
    chars: &[char],
    storage: &S,
) -> Result<SeparatorSpaces, S::Error> {
    let mut separators = BTreeSet::new();
    let mut object_separators = BTreeSet::new();
    let mut trailing = BTreeSet::new();
    let mut followed_by_flow = false;
    let mut followed_by_object = false;
    let mut at_paragraph_end = true;
    for atom in atoms.iter().rev() {
        match atom {
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Text { range, .. },
                ..
            }) if !range.is_empty() && chars[range.clone()].iter().all(|ch| *ch == ' ') => {
                if at_paragraph_end {
                    storage.insert_tree_set(&mut trailing, range.start)?;
                }
                if followed_by_flow {
                    storage.insert_tree_set(&mut separators, range.start)?;
                }
                if followed_by_object {
                    storage.insert_tree_set(&mut object_separators, range.start)?;
                }
            }
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Table { .. } | InlineFlowContent::Picture { .. },
                ..
            }) => {
                at_paragraph_end = false;
                followed_by_flow = true;
                followed_by_object = true;
            }
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Text { .. },
                ..
            }) => {
                at_paragraph_end = false;
                followed_by_flow = true;
                followed_by_object = false;
            }
            // 부동 앵커와 명시적 개행은 자기 줄을 소유한다.
            _ => {
                at_paragraph_end = false;
                followed_by_flow = false;
                followed_by_object = false;
            }
        }
    }
    let mut first_object = BTreeSet::new();
    let mut previous_separator = false;
    for atom in atoms {
        let position = match atom {
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Text { range, .. },
                ..
            }) if object_separators.contains(&range.start) => Some(range.start),
            _ => None,
        };
        // 개체는 가시 문자 축을 소비하지 않아도 작성 공백 그룹을 끊는다.
        if !previous_separator {
            if let Some(position) = position {
                storage.insert_tree_set(&mut first_object, position)?;
            }
        }
        previous_separator = position.is_some();
    }
    Ok(SeparatorSpaces {
        wrap: separators,
        trailing,
        object: object_separators,
        first_object,
    })
}
