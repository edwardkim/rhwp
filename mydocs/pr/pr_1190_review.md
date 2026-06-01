# PR #1190 검토 — Task #1187: BookReview.hwp 글상자 내용 clip 회귀 수정

- **작성일**: 2026-05-31
- **PR**: #1190 (OPEN)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터 — PageLayerTree/image 시리즈 #1185/#1175/#1174/#1163/#1019)
- **연결 이슈**: #1187 (body "관련 이슈: #1187" — Closes 미명시 → 자동 클로즈 없음, 수동 판단)
- **base/head**: `devel` ← `fix/1187-bookreview-textbox-clip` (`5fa8eebf`)
- **mergeable**: MERGEABLE / **BEHIND** (로컬 머지 필요)
- **규모**: 29 파일, +1266 / −51 (소스 ~16, golden SVG 2, docs/stage 11)
- **CI**: 전부 SUCCESS (**Canvas visual diff 포함**). WASM skip.

## 1. 문제와 원인

`samples/basic/BookReview.hwp` 1쪽에서 큰 점선 글상자 내용(5장/6장/에필로그 목차)이
글상자 하단 **밖으로 overflow** 되어 우측 하단 저자 정보 박스와 겹침. 원인:
- `layout_textbox_content` 가 글상자 내용을 clip 노드 없이 shape node 자식으로 직접 배치.
- SVG/paint layer 출력 경로가 Body/TableCell clip 만 처리, **TextBox clip 미처리**.

## 2. 수정 설계 검토

### (1) `src/renderer/layout/shape_layout.rs` (+30/−9) — TextBox 노드 도입
- 글상자 내용을 `RenderNodeType::TextBox`(inner_area bbox) 노드 아래로 모음.
  모든 자식 추가 경로(para/picture/shape/eq/table, 가로·세로·오버플로 4 return 경로)를
  `shape_node` → `&mut textbox_node` 로 변경.
- **`if !textbox_node.children.is_empty()` 가드** 로 빈 글상자에 불필요 clip 노드 생성 회피
  (4 경로 일관 적용).

### (2) `src/renderer/layout/paragraph_layout.rs` (+12/−2) — 이중 보정 방지
- `layout_composed_paragraph` 에 `suppress_column_top_vpos_fallback: bool` 추가.
- 두 column-top vpos fallback 분기(`spacing_before>0` else-if / `spacing_before==0` if)에
  `&& !suppress_column_top_vpos_fallback` 만 추가 → **additive, 기존 로직 보존**.
- **호출처 분리 (핵심 회귀 가드)**:
  - 글상자 호출 2곳(shape_layout.rs:1768/1961) = `true` → LINE_SEG.vpos 선배치 후
    column-top fallback 이중 적용 방지(5장/6장/에필로그 줄이 clip 안에 남도록).
  - table_cell_content / table_layout / table_partial / picture_footnote / paragraph_layout
    내부 호출 = **전부 `false`** → **표 셀의 기존 column-top vpos fallback 그대로 유지**.
  - 컴파일러가 전 호출처 인자 강제 + git grep 으로 누락 0 확인.

### (3) 렌더러 3경로 일관 적용 (feedback_image_renderer_paths_separate 준수)
- `src/renderer/svg.rs` (+14): TextBox → `textbox-clip-{id}` clipPath + `<g clip-path>` 진입/종료.
- `src/renderer/web_canvas.rs` (+8): `ClipKind::TextBox` save/clip/restore.
- `src/paint/builder.rs` (+68): TextBox → `ClipRect{TextBox} > Group{TextBox}` 래핑 + 단위 테스트.
- `src/paint/{json.rs,layer_tree.rs,schema.rs}` + `canvaskit_policy.rs` + `svg_layer.rs`:
  `ClipKind::TextBox`/`GroupKind::TextBox` arm 추가(match exhaustive), JSON `"clipKind":"textBox"`.
- schema additive minor bump 14→15.

### (4) `rhwp-studio/src/core/types.ts` (±1) — TS clipKind 유니온에 'textBox' 추가.

### (5) 테스트 + golden
- `tests/issue_1187_textbox_clip.rs` (+261): 회귀 테스트(글상자 clip 노드/bbox/overflow 차단).
- `src/paint/builder.rs`/`json.rs` 단위 테스트(TextBox clip layer/serialize).
- golden SVG `issue-267`/`issue-617`: TextBox clip 도입으로 의도적 변경(snapshot 갱신).

## 3. 위험 평가

- **낮음~중간.** 시각 회귀 수정이며 변경이 TextBox 경로에 집중. 표 셀 동작은 플래그 `false`
  로 보존(검증함). additive ClipKind/schema.
- **주의**: 시각 회귀 수정 → 작업지시자 직접 시각 판정이 핵심 게이트
  (feedback_visual_regression_grows / feedback_v076_regression_origin).
- golden SVG 2건은 의도적 변경 — 표면적 회귀 아님(PR body 명시 + svg_snapshot 8 passed).

## 4. 검증 결과 (로컬 머지 시뮬레이션 `pr1190-verify`)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| clippy(lib) | `cargo clippy --lib` | ✅ 0 warning/error |
| build | `cargo build` | ✅ Finished |
| 전체 테스트 | `cargo test --tests` | ✅ **1888 passed, 0 failed** |
| 회귀 | `issue_1187_textbox_clip` | ✅ 2 passed |
| snapshot | `svg_snapshot` | ✅ 8 passed |
| SVG 산출 | `export-svg BookReview -p 0` | ✅ `textbox-clip-33/52/103` 생성, 큰 글상자 clip `x=47.92 y=516.56 w=687.55 h=487.88` (PR 메모 일치) |
| CI(PR) | Build&Test / **Canvas visual diff** / CodeQL / Analyze ×3 | ✅ 전부 SUCCESS |

> 작업지시자 시각 판정용 SVG: `output/poc/pr1190/BookReview_page0.svg`

## 5. 판단 (예정)

코드·자동 검증 통과 → **머지 권고**. 단, **시각 회귀 수정이므로 작업지시자 직접 시각 판정을
게이트로** 둠. 승인 시 메인테이너 로컬 `--no-ff` 머지 + push(BEHIND). 머지 후 WASM 빌드 권장
(web_canvas 경로 변경 포함). `Refs #1187` 자동 클로즈 아님 → 시각 판정 후 수동 클로즈 판단.
결과는 `pr_1190_report.md` 에 기록.
