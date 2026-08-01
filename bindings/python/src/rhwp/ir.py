"""IR 타입 모델 — **자동 생성 파일. 손으로 고치지 마세요.**

생성: ``python tools/gen_models.py -o src/rhwp/ir.py``
출처: ``rhwp export-ir-schema`` (irSchemaVersion 1.0)

이 파일을 직접 수정하면 다음 생성 때 덮어써집니다. 모양을 바꾸려면 rhwp 본체의
``src/ir_schema.rs`` 를 고치세요 — 스키마가 단일 출처입니다.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

#: 이 모델이 생성된 IR 스키마 버전.
IR_SCHEMA_VERSION = "1.0"

@dataclass
class AutoNumberControl:
    """자동 번호 컨트롤 — 쪽·각주·표 번호가 여기서 나온다."""
    kind: Any
    """판별자"""
    number_shape: Optional[int] = None  # 봉투 키: numberShape
    """번호 표시 서식"""
    number_type: Optional[str] = None  # 봉투 키: numberType · 허용: page/footnote/endnote/picture/table/equation/totalPage
    """
    번호 종류 (page=쪽 번호, footnote=각주 번호, endnote=미주 번호, picture=그림 번호, table=표 번호,
    equation=수식 번호, totalPage=전체 쪽수)
    """

@dataclass
class BookmarkControl:
    """책갈피 컨트롤."""
    kind: Any
    """판별자"""
    name: str
    """책갈피 이름 — 하이퍼링크 대상이 된다"""

@dataclass
class BorderLine:
    """테두리 선 하나."""
    type: int
    """선 종류 (0=없음, 1=실선, 2=점선 …)"""
    color: Optional[int] = None
    """선 색 (0xBBGGRR)"""
    width: Optional[int] = None
    """선 굵기"""

@dataclass
class BorderFill:
    """테두리·채우기 묶음. 표 셀과 문단이 인덱스로 참조한다."""
    background_color: Optional[int] = None  # 봉투 키: backgroundColor
    """배경색 (0xBBGGRR)"""
    bottom: Optional[BorderLine] = None
    fill_type: Optional[int] = None  # 봉투 키: fillType
    """채우기 종류 (0=없음, 1=단색, 2=그러데이션, 4=이미지)"""
    left: Optional[BorderLine] = None
    right: Optional[BorderLine] = None
    top: Optional[BorderLine] = None

@dataclass
class Bullet:
    """글머리표 정의."""
    char: Optional[str] = None
    """글머리표 문자"""
    use_image: Optional[bool] = None  # 봉투 키: useImage
    """이미지 글머리표 여부"""

@dataclass
class CharShape:
    """글자 모양. 색은 HWP 관례대로 BGR 순서다."""
    base_size: int  # 봉투 키: baseSize
    """기본 크기 (HWPUNIT, 1pt = 100)"""
    bold: Optional[bool] = None
    """굵게"""
    font_ids: List[int] = field(default_factory=list)  # 봉투 키: fontIds
    """언어(한글·영문·한자…)별 fontFaces 인덱스"""
    italic: Optional[bool] = None
    """기울임"""
    shade_color: Optional[int] = None  # 봉투 키: shadeColor
    """음영색 (0xBBGGRR)"""
    strikeout: Optional[bool] = None
    """취소선"""
    text_color: Optional[int] = None  # 봉투 키: textColor
    """글자색 (0xBBGGRR)"""
    underline: Optional[bool] = None
    """밑줄"""

@dataclass
class CharShapeRef:
    """이 위치부터 글자 모양이 바뀐다."""
    char_shape_id: int  # 봉투 키: charShapeId
    """docInfo.charShapes 인덱스"""
    position: int
    """적용 시작 위치 (UTF-16 코드 유닛 오프셋)"""

@dataclass
class ColumnBreakType:
    """문단 앞 나누기 종류 (none=나누지 않음, column=단 나누기, page=쪽 나누기, section=구역 나누기)"""
    pass

@dataclass
class ColumnDefControl:
    """단 정의 컨트롤 — 문단 위치에서 단 구성이 바뀐다."""
    kind: Any
    """판별자"""
    column_type: Optional[str] = None  # 봉투 키: columnType · 허용: normal/distribute/parallel
    """단 종류 (normal=일반, distribute=배분, parallel=평행)"""
    count: Optional[int] = None
    """단 수"""
    gap: Optional[int] = None
    """단 간격 (HWPUNIT)"""
    same_width: Optional[bool] = None  # 봉투 키: sameWidth
    """단 너비 동일 여부"""

@dataclass
class FieldRange:
    """누름틀(필드) 하나가 덮는 텍스트 범위. fill_fields 가 쓰는 좌표."""
    end_index: int  # 봉투 키: endIndex
    """끝 텍스트 인덱스"""
    start_index: int  # 봉투 키: startIndex
    """시작 텍스트 인덱스"""
    name: Optional[str] = None
    """누름틀 이름"""
    value: Optional[str] = None
    """현재 채워진 값"""

@dataclass
class LineSeg:
    """줄 하나의 조판 결과. 쪽 번호를 계산하는 근거다."""
    text_start: int  # 봉투 키: textStart
    """이 줄이 시작하는 텍스트 오프셋"""
    vertical_pos: int  # 봉투 키: verticalPos
    """세로 위치 (HWPUNIT)"""
    base_line_gap: Optional[int] = None  # 봉투 키: baseLineGap
    """베이스라인 간격 (HWPUNIT)"""
    line_height: Optional[int] = None  # 봉투 키: lineHeight
    """줄 높이 (HWPUNIT)"""
    line_spacing: Optional[int] = None  # 봉투 키: lineSpacing
    """줄 간격 (HWPUNIT)"""
    seg_width: Optional[int] = None  # 봉투 키: segWidth
    """세그먼트 너비 (HWPUNIT)"""
    start_pos: Optional[int] = None  # 봉투 키: startPos
    """가로 시작 위치 (HWPUNIT)"""
    text_height: Optional[int] = None  # 봉투 키: textHeight
    """텍스트 높이 (HWPUNIT)"""

@dataclass
class Paragraph:
    """문단 하나. 텍스트와 그 위의 서식 참조·컨트롤을 담는다."""
    char_count: int  # 봉투 키: charCount
    """문자 수 (제어 문자 포함, UTF-16 코드 유닛 기준)"""
    controls: List[Control]
    """이 문단에 달린 컨트롤 (표·그림·각주 등)."""
    para_shape_id: int  # 봉투 키: paraShapeId
    """docInfo.paraShapes 인덱스"""
    text: str
    """문단 텍스트. 컨트롤 자리에는 제어 문자가 들어간다."""
    char_shapes: List[CharShapeRef] = field(default_factory=list)  # 봉투 키: charShapes
    """글자 모양이 바뀌는 지점 목록."""
    column_type: Optional[ColumnBreakType] = None  # 봉투 키: columnType
    field_ranges: List[FieldRange] = field(default_factory=list)  # 봉투 키: fieldRanges
    """누름틀 텍스트 범위."""
    line_segs: List[LineSeg] = field(default_factory=list)  # 봉투 키: lineSegs
    """줄 레이아웃 (조판 결과)."""
    style_id: Optional[int] = None  # 봉투 키: styleId
    """docInfo.styles 인덱스"""

@dataclass
class TableCell:
    """표 셀 하나. 병합된 셀은 좌상단 좌표 하나로만 나타난다 — 덮인 좌표는 목록에 없다."""
    col: int
    """열 (0 기준)"""
    col_span: int  # 봉투 키: colSpan
    """가로 병합 칸 수"""
    paragraphs: List[Paragraph]
    """셀 안의 문단 (중첩 구조)."""
    row: int
    """행 (0 기준)"""
    row_span: int  # 봉투 키: rowSpan
    """세로 병합 칸 수 (1 이면 병합 없음)"""
    height: Optional[int] = None
    """높이 (HWPUNIT)"""
    width: Optional[int] = None
    """너비 (HWPUNIT)"""

@dataclass
class TableControl:
    """표 컨트롤. set_cell 의 대상이다."""
    cells: List[TableCell]
    """셀 목록 (병합 포함)."""
    col_count: int  # 봉투 키: colCount
    """열 수"""
    kind: Any
    """판별자"""
    row_count: int  # 봉투 키: rowCount
    """행 수"""
    border_fill_id: Optional[int] = None  # 봉투 키: borderFillId
    """docInfo.borderFills 인덱스"""

@dataclass
class TextWrap:
    """개체와 본문의 배치 관계."""
    margin_bottom: Optional[int] = None  # 봉투 키: marginBottom
    """아래 바깥 여백 (HWPUNIT)"""
    margin_left: Optional[int] = None  # 봉투 키: marginLeft
    """왼쪽 바깥 여백 (HWPUNIT)"""
    margin_right: Optional[int] = None  # 봉투 키: marginRight
    """오른쪽 바깥 여백 (HWPUNIT)"""
    margin_top: Optional[int] = None  # 봉투 키: marginTop
    """위 바깥 여백 (HWPUNIT)"""
    style: Optional[str] = None  # 허용: square/tight/through/topAndBottom/behindText/inFrontOfText/inline
    """
    본문과의 배치 (square=어울림, tight=자리 차지, through=글 뒤로, topAndBottom=위/아래, behindText=글 뒤로,
    inFrontOfText=글 앞으로, inline=글자처럼 취급)
    """

@dataclass
class ShapeControl:
    """도형 컨트롤 (선·사각형·타원·다각형·글상자 등)."""
    kind: Any
    """판별자"""
    shape_type: str  # 봉투 키: shapeType · 허용: picture/rectangle/ellipse/line/polygon/arc/curve/textBox/ole/container
    """
    도형 종류 (picture=그림, rectangle=사각형, ellipse=타원, line=선, polygon=다각형, arc=호, curve=곡선,
    textBox=글상자, ole=OLE 개체, container=묶음 개체)
    """
    bin_data_id: Optional[int] = None  # 봉투 키: binDataId
    """그림일 때 bin_data 참조 id"""
    flip_horizontal: Optional[bool] = None  # 봉투 키: flipHorizontal
    """좌우 뒤집기"""
    flip_vertical: Optional[bool] = None  # 봉투 키: flipVertical
    """상하 뒤집기"""
    height: Optional[int] = None
    """높이 (HWPUNIT)"""
    rotation: Optional[int] = None
    """회전 각도 (1/100 도)"""
    text_wrap: Optional[TextWrap] = None  # 봉투 키: textWrap
    width: Optional[int] = None
    """너비 (HWPUNIT)"""
    x: Optional[int] = None
    """가로 위치 (HWPUNIT)"""
    y: Optional[int] = None
    """세로 위치 (HWPUNIT)"""

@dataclass
class PictureControl:
    """그림 컨트롤. 도장·서명 삽입의 대상이다."""
    kind: Any
    """판별자"""
    alt: Optional[str] = None
    """대체 텍스트 (접근성)"""
    bin_data_id: Optional[int] = None  # 봉투 키: binDataId
    """bin_data 참조 id — 실제 이미지 바이트를 가리킨다"""
    crop_bottom: Optional[int] = None  # 봉투 키: cropBottom
    """아래 자르기 (HWPUNIT)"""
    crop_left: Optional[int] = None  # 봉투 키: cropLeft
    """왼쪽 자르기 (HWPUNIT)"""
    crop_right: Optional[int] = None  # 봉투 키: cropRight
    """오른쪽 자르기 (HWPUNIT)"""
    crop_top: Optional[int] = None  # 봉투 키: cropTop
    """위 자르기 (HWPUNIT)"""
    height: Optional[int] = None
    """표시 높이 (HWPUNIT)"""
    original_height: Optional[int] = None  # 봉투 키: originalHeight
    """원본 높이 (HWPUNIT)"""
    original_width: Optional[int] = None  # 봉투 키: originalWidth
    """원본 너비 (HWPUNIT)"""
    text_wrap: Optional[TextWrap] = None  # 봉투 키: textWrap
    width: Optional[int] = None
    """표시 너비 (HWPUNIT)"""
    x: Optional[int] = None
    """가로 위치 (HWPUNIT)"""
    y: Optional[int] = None
    """세로 위치 (HWPUNIT)"""

@dataclass
class FootnoteControl:
    """각주·미주 컨트롤."""
    is_endnote: bool  # 봉투 키: isEndnote
    """미주 여부 (거짓이면 각주)"""
    kind: Any
    """판별자"""
    number: Optional[int] = None
    """번호"""
    paragraphs: List[Paragraph] = field(default_factory=list)
    """각주 본문."""

@dataclass
class FieldControl:
    """필드 컨트롤 — 누름틀·책갈피·하이퍼링크."""
    field_type: str  # 봉투 키: fieldType · 허용: clickHere/bookmark/hyperlink/formula/memo/unknown
    """
    필드 종류 (clickHere=누름틀, bookmark=책갈피, hyperlink=하이퍼링크, formula=계산식, memo=메모, unknown=그
    밖)
    """
    kind: Any
    """판별자"""
    instruction: Optional[str] = None
    """필드 지시문"""
    name: Optional[str] = None
    """필드 이름"""

@dataclass
class HeaderFooterControl:
    """머리말·꼬리말 컨트롤."""
    is_footer: bool  # 봉투 키: isFooter
    """꼬리말 여부 (거짓이면 머리말)"""
    kind: Any
    """판별자"""
    apply_to: Optional[str] = None  # 봉투 키: applyTo · 허용: both/even/odd
    """적용 대상 (both=양쪽, even=짝수 쪽, odd=홀수 쪽)"""
    paragraphs: List[Paragraph] = field(default_factory=list)
    """머리말·꼬리말 본문."""

@dataclass
class EquationControl:
    """수식 컨트롤. script 가 원문이고 렌더는 그것으로 다시 조판한다."""
    kind: Any
    """판별자"""
    base_unit: Optional[int] = None  # 봉투 키: baseUnit
    """기준 글자 크기 (HWPUNIT)"""
    height: Optional[int] = None
    """높이 (HWPUNIT)"""
    script: Optional[str] = None
    """수식 스크립트 (한글 수식 문법)"""
    width: Optional[int] = None
    """너비 (HWPUNIT)"""

@dataclass
class HyperlinkControl:
    """하이퍼링크 컨트롤."""
    kind: Any
    """판별자"""
    target: Optional[str] = None
    """링크 대상 (URL 또는 문서 내 책갈피)"""
    tooltip: Optional[str] = None
    """설명 풍선 문구"""

@dataclass
class PageNumberControl:
    """쪽 번호 위치 컨트롤."""
    kind: Any
    """판별자"""
    position: Optional[str] = None  # 허용: none/topLeft/topCenter/topRight/bottomLeft/bottomCenter/bottomRight/outsideTop/outsideBottom/insideTop/insideBottom
    """
    쪽 번호 위치 (none=없음, topLeft=위 왼쪽, topCenter=위 가운데, topRight=위 오른쪽, bottomLeft=아래 왼쪽,
    bottomCenter=아래 가운데, bottomRight=아래 오른쪽, outsideTop=바깥쪽 위, outsideBottom=바깥쪽 아래,
    insideTop=안쪽 위, insideBottom=안쪽 아래)
    """

@dataclass
class HiddenCommentControl:
    """숨은 설명(메모) 컨트롤 — 인쇄되지 않는 주석."""
    kind: Any
    """판별자"""
    paragraphs: List[Paragraph] = field(default_factory=list)
    """숨은 설명 본문."""

@dataclass
class RubyControl:
    """덧말(루비) 컨트롤 — 한자 음 표기 등."""
    kind: Any
    """판별자"""
    main_text: Optional[str] = None  # 봉투 키: mainText
    """본문 텍스트"""
    position: Optional[str] = None  # 허용: above/below
    """덧말 위치 (above=위, below=아래)"""
    ruby_text: Optional[str] = None  # 봉투 키: rubyText
    """덧말(윗주) 텍스트"""

@dataclass
class OtherControl:
    """IR 이 아직 세분화하지 않은 컨트롤. 라운드트립은 보존되지만 구조 접근은 제한된다."""
    ctrl_id: str  # 봉투 키: ctrlId
    """컨트롤 4바이트 식별자 (예: 'tbl ', 'secd')"""
    kind: Any
    """판별자"""

"""
문단에 달린 컨트롤. `kind` 로 갈라지는 태그 유니온이다. 소비자는 모르는 kind 를 만나면 OtherControl 로 취급해야 한다 — 새 컨트롤이
추가돼도 깨지지 않는다.
"""
Control = Any  # oneOf: TableControl | ShapeControl | PictureControl | FootnoteControl | FieldControl | HeaderFooterControl | EquationControl | BookmarkControl | HyperlinkControl | AutoNumberControl | PageNumberControl | ColumnDefControl | HiddenCommentControl | RubyControl | OtherControl

@dataclass
class ParaShape:
    """문단 모양."""
    alignment: str  # 허용: justify/left/right/center/distribute/divide
    """정렬 (justify=양쪽 정렬, left=왼쪽, right=오른쪽, center=가운데, distribute=배분, divide=나눔)"""
    indent: Optional[int] = None
    """들여쓰기 (HWPUNIT, 음수면 내어쓰기)"""
    left_margin: Optional[int] = None  # 봉투 키: leftMargin
    """왼쪽 여백 (HWPUNIT)"""
    line_spacing: Optional[int] = None  # 봉투 키: lineSpacing
    """줄 간격"""
    line_spacing_type: Optional[int] = None  # 봉투 키: lineSpacingType
    """줄 간격 종류 (0=비율, 1=고정, 2=여백만)"""
    right_margin: Optional[int] = None  # 봉투 키: rightMargin
    """오른쪽 여백 (HWPUNIT)"""
    spacing_bottom: Optional[int] = None  # 봉투 키: spacingBottom
    """문단 아래 간격 (HWPUNIT)"""
    spacing_top: Optional[int] = None  # 봉투 키: spacingTop
    """문단 위 간격 (HWPUNIT)"""

@dataclass
class FontFace:
    """글꼴 하나."""
    name: str
    """글꼴 이름"""
    substitute_name: Optional[str] = None  # 봉투 키: substituteName
    """대체 글꼴 이름"""
    type: Optional[str] = None
    """글꼴 종류 (ttf/htf 등)"""

@dataclass
class Numbering:
    """번호 매기기 정의."""
    levels: List[Any] = field(default_factory=list)
    """수준별 정의 (최대 7단계)"""

@dataclass
class Style:
    """스타일 — 문단·글자 모양의 이름 붙은 묶음."""
    name: str
    """스타일 이름 (한글)"""
    char_shape_id: Optional[int] = None  # 봉투 키: charShapeId
    """docInfo.charShapes 인덱스"""
    english_name: Optional[str] = None  # 봉투 키: englishName
    """스타일 이름 (영문)"""
    para_shape_id: Optional[int] = None  # 봉투 키: paraShapeId
    """docInfo.paraShapes 인덱스"""
    style_type: Optional[int] = None  # 봉투 키: styleType
    """종류 (0=문단, 1=글자)"""

@dataclass
class TabDef:
    """탭 정의."""
    auto_tab_left: Optional[bool] = None  # 봉투 키: autoTabLeft
    """왼쪽 자동 탭"""
    auto_tab_right: Optional[bool] = None  # 봉투 키: autoTabRight
    """오른쪽 자동 탭"""
    tabs: List[Any] = field(default_factory=list)
    """탭 목록"""

@dataclass
class DocInfo:
    """문서 전역 서식 테이블. 문단·글자는 여기의 인덱스를 참조한다 (정규화된 IR)."""
    char_shapes: List[CharShape]  # 봉투 키: charShapes
    """글자 모양 목록."""
    para_shapes: List[ParaShape]  # 봉투 키: paraShapes
    """문단 모양 목록."""
    border_fills: List[BorderFill] = field(default_factory=list)  # 봉투 키: borderFills
    """테두리·채우기 목록."""
    bullets: List[Bullet] = field(default_factory=list)
    """글머리표 정의 목록."""
    font_faces: List[FontFace] = field(default_factory=list)  # 봉투 키: fontFaces
    """글꼴 목록. charShapes[].fontId 가 이 배열을 가리킨다."""
    numberings: List[Numbering] = field(default_factory=list)
    """번호 매기기 정의 목록."""
    styles: List[Style] = field(default_factory=list)
    """스타일(문단 서식 묶음) 목록."""
    tab_defs: List[TabDef] = field(default_factory=list)  # 봉투 키: tabDefs
    """탭 정의 목록."""

@dataclass
class DocProperties:
    """문서 속성 — 번호 매기기 시작값."""
    section_count: int  # 봉투 키: sectionCount
    """구역 수"""
    endnote_start_number: Optional[int] = None  # 봉투 키: endnoteStartNumber
    """미주 시작 번호"""
    equation_start_number: Optional[int] = None  # 봉투 키: equationStartNumber
    """수식 시작 번호"""
    footnote_start_number: Optional[int] = None  # 봉투 키: footnoteStartNumber
    """각주 시작 번호"""
    page_start_number: Optional[int] = None  # 봉투 키: pageStartNumber
    """시작 쪽 번호"""
    picture_start_number: Optional[int] = None  # 봉투 키: pictureStartNumber
    """그림 시작 번호"""
    table_start_number: Optional[int] = None  # 봉투 키: tableStartNumber
    """표 시작 번호"""

@dataclass
class HwpVersion:
    """HWP 파일 포맷 버전 (5.0.3.0 형식)."""
    build: int
    """빌드 번호"""
    major: int
    """주 버전"""
    minor: int
    """부 버전"""
    revision: int
    """리비전"""

@dataclass
class FileHeader:
    """파일 헤더 — 포맷 버전과 저장 속성."""
    version: HwpVersion
    compressed: Optional[bool] = None
    """본문 스트림 압축 여부"""
    distributed: Optional[bool] = None
    """배포용 문서 여부"""
    encrypted: Optional[bool] = None
    """암호 보호 문서 여부"""

@dataclass
class Provenance:
    """문서 출처 — 파서가 확정하는 단일 진실. 레이아웃 분기가 이 값을 본다."""
    source_format: str  # 봉투 키: sourceFormat · 허용: hwp5/hwpx/hwp3/hml
    """원본 포맷 (hwp5=HWP 5.x 바이너리, hwpx=HWPX (OWPML), hwp3=HWP 3.x 레거시, hml=HML XML)"""
    converted: Optional[bool] = None
    """다른 포맷에서 변환된 문서인지"""

@dataclass
class SectionDef:
    """구역 쪽 설정. 길이 단위는 전부 HWPUNIT (1/7200 inch)."""
    page_height: int  # 봉투 키: pageHeight
    """용지 높이 (HWPUNIT)"""
    page_width: int  # 봉투 키: pageWidth
    """용지 너비 (HWPUNIT, 1/7200 inch)"""
    column_count: Optional[int] = None  # 봉투 키: columnCount
    """단 수"""
    landscape: Optional[bool] = None
    """가로 방향 여부"""
    margin_bottom: Optional[int] = None  # 봉투 키: marginBottom
    """아래 여백 (HWPUNIT)"""
    margin_footer: Optional[int] = None  # 봉투 키: marginFooter
    """꼬리말 여백 (HWPUNIT)"""
    margin_gutter: Optional[int] = None  # 봉투 키: marginGutter
    """제본 여백 (HWPUNIT)"""
    margin_header: Optional[int] = None  # 봉투 키: marginHeader
    """머리말 여백 (HWPUNIT)"""
    margin_left: Optional[int] = None  # 봉투 키: marginLeft
    """왼쪽 여백 (HWPUNIT)"""
    margin_right: Optional[int] = None  # 봉투 키: marginRight
    """오른쪽 여백 (HWPUNIT)"""
    margin_top: Optional[int] = None  # 봉투 키: marginTop
    """위 여백 (HWPUNIT)"""

@dataclass
class Section:
    """구역 하나 — 쪽 설정이 같은 문단 묶음."""
    paragraphs: List[Paragraph]
    """문단 목록 (본문 순서)."""
    section_def: SectionDef  # 봉투 키: sectionDef

@dataclass
class Document:
    """문서 하나의 공개 IR. 모든 포맷(HWP5·HWPX·HWP3·HML) 파서가 이 모양을 돌려준다."""
    doc_info: DocInfo  # 봉투 키: docInfo
    doc_properties: DocProperties  # 봉투 키: docProperties
    header: FileHeader
    provenance: Provenance
    sections: List[Section]
    """본문 구역 목록. 최소 1개."""
    preview: Optional[Preview] = None
    """미리보기(PrvImage/PrvText). 없으면 null."""

@dataclass
class Preview:
    """문서 미리보기."""
    has_image: Optional[bool] = None  # 봉투 키: hasImage
    """미리보기 이미지(PrvImage) 존재 여부"""
    image_format: Optional[str] = None  # 봉투 키: imageFormat · 허용: bmp/gif/jpeg/png/unknown
    """이미지 형식 (bmp=BMP, gif=GIF, jpeg=JPEG, png=PNG, unknown=미상)"""
    text: Optional[str] = None
    """미리보기 텍스트 (PrvText)"""

__all__ = [
    "IR_SCHEMA_VERSION",
    "AutoNumberControl",
    "BookmarkControl",
    "BorderFill",
    "BorderLine",
    "Bullet",
    "CharShape",
    "CharShapeRef",
    "ColumnBreakType",
    "ColumnDefControl",
    "Control",
    "DocInfo",
    "DocProperties",
    "Document",
    "EquationControl",
    "FieldControl",
    "FieldRange",
    "FileHeader",
    "FontFace",
    "FootnoteControl",
    "HeaderFooterControl",
    "HiddenCommentControl",
    "HwpVersion",
    "HyperlinkControl",
    "LineSeg",
    "Numbering",
    "OtherControl",
    "PageNumberControl",
    "ParaShape",
    "Paragraph",
    "PictureControl",
    "Preview",
    "Provenance",
    "RubyControl",
    "Section",
    "SectionDef",
    "ShapeControl",
    "Style",
    "TabDef",
    "TableCell",
    "TableControl",
    "TextWrap",
]
