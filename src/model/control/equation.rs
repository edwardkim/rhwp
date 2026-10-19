//! 원본 수식 속성과 선언 프레임의 런타임 출처.

use crate::model::shape::CommonObjAttr;

/// 수식 ('eqed' 컨트롤, HWP 스펙 표 105)
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Equation {
    /// 개체 공통 속성 (위치, 크기, 배치)
    pub common: CommonObjAttr,
    /// [#2727] HWPTAG_EQEDIT 속성 (HWP5 spec 표 105 attribute, UINT32).
    ///
    /// bit 0 = 수식이 차지하는 범위 (0 = 글자 단위, 1 = 줄 단위) — HWPX
    /// `hp:equation@lineMode` (`CHAR` / `LINE`) 와 대응한다. 나머지 비트는 의미
    /// 미상이므로 UINT32 전체를 원본 그대로 보존해 왕복시킨다. 기본값 0 은
    /// OWPML `lineMode` 기본값 `CHAR` 와 일치한다.
    pub attr: u32,
    /// 수식 스크립트 ("1 over 2" 등)
    pub script: String,
    /// 글자 크기 (HWPUNIT)
    pub font_size: u32,
    /// 글자 색 (0x00BBGGRR)
    pub color: u32,
    /// 기준선 오프셋
    pub baseline: i16,
    /// 미지의 UINT16 필드 (HWP5 spec errata) — hwplib `ForEQEdit.readUInt2()` 정합.
    /// HWP5 spec 표 105 에 누락되어 있으나 한컴 실제 저장본에 baseline 과 version_info
    /// 사이에 UINT16 zero 가 위치. Task #1061 발견.
    pub unknown: u16,
    /// EQEDIT 속성 (UINT32, HWPTAG_EQEDIT 첫 필드).
    /// bit 0: lineMode (0=글자 단위/CHAR, 1=줄 단위/LINE).
    /// 종전엔 파싱 후 버려지고 저장 시 0으로 고정되어 lineMode 유실. Issue #2727.
    pub eqedit: u32,
    /// 버전 정보
    pub version_info: String,
    /// 수식 글꼴명
    pub font_name: String,
    /// 라운드트립용 원본 ctrl_data
    pub raw_ctrl_data: Vec<u8>,
    /// [#4495] raw_ctrl_data 출처 봉인 — 봉인 시점 `common` 의 다이제스트.
    /// 계약은 Table::raw_ctrl_seal 과 동일 (`model::raw_provenance` 참조).
    #[serde(skip)]
    pub raw_ctrl_seal: Option<[u8; 32]>,
    /// 미계산 본문에서 가져온 선언 수식 프레임의 런타임 출처.
    /// 파일 속성이 아니며, 관련 수식 값이 바뀌면 렌더러가 재사용하지 않는다.
    #[serde(skip)]
    pub unpositioned_host_frame_seal: Option<[u8; 32]>,
}
