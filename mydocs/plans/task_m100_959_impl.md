# 구현 계획서 — Task #959: HWP5 시험지 page 1 문9 vertical 처짐 fix

- 이슈: [#959](https://github.com/edwardkim/rhwp/issues/959)
- 수행 계획서: [task_m100_959.md](task_m100_959.md)
- 브랜치: `local/task959`

## 1. 구현 단계 (5 단계)

### Stage 1 — 회귀 source 정밀 추적 (진단, 코드 변경 없음)

**목적**: 우측 단 문8 (y=562) → 문9 (y=1061) 사이 +250px advance 발생 step 정확 식별.

**작업**:
1. `dump-pages -p 0` 으로 시험지 page 1 의 paragraph 구조 확인 (단 0 / 단 1 의 items, 각 paragraph 의 h/vpos)
2. `dump` 으로 문8 ~ 문9 사이 paragraph 들의 controls/line_segs 분석
3. `RHWP_DEBUG_TAC_CURSOR` (PR #958 등록) 활용 — 각 paragraph 별 y_offset 입/출 비교
4. dump 의 vpos vs SVG 실제 y 차이 추적

**산출물**: Stage 1 보고서 — root cause 후보 (A/B/C/D/E) 중 하나로 좁힘 + 정확 line number 식별.

**완료 조건**: 문9 의 y_in 이 1061 ± 5 인 source paragraph + step 명확 식별.

### Stage 2 — 구현 계획 V2 (fix 위치 + 위험 평가)

**목적**: Stage 1 결과 기반 정밀 fix 방향 + 위험 평가.

**작업**:
1. Stage 1 식별 step 의 코드 분석
2. fix 방안 1~3 안 + 각 위험 평가
3. 안전한 fix 안 선정

**산출물**: `mydocs/plans/task_m100_959_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**작업**:
1. Fix 적용
2. cargo build --release
3. 시험지 page 1 SVG → 문9 y 위치 확인 (목표 ~y 800)
4. PNG render → 한컴 PDF (pdf/3-11월_실전_통합_2022.pdf) 정합

**완료 조건**: 시험지 page 1 우측 단 문9 가 한컴 PDF 와 같은 위치.

### Stage 4 — 다중 sample 회귀 검증

**작업**:
1. `cargo test --release --lib` 전체 (1288 tests)
2. 추가 sample SVG render + 시각 확인:
   - 시험지 4종 (3-09월/3-10월/3-11월) page 1
   - sample16 (HWP3)
   - exam_kor / exam_math / exam_eng
   - hwp3-sample10/11/13/14

**완료 조건**: cargo test 통과 + 다른 sample 시각 회귀 0.

### Stage 5 — 시각 검증 + 최종 보고서 + PR

**작업**:
1. 한컴 PDF 정합 비교 (시험지 4종)
2. rhwp-studio UI 시각 확인 (작업지시자)
3. Stage 5 보고서 + 최종 보고서 + orders 갱신
4. commit + PR (작업지시자 승인 필요)

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 진단 단계, 위험 없음 | - |
| 2 | 잘못된 fix 방향 선정 | 작업지시자 승인 |
| 3 | column layout 변경 → 광범위 회귀 | Stage 4 다중 sample 검증 |
| 4 | 회귀 발견 시 Stage 2 재진행 | iteration 명시 |
| 5 | 한컴 정합 미세 차이 잔존 | 본 task 범위 명시 (시험지 page 1 한정) |

## 3. 진행 규칙

- 자동진행 안함 모드
- 각 stage 종료 시 보고서 + 작업지시자 명시 승인
- 회귀 발견 시 즉시 보고 + revert 가능
