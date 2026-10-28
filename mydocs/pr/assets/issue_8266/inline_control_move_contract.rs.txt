//! 인라인 이동 계약 검사에서 사용하는 공개 API 주소와 차트 내용 조회.
use rhwp::document_core::{
    inline_control::{InlineControlAddress, InlineControlCaret, InlineControlOwner},
    DocumentCore,
};

pub(super) fn body(para: usize) -> InlineControlOwner {
    InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: para,
        cell_path: vec![],
    }
}
pub(super) fn source(owner: InlineControlOwner, ci: usize) -> InlineControlAddress {
    InlineControlAddress {
        owner,
        control_index: ci,
    }
}
pub(super) fn caret(owner: InlineControlOwner, offset: usize) -> InlineControlCaret {
    InlineControlCaret {
        owner,
        char_offset: offset,
    }
}
pub(super) fn chart_content(core: &DocumentCore) -> serde_json::Value {
    let mut data: serde_json::Value =
        serde_json::from_str(&core.get_chart_data_by_index_native(0).unwrap()).unwrap();
    assert_eq!(data["ok"].as_bool(), Some(true));
    // 저장 형식마다 달라지는 위치 정보 대신 차트 내용을 비교한다.
    let object = data.as_object_mut().unwrap();
    object.remove("source");
    object.remove("representations");
    data
}
