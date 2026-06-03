# 구현 계획서 — Task #962: 시험지 page 2 <보기> textbox content scramble

- 이슈: [#962](https://github.com/edwardkim/rhwp/issues/962)
- 수행 계획서: [task_m100_962.md](task_m100_962.md)
- 브랜치: `local/task962`

## 1. 구현 단계 (5 단계)

### Stage 1 — 정밀 추적 (진단, 코드 변경 없음)

**목적**: pi=118 textbox 내부 paragraph rendering 의 정확 결함 위치 식별.

**작업**:
1. `dump-pages -p 1` + `dump -s 0 -p 118` 으로 textbox 구조 + 내부 paragraphs 상세
2. 시각 검증 — rhwp SVG render 의 textbox 내 ㄱㄴㄷ 위치 + 수식 위치 정확 추적
3. 코드 path 추적:
   - `table_layout.rs` / `shape_layout.rs` 의 textbox 내부 paragraph 호출 경로
   - composer.rs 의 textbox 내부 paragraph 처리
   - paragraph_layout.rs 의 cell_ctx 보유 분기
4. 한컴 PDF 와 비교

산출물: 정확한 결함 위치 + 데이터 흐름 + root cause 후보 좁힘.

### Stage 2 — 구현 계획 V2 (fix 위치 + 위험)

산출물: `mydocs/plans/task_m100_962_impl_v2.md`

### Stage 3 — Fix 구현 + 단위 검증

**작업**:
1. Fix 적용
2. cargo build --release
3. 시험지 page 2 SVG render → 보기 textbox 내 ㄱㄴㄷ prefix + 수식 위치 정확 확인
4. PNG render → 한컴 PDF 정합

**완료 조건**: ㄱ. h(1)=3, ㄴ. 함수..., ㄷ. 함수... 정상 표시.

### Stage 4 — 다중 sample 회귀 검증 (가장 중요)

**작업**:
1. `cargo test --release --lib` 전체 (1288 tests)
2. textbox + inline 수식 sample 검증:
   - 시험지 4종 (3-09월/3-10월/3-11월)
   - exam_kor/math/eng (수식 다수)
   - hwp3-sample14 (caption + 수식)
   - 글상자 보유 sample (shortcut 등)
3. golden SVG diff 회귀 0

### Stage 5 — 시각 검증 + 최종 보고서 + PR

- 한컴 PDF 정합 비교
- 최종 보고서 + commit + PR

## 2. 위험 평가 (단계별)

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | 진단 단계 | 위험 없음 |
| 2 | 잘못된 fix 방향 선정 | 작업지시자 승인 |
| 3 | textbox / 글상자 / 수식 변경 → 광범위 회귀 | Stage 4 다중 sample 검증 |
| 4 | **회귀 다수 발견 가능성 매우 큼** | revert 옵션 |
| 5 | 미세 차이 잔존 | 본 task 범위 명시 |

## 3. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
- 본 session 안 완료 불가 시 Stage 1 (진단) 만 보존

## 4. 부분 진행 보존 방식

본 task 가 archive/task936 패턴 재현 시:
- Stage 1 (진단) 까지만 완료 후 보고서 + commit (코드 변경 없음)
- Stage 2 의 fix 안 결정은 새 session 으로
- 시도 + 회귀 발견 시 revert + Stage 1 보고서 보존
