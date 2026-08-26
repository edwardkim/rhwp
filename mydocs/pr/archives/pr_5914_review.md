---
kind: pr-review
status: code-ci-success-review-only-fast-pass-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5914 검토 - rowspan 선언 높이 축소

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5914](https://github.com/edwardkim/rhwp/pull/5914) / `@kevin9327` |
| 관련 issue | #5910 |
| source head | `9fb1e199b1349f61bf25f6555cc6020bfec0c2d4` |
| 작성 시점 참고 상태 | non-draft, mergeable `true`, `blocked`; 원 PR Archive B 실패 이력 있음 |
| 통합 적용 | `59c4c6a38`, `40423b836` |
| 선행 의존성 | #5913의 p122 보정 뒤 같은 integration candidate에서 검증 |
| 통합 code candidate | [#5954](https://github.com/edwardkim/rhwp/pull/5954) `046e7da61fcb6b529f0f96b9f492c037f2abf579` |

## 변경 검토

- `rowspan_declared_overflow_shrink`은 병합 셀 선언 높이가 걸친 단일 행 선언 합보다 작은 경우, 마지막
  걸침 행이 차이를 흡수하도록 계산한다. 저장된 `common.height`가 축소 후 행합을 정확히 확인할 때만
  적용하므로, 0 높이 등 손상 선언에는 기존 동작을 유지한다.
- HeightMeasurer와 LayoutEngine에 같은 바닥 규칙을 적용해 content height 아래로는 줄이지 않으며,
  row cut 회계도 줄어든 선언 높이를 읽도록 맞췄다. 이 세 경로가 다르지 않으면 kps-ai의 rowspan 묶음이
  다시 다음 쪽으로 밀릴 수 있다.
- `samples/kps-ai.hwp`의 한글 2020 metadata 기준 PDF는 77쪽이고, candidate의 `rhwp info`도 77쪽이다.
  원래 78쪽이던 page count 및 43쪽 표의 마지막 rowspan 행이 기준과 맞게 닫힌다.
- #5910으로 이후 페이지가 한 칸 당겨져 기존 #1073/#4698 분할 계약의 0-based page index를 65/66에서
  64/65로 고쳤다. 이 test 보정은 기능 범위 확대가 아니라 77쪽 정정의 직접 후속이다.

## 로컬 검증

- `rowspan_declared_overflow_shrink`, #1073 nested table split, #4698 fragment ownership focused gate를
  통과했다. #4698은 64쪽에 `민간/소프트웨어`, 65쪽에 `시장침해/가능성`이 배분됨을 확인한다.
- 통합 candidate 전체 nextest는 8,208 passed / 41 skipped였다. Native-Skia lib 3,949 passed / 13 ignored,
  Native-Skia fixture 2개, locked WASM build, fmt, clippy, doctest도 통과했다.
- Docker CLI가 없어 표준 Docker WASM은 실행하지 못했고 locked WASM wrapper 성공으로 대체했다.
  파생 `tests/generated/regression_suite_*` manifest drift는 현재 devel 문제로 이번 변경 범위에서 제외했다.

## 시각 증적

- 한글 2020 기준 `pdf/kps-ai-2022.pdf`와 `samples/kps-ai.hwp`를 45ㆍ46ㆍ65ㆍ66쪽에서 비교했다.

  - 원본 fixture SHA-256: `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`
  - 기준 PDF SHA-256: `7c064fd290368369a3c8eaa7d7b03668c46fb4dfe0fc18ba52d00456ffe01d28`

  ```bash
  venv/bin/python scripts/visual_sweep.py \
    --key pr5914-kps-ai --hwp samples/kps-ai.hwp --pdf pdf/kps-ai-2022.pdf \
    --pages 45-46,65-66 --rhwp-bin target/pr-review/release-test/rhwp \
    --out output/review-kevin9327-20260823/kps-ai
  ```

- 네 페이지 모두 page count, frame overflow, cell tail overflow, text band clip의 blocker 후보는 0건이다.
  사람 검토에서 46쪽 평가기준 표의 rowspan 경계와 66쪽 연속 조각의 `시장침해 가능성` 행이 기준 PDF와
  같은 쪽 흐름에 놓인다. 글꼴 raster 차이는 overlay 수치에 남지만 표 구조 수용 판단을 뒤집는 clipping이나
  page shift는 없었다.
- 보존 asset:
  `mydocs/pr/assets/pr_5914_kps_ai_p46_review.png`
  (`sha256:bf731a515e803ccc579dd7275593e2cbc4068f974c20772ba22361514a95d231`),
  `mydocs/pr/assets/pr_5914_kps_ai_p66_review.png`
  (`sha256:5e6308e9525f737e20aee006eb201fd14aa0fb60cdb726bea086925bfbae4e44`),
  `mydocs/pr/assets/pr_5914_kps_ai_visual_sweep_summary.json`
  (`sha256:62d06325eda30a9849a0cb6ba60e1281c3b9ba7ac0542f320a9b9f2fff19317a`).

## 판정

## CI와 최종 조건

- 통합 code candidate `046e7da61`의 GitHub Full CI(Build & Test, lint, Native Skia, build archive와
  모든 test shard), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/32646702946),
  [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/32646702916), Proptest 및 Adapter
  inter-diff가 모두 성공했다. 이는 review-only trailing commit 이전 code head의 검증 결과다.
- `upstream/devel@5057a7fcaf055b928e76115cdee4bc20bf0936f9`과의 merge-tree는
  `8f91cf6909e0ed64c005769e04411fc404770600`으로 충돌 없이 생성됐고 `git diff --check`도 통과했다.
- merge 전에는 trailing 문서 head의 fast-pass aggregate, `MERGEABLE/CLEAN`, #5914 source head 재확인,
  작업지시자 승인을 다시 확인한다.

**수용 권고.** 원 PR의 과거 Archive B 실패는 #5910 뒤쪽 page index가 바뀌어 생긴 test 기대값 불일치였으며,
위 계약 보정과 현재 로컬 검증으로 해소했다. 같은 candidate의 #5913도 TAC 그림 `PAPER` 크기 기준 메인터너
보정과 p122 기준 PDF 재검증으로 visual fidelity 보류가 해소됐으므로, 두 PR을 분리하지 않고 함께 진행할 수 있다.
