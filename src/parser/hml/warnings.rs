#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmlWarningCode {
    UnsupportedElement,
    UnsupportedAttribute,
    UnsupportedEquationSemantics,
    MissingResource,
    ExternalResourceBlocked,
    InvalidReference,
    LossyConversion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HmlWarning {
    pub code: HmlWarningCode,
    pub xml_path: String,
    pub message: String,
    /// 건너뛴 서브트리가 저장 시 원문 그대로 되돌려질 수 있는지 여부.
    /// TAIL/HEAD 바로 아래에서 건너뛴 서브트리만 보존 캡슐에 원문이 저장된다.
    pub preserved: bool,
}

impl HmlWarning {
    pub(crate) fn unsupported_element(xml_path: String, element: &str, preserved: bool) -> Self {
        Self {
            code: HmlWarningCode::UnsupportedElement,
            xml_path,
            message: format!("지원하지 않는 HML 요소를 건너뛰었습니다: {element}"),
            preserved,
        }
    }

    pub(crate) fn unsupported_attribute(xml_path: String, attribute: &str) -> Self {
        Self {
            code: HmlWarningCode::UnsupportedAttribute,
            xml_path,
            message: format!("지원하지 않는 HML 속성을 건너뛰었습니다: {attribute}"),
            preserved: false,
        }
    }

    pub(crate) fn unsupported_equation_semantics(xml_path: String, semantics: &str) -> Self {
        Self {
            code: HmlWarningCode::UnsupportedEquationSemantics,
            xml_path,
            message: format!("보존할 수 없는 HML 수식 의미를 건너뛰었습니다: {semantics}"),
            preserved: false,
        }
    }

    pub(crate) fn invalid_reference(xml_path: String, reference: String) -> Self {
        Self {
            code: HmlWarningCode::InvalidReference,
            xml_path,
            message: format!("잘못된 HML 리소스 참조를 기본값으로 대체했습니다: {reference}"),
            preserved: false,
        }
    }
}
