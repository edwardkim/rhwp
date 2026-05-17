# 구현 계획서 — Task #967: HWP3 hwp3-sample18.hwp 페이지 수 +2 inflate

- 이슈: [#967](https://github.com/edwardkim/rhwp/issues/967)
- 수행 계획서: [task_m100_967.md](task_m100_967.md)
- 브랜치: `local/task967`

## 1. 구현 단계 (5 단계)

### Stage 1 — 정밀 추적 (진단, 코드 변경 없음)

**목적**: pi=27 (page 2) + pi=164 (page 14) 가 별도 page 로 분기되는 정확한 원인 식별.

**작업**:
1. `dump samples/hwp3-sample18.hwp -s 0 -p 27` — pi=27 의 paragraph 속성 (line_segs, vpos, lh, ls, controls) 상세
2. 동일 `dump -p 164` — pi=164 상세 분석
3. pi=26 (page 1 마지막), pi=27 (page 2), pi=28 (page 3 시작) 비교
4. 동일 pi=163, pi=164, pi=165 비교
5. typeset.rs 의 page break trigger 추적:
   - `dump-pages -p 0` (page 1 마지막 paragraph 위치)
   - vpos-reset 가드 (typeset.rs:489-547)
   - next_will_vpos_reset 가드 (typeset.rs:555-577)
   - empty paragraph height 누적 계산
6. RHWP_DEBUG_TAC_CURSOR + page boundary debug instrument 추가 (필요시)

**산출물**: Stage 1 보고서 — root cause 후보 (A/B/C/D) 중 하나로 좁힘 + 정확 코드 위치.

### Stage 2 — 구현 계획 V2 (fix 위치 + 위험 평가)

**작업**:
1. Stage 1 식별 위치의 코드 분석
2. fix 방안 1~3 안 + 각 위험
3. 안전한 fix 안 선정

**산출물**: `mydocs/plans/task_m100_967_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**작업**:
1. Fix 적용
2. cargo build --release
3. sample18 page count 확인 (목표 67)
4. PNG render → 한컴 viewer 정합

**완료 조건**: sample18 페이지 수 67 + 시각 정합.

### Stage 4 — 다중 sample 회귀 검증 (가장 중요)

**작업**:
1. `cargo test --release --lib` 전체 (1288 tests)
2. 다중 sample page count 비교:
   - sample16 (sample 시리즈)
   - sample14, sample10, sample11, sample13
   - exam_kor/math/eng
   - 시험지 4종
3. golden SVG diff 회귀 0

**완료 조건**: cargo test + 다른 sample page count 회귀 0.

### Stage 5 — 시각 검증 + 최종 보고서 + PR

**작업**:
1. 한컴 PDF 정합 비교 (sample18)
2. rhwp-studio UI 시각 확인 (작업지시자)
3. 최종 보고서 + commit + PR

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 진단 (코드 변경 없음) | 위험 없음 |
| 2 | 잘못된 fix 방향 선정 | 작업지시자 승인 |
| 3 | typeset.rs / pagination 변경 → 광범위 회귀 | Stage 4 다중 sample |
| 4 | **회귀 다수 발견 가능성 매우 큼** | revert 옵션 + iteration |
| 5 | 한컴 정합 미세 차이 잔존 | 본 task 범위 명시 |

## 3. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**

## 4. 부분 진행 보존 방식

본 task 가 archive/task936 패턴 재현 시:
- Stage 1 (진단) 까지만 완료 후 commit (코드 변경 없음)
- Stage 2 의 fix 안 결정은 작업지시자 + 새 session 으로
- Stage 3+ 시도 후 회귀 발견 시 revert + Stage 1 보고서 보존
