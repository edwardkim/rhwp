# 구현 계획서 — Task #965: WMF SetTextAlign vertical bits fix (PR #918 Stage 33-A 포팅)

- 이슈: [#965](https://github.com/edwardkim/rhwp/issues/965)
- 수행 계획서: [task_m100_965.md](task_m100_965.md)
- 브랜치: `local/task965`
- 원본 commit: PR #918 의 `f53235c6` (Stage 33-A)

## 1. 구현 단계 (5 단계)

### Stage 1 — 현재 WMF baseline 측정

**목적**: Fix 적용 전 현재 WMF 처리 상태 baseline 측정 → Stage 4 회귀 비교 기준.

**작업**:
1. WMF 보유 sample 식별:
   - sample16 (page 18, 주전산센터 다이어그램)
   - sample14, sample17, sample18 (PR #918 검증 sample)
   - 시험지 4종
   - 기타 WMF 보유 sample 스캔
2. 각 sample SVG render + PNG 변환
3. text 위치 baseline 저장 (특히 WMF 박스 내 한글 text)

**산출물**: Stage 1 보고서 — WMF sample 목록 + baseline 측정 결과.

### Stage 2 — 구현 계획 V2 (PR #918 Stage 33-A 코드 cherry-pick)

**작업**:
1. PR #918 commit `f53235c6` 의 svg/mod.rs 변경만 추출 (renderer/svg.rs, woff2 제외)
2. 본 task 의 fix 코드 작성 (3 영역, ~50 라인)
3. 위험 평가

**산출물**: `mydocs/plans/task_m100_965_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**작업**:
1. Fix 적용:
   - `set_text_align` vertical bits 분기 (~2191-2206)
   - `ext_text_out` baseline y shift (~813-826)
   - 두번째 `ext_text_out` 분기 (~1541-1548)
2. cargo build --release
3. sample16 page 18 WMF 다이어그램 render:
   - "Windows 서버군", "Unix 서버군" 등 텍스트 박스 내부 정합 확인
4. PNG render → 한컴 viewer 정합

**완료 조건**: sample16 page 18 WMF 박스 내 한글 text 가 박스 내부 정상 표시.

### Stage 4 — 다중 sample 회귀 검증

**작업**:
1. `cargo test --release --lib` 전체 (1288 tests)
2. WMF 보유 sample 시각 회귀 검증 (Stage 1 baseline 과 비교)
3. golden SVG diff 회귀 0

**완료 조건**: cargo test 통과 + WMF baseline 대비 회귀 없음 (개선만).

### Stage 5 — 시각 검증 + 최종 보고서 + PR

**작업**:
1. 한컴 PDF 정합 비교 (sample16 page 18, sample14 등)
2. rhwp-studio UI 시각 확인 (작업지시자)
3. 최종 보고서 + commit + PR

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 진단 (코드 변경 없음) | 위험 없음 |
| 2 | cherry-pick 시 다른 변경 누락 | 핵심 영역만 정확 추출 |
| 3 | svg/mod.rs 변경 → WMF 모든 sample 영향 | Stage 4 baseline 비교 |
| 4 | 회귀 발견 시 fix 재시도 | revert 옵션 + iteration |
| 5 | 한컴 정합 미세 차이 잔존 | 본 task 범위 명시 |

## 3. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**

## 4. 부분 진행 보존 방식

본 task 가 회귀 다수 발생 시:
- Stage 1 (baseline) 까지만 commit (코드 변경 없음, baseline 정보 영구화)
- Stage 2-3 시도 후 회귀 발견 시 revert + Stage 1 보고서 보존
- 새 session 분리 가능
