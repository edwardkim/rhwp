---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6412
author: jeong-sik
---

# PR #6412 review - 저장 사다리 셀 문단 압축

## 검토 기준

- 원 PR head: `ba045cdc0a47da643188752782172502e8641412`
- 통합 적용 commit: `810db7289`
- 기준 base: `upstream/devel@19b89d967b1505cd4bdcdbba7d1f1413f32a1505`
- 작성 시점 원 PR 상태: Open, non-draft, `MERGEABLE/CLEAN`. 원 head의 Build & Test, CodeQL,
  Render Diff 및 Native Skia check는 성공 상태였다. 최종 통합 PR 생성과 merge 직전에 다시 확인해야 한다.

## 변경과 로컬 검증

- 저장된 vertical-position 사다리를 따르는 셀 문단에서 한 줄 폭이 cell inner width를 넘을 때만 압축을
  허용하고, `issue_6389_cell_stored_ladder_compresses_to_fit`으로 이 조건을 고정한다.
- CI-style suite 등록을 준비한 뒤 `node scripts/run-rust-test.mjs
  issue_6389_cell_stored_ladder_compresses_to_fit -- --cargo-profile release-test --target-dir
  target/pr-review --no-fail-fast`를 실행해 `1 passed`를 확인했다.
- 변경한 `text_overlap_baseline`의 16 partition을 같은 release-test target에서 실행해 `16/16 passed`
  (가장 긴 partition 9는 31.299초)를 확인했다.

## 시각 증적

- 원본: `samples/2025 행정업무운영 편람(최종).hwp`, SHA-256
  `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`.
- `rhwp info --json`은 `lastSavedWith.product=hancom-office-2024`,
  `version=13.0.0.3622`를 보고했다. 따라서 기준 PDF
  `pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf` (SHA-256
  `34db2aeefa4ae00b38c464571e7e17eef375ffd3ec29eb6d89ddcd67b63bb670`)를 사용했다.
- [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)를 따라
  p68만 `rsvg` rasterizer로 비교했다. 완료 page 1, 자동 후보 0,
  pixel match 90.60603%, ink/visual-accuracy proxy 77.33644%였다.
- 임시 산출물은 `output/visual_sweep_non_kevin_20260831_rsvg_2024/pr6412-issue6389-p68`이고, 장기 증적은
  [info JSON](../assets/pr_6412_issue6389_info.json),
  [metrics JSON](../assets/pr_6412_issue6389_p68_visual_sweep_metrics.json),
  [검토 요약 JSON](../assets/pr_6412_issue6389_p68_visual_sweep_summary.json),
  [대표 PNG](../assets/pr_6412_issue6389_p68_review.png)이다.
- 대표 PNG를 직접 열어 확인했다. 한국어 glyph와 도구 라벨은 판독 가능했고, 대상 표의 셀 내용이 cell
  boundary 밖으로 뚫고 나가는 blocker는 보이지 않았다. 낮은 ink proxy는 `rsvg`와 기준 PDF의 글꼴/글리프
  차이를 포함하므로 전체 fidelity 통과 수치로 사용하지 않았다. Chrome rasterizer는 이 호스트에 없었다.

## 판단

**수용 권고.** 변경 경로의 구조 assertion, baseline partition, Hancom 2024 기준 p68 직접 비교가 모두
대상 주장과 맞는다. 다만 통합 branch에 code/test 보정이 포함되므로 최종 통합 PR의 최신 head Full CI와
mergeability를 통과 조건으로 둔다.

## Merge 후 contributor PR comment 계획

- `devel` 반영 뒤에만 Visual Sweep 정본 direct link와 p68의 후보 0, pixel/proxy 수치, 위 사람 판정과
  `rsvg` 한계를 함께 게시한다.
- 대표 이미지는 다음 merge SHA 고정 URL로 표시한다. 아직 merge SHA가 없으므로 지금 게시하지 않는다.

  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6412_issue6389_p68_review.png`

- 실제 게시에는 UTF-8 without BOM `--body-file`을 쓰고 API 재조회로 한글과 줄바꿈을 확인한다.
