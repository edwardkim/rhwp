# Stage 1 (v2) — vpos_adjust 입력 분해 + 아키텍처 경로 확정

후보 A(누적기↔렌더러 vpos 공유)의 선행 분석. `vpos_adjust`가 per-para 렌더 높이를 정하는
입력을 **페이지네이션 시점 가용성** 기준으로 분류하고, SSOT 구현의 아키텍처 경로를 확정한다.

## 1. 현 아키텍처 (2-pass, 분리 모델)
| 패스 | 위치 | 높이 모델 | 시점 |
|------|------|----------|------|
| **페이지네이션** | `typeset.rs compute_en_metrics` | saved-delta + caps **근사** | 배치 결정 |
| **렌더** | `layout.rs build_single_column` → `HeightCursor::vpos_adjust` | vpos 기반 **실측** | 페이지 확정 후 |

`vpos_adjust`(height_cursor.rs:146–880, **735줄**)는 **렌더 패스에서만** 호출(layout.rs:3121).
typeset 은 이를 호출하지 않고 별도 근사 → **두 모델 divergence = #1357/#1363 의 본질**.

## 2. vpos_adjust 입력 분류

### (가) IR 파생 — 페이지네이션 시점 가용 ✅
- prev para line_segs: `seg.vertical_pos / line_height / line_spacing`
- curr para `curr_first_vpos`, `vpos_rewind`(curr_first < seg.vpos)
- para 텍스트: `prev_has_text`, `starts_with('문')`, empty 여부
- `para_is_treat_as_char_picture/equation_only`
- para_shape `spacing_before`(curr_sb)
- 섹션 config `endnote_between_notes_hu`

### (나) 단 기하 — 단 시작 후 가용 (조건부) ◐
- `col_area_y / col_area_height / col_anchor_y`
- `vpos_page_base / vpos_lazy_base`(단 배치 중 산출)

### (다) 렌더 러닝 상태 — 페이지네이션 시점 **불가** ❌ (핵심 장애)
- **`y_offset`**: 러닝 렌더 커서 y. **거의 모든 cap 분기의 게이트**
  (`y_offset > col_area_y + col_area_height*0.75/0.85/0.95`, `title_bottom_threshold` 등).
  이게 산출 대상(출력)이자 다음 para 입력 → 예측엔 **누적 시뮬레이션 필수**.
- **`prev_item_content_bottom_y`**: 렌더러가 기록한 직전 항목 실측 콘텐츠 하단
  (layout.rs:3110, `self.last_item_content_bottom`). compact gap 기준.
- `prev_layout_para`, `prev_item_was_partial_table`: 러닝.

### cap 분기(전량 (다) 의존)
`bottom_new_note_gap_cap`, `compact_endnote_stale_note_gap`, `compact_endnote_new_note_jump`,
`compact_endnote_title_bottom_backtrack`, `endnote_title_direct_bottom_fit`(layout.rs:3130) —
**모두 `y_offset` 임계 기반**. 즉 divergence 를 만드는 핵심 보정은 **렌더 위치 의존**이라
IR 만으로 예측 불가.

## 3. 핵심 결론 (아키텍처 경로)
**vpos_adjust 를 "예측"할 수 없다 — 러닝 렌더 y 에 의존하기 때문. 대신 페이지네이션이
렌더 y 진행을 "시뮬레이션"해야 한다.**

→ 후보 A 의 올바른 형태 = **typeset 미주 페이지네이션이 `HeightCursor` 기반 단-레이아웃
시뮬레이션을 실행**해, compute_en_metrics 근사를 **실제 vpos_adjust 누적으로 대체**.
`build_single_column`(layout.rs)의 미주 루프가 이미 `HeightCursor` 로 캡슐화되어 있으므로,
이를 **height-only 시뮬레이션 경로로 추출**해 양쪽(렌더=배치, typeset=예측)이 공유한다.

### chicken-and-egg 해소
페이지네이션은 단 내용을 "결정"하는데 시뮬레이션은 단 내용을 "입력"으로 받음 → para 단위
순차 시뮬레이션으로 해소(para 추가→vpos_adjust y 갱신→`y > 단 하단`이면 break/split→다음 단).
현 compute_en_metrics + split_endnote_to_fit 루프를 시뮬레이션 루프로 치환.

## 4. 구현 함의 (Stage 2 입력)
- `HeightCursor` 에 **height-only 모드**(prev_item_content_bottom_y 미실측 시 LINE_SEG 폴백,
  코드에 이미 부분 존재 — "height-only 경로처럼 값이 없으면")를 정식화.
- typeset 미주 루프가 `HeightCursor::vpos_adjust` 를 호출하도록 배선 → `acc` = 시뮬 y 증분.
- caps(typeset.rs `stale_forward_vpos`/`capped_new_endnote_advance`/`compact_local_rewind`)는
  vpos_adjust 가 동일 보정을 하므로 **중복 제거 가능** → 단계적 제거 + 재튜닝.
- `RHWP_EN_SSOT=A2` 플래그로 시뮬레이션 경로 게이트, v1 하니스/계측 계승.

## 5. 위험 재평가
- height-only 시뮬레이션은 `prev_item_content_bottom_y`(실측) 부재 → 일부 cap 이 LINE_SEG
  폴백으로 동작. 렌더(실측)와 미세 불일치 가능 → 그 잔차가 배치를 흔들면 부분 SSOT 한계.
- 미주 루프를 시뮬레이션으로 치환 시 split/advance_column 상호작용 전면 재검증(전 exam).
- 735줄 vpos_adjust 의 단 컨텍스트(col geometry, page/lazy base) 를 시뮬 시점에 정확 주입해야 함.

## 6. 다음 (Stage 2)
`build_single_column` 미주 루프 → 공유 `simulate_endnote_column` 추출 설계 + height-only
HeightCursor 정식화 + typeset 배선 지점 확정 + A2 플래그.
