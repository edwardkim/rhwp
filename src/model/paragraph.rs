//! 문단 (Paragraph, CharRun, LineSeg, RangeTag)

use super::control::Control;

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
    /// 문단 번호 시작 방식 오버라이드
    /// None = 앞 번호 목록에 이어 (기본)
    /// Some(NumberingRestart) = 이전 번호 이어 / 새 번호 시작
    pub numbering_restart: Option<NumberingRestart>,
}

/// 문단 번호 시작 방식
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq)]
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

/// 필드 텍스트 범위 (0x03 FIELD_BEGIN ~ 0x04 FIELD_END 사이 텍스트)
#[derive(Debug, Clone, Default)]
pub struct FieldRange {
    /// text 문자열 내 시작 인덱스 (포함)
    pub start_char_idx: usize,
    /// text 문자열 내 끝 인덱스 (미포함)
    pub end_char_idx: usize,
    /// controls[] 배열 내 인덱스 (해당 Field 컨트롤 참조)
    pub control_idx: usize,
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
}

impl Paragraph {
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
            Control::PageNumberPos(_) | Control::PageHide(_) => 0x0015,
            Control::Bookmark(_) => 0x0016,
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

    /// 문자의 UTF-16 코드 유닛 수를 반환한다.
    fn char_utf16_len(c: char) -> u32 {
        if (c as u32) > 0xFFFF {
            2
        } else {
            1
        }
    }

    /// char_offset 위치에 텍스트를 삽입한다.
    ///
    /// char_offset은 Rust 문자(char) 인덱스이다 (바이트 인덱스가 아님).
    /// 삽입 후 char_offsets, char_shapes, line_segs, char_count가 자동 갱신된다.
    ///
    /// char_offset이 text.chars().count()를 초과하면 인라인 컨트롤 뒤의
    /// 위치로 간주하여 올바른 UTF-16 위치에 삽입한다.
    pub fn insert_text_at(&mut self, char_offset: usize, new_text: &str) {
        if new_text.is_empty() {
            return;
        }

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

    /// char_offset 위치에서 문단을 분할한다.
    ///
    /// 현재 문단은 char_offset 이전까지만 유지되고,
    /// char_offset 이후의 텍스트와 메타데이터로 새 문단을 생성하여 반환한다.
    pub fn split_at(&mut self, char_offset: usize) -> Paragraph {
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

        // 5-1. field_ranges 분할 (필드 control은 원래 문단에 유지)
        self.field_ranges.retain(|fr| fr.end_char_idx <= split_pos);

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
            let move_to_new = Self::is_split_movable_control(&ctrl)
                && control_positions.get(ci).copied().unwrap_or(usize::MAX) >= char_offset;

            if move_to_new {
                new_controls.push(ctrl);
                new_ctrl_data_records.push(data);
            } else {
                kept_controls.push(ctrl);
                kept_ctrl_data.push(data);
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
            Self::compute_control_mask_for(&new_text, &new_controls, &Vec::new());

        Paragraph {
            text: new_text,
            char_offsets: new_char_offsets,
            char_shapes: new_char_shapes,
            line_segs: new_line_segs,
            range_tags: new_range_tags,
            field_ranges: Vec::new(), // controls가 이동하지 않으므로 새 문단에는 필드 없음
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
            numbering_restart: None,
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
            self.control_mask |= other.control_mask;
        }

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
