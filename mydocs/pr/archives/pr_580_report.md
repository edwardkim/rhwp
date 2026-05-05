# PR #580 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과

**PR**: [#580 Task #577: 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 정정 (closes #577)](https://github.com/edwardkim/rhwp/pull/580)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR close + Issue #577 close**
**처리일**: 2026-05-05

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`0acd13a6` 단독) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (메인테이너 시각 검증) |
| Devel merge commit | `ee85631` |
| Cherry-pick 충돌 | 0 건 (PR mergeable=CONFLICTING 표시는 PR base 시점 차이) |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #577 | CLOSED (수동 close + 안내 댓글) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |

## 2. 본질 결함 (PR 진단)

### 2.1 결함 가설

`exam_science.hwp` 페이지 1 — 2번 문제 보기 ⑤ (및 ②④) 이미지 하단이 cell-clip 영역을 약 10.81 px 초과하여 잘려 보이던 결함. `text_wrap=TopAndBottom AND vert_rel_to=Para` 인 비-TAC Picture 가 셀에 들어 있을 때 `compute_object_position` 호출에 사용하던 `para_y` 가 `layout_composed_paragraph` 의 advance(line_height ≈ 15.32 px) 를 포함하고 있어 이미지가 anchor 라인 한 줄만큼 아래로 밀려 있던 문제.

### 2.2 결함 메커니즘

```
관측 image_y - cell_y = 19.10 px
  = pad_top(3.78) + line_height(15.32, lh=1150 HU)
정정 후 image_y - cell_y = 3.78 px = pad_top   ← HWP IR 정합
```

→ HWP IR 표준은 anchor 시점이 paragraph 시작 좌표 (`para_y_before_compose`) 이지만 코드는 advance 후 `para_y` 사용 → 1라인 오프셋 발생.

### 2.3 본질 정정

```rust
let anchor_y = if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
              && matches!(pic.common.vert_rel_to, VertRelTo::Para)
{ para_y_before_compose } else { para_y };
```

**핵심 가드 정합성:**
- `text_wrap=TopAndBottom AND vert_rel_to=Para` 양 조건 명시 → 다른 케이스 (Char/Page anchor / Square wrap) 무영향
- `picture_footnote.rs::compute_object_position` 자체는 무변경 (다른 호출처 회귀 방지)
- 케이스별 명시 가드 (`feedback_hancom_compat_specific_over_general` 정합)

### 2.4 정량 측정

- exam_science page 1 보기 ①~⑤: image_y - cell_y = 19.10 px → **3.78 px (= pad_top)** ✓
- LAYOUT_OVERFLOW: exam_science 9.5 → 3.4 px / mel-001.hwp 8건 (3.5~18.8 px) → **0건**

## 3. PR 의 4 commits 분석 (cherry-pick 대상 식별)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `9fe0b312` Stage 1 — 분석·재현·기준선 캡처 | 컨트리뷰터 fork plans/working | 무관 |
| **`0acd13a6` Stage 2 — 본질 정정** | `table_layout.rs` +22/-3 + working stage2 | ⭐ cherry-pick |
| `9d571e5f` Stage 3 — 시각·자동 검증 | 컨트리뷰터 fork working | 무관 |
| `1f164668` Stage 4 — 최종 보고서 + orders | 컨트리뷰터 fork report + orders | 무관 |

→ 본질 1 commit 만 cherry-pick. PR #561/#564/#567/#570/#575 와 동일 패턴.

## 4. cherry-pick 진행

### 4.1 대상 commit (1개, 충돌 0)

```
50a80ab Task #577: Stage 2 — 셀 내부 TopAndBottom Picture anchor_y 도입
```

`Jaeook Ryu <jaeook.ryu@gmail.com>` author 보존.

### 4.2 변경 영역

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | +22 / -3 (조건부 anchor_y 도입 + 비-TAC Picture 분기 정정) |
| `mydocs/working/task_m100_577_stage2.md` | +64 (Stage 2 작업 보고서) |

## 5. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1131 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,571,643 bytes** (1m 33s, PR #575 baseline +39 bytes — table_layout.rs +22/-3 LOC 정합) |

## 6. 광범위 페이지네이션 회귀 sweep

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |
| Export 실패 fixture | 0 |
| 측정 도구 | `./target/release/rhwp export-svg` (60s timeout / fixture) |

→ **164 fixture / 1,614 페이지 / 페이지 수 회귀 0**. 케이스별 명시 가드 (`text_wrap=TopAndBottom AND vert_rel_to=Para`) 의 정합성 정량 입증.

## 7. SVG byte 차이 (PR 본문 영역 + 회귀 검증)

| Fixture | 페이지 수 | byte 차이 | 정정 영역 |
|---|---|---|---|
| **exam_science** | 4 | **2 (page 1, page 4)** | page 1 보기 ①~⑤ 정정 (PR 본문 권위 영역) + page 4 부수 효과 |
| **mel-001** | 21 | 0 | LAYOUT_OVERFLOW 8건→0건 정정이 SVG byte 단위는 무영향 (clip 영역 안으로만 이동) |

→ exam_science page 1 의 정정 영역 정합 + page 4 부수 효과 영역 + mel-001 의 LAYOUT_OVERFLOW 측정 정정 (SVG byte 무변경) 확인.

## 8. 시각 판정 (★ 게이트)

### 8.1 SVG 자료 + WASM 환경

- `output/svg/pr580_before/exam_science/` (devel 기준, 4 페이지)
- `output/svg/pr580_after/exam_science/` (cherry-pick 후, 4 페이지)
- `output/svg/pr580_before/mel-001/` + `output/svg/pr580_after/mel-001/` (회귀 검증, 21 페이지)
- WASM: `pkg/rhwp_bg.wasm` 4,571,643 bytes (다양한 hwp 직접 검증용)

### 8.2 작업지시자 시각 판정 결과

> 시각 검증 통과입니다.

→ ★ **통과**. PR 본문 측정 (image_y - cell_y = 19.10 → 3.78 px = pad_top, HWP IR 정합) + LAYOUT_OVERFLOW 정정 (exam_science 9.5→3.4 / mel-001 8건→0건) 의 시각적 효과 입증.

## 9. PR / Issue close 처리

### 9.1 PR #580 close
- 댓글 등록 (cherry-pick 결과 + 결정적 검증 + 광범위 sweep + 시각 판정 + 본 PR 의 본질 + 컨트리뷰터 협업 인정)
- close 처리

### 9.2 Issue #577 수동 close
- closes #577 키워드는 PR merge 가 아닌 close 로 자동 처리 안 됨 (PR #564/#570/#575 와 동일 패턴)
- 수동 close + 안내 댓글 (PR #580 cherry-pick 처리 완료 + 정량 측정 결과)

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_v076_regression_origin` — 정밀 측정 (image_y - cell_y = 19.10 → 3.78 = pad_top + line_height) 으로 결함 origin 식별
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (text_wrap=TopAndBottom AND vert_rel_to=Para 양 조건 동시 검사)
- ✅ `feedback_rule_not_heuristic` — HWP IR 표준 (anchor 시점 좌표 = paragraph 시작) 직접 사용 (휴리스틱 아닌 규칙)
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Task #577 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 협업 인정
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 활발한 외부 기여의 빠른 회전 (17번째 PR 처리)

## 11. 본 사이클 사후 처리

- [x] PR #580 close (cherry-pick 머지 + push)
- [x] Issue #577 수동 close (안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_580_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_580_review.md` → `mydocs/pr/archives/pr_580_review.md`)
- [ ] 5/5 orders 갱신 (PR #580 항목 추가)
