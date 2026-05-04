//! HWP 5.0 태그 상수 정의
//!
//! HWP 레코드의 태그 ID 값. HWPTAG_BEGIN (0x010) 기준 오프셋으로 정의.

/// 태그 시작 기준값
pub const HWPTAG_BEGIN: u16 = 0x010;

// ============================================================
// DocInfo 태그 (HWPTAG_BEGIN + 0 ~ 49)
// ============================================================

/// 문서 속성 (섹션 수, 시작 번호 등)
pub const HWPTAG_DOCUMENT_PROPERTIES: u16 = HWPTAG_BEGIN;
/// ID 매핑 테이블 (각 타입별 개수)
pub const HWPTAG_ID_MAPPINGS: u16 = HWPTAG_BEGIN + 1;
/// 바이너리 데이터 참조
pub const HWPTAG_BIN_DATA: u16 = HWPTAG_BEGIN + 2;
/// 글꼴 이름
pub const HWPTAG_FACE_NAME: u16 = HWPTAG_BEGIN + 3;
/// 테두리/채우기
pub const HWPTAG_BORDER_FILL: u16 = HWPTAG_BEGIN + 4;
/// 글자 모양
pub const HWPTAG_CHAR_SHAPE: u16 = HWPTAG_BEGIN + 5;
/// 탭 정의
pub const HWPTAG_TAB_DEF: u16 = HWPTAG_BEGIN + 6;
/// 번호 매기기
pub const HWPTAG_NUMBERING: u16 = HWPTAG_BEGIN + 7;
/// 글머리표
pub const HWPTAG_BULLET: u16 = HWPTAG_BEGIN + 8;
/// 문단 모양
pub const HWPTAG_PARA_SHAPE: u16 = HWPTAG_BEGIN + 9;
/// 스타일
pub const HWPTAG_STYLE: u16 = HWPTAG_BEGIN + 10;
/// 문서 데이터
pub const HWPTAG_DOC_DATA: u16 = HWPTAG_BEGIN + 11;
/// 배포용 문서 데이터 (복호화 시드)
pub const HWPTAG_DISTRIBUTE_DOC_DATA: u16 = HWPTAG_BEGIN + 12;
// (HWPTAG_BEGIN + 13 예약)
/// 호환 문서
pub const HWPTAG_COMPATIBLE_DOCUMENT: u16 = HWPTAG_BEGIN + 14;
/// 레이아웃 호환성
pub const HWPTAG_LAYOUT_COMPATIBILITY: u16 = HWPTAG_BEGIN + 15;
/// 변경 추적
pub const HWPTAG_TRACKCHANGE: u16 = HWPTAG_BEGIN + 16;

// ============================================================
// BodyText 태그 (HWPTAG_BEGIN + 50 ~)
// ============================================================

/// 문단 헤더
pub const HWPTAG_PARA_HEADER: u16 = HWPTAG_BEGIN + 50;
/// 문단 텍스트 (UTF-16LE)
pub const HWPTAG_PARA_TEXT: u16 = HWPTAG_BEGIN + 51;
/// 문단 글자 모양 참조
pub const HWPTAG_PARA_CHAR_SHAPE: u16 = HWPTAG_BEGIN + 52;
/// 문단 줄 세그먼트
pub const HWPTAG_PARA_LINE_SEG: u16 = HWPTAG_BEGIN + 53;
/// 문단 범위 태그
pub const HWPTAG_PARA_RANGE_TAG: u16 = HWPTAG_BEGIN + 54;
/// 컨트롤 헤더
pub const HWPTAG_CTRL_HEADER: u16 = HWPTAG_BEGIN + 55;
/// 리스트 헤더 (셀, 머리말/꼬리말 등의 문단 목록)
pub const HWPTAG_LIST_HEADER: u16 = HWPTAG_BEGIN + 56;
/// 용지 설정
pub const HWPTAG_PAGE_DEF: u16 = HWPTAG_BEGIN + 57;
/// 각주/미주 모양
pub const HWPTAG_FOOTNOTE_SHAPE: u16 = HWPTAG_BEGIN + 58;
/// 쪽 테두리/배경
pub const HWPTAG_PAGE_BORDER_FILL: u16 = HWPTAG_BEGIN + 59;
/// 그리기 개체 속성
pub const HWPTAG_SHAPE_COMPONENT: u16 = HWPTAG_BEGIN + 60;
/// 표 속성
pub const HWPTAG_TABLE: u16 = HWPTAG_BEGIN + 61;
/// 직선
pub const HWPTAG_SHAPE_COMPONENT_LINE: u16 = HWPTAG_BEGIN + 62;
/// 사각형
pub const HWPTAG_SHAPE_COMPONENT_RECTANGLE: u16 = HWPTAG_BEGIN + 63;
/// 타원
pub const HWPTAG_SHAPE_COMPONENT_ELLIPSE: u16 = HWPTAG_BEGIN + 64;
/// 호
pub const HWPTAG_SHAPE_COMPONENT_ARC: u16 = HWPTAG_BEGIN + 65;
/// 다각형
pub const HWPTAG_SHAPE_COMPONENT_POLYGON: u16 = HWPTAG_BEGIN + 66;
/// 곡선
pub const HWPTAG_SHAPE_COMPONENT_CURVE: u16 = HWPTAG_BEGIN + 67;
/// OLE 개체
pub const HWPTAG_SHAPE_COMPONENT_OLE: u16 = HWPTAG_BEGIN + 68;
/// 그림
pub const HWPTAG_SHAPE_COMPONENT_PICTURE: u16 = HWPTAG_BEGIN + 69;
/// 컨테이너 (그리기 묶음)
pub const HWPTAG_SHAPE_COMPONENT_CONTAINER: u16 = HWPTAG_BEGIN + 70;
/// 컨트롤 데이터
pub const HWPTAG_CTRL_DATA: u16 = HWPTAG_BEGIN + 71;
/// 수식
pub const HWPTAG_EQEDIT: u16 = HWPTAG_BEGIN + 72;
// (HWPTAG_BEGIN + 73 예약)
/// 글맵시
pub const HWPTAG_SHAPE_COMPONENT_TEXTART: u16 = HWPTAG_BEGIN + 74;
/// 양식 개체
pub const HWPTAG_FORM_OBJECT: u16 = HWPTAG_BEGIN + 75;
/// 메모 모양
pub const HWPTAG_MEMO_SHAPE: u16 = HWPTAG_BEGIN + 76;
/// 메모 리스트
pub const HWPTAG_MEMO_LIST: u16 = HWPTAG_BEGIN + 77;
/// 금칙 문자
pub const HWPTAG_FORBIDDEN_CHAR: u16 = HWPTAG_BEGIN + 78;
/// 차트 데이터
pub const HWPTAG_CHART_DATA: u16 = HWPTAG_BEGIN + 79;

// ============================================================
// 인라인 컨트롤 코드 (텍스트 내 특수 문자)
// ============================================================

/// 구역/단 정의 컨트롤
pub const CHAR_SECTION_COLUMN_DEF: u16 = 0x0002;
/// 필드 시작
pub const CHAR_FIELD_BEGIN: u16 = 0x0003;
/// 필드 끝
pub const CHAR_FIELD_END: u16 = 0x0004;
/// 인라인 컨트롤 (비텍스트)
pub const CHAR_INLINE_NON_TEXT: u16 = 0x0008;
/// 컨트롤 삽입 위치 (표, 도형, 그림 등)
pub const CHAR_EXTENDED_CTRL: u16 = 0x000B;
/// 문단 끝/나눔
pub const CHAR_LINE_BREAK: u16 = 0x000A;
/// 문단 나눔
pub const CHAR_PARA_BREAK: u16 = 0x000D;
/// 하이픈 (HWP 5.0 표 7: 코드 24)
pub const CHAR_HYPHEN: u16 = 0x0018;
/// 묶음 빈칸 / NO-BREAK SPACE (HWP 5.0 표 7: 코드 30)
pub const CHAR_NBSPACE: u16 = 0x001E;
/// 예약 (스펙 코드 25)
pub const CHAR_FIXED_WIDTH_SPACE: u16 = 0x0019;
/// 고정폭 빈칸 / FIGURE SPACE (HWP 5.0 표 7: 코드 31)
pub const CHAR_FIXED_WIDTH_SPACE_31: u16 = 0x001F;

// ============================================================
// 컨트롤 ID (4바이트 문자열)
// ============================================================

/// 구역 정의
pub const CTRL_SECTION_DEF: u32 = ctrl_id(b"secd");
/// 단 정의
pub const CTRL_COLUMN_DEF: u32 = ctrl_id(b"cold");
/// 표
pub const CTRL_TABLE: u32 = ctrl_id(b"tbl ");
/// 수식
pub const CTRL_EQUATION: u32 = ctrl_id(b"eqed");
/// 그리기 개체
pub const CTRL_GEN_SHAPE: u32 = ctrl_id(b"gso ");
/// 그림 도형 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_PICTURE_ID: u32 = ctrl_id(b"$pic");
/// 사각형 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_RECT_ID: u32 = ctrl_id(b"$rec");
/// 직선 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_LINE_ID: u32 = ctrl_id(b"$lin");
/// 타원 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_ELLIPSE_ID: u32 = ctrl_id(b"$ell");
/// 다각형 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_POLYGON_ID: u32 = ctrl_id(b"$pol");
/// 호 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_ARC_ID: u32 = ctrl_id(b"$arc");
/// 곡선 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_CURVE_ID: u32 = ctrl_id(b"$cur");
/// 연결선 (SHAPE_COMPONENT 내부 ctrl_id)
pub const SHAPE_CONNECTOR_ID: u32 = ctrl_id(b"$col");
/// 머리말
pub const CTRL_HEADER: u32 = ctrl_id(b"head");
/// 꼬리말
pub const CTRL_FOOTER: u32 = ctrl_id(b"foot");
/// 각주
pub const CTRL_FOOTNOTE: u32 = ctrl_id(b"fn  ");
/// 미주
pub const CTRL_ENDNOTE: u32 = ctrl_id(b"en  ");
/// 자동 번호
pub const CTRL_AUTO_NUMBER: u32 = ctrl_id(b"atno");
/// 새 번호
pub const CTRL_NEW_NUMBER: u32 = ctrl_id(b"nwno");
/// 쪽 번호 위치
pub const CTRL_PAGE_NUM_POS: u32 = ctrl_id(b"pgnp");
/// 감추기
pub const CTRL_PAGE_HIDE: u32 = ctrl_id(b"pghd");
/// 찾아보기 표식
pub const CTRL_INDEX_MARK: u32 = ctrl_id(b"idxm");
/// 책갈피
pub const CTRL_BOOKMARK: u32 = ctrl_id(b"bokm");
/// 글자 겹침
pub const CTRL_TCPS: u32 = ctrl_id(b"tcps");
/// 양식 개체
pub const CTRL_FORM: u32 = ctrl_id(b"form");
/// 덧말
pub const CTRL_CHAR_OVERLAP: u32 = ctrl_id(b"tdut");
/// 숨은 설명
pub const CTRL_HIDDEN_COMMENT: u32 = ctrl_id(b"tcmt");

// ============================================================
// 필드 컨트롤 ID (% 접두어)
// ============================================================

/// 필드: 누름틀
pub const FIELD_CLICKHERE: u32 = ctrl_id(b"%clk");
/// 필드: 하이퍼링크
pub const FIELD_HYPERLINK: u32 = ctrl_id(b"%hlk");
/// 필드: 책갈피
pub const FIELD_BOOKMARK: u32 = ctrl_id(b"%bmk");
/// 필드: 현재 날짜/시간
pub const FIELD_DATE: u32 = ctrl_id(b"%dte");
/// 필드: 문서 날짜/시간
pub const FIELD_DOCDATE: u32 = ctrl_id(b"%ddt");
/// 필드: 파일 경로
pub const FIELD_PATH: u32 = ctrl_id(b"%pat");
/// 필드: 메일 머지
pub const FIELD_MAILMERGE: u32 = ctrl_id(b"%mmg");
/// 필드: 상호 참조
pub const FIELD_CROSSREF: u32 = ctrl_id(b"%xrf");
/// 필드: 표 계산식
pub const FIELD_FORMULA: u32 = ctrl_id(b"%fmu");
/// 필드: 문서 요약
pub const FIELD_SUMMARY: u32 = ctrl_id(b"%smr");
/// 필드: 사용자 정보
pub const FIELD_USERINFO: u32 = ctrl_id(b"%usr");
/// 필드: 메모
pub const FIELD_MEMO: u32 = ctrl_id(b"%%me");
/// 필드: 개인정보 보안
pub const FIELD_PRIVATE_INFO: u32 = ctrl_id(b"%cpr");
/// 필드: 차례
pub const FIELD_TOC: u32 = ctrl_id(b"%toc");
/// 필드: 알 수 없음
pub const FIELD_UNKNOWN: u32 = ctrl_id(b"%unk");

/// ctrl_id가 필드 컨트롤인지 확인 (첫 바이트가 '%')
pub const fn is_field_ctrl_id(id: u32) -> bool {
    (id >> 24) == b'%' as u32
}

/// 4바이트 ASCII 문자열을 u32 컨트롤 ID로 변환 (컴파일 타임)
///
/// HWP 파일에서 ctrl_id는 첫 문자가 MSB에 위치하는 big-endian 문자열 인코딩이다.
/// 예: "secd" → 0x73656364 ('s'=MSB, 'd'=LSB)
/// 파일에서는 DWORD(LE)로 저장되므로 바이트 순서: [0x64, 0x63, 0x65, 0x73]
const fn ctrl_id(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}

/// 태그 ID를 사람이 읽을 수 있는 이름으로 변환
pub fn tag_name(tag_id: u16) -> &'static str {
    match tag_id {
        HWPTAG_DOCUMENT_PROPERTIES => "DOCUMENT_PROPERTIES",
        HWPTAG_ID_MAPPINGS => "ID_MAPPINGS",
        HWPTAG_BIN_DATA => "BIN_DATA",
        HWPTAG_FACE_NAME => "FACE_NAME",
        HWPTAG_BORDER_FILL => "BORDER_FILL",
        HWPTAG_CHAR_SHAPE => "CHAR_SHAPE",
        HWPTAG_TAB_DEF => "TAB_DEF",
        HWPTAG_NUMBERING => "NUMBERING",
        HWPTAG_BULLET => "BULLET",
        HWPTAG_PARA_SHAPE => "PARA_SHAPE",
        HWPTAG_STYLE => "STYLE",
        HWPTAG_DOC_DATA => "DOC_DATA",
        HWPTAG_DISTRIBUTE_DOC_DATA => "DISTRIBUTE_DOC_DATA",
        HWPTAG_COMPATIBLE_DOCUMENT => "COMPATIBLE_DOCUMENT",
        HWPTAG_LAYOUT_COMPATIBILITY => "LAYOUT_COMPATIBILITY",
        HWPTAG_TRACKCHANGE => "TRACKCHANGE",
        HWPTAG_PARA_HEADER => "PARA_HEADER",
        HWPTAG_PARA_TEXT => "PARA_TEXT",
        HWPTAG_PARA_CHAR_SHAPE => "PARA_CHAR_SHAPE",
        HWPTAG_PARA_LINE_SEG => "PARA_LINE_SEG",
        HWPTAG_PARA_RANGE_TAG => "PARA_RANGE_TAG",
        HWPTAG_CTRL_HEADER => "CTRL_HEADER",
        HWPTAG_LIST_HEADER => "LIST_HEADER",
        HWPTAG_PAGE_DEF => "PAGE_DEF",
        HWPTAG_FOOTNOTE_SHAPE => "FOOTNOTE_SHAPE",
        HWPTAG_PAGE_BORDER_FILL => "PAGE_BORDER_FILL",
        HWPTAG_SHAPE_COMPONENT => "SHAPE_COMPONENT",
        HWPTAG_TABLE => "TABLE",
        HWPTAG_SHAPE_COMPONENT_LINE => "SHAPE_LINE",
        HWPTAG_SHAPE_COMPONENT_RECTANGLE => "SHAPE_RECTANGLE",
        HWPTAG_SHAPE_COMPONENT_ELLIPSE => "SHAPE_ELLIPSE",
        HWPTAG_SHAPE_COMPONENT_ARC => "SHAPE_ARC",
        HWPTAG_SHAPE_COMPONENT_POLYGON => "SHAPE_POLYGON",
        HWPTAG_SHAPE_COMPONENT_CURVE => "SHAPE_CURVE",
        HWPTAG_SHAPE_COMPONENT_OLE => "SHAPE_OLE",
        HWPTAG_SHAPE_COMPONENT_PICTURE => "SHAPE_PICTURE",
        HWPTAG_SHAPE_COMPONENT_CONTAINER => "SHAPE_CONTAINER",
        HWPTAG_CTRL_DATA => "CTRL_DATA",
        HWPTAG_EQEDIT => "EQEDIT",
        HWPTAG_SHAPE_COMPONENT_TEXTART => "SHAPE_TEXTART",
        HWPTAG_FORM_OBJECT => "FORM_OBJECT",
        HWPTAG_MEMO_SHAPE => "MEMO_SHAPE",
        HWPTAG_MEMO_LIST => "MEMO_LIST",
        HWPTAG_FORBIDDEN_CHAR => "FORBIDDEN_CHAR",
        HWPTAG_CHART_DATA => "CHART_DATA",
        _ => "UNKNOWN",
    }
}

/// 컨트롤 ID를 사람이 읽을 수 있는 이름으로 변환
pub fn ctrl_name(ctrl_id: u32) -> &'static str {
    match ctrl_id {
        CTRL_SECTION_DEF => "SectionDef",
        CTRL_COLUMN_DEF => "ColumnDef",
        CTRL_TABLE => "Table",
        CTRL_EQUATION => "Equation",
        CTRL_GEN_SHAPE => "GenShape",
        CTRL_HEADER => "Header",
        CTRL_FOOTER => "Footer",
        CTRL_FOOTNOTE => "Footnote",
        CTRL_ENDNOTE => "Endnote",
        CTRL_AUTO_NUMBER => "AutoNumber",
        CTRL_NEW_NUMBER => "NewNumber",
        CTRL_PAGE_NUM_POS => "PageNumPos",
        CTRL_PAGE_HIDE => "PageHide",
        CTRL_INDEX_MARK => "IndexMark",
        CTRL_BOOKMARK => "Bookmark",
        CTRL_TCPS => "Tcps",
        CTRL_FORM => "Form",
        CTRL_CHAR_OVERLAP => "CharOverlap",
        CTRL_HIDDEN_COMMENT => "HiddenComment",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_values() {
        assert_eq!(HWPTAG_BEGIN, 0x010);
        assert_eq!(HWPTAG_DOCUMENT_PROPERTIES, 16);
        assert_eq!(HWPTAG_PARA_HEADER, 66);
        assert_eq!(HWPTAG_PARA_TEXT, 67);
        assert_eq!(HWPTAG_TABLE, 77);
    }

    #[test]
    fn test_ctrl_id() {
        assert_eq!(CTRL_TABLE, u32::from_be_bytes(*b"tbl "));
        assert_eq!(CTRL_SECTION_DEF, u32::from_be_bytes(*b"secd"));
        assert_eq!(CTRL_HEADER, u32::from_be_bytes(*b"head"));
        assert_eq!(CTRL_FOOTER, u32::from_be_bytes(*b"foot"));
    }

    #[test]
    fn test_tag_name() {
        assert_eq!(tag_name(HWPTAG_PARA_HEADER), "PARA_HEADER");
        assert_eq!(tag_name(HWPTAG_CHAR_SHAPE), "CHAR_SHAPE");
        assert_eq!(tag_name(0xFFFF), "UNKNOWN");
    }

    #[test]
    fn test_ctrl_name() {
        assert_eq!(ctrl_name(CTRL_TABLE), "Table");
        assert_eq!(ctrl_name(CTRL_GEN_SHAPE), "GenShape");
        assert_eq!(ctrl_name(0), "Unknown");
    }
}
