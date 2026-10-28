# Task M100 #1488 Stage 2 — cut 분할 패킹 보정 (여분 페이지 제거)

- 브랜치: `local/task_m100_1488`
- 작성일: 2026-06-25
- 단계: Stage 2 (구현)

## 변경 요약

`src/renderer/layout/table_layout.rs` `cell_units` — 셀 내부 **비가시(빈 텍스트) 오버레이
스페이서 문단이 만든 vpos 리셋을 `hard_break_before`(강제 페이지 분할)에서 제외**.

- `para_has_visible_text` 게이트 추가 (가시 문자 = `c > U+001F && c != U+FFFC`).
- 텍스트줄 유닛: `line_reset_before(li) && para_has_visible_text`
- 빈/원자 유닛: `reset_before && (has_table_in_para || para_has_visible_text)`
- 중첩표 유닛(4220행)·가시 텍스트 문단 사이 리셋(Task #993 의도)은 보존.

### 근거

`advance_row_cut`은 `hard_break_before` 유닛에서 가용 예산과 무관하게 컷을 종료한다.
문제 셀(sec1 pi=28)의 빈 오버레이 문단 `p[6..15]`는 본문 텍스트 위에 겹쳐 놓인 동일/역방향
vpos 줄을 가져, 리셋마다 fragment(=거의 빈 페이지)를 1장씩 양산했다. 빈(비가시) 문단의
리셋은 페이지 분할점이 아니므로 게이트로 제외한다. 빈 오버레이 문단 `internal_reset`은
#1488 의 `p[6..15]`와 **구조적으로 동일**하여 둘을 가를 신호가 없으므로, 가시성(텍스트 유무)을
유일한 판별 신호로 채택했다.

## 검증

| 항목 | before | after |
|------|--------|-------|
| 페이지 수 | 22 | **18** (정답지 PDF 18 일치) |
| sec1 pi=28 분할 fragment 소비 | 85/33/62/62/62px (빈 페이지 5장) | 186/945/261px (정상 패킹) |
| 가득 빈 used=0.0px 페이지(시각) | 다수 | 0 (페이지 4·16 표시 아티팩트는 PartialTable vpos 렌더, 실제 내용 있음) |
| 페이지 2 본문·도식 겹침 | 있음 | 해소 |

`LAYOUT_OVERFLOW`: sec1 p14 para28(기존 13.5px) 해소. 그 외 기존 overflow(sec0 p2/p3/p6,
sec1 p1/p2)는 Stage 3 대상(ROOT B). 신규 overflow 미발생(pi=28 분할 위치 이동분만).

## 단위 테스트

- `test_advance_row_cut_vpos_reset_hard_break`: 가시 문단(`visible_text_para`)으로 갱신 —
  가시 리셋 하드 브레이크 보존 검증 유지.
- `test_advance_row_cut_rowbreak_rewinds_internal_hard_break_orphan`: 가시 문단으로 갱신 —
  rewind-orphan 로직 검증 유지.
- `test_advance_row_cut_empty_overlay_reset_no_hard_break`: **신규** — 빈 오버레이 리셋이
  하드 브레이크가 아님을 검증(#1488 회귀 가드).
- `cargo test --lib row_cut`: 10/10 통과.
