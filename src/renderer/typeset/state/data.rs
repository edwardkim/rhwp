//! 상태의 불변 관측면. 쓰기 권한은 state 소유자에 한정한다.
use crate::renderer::typeset::{
    ColumnType, DeferredSquarePictureControl, DeferredTableControl, EndnoteParaSource, EndnoteRef,
    PageContent, PageItem, PageLayoutInfo, Paragraph, VisibleFloatExclusion,
};
pub(in crate::renderer::typeset) struct StateView {
    /// 완성된 페이지 목록
    pub(in crate::renderer::typeset) pages: Vec<PageContent>,
    /// 현재 단에 쌓이는 항목
    pub(in crate::renderer::typeset) current_items: Vec<PageItem>,
    /// 현재 단에서 소비된 높이 (px)
    pub(in crate::renderer::typeset) current_height: f64,
    /// 현재 단 시작 시점의 논리 높이 (px)
    pub(in crate::renderer::typeset) current_start_height: f64,
    /// 현재 단에 미주 흐름 항목이 포함되어 있는지 여부
    pub(in crate::renderer::typeset) current_endnote_flow: bool,
    /// [#5886] 현재 단에 문단-사이 compact 되감김을 넣었으면, 이후 문단도
    /// 렌더 순차 적층이 용지 밖으로 나가는지 시뮬한다.
    pub(in crate::renderer::typeset) column_had_compact_endnote_rewind: bool,
    /// [Task #1082] 현재 단에서 마지막으로 배치된 본문 FullParagraph 의 bottom vpos (HU,
    /// 섹션 절대값). 미주 vpos-delta 누적의 첫 항목 base 시드용. 단 advance 시 None.
    pub(in crate::renderer::typeset) prev_body_bottom_vpos: Option<i32>,
    /// [#2279] 현재 단의 flow 과소 누계 (px) — 문단 place 시 flow_advance_height 가
    /// spacing_after/trailing ls 를 트림한 차액(total_height − advance)의 합.
    /// 렌더러(layout)와 한글은 이 성분을 가산하므로, footer(발신명의) fit 판정의
    /// 렌더-정합 좌표 복원용. 단 advance 시 0.
    pub(in crate::renderer::typeset) flow_underrun: f64,
    /// [compat 2024] 이번 단에서 자리차지 표 앵커 문단의 선행 앵커 줄 세그를
    /// 흐름에서 회수한 양(px 누계). hangul2024_layout 에서만 쌓이며, 저장 vpos
    /// 되감김 쪽-경계 신호를 재적합으로 덮을 자격 판정에 쓴다 — 회수가 없던
    /// 쪽에서는 되감김 신호를 그대로 존중해 여타 문서 동작을 바꾸지 않는다.
    pub(in crate::renderer::typeset) hangul2024_reclaimed: f64,
    /// [compat 2024] 저장 리셋/되감김 신호를 덮은 빈 문단의 인덱스 — 그 문단만
    /// place 적합을 우회해 쪽 하단 여백으로 흘린다(이웃 빈 문단까지 흘리면
    /// 2024 보다 한 문단 과적재, idx22 실측).
    pub(in crate::renderer::typeset) hangul2024_spill_para: Option<usize>,
    /// [#2279 pi78] 이 문서에서 저장 ladder 의 host spacing 누락 서명(OMIT)이
    /// 검출됐는가 — 기계생성 압축 ladder 문서군 판별(문서 단위, 리셋 없음).
    /// 분할 진입 첫 줄 full-advance 요구는 이 문서군에만 적용한다.
    pub(in crate::renderer::typeset) stored_ladder_spacing_omitted: bool,
    /// [#2279 OMIT-eager] 구역 시작 사전 스캔으로 확정한 fresh-재계산 문서군
    /// (spacing-누락 스텝 + 본문 텍스트 쪽 리셋 + segless 본문 문단 동시 보유).
    /// 페이지말 빈 문단 재배치·sa 스냅 보존 규칙은 이 판별에만 발동한다 —
    /// lazy(#2383) 공유 플래그에 얹으면 저장 흐름 신뢰 문서(sample16 #2158 핀)
    /// 까지 번져 +1 회귀.
    pub(in crate::renderer::typeset) omit_fresh_recalc_doc: bool,
    /// [#5699 H1] 이 쪽에서 사다리-미계상 표 밴드 교정으로 확보한 흐름 바닥(px).
    /// 후속 문단의 저장 vpos 후방 스냅이 교정분을 되돌리지 못한다. 쪽 단위 리셋.
    pub(in crate::renderer::typeset) ladder_band_floor: f64,
    /// [#5699 H1] 이번 단에서 교정 판별이 발동한 표 (para, ctrl) — 단 flush 시
    /// 소속 페이지의 `ladder_band_tables` 로 이관해 렌더러와 판정을 공유한다.
    pub(in crate::renderer::typeset) current_ladder_band_tables: Vec<(usize, usize)>,
    /// 현재 단 인덱스
    pub(in crate::renderer::typeset) current_column: u16,
    /// 단 수
    pub(in crate::renderer::typeset) col_count: u16,
    /// 페이지 레이아웃
    pub(in crate::renderer::typeset) layout: PageLayoutInfo,
    /// 구역 인덱스
    pub(in crate::renderer::typeset) section_index: usize,
    /// 각주 높이 누적
    /// [#4090] Square 어울림 개체가 만든 세로 배제 밴드의 바닥(쪽 기준 px).
    /// 밴드를 벗어나거나 쪽이 끝날 때 흐름을 이 값으로 끌어올린다. 0.0 = 없음.
    pub(in crate::renderer::typeset) square_band_bottom: f64,
    /// Square 배제 밴드의 흐름 기준 상단. 옆 문단의 저장 좌표를 현재 흐름 좌표계로
    /// 옮겨 밴드 바닥을 확장할 때만 사용한다.
    pub(in crate::renderer::typeset) square_band_top: Option<f64>,
    pub(in crate::renderer::typeset) current_footnote_height: f64,
    /// [Task #1658 v3] 페이지 하단 고정 표(vert=쪽·valign=Bottom, 결재/서명 틀)의
    /// 하단 배타 영역 높이 — 겹침 허용이므로 합이 아닌 max(union). 본문 텍스트는
    /// 이 영역 위까지만 흐른다 (available_height 차감). 페이지 전환 시 리셋.
    pub(in crate::renderer::typeset) current_bottom_fixed_exclusion: f64,
    /// [Task #1658 v3] 이 페이지에서 하단 고정 표가 소비했을 저장-flow 높이 누계.
    /// 한글 저장 vpos 는 하단 틀도 문서순으로 누적하므로, 후속 틀의 vpos 동기화
    /// (#1611) 시 이 값을 차감해야 본문 텍스트 끝 위치가 복원된다.
    pub(in crate::renderer::typeset) bottom_fixed_consumed_flow: f64,
    /// [#2279 footer-오염] 이 페이지에 PAGE-앵커(vertRelTo=Page, vertAlign=Top)
    /// 절대배치 비-TAC 표가 배치됐는가. 이 표들의 저장 vpos 누적은 절대 위치
    /// 산물이라 본문 흐름과 무관하게 부풀며(36496000 pi3: 저장 스텝 +482px vs
    /// 흐름 +143px), 이후 footer 의 저장 vpos 동기화(target_y)를 오염시킨다.
    /// 켜져 있으면 footer 는 stored 동기화 없이 흐름 좌표로 판정한다.
    pub(in crate::renderer::typeset) page_has_page_abs_top_table: bool,
    /// [#2813] 저장 앵커 줄이 float 스택 아래를 인코딩하는 host 문단: 이 문단의
    /// PartialParagraph(앵커 줄) 아이템을 표들 뒤에 밀어 넣어(한글 문서순) 렌더가
    /// 표를 줄 이후 흐름에 배치하지 않게 한다. 값은 해당 para_idx.
    pub(in crate::renderer::typeset) defer_host_line_item_para: Option<usize>,
    /// 첫 각주 여부
    pub(in crate::renderer::typeset) is_first_footnote_on_page: bool,
    /// 현재 physical page에 실제로 그려질 각주 구분선이 이미 예약됐는가.
    /// 번호 없는 continuation tail은 첫 항목이어도 구분선을 생략할 수 있으므로,
    /// `is_first_footnote_on_page`만으로는 뒤에 오는 일반 note의 separator 비용을
    /// 판정할 수 없다.
    pub(in crate::renderer::typeset) current_page_has_footnote_separator: bool,
    /// 각주 구분선 오버헤드
    pub(in crate::renderer::typeset) footnote_separator_overhead: f64,
    /// 각주 사이 간격
    pub(in crate::renderer::typeset) footnote_between_notes_margin: f64,
    /// 각주 안전 여백
    pub(in crate::renderer::typeset) footnote_safety_margin: f64,
    /// [#2559] 현재 구역에 꼬리말 정의가 없는가. 빈 꼬리말 밴드는 각주가 먼저
    /// 사용하므로, 이 경우에만 각주 높이의 일부를 본문 가용 높이에서 회수한다.
    pub(in crate::renderer::typeset) section_has_no_footer: bool,
    /// 존(zone) y 오프셋 (다단 나누기 시 누적)
    pub(in crate::renderer::typeset) current_zone_y_offset: f64,
    /// 현재 존의 레이아웃 오버라이드
    pub(in crate::renderer::typeset) current_zone_layout: Option<PageLayoutInfo>,
    /// 다단 첫 페이지 여부
    pub(in crate::renderer::typeset) on_first_multicolumn_page: bool,
    /// Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지하는 높이 (px).
    /// col 1 이상으로 advance 시 zone_y_offset에 반영.
    pub(in crate::renderer::typeset) pending_body_wide_top_reserve: f64,
    /// visible text host 의 양수 offset 자리차지 표가 후속 문단을 밀어내는 구간.
    pub(in crate::renderer::typeset) visible_float_exclusions: Vec<VisibleFloatExclusion>,
    /// 현재 쪽에 실제 배치된 그림의 점유 영역(용지 좌표). 다음 단에도 간섭할 수 있다.
    pub(in crate::renderer::typeset) side_wrap_exclusions:
        std::collections::BTreeMap<(usize, usize), crate::renderer::layout_frame::FrameExclusion>,
    pub(in crate::renderer::typeset) inline_placements: std::collections::HashMap<
        (usize, usize),
        crate::renderer::float_placement::InlineBoxPlacement,
    >,
    pub(in crate::renderer::typeset) inline_flow_plans:
        std::collections::HashMap<usize, crate::renderer::inline_flow::InlineFlowPlan>,
    pub(in crate::renderer::typeset) paragraph_float_placements: std::collections::HashMap<
        (usize, usize),
        crate::renderer::float_placement::ParagraphFloatPlacement,
    >,
    /// 단 상대 TAC 물리 하단. 저장 host 높이와 별개로 다음 어울림 후보 줄을 제한한다.
    pub(in crate::renderer::typeset) inline_box_flow_bottom: f64,
    /// 같은 문단의 선행 RowBreak 표가 continuation 을 만들 때 후행 co-anchored 표를
    /// 후속 섹션 블록 뒤로 잠시 미루기 위한 큐.
    pub(in crate::renderer::typeset) deferred_table_controls: Vec<DeferredTableControl>,
    /// native HWP5 page-tail Square 그림을 다음 physical page의 시작에 넣는 큐.
    /// 단일 컬럼·caption 보유 picture 형상으로 한정한다.
    pub(in crate::renderer::typeset) deferred_next_page_square_pictures:
        Vec<DeferredSquarePictureControl>,
    /// 다음 physical page의 flush 시점에만 앞에 붙일 Square picture.
    /// `current_items`에 즉시 넣으면 out-of-flow 그림이 문단 fit/vpos 상태를 바꾸어
    /// p1356 뒤 본문을 한 쪽 더 분할한다. layout 순서에는 앞에 있어야 하지만,
    /// typeset 흐름 항목으로는 보이면 안 된다.
    pub(in crate::renderer::typeset) page_start_square_pictures: Vec<DeferredSquarePictureControl>,
    /// 표 조판 중 fragment별로 각주를 등록한 source 키. caller의 기존 표 완료 뒤
    /// 일괄 등록을 건너뛰어 중복을 막는다.
    pub(in crate::renderer::typeset) fragment_queued_table_footnotes:
        std::collections::HashSet<(usize, usize)>,
    /// RowBreak 표의 남은 cell-footnote를 새 physical page에 먼저 예약했는가.
    /// 호출자 루프가 표 host의 저장 VPOS를 다시 기록하지 않고, 새 page 본문은
    /// 독립한 VPOS cursor로 시작하도록 하는 1회 신호다.
    pub(in crate::renderer::typeset) reset_vpos_after_queued_table_footnote_page: bool,
    /// [Task #1753] 지연 이월되는 visible-host 자리차지 표 직전에 현재 쪽 잔여 공간으로
    /// 선행 배치(prefill)된 후속 문단들 — 메인 루프에서 스킵.
    pub(in crate::renderer::typeset) prefilled_paras: std::collections::HashSet<usize>,
    /// [Task #1755] 이월 전 쪽에 host 텍스트 줄을 PartialParagraph 로 pre-emit 한 문단 —
    /// layout 의 마지막 fragment 뒤 host 렌더 억제 신호(PaginationResult 로 전달).
    pub(in crate::renderer::typeset) pre_emitted_host_paras: std::collections::HashSet<usize>,
    /// [#2015] pre-emit 한 host 텍스트의 실제 높이(px). vert_offset(para_start 기준)를
    /// current_height(=para_start+host_h) 기준으로 환산할 때 감액분으로 쓴다. typeset 예산과
    /// layout(table_partial.rs) 배치가 동일 감액을 적용해 정합한다.
    pub(in crate::renderer::typeset) pre_emitted_host_heights:
        std::collections::HashMap<usize, f64>,
    /// [Task #359] 다음 pi 가 vpos-reset 가드를 발동할 예정 → 현재 pi 의 fit 안전마진 비활성화.
    /// 단독 항목 페이지 발생 차단용.
    pub(in crate::renderer::typeset) skip_safety_margin_once: bool,
    /// [Task #1725] tail-before-vpos-reset 문단 1회 각주 안전마진(40px) 비활성화.
    /// 각주 있는 페이지에서 한글 LINESEG 는 tail 문단을 본문에 배치(각주는 아래)하는데,
    /// rhwp 각주 예약(+40px 버퍼)이 tail 을 수 px 초과로 밀어 near-empty 페이지 over-pagination.
    pub(in crate::renderer::typeset) skip_footnote_margin_once: bool,
    /// 다음 저장 vpos-reset 직전 tail의 실제 저장 line bounds. 현재 flow와 저장 bottom의
    /// 차이만 다음 문단의 fit allowance로 쓴다.
    pub(in crate::renderer::typeset) tail_saved_bounds_once: Option<(f64, f64)>,
    /// #2439: 단일 양수-offset 빈 호스트 RowBreak 표 뒤 일반 문단의 1회 엄격 fit.
    /// 이 문단은 표의 실제 painted bottom 뒤에서 시작하므로 저장 page-tail 예외와
    /// trailing line-spacing 트림을 적용하지 않는다.
    pub(in crate::renderer::typeset) strict_plain_text_fit_after_empty_host_float_once: bool,
    /// [#2403] 소스분기 질의 표면 — 단일 소유, typeset 진입 시 set. 종전 4필드의
    /// 시멘틱 승계: hwp3_layout()=HWP3→HWP5 변환본(widow 방지 등 variant 분기,
    /// Task #1007) / hwp3_native_layout()=원본 HWP3 (저장 LINE_SEG 계약) /
    /// hwpx_stored_layout()=HWPX 원본 (빈 앵커 TAC 표 host_line_spacing 미가산,
    /// Task #1147) / hwp5_origin_hwpx()=rhwp HWP5→HWPX 산출물 (HWP5 저장 행높이·
    /// pagination marker 보존).
    pub(in crate::renderer::typeset) profile: crate::model::provenance::LayoutCompatibilityProfile,
    /// 문서 전체에 실제 저장 LineSeg가 있는지 여부. HWP5-origin CFB라도 LineSeg가 전부
    /// 비어 있으면 저장 vpos 기반 흐름이 아니라 재조판 흐름으로 취급한다.
    pub(in crate::renderer::typeset) has_stored_line_segs: bool,
    /// [Task #362] 한컴 빈 줄 감추기 옵션 (SectionDef bit 19). true 이면 페이지 시작에서
    /// overflow 유발하는 빈 paragraph 최대 2개까지 height=0 처리.
    pub(in crate::renderer::typeset) hide_empty_line: bool,
    /// [Task #362] 현재 페이지에서 감춘 빈 줄 수 (페이지마다 reset, 최대 2).
    pub(in crate::renderer::typeset) hidden_empty_lines: u32,
    /// [Task #362] 감춘 빈 줄이 적용된 페이지 인덱스 (페이지 변경 감지용).
    pub(in crate::renderer::typeset) hidden_empty_page_idx: usize,
    /// [Task #362] hide_empty_line 으로 감춘 paragraph 인덱스 (PaginationResult 에 포함).
    pub(in crate::renderer::typeset) hidden_empty_paras: std::collections::HashSet<usize>,
    /// [#6146] 저장 vpos 리셋으로 다음 쪽에 넘어가는 문단의 **자리차지 밴드**를 떠나는
    /// 쪽의 흐름 말미에 남긴 (문단, 컨트롤) 집합. 컨트롤 순회에서 다시 배치하지 않는다.
    pub(in crate::renderer::typeset) page_tail_spilled_floats:
        std::collections::HashSet<(usize, usize)>,
    /// [Task #836] 미주 목록 (섹션별 수집, 문서 끝에 렌더).
    pub(in crate::renderer::typeset) endnotes: Vec<EndnoteRef>,
    pub(in crate::renderer::typeset) endnote_paragraphs: Vec<Paragraph>,
    pub(in crate::renderer::typeset) endnote_para_sources: Vec<EndnoteParaSource>,
    /// [Task #1246] 현재 섹션 미주의 between-notes 마진(HU, 0=미적용). HeightCursor 가 미주 사이
    /// min-gap 보정에 사용. 모든 경계에서 동일한 섹션 설정값이므로 스칼라로 보관.
    pub(in crate::renderer::typeset) endnote_between_notes_hu: i32,
    /// 현재 섹션 미주의 정규화된 "구분선 위" 마진(HU).
    pub(in crate::renderer::typeset) endnote_separator_above_hu: i32,
    /// 현재 섹션 미주의 정규화된 "구분선 아래" 마진(HU).
    pub(in crate::renderer::typeset) endnote_separator_below_hu: i32,
    /// [Task #362] Square wrap 표의 column_start (HU). -1 = 비활성. 후속 같은 cs/sw paragraph 흡수용.
    pub(in crate::renderer::typeset) wrap_around_cs: i32,
    /// NO_LS 문서용 합성 어울림 배제 사각형들.
    /// (x0, x1, flow_top, flow_bottom) — x 는 페이지 px, y 는 단 공통 flow 좌표(px).
    /// Square 그림은 앵커 단 밖(다른 단 위)에 놓일 수 있으므로 페이지 공간으로 기억해
    /// 두고, 각 문단 배치 시 현재 단과의 교차로 텍스트 존(cs/sw)을 산출한다.
    /// 쪽이 바뀌면 비운다.
    pub(in crate::renderer::typeset) wrap_synth_rects: Vec<(f64, f64, f64, f64)>,
    /// [Task #362] Square wrap 표의 segment_width (HU). -1 = 비활성.
    pub(in crate::renderer::typeset) wrap_around_sw: i32,
    /// [Task #362] Square wrap 표가 있는 paragraph 인덱스 (WrapAroundPara 에 기록).
    pub(in crate::renderer::typeset) wrap_around_table_para: usize,
    /// 비-TAC Picture/Shape Square wrap: any_seg_matches만으로 후속 문단 판정 허용.
    /// 그림의 lineseg는 첫 seg cs=0일 수 있어 전체 seg 중 하나라도 일치하면 흡수.
    pub(in crate::renderer::typeset) wrap_around_any_seg: bool,
    /// [#6175] 밴드가 호스트 문단의 저장 사다리가 아니라 어울림 개체의 기하에서
    /// 유도됐다 — 개체 종류(묶음 포함)와 무관하게 뒤따르는 문단을 개체 옆으로 흘린다.
    pub(in crate::renderer::typeset) wrap_around_derived_band: bool,
    /// [#1955] 글뒤로/글앞으로(BehindText/InFrontOfText) 비-TAC 표 anchor 문단.
    /// 이 wrap 은 본문 플로우를 소비하지 않으므로(한글: 후속 문단이 anchor 쪽에
    /// 남음), 직후의 빈 후행 문단들을 anchor 첫 fragment 단에 소급 흡수한다.
    /// 비어있지 않은 문단/새 표/쪽나누기를 만나면 해제.
    pub(in crate::renderer::typeset) behind_float_table_para: Option<usize>,
    /// [#4533 HWP3] 현재 문단 다음 문단의 첫 저장 lineseg vpos — 자리차지
    /// 밴드 비예약(사다리 증거) 판별용. 문단 루프 머리에서 세팅.
    pub(in crate::renderer::typeset) next_para_first_stored_vpos: Option<i32>,
    /// [#5870] 다음 문단이 빈 host 자리차지 표 앵커인가 — 빈-host float 의
    /// 물리-사다리 여분 가산 발동 조건. 문단 루프 머리에서 세팅.
    pub(in crate::renderer::typeset) next_para_is_empty_float_table_anchor: bool,
    /// [#6312] 다음 문단이 개체 없는 실텍스트 본문인가 — 글 있는 자리차지 host
    /// 줄 상자 가산의 사다리 게이트.
    pub(in crate::renderer::typeset) next_para_is_plain_text: bool,
    /// [#1955] 글뒤로 표 후행 빈 문단의 보류 흡수 목록. 표 fragment 는 지연 flush
    /// 되므로 흡수 시점에는 anchor 첫 fragment 단을 찾을 수 없다 — 페이지 확정 후
    /// (최종 flush 뒤) 첫 fragment 단에 일괄 부착한다.
    pub(in crate::renderer::typeset) behind_pending_absorbs:
        Vec<crate::renderer::pagination::WrapAroundPara>,
    /// [#4514] 이 문단의 overlay 표가 #703 Shape 단축(흐름 소비 0)으로 배치되었음.
    /// 이 앵커는 #1955 흡수를 arming 하지 않는다 — 흡수의 전제("표가 fragment 로
    /// 플로우를 이미 소비")가 성립하지 않아, 후행 빈 문단이 유일한 흐름 공간이다
    /// (sample1-repro 저장 사다리: 필러가 각자 줄 높이만큼 전진해 표 높이를 채움.
    /// 흡수하면 갭이 어느 쪽에도 계상되지 않아 후속 표가 겹침 — 8쪽 555.5px).
    pub(in crate::renderer::typeset) overlay_shape_shortcut_para: Option<usize>,
    /// [#4568] 쪽 하단을 넘는 overlay 표의 **다음 단/쪽으로 넘길 잔여 행** 대기열.
    ///
    /// overlay 표는 흐름 소비가 0 이라 앵커 쪽에서 다음 쪽 항목을 바로 push 할 수 없다
    /// (다음 쪽은 이후 문단이 흐름을 넘길 때 비로소 생긴다). 그래서 앵커 쪽에서
    /// "몇 번째 행부터 잘렸는지"만 적어 두고, 단/쪽이 열릴 때 그 시작에 방출한다.
    /// `(para_index, control_index, start_row, reserve_px)`.
    ///
    /// [#5792] `reserve_px` 는 잔여 행이 새 쪽 흐름에서 예약해야 할 높이다 — 뒤따르는
    /// 흐름이 그 자리를 스스로 만드는 형상(#4514 필러 문단)에서는 0 이다.
    pub(in crate::renderer::typeset) pending_overlay_continuations: Vec<(usize, usize, usize, f64)>,
    /// [#4568] 현재 단에 이어 그릴 overlay 잔여 행 목록. `flush_column` 에서
    /// `ColumnContent::overlay_continuations` 로 옮긴다.
    pub(in crate::renderer::typeset) current_column_overlay_continuations:
        Vec<crate::renderer::pagination::OverlayContinuation>,
    /// [#4568] 현재 단의 overlay 표 앵커 컷 — `(para, ctrl, end_row)`.
    pub(in crate::renderer::typeset) current_column_overlay_cuts: Vec<(usize, usize, usize)>,
    /// [Task #362] 현재 단에서 표 옆에 배치되는 wrap-around paragraphs.
    /// flush_column 에서 ColumnContent 로 전달.
    pub(in crate::renderer::typeset) current_column_wrap_around_paras:
        Vec<crate::renderer::pagination::WrapAroundPara>,
    /// [Task #604 R3] 현재 단의 wrap text 문단 ↔ anchor 메타데이터.
    /// wrap_around state machine 매칭 시 등록. flush_column 에서 ColumnContent 로 전달.
    pub(in crate::renderer::typeset) current_column_wrap_anchors:
        std::collections::HashMap<usize, crate::renderer::pagination::WrapAnchorRef>,
    /// [Task #702] 현재 zone 의 ColumnType (Normal/Distribute/Parallel).
    /// process_multicolumn_break 에서 새 ColumnDef 매칭 시 갱신.
    /// Distribute 다단의 짧은 컬럼 vpos-reset 검출 임계값 완화에 사용.
    pub(in crate::renderer::typeset) current_zone_column_type: ColumnType,
    /// [Task #853] 현재 zone 의 "디자인 spacing"(px) — 1단 ColumnDef 의 `간격` 값.
    /// 한컴은 1단 ColumnDef 의 `간격`(가로 단 간격이지만 1단이라 무의미)을 zone 진입
    /// 세로 간격으로 쓴다(shortcut.hwp 1쪽 헤더 띠 = 10mm). zone 전환 시
    /// (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2) 를 zone_y_offset 에
    /// 더한다. 다단(2+) ColumnDef 의 `간격`은 가로 간격이므로 0 으로 둔다.
    pub(in crate::renderer::typeset) current_zone_design_spacing_px: f64,
    /// [Task #1027 Stage D] 컬럼 단위 vpos 스냅 상태 (렌더러 build_single_column 정합).
    /// current_height 상대공간(col_area_y=0)에서 HeightCursor 를 구동한다.
    pub(in crate::renderer::typeset) vpos_page_base: Option<i32>,
    pub(in crate::renderer::typeset) vpos_lazy_base: Option<i32>,
    /// [#2243] page_base 가 **저장**(비합성) lineseg 에서 왔는지. 저장-앵커 사다리는
    /// 저장 lineseg 없는 문단(생성기 누락 — 한글 fresh 재계산으로 성장 가능)을
    /// 만나면 연속성이 깨지므로 dirty 판단에 쓴다.
    pub(in crate::renderer::typeset) vpos_page_base_stored: bool,
    /// [#2243] 저장-앵커 사다리에 저장 lineseg 없는 문단이 끼어 dirty — 이후
    /// 사다리 역스냅(backward)은 fresh 성장분을 뭉갤 수 있어 금지(전방만 허용).
    pub(in crate::renderer::typeset) vpos_ladder_dirty: bool,
    pub(in crate::renderer::typeset) vpos_prev_layout_para: Option<usize>,
    pub(in crate::renderer::typeset) vpos_prev_partial_table: bool,
    /// 컬럼 시작 시점의 current_height (page_path anchor — 렌더러 col_anchor_y 대응).
    pub(in crate::renderer::typeset) vpos_col_anchor: f64,
    /// HWP3-origin 흐름에서는 vpos 보정에서 spacing_before 사전 차감을 생략한다(#1116).
    pub(in crate::renderer::typeset) skip_spacing_before_prededuct: bool,
    /// [#6753] 직전 문단이 `#2279 ①` 트림으로 흐름에서 뺀 `spacing_before`(px).
    /// lazy 기준 역산이 그 트림분을 기준점에 싣지 않도록 `HeightCursor` 로 넘긴다.
    pub(in crate::renderer::typeset) vpos_prev_trimmed_sb_px: f64,
}
