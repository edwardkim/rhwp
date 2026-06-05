# PR #1102 처리 보고 — 회전 90°/270° 이미지 bbox 이중회전 정정

## 1. 결정

**MERGE 수용** — SVG/native canvas 영역 영역 해소 우선 확보 + CanvasKit 영역 영역 후속 이슈 분리.

| 항목 | 값 |
|------|-----|
| 번호 | #1102 |
| 작성자 | HaimLee-4869 (Lee eunjung) — 기존 컨트리뷰터 (9번째 PR) |
| 연결 이슈 | 없음 (closes 미지정) |
| 후속 이슈 | **#1127** (paint/builder.rs 영역 영역 동일 정정 누락) |
| 처리일 | 2026-05-26 |
| Merge commit | `4e2c938ff4756617528d096328166f789ed8f189` |
| Merge 방식 | `gh pr merge 1102 --merge --admin` (BEHIND 영역) |

## 2. 검증 결과

### 자동 검증 (통과)

| 항목 | 결과 |
|------|------|
| cherry-pick `e8b6ee4c` (검증 전용) | ✅ 충돌 없음 (Auto-merging web_canvas.rs) |
| `cargo build --release` | ✅ 통과 |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (25.88s) |
| `cargo test --release --tests` | ✅ svg_snapshot 8 passed / tab_cross_run 1 passed |
| GitHub CI | ⚠️ 미실행 (fork 패턴) |

### 시각 검증 (간접)

- 한컴 회전 이미지 hp:pic fixture 영역 부재 → 직접 시각 검증 미수행
- 컨트리뷰터 자체 검증 보고 (bbox 좌표 수학적 정합 + 한컴2024 PDF 실측 170.6×242.7mm) 영역 영역 의존
- 한컴 2020/2022 영역 영역 재검증 권장 (메모리 룰 `feedback_pdf_not_authoritative`)

## 3. 변경 영역 요약

4 파일, +47/-12 (단일 commit):

| 파일 | 영역 |
|------|------|
| `src/renderer/render_tree.rs` (+21) | `ShapeTransform::effective_image_bbox()` 헬퍼 신규 — rotation 90°/270° (±1° 톨러런스) 시 bbox extent swap (중심 cx, cy 불변) |
| `src/renderer/svg.rs` (+3/-1) | `RenderNodeType::Image` 분기 영역 effective_image_bbox 적용 |
| `src/renderer/canvas.rs` (+18/-6) | 일반 path + LayerOp path 양쪽 적용 |
| `src/renderer/web_canvas.rs` (+5/-3) | `RenderNodeType::Image` 분기 적용 |

## 4. 별도 발견 — #1127 등록

PR #1102 검토 중 발견. paint/builder.rs:86 의 `RenderNodeType::Image` 분기 영역 영역 effective_image_bbox 미적용 → rhwp-studio (CanvasKit) 영역 영역 회전 이미지 결함 잔존.

**메모리 룰 `feedback_image_renderer_paths_separate` 위반** — 4개 renderer path sweep 영역 영역 1개 누락:
- ✅ svg.rs (PR #1102)
- ✅ canvas.rs (PR #1102)
- ✅ web_canvas.rs (PR #1102)
- ❌ paint/builder.rs (#1127 영역 영역 후속)

**이슈**: https://github.com/edwardkim/rhwp/issues/1127

## 5. 후속 권장 영역

| 항목 | 우선순위 | 영역 |
|------|---------|------|
| paint/builder.rs 정정 (rhwp-studio 결함 해소) | 본 PATCH 후속 | #1127 |
| 회전 이미지 fixture + svg_snapshot 회귀 가드 | M100 후속 | 별도 영역 |
| 한컴 2020/2022 PDF 영역 시각 재검증 | M100 후속 | 권위 자료 영역 |
| `effective_image_bbox` unit test | M100 후속 | 회귀 가드 |

## 6. 메모리 룰 정합

- ⚠️ `feedback_image_renderer_paths_separate` — 3/4 path 정합 (paint/builder 누락 → #1127)
- ✅ `feedback_visual_judgment_authority` — 컨트리뷰터 자체 정량 검증 영역 (수학적 정합) 영역 영역 의존, 직접 시각 검증 미수행
- ✅ `feedback_check_open_prs_first` — 동일 컨트리뷰터 사이클 (#1088 회귀 거절) 영역 점검
- ⚠️ `feedback_pdf_not_authoritative` — 한컴2024 PDF 정답지 등급 외 (한컴 2020/2022 재검증 후속 권장)
- ✅ `feedback_v076_regression_origin` — 자동 검증 신뢰 회피 + 결함 소견 영역 솔직 보고
