//! [#3762] `export-ir-schema` — 공개 IR 의 JSON Schema 를 기계 산출한다.
//!
//! 외부 소비자가 코드 생성을 하려면 **IR 의 모양이 기계가 읽을 수 있는 형태**로
//! 나와야 한다. capabilities 가 명령 표면의 자기서술이라면, 이 스키마는
//! **문서 모델의 자기서술**이다.
//!
//! ## 왜 손으로 쓰는가
//!
//! serde 파생에서 자동 추출하면 "직렬화 표현"이 새어 나온다 — 라운드트립 보존용
//! 원본 바이트(`raw_stream`·`extra_streams`)나 내부 shim(`is_hwp3_variant`)처럼
//! **바인딩이 알 필요도 없고 알아서도 안 되는** 필드까지 공개 계약이 된다. 여기서
//! 명시적으로 쓰는 목록이 곧 "우리가 외부에 약속하는 IR"이다.
//!
//! ## 버저닝
//!
//! `irSchemaVersion` 은 봉투 `schemaVersion`(명령별)과 **분리**된 전역 버전이다.
//! 필드 추가 = minor, 의미 변경·삭제 = major. major 는 분기 회고 승인 없이 금지.

use serde_json::{json, Value};

use crate::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// 공개 IR 스키마 버전 — 단일 출처는 [`crate::schema_registry`](#4329). 여기서는
/// 재수출만 해 기존 호출부 경로를 보존한다.
pub use crate::schema_registry::IR_SCHEMA_VERSION;

/// JSON Schema draft — 소비자(코드 생성기)가 파서를 고를 수 있게 명시한다.
const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// `$defs` 참조 하나를 만든다.
fn r(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

/// 설명이 달린 원시 타입.
fn prim(ty: &str, description: &str) -> Value {
    json!({ "type": ty, "description": description })
}

/// 정수 + 하한.
fn uint(description: &str) -> Value {
    json!({ "type": "integer", "minimum": 0, "description": description })
}

/// 배열 타입.
fn array_of(items: Value, description: &str) -> Value {
    json!({ "type": "array", "items": items, "description": description })
}

/// 객체 타입 — 필수 필드를 명시하고 추가 필드를 허용한다.
///
/// `additionalProperties: true` 는 의도적이다. IR 은 **추가-전용 진화** 계약이므로
/// 새 필드가 붙어도 기존 소비자가 깨지지 않아야 한다. false 로 두면 rhwp 가 필드를
/// 하나 더할 때마다 모든 바인딩이 동시에 실패한다.
fn object(properties: Value, required: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required,
        "additionalProperties": true,
    })
}

/// 열거형 — 허용 값과 각 값의 뜻.
fn enum_of(values: &[(&str, &str)], description: &str) -> Value {
    let names: Vec<&str> = values.iter().map(|(n, _)| *n).collect();
    let doc = values
        .iter()
        .map(|(n, d)| format!("{n}={d}"))
        .collect::<Vec<_>>()
        .join(", ");
    json!({
        "type": "string",
        "enum": names,
        "description": format!("{description} ({doc})"),
    })
}

// ── 최상위 ───────────────────────────────────────────────────────────────

fn document_def() -> Value {
    object(
        json!({
            "header": r("FileHeader"),
            "docProperties": r("DocProperties"),
            "docInfo": r("DocInfo"),
            "sections": array_of(r("Section"), "본문 구역 목록. 최소 1개."),
            "preview": json!({
                "oneOf": [r("Preview"), { "type": "null" }],
                "description": "미리보기(PrvImage/PrvText). 없으면 null.",
            }),
            "provenance": r("Provenance"),
        }),
        &[
            "header",
            "docProperties",
            "docInfo",
            "sections",
            "provenance",
        ],
        "문서 하나의 공개 IR. 모든 포맷(HWP5·HWPX·HWP3·HML) 파서가 이 모양을 돌려준다.",
    )
}

fn file_header_def() -> Value {
    object(
        json!({
            "version": r("HwpVersion"),
            "compressed": prim("boolean", "본문 스트림 압축 여부"),
            "encrypted": prim("boolean", "암호 보호 문서 여부"),
            "distributed": prim("boolean", "배포용 문서 여부"),
        }),
        &["version"],
        "파일 헤더 — 포맷 버전과 저장 속성.",
    )
}

fn hwp_version_def() -> Value {
    object(
        json!({
            "major": uint("주 버전"),
            "minor": uint("부 버전"),
            "build": uint("빌드 번호"),
            "revision": uint("리비전"),
        }),
        &["major", "minor", "build", "revision"],
        "HWP 파일 포맷 버전 (5.0.3.0 형식).",
    )
}

fn doc_properties_def() -> Value {
    object(
        json!({
            "sectionCount": uint("구역 수"),
            "pageStartNumber": uint("시작 쪽 번호"),
            "footnoteStartNumber": uint("각주 시작 번호"),
            "endnoteStartNumber": uint("미주 시작 번호"),
            "pictureStartNumber": uint("그림 시작 번호"),
            "tableStartNumber": uint("표 시작 번호"),
            "equationStartNumber": uint("수식 시작 번호"),
        }),
        &["sectionCount"],
        "문서 속성 — 번호 매기기 시작값.",
    )
}

fn doc_info_def() -> Value {
    object(
        json!({
            "fontFaces": array_of(r("FontFace"), "글꼴 목록. charShapes[].fontId 가 이 배열을 가리킨다."),
            "charShapes": array_of(r("CharShape"), "글자 모양 목록."),
            "paraShapes": array_of(r("ParaShape"), "문단 모양 목록."),
            "borderFills": array_of(r("BorderFill"), "테두리·채우기 목록."),
            "styles": array_of(r("Style"), "스타일(문단 서식 묶음) 목록."),
            "numberings": array_of(r("Numbering"), "번호 매기기 정의 목록."),
            "bullets": array_of(r("Bullet"), "글머리표 정의 목록."),
            "tabDefs": array_of(r("TabDef"), "탭 정의 목록."),
        }),
        &["charShapes", "paraShapes"],
        "문서 전역 서식 테이블. 문단·글자는 여기의 인덱스를 참조한다 (정규화된 IR).",
    )
}

// ── 구역·문단 ────────────────────────────────────────────────────────────

fn section_def() -> Value {
    object(
        json!({
            "sectionDef": r("SectionDef"),
            "paragraphs": array_of(r("Paragraph"), "문단 목록 (본문 순서)."),
        }),
        &["sectionDef", "paragraphs"],
        "구역 하나 — 쪽 설정이 같은 문단 묶음.",
    )
}

fn section_settings_def() -> Value {
    object(
        json!({
            "pageWidth": uint("용지 너비 (HWPUNIT, 1/7200 inch)"),
            "pageHeight": uint("용지 높이 (HWPUNIT)"),
            "marginLeft": uint("왼쪽 여백 (HWPUNIT)"),
            "marginRight": uint("오른쪽 여백 (HWPUNIT)"),
            "marginTop": uint("위 여백 (HWPUNIT)"),
            "marginBottom": uint("아래 여백 (HWPUNIT)"),
            "marginHeader": uint("머리말 여백 (HWPUNIT)"),
            "marginFooter": uint("꼬리말 여백 (HWPUNIT)"),
            "marginGutter": uint("제본 여백 (HWPUNIT)"),
            "landscape": prim("boolean", "가로 방향 여부"),
            "columnCount": uint("단 수"),
        }),
        &["pageWidth", "pageHeight"],
        "구역 쪽 설정. 길이 단위는 전부 HWPUNIT (1/7200 inch).",
    )
}

fn paragraph_def() -> Value {
    object(
        json!({
            "text": prim("string", "문단 텍스트. 컨트롤 자리에는 제어 문자가 들어간다."),
            "charCount": uint("문자 수 (제어 문자 포함, UTF-16 코드 유닛 기준)"),
            "paraShapeId": uint("docInfo.paraShapes 인덱스"),
            "styleId": uint("docInfo.styles 인덱스"),
            "columnType": r("ColumnBreakType"),
            "charShapes": array_of(r("CharShapeRef"), "글자 모양이 바뀌는 지점 목록."),
            "lineSegs": array_of(r("LineSeg"), "줄 레이아웃 (조판 결과)."),
            "fieldRanges": array_of(r("FieldRange"), "누름틀 텍스트 범위."),
            "controls": array_of(r("Control"), "이 문단에 달린 컨트롤 (표·그림·각주 등)."),
        }),
        &["text", "charCount", "paraShapeId", "controls"],
        "문단 하나. 텍스트와 그 위의 서식 참조·컨트롤을 담는다.",
    )
}

fn char_shape_ref_def() -> Value {
    object(
        json!({
            "position": uint("적용 시작 위치 (UTF-16 코드 유닛 오프셋)"),
            "charShapeId": uint("docInfo.charShapes 인덱스"),
        }),
        &["position", "charShapeId"],
        "이 위치부터 글자 모양이 바뀐다.",
    )
}

fn line_seg_def() -> Value {
    object(
        json!({
            "textStart": uint("이 줄이 시작하는 텍스트 오프셋"),
            "verticalPos": prim("integer", "세로 위치 (HWPUNIT)"),
            "lineHeight": prim("integer", "줄 높이 (HWPUNIT)"),
            "textHeight": prim("integer", "텍스트 높이 (HWPUNIT)"),
            "baseLineGap": prim("integer", "베이스라인 간격 (HWPUNIT)"),
            "lineSpacing": prim("integer", "줄 간격 (HWPUNIT)"),
            "startPos": prim("integer", "가로 시작 위치 (HWPUNIT)"),
            "segWidth": prim("integer", "세그먼트 너비 (HWPUNIT)"),
        }),
        &["textStart", "verticalPos"],
        "줄 하나의 조판 결과. 쪽 번호를 계산하는 근거다.",
    )
}

fn field_range_def() -> Value {
    object(
        json!({
            "name": prim("string", "누름틀 이름"),
            "startIndex": uint("시작 텍스트 인덱스"),
            "endIndex": uint("끝 텍스트 인덱스"),
            "value": prim("string", "현재 채워진 값"),
        }),
        &["startIndex", "endIndex"],
        "누름틀(필드) 하나가 덮는 텍스트 범위. fill_fields 가 쓰는 좌표.",
    )
}

// ── 컨트롤 ───────────────────────────────────────────────────────────────

fn control_def() -> Value {
    // json! 매크로 안에 변형을 나열하면 재귀 한도에 걸린다 — 배열을 먼저 만든다.
    let variants: Vec<Value> = [
        "TableControl",
        "ShapeControl",
        "PictureControl",
        "FootnoteControl",
        "FieldControl",
        "HeaderFooterControl",
        "EquationControl",
        "BookmarkControl",
        "HyperlinkControl",
        "AutoNumberControl",
        "PageNumberControl",
        "ColumnDefControl",
        "HiddenCommentControl",
        "RubyControl",
        "OtherControl",
    ]
    .iter()
    .map(|name| r(name))
    .collect();
    json!({
        "description":
            "문단에 달린 컨트롤. `kind` 로 갈라지는 태그 유니온이다. 소비자는 모르는 kind 를              만나면 OtherControl 로 취급해야 한다 — 새 컨트롤이 추가돼도 깨지지 않는다.",
        "oneOf": variants,
        "discriminator": { "propertyName": "kind" },
    })
}

fn picture_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "picture", "description": "판별자" }),
            "binDataId": uint("bin_data 참조 id — 실제 이미지 바이트를 가리킨다"),
            "x": prim("integer", "가로 위치 (HWPUNIT)"),
            "y": prim("integer", "세로 위치 (HWPUNIT)"),
            "width": uint("표시 너비 (HWPUNIT)"),
            "height": uint("표시 높이 (HWPUNIT)"),
            "originalWidth": uint("원본 너비 (HWPUNIT)"),
            "originalHeight": uint("원본 높이 (HWPUNIT)"),
            "cropLeft": prim("integer", "왼쪽 자르기 (HWPUNIT)"),
            "cropRight": prim("integer", "오른쪽 자르기 (HWPUNIT)"),
            "cropTop": prim("integer", "위 자르기 (HWPUNIT)"),
            "cropBottom": prim("integer", "아래 자르기 (HWPUNIT)"),
            "alt": prim("string", "대체 텍스트 (접근성)"),
            "textWrap": r("TextWrap"),
        }),
        &["kind"],
        "그림 컨트롤. 도장·서명 삽입의 대상이다.",
    )
}

fn equation_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "equation", "description": "판별자" }),
            "script": prim("string", "수식 스크립트 (한글 수식 문법)"),
            "baseUnit": uint("기준 글자 크기 (HWPUNIT)"),
            "width": uint("너비 (HWPUNIT)"),
            "height": uint("높이 (HWPUNIT)"),
        }),
        &["kind"],
        "수식 컨트롤. script 가 원문이고 렌더는 그것으로 다시 조판한다.",
    )
}

fn bookmark_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "bookmark", "description": "판별자" }),
            "name": prim("string", "책갈피 이름 — 하이퍼링크 대상이 된다"),
        }),
        &["kind", "name"],
        "책갈피 컨트롤.",
    )
}

fn hyperlink_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "hyperlink", "description": "판별자" }),
            "target": prim("string", "링크 대상 (URL 또는 문서 내 책갈피)"),
            "tooltip": prim("string", "설명 풍선 문구"),
        }),
        &["kind"],
        "하이퍼링크 컨트롤.",
    )
}

fn auto_number_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "autoNumber", "description": "판별자" }),
            "numberType": enum_of(
                &[
                    ("page", "쪽 번호"),
                    ("footnote", "각주 번호"),
                    ("endnote", "미주 번호"),
                    ("picture", "그림 번호"),
                    ("table", "표 번호"),
                    ("equation", "수식 번호"),
                    ("totalPage", "전체 쪽수"),
                ],
                "번호 종류",
            ),
            "numberShape": uint("번호 표시 서식"),
        }),
        &["kind"],
        "자동 번호 컨트롤 — 쪽·각주·표 번호가 여기서 나온다.",
    )
}

fn page_number_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "pageNumber", "description": "판별자" }),
            "position": enum_of(
                &[
                    ("none", "없음"),
                    ("topLeft", "위 왼쪽"),
                    ("topCenter", "위 가운데"),
                    ("topRight", "위 오른쪽"),
                    ("bottomLeft", "아래 왼쪽"),
                    ("bottomCenter", "아래 가운데"),
                    ("bottomRight", "아래 오른쪽"),
                    ("outsideTop", "바깥쪽 위"),
                    ("outsideBottom", "바깥쪽 아래"),
                    ("insideTop", "안쪽 위"),
                    ("insideBottom", "안쪽 아래"),
                ],
                "쪽 번호 위치",
            ),
        }),
        &["kind"],
        "쪽 번호 위치 컨트롤.",
    )
}

fn column_def_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "columnDef", "description": "판별자" }),
            "count": uint("단 수"),
            "columnType": enum_of(
                &[("normal", "일반"), ("distribute", "배분"), ("parallel", "평행")],
                "단 종류",
            ),
            "gap": uint("단 간격 (HWPUNIT)"),
            "sameWidth": prim("boolean", "단 너비 동일 여부"),
        }),
        &["kind"],
        "단 정의 컨트롤 — 문단 위치에서 단 구성이 바뀐다.",
    )
}

fn hidden_comment_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "hiddenComment", "description": "판별자" }),
            "paragraphs": array_of(r("Paragraph"), "숨은 설명 본문."),
        }),
        &["kind"],
        "숨은 설명(메모) 컨트롤 — 인쇄되지 않는 주석.",
    )
}

fn ruby_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "ruby", "description": "판별자" }),
            "mainText": prim("string", "본문 텍스트"),
            "rubyText": prim("string", "덧말(윗주) 텍스트"),
            "position": enum_of(
                &[("above", "위"), ("below", "아래")],
                "덧말 위치",
            ),
        }),
        &["kind"],
        "덧말(루비) 컨트롤 — 한자 음 표기 등.",
    )
}

fn text_wrap_def() -> Value {
    object(
        json!({
            "style": enum_of(
                &[
                    ("square", "어울림"),
                    ("tight", "자리 차지"),
                    ("through", "글 뒤로"),
                    ("topAndBottom", "위/아래"),
                    ("behindText", "글 뒤로"),
                    ("inFrontOfText", "글 앞으로"),
                    ("inline", "글자처럼 취급"),
                ],
                "본문과의 배치",
            ),
            "marginLeft": uint("왼쪽 바깥 여백 (HWPUNIT)"),
            "marginRight": uint("오른쪽 바깥 여백 (HWPUNIT)"),
            "marginTop": uint("위 바깥 여백 (HWPUNIT)"),
            "marginBottom": uint("아래 바깥 여백 (HWPUNIT)"),
        }),
        &[],
        "개체와 본문의 배치 관계.",
    )
}

fn table_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "table", "description": "판별자" }),
            "rowCount": uint("행 수"),
            "colCount": uint("열 수"),
            "cells": array_of(r("TableCell"), "셀 목록 (병합 포함)."),
            "borderFillId": uint("docInfo.borderFills 인덱스"),
        }),
        &["kind", "rowCount", "colCount", "cells"],
        "표 컨트롤. set_cell 의 대상이다.",
    )
}

fn table_cell_def() -> Value {
    object(
        json!({
            "row": uint("행 (0 기준)"),
            "col": uint("열 (0 기준)"),
            "rowSpan": uint("세로 병합 칸 수 (1 이면 병합 없음)"),
            "colSpan": uint("가로 병합 칸 수"),
            "width": uint("너비 (HWPUNIT)"),
            "height": uint("높이 (HWPUNIT)"),
            "paragraphs": array_of(r("Paragraph"), "셀 안의 문단 (중첩 구조)."),
        }),
        &["row", "col", "rowSpan", "colSpan", "paragraphs"],
        "표 셀 하나. 병합된 셀은 좌상단 좌표 하나로만 나타난다 — 덮인 좌표는 목록에 없다.",
    )
}

fn shape_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "shape", "description": "판별자" }),
            "shapeType": enum_of(
                &[
                    ("picture", "그림"),
                    ("rectangle", "사각형"),
                    ("ellipse", "타원"),
                    ("line", "선"),
                    ("polygon", "다각형"),
                    ("arc", "호"),
                    ("curve", "곡선"),
                    ("textBox", "글상자"),
                    ("ole", "OLE 개체"),
                    ("container", "묶음 개체"),
                ],
                "도형 종류",
            ),
            "x": prim("integer", "가로 위치 (HWPUNIT)"),
            "y": prim("integer", "세로 위치 (HWPUNIT)"),
            "width": uint("너비 (HWPUNIT)"),
            "height": uint("높이 (HWPUNIT)"),
            "binDataId": uint("그림일 때 bin_data 참조 id"),
            "textWrap": r("TextWrap"),
            "rotation": prim("integer", "회전 각도 (1/100 도)"),
            "flipHorizontal": prim("boolean", "좌우 뒤집기"),
            "flipVertical": prim("boolean", "상하 뒤집기"),
        }),
        &["kind", "shapeType"],
        "도형 컨트롤 (선·사각형·타원·다각형·글상자 등).",
    )
}

fn footnote_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "footnote", "description": "판별자" }),
            "isEndnote": prim("boolean", "미주 여부 (거짓이면 각주)"),
            "number": uint("번호"),
            "paragraphs": array_of(r("Paragraph"), "각주 본문."),
        }),
        &["kind", "isEndnote"],
        "각주·미주 컨트롤.",
    )
}

fn field_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "field", "description": "판별자" }),
            "fieldType": enum_of(
                &[
                    ("clickHere", "누름틀"),
                    ("bookmark", "책갈피"),
                    ("hyperlink", "하이퍼링크"),
                    ("formula", "계산식"),
                    ("memo", "메모"),
                    ("unknown", "그 밖"),
                ],
                "필드 종류",
            ),
            "name": prim("string", "필드 이름"),
            "instruction": prim("string", "필드 지시문"),
        }),
        &["kind", "fieldType"],
        "필드 컨트롤 — 누름틀·책갈피·하이퍼링크.",
    )
}

fn header_footer_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "headerFooter", "description": "판별자" }),
            "isFooter": prim("boolean", "꼬리말 여부 (거짓이면 머리말)"),
            "applyTo": enum_of(
                &[("both", "양쪽"), ("even", "짝수 쪽"), ("odd", "홀수 쪽")],
                "적용 대상",
            ),
            "paragraphs": array_of(r("Paragraph"), "머리말·꼬리말 본문."),
        }),
        &["kind", "isFooter"],
        "머리말·꼬리말 컨트롤.",
    )
}

fn other_control_def() -> Value {
    object(
        json!({
            "kind": json!({ "const": "other", "description": "판별자" }),
            "ctrlId": prim("string", "컨트롤 4바이트 식별자 (예: 'tbl ', 'secd')"),
        }),
        &["kind", "ctrlId"],
        "IR 이 아직 세분화하지 않은 컨트롤. 라운드트립은 보존되지만 구조 접근은 제한된다.",
    )
}

// ── 서식 ─────────────────────────────────────────────────────────────────

fn font_face_def() -> Value {
    object(
        json!({
            "name": prim("string", "글꼴 이름"),
            "substituteName": prim("string", "대체 글꼴 이름"),
            "type": prim("string", "글꼴 종류 (ttf/htf 등)"),
        }),
        &["name"],
        "글꼴 하나.",
    )
}

fn char_shape_def() -> Value {
    object(
        json!({
            "fontIds": array_of(uint("언어별 글꼴 인덱스"), "언어(한글·영문·한자…)별 fontFaces 인덱스"),
            "baseSize": prim("integer", "기본 크기 (HWPUNIT, 1pt = 100)"),
            "bold": prim("boolean", "굵게"),
            "italic": prim("boolean", "기울임"),
            "underline": prim("boolean", "밑줄"),
            "strikeout": prim("boolean", "취소선"),
            "textColor": prim("integer", "글자색 (0xBBGGRR)"),
            "shadeColor": prim("integer", "음영색 (0xBBGGRR)"),
        }),
        &["baseSize"],
        "글자 모양. 색은 HWP 관례대로 BGR 순서다.",
    )
}

fn para_shape_def() -> Value {
    object(
        json!({
            "alignment": enum_of(
                &[
                    ("justify", "양쪽 정렬"),
                    ("left", "왼쪽"),
                    ("right", "오른쪽"),
                    ("center", "가운데"),
                    ("distribute", "배분"),
                    ("divide", "나눔"),
                ],
                "정렬",
            ),
            "leftMargin": prim("integer", "왼쪽 여백 (HWPUNIT)"),
            "rightMargin": prim("integer", "오른쪽 여백 (HWPUNIT)"),
            "indent": prim("integer", "들여쓰기 (HWPUNIT, 음수면 내어쓰기)"),
            "spacingTop": prim("integer", "문단 위 간격 (HWPUNIT)"),
            "spacingBottom": prim("integer", "문단 아래 간격 (HWPUNIT)"),
            "lineSpacing": prim("integer", "줄 간격"),
            "lineSpacingType": uint("줄 간격 종류 (0=비율, 1=고정, 2=여백만)"),
        }),
        &["alignment"],
        "문단 모양.",
    )
}

fn border_fill_def() -> Value {
    object(
        json!({
            "left": r("BorderLine"),
            "right": r("BorderLine"),
            "top": r("BorderLine"),
            "bottom": r("BorderLine"),
            "fillType": uint("채우기 종류 (0=없음, 1=단색, 2=그러데이션, 4=이미지)"),
            "backgroundColor": prim("integer", "배경색 (0xBBGGRR)"),
        }),
        &[],
        "테두리·채우기 묶음. 표 셀과 문단이 인덱스로 참조한다.",
    )
}

fn border_line_def() -> Value {
    object(
        json!({
            "type": uint("선 종류 (0=없음, 1=실선, 2=점선 …)"),
            "width": uint("선 굵기"),
            "color": prim("integer", "선 색 (0xBBGGRR)"),
        }),
        &["type"],
        "테두리 선 하나.",
    )
}

fn style_def() -> Value {
    object(
        json!({
            "name": prim("string", "스타일 이름 (한글)"),
            "englishName": prim("string", "스타일 이름 (영문)"),
            "paraShapeId": uint("docInfo.paraShapes 인덱스"),
            "charShapeId": uint("docInfo.charShapes 인덱스"),
            "styleType": uint("종류 (0=문단, 1=글자)"),
        }),
        &["name"],
        "스타일 — 문단·글자 모양의 이름 붙은 묶음.",
    )
}

fn numbering_def() -> Value {
    object(
        json!({
            "levels": array_of(
                object(
                    json!({
                        "format": prim("string", "번호 서식 (예: '^1.')"),
                        "startNumber": uint("시작 번호"),
                        "alignment": uint("정렬"),
                    }),
                    &[],
                    "수준 하나",
                ),
                "수준별 정의 (최대 7단계)",
            ),
        }),
        &[],
        "번호 매기기 정의.",
    )
}

fn bullet_def() -> Value {
    object(
        json!({
            "char": prim("string", "글머리표 문자"),
            "useImage": prim("boolean", "이미지 글머리표 여부"),
        }),
        &[],
        "글머리표 정의.",
    )
}

fn tab_def_def() -> Value {
    object(
        json!({
            "autoTabLeft": prim("boolean", "왼쪽 자동 탭"),
            "autoTabRight": prim("boolean", "오른쪽 자동 탭"),
            "tabs": array_of(
                object(
                    json!({
                        "position": uint("탭 위치 (HWPUNIT)"),
                        "type": uint("탭 종류 (0=왼쪽, 1=오른쪽, 2=가운데, 3=소수점)"),
                        "leader": uint("채움 문자 종류"),
                    }),
                    &["position"],
                    "탭 하나",
                ),
                "탭 목록",
            ),
        }),
        &[],
        "탭 정의.",
    )
}

// ── 기타 ─────────────────────────────────────────────────────────────────

fn preview_def() -> Value {
    object(
        json!({
            "text": prim("string", "미리보기 텍스트 (PrvText)"),
            "hasImage": prim("boolean", "미리보기 이미지(PrvImage) 존재 여부"),
            "imageFormat": enum_of(
                &[("bmp", "BMP"), ("gif", "GIF"), ("jpeg", "JPEG"), ("png", "PNG"), ("unknown", "미상")],
                "이미지 형식",
            ),
        }),
        &[],
        "문서 미리보기.",
    )
}

fn provenance_def() -> Value {
    object(
        json!({
            "sourceFormat": enum_of(
                &[
                    ("hwp5", "HWP 5.x 바이너리"),
                    ("hwpx", "HWPX (OWPML)"),
                    ("hwp3", "HWP 3.x 레거시"),
                    ("hml", "HML XML"),
                ],
                "원본 포맷",
            ),
            "converted": prim("boolean", "다른 포맷에서 변환된 문서인지"),
        }),
        &["sourceFormat"],
        "문서 출처 — 파서가 확정하는 단일 진실. 레이아웃 분기가 이 값을 본다.",
    )
}

fn column_break_type_def() -> Value {
    enum_of(
        &[
            ("none", "나누지 않음"),
            ("column", "단 나누기"),
            ("page", "쪽 나누기"),
            ("section", "구역 나누기"),
        ],
        "문단 앞 나누기 종류",
    )
}

// ── 조립 ─────────────────────────────────────────────────────────────────

/// 전체 IR JSON Schema 를 만든다.
///
/// 반환 구조는 JSON Schema 2020-12 이고, 최상위에 `irSchemaVersion` 을 덧붙여
/// 소비자가 스키마 자체의 버전을 알 수 있게 한다 (스키마의 스키마 문제를 피하려고
/// `$defs` 밖 최상위 키로 둔다).
pub fn ir_schema() -> Value {
    // 정의가 40개를 넘어 json! 매크로 재귀 한도에 걸린다 — 맵으로 조립한다.
    let defs: serde_json::Map<String, Value> = [
        ("Document", document_def()),
        ("FileHeader", file_header_def()),
        ("HwpVersion", hwp_version_def()),
        ("DocProperties", doc_properties_def()),
        ("DocInfo", doc_info_def()),
        ("Section", section_def()),
        ("SectionDef", section_settings_def()),
        ("Paragraph", paragraph_def()),
        ("CharShapeRef", char_shape_ref_def()),
        ("LineSeg", line_seg_def()),
        ("FieldRange", field_range_def()),
        ("Control", control_def()),
        ("TableControl", table_control_def()),
        ("TableCell", table_cell_def()),
        ("ShapeControl", shape_control_def()),
        ("PictureControl", picture_control_def()),
        ("EquationControl", equation_control_def()),
        ("BookmarkControl", bookmark_control_def()),
        ("HyperlinkControl", hyperlink_control_def()),
        ("AutoNumberControl", auto_number_control_def()),
        ("PageNumberControl", page_number_control_def()),
        ("ColumnDefControl", column_def_control_def()),
        ("HiddenCommentControl", hidden_comment_control_def()),
        ("RubyControl", ruby_control_def()),
        ("TextWrap", text_wrap_def()),
        ("FootnoteControl", footnote_control_def()),
        ("FieldControl", field_control_def()),
        ("HeaderFooterControl", header_footer_control_def()),
        ("OtherControl", other_control_def()),
        ("FontFace", font_face_def()),
        ("CharShape", char_shape_def()),
        ("ParaShape", para_shape_def()),
        ("BorderFill", border_fill_def()),
        ("BorderLine", border_line_def()),
        ("Style", style_def()),
        ("Numbering", numbering_def()),
        ("Bullet", bullet_def()),
        ("TabDef", tab_def_def()),
        ("Preview", preview_def()),
        ("Provenance", provenance_def()),
        ("ColumnBreakType", column_break_type_def()),
    ]
    .into_iter()
    .map(|(name, def)| (name.to_string(), def))
    .collect();

    json!({
        "$schema": SCHEMA_DIALECT,
        // [#4329] $id 의 버전 조각도 레지스트리 상수에서 파생 — 리터럴 산개 금지.
        "$id": format!("https://github.com/edwardkim/rhwp/schema/ir/{IR_SCHEMA_VERSION}"),
        "title": "rhwp Document IR",
        "irSchemaVersion": IR_SCHEMA_VERSION,
        "description":
            "rhwp 가 모든 포맷(HWP5·HWPX·HWP3·HML)에서 만들어 내는 공통 문서 IR 의 공개 계약.              라운드트립 보존용 원본 바이트와 내부 shim 은 공개 표면이 아니므로 제외한다.              진화 규약: 필드 추가 = minor, 의미 변경·삭제 = major.",
        "$ref": "#/$defs/Document",
        "$defs": defs,
    })
}

/// `export-ir-schema` 봉투 — 스키마 본문과 메타를 함께 싣는다.
pub fn envelope() -> Value {
    let schema = ir_schema();
    let def_count = schema
        .get("$defs")
        .and_then(|d| d.as_object())
        .map(|o| o.len())
        .unwrap_or(0);
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "irSchemaVersion": IR_SCHEMA_VERSION,
        "dialect": SCHEMA_DIALECT,
        "definitionCount": def_count,
        "schema": schema,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_has_document_root_and_defs() {
        let schema = ir_schema();
        assert_eq!(schema["$ref"], "#/$defs/Document");
        assert!(schema["$defs"]["Document"].is_object());
        assert_eq!(schema["irSchemaVersion"], IR_SCHEMA_VERSION);
    }

    #[test]
    fn every_ref_resolves_to_a_definition() {
        // 끊어진 $ref 는 코드 생성기를 즉시 망가뜨린다 — 스키마의 최소 건전성이다.
        let schema = ir_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut missing = Vec::new();
        collect_refs(&schema, &mut |name| {
            if !defs.contains_key(name) {
                missing.push(name.to_string());
            }
        });
        assert!(missing.is_empty(), "정의되지 않은 참조: {missing:?}");
    }

    #[test]
    fn definitions_are_reachable_from_root() {
        // 아무도 가리키지 않는 정의는 죽은 계약이다.
        let schema = ir_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut referenced = std::collections::HashSet::new();
        referenced.insert("Document".to_string()); // 루트
        collect_refs(&schema, &mut |name| {
            referenced.insert(name.to_string());
        });
        let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
        assert!(orphans.is_empty(), "아무도 참조하지 않는 정의: {orphans:?}");
    }

    #[test]
    fn objects_allow_additional_properties() {
        // 추가-전용 진화 계약: 새 필드가 붙어도 기존 소비자가 깨지면 안 된다.
        let schema = ir_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        for (name, def) in defs {
            if def["type"] == "object" {
                assert_eq!(
                    def["additionalProperties"], true,
                    "{name} 이 추가 필드를 막고 있다 — IR 은 추가-전용 진화다"
                );
            }
        }
    }

    #[test]
    fn envelope_reports_definition_count() {
        let env = envelope();
        assert_eq!(env["schemaVersion"], "1.0");
        assert_eq!(env["irSchemaVersion"], IR_SCHEMA_VERSION);
        let count = env["definitionCount"].as_u64().expect("definitionCount");
        assert!(count >= 25, "정의가 너무 적다: {count}");
    }

    /// `$ref` 를 재귀 수집한다.
    fn collect_refs(value: &Value, sink: &mut impl FnMut(&str)) {
        match value {
            Value::Object(map) => {
                for (key, item) in map {
                    if key == "$ref" {
                        if let Some(path) = item.as_str() {
                            if let Some(name) = path.strip_prefix("#/$defs/") {
                                sink(name);
                            }
                        }
                    } else {
                        collect_refs(item, sink);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    collect_refs(item, sink);
                }
            }
            _ => {}
        }
    }
}
