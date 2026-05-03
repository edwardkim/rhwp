# Task #526 최종 보고서 — exam_science 인라인 표 + 수식 stack 결함 정정

## 요약

`paragraph_layout.rs::layout_inline_table_paragraph` (88-603) 가 인라인 표만 처리하고 동일 문단의 TAC 수식·Picture·Form 에 대한 `set_inline_shape_position` 등록을 누락한 결함을 정정. 영향 범위는 사용자 보고된 pi=61 (9개 수식 stack) 단독이 아닌 **5개 문단(pi=61, 79, 110, 118, 120) / 33개 stack 인스턴스**. enum 기반 `InlineTac { Table | Equation }` 통합 처리로 `layout_composed_paragraph:1845-1899` 의 검증된 인라인 수식 패턴을 이식.

**상태**: 완료 (회귀 0, 시각 정합 작업지시자 승인).

## 1. 본질 (Stage 1 §2-§5)

`layout.rs:2003-2025` 에서 인라인 표 보유 문단을 `layout_inline_table_paragraph` 로 dispatch. 그러나 이 메서드는:

| 영역 | 처리 |
|------|------|
| 인라인 표 (`Table tac=true`) | ✓ `layout_table` 호출 |
| 인라인 수식 (`Equation`) | ✗ **완전 무시** |
| 인라인 Picture/Form | ✗ **완전 무시** |
| `set_inline_shape_position` 호출 | ✗ **0건** (메서드 전체) |

→ shape pass `shape_layout::layout_shape:135` 가 `inline_pos.is_none()` 으로 fallback 경로 (140-152) 진입 → 모든 수식이 `(col_area.x, para_y)` 한 점에 stack.

**대조군 pi=18** (인라인 표 없는 7개 수식 단락) → `layout_paragraph` → `layout_composed_paragraph` → `set_inline_shape_position` 호출 → 정상 인라인 배치. 인라인 표 유무가 유일한 차이.

## 2. 정정 (Stage 2)

A안 채택 — `layout_inline_table_paragraph` 안에 수식 처리 추가. 단일 파일 변경.

| 변경 영역 | 라인 | 변경 |
|-----------|------|------|
| 정의 | 117-149 | `inline_tables: Vec<(usize, &Table)>` → `inline_tac_controls: Vec<(usize, InlineTac)>` (enum: Table | Equation) + 디버그 로그 갱신 (LAYOUT_INLINE_TBL → LAYOUT_INLINE_TAC) |
| 선행 prepend | 162-167 | `inline_tables.len()` → `inline_tac_controls.len()` |
| 폭 계산 | 184-211 | `table_widths` → `tac_widths` (match 분기로 표/수식 폭 통합) |
| 총 폭 | 232 | `seg_widths + table_widths` → `seg_widths + tac_widths` |
| 변수명 | 330 | `table_idx` → `tac_idx` |
| segment-끝 컨트롤 | 537-578 | `if table_idx < inline_tables.len()` → `match kind` 종류별 분기 |
| 후행 컨트롤 | 580-617 | 표 전용 while → 종류별 분기 while |
| 신규 메서드 | 21-72 | `render_inline_equation_node` — `layout_composed_paragraph:1845-1899` 패턴 이식 |

총 변경: 1 파일, +187/-88 라인 (`8e07672`).

다른 파일(`shape_layout.rs`, `composer.rs`, `layout.rs`) 무수정.

## 3. 검증 (Stage 3)

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✓ |
| `cargo test --lib --release` | ✓ 1111 passed / 0 failed |
| `cargo clippy --release --lib -- -D warnings` | ✓ warning 0 |
| `scripts/svg_regression_diff.sh build HEAD~1 HEAD` | ✓ 167/170 byte-identical |

**SVG 회귀**: 7 샘플 170 페이지 중

| 샘플 | 결과 |
|------|------|
| 2010-01-06, aift, exam_eng, exam_kor, exam_math, synam-001 | byte-identical (회귀 0) |
| exam_science (4 페이지) | 002/003/004 변경 = 의도된 정정만 |

**Stack 해소** (BEFORE → AFTER, 좌표별 ≥4 인스턴스):

| 페이지 | 좌표 | BEFORE | AFTER |
|--------|------|--------|-------|
| 002 | (534.8, 1206.91) — pi=61 | 9 | 0 |
| 003 | (534.8, 387.52) — pi=79 | 9 | 0 |
| 004 | (534.8, 422.13) — pi=118 | 8 | 0 |
| 004 | (70.67, 1169.43) — pi=110 | 7 | 0 |

**총 33개 stack 인스턴스 → 0개**. 모든 영향 수식이 distinct (gx, gy) 좌표로 분산. pi=120 (수식 1개) 도 좌표 정정 완료 (stack 클러스터에는 안 잡히나 byte 변경에 포함).

**시각 정합**: 작업지시자 승인 (한컴 PDF 비교).

## 4. 영향 범위 확정

| pi | 페이지 | 컨트롤 구성 | 표 위치 | 수식 수 | 결과 |
|----|--------|-------------|---------|---------|------|
| 61 | 2 (우측 단) | 표(0) + 수식(1..9) | 선두 | 9 | ★ 사용자 보고 |
| 79 | 3 | 수식(0) + 표(1) + 수식(2) + 표(3) + 수식(4..10) | 중간 (2건) | 9 | 추가 발견 |
| 110 | 4 | 수식(0) + 표(1) + 수식(2..7) | 중간 | 7 | 추가 발견 |
| 118 | 4 | 수식(0..6) + 표(7) + 수식(8) | 후미 | 8 | 추가 발견 |
| 120 | 4 | 수식(0) + 표(1) | 후미 | 1 | 추가 발견 |

사용자는 가장 시각적으로 두드러진 pi=61 만 보고했으나, 동일 결함이 5개 문단에 잠복. 본 정정으로 모두 동시 해소.

## 5. 위험·실측

| 위험 (Stage 2 §4) | 결과 |
|-------------------|------|
| 수식 baseline 미세 차이 | `layout_composed_paragraph:1862-1867` 동일 식 적용. 회귀 0 확인. |
| `wrapped_below_table` 분기 baseline 혼동 | `cur_baseline` 분기로 명시. 시각 정합 OK. |
| 수식 폭이 텍스트 폭에 포함되어 reflow | `total_width` 통합 흡수. start_x 정합. |
| segments-control 갯수 mismatch | composer.rs::find_control_text_positions 동일 필터 사용. 영향 5문단 모두 1:1 정렬 확인. |
| 다른 샘플 회귀 | 6 샘플 byte-identical, 회귀 0. |

## 6. 메모

- **확장 여지**: `InlineTac` enum 은 향후 Picture/Form 인라인 처리 추가 시 동일 패턴으로 확장 가능 (현 샘플에 케이스 없음).
- **본질**: `layout_inline_table_paragraph` 는 Task #517 (#467/#491/#496 영역) 시점부터 표 전용 가정으로 작성된 잔재 가능성. 인라인 표가 단독 등장하는 경우만 테스트되어 표 + 수식 혼재가 미발견. 본 task 가 그 가정을 보완.
- **회귀 검증 인프라**: `scripts/svg_regression_diff.sh` (#517 Phase 1) 가 본 task 의 33개 stack 해소 + 회귀 0 확인에 결정적 역할.

## 7. 커밋 이력 (local/task526)

| Commit | 단계 | 내용 |
|--------|------|------|
| `7261f9d` | 사전 | 수행 계획서 + 이슈 등록 |
| `689fec8` | Stage 1 | 진단 — layout_inline_table_paragraph TAC 수식 등록 누락 확정 |
| `6b4c949` | 사전 | 구현 계획서 (A안) |
| `8e07672` | Stage 2 | layout_inline_table_paragraph 인라인 수식 처리 추가 |
| `f764466` | Stage 3 | 회귀 검증 — 33개 stack → 0, 6 샘플 byte-identical |

다음 commit: 본 보고서 + orders 갱신.

## 8. 종료 조건 충족

- [x] 본질 식별 (Stage 1)
- [x] 구현 계획서 승인 (A안)
- [x] 코드 변경 + 빌드 + 단위 테스트 + Clippy 통과 (Stage 2)
- [x] 회귀 검증 통과 (167/170 byte-identical, 의도된 정정만 3페이지) (Stage 3)
- [x] 시각 정합 작업지시자 승인
- [ ] merge (local/task526 → local/devel → devel + push)
- [ ] gh issue close 526
