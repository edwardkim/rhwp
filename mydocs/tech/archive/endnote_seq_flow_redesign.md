---
kind: decision
status: historical
canonical: mydocs/tech/rendering_engine_design.md
last_verified: 2026-07-17
---

# 미주 레이아웃 재설계 — 하이브리드 순차-flow 모델 (#1184 / #1257)

상태: 설계 확정(작업지시자 승인). 구현은 `RHWP_EN_SEQ_FLOW` 플래그 게이트로 점진.

## 1. 현행 모델의 문제 (절대-vpos)

- typeset(`typeset.rs` 미주 방출 ~2050-2700)이 미주 para line_seg vpos 를 `vpos_offset` 누적으로
  절대화하고, render(`height_cursor.rs::vpos_adjust`)가 `(vpos − 단 첫 vpos)`로 y 환산.
- 결함:
  1. 단일 미주 내부 **비단조 vpos**(한컴 2D: 다른풀이/보기 박스) → 단 used 과대(`hwp_used`),
     under-fill, 가짜 advance → 페이지 과/부족.
  2. between-notes 7mm 가 typeset vpos 에 미반영(`pagination_margin`=0) + render line_spacing
     주입 → typeset↔render **desync**. render 분기들(safe_backtrack/applied/forward-suppress/
     column-cap)이 제각기 7mm 누락 → 제목 cram.
  3. vpos 인플레이션(POC) → 컬럼 경계 rebase 실패 → **off-page 오버플로우**(3-11 +1752px).
- ~20개 누적 특례로 문서별 페이지 수만 맞춰온 상태(취약).

## 2. 새 모델 — 하이브리드 순차-flow

**원칙: 미주 사이는 순차 누적(단일 진실원천), 미주 내부만 상대 2D 보존.**

```
# 각 미주 단위 메타
note_min_vpos[n]  = min(seg.vpos)                       # 미주 내부 최상단
note_height[n]    = max(seg.vpos+lh+ls) − note_min_vpos # 가시 세로 span (비단조 흡수)

# 순차 배치 (단 내부)
y_cursor = column_top
for each note n in column:
    note_top = y_cursor
    place note: for each seg → seg_y = note_top + px(seg.vpos − note_min_vpos)   # 2D 보존
    y_cursor = note_top + note_height[n] + between_notes_gap                       # 7mm 명시 갭

# 컬럼/페이지 경계
if y_cursor + next_note_first_line_h > column_bottom:
    advance_column_or_page(); y_cursor = column_top      # rebase → 인플레이션 0
```

- **between_notes_gap**: 미주 사이(7mm 등) = `endnote_between_notes_margin(shape)`. 순차 누적에
  명시 포함 → pagination·render 동일(desync·이중가산 원천 제거).
- **note_height**: min/max span 으로 비단조 vpos 흡수 → under-fill 제거.
- **컬럼 리셋**: 단 advance 시 y_cursor=column_top → 절대 vpos 인플레이션 없음 → off-page 제거.
- **typeset·render 동일 누적**: 한쪽이 height-pass, 다른쪽이 배치하되 **같은 note_top 수식** 사용.

### 경계 케이스
1. **미주가 남은 공간보다 큼**: first-line 만 들어가면 시작, 나머지 다음 단으로 분할
   (현 PartialParagraph 메커니즘 재사용). 미주가 한 단보다 큼 → 단 경계마다 연속 분할.
2. **미주 내부 페이지 분할**: note 내부 상대 좌표 유지하되 split 지점 이후는 다음 단 note_top
   기준 재배치.
3. **구분선/머리 미주**: 본문→미주 전환 시 separator 높이 + 첫 미주 note_top.
4. **다단(col_count>1)**: 단 채우면 다음 단, 마지막 단 채우면 다음 페이지.

## 3. 구현 전략 (플래그 게이트)

- `RHWP_EN_SEQ_FLOW=1` 일 때만 새 경로. 기본은 기존 모델(fallback).
- **게이트(전환 전 필수)**: 전 미주 문서 **페이지 수 패리티**(2022 23/23, 2023 20, 10월 18,
  미주사이20 24, 구분선아래20 23, 3-11 21) + 갭 정합(한컴 PDF) + 오버플로우 0.
- 패리티 달성 후 기본 전환 → 구 특례(safe_backtrack/applied 7mm/column-cap 7mm/#1246 rescue/
  #1256 복원) 정리.

## 4. 단계 (구현)

- Stage 2: 새 모델 골격 — note 메타 추출 + 순차 배치(단단/단순 문서) 골격, 플래그 게이트.
- Stage 3: 다단 + 컬럼/페이지 경계 + 분할.
- Stage 4: between-notes 갭 + 구분선/머리 + 2D 내부 보존 정밀.
- Stage 5: 전 문서 페이지수 패리티 + 갭 정합 + 오버플로우 0 (게이트).
- Stage 6: 기본 전환 + 구 특례 정리 + 테스트/보고.

## 5. 회귀 가드
- 매 Stage 전 문서 페이지 수 비교(플래그 on/off). 패리티 깨지면 그 Stage 에서 중단·보정.
- `cargo test` 전체 + #1189/#1209→#1256/#1246/#1082.
