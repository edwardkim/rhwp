# Task #279 Stage 1 — 작성자 핵심 3 커밋 cherry-pick

## 목표

[@seanshin](https://github.com/seanshin) 의 PR [#282](https://github.com/edwardkim/rhwp/pull/282) 에서 #279 핵심 3 커밋을 `local/task279` (origin/devel 기반) 에 cherry-pick 하여 작성자 author 정보를 보존한다.

## 절차

```bash
git checkout -b local/task279 origin/devel   # 사전 완료
git fetch https://github.com/seanshin/rhwp.git feature/task279-toc-leader-tab:pr282-task279

# 작성자 fork 의 핵심 3 커밋:
#   5d1c80f 수행계획서
#   d48af5c Stage 2+3 핵심 수정 (3 파일)
#   76436df Stage 3 보고서

git cherry-pick 5d1c80f   # → f27477e (author=hyoun mouk shin)
git cherry-pick d48af5c   # → 2eb1be5 (author=hyoun mouk shin)
git cherry-pick 76436df   # → 4770a8a (author=hyoun mouk shin)
```

cherry-pick 모두 conflict 없이 자동 처리됨 (Auto-merging svg.rs / web_canvas.rs / text_measurement.rs — 단순 인접 영역 변경).

이후 메인테이너가 강화 버전 수행계획서 + 신규 구현계획서를 별도 커밋으로 추가:

```bash
git add mydocs/plans/task_m100_279.md mydocs/plans/task_m100_279_impl.md
git commit  # → cfb8ba6 (author=edward, Co-Authored-By: hyoun mouk shin)
```

## 결과 — 4 커밋

| commit | author | 내용 |
|--------|--------|------|
| `f27477e` | hyoun mouk shin | 작성자 수행계획서 (`mydocs/plans/task_m100_279.md` v1) |
| `2eb1be5` | hyoun mouk shin | **핵심 수정**: `svg.rs` (+2/-2), `web_canvas.rs` (+6/-1), `layout/text_measurement.rs` (+4/-2) |
| `4770a8a` | hyoun mouk shin | 작성자 Stage 3 보고서 (`mydocs/working/task_m100_279_stage3.md`) |
| `cfb8ba6` | edward (Co-Authored-By: hyoun mouk shin) | 메인테이너 강화 수행계획서 + 구현계획서 (7 기여 인정 항목 명문화) |

## 코드 변경 검증

`git diff origin/devel -- src/renderer/svg.rs src/renderer/web_canvas.rs src/renderer/layout/text_measurement.rs`:

### 1. `text_measurement.rs::find_next_tab_stop`

```diff
-        // 탭 위치가 사용 가능 너비를 초과하면 available_width로 클램핑
-        let pos = if ts.position > available_width && available_width > 0.0 {
+        // type=1(오른쪽) 탭은 단 기준 절대 위치이므로 available_width 클램핑 제외.
+        // 들여쓰기(left_margin)가 있는 문단에서도 오른쪽 탭이 동일 위치에 정렬되도록 한다.
+        // type=0(왼쪽)/2(가운데) 탭은 종전대로 클램핑하여 텍스트 영역 밖으로 넘어가지 않게 한다.
+        let pos = if ts.tab_type != 1 && ts.position > available_width && available_width > 0.0 {
             available_width
         } else {
             ts.position
```

### 2. `svg.rs` — fill_type=3 (점선)

```diff
                 3 => {
-                    // 점선 ···
+                    // 점선 ··· — round cap으로 원형 점 표현 (한컴 동등)
                     self.output.push_str(&format!(
-                        "...stroke-width=\"0.5\" stroke-dasharray=\"1 2\"/>\n",
+                        "...stroke-width=\"1.0\" stroke-dasharray=\"0.1 3\" stroke-linecap=\"round\"/>\n",
                         lx1, ly, lx2, ly, color,
                     ));
                 }
```

### 3. `web_canvas.rs` — fill_type=3 (점선)

```diff
-                3 => draw_line(&self.ctx, ly, 0.5, &[1.0, 2.0]),  // 점선
+                3 => {
+                    // 점선 ··· — round cap으로 원형 점 표현 (한컴 동등)
+                    self.ctx.set_line_cap("round");
+                    draw_line(&self.ctx, ly, 1.0, &[0.1, 3.0]);
+                    self.ctx.set_line_cap("butt");
+                }
```

## Stage 1 완료 조건 점검

- [x] 3 작성자 커밋 cherry-pick 완료 (conflict 없음)
- [x] author=hyoun mouk shin 보존 (3 커밋 모두 `git log --pretty=format:'%an'` 확인)
- [x] 메인테이너 신규 커밋에 `Co-Authored-By: hyoun mouk shin` trailer 포함
- [x] 코드 변경 적용 확인 (3 파일 diff)
- [x] working tree 정상 (관련 파일 한정)

## 다음 단계

Stage 2 — 빌드 + 단위/통합 테스트 + clippy + wasm32 검증.
