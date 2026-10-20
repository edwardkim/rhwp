//! 수용한 저장 줄의 기준값과 물리 단 앵커를 한 쌍으로 전달한다.

/// `column_anchor_y`는 InlineBoxPlacement의 y와 같은 단 상대 px 좌표다.
/// 실제 쪽 좌표를 소비하는 배치는 단의 원점을 한 번 더한다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredColumnFrame {
    pub source_origin_hu: i32,
    pub column_anchor_y: f64,
}
