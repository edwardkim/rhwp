# 최종 결과보고서 — #1027: 세로 측정 정합 (페이지네이터 ↔ 렌더러)

- 이슈: #1027 (M100) / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 검증 권위: `pdf/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서-2022.pdf` (한컴 2022)
- 검증 샘플: `samples/2. 인공지능(AI) … 제안요청서.hwpx` (184p, 비공개), `samples/k-water-rfp.hwp`

## 1. 문제 (증상 A)

한 줄 주석("※ 추진일정은 … 변경할 수 있음", pi=127)이 한컴 2022 PDF 에선 **8쪽** 하단에
들어가는데 rhwp 는 **9쪽**으로 밀림. 페이지네이터(`typeset.rs TypesetEngine`)가 단락마다
`current_height += total_height`(sb + Σ(lh+ls) + sa)로 누적해 렌더러보다 ~43.6px 과측정,
8쪽에서 주석을 거부.

## 2. 근본 원인

페이지네이터와 렌더러(`layout.rs`)가 단락 y-advance 를 **다른 방식**으로 계산:
- 페이지네이터: `total_height` 누적 (단락당 +sb/+trailing_ls drift).
- 렌더러: y_offset 진행 후 LINE_SEG vpos 에 스냅(VPOS_CORR, ≤8px 백워드 클램프).

두 측정 공간 불일치 → 콘텐츠가 한컴과 다른 쪽 배치. 단일 공식/앵커 보정은 단락마다
formula↔vpos 관계가 달라 실패(이전 세션 Stage 3 결론). **공유 측정 엔진** 필요.

## 3. 해결 여정 (커밋)

| 단계 | 내용 | 커밋 |
|------|------|------|
| 설계 | 공유 측정 엔진 설계서 | 11038648 |
| A | VPOS_CORR 클램프 → 순수 함수 `vpos_corrected_end_y()` 추출 (무동작) | cbb01301 |
| B | overlay-shape bypass 가드 → `para_has_overlay_shape()` 추출 (무동작) | 04660e83 |
| C | `HeightCursor` 구조체 추출 + 렌더러 위임 (무동작) + parity 단위테스트 8 | 789f408a |
| D+E1+E2 | 페이지네이터 단단 측정 정합 (실동작) | b163445b |

### Stage D+E1+E2 (핵심 수정, 단단)
- **D**: 항목 fit 직전 `HeightCursor::vpos_adjust` 로 `current_height` 를 vpos 에 스냅
  (누적 drift 제거, 렌더러와 동일 측정).
- **E1**: treat_as_char 인라인 표를 호스트 LINE_SEG(`fmt.total_height`)로 advance
  (기존 effective_height 만 더해 16.9px 과소측정 → 표 이후 overflow).
- **E2**: atomic top-fit 60px 스필에서 위아래(TopAndBottom) 글상자 제외
  (한컴은 본문 항목처럼 다음 페이지로 넘김).

## 4. 최종 결과 (한컴 2022 PDF 정합)

| 항목 | 수정 전 | 수정 후 | 한컴 PDF |
|------|---------|---------|----------|
| 노트 "추진일정은"(pi=127) | 9쪽(오류) | **8쪽** | 8쪽 ✓ |
| 글상자(pi=142) | — | **10쪽** | 10쪽 ✓ |
| AI 184p 페이지 수 | 185 | 185 | — |
| AI 184p LAYOUT_OVERFLOW | 13 | **13** (−para642 19.7 +para429 5.1, 순개선) | — |
| k-water-rfp | 29p / 3 ov | 불변 | — |
| svg_snapshot(공개) | 5 pass / 3 debt | 5 pass / 3 debt | — |
| lib 테스트 | 1308 | **1316** (+8 parity) | — |
| clippy | 0 | 0 | — |

진단으로 확인: **plain 문단은 이미 정합**, 불일치는 표·Shape advance 에 국한, 표만 고치면
Shape 로 옮겨가는 whack-a-mole → 표+Shape 동시 정합으로 깨끗이 해결.

## 5. 잔여·후속 과제

### (a) 다단(multi-column) 측정 정합 — 보류 (Stage E3)
vpos 스냅을 다단(col_count>1)으로 확장 시 우측 단(col=1)의 #412 per-column page/lazy base
처리가 빠져 exam_eng 가 8→10 페이지 회귀(overflow 동일, 분산만 악화). **proper #412
per-column base 로직** 선행 필요. 별도 후속 과제.

### (b) 병합본 골든 부채 재판정 — 별도 과제
svg_snapshot 3건(issue-267/617/677) + issue_598(각주 마커 nav)은 **병합 시 골든=theirs 로
둔 사전 부채**로, 본 타스크(Stage A~F) 변경과 **무관**함을 확인:
- 267(KTX TOC): 페이지번호 10px 가로 정렬 차 — 글자처럼표 0, E1/E2 무관.
- 677(복학원서): 글자처럼표 0 — E1/E2 무관.
- 617(exam_kor): 다단(스냅 미적용) + 718줄 거대 diff = stale 골든 지배적.
- issue_598: Stage C HEAD 에서도 실패(stash 검증).
페이지별 한컴 PDF 시각 대조가 필요한 별개 작업이므로 무작정 `UPDATE_GOLDEN` 금지.
별도 타스크로 재판정 권장.

### (c) #1025 page-larger 단일 셀 분할 — 비범위 (별도)

## 6. 산출물

- 설계: `tech/shared_layout_measurement_engine.md`
- 계획: `plans/task_m100_1027.md`, `_impl.md`, `_stageD_impl.md`, `_stageE_impl.md`
- 단계보고: `working/task_m100_1027_stageA/B/C/D/E.md`
- 신규 모듈: `src/renderer/height_cursor.rs` (HeightCursor + parity 8 테스트)
