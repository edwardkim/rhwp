# Task #544 v2 최종 보고서

**제목**: Task #544 / #547 / #548 회귀 정정 — passage 글상자 우측 시프트 + 박스 안 본문 inset 이중 적용 + 셀 내부 inline TAC Shape margin 누락

**브랜치**: `local/task544_v2` (`local/devel` 분기)
**작업 기간**: 2026-05-03 (Stage 1~4 단일 세션)
**이슈 등록**: 작업지시자 결정으로 미등록 (v2 suffix)

## 요약

merge `a7e43f99` (Task #517~528 — Layout Phase 0~2) 가 `paragraph_layout.rs` 충돌 해결 중 Task #544 / #547 / #548 정정을 함께 revert. 작업지시자 보고: 「[1~3]다음 글을 읽고 물음에 답하시오.」 글상자가 오른쪽으로 밀려있음 (회귀).

**Stage 1 의 핵심 finding**: Task #552 (`next_para_starts_visible_border` trailing-ls 보존) 가 #544 (2) 의 `paragraph_border_y_correction_px` Cell 효과를 흡수 → **Phase B 재적용 불필요**. 4 단계 분할이 실질 3 단계 + 보고서로 압축, 코드 단순화.

## 변경 본문

### 1. paragraph_layout.rs (Stage 2 — Phase A)

| 영역 | 변경 |
|------|------|
| line 693~717 → 693~700 | inner_pad 분기 제거 (`has_visible_stroke`/`bs_left_px`/`bs_right_px`/`inner_pad_left/right` 변수 제거). `margin_left = box_margin_left` 단일 룰 (Task #547) |
| line ~2670~2680 | box_x/w 산식 정정. override 케이스 `(ox, ow)`, 일반 케이스 `(col_area.x, col_area.width)`. paragraph border outline = col_area 전체, margin 미적용 (Task #544 (1)) |

### 2. table_layout.rs (Stage 3 — Phase C)

| 영역 | 변경 |
|------|------|
| line 13~28 | `effective_margin_left_line` 헬퍼 함수 추가. `paragraph_layout` 의 line_indent 산식과 동일 (positive: line 0 / hanging: line ≥1 / 0: 모든 line margin 만) |
| line ~1512 | `para_margin_left_px` / `para_indent_px` 추출 (ParaShape 에서) |
| line ~1530 / ~1582 / ~1674 | inline_x 산출 3 분기 (paragraph 시작 / Picture target_line reset / Shape target_line reset) Left/Justify 케이스에 `+ line_margin` 가산 (Task #548) |

### 3. integration_tests.rs (Stage 1 ~ Stage 3)

- `test_544_passage_box_coords_match_pdf_p4` 추가 (Stage 1, ignore → Stage 2 GREEN)
- `test_547_passage_text_inset_match_pdf_p4` 추가 (Stage 1, ignore → Stage 2 GREEN)
- `test_548_cell_inline_shape_first_line_indent_p8` 추가 (Stage 1, ignore → Stage 3 GREEN)

## 코드 영향 (누계)

| 파일 | 변경 LOC |
|------|----------|
| `src/renderer/layout/paragraph_layout.rs` | -22 / +8 (inner_pad 분기 제거 + box_x/w) |
| `src/renderer/layout/table_layout.rs` | +40 / -4 (헬퍼 + 3 분기) |
| `src/renderer/layout/integration_tests.rs` | +209 (3 테스트 + 트리비얼 ignore 해제) |
| `mydocs/plans/task_m100_544_v2.md` | +95 (수행계획서) |
| `mydocs/plans/task_m100_544_v2_impl.md` | +186 (구현계획서) |
| `mydocs/working/task_m100_544_v2_stage1.md` | +85 |
| `mydocs/working/task_m100_544_v2_stage2.md` | +120 |
| `mydocs/working/task_m100_544_v2_stage3.md` | +112 |

## 검증

### 단위 테스트

| 단계 | passed | ignored | 비고 |
|------|--------|---------|------|
| Pre-Stage 1 | 1119 | 2 | baseline |
| Stage 1 (RED 추가) | 1119 | 5 | +3 ignore 추가 |
| Stage 2 (Phase A) | 1121 | 3 | +2 GREEN (test_544/#547) |
| Stage 3 (Phase C) | 1122 | 2 | +1 GREEN (test_548) |
| **최종** | **1122** | **2** | **회귀 0건** |

### 회귀 가드 (issue_*)

| Suite | 결과 |
|-------|------|
| issue_301 | 1 GREEN |
| issue_418 | 1 GREEN |
| issue_501 | 1 GREEN |
| issue_505 | 9 GREEN |
| issue_514 | 3 GREEN |
| issue_516 | 8 GREEN |
| issue_530 | 1 GREEN |
| issue_546 | 1 GREEN |

→ **24 / 24 GREEN, 회귀 0건**.

### Task #552 무회귀 (양립 검증 사후 확정)

`test_552_passage_box_top_gap_p2_4_6` (`--ignored` 실행) → **GREEN 유지**.

→ Stage 1 finding (Task #552 가 #544 (2) 효과 흡수) 사후 검증 완료.

### svg_snapshot

```
test result: ok. 6 passed; 0 failed; 0 ignored
```

- table_text_page_0
- issue_157_page_1
- issue_267_ktx_toc_page
- form_002_page_0
- render_is_deterministic_within_process
- issue_147_aift_page3

### 빌드

- `cargo build --release`: 47.56s, 0 error
- `cargo clippy --lib`: 본 task 신규 결함 **0건**
  - 기존 잔존 결함 2건 (`table_ops.rs:1007`, `object_ops.rs:298`) — orders 메모에 이미 기록, 본 task 무관

## 측정값 (PDF 한컴 2010 정합)

### test_544 — 21_언어_기출 페이지 4 [7~9] passage 박스 좌표

| 좌표 | Stage 1 (수정 전) | 최종 (수정 후) | PDF 기대 |
|------|--------------------|----------------|----------|
| box_top_y | (이미 정합) | 정합 유지 | 233.8 |
| box_left_x | 128.51 | **117.0 ±2** | 117.0 |
| box_width | (미측정) | **425.1 ±2** | 425.1 |

### test_547 — 페이지 4 [7~9] 박스 안 본문 텍스트 inset

| 좌표 | Stage 1 (수정 전) | 최종 (수정 후) | PDF 기대 |
|------|--------------------|----------------|----------|
| 박스 안 본문 min_x | 139.89 | **128.5 ±2** | 128.5 |

### test_548 — 페이지 8 셀 5 line 0 [푸코] inline shape

| 좌표 | Stage 1 (수정 전) | 최종 (수정 후) | PDF 기대 |
|------|--------------------|----------------|----------|
| puko_x | 131.04 | **155.6 ±2** | 155.6 |

→ paragraph_layout 텍스트 위치 (185.83) 와 shape 위치 일관성 회복.

## 검증 게이트 (작업지시자)

- 1차 SVG 시각 판정: 21_언어_기출 페이지 2 [1~3] / [4~6] / 페이지 4 [7~9] / 페이지 8 셀 5 [푸코]
- 2차 rhwp-studio web Canvas 시각 판정

## 잔존 / 후속

1. **Task #544 (2) y 보정**: Task #552 가 흡수했으므로 본 task 에서 미적용. 향후 다른 case 에서 trailing-ls 회귀 발견 시 별도 task 필요 시 재고려.
2. **paragraph margin 영향 광범위 케이스**: 본 task 가 paragraph margin ≠ 0 인 모든 paragraph 의 박스 좌표를 변경. svg_snapshot 6/6 GREEN 으로 fixture 무회귀 확인했으나, 다른 fixture (exam_kor / exam_math / exam_science / exam_eng / exam_social / aift / treatise / hwpspec) 의 광범위 회귀는 시각 판정 게이트에서 의도된 정정 vs 회귀 구분 필요.
3. **Clippy 기존 결함**: `table_ops.rs:1007`, `object_ops.rs:298` (`#[deny(clippy::panicking_unwrap)]`) — orders 메모에 이미 기록. 별도 task 후보.

## Commit 이력

| Stage | Commit | 메시지 |
|-------|--------|--------|
| 1 | `c17efa27` | Task #544 v2 Stage 1: TDD RED 복원 + Task #552 양립 사전 검증 |
| 2 | `05beb208` | Task #544 v2 Stage 2: Phase A 재적용 (paragraph border 좌표/inset 산식 정정) |
| 3 | `9dc40ddb` | Task #544 v2 Stage 3: Phase C 재적용 (#548 셀 inline TAC Shape margin_left + indent) |

## 참조

- 원 commits: `7ba2ecbe` (#544 S2), `b3586723` (#547 S2), `9576f364` (#548 S2)
- 원 TDD commits: `965ea51a` (#544 S1), `9bec6d8a` (#547 S1), `f4bced43` (#548 S1)
- revert 경위: merge `a7e43f99` (Task #517~528 conflict 해결)
- 양립 task: `1934161f` (#552 — `next_para_starts_visible_border` trailing-ls 보존)
- 샘플: `samples/21_언어_기출_편집가능본.hwp`, PDF 한컴 2010

## 머지 절차 (작업지시자 시각 판정 후)

```
git checkout local/devel
git merge local/task544_v2 --no-ff -m \
  "Merge local/task544_v2: Task #544 v2 회귀 정정 (#544/#547/#548 재적용)"
git checkout devel
git merge local/devel --no-ff -m \
  "Merge local/devel: Task #544 v2 회귀 정정"
git push origin devel
```
