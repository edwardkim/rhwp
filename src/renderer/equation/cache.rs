//! 같은 수식의 흐름 프레임과 paint 조각을 한 번만 구성한다.
//!
//! 미주 fit 판정은 같은 단을 여러 번 실제 배치 경로로 재생하므로 같은 수식의
//! 흐름 프레임 질의와 paint 조각 생성이 문서당 수십만 번 반복된다. 두 결과는 아래
//! 키의 값(과 paint의 dpi)만으로 정해지므로, 측정과 실제 paint가 같은 값을 계속
//! 소비한다. 결과를 만드는 경로가 수식의 새 값을 읽게 되면 키에도 넣는다.

use super::PaintParts;
use crate::model::control::Equation;
use std::cell::RefCell;
use std::collections::HashMap;

/// 한 스레드가 보관하는 최대 수식 수. 넘으면 비우고 다시 채운다.
const FLOW_CAPACITY: usize = 16 * 1024;
/// paint 조각은 SVG 문자열을 가지므로 더 작게 보관한다.
const PAINT_CAPACITY: usize = 4 * 1024;

#[derive(PartialEq, Eq, Hash)]
struct EquationKey {
    script: String,
    font_name: String,
    version_info: String,
    font_size: u32,
    attr: u32,
    eqedit: u32,
    color: u32,
    baseline: i16,
    width: u32,
    height: u32,
    size_protect: bool,
    treat_as_char: bool,
    host_frame_seal: Option<([u8; 32], bool)>,
}

impl EquationKey {
    fn new(equation: &Equation) -> Self {
        Self {
            script: equation.script.clone(),
            font_name: equation.font_name.clone(),
            version_info: equation.version_info.clone(),
            font_size: equation.font_size,
            attr: equation.attr,
            eqedit: equation.eqedit,
            color: equation.color,
            baseline: equation.baseline,
            width: equation.common.width,
            height: equation.common.height,
            size_protect: equation.common.size_protect,
            treat_as_char: equation.common.treat_as_char,
            host_frame_seal: equation
                .unpositioned_host_frame_seal
                .as_ref()
                .map(|seal| (seal.signature, seal.legacy_layout)),
        }
    }
}

thread_local! {
    static FLOW_METRICS: RefCell<HashMap<EquationKey, (u32, u32, i32)>> =
        RefCell::new(HashMap::new());
    static PAINT_PARTS: RefCell<HashMap<(EquationKey, u64), PaintParts>> =
        RefCell::new(HashMap::new());
}

fn insert_bounded<K: Eq + std::hash::Hash, V>(
    cache: &mut HashMap<K, V>,
    capacity: usize,
    key: K,
    value: V,
) {
    if cache.len() >= capacity {
        cache.clear();
    }
    cache.insert(key, value);
}

pub(super) fn flow_metrics(
    equation: &Equation,
    compute: impl Fn(&Equation) -> (u32, u32, i32),
) -> (u32, u32, i32) {
    let key = EquationKey::new(equation);
    if let Some(metrics) = FLOW_METRICS.with(|cache| cache.borrow().get(&key).copied()) {
        return metrics;
    }
    let metrics = compute(equation);
    FLOW_METRICS.with(|cache| {
        insert_bounded(&mut cache.borrow_mut(), FLOW_CAPACITY, key, metrics);
    });
    metrics
}

pub(super) fn paint_parts(
    equation: &Equation,
    dpi: f64,
    compute: impl Fn(&Equation, f64) -> PaintParts,
) -> PaintParts {
    let key = (EquationKey::new(equation), dpi.to_bits());
    if let Some(parts) = PAINT_PARTS.with(|cache| cache.borrow().get(&key).cloned()) {
        return parts;
    }
    let parts = compute(equation, dpi);
    PAINT_PARTS.with(|cache| {
        insert_bounded(&mut cache.borrow_mut(), PAINT_CAPACITY, key, parts.clone());
    });
    parts
}
