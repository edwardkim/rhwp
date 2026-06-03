# 구현 계획서 — Task #957: sample16 page 18 typeset cursor over-advance fix

- 이슈: [#957](https://github.com/edwardkim/rhwp/issues/957)
- 수행 계획서: [task_m100_957.md](task_m100_957.md)
- 브랜치: `local/task957`

## 1. 구현 단계 (5 단계)

### Stage 1 — Cursor over-advance source 식별 (진단, 코드 변경 없음)

**목적**: pi=394 → pi=395 사이의 +430px 추가 누적 발생 정확 위치 식별.

**작업**:
1. typeset.rs / layout.rs 에 `RHWP_DEBUG_TAC_CURSOR` 환경변수 추가 — pi=394 처리 step 별 y_offset/y 변화 추적
2. pi=394 의 controls 처리 path 추적:
   - inline TAC table emission (caption "가.", "나.")
   - picture emission (TAC + TopAndBottom diagram)
   - shape_item emission
   - layout_partial_paragraph emission
3. 각 step 의 y_offset 입/출 값 비교하여 +430px 누적 발생 step 식별

**산출물**: Stage 1 보고서 — root cause 후보 (A/B/C/D) 중 하나로 좁힘 + 정확 line number 식별.

**완료 조건**: pi=395 y_in = 1197.9 의 source step 명확 식별.

### Stage 2 — 구현 계획서 V2 (fix 위치 + 위험 평가)

**목적**: Stage 1 결과 기반 정밀 fix 방향 + 위험 평가.

**작업**:
1. Stage 1 식별 step 의 코드 분석 (기존 동작 + 의도)
2. fix 방안 1~3 안 + 각 위험 평가
3. 안전한 fix 안 선정 (작업지시자 승인 필요)

**산출물**: `mydocs/plans/task_m100_957_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**목적**: Stage 2 의 fix 적용 + sample16 page 18 시각 정합 확인.

**작업**:
1. Fix 적용 (1줄 ~ 10줄 수준 예상)
2. cargo build --release
3. sample16 page 18 SVG render → pi=395 y 위치 확인 (목표 y ≈ 770~800)
4. PNG render → 시각 정합 확인 (한컴 viewer 의 page 16 정합)

**완료 조건**: sample16 page 18 의 "○ 통합모델..." 본문이 같은 페이지에 표시.

### Stage 4 — 다중 sample 회귀 검증

**목적**: 본 fix 가 다른 sample (특히 TAC + TopAndBottom 다이어그램 포함) 회귀 없는지 검증.

**작업**:
1. cargo test --release --lib 전체 (1288 tests, golden SVG diff 포함)
2. 추가 sample 검증:
   - hwp3-sample10/11/13 (다이어그램 포함)
   - 시험지 (3-09월/3-10월/3-11월)
   - exam_kor / exam_math / exam_eng
   - shortcut.hwp (PR #842 영역)
3. 회귀 발견 시 fix revisit

**완료 조건**: cargo test 1288 통과 + 다른 sample 시각 회귀 0.

### Stage 5 — 시각 검증 + 최종 보고서

**작업**:
1. 한컴 PDF 정합 비교 (pdf/hwp3-sample16-hwp5-2022.pdf 외)
2. rhwp-studio UI 시각 확인 (사용자 보고 page)
3. Stage 5 보고서 + 최종 보고서 + orders 갱신
4. commit + PR 준비 (작업지시자 승인 필요)

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 디버그 print 영구화 → 성능 미세 영향 | 환경변수 가드 (RHWP_DEBUG_TAC_CURSOR=1 시만) |
| 2 | 잘못된 fix 방향 선정 → Stage 3-4 회귀 | 작업지시자 승인 + 위험 명시 |
| 3 | typeset.rs / layout.rs 변경 → 다른 sample 회귀 | Stage 4 다중 sample 검증 |
| 4 | 회귀 발견 시 Stage 2 재진행 | iteration 명시 |
| 5 | 한컴 정합 미세 차이 잔존 | 본 task 범위 명시 (sample16 page 18 한정) |

## 3. 일정

본 fix 의 깊은 분석 + 다수 sample 회귀 검증이 필요. 사용자가 "수 일" 진행 의사 확인됨 (#952 옵션 C). 단계별 진행 + 매 단계 승인.

## 4. 진행 규칙

- 자동진행 안함 모드
- 각 stage 종료 시 보고서 + 작업지시자 명시 승인 후 다음 stage 진행
- 회귀 발견 시 즉시 보고 + revert 가능
