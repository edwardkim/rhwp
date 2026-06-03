# Stage 3 종합 결론 — #1027: 페이지네이터↔렌더러 측정 정합

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage 3 — 증상 A(콘텐츠 다음쪽 밀림) 수정 시도 종합

## 1. 확정 사항

- **근본 원인**: 기본 페이지네이터(`typeset.rs TypesetEngine`)는 단락 높이를 **formula `total_height`(= sb + Σ(lh+ls) + sa)** 로 누적 → 단락당 +sb(+sa) drift. 렌더러(layout.rs)는 LINE_SEG **vpos 에 스냅**(VPOS_CORR, 단계당 ≤8px 백워드 클램프 `MAX_BACKWARD_PX`)하여 실제 advance 가 vpos 추종. 두 측정 공간 불일치로 페이지 8 에서 43.6px 과측정 → 노트가 9쪽으로 밀림.
- **수정 surface**: `typeset.rs` `typeset_paragraph` fit(1596)·누적(1606). (engine.rs 아님 — 메모리 `two-pagination-engines`.)

## 2. 시도 매트릭스 (모두 노트↔무회귀 동시 달성 실패)

| 시도 | 노트 | 페이지(185) | LAYOUT_OVERFLOW(12) | 비고 |
|------|------|------------|------|------|
| baseline (total_height) | 9쪽 ❌ | 185 | 12 | — |
| 앵커 full-snap (base=텍스트) | 8쪽 | 168 | 136 | 과밀 |
| 앵커 8px 클램프 | 9쪽 | 186 | 13 | 클램프가 19.6px/단락 drift 못 따라잡음 |
| 앵커 full-snap (base=페이지상단 역산) | 8쪽 | 181 | 69 | vpos<실측 단락 과밀 |
| 누적 lines_total (sb+sa 제거) | **8쪽 ✅** | 183 | 27 | 56~81px FullParagraph overflow 신규 + 통합테스트 1건 실패 |

- lib 테스트 1308 pass, svg_snapshot 5 pass(공개 골든 무회귀) — 단 lines_total 은 sb/sa 가 **실제 렌더되는** 단락에서 과소 계상 → 신규 overflow.

## 3. 결론: 단일 공식/앵커로 불가

formula(total_height/lines_total)와 vpos 의 관계가 **단락마다 다르다**(HWP 가 sb/sa 를 vpos delta 에 인코딩하는 정도가 가변). 따라서:
- total_height: sb 과다 → 노트 밀림.
- lines_total: sb/sa 과소 → 다른 단락 overflow.
- 앵커: vpos 과소 단락에서 과밀.

**진짜 정합은 페이지네이터가 렌더러와 동일한 "실제 advance 누적 + vpos 클램프 보정" 을 수행**해야 한다(공유 layout 측정). 이는 typeset_paragraph 누적을 renderer 의 `paragraph_layout` body advance 와 동일 로직으로 만드는 **공유 측정 엔진 리팩터**(대규모).

## 4. 권고

- (1) **공유 측정 엔진**: renderer `paragraph_layout` 의 vpos-corrected advance 계산을 typeset 누적과 공유 — 근본적, 대규모, 광범위 골든 재판정. 별도 집중 타스크 권장.
- (2) **known limitation 수용**: 노트/SFR-008 의 미세 오분할(저영향, 시각상 다음쪽 상단)을 기록 보류. 공개 골든·테스트는 무회귀(현 baseline 유지).

현 브랜치는 클린(모든 시도 되돌림). 근본 원인·수정 surface·시도 결과를 확정 기록함.
