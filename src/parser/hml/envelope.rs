//! HML 저장 시 손실 없이 되돌려 쓰기 위한 미지원 서브트리 원문 보존 캡슐.
//!
//! 파서가 건너뛴 요소 중 `HEAD`/`BODY`/`TAIL` 바로 아래에 위치한 것만 원문 XML을
//! 바이트 단위로 그대로 캡처한다 (본문 인라인 미지원 요소는 대상이 아님).

/// 미지원 서브트리 하나의 원문 캡처 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreservedFragment {
    /// 캡처된 서브트리의 직계 부모 요소 이름 ("HEAD", "BODY", "TAIL").
    pub parent: String,
    /// 같은 부모 아래에서의 등장 순서 (0부터).
    pub order: usize,
    /// 원본 위치보다 앞에 있던, serializer가 다시 생성하는 형제 수.
    /// HEAD에서는 MAPPINGTABLE, BODY에서는 SECTION 수를 기준으로 위치를 뜻한다.
    pub modeled_siblings_before: usize,
    /// 경고와 대응하는 xml_path (예: "/HWPML/TAIL/SCRIPTCODE").
    pub xml_path: String,
    /// 시작 태그부터 종료 태그까지의 원문 XML (바이트 그대로).
    pub raw_xml: String,
}
