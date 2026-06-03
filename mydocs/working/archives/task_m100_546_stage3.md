# Task #546 Stage 3 완료 보고서 — 옵션 A 적용 (`82e41ba` 전체 revert)

## 결과 요약

**옵션 A (전체 revert) 적용 완료**. exam_science.hwp 가 v0.7.9 정합 상태로 복원.

| 항목 | revert 후 |
|------|----------|
| 총 페이지 | **4** (회귀 6 → 4) ✅ |
| p2 단 0 items | **37** (회귀 2 → 37) ✅ |
| p2 단 0 used | 1133.6 px ✅ |
| p2 첫 paragraph | `pi=32 "7.-다음은 학생 가 수행한 탐구 활동이다."` ✅ |
| 변경 LOC | -94 (typeset.rs -36 + layout.rs -58) |

## 옵션 C 시도 결과 (참고 — 0 효과)

Stage 2 의 권장이었던 **옵션 C (페이지/단 경계 인식 추가)** 1차 시도:

1. `advance_column_or_new_page` 에 `wrap_around_pic_bottom_px = 0.0` 등 stale 값 클리어
2. `reset_for_new_page` 에 동일 처리
3. wrap zone 종료 시 `bottom_px <= col_h` 검사

**결과 — 0 효과** (회귀 동일).

진단 로그 분석으로 모든 보정값 (`bottom_px`) 이 `col_h` 이하임을 확인:

| Square wrap | wrap zone 종료 | col_count | cur_col | cur_h | bottom_px | col_h |
|-------------|---------------|-----------|---------|-------|-----------|-------|
| pi=21 (그림 1) | pi=22 | 2 | 1 | 531.69 | 668.09 | 1215.15 |
| pi=37 (그림 2) | pi=38 | 2 | 0 | 628.85 | 752.05 | 1215.15 |
| pi=60 (그림 3) | pi=61 | 2 | 0 | 898.43 | 1052.87 | 1215.15 |

→ column 경계 검사 (`bottom_px <= col_h`) 가 항상 통과 → 옵션 C 가드 trigger 안 됨.

**결함의 새로운 본질**: `wrap_around_pic_bottom_px` 자체는 col_h 이내이지만, `current_height = max(current_height, bottom_px)` 보정이 wrap-around paragraph 들의 누적 height 와 결합하여 후속 paragraph 들을 페이지/단 끝으로 advance. exam_science.hwp 는 wrap-around paragraph 가 그림 옆에 동시 배치되는 케이스라 보정이 부작용.

→ **옵션 C 는 페이지/단 경계 검사로 못 잡는 본질** → 작업지시자 결정으로 옵션 A (전체 revert) 진행.

## 옵션 A revert 적용

### 변경 영역

```diff
 src/renderer/layout.rs  | 58 -------------------------------------------------
 src/renderer/typeset.rs | 36 ------------------------------
 2 files changed, 94 deletions(-)
```

### 제거된 영역

#### `src/renderer/layout.rs` (-58 LOC)

`82e41ba` 가 추가했던 `wrap_pic_bottom_y` 계산 + `wrap_anchor_shape_seen` 가드 + Square wrap 그림 처리 후 첫 일반 paragraph 의 `y_offset` 보정 (모두 Para-relative 분기에서 `wrap_pic_bottom_y = 0.0` 으로 설정되어 현재 영향 0 인 영역).

#### `src/renderer/typeset.rs` (-36 LOC)

- `TypesetState::wrap_around_pic_bottom_px` 필드 (line 144)
- `TypesetState::new` 의 초기화
- wrap zone 종료 시 `current_height = max(current_height, wrap_around_pic_bottom_px)` 보정 (line 506-510)
- non-TAC Picture/Shape Square wrap 분기에서 `wrap_around_pic_bottom_px` 계산 (line 644-679)

## TDD 회귀 테스트 신규 (`tests/issue_546.rs`)

```rust
#[test]
fn issue_546_exam_science_p2_pagination_restored() {
    let bytes = fs::read("samples/exam_science.hwp").unwrap();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
    
    // v0.7.9 정답지: 4 페이지 (회귀 시: 6)
    assert_eq!(doc.page_count(), 4, "exam_science.hwp 는 4 페이지여야 함");
}
```

## 결정적 검증 (Stage 4 사전)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ 1113 passed (PR #538 시점과 동일) |
| `cargo test --test issue_546` (신규) | ✅ 1 passed |
| `cargo test --test issue_505` | ✅ 9/9 passed |
| `cargo test --test issue_530` | ✅ 1 passed |
| `cargo test --test issue_418/501` | ✅ 회귀 0 |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed (table_text 포함) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

## 광범위 fixture sweep (페이지 수 정합)

| fixture | v0.7.9 (main) | revert 후 | 정합 |
|---------|--------------|-----------|------|
| exam_kor.hwp | 20 | 20 | ✅ |
| exam_eng.hwp | 8 | 8 | ✅ |
| exam_math.hwp | 20 | 20 | ✅ |
| exam_science.hwp | **4** | **4** | ✅ (회귀 정정) |
| synam-001.hwp | 35 | 35 | ✅ |
| 복학원서.hwp | 1 | 1 | ✅ |
| aift.hwp | 77 | 77 | ✅ |
| 2010-01-06.hwp | 6 | 6 | ✅ |
| 21_언어_기출_편집가능본.hwp | (PR #538 fixture) | 15 | ✅ |

→ 모든 fixture 가 v0.7.9 와 100% 정합. 옵션 A revert 가 광범위 영향 없이 회귀 정정만 적용.

## Task #460 보완5 의 손실 (작업지시자 검토 사항)

`82e41ba` 의 본 의도 (HWP3 Square wrap 그림 아래 텍스트 y위치 정합) 는 본 revert 로 손실됨. 다음 단계:

1. **Stage 5 시각 판정 시점에 점검** — HWP3 Square wrap fixture (Task #460 의 정합 결과) 의 시각 결함 재발 여부
2. **재발 시 별도 task 분리** — 페이지네이션 안전한 방식 (예: 그림 옆 paragraph 가 동시 wrap-around 안 되는 케이스만 보정 적용) 으로 재시도
3. **재발 안 함** — 옵션 A 가 정합한 정정 (Task #460 보완5 가 처음부터 부작용 큰 정정이었거나, 다른 commit 이 보완5 의 본질을 흡수)

## 다음 단계

Stage 3 보고서 승인 후 **Stage 4** 진행:
- 결정적 검증 재실행 + 통합 테스트
- 광범위 fixture svg byte-identical 점검 (페이지 수 외에도 paragraph y위치)
- WASM 빌드 + studio 동기화

## 산출물

- `src/renderer/layout.rs` (-58 LOC)
- `src/renderer/typeset.rs` (-36 LOC)
- `tests/issue_546.rs` (신규 회귀 테스트 1건)
- 본 보고서 (`mydocs/working/task_m100_546_stage3.md`)

## 메모리 정합

- ✅ `feedback_v076_regression_origin` — 옵션 A 의 결과가 v0.7.9 정합 상태로 복원
- ✅ `feedback_visual_regression_grows` — 광범위 fixture sweep 으로 다른 회귀 없음 검증
- ✅ `feedback_image_renderer_paths_separate` — 본 정정은 typeset 단계 (renderer 분기 영향 없음)
- ✅ `feedback_hancom_compat_specific_over_general` — 옵션 C 의 일반화 정정이 효과 없음 → 옵션 A 의 case-specific revert 로 정합
