# PR #629 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과

**PR**: [#629 Task #628: nested cell inline_shape_positions 키 충돌 정정 — 글상자 안 이미지 미렌더링 (closes #628)](https://github.com/edwardkim/rhwp/pull/629)
**작성자**: @planet6897 (Jaeuk Ryu) / Jaeook Ryu (commit author)
**관련**: closes #628
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR/Issue close**
**처리일**: 2026-05-06

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`04ce0d22` 단독, src 7 파일 +62/-28) + devel merge + push + PR/Issue close |
| 시각 판정 | ★ **통과** (작업지시자 시각 판정) |
| Devel merge commit | `08a5104` |
| Cherry-pick commit (local/devel) | `c353cfc` |
| Cherry-pick 충돌 | 본질 src 7 파일 0건 (orders add/add 충돌은 ours 보존) |
| Author 보존 | ✅ Jaeook Ryu (jaeook.ryu@gmail.com) 보존 |
| PR #629 close | ✅ 한글 댓글 등록 + close |
| Issue #628 close | ✅ 수동 close (closes #628 키워드는 cherry-pick merge 로 자동 처리 안 됨, 안내 댓글 등록) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,684 페이지 / 회귀 0 |

## 2. 본질 결함 (PR 진단)

`PageRenderTree.inline_shape_positions` 의 키 `(section, para, control)` 에서 `para` 가 두 가지 의미 혼용:

1. **paragraph_layout 호출 시** → 섹션 단위 paragraph 인덱스 (예: pi=119, pi=127)
2. **layout_table → 셀 paragraph 호출 시** → 셀 내부 paragraph 인덱스 (`cp_idx`, 보통 0)

서로 다른 셀 컨텍스트가 동일 키 `(0, 0, 1)` 등을 공유 → 다른 paragraph 의 double-nested 셀 처리가 키를 미리 점유 → 20번 외부 1×1 표 처리 시 `already_rendered_inline=true` 오판 → `table_layout.rs:1900` 분기에서 내부 2×3 표의 `layout_table` 재귀 호출 스킵 → 그 안의 그림 미렌더.

**비대칭 발현**:

| 문항 | 표 nesting | 결과 |
|---|---|---|
| 19번 (pi=119) | 2×3 표 → 셀 → 그림 (단일 nesting) | ✅ 정상 |
| 20번 (pi=127) | **1×1 → 셀 → 2×3 → 셀 → 그림 (이중 nesting)** | ❌ 누락 |

## 3. 본질 정정 (`InlineShapeKey` 신규 타입)

```rust
// 신규
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);
//                          section, para, control, cell_path
inline_shape_positions: HashMap<InlineShapeKey, (f64, f64)>
```

`cell_path` = 외→내 nesting 순서의 `(control_index, cell_index, cell_para_index)` 튜플 목록. **섹션 단위는 빈 Vec, 셀 단위는 `CellContext.path` 전체**.

`set/get_inline_shape_position` 시그니처에 `cell_ctx: Option<&CellContext>` 추가. 호출처 13곳 일괄 패치 (셀 단위 9 + 섹션 단위 4):
- 셀 단위: `paragraph_layout.rs` 6곳 (`cell_ctx.as_ref()` 전달) + `table_layout.rs` 2곳 + `table_partial.rs` 1곳
- 섹션 단위: `layout.rs` 4곳 (`None` 전달) + `cursor_rect.rs` 1곳 (`None` 전달) + `shape_layout.rs` 2곳 (`None` 전달)

**`cursor_rect.rs:532` hit-test 가드 추가**:
```rust
let (si, pi, ci, ref cell_path) = *key;
// 셀 내부 inline shape 은 cursor hit-test 에서 별도 처리 — 섹션 단위만 검사
if !cell_path.is_empty() { continue; }
```

→ 셀 내부 inline shape 의 cursor hit-test 별도 처리 영역 차단 (cursor 동작 회귀 0 보존).

## 4. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ issue_546 1 + issue_554 12 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,590,307 bytes** (1m 27s, PR #578 baseline 4,583,156 +7,151 — render_tree.rs +43 LOC + InlineShapeKey Vec allocation 정합) |

## 5. 정량 측정 (PR 본문 100% 재현)

### 5.1 exam_science page 별 image 수

| 페이지 | BEFORE images | AFTER images | byte 차이 | 평가 |
|---|---|---|---|---|
| page 1 | 12 | 12 | identical | ✅ 회귀 0 |
| page 2 | 2 | 2 | identical | ✅ 회귀 0 |
| page 3 | 2 | 2 | identical | ✅ 회귀 0 |
| **page 4** | **3** | **4** | **differ (461,151 → 532,816, +71,665 bytes)** | ✅ **20번 실린더 이미지 +1** (정정 영역) |

### 5.2 page 4 추가 image 위치

```
x=568.00  y=783.92  width=376.65  height=101.81
```

→ **PR 본문 명세 `99.7×26.9mm IR 정확 매칭` 100% 일치** (실린더 이미지 `bin_id=2` 정상 위치).

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,684** |
| 총 페이지 (AFTER) | **1,684** |
| **fixture 별 페이지 수 차이** | **0** |

→ 키 namespace 분리가 페이지네이션에 영향 없음.

## 7. 시각 판정 (★ 게이트)

작업지시자 시각 검증 결과:
> 메인테이너 시각 판정 통과입니다.

→ ★ **통과**.

권위 영역: **page 4 — 20번 문항** 글상자 안 (외부 1×1 → 내부 2×3 → 셀 → 그림 이중 nesting) 실린더 이미지 (`bin_id=2`, 99.7×26.9mm) 정상 위치 (`x=568 y=783.92 w=376.65 h=101.81`) 출력.

## 8. PR / Issue close 처리

### 8.1 PR #629 close
- 댓글 등록 (한글, cherry-pick 결과 + 결정적 검증 + 광범위 sweep + PR 본문 100% 재현 + 시각 판정 ★ 통과 + 컨트리뷰터 협업 인정)
- close 처리

### 8.2 Issue #628
- closes #628 키워드는 cherry-pick merge 로 자동 처리 안 됨 (PR #570 등 동일 패턴) → 수동 close + 안내 댓글

## 9. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (164 fixture / 1,684 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 키 namespace 분리만 수행, 값/계산 로직 무변경 (case-specific 가드)
- ✅ `feedback_rule_not_heuristic` — `CellContext.path` (HWP IR 표준 nesting 경로) 직접 사용, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_pdf_not_authoritative` — PR 본문 IR 명세 (99.7×26.9mm) 검증, PDF 미사용
- ✅ `feedback_per_task_pr_branch` — Task #628 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 한글 답변 (PR #599 학습 적용)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 5/6 처리분 점검 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/6 v0.7.10 후 두 번째 PR 처리분, PR #578 후속) 영역 정합
- ✅ `feedback_image_renderer_paths_separate` — 본 PR 의 키 namespace 분리는 SVG/Canvas 양쪽 동일 영향 (renderer 별 분기 없음, 데이터 구조 영역)

## 10. 본 PR 의 본질 — v0.7.10 후 두 번째 처리 PR

본 PR 의 처리 본질에서 가장 우수한 점:

1. **근본 원인 정확 식별** — `inline_shape_positions` 키 namespace 충돌의 origin (섹션 단위 `para` vs 셀 내부 `cp_idx`) 직관적이지 않은 영역을 정확히 진단 + 19번 (단일 nesting) 정상 / 20번 (이중 nesting) 만 발현하던 비대칭의 본질 정합
2. **HWP IR 표준 직접 사용** — `CellContext.path` (외→내 nesting 경로) 활용, 휴리스틱 미도입
3. **회귀 위험 영역 좁힘** — 키 namespace 분리만 수행, 값/계산 로직 무변경. 섹션 단위 호출 (`None` 전달) 은 기존 동작 유지
4. **호출처 13곳 일괄 패치 + cursor 가드** — `cursor_rect.rs` hit-test 루프에 `cell_path.is_empty()` 가드로 cursor 동작 회귀 0 보존
5. **v0.7.10 후 두 번째 처리** — 본 사이클 (5/6) PR #578 후속 영역 정합 (한글 답변 / commit 단위 cherry-pick / 광범위 sweep)

## 11. 본 사이클 사후 처리

- [x] PR #629 close (cherry-pick 머지 + push + 한글 댓글)
- [x] Issue #628 close (수동 close + 안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_629_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_629_review.md` → `mydocs/pr/archives/pr_629_review.md`)
- [ ] 5/6 orders 갱신 (PR #629 항목 추가)
