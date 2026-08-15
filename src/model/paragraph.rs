//! 문단 (Paragraph, CharRun, LineSeg, RangeTag)

use super::control::{Control, CTRL_CHAR_CODE_UNITS};
use serde::{Deserialize, Serialize};

/// 문단 (HWPTAG_PARA_HEADER + 하위 레코드)
#[derive(Debug, Default, Clone)]
pub struct Paragraph {
    /// 문자 수 (제어 문자 포함)
    pub char_count: u32,
    /// 컨트롤 마스크
    pub control_mask: u32,
    /// 문단 모양 ID 참조
    pub para_shape_id: u16,
    /// 문단 스타일 ID 참조
    pub style_id: u8,
    /// 단 나누기 종류
    pub column_type: ColumnBreakType,
    /// 원본 break_type 바이트 (라운드트립 보존용, 0이면 column_type에서 재구성)
    pub raw_break_type: u8,
    /// 문단 텍스트 (UTF-16에서 변환된 문자열)
    pub text: String,
    /// 텍스트 문자별 UTF-16 코드 유닛 위치 (LineSeg/CharShapeRef 위치와 매핑용)
    /// char_offsets[i] = text[i]에 해당하는 원본 UTF-16 코드 유닛 인덱스
    pub char_offsets: Vec<u32>,
    /// 글자 모양 변경 위치 목록
    pub char_shapes: Vec<CharShapeRef>,
    /// 줄 레이아웃 정보
    pub line_segs: Vec<LineSeg>,
    /// [#4677] `line_segs` **끝쪽** 몇 줄이 조판 전용 보강 줄인가.
    ///
    /// HWPX RowBreak 표 셀은 문단별 `<hp:linesegarray>` 를 생략하면서도 셀 높이는 남긴다.
    /// 그 높이에 맞춰 줄을 보강해야 쪽 나눔이 한컴과 같아지지만
    /// (`DocumentCore::fit_hwpx_rowbreak_synthetic_cell_lines`), 그 줄은 **본문에 없는 줄**
    /// 이라 HWP5 로 저장하면 안 된다 — 한글 2022 는 그런 셀 문단을 만나면 본문 전체를
    /// 버리고 빈 1쪽 문서로 연다(rhwp 재파싱은 통과하는 함정).
    ///
    /// 파일에 실리는 값이 아니라 IR 안에서만 의미가 있다. `line_segs` 를 통째로 다시
    /// 계산하는 경로(reflow)는 이 값을 0 으로 되돌린다.
    pub layout_only_fill_lines: usize,
    /// 영역 태그 정보
    pub range_tags: Vec<RangeTag>,
    /// 필드 텍스트 범위 (0x03~0x04 사이 텍스트 인덱스 + 컨트롤 인덱스)
    pub field_ranges: Vec<FieldRange>,
    /// 고아 FIELD_END (다단락 필드의 종료 마커 — begin 이 다른 문단). HWPX 전용 (Task #1556).
    pub orphan_field_ends: Vec<OrphanFieldEnd>,
    /// 컨트롤 목록 (표, 그림, 각주 등)
    pub controls: Vec<Control>,
    /// 각 컨트롤에 대응하는 CTRL_DATA 레코드 (라운드트립 보존용)
    /// controls[i]에 대응하는 CTRL_DATA가 있으면 ctrl_data_records[i] = Some(data)
    pub ctrl_data_records: Vec<Option<Vec<u8>>>,
    /// char_count의 최상위 비트 (bit 31) 파싱값.
    /// HWP5 저장기는 현재 list scope의 마지막 문단 여부로 bit 31을 재생성한다.
    pub char_count_msb: bool,
    /// PARA_HEADER tail 보존용 바이트.
    ///
    /// raw_header_extra[0..6]은 numCharShapes/numRangeTags/numLineSegs 자리지만,
    /// HWP5 저장기는 실제 배열 길이로 count 필드를 재생성하고 이 구간을 쓰지 않는다.
    /// raw_header_extra[6..]은 instanceId 및 변경추적 suffix로 보존한다.
    pub raw_header_extra: Vec<u8>,
    /// 원본에 PARA_TEXT 레코드가 존재했는지 (라운드트립 보존용)
    pub has_para_text: bool,
    /// TAB 확장 데이터 (라운드트립 보존용)
    /// 각 탭 문자의 7 code unit (탭 너비, 종류 등) — text 내 '\t' 순서와 1:1 대응
    pub tab_extended: Vec<[u16; 7]>,
    /// 제목 차례 표시 (`<hp:t>` 안의 `<hp:titleMark/>`, HWP5 인라인 `Mtit`/`Mign`)
    ///
    /// 텍스트가 아니라 **텍스트 축 위에 놓인 8유닛 슬롯**이라 `text` 에 싣지 않는다.
    /// 대신 `field_ranges`·`tab_extended` 와 같은 부수 채널로 위치만 보존한다 —
    /// `text` 에 문자를 넣으면 추출·렌더·비교 축이 전부 달라진다.
    pub title_marks: Vec<TitleMark>,
    /// 문단 번호 시작 방식 오버라이드
    /// None = 앞 번호 목록에 이어 (기본)
    /// Some(NumberingRestart) = 이전 번호 이어 / 새 번호 시작
    pub numbering_restart: Option<NumberingRestart>,
    /// [#4149] 셀 단일줄 과밀 판정 memo — `recompose_stored_single_line_if_overflowing`
    /// 전용 파생 캐시. 판정 입력은 (text, char_shapes, 셀 내폭)뿐이다. 제약:
    /// - 직렬화 금지: `Paragraph` 는 serde derive 가 없고 HWP/HWPX 저장기는 필드를
    ///   명시 기록하므로 파일로 새지 않는다. 새 직렬화 경로를 추가하면 이 필드를 제외할 것.
    /// - 스레드: `DocumentCore` 의 `Send` 단언이 `Arc<Vec<Paragraph>>`
    ///   (document_core/mod.rs render normalization 캐시) 경유로 `Paragraph: Sync` 를
    ///   요구한다 — `Cell` 불가, `AtomicU64` 패킹 사용.
    /// - text/char_shapes 를 바꾸는 모든 경로는 `invalidate_single_line_overflow_memo`
    ///   호출 필수 (Clone 은 memo 를 함께 복제하지만, 복제본도 자기 상태 기준으로
    ///   유효하므로 안전 — 이후 변이 시 무효화 규약은 동일하게 적용).
    pub single_line_overflow_memo: SingleLineOverflowMemo,
}

/// [#4149] 단일줄 과밀 판정 memo 저장소 — `AtomicU64` 1개에 (폭 키, 판정) 패킹.
///
/// 인코딩: `0` = 미판정. 그 외 `(width_key as u64) << 1 | overflowed`.
/// `width_key` 는 셀 내폭의 `f32` 비트 — guard 가 내폭 > 0 을 보장하므로 키가 0 이
/// 될 수 없어 유효 인코딩은 sentinel `0` 과 충돌하지 않는다. 폭이 바뀌면(셀 크기
/// 조정) 키 불일치로 자연 재판정된다. f32 축약의 키 충돌은 인접 ulp 폭(상대 ~2⁻²⁴)
/// 뿐이라 ×1.8 임계 판정에 영향이 없다.
///
/// `Relaxed` 순서로 충분하다 — 값은 (문단, 폭)의 결정적 함수라 경합 시 최악이
/// 중복 측정일 뿐 오답이 없다.
#[derive(Debug, Default)]
pub struct SingleLineOverflowMemo(std::sync::atomic::AtomicU64);

impl SingleLineOverflowMemo {
    /// 셀 내폭(px) → memo 폭 키.
    #[inline]
    pub fn width_key(cell_inner_width_px: f64) -> u32 {
        (cell_inner_width_px as f32).to_bits()
    }

    /// 저장된 판정 조회 — 폭 키가 일치할 때만 `Some(overflowed)`.
    #[inline]
    pub fn get(&self, width_key: u32) -> Option<bool> {
        let v = self.0.load(std::sync::atomic::Ordering::Relaxed);
        if v != 0 && (v >> 1) as u32 == width_key {
            Some(v & 1 == 1)
        } else {
            None
        }
    }

    /// 판정 저장. `width_key == 0`(내폭 ≤ 0)은 sentinel 과 겹치므로 저장하지 않는다.
    #[inline]
    pub fn set(&self, width_key: u32, overflowed: bool) {
        if width_key == 0 {
            return;
        }
        let v = ((width_key as u64) << 1) | (overflowed as u64);
        self.0.store(v, std::sync::atomic::Ordering::Relaxed);
    }

    /// 미판정 상태로 되돌린다.
    #[inline]
    pub fn clear(&self) {
        self.0.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 미판정 여부 (invalidation 검증용).
    #[inline]
    pub fn is_unjudged(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed) == 0
    }
}

impl Clone for SingleLineOverflowMemo {
    fn clone(&self) -> Self {
        // 파생 캐시 복제 — 복제본도 자기 (text, char_shapes) 기준으로 유효하다.
        Self(std::sync::atomic::AtomicU64::new(
            self.0.load(std::sync::atomic::Ordering::Relaxed),
        ))
    }
}

/// 문단 스코프 메타데이터 — 문단 병합의 역연산(undo)에서 복원해야 하는 값들.
///
/// `split_at` 이 만드는 새 문단은 앞 문단(`self`)에서 파생되므로 이 값들을 재현할 수
/// 없다. 사용자가 Enter 로 나눌 때는 앞 문단의 서식을 잇는 것이 옳지만, 병합의
/// 역연산으로 쓰일 때는 사라진 뒷 문단의 원래 값이어야 한다. 병합 시 캡처해
/// undo 에서 되돌린다 (Task #2342).
///
/// `char_count_msb` 는 여기 없다 — 저장기가 list scope 의 마지막 문단 여부로
/// 재생성하므로(`serializer/body_text.rs` `char_count_raw`) 파생값이다.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParaMeta {
    pub para_shape_id: u16,
    pub style_id: u8,
    pub column_type: ColumnBreakType,
    pub raw_break_type: u8,
    pub numbering_restart: Option<NumberingRestart>,
    /// PARA_HEADER tail — instanceId 및 변경추적 suffix라 문단마다 고유하다.
    pub raw_header_extra: Vec<u8>,
    /// TAB 확장 데이터 — 문단 전체가 통째로 이동하므로 분할 없이 그대로 옮긴다.
    pub tab_extended: Vec<[u16; 7]>,
}

/// 문단 번호 시작 방식
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NumberingRestart {
    /// 이전 번호 목록에 이어 (다른 번호 체계 후 복귀 시 이전 카운터 복원)
    ContinuePrevious,
    /// 새 번호 목록 시작 (지정 값부터)
    NewStart(u32),
}

/// 문단 텍스트 내 제어 문자 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CtrlChar {
    /// 구역 정의/단 정의
    SectionColumnDef,
    /// 필드 시작
    FieldBegin,
    /// 필드 끝
    FieldEnd,
    /// 탭
    Tab,
    /// 줄 끝 (line break)
    LineBreak,
    /// 그리기 개체/표
    DrawTableObject,
    /// 문단 끝 (para break)
    ParaBreak,
    /// 숨은 설명
    HiddenComment,
    /// 머리말/꼬리말
    HeaderFooter,
    /// 각주/미주
    FootnoteEndnote,
    /// 자동 번호
    AutoNumber,
    /// 페이지 컨트롤
    PageControl,
    /// 책갈피
    Bookmark,
    /// 덧말/글자겹침
    Ruby,
    /// 하이픈
    Hyphen,
    /// 묶음 빈칸
    NonBreakingSpace,
    /// 고정폭 빈칸
    FixedWidthSpace,
    /// 일반 문자
    Char(char),
}

/// 단 나누기 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum ColumnBreakType {
    #[default]
    None,
    /// 구역 나누기
    Section,
    /// 다단 나누기
    MultiColumn,
    /// 쪽 나누기
    Page,
    /// 단 나누기
    Column,
}

/// 글자 모양 참조 (문단 내 위치별 글자 모양)
#[derive(Debug, Clone, Default)]
pub struct CharShapeRef {
    /// 글자 모양이 바뀌는 시작 위치
    pub start_pos: u32,
    /// 글자 모양 ID
    pub char_shape_id: u32,
}

/// 줄 레이아웃 정보 (HWPTAG_PARA_LINE_SEG)
///
/// **표준**: `mydocs/tech/document_ir_lineseg_standard.md` (Task #604)
/// 모든 i32 필드는 HWPUNIT (1 inch = 7200 HWPUNIT).
#[derive(Debug, Clone, Default)]
pub struct LineSeg {
    /// 본 줄이 차지하는 텍스트 시작 위치 (UTF-16 code unit, 문단 시작 기준)
    pub text_start: u32,
    /// 페이지 내 흐름 y 좌표 (HWPUNIT, 페이지 상단 기준 누적 절대값)
    /// HWP3 파서는 현재 항상 0 (별도 task 에서 누적 계산 정정 권고)
    pub vertical_pos: i32,
    /// 줄 높이 (HWPUNIT, line_spacing 포함)
    pub line_height: i32,
    /// 텍스트 부분의 높이 (HWPUNIT)
    pub text_height: i32,
    /// 베이스라인까지 거리 (HWPUNIT, 줄 시작 기준)
    pub baseline_distance: i32,
    /// 줄간격 (HWPUNIT)
    pub line_spacing: i32,
    /// wrap zone x 오프셋 (HWPUNIT, 단(column) 좌측 기준). 0 = wrap 없음
    pub column_start: i32,
    /// 줄 너비 (HWPUNIT). 단 너비와 같으면 wrap 없음
    pub segment_width: i32,
    /// 비트 플래그 (HWP5 PARA_LINE_SEG tag).
    ///
    /// 공식 스펙 기준:
    /// - bit 0: 페이지의 첫 줄
    /// - bit 1: 컬럼의 첫 줄
    /// - bit 16: 텍스트가 배열되지 않은 빈 세그먼트
    /// - bit 17: 줄의 첫 세그먼트
    /// - bit 18: 줄의 마지막 세그먼트
    /// - bit 19: auto-hyphenation 수행
    /// - bit 20: indentation 적용
    /// - bit 21: 문단 머리 모양 적용
    /// - bit 31: 구현 편의 property
    pub tag: u32,
}

impl LineSeg {
    pub const TAG_FIRST_LINE_OF_PAGE: u32 = 1 << 0;
    pub const TAG_FIRST_LINE_OF_COLUMN: u32 = 1 << 1;
    pub const TAG_EMPTY_SEGMENT: u32 = 1 << 16;
    pub const TAG_FIRST_SEGMENT: u32 = 1 << 17;
    pub const TAG_LAST_SEGMENT: u32 = 1 << 18;
    pub const TAG_AUTO_HYPHENATION: u32 = 1 << 19;
    pub const TAG_INDENTATION: u32 = 1 << 20;
    pub const TAG_PARAGRAPH_HEAD: u32 = 1 << 21;
    pub const TAG_IMPLEMENTATION_PROPERTY: u32 = 1 << 31;

    /// 한 줄이 하나의 세그먼트로만 구성될 때 사용하는 HWP5 tag 조합.
    pub const TAG_SINGLE_SEGMENT_LINE: u32 = Self::TAG_FIRST_SEGMENT | Self::TAG_LAST_SEGMENT;
    /// HWP5 출처 문단의 원본 LineSeg 부재 의미를 HWPX 재파스에서도 보존하기 위한 tag 조합.
    pub const TAG_MISSING_LINESEG_PLACEHOLDER: u32 =
        Self::TAG_SINGLE_SEGMENT_LINE | Self::TAG_EMPTY_SEGMENT | Self::TAG_IMPLEMENTATION_PROPERTY;

    /// HWP5 원본에서 LineSeg가 없던 문단을 HWPX 산출물에 명시할 때 쓰는 LineSeg.
    pub fn missing_lineseg_placeholder() -> Self {
        Self {
            tag: Self::TAG_MISSING_LINESEG_PLACEHOLDER,
            ..Self::default()
        }
    }

    /// rhwp가 HWP5 -> HWPX export 중 생성한 원본 LineSeg 부재 보존용 LineSeg인지 여부.
    pub fn is_missing_lineseg_placeholder(&self) -> bool {
        self.line_height == 0
            && self.text_height == 0
            && self.baseline_distance == 0
            && self.line_spacing == 0
            && self.tag & Self::TAG_MISSING_LINESEG_PLACEHOLDER
                == Self::TAG_MISSING_LINESEG_PLACEHOLDER
    }

    /// 페이지의 첫 줄인지 여부
    pub fn is_first_line_of_page(&self) -> bool {
        self.tag & Self::TAG_FIRST_LINE_OF_PAGE != 0
    }

    /// 본 줄이 wrap zone (그림/표 옆) 안에 있는지 (포맷 무관 표준).
    ///
    /// 표준: `mydocs/tech/document_ir_lineseg_standard.md`
    ///
    /// `col_w_hu`: 단 너비 (HWPUNIT). 본 줄의 segment_width 와 비교.
    ///
    /// 판정 본질:
    /// - `column_start > 0`: 단 좌측에서 떨어진 위치 시작 → wrap zone
    /// - `segment_width > 0 AND segment_width < col_w_hu`: 너비가 단 너비보다 작음 → wrap zone
    /// - 모두 거짓: full width 정상 줄
    pub fn is_in_wrap_zone(&self, col_w_hu: i32) -> bool {
        self.column_start > 0 || (self.segment_width > 0 && self.segment_width < col_w_hu)
    }

    /// 컬럼의 첫 줄인지 여부
    pub fn is_first_line_of_column(&self) -> bool {
        self.tag & Self::TAG_FIRST_LINE_OF_COLUMN != 0
    }

    /// 텍스트가 배열되지 않은 빈 세그먼트인지 여부
    pub fn is_empty_segment(&self) -> bool {
        self.tag & Self::TAG_EMPTY_SEGMENT != 0
    }

    /// 줄의 첫 세그먼트인지 여부
    pub fn is_first_segment(&self) -> bool {
        self.tag & Self::TAG_FIRST_SEGMENT != 0
    }

    /// 줄의 마지막 세그먼트인지 여부
    pub fn is_last_segment(&self) -> bool {
        self.tag & Self::TAG_LAST_SEGMENT != 0
    }

    /// indentation 적용 여부
    pub fn has_indentation(&self) -> bool {
        self.tag & Self::TAG_INDENTATION != 0
    }

    /// 문단 머리 모양 적용 여부
    pub fn has_paragraph_head(&self) -> bool {
        self.tag & Self::TAG_PARAGRAPH_HEAD != 0
    }
}

/// 영역 태그 (HWPTAG_PARA_RANGE_TAG)
#[derive(Debug, Clone, Default)]
pub struct RangeTag {
    /// 영역 시작
    pub start: u32,
    /// 영역 끝
    pub end: u32,
    /// 태그 (상위 8비트: 종류, 하위 24비트: 데이터)
    pub tag: u32,
}

/// 제목 차례 표시 — 이 문단을 제목 차례에 넣을지 표시하는 인라인 마커.
///
/// HWP5 는 컨트롤 문자 `0x08` + ctrl_id 로 PARA_TEXT 안에 직접 싣는다(CTRL_HEADER 없음).
/// 실측(한글 2022 양방향, 06699 한 문서에서 둘 다 확인):
///
/// | HWP5 ctrl_id | HWPX |
/// |---|---|
/// | `Mtit` | `<hp:titleMark ignore="1"/>` |
/// | `Mign` | `<hp:titleMark ignore="0"/>` |
///
/// 8 code unit 을 점유하므로 이 마커를 버리면 문단 축이 그만큼 짧아지고,
/// 한글은 축이 어긋난 `<hp:lineseg textpos>` 를 만나면 본문을 통째로 버린다
/// (10k 스윕 F-절단군 — 77 문서·2,237 개).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TitleMark {
    /// `text` 문자열 내 삽입 위치 (이 인덱스의 문자 **앞**에 놓인다)
    pub char_idx: usize,
    /// `ignore` 속성 — `true` 면 `Mtit`, `false` 면 `Mign`
    pub ignore: bool,
}

/// 필드 텍스트 범위 (0x03 FIELD_BEGIN ~ 0x04 FIELD_END 사이 텍스트)
#[derive(Debug, Clone, Default)]
pub struct FieldRange {
    /// text 문자열 내 시작 인덱스 (포함)
    pub start_char_idx: usize,
    /// text 문자열 내 끝 인덱스 (미포함)
    pub end_char_idx: usize,
    /// controls[] 배열 내 인덱스 (해당 Field 컨트롤 참조)
    pub control_idx: usize,
    /// 같은 문단 내 짝을 이루는 `<hp:fieldEnd>` 자신의 `fieldid` 속성값 (0 이면 없음/생략).
    ///
    /// `fieldBegin` 의 `id`(문서 내 고유 ID, `Field::field_id` 로 보존)와 달리, `fieldEnd`
    /// 자신의 `fieldid` 는 별개 값으로 관찰되며(HYPERLINK 등) IR 로 옮기지 않으면 직렬화 시
    /// 항상 소실된다. 고아(다단락) fieldEnd 는 `OrphanFieldEnd::field_id` 로 이미 보존하므로,
    /// 같은 문단 내 짝(matched) 경로에도 대칭적으로 보존한다.
    pub end_field_id: u32,
    /// FIELD_BEGIN 과 FIELD_END **사이에 있는 컨트롤 슬롯 수**.
    ///
    /// 표·그림처럼 텍스트 문자를 만들지 않는 인라인 개체를 감싼 누름틀은
    /// `start_char_idx == end_char_idx`(텍스트 축 0길이)가 된다. 이때 직렬화기가
    /// fieldEnd 를 자기 fieldBegin 직후에 놓으면 개체가 필드 **밖으로** 밀려나
    /// 빈 누름틀이 되고, 한글은 빈 누름틀의 안내문("이곳을 마우스로 누르고 …")을
    /// 본문으로 표시한다(10k 스윕 G-순수증식 16경로 근인). 이 값만큼 슬롯을
    /// 지나서 fieldEnd 를 놓으면 원본 범위가 보존된다.
    pub inner_slot_count: usize,
}

/// 고아 FIELD_END (0x04) — 짝이 되는 FIELD_BEGIN 이 다른 문단에 있는
/// 다단락 필드의 종료 마커. begin 문단에서 `Control::Field` 로 보존되는 것과 달리,
/// end 문단에는 컨트롤·FieldRange 가 없어 8유닛 슬롯을 표현할 산출물이 없다.
/// 이를 기록해 직렬화기가 `<hp:fieldEnd>` 를 같은 위치에 복원한다 (Task #1556).
#[derive(Debug, Clone, Default)]
pub struct OrphanFieldEnd {
    /// text 문자열 내 위치 (이 인덱스 직전에 8유닛 fieldEnd 슬롯이 놓인다).
    /// 텍스트 끝이면 `text.chars().count()`.
    pub char_idx: usize,
    /// `<hp:fieldEnd beginIDRef="..">` — 짝 fieldBegin 의 id 참조.
    pub begin_id_ref: u32,
    /// `<hp:fieldEnd fieldid="..">` — 필드 인스턴스 id.
    pub field_id: u32,
    /// 짝 필드의 HWP5 `ctrl_id`(`%clk` 등). HWP5 저장기가 종료 마커를 쓸 때 쓴다.
    ///
    /// 0 이면 모른다는 뜻이고, 그때는 HWP5 저장에서 마커를 내지 않는다 — 필드 종류를
    /// 지어내면 한글이 짝을 못 맞춘다. HWPX 에서 들어온 고아 마커가 이 경우다.
    pub begin_ctrl_id: u32,
}

impl Paragraph {
    /// `ctrl_data_records` 를 `controls` 길이에 맞춰 `None` 으로 패딩한다.
    ///
    /// [#3214] `ctrl_data_records[i]` 는 `controls[i]` 대응이지만, **두 배열의 길이가 항상
    /// 같지는 않다**. CTRL_DATA 는 HWP5 바이너리 전용 레코드라 HWPX 파서는 이 배열을 채우지
    /// 않고(`parser/hwpx` 에 적재 지점이 없다), HWP3 파서와 편집 커맨드만 쌍으로 관리한다.
    /// 즉 HWPX 로 연 문서는 `ctrl_data_records.len() < controls.len()` 이 정상 상태다.
    ///
    /// 그런데 컨트롤 삽입 경로는 삽입 위치를 `controls` 기준으로 계산한 뒤 그 인덱스를 그대로
    /// `ctrl_data_records.insert(idx, None)` 에 넘긴다. 두 길이가 어긋나 있으면
    /// `insertion index (is N) should be <= len (is M)` 로 패닉하고, WASM 에서는 패닉이
    /// 객체 borrow 를 오염시켜 이후 모든 호출이 `recursive use of an object` 로 실패한다.
    ///
    /// 인덱스 대응에 기대는 쓰기를 하기 전에 이 함수로 정렬한다. 삽입 **전에** 호출하면
    /// `insert_idx <= ctrl_data_records.len()` 이 보장된다.
    pub fn align_ctrl_data_records(&mut self) {
        while self.ctrl_data_records.len() < self.controls.len() {
            self.ctrl_data_records.push(None);
        }
    }

    pub(crate) fn is_split_movable_control(ctrl: &Control) -> bool {
        matches!(
            ctrl,
            Control::Shape(_)
                | Control::Table(_)
                | Control::Picture(_)
                | Control::Equation(_)
                | Control::Footnote(_)
                | Control::Endnote(_)
                | Control::AutoNumber(_)
                | Control::CharOverlap(_)
        )
    }

    fn control_mask_bit(ctrl: &Control) -> u32 {
        match ctrl {
            Control::SectionDef(_) | Control::ColumnDef(_) => 0x0002,
            Control::Field(_) => 0x0003,
            Control::Table(_)
            | Control::Shape(_)
            | Control::Picture(_)
            | Control::Hyperlink(_)
            | Control::Ruby(_)
            | Control::Equation(_)
            | Control::Form(_)
            | Control::Unknown(_) => 0x000B,
            Control::HiddenComment(_) => 0x000F,
            Control::Header(_) | Control::Footer(_) => 0x0010,
            Control::Footnote(_) | Control::Endnote(_) => 0x0011,
            Control::AutoNumber(_) | Control::NewNumber(_) => 0x0012,
            Control::PageNumberPos(_) | Control::PageHide(_) | Control::PageNumCtrl(_) => 0x0015,
            Control::Bookmark(_) | Control::IndexMark(_) => 0x0016,
            Control::CharOverlap(_) => 0x0017,
        }
    }

    fn compute_control_mask_for(
        text: &str,
        controls: &[Control],
        field_ranges: &[FieldRange],
    ) -> u32 {
        let mut mask = 0u32;
        for ctrl in controls {
            mask |= 1u32 << Self::control_mask_bit(ctrl);
        }
        if !field_ranges.is_empty() {
            mask |= 1u32 << 0x0004;
        }
        if text.contains('\t') {
            mask |= 1u32 << 0x0009;
        }
        if text.contains('\n') {
            mask |= 1u32 << 0x000A;
        }
        mask
    }

    fn split_logical_control_positions(&self) -> Vec<usize> {
        if self.text.is_empty() && self.char_offsets.is_empty() {
            let mut inline_seen = 0usize;
            let mut positions = Vec::with_capacity(self.controls.len());
            for ctrl in &self.controls {
                positions.push(inline_seen);
                if Self::is_split_movable_control(ctrl) {
                    inline_seen += 1;
                }
            }
            return positions;
        }

        let text_positions = self.control_text_positions();
        let text_len = self.text.chars().count();
        let mut inline_seen = 0usize;
        let mut positions = Vec::with_capacity(self.controls.len());

        for (ci, ctrl) in self.controls.iter().enumerate() {
            let text_pos = text_positions.get(ci).copied().unwrap_or(text_len);
            positions.push(text_pos + inline_seen);
            if Self::is_split_movable_control(ctrl) {
                inline_seen += 1;
            }
        }

        positions
    }

    fn split_text_pos_for_logical_offset(
        &self,
        logical_offset: usize,
        control_positions: &[usize],
    ) -> usize {
        let controls_before = self
            .controls
            .iter()
            .enumerate()
            .filter(|(_, ctrl)| Self::is_split_movable_control(ctrl))
            .filter(|(ci, _)| {
                control_positions.get(*ci).copied().unwrap_or(usize::MAX) < logical_offset
            })
            .count();

        logical_offset
            .saturating_sub(controls_before)
            .min(self.text.chars().count())
    }

    /// 빈 문단을 생성한다 (문단 끝 마커만 포함).
    ///
    /// `para_shape_id`/`style_id` 는 0, `char_shapes` 는 빈 채로 남는다. 이 0 은
    /// "기본 서식" 이 아니라 그 문서 `header.xml` 의 **0번 항목**이며, 저장기는 빈
    /// `char_shapes` 를 `charPrIDRef="0"` 으로 쓴다. 따라서 이미 존재하는 문서에
    /// 문단을 끼워 넣을 때 이 함수를 쓰면 그 문서의 0번 문단모양·글자모양이 적용된다.
    ///
    /// 상속할 이웃 문단이 있는 경우 [`Paragraph::new_empty_like`] 를 쓴다. 이 함수는
    /// 상속원이 아예 없는 경우 — 새 빈 문서 생성, HTML 임포트, 문단이 하나도 없던
    /// 셀을 파싱할 때 — 에만 쓴다.
    pub fn new_empty() -> Self {
        Paragraph {
            char_count: 1, // 끝 마커(0x000D) 포함
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// `template` 의 서식을 상속한 빈 문단을 생성한다.
    ///
    /// 문단모양(`para_shape_id`), 스타일(`style_id`), 끝 글자모양(마지막
    /// `char_shapes` 엔트리)만 가져온다. 텍스트·컨트롤·필드는 상속하지 않는다.
    /// 새 문단은 템플릿 문단 *뒤에* 이어지므로(문단 끝 Enter), 혼합 글자모양
    /// 문단에서는 첫 엔트리가 아니라 문단 끝의 글자모양이 상속 기준이다.
    pub fn new_empty_like(template: &Paragraph) -> Self {
        Paragraph {
            para_shape_id: template.para_shape_id,
            style_id: template.style_id,
            char_shapes: template
                .char_shapes
                .last()
                .map(|cs| CharShapeRef {
                    start_pos: 0,
                    char_shape_id: cs.char_shape_id,
                })
                .into_iter()
                .collect(),
            ..Paragraph::new_empty()
        }
    }

    /// [#4149] 단일줄 과밀 판정 memo 무효화 — text/char_shapes 를 바꾸는 경로 필수.
    #[inline]
    pub fn invalidate_single_line_overflow_memo(&self) {
        self.single_line_overflow_memo.clear();
    }

    /// 문자의 UTF-16 코드 유닛 수를 반환한다.
    fn char_utf16_len(c: char) -> u32 {
        if (c as u32) > 0xFFFF {
            2
        } else {
            1
        }
    }

    /// char_offset 위치에 텍스트를 삽입한다.
    /// 인라인 컨트롤(각주/미주/수식/새 번호 등, 8 code unit)을 char_offset 위치에 삽입할 때
    /// 문단 메타데이터를 일괄 시프트한다.
    ///
    /// `char_offsets[safe_offset..]` 를 +8 하고, 삽입 지점(UTF-16) 이후의
    /// `char_shapes.start_pos`·`range_tags.start/end`·`line_segs.text_start` 도 +8
    /// 시프트한다. 종전에는 각 삽입 경로가 char_offsets 만 밀고 char_shapes/range_tags 를
    /// 그대로 둬서, 삽입 지점 이후 글자모양 run 경계가 텍스트와 어긋났다(글자모양 오염).
    /// `insert_text_at` 의 시프트 규약과 동형이다.
    ///
    /// **이 문단의 UTF-16 좌표를 들고 있는 것은 전부 여기서 함께 민다.** 하나라도 빠지면
    /// 그것만 8 만큼 어긋난 채 남아, 다음에 그 문단을 다시 조판할 때 값이 튀어 원인이
    /// 삽입이 아닌 곳에서 찾아진다(#4347 에서 line_segs 가 그랬다).
    pub(crate) fn shift_for_inline_control_insert(&mut self, char_offset: usize) {
        if self.char_offsets.is_empty() {
            return;
        }
        let text_len = self.text.chars().count();
        let safe_offset = char_offset.min(text_len);
        // 컨트롤이 삽입되는 UTF-16 위치 — char_offsets 시프트 전에 계산한다.
        let insert_pos: u32 = if safe_offset < self.char_offsets.len() {
            self.char_offsets[safe_offset]
        } else {
            let last_idx = self.char_offsets.len() - 1;
            let last_w = self
                .text
                .chars()
                .nth(last_idx)
                .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                .unwrap_or(1);
            self.char_offsets[last_idx] + last_w
        };
        for co in self.char_offsets[safe_offset..].iter_mut() {
            *co += 8;
        }
        self.shift_position_metadata_for_stream_insertion(insert_pos, 8);
    }

    /// 선행 확장 제어문자가 들어갈 만큼 첫 텍스트 앞 스트림 좌표를 확보한다.
    ///
    /// `SectionDef`·`ColumnDef`처럼 문단 첫 글자보다 앞에 와야 하는 확장 제어문자는 각각
    /// 8 UTF-16 code unit을 쓴다. 이미 확보된 선행 공간은 보존하고 부족한 만큼만 모든
    /// 텍스트 좌표를 민다. `char_offsets`와 같은 좌표계를 쓰는 글자모양, range tag,
    /// 줄 시작도 함께 갱신해야 한다.
    ///
    /// 호출 전에 제어문자를 `controls`의 선두에 넣고, 그 연속 개수를 넘긴다.
    pub(crate) fn reserve_leading_extended_control_slots(&mut self, control_count: usize) {
        const EXTENDED_CONTROL_CODE_UNITS: u32 = 8;

        let required_room = u32::try_from(control_count)
            .unwrap_or(u32::MAX)
            .saturating_mul(EXTENDED_CONTROL_CODE_UNITS);
        let existing_room = self.char_offsets.first().copied().unwrap_or(0);
        let shift = required_room.saturating_sub(existing_room);
        if shift == 0 {
            return;
        }

        for offset in &mut self.char_offsets {
            *offset += shift;
        }
        self.shift_position_metadata_for_stream_insertion(existing_room, shift);
    }

    /// 스트림 삽입으로 이동한 텍스트 좌표와 같은 기준을 쓰는 문단 메타데이터를 갱신한다.
    fn shift_position_metadata_for_stream_insertion(&mut self, insert_pos: u32, shift: u32) {
        if shift == 0 {
            return;
        }
        // [#4149] 제어문자 삽입은 char_shapes 경계를 옮긴다 — memo 무효화 (보수적).
        self.invalidate_single_line_overflow_memo();
        // 문단 시작(pos 0)에 고정된 첫 스타일은 유지(insert_text_at 과 동일).
        for cs in &mut self.char_shapes {
            if cs.start_pos > insert_pos || (cs.start_pos == insert_pos && cs.start_pos > 0) {
                cs.start_pos += shift;
            }
        }
        for rt in &mut self.range_tags {
            if rt.start >= insert_pos {
                rt.start += shift;
            }
            if rt.end >= insert_pos {
                rt.end += shift;
            }
        }
        // [#4347] 줄 시작도 같은 좌표계(UTF-16 code unit)를 쓴다 — 함께 밀지 않으면 저장된
        // 줄 나눔이 삽입 지점 뒤로 8 만큼 어긋난다. 눈에 안 띄다가 문단을 다시 조판하는
        // 순간(그림 배치 토글 따위) 값이 갑자기 8 뛰어 "왕복이 원복을 깼다"로 보인다.
        // 첫 줄은 문단 시작에 고정한다 — 넣은 컨트롤이 그 줄에 든다(char_shapes 와 같은 규약).
        for seg in &mut self.line_segs {
            if seg.text_start > insert_pos || (seg.text_start == insert_pos && seg.text_start > 0) {
                seg.text_start += shift;
            }
        }
    }

    ///
    /// char_offset은 Rust 문자(char) 인덱스이다 (바이트 인덱스가 아님).
    /// 삽입 후 char_offsets, char_shapes, line_segs, char_count가 자동 갱신된다.
    ///
    /// char_offset이 text.chars().count()를 초과하면 인라인 컨트롤 뒤의
    /// 위치로 간주하여 올바른 UTF-16 위치에 삽입한다.
    ///
    /// 반환값은 **실제로 삽입된 char 오프셋**이다. 위 컨트롤 갭 처리 때문에 요청 값과
    /// 다를 수 있으므로, 삽입 뒤 위치를 알려야 하는 호출부는 요청 값이 아니라 이 값을
    /// 기준으로 삼는다 (Task #3216).
    pub fn insert_text_at(&mut self, char_offset: usize, new_text: &str) -> usize {
        if new_text.is_empty() {
            return char_offset.min(self.text.chars().count());
        }
        // [#4149] text 변이 — 단일줄 과밀 memo 무효화.
        self.invalidate_single_line_overflow_memo();

        let text_chars: Vec<char> = self.text.chars().collect();
        let text_len = text_chars.len();

        // char_offset > text_len: 인라인 컨트롤 뒤의 위치
        // navigable_text_len이 text_len보다 클 수 있음 (인라인 컨트롤 포함)
        // 이 경우 char_offset을 text_len으로 clamp하되, UTF-16 위치는
        // 마지막 문자 + 후행 컨트롤 갭을 포함한 값으로 계산
        let effective_char_offset = char_offset.min(text_len);
        let control_positions = self.control_text_positions();
        let inserts_before_inline_control = char_offset <= text_len
            && self
                .controls
                .iter()
                .zip(control_positions.iter())
                .any(|(ctrl, &pos)| {
                    pos == effective_char_offset
                        && matches!(
                            ctrl,
                            Control::Shape(_)
                                | Control::Table(_)
                                | Control::Picture(_)
                                | Control::Equation(_)
                                | Control::Footnote(_)
                                | Control::Endnote(_)
                                | Control::AutoNumber(_)
                        )
                });

        // 바이트 삽입 위치 계산
        let byte_offset: usize = text_chars[..effective_char_offset]
            .iter()
            .map(|c| c.len_utf8())
            .sum();

        // 삽입 지점의 UTF-16 위치 결정
        let utf16_insert_pos: u32 = if char_offset > text_len && !self.char_offsets.is_empty() {
            // 텍스트 끝 이후 (인라인 컨트롤 뒤): 마지막 문자의 UTF-16 위치 + 폭 + 후행 갭
            let last_idx = self.char_offsets.len() - 1;
            let last_char_end =
                self.char_offsets[last_idx] + Self::char_utf16_len(text_chars[last_idx]);
            // 후행 컨트롤 수 = char_offset - text_len
            let trailing_ctrl_count = (char_offset - text_len) as u32;
            last_char_end + trailing_ctrl_count * 8
        } else if inserts_before_inline_control {
            if effective_char_offset == 0 {
                0
            } else if !self.char_offsets.is_empty() {
                let prev_idx = effective_char_offset - 1;
                self.char_offsets[prev_idx] + Self::char_utf16_len(text_chars[prev_idx])
            } else {
                0
            }
        } else if effective_char_offset < self.char_offsets.len() {
            self.char_offsets[effective_char_offset]
        } else if !self.char_offsets.is_empty() {
            let last_idx = self.char_offsets.len() - 1;
            self.char_offsets[last_idx] + Self::char_utf16_len(text_chars[last_idx])
        } else {
            // 텍스트가 비어있을 때: 기존 컨트롤 뒤에 삽입 (각 컨트롤 = 8 code units)
            (self.controls.len() as u32) * 8
        };
        let char_offset = effective_char_offset;

        // 새 텍스트의 UTF-16 총 길이
        let new_chars: Vec<char> = new_text.chars().collect();
        let utf16_delta: u32 = new_chars.iter().map(|c| Self::char_utf16_len(*c)).sum();

        // 1. 텍스트 삽입
        self.text.insert_str(byte_offset, new_text);

        // 2. char_offsets 재구축
        // 삽입 지점 이후의 기존 오프셋을 시프트
        for offset in self.char_offsets[char_offset..].iter_mut() {
            *offset += utf16_delta;
        }
        // 새 문자들의 오프셋 삽입
        let mut new_offsets = Vec::with_capacity(new_chars.len());
        let mut pos = utf16_insert_pos;
        for c in &new_chars {
            new_offsets.push(pos);
            pos += Self::char_utf16_len(*c);
        }
        // char_offset 위치에 새 오프셋 삽입
        let mut updated_offsets = Vec::with_capacity(self.char_offsets.len() + new_offsets.len());
        updated_offsets.extend_from_slice(&self.char_offsets[..char_offset]);
        updated_offsets.extend_from_slice(&new_offsets);
        updated_offsets.extend_from_slice(&self.char_offsets[char_offset..]);
        self.char_offsets = updated_offsets;

        // 3. char_shapes: 삽입 지점 이후의 start_pos를 시프트
        // 문단 시작(pos 0)에 삽입할 때 첫 번째 스타일(start_pos=0)은 유지
        for cs in &mut self.char_shapes {
            if cs.start_pos > utf16_insert_pos {
                cs.start_pos += utf16_delta;
            } else if cs.start_pos == utf16_insert_pos && cs.start_pos > 0 {
                cs.start_pos += utf16_delta;
            }
        }

        // 4. line_segs: 삽입 지점 이후의 text_start를 시프트
        for ls in &mut self.line_segs {
            if ls.text_start > utf16_insert_pos {
                ls.text_start += utf16_delta;
            } else if ls.text_start == utf16_insert_pos && ls.text_start > 0 {
                ls.text_start += utf16_delta;
            }
        }

        // 5. range_tags: 삽입 지점 이후의 start/end를 시프트
        for rt in &mut self.range_tags {
            if rt.start >= utf16_insert_pos {
                rt.start += utf16_delta;
            }
            if rt.end >= utf16_insert_pos {
                rt.end += utf16_delta;
            }
        }

        // 5-1. field_ranges: 삽입 지점 이후의 char 인덱스 시프트
        let inserted_len = new_chars.len();
        for fr in &mut self.field_ranges {
            if fr.start_char_idx > char_offset {
                fr.start_char_idx += inserted_len;
            }
            if fr.end_char_idx >= char_offset {
                fr.end_char_idx += inserted_len;
            }
        }

        // 6. char_count 갱신
        self.char_count += new_chars.len() as u32;

        effective_char_offset
    }

    /// char_offset 위치에서 count개의 문자를 삭제한다.
    ///
    /// char_offset은 Rust 문자(char) 인덱스이다 (바이트 인덱스가 아님).
    /// 삭제 후 char_offsets, char_shapes, line_segs, char_count가 자동 갱신된다.
    /// 반환값: 실제 삭제된 문자 수.
    pub fn delete_text_at(&mut self, char_offset: usize, count: usize) -> usize {
        if count == 0 {
            return 0;
        }

        let text_chars: Vec<char> = self.text.chars().collect();
        let text_len = text_chars.len();

        if char_offset >= text_len {
            return 0;
        }

        // [#4149] text 변이 — 단일줄 과밀 memo 무효화.
        self.invalidate_single_line_overflow_memo();

        // 실제 삭제할 문자 수 (범위 클램핑)
        let actual_count = count.min(text_len - char_offset);
        let del_end = char_offset + actual_count;

        // 바이트 범위 계산
        let byte_start: usize = text_chars[..char_offset].iter().map(|c| c.len_utf8()).sum();
        let byte_end: usize = text_chars[..del_end].iter().map(|c| c.len_utf8()).sum();

        // 삭제 범위의 UTF-16 시작/끝 위치 결정
        let utf16_start: u32 = if char_offset < self.char_offsets.len() {
            self.char_offsets[char_offset]
        } else {
            0
        };
        let utf16_delta: u32 = text_chars[char_offset..del_end]
            .iter()
            .map(|c| Self::char_utf16_len(*c))
            .sum();
        let utf16_end = utf16_start + utf16_delta;

        // 1. 텍스트 삭제
        self.text.drain(byte_start..byte_end);

        // 2. char_offsets: 삭제 범위 제거 + 이후 엔트리 시프트
        let mut updated_offsets =
            Vec::with_capacity(self.char_offsets.len().saturating_sub(actual_count));
        updated_offsets.extend_from_slice(&self.char_offsets[..char_offset]);
        for &offset in &self.char_offsets[del_end..] {
            updated_offsets.push(offset - utf16_delta);
        }
        self.char_offsets = updated_offsets;

        // 3. char_shapes: 삭제 범위 이후 → utf16_delta만큼 감소
        for cs in &mut self.char_shapes {
            if cs.start_pos >= utf16_end {
                cs.start_pos -= utf16_delta;
            } else if cs.start_pos > utf16_start {
                // 삭제 범위 내 → 삭제 시작으로 클램핑
                cs.start_pos = utf16_start;
            }
        }
        // [#3576, #4271] 클램핑으로 같은 start_pos 에 몰린 ref 를 정리한다.
        // char_shapes 는 start_pos 오름차순의 '서로 다른' 경계여야 한다.
        //
        // 삭제 뒤 오른쪽 텍스트가 남으면 utf16_end 의 ref 도 utf16_start 로 이동한다.
        // 이때는 마지막 ref 가 살아남은 오른쪽 텍스트의 글자모양이므로 마지막 것을
        // 보존해야 한다. 첫 ref 를 남기면 삽입+서식 적용을 undo 한 뒤 삽입 런의 서식이
        // 원문 오른쪽에 새어 남는다. 반대로 문단 끝까지 삭제한 경우에는 오른쪽 텍스트가
        // 없으므로 기존 동작대로 첫 ref 를 보존한다.
        let preserve_right_shape = del_end < text_len;
        let mut deduped = Vec::<CharShapeRef>::with_capacity(self.char_shapes.len());
        for cs in self.char_shapes.drain(..) {
            if let Some(previous) = deduped.last_mut() {
                if previous.start_pos == cs.start_pos {
                    if preserve_right_shape && cs.start_pos == utf16_start {
                        *previous = cs;
                    }
                    continue;
                }
            }
            deduped.push(cs);
        }
        self.char_shapes = deduped;

        // 4. line_segs: 삭제 범위 이후 → utf16_delta만큼 감소
        for ls in &mut self.line_segs {
            if ls.text_start >= utf16_end {
                ls.text_start -= utf16_delta;
            } else if ls.text_start > utf16_start {
                ls.text_start = utf16_start;
            }
        }

        // 5. range_tags: 삭제 범위에 따라 축소/조정
        for rt in &mut self.range_tags {
            if rt.start >= utf16_end {
                rt.start -= utf16_delta;
            } else if rt.start > utf16_start {
                rt.start = utf16_start;
            }
            if rt.end >= utf16_end {
                rt.end -= utf16_delta;
            } else if rt.end > utf16_start {
                rt.end = utf16_start;
            }
        }

        // 5-1. field_ranges: 삭제 범위에 따라 축소/조정
        for fr in &mut self.field_ranges {
            if fr.start_char_idx >= del_end {
                fr.start_char_idx -= actual_count;
            } else if fr.start_char_idx > char_offset {
                fr.start_char_idx = char_offset;
            }
            if fr.end_char_idx >= del_end {
                fr.end_char_idx -= actual_count;
            } else if fr.end_char_idx > char_offset {
                fr.end_char_idx = char_offset;
            }
        }
        // start > end (역전)인 경우만 제거. start == end (빈 필드)는 유효한 상태이므로 유지.
        // IME 조합 중 delete→insert 사이클에서 필드가 일시적으로 비워질 수 있음.
        self.field_ranges
            .retain(|fr| fr.start_char_idx <= fr.end_char_idx);

        // 6. char_count 갱신
        self.char_count -= actual_count as u32;

        actual_count
    }

    /// 병합으로 사라질 문단의 스코프 메타데이터를 캡처한다 (undo 복원용).
    pub fn capture_meta(&self) -> ParaMeta {
        ParaMeta {
            para_shape_id: self.para_shape_id,
            style_id: self.style_id,
            column_type: self.column_type,
            raw_break_type: self.raw_break_type,
            numbering_restart: self.numbering_restart,
            raw_header_extra: self.raw_header_extra.clone(),
            tab_extended: self.tab_extended.clone(),
        }
    }

    /// 캡처한 메타데이터를 되돌린다 — 병합 undo 의 `split_at` 직후에 호출한다.
    pub fn apply_meta(&mut self, meta: ParaMeta) {
        self.para_shape_id = meta.para_shape_id;
        self.style_id = meta.style_id;
        self.column_type = meta.column_type;
        self.raw_break_type = meta.raw_break_type;
        self.numbering_restart = meta.numbering_restart;
        self.raw_header_extra = meta.raw_header_extra;
        self.tab_extended = meta.tab_extended;
    }

    /// char_offset 위치에서 문단을 분할한다.
    ///
    /// 현재 문단은 char_offset 이전까지만 유지되고,
    /// char_offset 이후의 텍스트와 메타데이터로 새 문단을 생성하여 반환한다.
    ///
    /// 새 문단의 문단 모양·스타일·번호 시작 방식 등은 `self` 에서 상속된다 — Enter
    /// 분할의 시맨틱이다. 병합의 역연산으로 쓰는 호출부는 `apply_meta` 로 사라진
    /// 문단의 원래 값을 되돌려야 한다 (Task #2342).
    pub fn split_at(&mut self, char_offset: usize) -> Paragraph {
        // [#4149] 분할은 양쪽 text 를 모두 바꾼다 — 앞 절반 memo 무효화.
        // 새 절반은 아래 구성에서 미판정(None)으로 시작한다.
        self.invalidate_single_line_overflow_memo();
        let control_positions = self.split_logical_control_positions();
        let split_pos = self.split_text_pos_for_logical_offset(char_offset, &control_positions);
        let text_chars: Vec<char> = self.text.chars().collect();

        // 분할 지점의 UTF-16 위치
        let utf16_split: u32 = if split_pos < self.char_offsets.len() {
            self.char_offsets[split_pos]
        } else if !self.char_offsets.is_empty() {
            let last = self.char_offsets.len() - 1;
            self.char_offsets[last] + Self::char_utf16_len(text_chars[last])
        } else {
            split_pos as u32
        };

        // === 새 문단 구성 ===

        // 1. 텍스트 분할
        let byte_offset: usize = text_chars[..split_pos].iter().map(|c| c.len_utf8()).sum();
        let new_text = self.text[byte_offset..].to_string();
        self.text.truncate(byte_offset);

        // 2. char_offsets 분할
        let new_char_offsets: Vec<u32> = self.char_offsets[split_pos..]
            .iter()
            .map(|&off| off - utf16_split)
            .collect();
        self.char_offsets.truncate(split_pos);

        // 2-1. 제목 차례 표시 분할 — 문자 인덱스 기준이라 뒤 절반은 원점을 옮긴다.
        let new_title_marks: Vec<TitleMark> = self
            .title_marks
            .iter()
            .filter(|m| m.char_idx >= split_pos)
            .map(|m| TitleMark {
                char_idx: m.char_idx - split_pos,
                ignore: m.ignore,
            })
            .collect();
        self.title_marks.retain(|m| m.char_idx < split_pos);

        // 3. char_shapes 분할
        let mut new_char_shapes: Vec<CharShapeRef> = Vec::new();
        // 분할 지점에서의 활성 스타일 찾기
        let mut active_style_id: u32 = self
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        for cs in &self.char_shapes {
            if cs.start_pos <= utf16_split {
                active_style_id = cs.char_shape_id;
            }
        }

        // 분할 지점 이후의 char_shapes를 새 문단으로 이동 (위치 조정)
        let mut has_zero_pos = false;
        for cs in &self.char_shapes {
            if cs.start_pos >= utf16_split {
                let new_pos = cs.start_pos - utf16_split;
                if new_pos == 0 {
                    has_zero_pos = true;
                }
                new_char_shapes.push(CharShapeRef {
                    start_pos: new_pos,
                    char_shape_id: cs.char_shape_id,
                });
            }
        }
        // 새 문단의 시작(pos 0)에 스타일이 없으면 활성 스타일 추가
        if !has_zero_pos {
            new_char_shapes.insert(
                0,
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: active_style_id,
                },
            );
        }

        // 원래 문단의 char_shapes: 분할 지점 이후 제거
        self.char_shapes.retain(|cs| cs.start_pos < utf16_split);
        if self.char_shapes.is_empty() {
            self.char_shapes.push(CharShapeRef {
                start_pos: 0,
                char_shape_id: active_style_id,
            });
        }

        // 4. line_segs: 원본 치수를 보존하여 리플로우 시 올바른 줄간격 유지
        //    split_at 후 reflow_line_segs()가 첫 번째 LineSeg의 치수를 참조하므로,
        //    원본 HWP의 줄높이/텍스트높이 등을 유지해야 한다.
        let orig_line_seg = self.line_segs.first().cloned();
        let (lh, th, bd, ls, sw, tag) = match orig_line_seg {
            Some(ref o) if o.line_height > 0 => (
                o.line_height,
                o.text_height,
                o.baseline_distance,
                o.line_spacing,
                o.segment_width,
                o.tag,
            ),
            _ => (400, 400, 320, 0, 0, LineSeg::TAG_SINGLE_SEGMENT_LINE),
        };
        let new_line_segs = vec![LineSeg {
            text_start: 0,
            line_height: lh,
            text_height: th,
            baseline_distance: bd,
            line_spacing: ls,
            segment_width: sw,
            tag,
            ..Default::default()
        }];
        // [Task #2299] 앞 절반은 원본 첫 LineSeg 의 vertical_pos 를 유지한다 —
        // 분할해도 문단의 첫 줄 위치는 변하지 않고, 저장 vpos 가 단/쪽 리셋 인코딩인
        // 문단(예: 다단 col1 첫 문단)을 분할해도 그 신호가 살아남아야 한다.
        // 새 절반의 vpos=0 은 배치 전 placeholder 로, 호출측 recalc 가
        // ignore_reset_at 으로 흐름에 연결한다.
        let orig_vpos = orig_line_seg.as_ref().map(|o| o.vertical_pos).unwrap_or(0);
        self.line_segs = vec![LineSeg {
            text_start: 0,
            vertical_pos: orig_vpos,
            line_height: lh,
            text_height: th,
            baseline_distance: bd,
            line_spacing: ls,
            segment_width: sw,
            tag,
            ..Default::default()
        }];

        // 5. range_tags 분할
        let mut new_range_tags: Vec<RangeTag> = Vec::new();
        let mut kept_range_tags: Vec<RangeTag> = Vec::new();
        for rt in &self.range_tags {
            if rt.start >= utf16_split {
                // 완전히 새 문단 쪽
                new_range_tags.push(RangeTag {
                    start: rt.start - utf16_split,
                    end: rt.end - utf16_split,
                    tag: rt.tag,
                });
            } else if rt.end <= utf16_split {
                // 완전히 원래 문단 쪽
                kept_range_tags.push(rt.clone());
            }
            // 경계에 걸치는 태그는 양쪽에서 제거 (단순화)
        }
        self.range_tags = kept_range_tags;

        // 5-1. field_ranges 분할
        let mut new_field_ranges: Vec<FieldRange> = Vec::new();
        let mut kept_field_ranges: Vec<FieldRange> = Vec::new();
        // 5-1a. controls 분할 시 control_idx 리매핑을 위한 맵 (old → new)
        let mut moved_control_idx_map: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for fr in &self.field_ranges {
            if fr.start_char_idx >= split_pos {
                // 완전히 새 문단 쪽 → 인덱스 조정 후 이관 (control_idx는 5-2에서 리매핑)
                new_field_ranges.push(FieldRange {
                    start_char_idx: fr.start_char_idx - split_pos,
                    end_char_idx: fr.end_char_idx - split_pos,
                    control_idx: fr.control_idx,
                    end_field_id: fr.end_field_id,
                    inner_slot_count: fr.inner_slot_count,
                });
            } else if fr.end_char_idx <= split_pos {
                // 완전히 원래 문단 쪽
                kept_field_ranges.push(fr.clone());
            } else {
                // 경계에 걸친 필드: 원래 문단에서 종료
                kept_field_ranges.push(FieldRange {
                    start_char_idx: fr.start_char_idx,
                    end_char_idx: split_pos,
                    control_idx: fr.control_idx,
                    end_field_id: fr.end_field_id,
                    // 문단이 잘려 안쪽 슬롯 소속이 불확실해진다 — 보수적으로 0.
                    inner_slot_count: 0,
                });
            }
        }
        self.field_ranges = kept_field_ranges;

        // Field는 보이는 문자 오프셋에서 한 글자를 차지하지 않는다. 다만 새 문단으로
        // 이관되는 FieldRange가 참조하는 Field control은 범위와 함께 이동해야 한다.
        // 일반 이동형 컨트롤로 분류하면 split의 논리 offset 계산에 Field가 더해져
        // ClickHere 복사/붙여넣기 경계가 한 글자씩 어긋난다.
        let moved_field_control_indices: std::collections::HashSet<usize> = new_field_ranges
            .iter()
            .map(|field_range| field_range.control_idx)
            .collect();

        // 5-2. controls 분할
        //
        // TAC 그림/표/수식 등은 본문에서 한 글자처럼 취급되므로 문단 분할 시
        // logical offset 기준으로 앞뒤 문단에 나뉘어야 한다. SectionDef/ColumnDef 같은
        // 구조 control은 문단 시작에 붙은 문서 구조 정보라 원래 문단에 둔다.
        let old_controls = std::mem::take(&mut self.controls);
        let old_ctrl_data = std::mem::take(&mut self.ctrl_data_records);
        let mut kept_controls = Vec::with_capacity(old_controls.len());
        let mut kept_ctrl_data = Vec::new();
        let mut new_controls = Vec::new();
        let mut new_ctrl_data_records = Vec::new();

        for (ci, ctrl) in old_controls.into_iter().enumerate() {
            let data = old_ctrl_data.get(ci).cloned().flatten();
            let move_to_new = (Self::is_split_movable_control(&ctrl)
                && control_positions.get(ci).copied().unwrap_or(usize::MAX) >= char_offset)
                || moved_field_control_indices.contains(&ci);

            if move_to_new {
                moved_control_idx_map.insert(ci, new_controls.len());
                new_controls.push(ctrl);
                new_ctrl_data_records.push(data);
            } else {
                kept_controls.push(ctrl);
                kept_ctrl_data.push(data);
            }
        }
        // field_ranges의 control_idx를 새 문단의 controls 배열에 맞게 리매핑
        for fr in &mut new_field_ranges {
            if let Some(&new_idx) = moved_control_idx_map.get(&fr.control_idx) {
                fr.control_idx = new_idx;
            }
        }
        self.controls = kept_controls;
        self.ctrl_data_records = kept_ctrl_data;

        // 6. char_count 갱신
        //    원본 문단에 남은 controls는 각각 8 code unit을 차지하므로 반영 필요
        let new_text_char_count = new_text.chars().count() as u32;
        let ctrl_code_units: u32 = self.controls.len() as u32 * 8;
        self.char_count = split_pos as u32 + ctrl_code_units + 1; // +1 for paragraph end marker
        let new_char_count = new_text_char_count + new_controls.len() as u32 * 8 + 1;

        // 7. has_para_text: 빈 문단(텍스트 없고 컨트롤 없음)이면 PARA_TEXT 불필요
        //    HWP 프로그램은 cc=1(빈 문단)에 PARA_TEXT가 있으면 파일 손상으로 판단
        self.has_para_text = !(self.text.is_empty() && self.controls.is_empty());
        let new_has_para_text = !new_text.is_empty() || !new_controls.is_empty();

        self.control_mask =
            Self::compute_control_mask_for(&self.text, &self.controls, &self.field_ranges);
        let new_control_mask =
            Self::compute_control_mask_for(&new_text, &new_controls, &new_field_ranges);

        Paragraph {
            text: new_text,
            char_offsets: new_char_offsets,
            char_shapes: new_char_shapes,
            line_segs: new_line_segs,
            // 분리된 문단의 줄은 새로 계산된 것이라 조판 전용 보강 줄이 없다 (#4677).
            layout_only_fill_lines: 0,
            range_tags: new_range_tags,
            field_ranges: new_field_ranges, // 새 문단으로 이관된 필드 범위
            orphan_field_ends: Vec::new(),
            char_count: new_char_count,
            para_shape_id: self.para_shape_id,
            style_id: self.style_id,
            column_type: ColumnBreakType::None,
            raw_break_type: 0,
            control_mask: new_control_mask,
            controls: new_controls,
            ctrl_data_records: new_ctrl_data_records,
            char_count_msb: false,
            raw_header_extra: self.raw_header_extra.clone(),
            has_para_text: new_has_para_text,
            tab_extended: Vec::new(),
            title_marks: new_title_marks,
            numbering_restart: None,
            // [#4149] 분할 산출 문단은 미판정으로 시작한다.
            single_line_overflow_memo: SingleLineOverflowMemo::default(),
        }
    }

    /// 다른 문단의 텍스트와 메타데이터를 현재 문단 끝에 결합한다.
    ///
    /// 병합 후 other 문단의 내용은 현재 문단에 포함된다.
    /// 반환값: 병합 지점의 char offset (원래 텍스트의 길이).
    pub fn merge_from(&mut self, other: &Paragraph) -> usize {
        // 텍스트 없이 컨트롤만 있는 문단(예: 그림 복사 클립보드)도 병합 대상 (#1323)
        if other.text.is_empty() && other.controls.is_empty() {
            return self.text.chars().count();
        }
        // [#4149] 병합은 text/char_shapes 를 바꾼다 — memo 무효화 (미판정 재시작).
        self.invalidate_single_line_overflow_memo();

        let self_text_len = self.text.chars().count();

        // 현재 문단 끝의 UTF-16 위치.
        // 마지막 문자 뒤(trailing) 컨트롤은 char_offsets에 갭이 인코딩되어 있지 않으므로
        // 컨트롤당 8 code unit을 가산해야 other 텍스트가 컨트롤 갭 뒤로 이어진다 (#1323).
        let trailing_ctrl_units: u32 = self
            .control_text_positions()
            .iter()
            .filter(|&&p| p >= self_text_len)
            .count() as u32
            * 8;
        let utf16_end: u32 = if !self.char_offsets.is_empty() {
            let last = self.char_offsets.len() - 1;
            let text_chars: Vec<char> = self.text.chars().collect();
            self.char_offsets[last] + Self::char_utf16_len(text_chars[last])
        } else {
            0
        } + trailing_ctrl_units;

        // 1. 텍스트 결합
        self.text.push_str(&other.text);

        // 2. char_offsets 결합 (other의 오프셋에 utf16_end 추가)
        for &off in &other.char_offsets {
            self.char_offsets.push(off + utf16_end);
        }

        // 2-1. 제목 차례 표시 결합 — 문자 인덱스 축이라 앞 문단 길이만큼 민다.
        for m in &other.title_marks {
            self.title_marks.push(TitleMark {
                char_idx: m.char_idx + self_text_len,
                ignore: m.ignore,
            });
        }

        // 3. char_shapes 결합 (other의 start_pos에 utf16_end 추가)
        for cs in &other.char_shapes {
            let new_pos = cs.start_pos + utf16_end;
            // 중복 위치에 같은 스타일이면 스킵
            if self
                .char_shapes
                .last()
                .map(|last| last.start_pos == new_pos && last.char_shape_id == cs.char_shape_id)
                .unwrap_or(false)
            {
                continue;
            }
            self.char_shapes.push(CharShapeRef {
                start_pos: new_pos,
                char_shape_id: cs.char_shape_id,
            });
        }

        // 4. line_segs: 원본 치수를 보존하여 리플로우 시 올바른 줄간격 유지
        let orig_line_seg = self.line_segs.first().cloned();
        let (lh, th, bd, ls, sw, tag) = match orig_line_seg {
            Some(ref o) if o.line_height > 0 => (
                o.line_height,
                o.text_height,
                o.baseline_distance,
                o.line_spacing,
                o.segment_width,
                o.tag,
            ),
            _ => (400, 400, 320, 0, 0, LineSeg::TAG_SINGLE_SEGMENT_LINE),
        };
        // [Task #2299] split_at 과 동일하게 호스트(self)의 원본 vertical_pos 를
        // 유지한다 — 병합해도 문단 첫 줄 위치는 변하지 않으며, 0 placeholder 로
        // 재생성하면 편집발 vpos 재계산이 이를 저장 단/쪽 리셋으로 오인해 병합
        // 문단을 구역 상단 좌표에 동결시킨다 (밴드 내 range-delete 시 +1 팬텀 쪽).
        let orig_vpos = orig_line_seg.as_ref().map(|o| o.vertical_pos).unwrap_or(0);
        self.line_segs = vec![LineSeg {
            text_start: 0,
            vertical_pos: orig_vpos,
            line_height: lh,
            text_height: th,
            baseline_distance: bd,
            line_spacing: ls,
            segment_width: sw,
            tag,
            ..Default::default()
        }];

        // 5. range_tags 결합 (other의 start/end에 utf16_end 추가)
        for rt in &other.range_tags {
            self.range_tags.push(RangeTag {
                start: rt.start + utf16_end,
                end: rt.end + utf16_end,
                tag: rt.tag,
            });
        }

        // 5-1. field_ranges 결합 (other의 char 인덱스에 self_text_len 추가)
        //      ctrl_offset은 병합 전 self.controls.len() — 5-2의 controls 병합보다 먼저 캡처
        let ctrl_offset = self.controls.len();
        for fr in &other.field_ranges {
            self.field_ranges.push(FieldRange {
                start_char_idx: fr.start_char_idx + self_text_len,
                end_char_idx: fr.end_char_idx + self_text_len,
                control_idx: fr.control_idx + ctrl_offset,
                end_field_id: fr.end_field_id,
                inner_slot_count: fr.inner_slot_count,
            });
        }

        // 5-2. controls / ctrl_data_records / control_mask 병합 (#1323)
        //      ctrl_data_records[i]는 controls[i] 대응이므로 self 쪽을 None 패딩 후 이어붙인다.
        if !other.controls.is_empty() {
            while self.ctrl_data_records.len() < self.controls.len() {
                self.ctrl_data_records.push(None);
            }
            for i in 0..other.controls.len() {
                self.ctrl_data_records
                    .push(other.ctrl_data_records.get(i).cloned().flatten());
            }
            self.controls.extend(other.controls.iter().cloned());
        }
        // control_mask는 tab/개행 등 텍스트 기반 비트(#1323 이후 확장분)도 포함하므로,
        // other.controls가 비어 있어도 other.text의 tab/개행이 손실되지 않도록
        // split_at과 동일하게 병합 후 상태 전체를 재계산한다.
        self.control_mask =
            Self::compute_control_mask_for(&self.text, &self.controls, &self.field_ranges);

        // 6. char_count 갱신: 텍스트 + 컨트롤(각 8 code unit) + 문단끝(1)
        //    split_at의 ctrl_code_units 계산과 정합. HWPX 직렬화가 char_count에서
        //    컨트롤 수를 역산하므로 컨트롤 유닛 포함 필수.
        self.char_count = (self_text_len + other.text.chars().count()) as u32
            + self.controls.len() as u32 * 8
            + 1;

        // 7. has_para_text: 병합 후 텍스트/컨트롤이 있으면 PARA_TEXT 필요
        if !self.text.is_empty() || !self.controls.is_empty() {
            self.has_para_text = true;
        }

        self_text_len
    }

    /// 주어진 문자 오프셋(char index)에 해당하는 CharShapeRef의 char_shape_id를 반환한다.
    ///
    /// char_shapes가 비어있으면 None을 반환한다.
    /// char_offset에 해당하는 UTF-16 위치를 찾아서 가장 적절한 CharShapeRef를 선택한다.
    pub fn char_shape_id_at(&self, char_offset: usize) -> Option<u32> {
        if self.char_shapes.is_empty() {
            return None;
        }

        // char_offset → UTF-16 위치 변환
        let utf16_pos = if char_offset < self.char_offsets.len() {
            self.char_offsets[char_offset]
        } else if !self.char_offsets.is_empty() {
            // 문단 끝 위치
            let last = *self.char_offsets.last().unwrap();
            let last_char = self.text.chars().nth(self.char_offsets.len() - 1);
            last + last_char
                .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                .unwrap_or(1)
        } else {
            0
        };

        // utf16_pos 이하인 가장 큰 start_pos를 가진 CharShapeRef 찾기
        let mut result_id = self.char_shapes[0].char_shape_id;
        for csr in &self.char_shapes {
            if csr.start_pos <= utf16_pos {
                result_id = csr.char_shape_id;
            } else {
                break;
            }
        }
        Some(result_id)
    }

    /// 인라인 컨트롤이 텍스트의 어느 character 인덱스에 위치하는지 반환한다.
    ///
    /// 일반 경로에서는 `char_offsets` 갭 (인라인 컨트롤당 8 UTF-16 코드 유닛) 의 길이만으로
    /// position 을 분배한다 — 컨트롤 variant 를 보지 않으므로 각주·미주, 그림, 표, 수식,
    /// 자동번호 등 모든 inline 컨트롤이 동일하게 character offset 을 부여받는다.
    ///
    /// `char_offsets` 가 비어있는 폴백 경로에서는 본문 흐름에서 1칸을 차지하는
    /// Shape/Table/Picture/Equation/Footnote/Endnote 만 폭 1을 가산하고,
    /// 그 외 컨트롤은 모두 position 0 에 누적된다 (정밀도 손실 분기).
    ///
    /// # Returns
    ///
    /// `positions[i]` = `controls[i]` 가 삽입되어야 할 텍스트 character 인덱스.
    /// 컨트롤이 없으면 빈 벡터.
    pub fn control_text_positions(&self) -> Vec<usize> {
        let offsets = &self.char_offsets;
        let total_controls = self.controls.len();

        if total_controls == 0 {
            return vec![];
        }

        if offsets.is_empty() {
            // char_offsets가 없는 경우: 인라인 컨트롤을 순차적으로 배치
            // secd/cold 등 비인라인 컨트롤은 position 0, 인라인 컨트롤은 순차 증가
            let mut pos = 0usize;
            let mut positions = Vec::with_capacity(total_controls);
            for ctrl in &self.controls {
                positions.push(pos);
                if matches!(
                    ctrl,
                    Control::Shape(_)
                        | Control::Table(_)
                        | Control::Picture(_)
                        | Control::Equation(_)
                        | Control::Footnote(_)
                        | Control::Endnote(_)
                        | Control::AutoNumber(_)
                ) {
                    pos += 1;
                }
            }
            return positions;
        }

        let chars: Vec<char> = self.text.chars().collect();
        let mut positions = Vec::with_capacity(total_controls);

        // 첫 문자 이전의 갭: 확장 컨트롤이 텍스트 시작 전에 있는 경우
        let gap_before = offsets[0] as usize;
        let n_ctrls_before = gap_before / 8;
        for _ in 0..n_ctrls_before {
            if positions.len() >= total_controls {
                break;
            }
            positions.push(0);
        }

        // 연속된 문자 사이의 갭
        for i in 0..offsets.len().saturating_sub(1) {
            if positions.len() >= total_controls {
                break;
            }
            let current_off = offsets[i] as usize;
            let next_off = offsets[i + 1] as usize;
            let char_width = if chars.get(i).map_or(false, |&c| c as u32 > 0xFFFF) {
                2
            } else {
                1
            };
            // [#3466] 자동번호(제어문자 0x12)는 8 코드 유닛을 점유하면서 가시 placeholder 를
            // **한 글자 남긴다**(파서 두 경로 공통). 그래서 그 글자 뒤에는 8 갭이 남지 않고
            // (8 − 1 = 7 → 7/8 = 0) 갭 분배에서 누락돼, 뒤따르는 컨트롤이 한 칸씩 앞당겨졌다
            // — 수식 k 가 수식 k+1 자리로 가고 마지막 수식은 폴백으로 문단 끝에 붙었다.
            //
            // 판별자는 셋을 **모두** 요구한다. stride 8 하나로는 부족하다 — 탭도 가시 글자를
            // 남기며 8 코드 유닛을 점유하지만(HWP5 `0x09`, HWPX `'\t'` 폭 8) `controls[]` 에는
            // 들어가지 않으므로, stride 만 보면 탭이 컨트롤 자리를 가로채 반대 방향으로 어순이
            // 깨진다.
            //   (1) 이 글자의 stride 가 정확히 8 — 일반 글자는 자기 폭 + 8k 라 8 이 될 수 없다
            //   (2) 그 글자가 자동번호 placeholder 인 공백
            //   (3) 지금 자리를 기다리는 컨트롤이 번호 컨트롤
            // (3) 덕분에 HWPX `newNum`(placeholder 없이 순수 8 갭)은 종전 경로를 그대로 탄다.
            let pending_is_number_control = matches!(
                self.controls.get(positions.len()),
                Some(Control::AutoNumber(_) | Control::NewNumber(_))
            );
            if pending_is_number_control
                && char_width == 1
                && chars.get(i) == Some(&' ')
                && next_off == current_off + 8
            {
                positions.push(i);
                continue;
            }
            if next_off > current_off + char_width {
                let gap = next_off - current_off - char_width;
                let n_ctrls = gap / 8;
                for _ in 0..n_ctrls {
                    if positions.len() >= total_controls {
                        break;
                    }
                    positions.push(i + 1); // 현재 문자 다음에 삽입
                }
            }
        }

        // 갭 분석으로 발견되지 않은 컨트롤의 위치를 텍스트의 `\u{FFFC}` marker
        // 위치로 매핑한다. HWP3 파서가 char_offsets 에 control gap (8) 을 추가하지
        // 않고 sequential [0,1,2,...] 만 push 하는 경우 갭 분석이 실패하여 control
        // 들이 모두 paragraph 끝에 몰리는 회귀를 차단 (sample16 paragraph 394:
        // 3 picture 가 같은 line 에 중복 emit). 갭 분석으로 채워진 positions 의
        // 마지막 인덱스 이후를 search start 로 사용하여 중복 매핑 방지.
        let already_filled = positions.len();
        let mut search_start = positions.last().copied().unwrap_or(0);
        while positions.len() < total_controls {
            // search_start 위치부터 다음 \u{FFFC} marker 찾기
            let next_marker = chars[search_start..]
                .iter()
                .position(|&c| c == '\u{FFFC}')
                .map(|rel| search_start + rel);
            match next_marker {
                Some(abs_pos) => {
                    positions.push(abs_pos);
                    search_start = abs_pos + 1;
                }
                None => {
                    // marker 더 이상 없으면 기존 동작 (chars.len() push)
                    positions.push(chars.len());
                }
            }
        }
        let _ = already_filled; // 향후 디버그용 (현재 미사용)

        positions
    }

    /// Return each control's source `PARA_TEXT` UTF-16 start position.
    ///
    /// `char_offsets` point after every control gap preceding a visible
    /// character. Consumers which anchor geometry in the raw stream therefore
    /// need to reconstruct the individual starts inside that gap instead of
    /// using the visible-text position alone.
    pub(crate) fn control_utf16_positions(&self) -> Vec<u32> {
        let text_positions = self.control_text_positions();
        let text_chars = self.text.chars().collect::<Vec<_>>();
        let text_end = self
            .char_offsets
            .last()
            .zip(text_chars.last())
            .map(|(offset, ch)| *offset + ch.len_utf16() as u32)
            .unwrap_or_else(|| text_chars.iter().map(|ch| ch.len_utf16() as u32).sum());
        let mut raw_positions = vec![text_end; text_positions.len()];

        let mut group_start = 0;
        while group_start < text_positions.len() {
            let text_position = text_positions[group_start];
            let mut group_end = group_start + 1;
            while text_positions.get(group_end) == Some(&text_position) {
                group_end += 1;
            }

            let count = (group_end - group_start) as u32;
            let first_raw = self
                .char_offsets
                .get(text_position)
                .copied()
                .map(|offset| offset.saturating_sub(count * CTRL_CHAR_CODE_UNITS))
                .unwrap_or(text_end);
            for (ordinal, raw_position) in
                raw_positions[group_start..group_end].iter_mut().enumerate()
            {
                *raw_position = first_raw + ordinal as u32 * CTRL_CHAR_CODE_UNITS;
            }
            group_start = group_end;
        }

        raw_positions
    }

    /// 편집/커서 이동용 control position 을 반환한다.
    ///
    /// [`Self::control_text_positions`] 는 HWP/HWPX record stream 의 raw text position 을
    /// 보존한다. 반면 커서 이동은 `SectionDef`, `ColumnDef` 같은 구조 컨트롤을 건너뛰고,
    /// Shape/Table/Picture/Equation/Footnote/Endnote 같은 인라인 개체만 한 글자 폭으로 센다.
    pub fn logical_control_positions(&self) -> Vec<usize> {
        if self.text.is_empty() && self.char_offsets.is_empty() {
            let mut inline_seen = 0usize;
            let mut positions = Vec::with_capacity(self.controls.len());
            for ctrl in &self.controls {
                positions.push(inline_seen);
                if ctrl.is_logical_inline() {
                    inline_seen += 1;
                }
            }
            return positions;
        }

        let text_positions = self.control_text_positions();
        let text_len = self.text.chars().count();
        let mut inline_seen = 0usize;
        let mut positions = Vec::with_capacity(self.controls.len());
        for (ci, ctrl) in self.controls.iter().enumerate() {
            let text_pos = text_positions.get(ci).copied().unwrap_or(text_len);
            positions.push(text_pos + inline_seen);
            if ctrl.is_logical_inline() {
                inline_seen += 1;
            }
        }
        positions
    }

    /// `char_offsets` 중 UTF-16 위치 `utf16_pos` 이상인 첫 번째 codepoint 의
    /// 인덱스를 반환한다. 모든 entry 가 작으면 `char_offsets.len()` (텍스트 끝).
    ///
    /// `char_shapes[i].start_pos` 와 `line_segs[i].text_start` 같은 UTF-16
    /// 단위 위치 필드를 codepoint 인덱스로 정규화할 때 사용한다.
    ///
    /// 본 메서드는 `document_core::helpers::utf16_pos_to_char_idx` 와 동일
    /// 알고리즘이나, 시그니처 호환성 (raw `&[u32]` vs `&self`) 때문에 본체를
    /// 자체 보유한다 — 의존성 방향 (model ← document_core) 보존. 알고리즘이
    /// 1줄 (`iter().position`) 이라 silent drift 위험은 무시 가능.
    ///
    /// # Returns
    ///
    /// `char_offsets` 첫 entry 가 `utf16_pos` 이상인 인덱스. 모든 entry 가
    /// 작으면 `char_offsets.len()`.
    pub fn utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize {
        self.char_offsets
            .iter()
            .position(|&off| off >= utf16_pos)
            .unwrap_or(self.char_offsets.len())
    }

    /// [start_char_offset, end_char_offset) 범위에 new_char_shape_id를 적용한다.
    ///
    /// CharShapeRef 배열을 분할/교체하여 지정 범위만 새 ID로 변경한다.
    /// 범위 경계에서 기존 CharShapeRef가 부분적으로 겹치면 분할한다.
    /// 적용 후 연속 동일 ID는 병합한다.
    pub fn apply_char_shape_range(
        &mut self,
        start_char_offset: usize,
        end_char_offset: usize,
        new_char_shape_id: u32,
    ) {
        if start_char_offset >= end_char_offset || self.char_offsets.is_empty() {
            return;
        }
        // [#4149] char_shapes 변이 — 단일줄 과밀 memo 무효화.
        self.invalidate_single_line_overflow_memo();
        if self.char_shapes.is_empty() {
            self.char_shapes.push(CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            });
        }

        // char offset → UTF-16 위치 변환
        let utf16_start = if start_char_offset < self.char_offsets.len() {
            self.char_offsets[start_char_offset]
        } else {
            return;
        };
        let utf16_end = if end_char_offset < self.char_offsets.len() {
            self.char_offsets[end_char_offset]
        } else if !self.char_offsets.is_empty() {
            let last = *self.char_offsets.last().unwrap();
            let last_char = self.text.chars().nth(self.char_offsets.len() - 1);
            last + last_char
                .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                .unwrap_or(1)
        } else {
            return;
        };

        if utf16_start >= utf16_end {
            return;
        }

        // 문단 내 텍스트가 차지하는 UTF-16 영역의 끝 위치 (복원 범위 제한용)
        // 컨트롤이 있으면 char_offsets가 0이 아닌 위치에서 시작하므로
        // 단순 텍스트 길이가 아닌 마지막 문자의 UTF-16 끝 위치를 사용해야 한다.
        let text_utf16_end: u32 = if !self.char_offsets.is_empty() {
            let last_idx = self.char_offsets.len() - 1;
            let last_char = self.text.chars().nth(last_idx);
            self.char_offsets[last_idx]
                + last_char
                    .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                    .unwrap_or(1)
        } else {
            self.text
                .chars()
                .map(|c| if (c as u32) > 0xFFFF { 2u32 } else { 1u32 })
                .sum()
        };

        // 새 CharShapeRef 배열을 구축
        let mut new_refs: Vec<CharShapeRef> = Vec::new();

        for (i, csr) in self.char_shapes.iter().enumerate() {
            let seg_start = csr.start_pos;
            // 다음 CharShapeRef의 start_pos 또는 문단 끝
            let seg_end = if i + 1 < self.char_shapes.len() {
                self.char_shapes[i + 1].start_pos
            } else {
                u32::MAX
            };

            if seg_end <= utf16_start || seg_start >= utf16_end {
                // 범위와 겹치지 않음 — 그대로 유지
                new_refs.push(csr.clone());
            } else {
                // 겹침 발생
                // 범위 앞부분 (seg_start < utf16_start)
                if seg_start < utf16_start {
                    new_refs.push(CharShapeRef {
                        start_pos: seg_start,
                        char_shape_id: csr.char_shape_id,
                    });
                }

                // 새 ID 삽입 (범위 시작점)
                let insert_start = utf16_start.max(seg_start);
                // 이미 같은 위치에 new_char_shape_id가 있는지 확인
                let already_inserted = new_refs
                    .last()
                    .map(|r| r.start_pos == insert_start && r.char_shape_id == new_char_shape_id)
                    .unwrap_or(false);
                if !already_inserted {
                    // 이전 ref가 같은 start_pos인데 다른 ID이면 교체
                    if let Some(last) = new_refs.last_mut() {
                        if last.start_pos == insert_start {
                            last.char_shape_id = new_char_shape_id;
                        } else {
                            new_refs.push(CharShapeRef {
                                start_pos: insert_start,
                                char_shape_id: new_char_shape_id,
                            });
                        }
                    } else {
                        new_refs.push(CharShapeRef {
                            start_pos: insert_start,
                            char_shape_id: new_char_shape_id,
                        });
                    }
                }

                // 범위 뒷부분 복원 (utf16_end < seg_end, 텍스트 범위 내일 때만)
                if utf16_end < seg_end && utf16_end < text_utf16_end {
                    new_refs.push(CharShapeRef {
                        start_pos: utf16_end,
                        char_shape_id: csr.char_shape_id,
                    });
                }
            }
        }

        // 연속 동일 ID 병합
        let mut merged: Vec<CharShapeRef> = Vec::new();
        for r in new_refs {
            if let Some(last) = merged.last() {
                if last.char_shape_id == r.char_shape_id {
                    continue; // 동일 ID 연속 → 뒤의 것 제거
                }
            }
            merged.push(r);
        }

        self.char_shapes = merged;
    }

    /// 문단의 글자 모양을 단일 CharShapeRef로 초기화한다.
    pub fn set_single_char_shape(&mut self, char_shape_id: u32) {
        // [#4149] char_shapes 변이 — 단일줄 과밀 memo 무효화.
        self.invalidate_single_line_overflow_memo();
        self.char_shapes.clear();
        self.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id,
        });
    }

    /// 스타일 기본 글자 모양 run만 새 ID로 바꾸고 직접 지정된 run은 유지한다.
    pub fn replace_style_char_shape_preserving_overrides(
        &mut self,
        old_char_shape_id: u32,
        new_char_shape_id: u32,
    ) {
        if self.char_shapes.is_empty() {
            self.set_single_char_shape(new_char_shape_id);
            return;
        }

        let mut replaced = false;
        for csr in &mut self.char_shapes {
            if csr.char_shape_id == old_char_shape_id {
                csr.char_shape_id = new_char_shape_id;
                replaced = true;
            }
        }

        if replaced {
            // [#4149] char_shapes 변이 — 단일줄 과밀 memo 무효화.
            self.invalidate_single_line_overflow_memo();
            self.merge_adjacent_char_shapes();
        }
    }

    /// 문단 전체에 글자 스타일의 CharShape를 적용한다.
    pub fn apply_char_shape_to_entire_text(&mut self, char_shape_id: u32) {
        let text_len = self.text.chars().count();
        if text_len == 0 || self.char_offsets.is_empty() {
            self.set_single_char_shape(char_shape_id);
            return;
        }
        self.apply_char_shape_range(0, text_len, char_shape_id);
    }

    fn merge_adjacent_char_shapes(&mut self) {
        let mut merged: Vec<CharShapeRef> = Vec::new();
        for csr in self.char_shapes.drain(..) {
            if let Some(last) = merged.last() {
                if last.char_shape_id == csr.char_shape_id {
                    continue;
                }
            }
            merged.push(csr);
        }
        self.char_shapes = merged;
    }
}

#[cfg(test)]
mod tests;
