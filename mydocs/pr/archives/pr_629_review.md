# PR #629 검토 보고서

**PR**: [#629 Task #628: nested cell inline_shape_positions 키 충돌 정정 — 글상자 안 이미지 미렌더링 (closes #628)](https://github.com/edwardkim/rhwp/pull/629)
**작성자**: @planet6897 (Jaeuk Ryu)
**상태**: OPEN, **mergeable=UNKNOWN** (PR base 8 commits 뒤 — 본 사이클 #578 후속 commits 영역, 본질 충돌 0)
**관련**: closes #628
**처리 결정**: ⏳ **옵션 A 진행 중 — 시각 판정 게이트 대기** (작업지시자 승인 후 cherry-pick 적용 + 결정적 재검증 통과)
**검토 시작일**: 2026-05-06

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `PageRenderTree.inline_shape_positions` 의 키 `(section, para, control)` 에서 `para` 가 두 가지 의미 (섹션 단위 paragraph 인덱스 / 셀 내부 paragraph 인덱스) 로 혼용되어 nested cell 영역에서 키 충돌이 발생하는가? 본 환경에서 결함 + 정정 효과 재현되는가?
2. **회귀 위험** — 호출처 13곳 (`set_inline_shape_position` / `get_inline_shape_position`) 일괄 패치 + cursor_rect.rs hit-test 가드 추가가 다른 영역 (cursor 동작, 비-셀 inline shape) 에 회귀를 유발하지 않는가?
3. **PR base skew (5/6 등록 — 본 사이클 #578 후속 영역)** — 본질 영역과 본 사이클 처리분 0 중첩?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #628 nested cell inline_shape_positions 키 충돌 정정 — 글상자 안 이미지 미렌더링 | 정합 |
| author | @planet6897 (Jaeuk Ryu, jaeook.ryu@gmail.com) | 본 사이클 다수 PR 동일 컨트리뷰터 |
| changedFiles | 13 / +532 / -28 | 본질 src 7 파일 (+62/-28) + plans/working/report 컨트리뷰터 fork (+470) |
| 본질 변경 | `render_tree.rs` (+43/-11) — `InlineShapeKey` 타입 정의 + `cell_path` 차원 추가 | 단일 핵심 |
| 본질 호출처 | layout.rs / paragraph_layout.rs / shape_layout.rs / table_layout.rs / table_partial.rs / cursor_rect.rs | 13곳 일괄 패치 |
| **mergeable** | UNKNOWN (UI), 본 환경 cherry-pick 충돌 0 (orders 만 add/add) | base skew 8 commits |
| 코드 본질 | 단일 commit `04ce0d22` | ⭐ cherry-pick |
| Issue | closes #628 | ✅ |
| CI | 모두 SUCCESS (Build & Test / CodeQL × 3 / Canvas visual diff) | ✅ |

## 3. PR 의 1 commit 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`04ce0d22`** Task #628 본질 정정 + plans/working/report | src 7 파일 + 컨트리뷰터 fork plans/working/report | ⭐ src 7 파일만 cherry-pick (orders 충돌 ours / fork plans/working/report 본 환경 미도입) |

→ **본질 cherry-pick = `04ce0d22` src 7 파일만**. PR #561~#600 동일 패턴.

## 4. 본질 변경 영역 (`render_tree.rs` +43/-11)

### 4.1 결함 가설 (PR 본문 + Issue #628 인용)

`PageRenderTree.inline_shape_positions` 의 키 `(section, para, control)` 에서 `para` 가 두 가지 의미 혼용:

1. **paragraph_layout 호출 시** → 섹션 단위 paragraph 인덱스 (예: pi=119, pi=127)
2. **layout_table → 셀 paragraph 호출 시** → 셀 내부 paragraph 인덱스 (`cp_idx`, 보통 0)

서로 다른 셀 컨텍스트가 동일 키 `(0, 0, 1)` 등을 공유 → 다른 paragraph 의 double-nested 셀 처리가 키를 미리 점유 → 20번 외부 1×1 표 처리 시 `already_rendered_inline=true` 오판 → `table_layout.rs:1900` 분기에서 내부 2×3 표의 `layout_table` 재귀 호출 스킵 → 그 안의 그림 미렌더.

**비대칭 발현**:

| 문항 | 표 nesting | 결과 |
|---|---|---|
| 19번 (pi=119) | 2×3 표 → 셀 → 그림 (단일 nesting) | ✅ 정상 |
| 20번 (pi=127) | **1×1 → 셀 → 2×3 → 셀 → 그림 (이중 nesting)** | ❌ 누락 |

### 4.2 정정 (`InlineShapeKey` 신규 타입)

```rust
// 신규
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);
//                          section, para, control, cell_path
inline_shape_positions: HashMap<InlineShapeKey, (f64, f64)>
```

`cell_path` = 외→내 nesting 순서의 `(control_index, cell_index, cell_para_index)` 튜플 목록. **섹션 단위는 빈 Vec, 셀 단위는 `CellContext.path` 전체**.

`set/get_inline_shape_position` 시그니처에 `cell_ctx: Option<&CellContext>` 추가. 호출처 13곳 일괄 패치 (셀 단위 9 + 섹션 단위 4):
- 셀 단위: `paragraph_layout.rs` 6곳 (각 `cell_ctx.as_ref()` 전달) + `table_layout.rs` 2곳 + `table_partial.rs` 1곳
- 섹션 단위: `layout.rs` 4곳 (`None` 전달) + `cursor_rect.rs` 1곳 (`None` 전달) + `shape_layout.rs` 2곳 (`None` 전달)

**`cursor_rect.rs:532` hit-test 가드 추가**:
```rust
let (si, pi, ci, ref cell_path) = *key;
// 셀 내부 inline shape 은 cursor hit-test 에서 별도 처리 — 섹션 단위만 검사
if !cell_path.is_empty() { continue; }
```

→ 셀 내부 inline shape 의 cursor hit-test 별도 처리 영역 차단 (cursor 동작 회귀 0 보존).

### 4.3 정량 측정 (본 환경 직접 재현)

**BEFORE (devel HEAD `18f5161`)**:
```
page 1: 12 images
page 2: 2 images
page 3: 2 images
page 4: 3 images   ← 결함
```

**AFTER (cherry-pick `04ce0d22` 적용)**:
```
page 1: 12 images
page 2: 2 images
page 3: 2 images
page 4: 4 images   ← 정정 (+1)
```

**page 4 추가 image 위치**:
```
x=568.00 y=783.92 width=376.65 height=101.81
```

→ **PR 본문 `x=568 y=783.92 width=376.65 height=101.81 = 99.7×26.9mm IR` 명세 100% 일치** (실린더 이미지 `bin_id=2` 정상 위치).

### 4.4 SVG byte 차이 (PR 본문 100% 재현)

| 페이지 | byte 차이 | 평가 |
|---|---|---|
| page 1 | identical | ✅ 회귀 0 |
| page 2 | identical | ✅ 회귀 0 |
| page 3 | identical | ✅ 회귀 0 |
| **page 4** | **differ (461,151 → 532,816, +71,665 bytes)** | ✅ 20번 실린더 이미지 신규 emit (이미지 데이터 base64 포함) |

→ page 1/2/3 byte-identical (회귀 0) + page 4 의도된 정정.

## 5. 본 환경 직접 검증 (임시 브랜치 `pr629-cherry-test`)

| 단계 | 결과 |
|------|------|
| `04ce0d22` cherry-pick | ✅ 본질 src 7 파일 충돌 0 (`mydocs/orders/20260506.md` add/add 충돌은 ours 보존) |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ issue_546 1 + issue_554 12 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **본 환경 base skew 영역 (5/6 등록 → 본 사이클 #578 후속 commits 8개 차이) 영향 0** — 본질 src 7 파일 충돌 0 + 결정적 검증 모두 통과.

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,684** |
| 총 페이지 (AFTER) | **1,684** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 키 namespace 분리가 페이지네이션에 영향 없음 (회귀 0).

## 7. PR 본문의 자기 검증 결과 (본 환경 재검증)

| 검증 | PR 본문 결과 | 본 환경 재검증 |
|------|---------|----------|
| `cargo test --release --lib` | 1134 passed | ✅ **1140 passed** (본 환경 baseline 정합, +6 = task576 신규) |
| 회귀 sweep 5 샘플 56 페이지 | byte-identical | ✅ exam_science 4 페이지 (page 1/2/3 identical, page 4 의도된 정정) |
| `cargo clippy --release --lib` | 신규 경고 0 | ✅ 0건 |
| exam_science page 4: 3 → 4 images | 명시 영역 | ✅ 본 환경 BEFORE 3 → AFTER 4 정합 |
| 20번 이미지 위치 (x=568 y=783.92 w=376.65 h=101.81) | 명시 영역 | ✅ 본 환경 정확 일치 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |
| Docker WASM 빌드 | (미명시) | ✅ **4,590,307 bytes** (PR #578 baseline +7,151) |

## 8. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 깨끗** — src 7 파일 충돌 0, base skew 영향 없음
- ✅ **결정적 검증 정합** — 1140 passed / clippy 0 / svg_snapshot 6/6 / issue_546 + issue_554 모두 통과
- ✅ **정량 측정 정합** — BEFORE 3 → AFTER 4 images, 20번 이미지 위치 PR 본문 명세 100% 일치
- ✅ **광범위 sweep 회귀 0** — 164 fixture / 1,684 페이지 / 차이 0
- ✅ **케이스별 명시 가드** — 키 namespace 분리만 수행, 값/계산 로직 무변경 (`feedback_hancom_compat_specific_over_general` 정합)
- ✅ **HWP IR 표준 직접 사용** — `CellContext.path` (외→내 nesting 경로) 활용, 휴리스틱 미도입 (`feedback_rule_not_heuristic` 정합)
- ✅ **회귀 위험 영역 좁힘** — 섹션 단위 호출 (`None` 전달) 은 기존 `(sec, para, ctrl, [])` 와 동등, 셀 단위 호출 (`Some(ctx)` 전달) 만 stale-key 충돌 차단
- ✅ **cursor 가드** — `cursor_rect.rs` hit-test 루프에 `cell_path.is_empty()` 가드 → 셀 내부 inline shape 의 cursor hit-test 별도 처리 영역 보존

### 우려 영역
- ⚠️ **PR base UNKNOWN — 본 환경 검증으로 정합 확정** (UI 표시는 PR base diff 8 commits 뒤 영역, 본질 0 중첩 확인)
- ⚠️ **작업지시자 시각 판정 게이트** — exam_science page 4 의 20번 실린더 이미지 정상 위치 (글상자 안 + 99.7×26.9mm) 시각 검증 필수
- ✅ **WASM 빌드** — Docker WASM 빌드 완료, **4,590,307 bytes** (PR #578 baseline 4,583,156 +7,151 — render_tree.rs +43 LOC + InlineShapeKey Vec allocation 정합)

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `04ce0d22` src 7 파일 충돌 0
- ✅ **결정적 검증** — 1140 passed / clippy 0 / svg_snapshot 6/6
- ✅ **정량 측정 정합** — BEFORE 3 → AFTER 4 images, 위치 PR 본문 명세 정확 일치
- ✅ **광범위 sweep 회귀 0** — 164 fixture / 1,684 페이지 / 차이 0
- ✅ **HWP IR 표준 직접 사용** — `CellContext.path` 활용, 휴리스틱 미도입
- ⏳ **시각 판정 + WASM 빌드 별도 진행 필요**

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `04ce0d22` 의 src 7 파일만 cherry-pick (orders 충돌 ours / 컨트리뷰터 fork plans/working/report 본 환경 미도입)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep 통과 확인
- WASM 빌드 산출물 검증
- SVG 생성 + 작업지시자 시각 판정 (★ 게이트, exam_science page 4 — 20번 실린더 이미지 정상 위치)
- 통과 시 devel merge + push + PR close (한글 댓글)

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장.

## 10. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (164 fixture / 1,684 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 키 namespace 분리만 수행, 값/계산 로직 무변경 (case-specific 가드)
- ✅ `feedback_rule_not_heuristic` — `CellContext.path` (HWP IR 표준 nesting 경로) 직접 사용, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 권장 (exam_science page 4)
- ✅ `feedback_pdf_not_authoritative` — PR 본문 IR 명세 (99.7×26.9mm) 검증, PDF 미사용
- ✅ `feedback_per_task_pr_branch` — Task #628 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 환경 5/6 처리분 (PR #578) 후속 사이클 정합 점검
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/6 v0.7.10 후 첫 PR 처리분 - PR #578 후속) 영역 정합
- ✅ `feedback_image_renderer_paths_separate` — 본 PR 의 키 namespace 분리는 SVG/Canvas 양쪽 동일 영향 (renderer 별 분기 없음, 데이터 구조 영역)

## 9.5 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.5.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| `04ce0d22` cherry-pick | ✅ 본질 src 7 파일 충돌 0, **author Jaeook Ryu 보존** |
| `mydocs/orders/20260506.md` | ours 보존 (본 환경 5/6 orders 유지) |
| 컨트리뷰터 fork plans/working/report | 본 환경 미도입 (PR 검토는 `pr/` 폴더 정책) |
| local/devel commit | `c353cfc` |

### 9.5.2 결정적 재검증 (local/devel cherry-pick 후)

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| **Docker WASM 빌드** | ✅ **4,590,307 bytes** (1m 27s, PR #578 baseline 4,583,156 +7,151 — render_tree.rs +43 LOC + InlineShapeKey Vec allocation 정합) |

### 9.5.3 광범위 페이지네이션 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,684** |
| 총 페이지 (AFTER) | **1,684** |
| **fixture 별 페이지 수 차이** | **0** |

→ 본 PR 의 키 namespace 분리가 페이지네이션에 영향 없음 (회귀 0).

### 9.5.4 SVG byte 차이 + 정량 측정 (PR 본문 100% 재현)

| 페이지 | BEFORE images | AFTER images | byte 차이 | 평가 |
|---|---|---|---|---|
| page 1 | 12 | 12 | identical | ✅ 회귀 0 |
| page 2 | 2 | 2 | identical | ✅ 회귀 0 |
| page 3 | 2 | 2 | identical | ✅ 회귀 0 |
| **page 4** | **3** | **4** | **differ (461,151 → 532,816, +71,665 bytes)** | ✅ **20번 실린더 이미지 +1** (정정 영역) |

**page 4 추가 image 위치 (4번째)**:
```
x=568.00 y=783.92 width=376.65 height=101.81
```

→ **PR 본문 명세 `99.7×26.9mm IR 정확 매칭` 100% 일치**.

### 9.5.5 시각 판정 자료 (작업지시자 검증용)

| 자료 | 위치 | 비고 |
|---|---|---|
| **Before** (devel HEAD `18f5161`, fix 미적용) | `output/svg/pr629_before/exam_science/exam_science_00{1..4}.svg` | 4 페이지, page 4 = 3 images |
| **After** (cherry-pick `c353cfc` 적용) | `output/svg/pr629_after/exam_science/exam_science_00{1..4}.svg` | 4 페이지, page 4 = **4 images** |
| **차이 페이지** | page 4 단독 | page 1/2/3 byte-identical |

**시각 판정 권위 영역**:
- **page 4 — 20번 문항** 글상자 안 (외부 1×1 → 내부 2×3 → 셀 → 그림 이중 nesting) **실린더 이미지** (`bin_id=2`, 99.7×26.9mm) 정상 위치 (`x=568 y=783.92 w=376.65 h=101.81`) 출력 확인.
- 19번 (`bin_id=1`, 단일 nesting) + 다른 영역은 회귀 없음.

**WASM 산출물**: `pkg/rhwp_bg.wasm` 4,590,307 bytes (Docker WASM 빌드 1m 27s, PR #578 baseline +7,151 bytes — render_tree.rs +43 LOC + InlineShapeKey Vec allocation 정합).

### 9.5.6 다음 단계

5. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_science page 4 — 20번 실린더 이미지 정상 위치) — 본 단계 대기 중
6. ⏳ 통과 시 devel merge + push + PR close (한글 댓글) + Issue #628 close (closes #628 자동 처리)
7. ⏳ 처리 보고서 (`pr_629_report.md`) 작성 + archives 이동 + 5/6 orders 갱신

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**옵션 A 진행 — 시각 판정 게이트 대기**.
