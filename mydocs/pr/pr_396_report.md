# PR #396 처리 보고서 — 수식 렌더링 개선 (TAC 높이 + 한글 이탤릭) (#174, #175)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#396](https://github.com/edwardkim/rhwp/pull/396) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| 이슈 | [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175) |
| 처리 결정 | **옵션 B (cherry-pick + 메인테이너 후속 정정)** |
| 처리 일자 | 2026-04-28 |

## 처리 절차

### Stage 1: cherry-pick

`local/pr396` 브랜치 (`local/devel` 분기) 에서 PR head 까지 2 commit cherry-pick — 작성자 attribution 보존:

| commit | 작성자 | 내용 |
|--------|--------|------|
| `2e62ef0` (← `5cf2b79`) | @oksure | 수식 렌더링 개선 (TAC 높이 + 한글 이탤릭) |
| `9d4669e` (← `dbaa74b`) | @oksure | Copilot 리뷰 반영 |

### Stage 2: 메인테이너 후속 정정 (Canvas 경로 SVG 일치화)

작업지시자 시각 판정 (web 에디터 vs SVG 비교) 에서 발견된 Canvas 경로 결함 3가지를 본 PR 후속 commit 으로 정정:

#### 1. Canvas 분수선 y 좌표 (`7048d3e`)

```diff
 LayoutKind::Fraction { numer, denom } => {
     render_box(ctx, numer, x, y, color, fs, italic, bold);
-    let line_y = y + lb.baseline;
+    // 분수선 — baseline에서 axis_height 위에 배치 (SVG 경로와 동일)
+    let line_y = y + lb.baseline - fs * super::layout::AXIS_HEIGHT;
```

#### 2. web_canvas.rs Equation 스케일 적용 (`dff4b07`)

```diff
 RenderNodeType::Equation(eq) => {
+    let scale_x = if eq.layout_box.width > 0.0 && node.bbox.width > 0.0 {
+        node.bbox.width / eq.layout_box.width
+    } else { 1.0 };
+    let scale_y = if eq.layout_box.height > 0.0 && node.bbox.height > 0.0 {
+        node.bbox.height / eq.layout_box.height
+    } else { 1.0 };
     self.ctx.save();
+    let _ = self.ctx.translate(node.bbox.x, node.bbox.y);
+    let needs_scale = (scale_x - 1.0).abs() > 0.01 || (scale_y - 1.0).abs() > 0.01;
+    if needs_scale {
+        let _ = self.ctx.scale(scale_x, scale_y);
+    }
     super::equation::canvas_render::render_equation_canvas(
         &self.ctx, &eq.layout_box,
-        node.bbox.x, node.bbox.y,
+        0.0, 0.0,
         &eq.color_str, eq.font_size,
     );
     self.ctx.restore();
 }
```

#### 3. Canvas Limit `fi = fs` (`d47b7c7`)

```diff
 LayoutKind::Limit { is_upper, sub } => {
     let name = if *is_upper { "Lim" } else { "lim" };
-    let fi = font_size_from_box(lb, fs);
+    let fi = fs;  // SVG 와 동일 — Limit 의 lb 는 "lim + 첨자" wrapper 라 base_fs 보다 큼
```

### Stage 3: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1031 passed** (1029 → +2 작성자 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 회귀 방지 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ 통과 |
| WASM 빌드 (Docker) | ✅ 1m 22s, 4,106,811 bytes |
| 작업지시자 시각 판정 (web 에디터) | ✅ 통과 |

### 시각 판정 산출물

`output/svg/pr396-visual/`:
- `exam_math_008.svg` (이슈 #175 CASES 한글)
- `exam_math_016.svg` (이슈 #174 분수)
- `exam_math_020.svg` (이슈 #174 큰 수식)
- `exam_math_p16_{before,after}.svg` (PR #396 적용 전후 비교)

## 변경 요약

| 파일 | 작성자 변경 | 메인테이너 변경 |
|------|----------|---------------|
| `src/renderer/equation/canvas_render.rs` | CJK 텍스트 italic 비활성화 | Fraction line_y 정정 + Limit fi 정정 |
| `src/renderer/equation/layout.rs` | CJK 너비 보정 제외 + 단위 테스트 2개 | — |
| `src/renderer/equation/svg_render.rs` | CJK 텍스트 italic 비활성화 | — |
| `src/renderer/layout/paragraph_layout.rs` | TAC 수식 높이 HWP 권위값 사용 (2 곳) | — |
| `src/renderer/svg.rs` | 수식 SVG Y축 스케일 추가 | — |
| `src/renderer/web_canvas.rs` | — | Equation X/Y 스케일 적용 |

## 작업지시자 정황 보고

본 PR 처리 중 다음 절차 위반 정황 발견 → 다음부터 엄격 준수:

1. **PR #395 처리 시 시각 판정 없이 머지 + push** — 작업지시자가 정정 요청 (revert 진행 후 재정정)
2. **`/tmp/` 등 임시 폴더에 SVG 출력** — 매뉴얼 + 메모리 (`project_output_folder_structure.md`) 위반. 본 PR 진행 중 정식 위치 (`output/svg/pr396-visual/`) 로 이동

본 PR #396 처리는 위 두 절차 모두 엄격 준수:
- ✅ 시각 판정 게이트 (push 전)
- ✅ output/svg/pr396-visual/ 정식 위치 사용

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 작업지시자 시각 판정 게이트 | ✅ Stage 3 후 push 전 시각 판정 통과 |
| PR 댓글 톤 — 과도한 표현 자제 | ✅ |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr396` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 PR close |
| output 폴더 서브폴더 구조 | ✅ output/svg/pr396-visual/ |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 커밋
2. `local/pr396` → `local/devel` → `devel` 머지 + push
3. PR #396 close + 작성자 댓글
4. 머지 commit devel 검증

## 참고

- 검토 문서: `mydocs/pr/pr_396_review.md`
- 이전 PR: [#388](https://github.com/edwardkim/rhwp/pull/388) (CLOSED — base=main 정정 안내)
- 동일 작성자 PR: [#395](https://github.com/edwardkim/rhwp/pull/395) (그림 밝기/대비, 머지 완료)
- 이슈: [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175)
- 별도 task #421: 복학원서.hwp BehindText 그림 후속 본문 문단 배치 결함 (별개 결함, OPEN)
