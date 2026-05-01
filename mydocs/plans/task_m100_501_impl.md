# Task #501 구현 계획서

## 정정 방향 — 옵션 C (TAC 표 raw_table_height 에서 IR cell.h 권위 기준)

### 본질

TAC 표 (`treat_as_char=true`) 는 한컴이 표 높이를 IR `common.height` 로 고정 + 셀 높이를 IR `cell.height` 로 고정. rhwp 의 `measure_table_impl` 이 `required_height = content_height + pad` 폴백을 사용하면서 `corrected_line_height` 보정 또는 rs deficit 누적으로 row_heights 를 IR 값보다 크게 만들어 → 비례 축소 → 행 높이 잘못 축소.

### 정정 핵심

TAC 표 (`treat_as_char=true`) 영역에서 **IR cell.height (HWP 저장값) 를 row_heights 의 권위 기준** 으로 사용. 즉:
- `required_height` (content + pad) 가 IR cell.height 보다 크면 **IR 우선** (클리핑)
- rs>1 deficit 추가 차단 (TAC 표 한정)

비-TAC 표는 기존 정책 유지 (IR cell.height vs content_height max).

## 구현 단계 (3 stages)

### Stage 1 — Red 테스트 추가 + 정밀 측정

**산출물**: `mydocs/working/task_m100_501_stage1_impl.md`

1. mel-001 p2 pi=22 의 raw_table_height 측정 (디버그 로그 또는 단위 테스트)
2. 단위 테스트 추가 (Red):
   - mel-001 의 측정된 row_heights 가 IR cell.height 정합 (각 행 17.07~26.4px, 비례 축소 미적용)
3. 현재 회귀 재현 확인

**파일**:
- 신규: `src/renderer/height_measurer.rs::tests` (Red 테스트)

### Stage 2 — TAC 표 IR 우선 정책 구현

**산출물**: `mydocs/working/task_m100_501_stage2_impl.md`

1. `height_measurer.rs::measure_table_impl` 의 2단계 (content_height 누적) 에서 TAC 표 한정 IR 우선:
   ```rust
   if table.common.treat_as_char {
       // TAC 표: IR cell.height 권위 기준, content_height 가 더 크면 클리핑
       // required_height = content + pad 적용 차단
   } else {
       // 비-TAC 표: 기존 정책 유지
       if required_height > row_heights[r] {
           row_heights[r] = required_height;
       }
   }
   ```
2. 2-b단계 (rs>1 deficit) 도 TAC 표 한정 차단:
   ```rust
   if table.common.treat_as_char {
       // TAC 표: rs>1 deficit 추가 차단 (IR cell.height 신뢰)
   } else {
       // 비-TAC 표: 기존 deficit 정책 유지
   }
   ```
3. `resolve_row_heights` (table_layout.rs) 도 동일 정책 적용

**파일**:
- 변경: `src/renderer/height_measurer.rs` (measure_table_impl)
- 변경: `src/renderer/layout/table_layout.rs` (resolve_row_heights)

### Stage 3 — 검증 + 광범위 회귀 점검

**산출물**: `mydocs/working/task_m100_501_stage3_impl.md`

1. cargo test --lib (Stage 1 의 Red 테스트 → Green)
2. cargo test --test svg_snapshot 6/6
3. cargo test --test issue_418 1/1
4. cargo clippy --lib -- -D warnings 0건
5. WASM 빌드
6. 광범위 회귀 점검:
   - mel-001 p2 시각 (정정 후)
   - synam-001, k-water-rfp, aift (TAC 표 다수 영역)
   - exam_kor, exam_science, exam_social, exam_eng, 21_언어_기출 (TAC 표 시험지)
   - hwpspec (비-TAC 영역 회귀 0)
7. byte 비교: 광범위 샘플의 SVG 출력 비교, 정정 영역 외 회귀 0

## 검증 게이트

| 검증 | 기준 |
|------|------|
| cargo test --lib | 회귀 0 (1086 baseline + 신규 1) |
| cargo test --test svg_snapshot | 6/6 |
| cargo test --test issue_418 | 1/1 |
| cargo clippy --lib -- -D warnings | 0건 |
| WASM 빌드 | 정합 |
| 광범위 byte 비교 | 정정 영역 외 회귀 0 |
| **작업지시자 시각 검증** | mel-001 p2 + 다른 샘플 회귀 0 (Stage 5 게이트) |

## 위험 영역 + 완화

| 위험 | 완화 |
|------|------|
| TAC 표 광범위 영역 영향 (시험지 + 박스 + 인라인 표) | 비-TAC 영역 정책 보존 + Stage 3 광범위 byte 비교 |
| corrected_line_height 보정 (#185) 무력화 가능성 | TAC 표는 IR 우선, 비-TAC 표는 기존 보정 그대로 |
| rs>1 deficit 차단으로 셀 콘텐츠 클리핑 발생 | mel-001 패턴은 IR 정합으로 클리핑 없음 (확인) — 다른 샘플도 점검 |
| 1x1 래퍼 표 (트러블슈팅 4번) 영역 영향 | layout_table 진입부 래퍼 감지 우선 적용됨 — 본 변경 영향 없음 |

## 메모리 룰 적용

- `feedback_hancom_compat_specific_over_general` — TAC 표는 한컴 정합 룰 (IR cell.h 권위 기준), 일반화 회피
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 검증을 게이트로
- `feedback_release_manual_required` — 본 정정은 릴리즈 영역 (페이지 분할 영향) 이 아니므로 매뉴얼 정독 미필요

## 다음 단계

작업지시자 승인 후 Stage 1 (Red 테스트 추가 + 정밀 측정) 진행.
