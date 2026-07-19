# #2320 2단계 완료보고 — 비-0 단 vpos 되감김 분할 정정

- 계획서: `mydocs/plans/task_m100_2320.md`
- 브랜치: `local/task2320`

## 계획 대비 변경점 2건

1. **정정 지점이 typeset**: 계획서가 지목한 `pagination/engine.rs` 는 legacy
   Paginator(`RHWP_USE_PAGINATOR=1` fallback)였고, 주 페이지네이션은
   **TypesetEngine**(typeset.rs)이다 — 계측으로 확인(pi=29 가 Paginator 경로
   미도달). 동일한 `current_column == 0` 게이트가 typeset 에도 있어 그곳을
   정정했고, legacy 쪽도 일관성으로 함께 해제.
2. **near-top 가드 추가**: 비-0 단 확장을 any-decrease 로 열었더니 전 샘플
   스윕에서 `143E433F503322BD33.hwp(x)` 1→2쪽 위양성 발생 — 되감김 목표가
   **페이지 중간(≈40%)**인 어울림 흐름 잔재였다. 목표가 **단 상단 근방
   (본문 높이 15% 이내)**일 때만 경계로 인정하는 전용 감지
   (`detect_near_top_rewind_breaks`)로 좁혔다 (treatise 4926HU≈7% ✓ /
   143E 26644HU≈40% ✗). 단 0 시작 경로의 기존 감지는 불변.

## 수정 내용

| 파일 | 내용 |
|------|------|
| `src/renderer/typeset.rs` | ①`typeset_paragraph` 게이트: 비-0 단 시작 + 비-미주 흐름이면 `detect_near_top_rewind_breaks` 로 경계 감지 ②`detect_near_top_rewind_breaks` 신설 (임계 0.15) ③`typeset_multicolumn_paragraph`: 마지막 단 분할을 `advance_column_or_new_page` 로 새 페이지 진행 (종전엔 flush 후 단/높이 미갱신으로 잔여 조각이 눌러앉음) |
| `src/renderer/pagination/engine.rs` | legacy Paginator 동일 게이트 해제 (일관성) |
| `tests/issue_2320_vpos_rewind_page_break.rs` | 143E 위양성 방지 핀 테스트 추가 (3건으로) |

## 게이트

- issue_2320 테스트 **3/3** (red→green + 단 0 분할 불변 + 143E 위양성 핀)
- `cargo test --tests` release-test 실패 0 / fmt / clippy 0
- **전 샘플(632) 로드 페이지 수 devel 대조: 완전 일치** (의도외 변동 0)
- LAYOUT_OVERFLOW (treatise 전체): devel **5건**(내용 소실 최대 106.8px,
  p2 pi=29/30 + p4 pi=51/52 + pi=74) → **2건**(pi=57 10.3px 신규 미세
  draw 드리프트 + pi=74 5.7px 기존). p4 의 pi=51/52 동종 결함(47.9/73.2px)
  까지 함께 해소
- 한컴 2022 PDF 대조: **p2/p4/p5 단 끝·시작 내용 일치** (p2: "둘째,
  Solicited…so" 에서 단 종료 — 정정 전 잘림·소실 해소. p5: pi=51/57 의
  0-리셋 분할 정합)

## 관찰 (범위 외 기록)

- pi=57 잔여 10.3px 는 계측-배치 y 미세 드리프트(1줄 미만)로, p5 PNG 는
  한컴과 일치 — 시각 영향 없음. 계측/배치 불일치 일반 축은 이슈 본문의
  부수 관찰로 남김.

## 다음 단계

3단계(계획서상 게이트 통합 실행)는 본 단계에서 선행 완료된 항목이 많아
시각 판정 자료 정리로 축소 — 4단계 시각 판정 요청과 병합 제안.
