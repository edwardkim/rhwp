/**
 * IR 타입 — **자동 생성 파일. 손으로 고치지 마세요.**
 *
 * 재생성: `npm run gen:types` (tools/gen-types.ts)
 * 출처:   `rhwp export-ir-schema` — irSchemaVersion 1.0, 정의 41개
 *
 * 이 파일을 직접 고치면 다음 생성에서 사라집니다. 모양을 바꾸려면 rhwp 본체의
 * `src/ir_schema.rs` 를 고치세요 — 스키마가 단일 출처이고 이 파일은 그 그림자입니다.
 *
 * 모든 인터페이스에 두 규약이 적용됩니다:
 *
 * - **전 필드 `readonly`** — 봉투는 도구가 준 관찰값이지 편집 대상이 아닙니다.
 *   여기서 값을 고쳐도 문서는 바뀌지 않으므로, 고칠 수 있게 두면 그 오해가 조용히
 *   자란다.
 * - **인덱스 시그니처(`readonly [key: string]: unknown`)** — IR 은 추가-전용으로
 *   진화합니다(`additionalProperties: true`). rhwp 가 필드를 하나 더할 때마다 모든
 *   소비자가 타입 오류로 깨지면 계약이 아니라 족쇄가 됩니다.
 *
 * 정의와 필드는 이름순입니다 — `interface` 는 호이스팅되므로 순서에 의미가 없고,
 * 이름순이어야 스키마가 조금 바뀔 때 diff 도 조금만 바뀝니다.
 *
 * @packageDocumentation
 */
/** 이 파일을 만들어 낸 IR 스키마 버전. 봉투 `schemaVersion`(명령별)과는 별개입니다. */
export declare const IR_SCHEMA_VERSION = "1.0";
/** 자동 번호 컨트롤 — 쪽·각주·표 번호가 여기서 나온다. */
export interface AutoNumberControl {
    /** 판별자 */
    readonly kind: 'autoNumber';
    /** 번호 표시 서식 */
    readonly numberShape?: number;
    /**
     * 번호 종류 (page=쪽 번호, footnote=각주 번호, endnote=미주 번호, picture=그림 번호, table=표
     * 번호, equation=수식 번호, totalPage=전체 쪽수)
     */
    readonly numberType?: 'page' | 'footnote' | 'endnote' | 'picture' | 'table' | 'equation' | 'totalPage';
    readonly [key: string]: unknown;
}
/** 책갈피 컨트롤. */
export interface BookmarkControl {
    /** 판별자 */
    readonly kind: 'bookmark';
    /** 책갈피 이름 — 하이퍼링크 대상이 된다 */
    readonly name: string;
    readonly [key: string]: unknown;
}
/** 테두리·채우기 묶음. 표 셀과 문단이 인덱스로 참조한다. */
export interface BorderFill {
    /** 배경색 (0xBBGGRR) */
    readonly backgroundColor?: number;
    readonly bottom?: BorderLine;
    /** 채우기 종류 (0=없음, 1=단색, 2=그러데이션, 4=이미지) */
    readonly fillType?: number;
    readonly left?: BorderLine;
    readonly right?: BorderLine;
    readonly top?: BorderLine;
    readonly [key: string]: unknown;
}
/** 테두리 선 하나. */
export interface BorderLine {
    /** 선 색 (0xBBGGRR) */
    readonly color?: number;
    /** 선 종류 (0=없음, 1=실선, 2=점선 …) */
    readonly type: number;
    /** 선 굵기 */
    readonly width?: number;
    readonly [key: string]: unknown;
}
/** 글머리표 정의. */
export interface Bullet {
    /** 글머리표 문자 */
    readonly char?: string;
    /** 이미지 글머리표 여부 */
    readonly useImage?: boolean;
    readonly [key: string]: unknown;
}
/** 글자 모양. 색은 HWP 관례대로 BGR 순서다. */
export interface CharShape {
    /** 기본 크기 (HWPUNIT, 1pt = 100) */
    readonly baseSize: number;
    /** 굵게 */
    readonly bold?: boolean;
    /** 언어(한글·영문·한자…)별 fontFaces 인덱스 */
    readonly fontIds?: readonly number[];
    /** 기울임 */
    readonly italic?: boolean;
    /** 음영색 (0xBBGGRR) */
    readonly shadeColor?: number;
    /** 취소선 */
    readonly strikeout?: boolean;
    /** 글자색 (0xBBGGRR) */
    readonly textColor?: number;
    /** 밑줄 */
    readonly underline?: boolean;
    readonly [key: string]: unknown;
}
/** 이 위치부터 글자 모양이 바뀐다. */
export interface CharShapeRef {
    /** docInfo.charShapes 인덱스 */
    readonly charShapeId: number;
    /** 적용 시작 위치 (UTF-16 코드 유닛 오프셋) */
    readonly position: number;
    readonly [key: string]: unknown;
}
/**
 * 문단 앞 나누기 종류 (none=나누지 않음, column=단 나누기, page=쪽 나누기, section=구역 나누기)
 */
export type ColumnBreakType = 'none' | 'column' | 'page' | 'section';
/** 단 정의 컨트롤 — 문단 위치에서 단 구성이 바뀐다. */
export interface ColumnDefControl {
    /** 단 종류 (normal=일반, distribute=배분, parallel=평행) */
    readonly columnType?: 'normal' | 'distribute' | 'parallel';
    /** 단 수 */
    readonly count?: number;
    /** 단 간격 (HWPUNIT) */
    readonly gap?: number;
    /** 판별자 */
    readonly kind: 'columnDef';
    /** 단 너비 동일 여부 */
    readonly sameWidth?: boolean;
    readonly [key: string]: unknown;
}
/**
 * 문단에 달린 컨트롤. `kind` 로 갈라지는 태그 유니온이다. 소비자는 모르는 kind 를 만나면
 * OtherControl 로 취급해야 한다 — 새 컨트롤이 추가돼도 깨지지 않는다.
 *
 * 판별자: `kind`
 */
export type Control = TableControl | ShapeControl | PictureControl | FootnoteControl | FieldControl | HeaderFooterControl | EquationControl | BookmarkControl | HyperlinkControl | AutoNumberControl | PageNumberControl | ColumnDefControl | HiddenCommentControl | RubyControl | OtherControl;
/** 문서 전역 서식 테이블. 문단·글자는 여기의 인덱스를 참조한다 (정규화된 IR). */
export interface DocInfo {
    /** 테두리·채우기 목록. */
    readonly borderFills?: readonly BorderFill[];
    /** 글머리표 정의 목록. */
    readonly bullets?: readonly Bullet[];
    /** 글자 모양 목록. */
    readonly charShapes: readonly CharShape[];
    /** 글꼴 목록. charShapes[].fontId 가 이 배열을 가리킨다. */
    readonly fontFaces?: readonly FontFace[];
    /** 번호 매기기 정의 목록. */
    readonly numberings?: readonly Numbering[];
    /** 문단 모양 목록. */
    readonly paraShapes: readonly ParaShape[];
    /** 스타일(문단 서식 묶음) 목록. */
    readonly styles?: readonly Style[];
    /** 탭 정의 목록. */
    readonly tabDefs?: readonly TabDef[];
    readonly [key: string]: unknown;
}
/** 문서 속성 — 번호 매기기 시작값. */
export interface DocProperties {
    /** 미주 시작 번호 */
    readonly endnoteStartNumber?: number;
    /** 수식 시작 번호 */
    readonly equationStartNumber?: number;
    /** 각주 시작 번호 */
    readonly footnoteStartNumber?: number;
    /** 시작 쪽 번호 */
    readonly pageStartNumber?: number;
    /** 그림 시작 번호 */
    readonly pictureStartNumber?: number;
    /** 구역 수 */
    readonly sectionCount: number;
    /** 표 시작 번호 */
    readonly tableStartNumber?: number;
    readonly [key: string]: unknown;
}
/** 문서 하나의 공개 IR. 모든 포맷(HWP5·HWPX·HWP3·HML) 파서가 이 모양을 돌려준다. */
export interface Document {
    readonly docInfo: DocInfo;
    readonly docProperties: DocProperties;
    readonly header: FileHeader;
    /** 미리보기(PrvImage/PrvText). 없으면 null. */
    readonly preview?: Preview | null;
    readonly provenance: Provenance;
    /** 본문 구역 목록. 최소 1개. */
    readonly sections: readonly Section[];
    readonly [key: string]: unknown;
}
/** 수식 컨트롤. script 가 원문이고 렌더는 그것으로 다시 조판한다. */
export interface EquationControl {
    /** 기준 글자 크기 (HWPUNIT) */
    readonly baseUnit?: number;
    /** 높이 (HWPUNIT) */
    readonly height?: number;
    /** 판별자 */
    readonly kind: 'equation';
    /** 수식 스크립트 (한글 수식 문법) */
    readonly script?: string;
    /** 너비 (HWPUNIT) */
    readonly width?: number;
    readonly [key: string]: unknown;
}
/** 필드 컨트롤 — 누름틀·책갈피·하이퍼링크. */
export interface FieldControl {
    /**
     * 필드 종류 (clickHere=누름틀, bookmark=책갈피, hyperlink=하이퍼링크, formula=계산식,
     * memo=메모, unknown=그 밖)
     */
    readonly fieldType: 'clickHere' | 'bookmark' | 'hyperlink' | 'formula' | 'memo' | 'unknown';
    /** 필드 지시문 */
    readonly instruction?: string;
    /** 판별자 */
    readonly kind: 'field';
    /** 필드 이름 */
    readonly name?: string;
    readonly [key: string]: unknown;
}
/** 누름틀(필드) 하나가 덮는 텍스트 범위. fill_fields 가 쓰는 좌표. */
export interface FieldRange {
    /** 끝 텍스트 인덱스 */
    readonly endIndex: number;
    /** 누름틀 이름 */
    readonly name?: string;
    /** 시작 텍스트 인덱스 */
    readonly startIndex: number;
    /** 현재 채워진 값 */
    readonly value?: string;
    readonly [key: string]: unknown;
}
/** 파일 헤더 — 포맷 버전과 저장 속성. */
export interface FileHeader {
    /** 본문 스트림 압축 여부 */
    readonly compressed?: boolean;
    /** 배포용 문서 여부 */
    readonly distributed?: boolean;
    /** 암호 보호 문서 여부 */
    readonly encrypted?: boolean;
    readonly version: HwpVersion;
    readonly [key: string]: unknown;
}
/** 글꼴 하나. */
export interface FontFace {
    /** 글꼴 이름 */
    readonly name: string;
    /** 대체 글꼴 이름 */
    readonly substituteName?: string;
    /** 글꼴 종류 (ttf/htf 등) */
    readonly type?: string;
    readonly [key: string]: unknown;
}
/** 각주·미주 컨트롤. */
export interface FootnoteControl {
    /** 미주 여부 (거짓이면 각주) */
    readonly isEndnote: boolean;
    /** 판별자 */
    readonly kind: 'footnote';
    /** 번호 */
    readonly number?: number;
    /** 각주 본문. */
    readonly paragraphs?: readonly Paragraph[];
    readonly [key: string]: unknown;
}
/** 머리말·꼬리말 컨트롤. */
export interface HeaderFooterControl {
    /** 적용 대상 (both=양쪽, even=짝수 쪽, odd=홀수 쪽) */
    readonly applyTo?: 'both' | 'even' | 'odd';
    /** 꼬리말 여부 (거짓이면 머리말) */
    readonly isFooter: boolean;
    /** 판별자 */
    readonly kind: 'headerFooter';
    /** 머리말·꼬리말 본문. */
    readonly paragraphs?: readonly Paragraph[];
    readonly [key: string]: unknown;
}
/** 숨은 설명(메모) 컨트롤 — 인쇄되지 않는 주석. */
export interface HiddenCommentControl {
    /** 판별자 */
    readonly kind: 'hiddenComment';
    /** 숨은 설명 본문. */
    readonly paragraphs?: readonly Paragraph[];
    readonly [key: string]: unknown;
}
/** HWP 파일 포맷 버전 (5.0.3.0 형식). */
export interface HwpVersion {
    /** 빌드 번호 */
    readonly build: number;
    /** 주 버전 */
    readonly major: number;
    /** 부 버전 */
    readonly minor: number;
    /** 리비전 */
    readonly revision: number;
    readonly [key: string]: unknown;
}
/** 하이퍼링크 컨트롤. */
export interface HyperlinkControl {
    /** 판별자 */
    readonly kind: 'hyperlink';
    /** 링크 대상 (URL 또는 문서 내 책갈피) */
    readonly target?: string;
    /** 설명 풍선 문구 */
    readonly tooltip?: string;
    readonly [key: string]: unknown;
}
/** 줄 하나의 조판 결과. 쪽 번호를 계산하는 근거다. */
export interface LineSeg {
    /** 베이스라인 간격 (HWPUNIT) */
    readonly baseLineGap?: number;
    /** 줄 높이 (HWPUNIT) */
    readonly lineHeight?: number;
    /** 줄 간격 (HWPUNIT) */
    readonly lineSpacing?: number;
    /** 세그먼트 너비 (HWPUNIT) */
    readonly segWidth?: number;
    /** 가로 시작 위치 (HWPUNIT) */
    readonly startPos?: number;
    /** 텍스트 높이 (HWPUNIT) */
    readonly textHeight?: number;
    /** 이 줄이 시작하는 텍스트 오프셋 */
    readonly textStart: number;
    /** 세로 위치 (HWPUNIT) */
    readonly verticalPos: number;
    readonly [key: string]: unknown;
}
/** 번호 매기기 정의. */
export interface Numbering {
    /** 수준별 정의 (최대 7단계) */
    readonly levels?: readonly {
        /** 정렬 */
        readonly alignment?: number;
        /** 번호 서식 (예: '^1.') */
        readonly format?: string;
        /** 시작 번호 */
        readonly startNumber?: number;
        readonly [key: string]: unknown;
    }[];
    readonly [key: string]: unknown;
}
/** IR 이 아직 세분화하지 않은 컨트롤. 라운드트립은 보존되지만 구조 접근은 제한된다. */
export interface OtherControl {
    /** 컨트롤 4바이트 식별자 (예: 'tbl ', 'secd') */
    readonly ctrlId: string;
    /** 판별자 */
    readonly kind: 'other';
    readonly [key: string]: unknown;
}
/** 쪽 번호 위치 컨트롤. */
export interface PageNumberControl {
    /** 판별자 */
    readonly kind: 'pageNumber';
    /**
     * 쪽 번호 위치 (none=없음, topLeft=위 왼쪽, topCenter=위 가운데, topRight=위 오른쪽,
     * bottomLeft=아래 왼쪽, bottomCenter=아래 가운데, bottomRight=아래 오른쪽, outsideTop=바깥쪽
     * 위, outsideBottom=바깥쪽 아래, insideTop=안쪽 위, insideBottom=안쪽 아래)
     */
    readonly position?: 'none' | 'topLeft' | 'topCenter' | 'topRight' | 'bottomLeft' | 'bottomCenter' | 'bottomRight' | 'outsideTop' | 'outsideBottom' | 'insideTop' | 'insideBottom';
    readonly [key: string]: unknown;
}
/** 문단 모양. */
export interface ParaShape {
    /**
     * 정렬 (justify=양쪽 정렬, left=왼쪽, right=오른쪽, center=가운데, distribute=배분,
     * divide=나눔)
     */
    readonly alignment: 'justify' | 'left' | 'right' | 'center' | 'distribute' | 'divide';
    /** 들여쓰기 (HWPUNIT, 음수면 내어쓰기) */
    readonly indent?: number;
    /** 왼쪽 여백 (HWPUNIT) */
    readonly leftMargin?: number;
    /** 줄 간격 */
    readonly lineSpacing?: number;
    /** 줄 간격 종류 (0=비율, 1=고정, 2=여백만) */
    readonly lineSpacingType?: number;
    /** 오른쪽 여백 (HWPUNIT) */
    readonly rightMargin?: number;
    /** 문단 아래 간격 (HWPUNIT) */
    readonly spacingBottom?: number;
    /** 문단 위 간격 (HWPUNIT) */
    readonly spacingTop?: number;
    readonly [key: string]: unknown;
}
/** 문단 하나. 텍스트와 그 위의 서식 참조·컨트롤을 담는다. */
export interface Paragraph {
    /** 문자 수 (제어 문자 포함, UTF-16 코드 유닛 기준) */
    readonly charCount: number;
    /** 글자 모양이 바뀌는 지점 목록. */
    readonly charShapes?: readonly CharShapeRef[];
    readonly columnType?: ColumnBreakType;
    /** 이 문단에 달린 컨트롤 (표·그림·각주 등). */
    readonly controls: readonly Control[];
    /** 누름틀 텍스트 범위. */
    readonly fieldRanges?: readonly FieldRange[];
    /** 줄 레이아웃 (조판 결과). */
    readonly lineSegs?: readonly LineSeg[];
    /** docInfo.paraShapes 인덱스 */
    readonly paraShapeId: number;
    /** docInfo.styles 인덱스 */
    readonly styleId?: number;
    /** 문단 텍스트. 컨트롤 자리에는 제어 문자가 들어간다. */
    readonly text: string;
    readonly [key: string]: unknown;
}
/** 그림 컨트롤. 도장·서명 삽입의 대상이다. */
export interface PictureControl {
    /** 대체 텍스트 (접근성) */
    readonly alt?: string;
    /** bin_data 참조 id — 실제 이미지 바이트를 가리킨다 */
    readonly binDataId?: number;
    /** 아래 자르기 (HWPUNIT) */
    readonly cropBottom?: number;
    /** 왼쪽 자르기 (HWPUNIT) */
    readonly cropLeft?: number;
    /** 오른쪽 자르기 (HWPUNIT) */
    readonly cropRight?: number;
    /** 위 자르기 (HWPUNIT) */
    readonly cropTop?: number;
    /** 표시 높이 (HWPUNIT) */
    readonly height?: number;
    /** 판별자 */
    readonly kind: 'picture';
    /** 원본 높이 (HWPUNIT) */
    readonly originalHeight?: number;
    /** 원본 너비 (HWPUNIT) */
    readonly originalWidth?: number;
    readonly textWrap?: TextWrap;
    /** 표시 너비 (HWPUNIT) */
    readonly width?: number;
    /** 가로 위치 (HWPUNIT) */
    readonly x?: number;
    /** 세로 위치 (HWPUNIT) */
    readonly y?: number;
    readonly [key: string]: unknown;
}
/** 문서 미리보기. */
export interface Preview {
    /** 미리보기 이미지(PrvImage) 존재 여부 */
    readonly hasImage?: boolean;
    /** 이미지 형식 (bmp=BMP, gif=GIF, jpeg=JPEG, png=PNG, unknown=미상) */
    readonly imageFormat?: 'bmp' | 'gif' | 'jpeg' | 'png' | 'unknown';
    /** 미리보기 텍스트 (PrvText) */
    readonly text?: string;
    readonly [key: string]: unknown;
}
/** 문서 출처 — 파서가 확정하는 단일 진실. 레이아웃 분기가 이 값을 본다. */
export interface Provenance {
    /** 다른 포맷에서 변환된 문서인지 */
    readonly converted?: boolean;
    /** 원본 포맷 (hwp5=HWP 5.x 바이너리, hwpx=HWPX (OWPML), hwp3=HWP 3.x 레거시, hml=HML XML) */
    readonly sourceFormat: 'hwp5' | 'hwpx' | 'hwp3' | 'hml';
    readonly [key: string]: unknown;
}
/** 덧말(루비) 컨트롤 — 한자 음 표기 등. */
export interface RubyControl {
    /** 판별자 */
    readonly kind: 'ruby';
    /** 본문 텍스트 */
    readonly mainText?: string;
    /** 덧말 위치 (above=위, below=아래) */
    readonly position?: 'above' | 'below';
    /** 덧말(윗주) 텍스트 */
    readonly rubyText?: string;
    readonly [key: string]: unknown;
}
/** 구역 하나 — 쪽 설정이 같은 문단 묶음. */
export interface Section {
    /** 문단 목록 (본문 순서). */
    readonly paragraphs: readonly Paragraph[];
    readonly sectionDef: SectionDef;
    readonly [key: string]: unknown;
}
/** 구역 쪽 설정. 길이 단위는 전부 HWPUNIT (1/7200 inch). */
export interface SectionDef {
    /** 단 수 */
    readonly columnCount?: number;
    /** 가로 방향 여부 */
    readonly landscape?: boolean;
    /** 아래 여백 (HWPUNIT) */
    readonly marginBottom?: number;
    /** 꼬리말 여백 (HWPUNIT) */
    readonly marginFooter?: number;
    /** 제본 여백 (HWPUNIT) */
    readonly marginGutter?: number;
    /** 머리말 여백 (HWPUNIT) */
    readonly marginHeader?: number;
    /** 왼쪽 여백 (HWPUNIT) */
    readonly marginLeft?: number;
    /** 오른쪽 여백 (HWPUNIT) */
    readonly marginRight?: number;
    /** 위 여백 (HWPUNIT) */
    readonly marginTop?: number;
    /** 용지 높이 (HWPUNIT) */
    readonly pageHeight: number;
    /** 용지 너비 (HWPUNIT, 1/7200 inch) */
    readonly pageWidth: number;
    readonly [key: string]: unknown;
}
/** 도형 컨트롤 (선·사각형·타원·다각형·글상자 등). */
export interface ShapeControl {
    /** 그림일 때 bin_data 참조 id */
    readonly binDataId?: number;
    /** 좌우 뒤집기 */
    readonly flipHorizontal?: boolean;
    /** 상하 뒤집기 */
    readonly flipVertical?: boolean;
    /** 높이 (HWPUNIT) */
    readonly height?: number;
    /** 판별자 */
    readonly kind: 'shape';
    /** 회전 각도 (1/100 도) */
    readonly rotation?: number;
    /**
     * 도형 종류 (picture=그림, rectangle=사각형, ellipse=타원, line=선, polygon=다각형, arc=호,
     * curve=곡선, textBox=글상자, ole=OLE 개체, container=묶음 개체)
     */
    readonly shapeType: 'picture' | 'rectangle' | 'ellipse' | 'line' | 'polygon' | 'arc' | 'curve' | 'textBox' | 'ole' | 'container';
    readonly textWrap?: TextWrap;
    /** 너비 (HWPUNIT) */
    readonly width?: number;
    /** 가로 위치 (HWPUNIT) */
    readonly x?: number;
    /** 세로 위치 (HWPUNIT) */
    readonly y?: number;
    readonly [key: string]: unknown;
}
/** 스타일 — 문단·글자 모양의 이름 붙은 묶음. */
export interface Style {
    /** docInfo.charShapes 인덱스 */
    readonly charShapeId?: number;
    /** 스타일 이름 (영문) */
    readonly englishName?: string;
    /** 스타일 이름 (한글) */
    readonly name: string;
    /** docInfo.paraShapes 인덱스 */
    readonly paraShapeId?: number;
    /** 종류 (0=문단, 1=글자) */
    readonly styleType?: number;
    readonly [key: string]: unknown;
}
/** 탭 정의. */
export interface TabDef {
    /** 왼쪽 자동 탭 */
    readonly autoTabLeft?: boolean;
    /** 오른쪽 자동 탭 */
    readonly autoTabRight?: boolean;
    /** 탭 목록 */
    readonly tabs?: readonly {
        /** 채움 문자 종류 */
        readonly leader?: number;
        /** 탭 위치 (HWPUNIT) */
        readonly position: number;
        /** 탭 종류 (0=왼쪽, 1=오른쪽, 2=가운데, 3=소수점) */
        readonly type?: number;
        readonly [key: string]: unknown;
    }[];
    readonly [key: string]: unknown;
}
/** 표 셀 하나. 병합된 셀은 좌상단 좌표 하나로만 나타난다 — 덮인 좌표는 목록에 없다. */
export interface TableCell {
    /** 열 (0 기준) */
    readonly col: number;
    /** 가로 병합 칸 수 */
    readonly colSpan: number;
    /** 높이 (HWPUNIT) */
    readonly height?: number;
    /** 셀 안의 문단 (중첩 구조). */
    readonly paragraphs: readonly Paragraph[];
    /** 행 (0 기준) */
    readonly row: number;
    /** 세로 병합 칸 수 (1 이면 병합 없음) */
    readonly rowSpan: number;
    /** 너비 (HWPUNIT) */
    readonly width?: number;
    readonly [key: string]: unknown;
}
/** 표 컨트롤. set_cell 의 대상이다. */
export interface TableControl {
    /** docInfo.borderFills 인덱스 */
    readonly borderFillId?: number;
    /** 셀 목록 (병합 포함). */
    readonly cells: readonly TableCell[];
    /** 열 수 */
    readonly colCount: number;
    /** 판별자 */
    readonly kind: 'table';
    /** 행 수 */
    readonly rowCount: number;
    readonly [key: string]: unknown;
}
/** 개체와 본문의 배치 관계. */
export interface TextWrap {
    /** 아래 바깥 여백 (HWPUNIT) */
    readonly marginBottom?: number;
    /** 왼쪽 바깥 여백 (HWPUNIT) */
    readonly marginLeft?: number;
    /** 오른쪽 바깥 여백 (HWPUNIT) */
    readonly marginRight?: number;
    /** 위 바깥 여백 (HWPUNIT) */
    readonly marginTop?: number;
    /**
     * 본문과의 배치 (square=어울림, tight=자리 차지, through=글 뒤로, topAndBottom=위/아래,
     * behindText=글 뒤로, inFrontOfText=글 앞으로, inline=글자처럼 취급)
     */
    readonly style?: 'square' | 'tight' | 'through' | 'topAndBottom' | 'behindText' | 'inFrontOfText' | 'inline';
    readonly [key: string]: unknown;
}
