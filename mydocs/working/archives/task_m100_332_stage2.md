# Task #332 Stage 2 — layout per-paragraph advance 를 `height_for_fit` 와 정합 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/layout/paragraph_layout.rs:2432-2446`)

```diff
             col_node.children.push(line_node);
-            // 줄간격 적용: 셀 내 마지막 문단의 마지막 줄에서만 trailing spacing 제외
+            // 줄간격 적용 — typeset 의 height_for_fit 모델과 정합:
+            //   - 셀 내 마지막 문단의 마지막 줄: 기존대로 trailing 제외
+            //   - 일반 문단의 마지막 visible 줄(=문단 전체 마지막 줄): trailing 제외 (Task #332)
+            //   - partial 문단(split 된 경우)의 마지막 visible 줄: trailing 유지 (다음 단의 첫 줄과의 간격)
             let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
-            if !is_cell_last_line || cell_ctx.is_none() {
+            let is_para_last_line = cell_ctx.is_none()
+                && line_idx + 1 == end
+                && end == composed.lines.len();
+            if (is_cell_last_line && cell_ctx.is_some()) || is_para_last_line {
+                y += line_height;
+            } else {
                 let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
                 y += line_height + line_spacing_px;
-            } else {
-                y += line_height;
             }
```

split 된 partial 문단의 마지막 visible 줄은 trailing 유지(다음 단/페이지 첫 줄과의 간격 역할). 문단 전체가 한 단에 들어간 경우만 trail_ls 제외.

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (issue-147, issue-157)
cargo test --test '*' (기타)      → 모두 passed
```

### golden SVG baseline 변경 (의도된 변경)

`issue-157/page-1.svg`, `issue-147/aift-page3.svg` 두 개의 baseline 이 변경됨. 모든 y 좌표가 약 **9.6 px 위로 shift** — 첫 문단의 trail_ls 가 advance 누적에서 제외되어 후속 요소 전체가 위로 이동한 결과. 의도된 변경.

샘플 diff (`issue-157/page-1.svg`):
```
< clipPath body-clip-3 ... height="1042.93"
> clipPath body-clip-3 ... height="1033.33"   # body 클리핑 9.6px 감소

< clipPath cell-clip-25 ... y="246.43"
> clipPath cell-clip-25 ... y="236.83"        # 모든 셀 9.6px 위로
...
```

**baseline 갱신 정책**: 본 단계 commit 후 사용자 승인을 받아 `UPDATE_GOLDEN=1` 으로 갱신할지, Stage 5 까지의 누적 변화를 한 번에 갱신할지 결정 필요.

### 수동 회귀 (21_언어 page 0)

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 --debug-overlay
```

| 단계 | LAYOUT_OVERFLOW |
|------|-----------------|
| Stage 1 | col=0 pi=9 (7.7px), col=0 pi=10 (9.5px) — **2 건** |
| Stage 2 | col=0 pi=10 partial (9.5px) — **1 건** ← pi=9 의 7.7px 해소 |

pi=10 partial 의 9.5px overflow 는 vpos correction(`layout.rs:1392`) 이 다시 trail_ls 를 더하기 때문. Stage 3 에서 `vpos_end` 의 trail_ls 제외 + collapse 가드로 해소 예정.

## 다음 단계

Stage 3a: `layout.rs:1367` 의 `vpos_end = vp + lh + ls` → `= vp + lh` 로 변경 (trail_ls 제외).
Stage 3b: `layout.rs:1392` 의 `end_y >= y_offset - 1.0` 단방향 → 양방향 + collapse 가드.

## 미결 사항

- golden SVG 2 개 baseline 갱신: Stage 5 누적 후 일괄 갱신 vs 각 단계마다 갱신 — 사용자 결정 필요. 본 보고서에서는 **Stage 5 완료 시 일괄 갱신** 을 제안.
