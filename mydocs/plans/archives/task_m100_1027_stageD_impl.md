# Stage D 구현계획서 — #1027: 페이지네이터에 HeightCursor 통합 (단단 우선)

- 이슈: #1027 / 브랜치 `local/task1027`
- 설계서: `tech/shared_layout_measurement_engine.md` Stage D
- 선행: Stage A/B(순수 함수 추출), Stage C(HeightCursor + 렌더러 위임, 무동작·커밋 789f408a)

## 1. 근본 원인 (Stage 1·3 확정)

`typeset.rs format_paragraph`: `total_height = sb + Σ(lh+ls) + sa`.
단단 누적 `st.current_height += total_height` → 단락마다 **sb·trailing_ls 누적 drift**
(k-water p8: 43.6px 과측정 → 노트 "추진일정은" 8쪽 거부 → 9쪽 밀림).

렌더러는 단락마다 ≈total_height advance 후 **다음 항목에서 VPOS_CORR 로 vpos 에 스냅**
(`build_single_column` → Stage C `HeightCursor::vpos_adjust`) → drift 제거.

## 2. 핵심 설계 (최소 통합)

**페이지네이터 packing 루프에 렌더러와 동일한 vpos 스냅을 삽입**한다. 누적식
(`current_height += total_height`)은 그대로 두고, 각 항목 fit **직전에** `vpos_adjust`
로 `current_height` 를 vpos-정합 위치로 보정한다. 렌더러 동작과 1:1.

좌표계: HeightCursor 를 **current_height 상대 공간**(col_area_y=0)에서 구동.
- `col_area_y = 0`, `col_anchor_y =` 컬럼 시작 current_height(=`pending_body_wide_top_reserve`).
- `col_area_height =` `available_body_height()` (렌더러 col_area.height 대응).
- `vpos_adjust(current_height, …)` 반환값 = 보정된 current_height.

## 3. 변경 항목

### (a) TypesetState 에 HeightCursor 보유
- 필드 `hcursor: HeightCursor` 추가.
- 컬럼 경계에서 reinit: `reset_for_new_page`(current_height=0), `advance_column_or_new_page`
  (current_height=reserve). page_base/lazy_base/prev_para/prev_partial 초기화.

### (b) page_base 지연 초기화
- 렌더러는 `col_content.items.first()` 의 vpos 를 page_base 로 둠.
- 페이지네이터는 항목을 쌓는 중이므로, **컬럼 첫 FullParagraph/PartialParagraph/Table 배치
  시점**에 `hcursor.vpos_page_base = 그 항목 first seg vpos` 로 설정(prev_para==None 일 때).

### (c) typeset_paragraph fit 직전 vpos 보정
- `let y = st.hcursor.vpos_adjust(st.current_height, para_idx, paragraphs, styles); st.current_height = y;`
- 이후 기존 fit 분기(`current_height + height_for_fit <= available`) 그대로.
- place 후 기존 `current_height += total_height` 유지.

### (d) 항목 후 상태 추적 (렌더러 정합)
- 매 항목 후 `hcursor.prev_layout_para = Some(para_idx)`.
- `hcursor.prev_item_was_partial_table = (직전이 PartialTable)`.
- 표/Shape/PartialTable 배치 후 `page_base=None; lazy_base=None`
  (단, Para-float TopAndBottom 표 예외 — 렌더러 2513 정합).

### (e) typeset_paragraph 시그니처에 `paragraphs`,`styles` 전달
- 호출부(typeset.rs:925) 인자 추가.

## 4. 비범위 (Stage E 로 이연)
- 다단(col_count>1): 단별 vpos-reset·zone 상호작용. Stage E.
- TopAndBottom flow-around post-jump(렌더러 2266~2352): Stage E.
- 표/분할표 자체의 vpos 보정(현재 placement 로직 유지). Stage D 는 **표 전후 paragraph**
  의 base 추적까지만.

## 5. 검증 게이트 (각 단계)
- **노트(추진일정) 8쪽 배치** (한컴 2022 PDF 정합) — Stage D 1차 목표.
- LAYOUT_OVERFLOW ≤ 12 (k-water 현 3 유지/개선).
- k-water 페이지 수·svg_snapshot(공개 5건) 회귀 차단.
- lib 테스트 1316 + Stage C parity 유지. clippy 0.
- 광범위: 다른 단단 표/박스 문서 페이지 수 점검.

## 6. 리스크·완화
- **최고 위험**: 페이지 분할 결정 변경 → 전 문서 페이지네이션 영향.
- 완화: 누적식 불변(증분만 보정 삽입) → 변화량 최소. 단단 한정. 게이트 미달 시 즉시 롤백·보고.
- col_area_height 상대공간 매핑이 footnote/zone 차감과 어긋날 수 있음 → available_body_height
  사용으로 렌더러 col_area.height 와 정합 유지, 클램프 상한 영향 최소.
