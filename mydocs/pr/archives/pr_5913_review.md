---
kind: pr-review
status: code-ci-success-review-only-fast-pass-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5913 검토 - p122 저장 vpos 쪽 경계

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5913](https://github.com/edwardkim/rhwp/pull/5913) / `@kevin9327` |
| 관련 issue | #5907 |
| source head | `7c7e51594a1dd3179721951a8a157396f02d27c9` |
| 작성 시점 참고 상태 | non-draft, mergeable `true`, `blocked`; 원 PR Archive B/C 실패 이력 있음 |
| 통합 적용 | `21c55a882` (`7c7e515` 체리픽) |
| 메인터너 보정 | `da2a3123c`, `046e7da61` |
| 통합 code candidate | [#5954](https://github.com/edwardkim/rhwp/pull/5954) `046e7da61fcb6b529f0f96b9f492c037f2abf579` |

## 변경과 보정 근거

- 원 변경은 `p122.hwp`의 연속 문단이 모두 저장 vpos 0을 주장하는 충돌을 쪽 경계로 읽어, 한글 2022
  정본과 같이 1쪽에서 3쪽으로 바로잡는다. `TypesetEngine`과 paginator 폴백에 같은 판정을 두고
  `p122_stored_vpos_page_break` 계약을 추가했다.
- 원 PR의 넓은 guard는 일반 텍스트 문단의 vpos 0 연쇄까지 쪽 경계로 해석했다. 원 CI의
  `outline_navigation_table_cell_number` 2건과 `issue_1510` 3건을 각각 45쪽, 32쪽으로 늘리는
  회귀를 로컬에서 재현했다.
- 메인터너 보정은 양쪽 문단이 가시 텍스트가 없고 직전 문단이 control anchor인 경우만 허용한다.
  p122의 빈 SectionDef/ColumnDef - 그림 - 빈 문단 전환은 유지하면서, 일반 문단과 빈 SectionDef에서
  시작하는 일반 본문 사이의 오탐을 차단한다. 이 보정이 필요한 이유와 재현 결과는
  [통합 구현 기록](pr_5913_5914_review_impl.md)에 남긴다.
- 추가 검토에서 `treat_as_char` 그림 경로가 HWP5 `PAPER`/`PAGE` 크기 기준을 일반 HWPUNIT으로
  해석함을 확인했다. 따라서 `42520`을 42.52mm로 축소해 p122 2쪽 그림 전체를 표시하고 있었다.
  메인터너 보정은 인라인 그림도 `resolve_object_size`로 기준 영역의 1/100%를 해석하도록 통일했다.
  그림을 실제 크기로 확대한 뒤 PDF처럼 쪽 바깥을 clip하는 동작이 이 PR의 user-visible 요구와 맞는다.

## 로컬 검증

- 현 보정의 `p122_stored_vpos_page_break` focused gate: 3 passed, 132 skipped.
- 전체 Rust 회귀:
  `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - 8,208 passed, 41 skipped.
- Native-Skia lib gate도 같은 작업 트리에서 exit code 0으로 통과했다.

## 시각 증적

- 원본 `samples/p122.hwp`와 한글 2022 기준 `pdf/p122-2022.pdf`를 대상으로, 다음 명령으로 1-3쪽을
  비교했다.

  - 원본 fixture SHA-256: `b4d66baa093d564ac728c84d591b3ea7d6bc4cfef160bde81ee13afc6d927e82`
  - 기준 PDF SHA-256: `379cbf8c5b571be12b123180ceba7b266d43f54c84f6cd0b660c693317fc90cc`

  ```bash
  venv/bin/python scripts/visual_sweep.py \
    --key pr5913-p122-maintainer --hwp samples/p122.hwp --pdf pdf/p122-2022.pdf \
    --pages 1-3 --rhwp-bin target/pr-review/release-test/rhwp \
    --out output/review-kevin9327-20260823/p122-maintainer
  ```

- 실제 재실행 key는 `pr5913-p122-maintainer`다. candidate와 기준 PDF는 모두 3쪽이며, 1ㆍ3쪽 공백
  페이지의 overlay는 각각 100% 일치했다. 2쪽은 pixel match 99.73398%, ink match 99.64853%였다.
  전체 평균 pixel match는 99.91133%, visual accuracy proxy는 99.88284%이며, 구조ㆍflow heuristic
  blocker는 0건이다.
- 2쪽 SVG의 그림은 본문 상단 y=132.27px에 놓였고, `PAPER` 425.20% x 222.38%를 해석한
  3374.84 x 2496.23px 크기로 렌더됐다. 기준 PDF와 마찬가지로 확대된 그림의 쪽 안쪽 영역만
  보이므로, 이전의 축소된 전체 그림 차이는 남지 않았다.
- 보존 asset:
  `mydocs/pr/assets/pr_5913_p122_p2_review.png`
  (`sha256:37a4498a043aa12b24babc3978943d99d7eb661a4683ceb251f69b47cf6fbd16`),
  `mydocs/pr/assets/pr_5913_p122_visual_sweep_summary.json`
  (`sha256:180738e4480b261309a0955e4143a75b79950079422b75528170e115c48ab6ca`).

## 판정

## CI와 최종 조건

- 통합 code candidate `046e7da61`의 GitHub Full CI(Build & Test, lint, Native Skia, build archive와
  모든 test shard), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/32646702946),
  [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/32646702916), Proptest 및 Adapter
  inter-diff가 모두 성공했다. 이 결과는 review-only trailing commit 이전 code head의 사실이다.
- `upstream/devel@5057a7fcaf055b928e76115cdee4bc20bf0936f9`과의 merge-tree는
  `8f91cf6909e0ed64c005769e04411fc404770600`으로 충돌 없이 생성됐고 `git diff --check`도 통과했다.
- 이 trailing 문서 head는 `mydocs/`와 review asset만 추가한다. merge 전에는 이 새 head의 fast-pass
  aggregate, `MERGEABLE/CLEAN`, #5913 source head 재확인, 작업지시자 승인을 다시 확인한다.

**수용 권고.** p122 vpos 쪽 경계와 TAC 그림의 `PAPER` 크기 해석을 현재 candidate에서 재현ㆍ고정했고,
전체 Rust 회귀, Native-Skia gate, 한글 2022 기준 PDF 시각 sweep 및 exact code head GitHub CI를 통과했다.
