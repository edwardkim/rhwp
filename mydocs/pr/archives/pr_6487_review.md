---
kind: pr-review
status: maintainer-corrected
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6487
author: planet6897
---

# PR #6487 review - 4성분 JPEG을 RGB PNG로 정규화

## 검토 기준과 메인터너 보정

- 원 PR head: `c4fca0b6899192568a28ca7fe5ceaa4478cd05fb`
- 통합 적용 commits: `8012d5c0df247d619dc10ffef0aa4b9c32de81c1`,
  `c4fca0b6899192568a28ca7fe5ceaa4478cd05fb`
- 메인터너 보정: `e4617834596e6dd86c9102c3db961d156dc3eef1`
  (`보정: CMYK JPEG marker fill 판별 안정화`),
  `160746888a3d42ec48da0f66cbd9af6a5e84039f`
  (`보정: CMYK JPEG 계약을 integration suite로 이동`)
- 통합 기준 base: `upstream/devel@77bcaaa49c89dc12761282c759717188a880064c`
- 작성 시점 원 PR은 Open/non-draft, `MERGEABLE/CLEAN`이며 Build & Test와 CodeQL이 성공했다. merge 직전에
  보정 포함 최신 통합 head의 CI와 mergeability를 확인한다.

원 head의 실제 CMYK fixture는 통과하지만 JPEG은 SOF marker 앞에 `0xFF` fill byte를 반복할 수 있다.
원 구현은 첫 fill byte를 marker로 해석해 유효한 4성분 JPEG을 놓칠 수 있었으므로, 메인터너가 SOI·segment
길이 검증과 fill-byte skip을 추가했다. `FF FF C0` SOF0 minimal header를 4성분으로 판별하는 새
integration contract는 기존 #6310 fixture suite에 둬 source-side `#[cfg(test)]` 기준선은 올리지 않는다.

## 검토와 검증

- `160746888` 뒤 `node scripts/rust-unit-test-tiers.mjs --check`가 source-side 4,221 tests/299 modules로
  통과했고, `node scripts/rust-test-suite-manifest.mjs --check`도 4,643 static test attrs/48 of 48
  integration target으로 통과했다.
- 보정 후 host all-target와 WASM library `cargo clippy ... -D warnings`, `cargo fmt --all -- --check`,
  `git diff --check`가 통과했다. 4성분 marker-fill, HWPX ZOOM, 밑줄, 인용부호 계약을 포함한 focused
  nextest 23건도 통과했다.
- 보정 전 통합 candidate에서 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`를 실행해 8,794 passed, 43 skipped를 확인했다. 이후 보정의
  영향 범위는 JPEG header scan뿐이므로 해당 단위·fixture regression과 lint를 다시 실행했다.
- 2022 저장 HWPX를 2020 기준 PDF `pdf/pr6481-visual/pr6481-issue6310-press-release-cell-logo-2020.pdf`
  (51쪽, SHA-256 `3b6ffbfd48889687076514b0fa367f547b1d73858c63e513277d906ff9562626`)와 1쪽 visual sweep 했다.
  output SVG에는 `data:image/png;base64`가 1개이며, CMYK 로고가 RGB PNG 경로로 정규화됐음을 확인했다.
- 결과: flagged 후보 0건, pixel match 95.39635%, proxy 20.38287%. proxy는 header logo와 전체 layout/font 차이를
  함께 포함해 낮게 산출될 수 있으므로 단독 합격 기준이 아니다. 대표 PNG를 직접 확인해 로고가 한 번만
  표시되고 반복 tile·색 번짐이 없으며 제목/표 frame이 유지되는 것을 확인했다.
- 안정 증적: `mydocs/pr/assets/pr_6487_issue6310_p001_review.png`,
  `mydocs/pr/assets/pr_6487_issue6310_visual_sweep_summary.json`,
  `mydocs/pr/assets/pr_6487_issue6310_overlay_metrics.json`.

## Merge 후 contributor PR comment 계획

- [Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
  링크하고, 원 PR 뒤 `e4617834596e6dd86c9102c3db961d156dc3eef1`의 marker-fill 보정이 포함됐음을 적는다.
- 1쪽, flagged 0건, pixel match 95.39635%, proxy 20.38287%, RGB PNG embed와 사람이 확인한 로고의 단일
  정상 표시, proxy의 한계를 함께 게시한다.
- merge 후
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6487_issue6310_p001_review.png`
  를 표시하고 `--body-file` 게시 뒤 API로 실제 Markdown을 재확인한다.

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 PR의 4성분 정규화 방향과 fixture는 수용 가능하나, 유효한 marker-fill
JPEG을 놓치지 않도록 통합 head에 범위를 제한한 `e4617834596e6dd86c9102c3db961d156dc3eef1` 보정이 필요했다.
두 보정 포함 통합 PR의 최신 CI 성공과 mergeability 재확인이 merge 조건이다.
