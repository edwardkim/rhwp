# Task #521 Stage 2: 수정 시도 — 보류 보고서

## 시도한 수정 (모두 revert)

### 시도 1: `typeset.rs:1230` host_line_spacing tac=true 포함

**변경**: `is_tac` 분기 제거하여 tac=true 표도 host paragraph 의 line_spacing 을 `host_spacing.after` 에 포함.

**결과**: **172 샘플 페이지 모두 0 diff** — 변경이 production output 에 영향 없음. `host_spacing.after` 가 별도 cap 메커니즘으로 무효화되거나, pi=104 가 다른 코드 경로를 사용하는 것으로 추정.

### 시도 2: `typeset.rs:1497` typeset_tac_table fmt.total_height 사용

**변경**: 단일 TAC 표 분기에서 `fmt.height_for_fit` 대신 `fmt.total_height` 사용.

**결과**: synam-001 11페이지 변경, exam_eng 0 diff. **target paragraph (pi=104) 영향 없음** — pi=104 의 `composed.lines` 가 비어 있어 `fmt.total_height = 0` 으로 fallback 분기 (`ft.total_height`) 사용되는 것으로 추정.

### 시도 3: 시도 1+2 + `cap` 의 `ls/2` 를 full ls 로

**결과**: synam-001 34페이지 + aift 13페이지 + exam_science 1페이지 변경. **exam_eng 여전히 0 diff**.

### 시도 4: `layout.rs:2683` (layout_partial_table follow-up) host_ls 추가

**결과**: 172 페이지 모두 0 diff. pi=104 가 partial_table 경로 미사용으로 추정.

### 시도 5: `layout.rs:2515` `line_segs.get(control_index)` → `last()` (미시도)

pi=104 의 control_index=2 vs line_segs.len()=1 mismatch 추적. 그러나 다음 위험으로 시도 보류:
- 이 라인이 정상적으로 동작하는 다른 케이스 (text + tac=true 표) 에서 회귀 가능
- 정확한 tac_idx 매핑 필요 (단순 `last()` 는 부정확)

## 본 task 한계 — 보류 권장

본 task 의 본질은 처음 가설보다 깊고 복잡:

1. **Pagination vs Layout 분리**: Pagination 의 `current_height` 는 ✓ 정합 가능, 하지만 Layout 의 `y_offset` 계산은 별도 경로 다수
2. **다중 코드 경로**: pi=104 같은 "table only paragraph (no text + tac=true table)" 가 어느 layout 경로를 타는지 명확하지 않음
3. **회귀 위험 광범위**: synam-001 34페이지 회귀 발생한 변경처럼, 작은 fix 도 다른 케이스 큰 영향
4. **검증 도구 부재**: 변경된 페이지 33+개 모두 PDF 와 시각 비교 필요 — 수동 검증 비용 매우 큼

## 권장 사항

본 task 를 **보류 / Layout 리팩터링 Phase 3/4 흡수**:

- `mydocs/orders/20260502.md` 의 `Layout 리팩터링 (Phase 0~2)` 섹션 참조 — Phase 3 (다중행 인라인 표 baseline 정렬) 는 회귀 위험 큼으로 한컴 환경 검증 가능 시점에 진행 권고됨
- 본 task 본질도 동일 (인라인 표 + 후속 paragraph 정합) — Phase 3 의 일부로 통합 처리 적합
- 별도 task 로 본질 2 (`spacing /2.0` 가설 검증) 도 같은 시점에 함께 진행 권고

## 정리

- 모든 source 변경 revert 완료 (`git checkout src/`)
- Stage 2 보고서는 시도 + 보류 사유 기록
- Task 자체는 Stage 3 (회귀 검증) 로 넘어가지 않고 보류 결정 대기

## 작업지시자 결정 요청

- **A) 보류 / Phase 3 흡수** (권장)
- **B) 시도 5 진행** (line_segs.get(control_index) → tac_idx 매핑) — 위험 중간
- **C) 본질 2 (spacing /2.0) 부터 시도** — 위험 매우 큼
- **D) Task close** — 본 issue 는 known issue 로 보류 표시 후 close
