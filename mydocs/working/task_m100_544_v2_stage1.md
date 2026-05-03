# Task #544 v2 Stage 1 보고서

## 목적

merge `a7e43f99` (Task #517~528) 후 Task #544 / #547 / #548 정정이 함께 revert. 작업지시자 보고: "[1~3] 다음 글상자가 오른쪽으로 밀려있음".

본 Stage 는:
1. TDD RED 통합 테스트 3 건 복원 (`#[ignore]`)
2. 측정값 확보로 fix 범위 확정
3. **Task #552 와 #544 (2) 양립 사전 검증** — Phase B 필요/불필요 결정

## 작업 요약

### TDD RED 복원

`src/renderer/layout/integration_tests.rs` 끝에 3 테스트 추가 (각 원 commit 의 코드 그대로):

| 테스트 | 원 commit | 위치 |
|--------|----------|------|
| `test_544_passage_box_coords_match_pdf_p4` | `965ea51a` | line ~911~ |
| `test_547_passage_text_inset_match_pdf_p4` | `9bec6d8a` | line ~1003~ |
| `test_548_cell_inline_shape_first_line_indent_p8` | `f4bced43` | line ~1075~ |

`#[ignore = "Task #5xx RED — fix 적용 전 실패 expected"]` 모두 보존.

### baseline 검증

```
cargo test --lib
test result: ok. 1119 passed; 0 failed; 5 ignored
```

기존 1119 GREEN 무회귀. 추가된 3 건은 ignore.

### 사전 측정 (`--ignored`)

각 테스트 단독 실행. 측정값 (수정 전 RED):

| 테스트 | 첫 fail assertion | 측정값 | PDF 기대 | drift |
|--------|------------------|--------|----------|-------|
| test_544 | **box_left_x** | 128.51 | 117.00 | **+11.51 px** |
| test_547 | min_x | 139.89 | 128.50 | **+11.39 px** |
| test_548 | puko_x | 131.04 | 155.60 | **-24.56 px** |

## 핵심 finding — Task #552 양립 검증

### test_544 의 assertion 순서

원 테스트 코드의 assertion 순서:
1. `box_top_y` vs PDF 233.8 (±2 px)
2. `box_left_x` vs PDF 117.0 (±2 px)
3. `box_width` vs PDF 425.1 (±2 px)

측정 결과 첫 fail = **assertion 2 (box_left_x)**.

→ **assertion 1 (box_top_y) 는 통과** (otherwise 첫 fail 로 잡혔어야 함).

### 결론

**Task #552 가 #544 (2) 의 trailing-ls 박스 top y 회귀를 이미 흡수**.

원리:
- Task #544 (2): `paragraph_border_y_correction_px` Cell + layout.rs vpos correction skip 케이스에서 trailing-ls 만큼 보정값 set → paragraph_layout 진입 시 `bg_y_start` 에 가산
- Task #552: `next_para_starts_visible_border` Cell + paragraph_layout `is_full_paragraph_end` 분기에서 다음 paragraph 가 border 시작이면 trailing ls 를 `y` 에 보존 → `bg_y_start = y_start` 에 자동 반영

두 메커니즘은 다른 진입점 (`y` 누적 vs `bg_y_start` 보정) 이지만 결과적으로 동일 케이스 (페이지 시작 paragraph 직후 border-start transition) 를 처리.

→ **Phase B (#544 (2)) 재적용 불필요**. 적용 시 이중 보정 (trailing-ls 두 번) 위험.

## 수행계획서 변경

- ~~Phase B — Task #544 (2) y 보정~~ **skip**
- 4 단계 분할 → **3 단계 + 보고서** 로 압축 가능 (Stage 3 가 Phase C 만 처리)

수정된 stage 구성:
1. **Stage 1** (본 단계, 완료): TDD RED 복원 + 양립 검증
2. **Stage 2**: Phase A 재적용 (#547 + #544 (1)) → test_544 / test_547 GREEN
3. **Stage 3**: Phase C 재적용 (#548 셀 inline TAC Shape) → test_548 GREEN + 광범위 회귀
4. **Stage 4**: 최종 보고서 + orders 갱신

## 잔존 / 위험

1. test_544 의 `box_top_y` assertion 은 Phase A 만 적용 후에도 통과해야 함. Stage 2 후 재측정 필요 (이론적으로 box_left_x / width fix 만 영향 → top_y 무변).
2. Phase A 적용 시 paragraph margin_left ≠ 0 인 모든 paragraph 의 박스 좌표가 변경. 광범위 회귀는 의도된 정정이지만 시각 판정 게이트 필수.

## 산출

- `src/renderer/layout/integration_tests.rs` (+209 LOC, 3 테스트)
- `mydocs/working/task_m100_544_v2_stage1.md` (본 문서)

## 다음 단계

Stage 2 — Phase A 재적용:
- `paragraph_layout.rs:690~716` inner_pad 분기 제거 (#547)
- `paragraph_layout.rs:2687-2691` box_x/w 산식 정정 (#544 (1))
- test_544 / test_547 의 `#[ignore]` 제거 → GREEN 전환
