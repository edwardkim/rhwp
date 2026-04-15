//! 문서 트리 경로 타입
//!
//! 문서 트리 내 임의 깊이의 요소를 가리키는 경로를 정의한다.
//! 중첩 표 편집 등 임의 깊이 접근을 지원한다.

/// 문서 트리 경로 세그먼트
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// 본문 문단 인덱스
    Paragraph(usize),
    /// 컨트롤 인덱스 (표, 그림 등)
    Control(usize),
    /// 표 셀 (row, col)
    Cell(u16, u16),
}

/// 문서 트리 내 임의 위치를 가리키는 경로
///
/// 최상위 표 접근 예시:
///   `[Paragraph(5), Control(0)]`
///
/// 중첩 표 접근 예시:
///   `[Paragraph(5), Control(0), Cell(1, 2), Paragraph(0), Control(0)]`
pub type DocumentPath = Vec<PathSegment>;

/// 기존 3-tuple (parent_para_idx, control_idx)에서 DocumentPath를 생성한다.
pub fn path_from_flat(parent_para_idx: usize, control_idx: usize) -> DocumentPath {
    vec![
        PathSegment::Paragraph(parent_para_idx),
        PathSegment::Control(control_idx),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_from_flat() {
        let path = path_from_flat(3, 1);
        assert_eq!(
            path,
            vec![PathSegment::Paragraph(3), PathSegment::Control(1),]
        );
    }

    #[test]
    fn test_nested_path_construction() {
        // 구역 내 문단5 → 컨트롤0(표) → 셀(1,2) → 문단0 → 컨트롤0(중첩표)
        let path: DocumentPath = vec![
            PathSegment::Paragraph(5),
            PathSegment::Control(0),
            PathSegment::Cell(1, 2),
            PathSegment::Paragraph(0),
            PathSegment::Control(0),
        ];
        assert_eq!(path.len(), 5);
        assert_eq!(path[2], PathSegment::Cell(1, 2));
    }
}
