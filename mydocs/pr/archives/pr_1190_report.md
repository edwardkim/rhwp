# PR #1190 처리 보고서 — Task #1187: BookReview.hwp 글상자 내용 clip 회귀 수정

- **작성일**: 2026-05-31
- **PR**: #1190 → **MERGED** (devel, 머지커밋 `c8f3a201`)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터 — PageLayerTree/image 시리즈)
- **연결 이슈**: #1187 → **CLOSED** (body "관련 이슈" — Closes 미명시라 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 SVG 시각 판정 통과)

## 결정 사유

`BookReview.hwp` 1쪽 큰 글상자 내용(5장/6장/에필로그 목차)이 글상자 하단 밖으로 overflow 되어
저자 정보 박스와 겹치는 회귀. 글상자 내용을 `RenderNodeType::TextBox` clip 노드로 감싸고
SVG/paint layer/web_canvas 세 경로에서 동일하게 clip 처리하여 해결. 표 셀 동작은 플래그로
보존. 시각 회귀 수정이므로 작업지시자 직접 시각 판정을 게이트로 두었고, **통과**.

## 변경 요약 (29 파일, +1266 / −51)

| 영역 | 변경 |
|------|------|
| `src/renderer/layout/shape_layout.rs` (+30/−9) | 글상자 내용을 `RenderNodeType::TextBox`(inner_area) 노드로 모음. 4 return 경로 일관, `if !children.is_empty()` 가드 |
| `src/renderer/layout/paragraph_layout.rs` (+12/−2) | `suppress_column_top_vpos_fallback` 플래그 — 글상자 호출만 true, 표/footnote false (vpos 이중 보정 방지, 표 셀 동작 보존) |
| `src/renderer/svg.rs` / `web_canvas.rs` / `src/paint/builder.rs` | TextBox clip 3경로 일관 적용 (feedback_image_renderer_paths_separate) |
| `src/paint/{json.rs,layer_tree.rs,schema.rs}` + `canvaskit_policy.rs` + `svg_layer.rs` | `ClipKind::TextBox`/`GroupKind::TextBox` arm, JSON `"clipKind":"textBox"`, schema 14→15 |
| `rhwp-studio/src/core/types.ts` | clipKind 유니온에 'textBox' |
| `tests/issue_1187_textbox_clip.rs` (+261) + builder/json 단위 | 회귀 + clip layer/serialize |
| golden `issue-267`/`issue-617` | TextBox clip 도입 의도적 변경 |

## 검증 결과

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| clippy(lib) | `cargo clippy --lib` | ✅ 0 |
| build | `cargo build` | ✅ Finished |
| 전체 테스트 | `cargo test --tests` | ✅ **1883 passed, 0 failed** |
| SVG 산출 | `export-svg BookReview -p 0` | ✅ `textbox-clip-52` (bottom 1004.44) — PR 메모 일치 |
| **시각 판정** | 작업지시자 SVG 직접 판정 | ✅ **통과** |
| WASM | `docker compose ... wasm` | ✅ pkg 빌드 |
| CI(PR) | Build&Test / **Canvas visual diff** / CodeQL / Analyze ×3 | ✅ 전부 SUCCESS |

## 처리 절차

1. PR 정보 확인 — MERGEABLE/BEHIND, CI(Canvas diff 포함) green. 컨트리뷰터 사이클 점검.
2. 4개 영역 소스(layout/paint/renderer 3경로/test) + golden + 회귀 가드(플래그 호출처 분리) 검토.
3. 로컬 `pr1190-verify` 머지 시뮬레이션 전체 검증 + 시각 판정용 SVG 산출 → `pr_1190_review.md` → 승인.
4. **작업지시자 SVG 시각 판정 통과**.
5. 메인테이너 로컬 `--no-ff` 머지(`1a7c4eed..c8f3a201`) + 재검증 + push. PR head 조상 확인.
6. WASM Docker 빌드. 이슈 #1187 수동 클로즈 + PR 코멘트 + 보고서.

## 비고

- BEHIND → 메인테이너 로컬 통합. cross-repo `--no-ff` 이지만 이번엔 GitHub 자동 MERGED 인식
  (#1159 와 동일). 이슈 #1187 은 Closes 미명시라 수동 클로즈.
- web_canvas 경로 변경 포함 → WASM 빌드로 rhwp-studio 직접 확인 가능.
- @postmelee 의 PageLayerTree/렌더링 누적 기여(#1185/#1175/#1174/#1163/#1019)의 일부.
