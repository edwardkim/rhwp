# PR #2140 검토 — C1d 라인 누적/표식 렌더 (#1431 Track C)

- PR: https://github.com/edwardkim/rhwp/pull/2140
- 이슈: #2129 (`Refs`, 자동 close 아님), 상위 트래킹 #1431 Track C
- 작성자: `johndoekim` (기존 기여자)
- 작성일: 2026-07-10
- 리뷰 기준: GitHub `baseRefOid=595f1a486a32503149698615727bd634293d840c`,
  `headRefOid=66660e99c2ba6703bd5067c4cbb4c0cf10403dcc`
- merge: 2026-07-10, merge commit `38d1b5a2d7a7b6f068f6d436b94066dcf48497dd`
- 후속 기록 방식: 옵션 2, 원 코드 PR merge 후 docs-only review/asset PR 분리

## 결론

**blocking finding 없음. approve 및 merge 완료.**

#2129 C1d 범위인 라인 차트의 stacked/percentStacked 누적 렌더와 plot-level marker 렌더는
파서/모델/렌더러/통합 테스트/시각 sweep 기준으로 의도와 맞다. PR 본문에 적힌 범례 순서 역전,
스와치/세부 스타일 fidelity, `line3DChart`, 콤보 라인 누적 등은 범위 외 C2 잔차로 남기는 판단도
타당하다.

주의: 로컬 `upstream/devel`은 한때 GitHub PR base보다 앞선 커밋을 가리켜 PR 외 diff가 섞여 보였다.
이번 검토는 GitHub PR의 실제 `baseRefOid`인 `595f1a486...` 기준으로 수행했다.

## 코드 검토

- `src/ooxml_chart/mod.rs`
  - `line_grouping`, `line_markers`를 별도 필드로 추가해 bar/line combo에서 `grouping` 공유 오염을
    피한다.
- `src/ooxml_chart/parser.rs`
  - `c:grouping`을 현재 plot type에 따라 Column/Bar는 기존 `grouping`, Line은 `line_grouping`에
    저장한다.
  - plot-level `<c:marker val="1|true"/>`만 `line_markers`로 채택하고, series 내부 `c:marker`
    래퍼는 배제된다.
- `src/ooxml_chart/renderer.rs`
  - `render_line`이 `line_grouping`에 따라 독립/누적/100% 누적을 분기한다.
  - stacked 축은 카테고리별 양수 누적합을 기준으로 `nice_axis(0, max_sum, ...)`를 사용하고,
    percentStacked는 0-100% 축으로 고정한다.
  - 라인 점 x 좌표는 카테고리 슬롯 중앙을 사용한다.
  - `line_markers`가 켜진 경우 계열별 마커를 각 점에 렌더한다.
- `tests/issue_2129_line_stacked.rs`
  - 라인 5종 x hwp/hwpx 샘플을 대상으로 placeholder 제거, 축 라벨, marker 수, 무회귀를 고정한다.

## 검증

GitHub CI는 모두 통과했다. WASM Build는 workflow 조건상 skip이며 CI preflight는 pass다.

로컬 검증:

- `cargo fmt --check` pass
- `git diff --check 595f1a486a32503149698615727bd634293d840c...local/pr2140` pass
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib ooxml_chart -- --nocapture` pass
  - 72 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2129_line_stacked -- --nocapture`
  pass
  - 6 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1882_chart_style_gaps
  --test issue_1431_scatter --test issue_1453_chart_3d_ofpie_routing -- --nocapture` pass
  - 4 + 1 + 2 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - 전체 test command exit 0
  - 신규 `issue_2129_line_stacked` 6/6 및 기존 차트/회귀 suite 포함

## 시각 검증

기준 PDF는 이미 저장소의 `pdf/chart/라인/*-2022.pdf`에 존재해 MCP 변환은 사용하지 않았다.
`scripts/task1274_visual_sweep.py`로 라인 5종 x hwp/hwpx 10개를 모두 비교했다.

결과 요약:

| target | pages | flagged | pixel match | visual proxy |
|---|---:|---:|---:|---:|
| plain hwpx/hwp | 1/1 | 0 | 98.87188 | 20.35629 |
| marker hwpx/hwp | 1/1 | 0 | 98.80392 | 19.60651 |
| stacked hwpx/hwp | 1/1 | 0 | 98.83779 | 18.71519 |
| stacked marker hwpx/hwp | 1/1 | 0 | 98.77252 | 18.32090 |
| percent stacked hwpx/hwp | 1/1 | 0 | 98.73618 | 16.40828 |

대표 증적:

- `mydocs/pr/assets/pr_2140_line_stacked_marker_hwpx_review.png`
  - sha256 `97678f1cb5d1c41f66e46947ef8d7275a1e3a3db5bffc7546899455dad0b79b6`
- `mydocs/pr/assets/pr_2140_line_percent_hwpx_review.png`
  - sha256 `74c349951e10e6a080fb89744f702bbf5cb562b46ce0ee3816991d75c2ee757a`
- `mydocs/pr/assets/pr_2140_line_marker_hwpx_review.png`
  - sha256 `9d9516efe8439f13e8f30d545fceeeffc0d91464f0dcfc1cd3d0cbb7f5e178ee`

시각적으로 누적/100% 누적 축과 라인 기하, 표식 렌더는 확인된다. 범례 순서와 세부 스타일 차이는
PR 본문에 기록된 C2 잔차로 보며, 이번 PR의 merge blocker로 보지 않는다.

## 처리안

PR #2140은 approve 후 merge 완료했다. #2129가 `Refs`만 걸려 있으므로, 후속 docs-only PR merge 후
maintainer 판단으로 #2129에 C1d 처리 완료 코멘트를 남기고 close할지 별도 결정한다. #1431은 Track C
잔여 축이 있으므로 이 PR만으로 close하지 않는다.
