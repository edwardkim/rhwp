# Stage 1 — task_m100_4159 진단·래칫 준비

- **이슈**: [#4159](https://github.com/edwardkim/rhwp/issues/4159)
- **계획서**: [`mydocs/plans/task_m100_4159.md`](../plans/task_m100_4159.md)
- **브랜치**: `task_m100_4159_nested_table_bottom_clip`
- **분기 기준**: `upstream/devel` `06f8ebcca`
- **PR 직전 통합 기준**: `upstream/devel` `23ff5b6f1` (merge commit `e6f09003b`)
- **작업 시각**: 2026-08-07 KST

## 1. 기준 진단

실제 fixture 물리 3쪽의 bottom `Line`은 생성돼 있지만 조상 분할 셀 clip 밖에 있다.

| 노드 | 하단 |
| --- | ---: |
| 조상 partial Table | 827.3px |
| 조상 `clip=true` TableCell | 824.88px |
| 종료 nested Table | 827.3px |
| nested bottom Line stroke | 827.27px |

재귀 자식 표가 조상 `inner_area.y = cell_y + pad_top`에서 시작하면서 조상 조각과 같은
711.5px fragment 높이를 쓴다. `layout_partial_table()`의 최종 자손 포섭은 Table bbox만
확장하므로 TableCell bbox를 clip으로 쓰는 SVG와 Canvas2D에서 선이 잘린다.

## 2. 구현 판정 기준

- `cell_cut_window`가 마지막 유닛까지 포함하는 종료 셀만 확장 후보로 삼는다.
- 실제 재귀 Table 자손의 하단이 셀 하단을 넘는 경우에만 필요한 만큼 확장한다.
- 다음 continuation이 있는 `eu < units.len()` 조각은 기존 셀 clip을 보존한다.
- cell background·외부 edge grid의 조판 위치는 바꾸지 않고 clip 포섭만 정합시킨다.

## 3. 다음 단계

실제 fixture 구조 래칫을 red로 고정한 뒤 최소 구현, SVG·Canvas2D 시각 증적, 기존 #2007
페이지네이션 회귀를 순서대로 수행한다.

## 4. red → green 구현

실제 물리 3쪽 구조 래칫은 수정 전 다음 값으로 실패했다.

```text
line_bottom=827.273, clip_bottom=824.880
```

`table_partial.rs`에 종료 분할 셀 clip 포섭 헬퍼를 추가했다. `cell_cut_window`의 끝이
`usize::MAX`인 마지막 유닛 조각이고 `clip=true`인 셀만 대상으로, 실제 재귀 `Table` 자손의
최하단까지 셀 bbox를 확장한다. 비종료 조각과 중첩 표가 없는 셀은 그대로 둔다. 셀 배경과
외부 edge grid의 좌표는 바꾸지 않는다.

합성 unit은 terminal 셀이 80px에서 재귀 stroke 하단인 86px까지 확장되고, 같은 노드에
nonterminal을 지정하면 80px를 유지하는 계약을 고정한다.

## 5. 실제 fixture 래칫

- 물리 2쪽에 제2호와 폭 500px 이상의 종료 bottom 선이 미리 나오지 않는다.
- 물리 3쪽 종료 bottom `Line`의 stroke 하단이 모든 `clip=true` 조상 TableCell 안에 있다.
- SVG의 outer `cell-clip` 하단과 bottom stroke 하단이 모두 `827.273px`다.
- 기존 #4069 계약인 17쪽, 2·3쪽 cursor, 10·11쪽 저장 프레임, 15·16쪽 자식 표 이음을 유지한다.

## 6. 집중 검증 결과

| 검증 | 결과 |
| --- | --- |
| 수정 전 실제 구조 래칫 | RED, `827.273 > 824.880` 재현 |
| `cargo test --lib issue_4159_` | PASS, 2 passed |
| release-test + Native Skia `--lib issue_4159_` | PASS, 2 passed |
| release-test `issue_2007_nested_cell_pagination` | PASS, 6 passed |
| focused Clippy, `cargo fmt --check`, `git diff --check` | PASS |
| `rhwp-studio: npx tsc --noEmit` | PASS |
| 표준 release WASM build | PASS, compile·wasm-bindgen·wasm-opt·`pkg` packaging 완료 |
| `npm run e2e:issue-4159` | PASS, 17쪽 + 종료선 픽셀 1,196/1,203 |
| `npm run e2e:issue-536` | PASS, 인접 물리 2쪽 6개 계약 |
| `npm run e2e:manifest-check` | PASS, tracked 86개 / manifest 86행 |

## 7. 시각 증적

`output/4159/`:

- `hancom-p003.png` — 한컴 2020 기준 물리 3쪽
- `issue2007_p003_bottom_border_canvas2d.png` — 수정 후 새 WASM Canvas2D 물리 3쪽
- `issue2007_p003_bottom_border_crop.png` — 종료 수평선 픽셀 crop
- `render_tree_003_fixed.json` — 브라우저 PageLayerTree
- `render-tree-fixed/render_tree_003.json` — native PageRenderTree
- `svg-fixed/issue2007_nested_cell_pagination_42065_003.svg` — 수정 후 SVG

한컴 기준과 수정 후 Canvas2D 모두 좌·중·우 세로선과 표 전체 bottom 선이 닫혀 있다.

작업지시자 rhwp-studio 시각 판정도 통과했다. 물리 2쪽의 사각형 숫자가 정상 출력되고,
물리 3쪽의 표 하단 테두리가 전체 너비로 정상 출력되는 것을 확인했다.

## 8. 전체 PR 검증

작업지시자 승인 뒤 `local_validation.md` 4.3의 전체 게이트를 순서대로 실행했다.

| 검증 | 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo build --release` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --release --lib` | PASS, 3,305 passed / 10 ignored |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | PASS |
| Native Skia `skia --lib` | PASS, 58 passed |
| Native Skia `issue_2225_missing_picture_placeholder` | PASS, 2 passed |
| Native Skia `render_p37_direct_pdf_export` | PASS, 4 passed |
| `cargo fmt --check`, `git diff --check` | PASS |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --doc` | PASS, 4 passed / 2 ignored |
| `rhwp-studio: npx tsc --noEmit` | PASS |
| `rhwp-studio: npm test` | PASS, 802 passed |
| 새 release WASM build | PASS, compile·wasm-bindgen·wasm-opt·`pkg` packaging 완료 |
| 새 WASM의 #4159 / #536 E2E | PASS |
| `npm run e2e:manifest-check` | PASS, tracked 86개 / manifest 86행 |

검증 후보를 원본 저장소 branch에 push하고 `devel` 대상 Open PR
[#4174](https://github.com/edwardkim/rhwp/pull/4174)를 생성했다. #4159 comment·close와 PR merge는
아직 수행하지 않았다.

PR 생성 승인 뒤 원격 `devel`이 `23ff5b6f1`까지 전진한 것을 확인했다. 이를 merge commit
`e6f09003b`으로 통합한 새 code head에서 위 전체 게이트를 다시 실행했으며 모두 통과했다.
PR 고유 diff는 9개 파일, 589 additions / 2 deletions로 유지되고 `upstream/devel`은 새 head의
조상이다.
