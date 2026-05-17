# 구현 계획서 — Task #960: 시험지 page 2 문14 multi-line equation off-by-one

- 이슈: [#960](https://github.com/edwardkim/rhwp/issues/960)
- 수행 계획서: [task_m100_960.md](task_m100_960.md)
- 브랜치: `local/task960`

## 1. 구현 단계 (5 단계)

### Stage 1 — 정밀 추적 (진단, 코드 변경 없음)

**목적**: cases 와 h(x)=lim 수식이 잘못된 line_seg 에 매핑되는 정확한 step 식별.

**작업**:
1. `RHWP_DEBUG_TAC_CURSOR` instrument 추가 (Task #957 영구화 코드 cherry-pick — upstream/devel 에 미반영)
2. **`composer.rs` 의 paragraph 분할 + control 매핑 분석** — text 의 FFFC index 를 어느 line 에 할당하는지
3. **`paragraph_layout.rs` 의 inline TAC 처리 (line 1969~) 분석** — `run_tacs` 가 어떻게 구성되는지
4. multi-line equation height 가 다음 line 의 y 계산을 어떻게 영향 주는지 추적

산출물: 정확한 결함 위치 + line number + 데이터 흐름 documentation.

### Stage 2 — 구현 계획 V2 (fix 위치 + 위험 평가)

**목적**: Stage 1 결과 기반 정밀 fix 안 + 위험.

산출물: `mydocs/plans/task_m100_960_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**작업**:
1. Fix 적용
2. cargo build --release
3. 시험지 page 2 SVG render → 4 수식 (f, g, cases, h=lim) y 위치 확인
4. PNG render → 한컴 PDF 정합

**완료 조건**: cases 가 ls[1] (~347), h(x)=lim 가 ls[2] (~379) 정확 위치.

### Stage 4 — 다중 sample 회귀 검증 (가장 중요)

**작업**:
1. `cargo test --release --lib` 전체 (1288 tests)
2. multi-line equation 보유 sample 시각 확인 (특히):
   - exam_kor / exam_math / exam_eng (모든 페이지)
   - 시험지 4종 (3-09월/3-10월/3-11월)
   - hwp3-sample10~14 (수식 보유 가능성)
3. golden SVG diff 회귀 0

**완료 조건**: cargo test + 시각 회귀 0.

### Stage 5 — 시각 검증 + 최종 보고서 + PR

**작업**:
1. 한컴 PDF 정합 비교
2. rhwp-studio UI 시각 확인 (작업지시자)
3. 최종 보고서 + commit + PR

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 진단 단계 | 위험 없음 |
| 2 | 잘못된 fix 방향 선정 | 작업지시자 승인 |
| 3 | composer/paragraph_layout 변경 → 광범위 회귀 | Stage 4 다중 sample 검증 |
| 4 | **회귀 다수 발견 가능성 매우 큼** | revert 옵션 + iteration |
| 5 | 한컴 정합 미세 차이 잔존 | 본 task 범위 명시 |

## 3. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
- 본 session 안 완료 불가 시 부분 진행 보존 + 새 session 분리

## 4. 부분 진행 보존 방식

본 task 가 archive/task936 패턴 재현 시:
- Stage 1 (진단) 까지만 완료 후 보고서 + commit (코드 변경 없음, 디버그 instrument 만)
- Stage 2 의 fix 안 결정은 작업지시자 + 새 session 으로
- Stage 3+ 시도 후 회귀 발견 시 revert + Stage 1 보고서 + 부분 진행 보존
