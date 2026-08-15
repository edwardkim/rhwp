//! DSEL 축·속성 이름의 오타 진단 보조.

use super::ast::{Axis, AXIS_NAMES};
use super::error::SelectorError;

/// 오타에 가장 가까운 후보를 찾는다 — 진단의 `hint` 로 나간다.
///
/// 편집 거리 2 이내만 후보로 본다. 3 이상을 허용하면 `para` 의 후보로 `cell` 이
/// 나오는 식이라 힌트가 오히려 방해가 된다.
pub fn nearest(
    name: &str,
    candidates: impl IntoIterator<Item = &'static str>,
) -> Option<&'static str> {
    let mut best: Option<(usize, &'static str)> = None;
    for cand in candidates {
        let d = edit_distance(name, cand);
        if d > 2 {
            continue;
        }
        // 같은 거리면 사전순 앞선 쪽으로 고정해 사전 선언 순서에 의존하지 않는다.
        match best {
            Some((best_distance, best_candidate))
                if best_distance < d || (best_distance == d && best_candidate <= cand) => {}
            _ => best = Some((d, cand)),
        }
    }
    best.map(|(_, candidate)| candidate)
}

/// 두 문자열의 Levenshtein 거리 (문자 단위).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (index, left) in a.iter().enumerate() {
        current[0] = index + 1;
        for (other_index, right) in b.iter().enumerate() {
            let cost = usize::from(left != right);
            current[other_index + 1] = (previous[other_index + 1] + 1)
                .min(current[other_index] + 1)
                .min(previous[other_index] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// 축 이름 오타를 후보와 함께 거절한다.
pub fn unknown_axis(name: &str, offset: usize) -> SelectorError {
    let error = SelectorError::resolve(offset, format!("알 수 없는 축 `{name}`"))
        .expecting(AXIS_NAMES.iter().map(|(name, _)| *name));
    match nearest(name, AXIS_NAMES.iter().map(|(name, _)| *name)) {
        Some(candidate) => error.hinting(format!("`{candidate}` 를 뜻했나")),
        None => error,
    }
}

/// 속성 이름 오타를 그 축의 후보와 함께 거절한다.
pub fn unknown_attr(axis: Axis, name: &str, offset: usize) -> SelectorError {
    let names: Vec<&'static str> = axis
        .attributes()
        .iter()
        .map(|attribute| attribute.name)
        .collect();
    let error = SelectorError::resolve(
        offset,
        format!("축 `{}` 에 없는 속성 `{name}`", axis.name()),
    )
    .expecting(names.clone());
    if let Some(candidate) = nearest(name, names) {
        return error.hinting(format!("`{candidate}` 를 뜻했나"));
    }
    // 축을 적지 않은 스텝은 `*` 이고 `*` 에는 공통 속성밖에 없다.
    if axis == Axis::Any {
        return error.hinting(
            "축을 생략하면 `*` 이라 공통 속성만 쓸 수 있다 — 술어는 축에 붙여 `para[len>0]` 처럼 적는다 (공백은 자손 결합자다)",
        );
    }
    error
}
