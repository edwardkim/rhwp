# PR #1875 리뷰 - 미주 배치/구분선 렌더링 정합

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1875 |
| 제목 | `fix(endnote): 미주 배치·구분선·번호 렌더링 한컴 정합 (3건)` |
| 작성자 | `humdrum00001010` |
| base | `devel` |
| head | `humdrum00001010:endnote-eod-placement` |
| 문서 작성 시점 head | `a85f17ee82334192092f4c92afea43aa44e01f80` |
| 변경 규모 | 9 files, +185/-18 |
| reviewer assign | `jangster77` 지정 완료 |
| merge 결과 | 2026-07-04 merge 완료, merge commit `2b21caef86ac1d3aa8918a30e937bcb7be5f6c18` |

상태값은 문서 작성 시점 참고값이다. merge 전 최신 head, mergeable, required checks 를 재확인했고,
최종 head 기준 GitHub Actions 통과 후 admin merge 했다.

## 변경 범위

- HWPX `endNotePr/noteLine.length`를 `i16`으로 절단하지 않고 `i32`로 보존한다.
- `FootnotePlacement::EachColumn`으로 들어오는 미주 `END_OF_DOCUMENT` 배치를 문서 끝 렌더링으로 연결한다.
- 마지막 구역에서 앞 구역 미주 본문을 문서 순서대로 렌더하고, 참조 위첨자는 원 위치에 둔다.
- 미주 구분선 직전 본문 trailing line spacing을 note 영역에서 제외하고, 첫 미주 본문을 구분선 아래 여백 바닥으로 보정한다.

핵심 코드 위치:

- `src/parser/hwpx/section.rs:908`: `noteLine.length`를 `i32` 원본 값으로 보존한다.
- `src/document_core/queries/rendering.rs:2377`: 앞 구역 `END_OF_DOCUMENT` 미주 본문을 `DeferredEndnote`로 수집한다.
- `src/renderer/typeset.rs:3538`: `Suppress`/`RenderAll`로 미주 본문 렌더 위치를 제어한다.
- `src/renderer/layout.rs:3693`: 구분선 위치와 구분선 아래 본문 floor 보정을 수행한다.

## 렌더 영향 및 visual sweep 판정

이 PR은 `src/renderer/**`, `typeset`, pagination, 구분선 위치, 미주 본문 배치를 수정하므로 visual sweep 대상이다.

PR 본문에는 작성자가 제공한 비교 PNG와 HWPX fixture 링크가 있고, 리뷰 중 한컴 2024 기준 PDF 2건이 로컬에 추가되었다.

기준 PDF:

- `pdf/endnote_end_of_document_fixture-2024.pdf`
  - `Creator: Hwp 2024 13.0.0.3622`
  - `Producer: Hancom PDF 1.3.0.550`
  - 2 pages.
- `pdf/endnote_multi_note_fixture-2024.pdf`
  - `Creator: Hwp 2024 13.0.0.3622`
  - `Producer: Hancom PDF 1.3.0.550`
  - 3 pages.

visual sweep은 기준 PDF가 있는 대표 페이지를 대상으로 수행했다.

## 로컬 검증

로컬 검증은 `pr1875-merge-test` 브랜치에서 수행했다.

- merge 시뮬레이션: `local/pr1875` 기준 `upstream/devel` 병합 시 `Already up to date`, 충돌 없음.
- PR review cargo 전 `target` 하위 항목 삭제 완료.
- `git diff --check`: 통과.
- `cargo fmt --check`: 통과.
- `env CARGO_INCREMENTAL=0 cargo test test_parse_endnote_long_note_line_keeps_hwp5_low_word --lib`: 1 passed.
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1139_inline_picture_duplicate`: 85 passed.
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1082_endnote_multicolumn_drift`: 5 passed.
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과. `svg_snapshot` 8 passed 포함.

외부 fixture 최소 확인:

- `/private/tmp/rhwp-pr1875-fixtures/endnote_end_of_document_fixture.hwpx`
  - `dump-pages`: 2페이지.
  - p1에는 미주 본문이 남지 않고, p2 마지막 구역에 `EndnoteSeparator len=14692344`와 `src=s0:p1:ci0:note0` 미주 본문이 배치됨.
  - `export-svg`: 2개 SVG 생성 성공.
- `/private/tmp/rhwp-pr1875-fixtures/endnote_multi_note_fixture.hwpx`
  - `dump-pages`: 3페이지.
  - p3 마지막 구역에 앞 구역 미주 A/B가 순서대로 배치됨.
  - `export-svg`: 3개 SVG 생성 성공.
- `/private/tmp/rhwp-pr1875-fixtures/endnote_placement_3col.png`
  - PR 작성자 제공 비교 PNG로 확인했지만, 기준 PDF 원본이 없어 프로젝트 visual sweep 기준의 최종 근거로 쓰지는 않는다.

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 수행했다.

2구역 `END_OF_DOCUMENT` fixture:

- 명령: `python3 scripts/task1274_visual_sweep.py --key pr1875-eod --hwp /private/tmp/rhwp-pr1875-fixtures/endnote_end_of_document_fixture.hwpx --pdf pdf/endnote_end_of_document_fixture-2024.pdf --pages 1-2 --out output/pr1875-visual/eod`
- SVG pages: 2.
- PDF pages: 2.
- selected pages: 1, 2.
- flagged: 0/2.
- p1 review: `output/pr1875-visual/eod/pr1875-eod/review/review_001.png`.
- p2 review: `output/pr1875-visual/eod/pr1875-eod/review/review_002.png`.
- p1 `visual_accuracy_proxy_percent`: 27.08907.
- p2 `visual_accuracy_proxy_percent`: 15.85551.
- 최종 asset:
  - `mydocs/pr/assets/pr_1875_endnote_eod_review_p001.png`
  - `mydocs/pr/assets/pr_1875_endnote_eod_review_p002.png`

3구역 다중 미주 fixture:

- 명령: `python3 scripts/task1274_visual_sweep.py --key pr1875-multi --hwp /private/tmp/rhwp-pr1875-fixtures/endnote_multi_note_fixture.hwpx --pdf pdf/endnote_multi_note_fixture-2024.pdf --page 3 --out output/pr1875-visual/multi`
- selected page: 3.
- flagged: 0/1.
- p3 review: `output/pr1875-visual/multi/pr1875-multi/review/review_003.png`.
- p3 `visual_accuracy_proxy_percent`: 16.38899.
- 최종 asset:
  - `mydocs/pr/assets/pr_1875_endnote_multi_review_p003.png`

사람 판정:

- 자동 후보는 0건이다.
- `visual_accuracy_proxy_percent`는 낮지만, 이 값은 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다. fixture의 잉크량이 작고 폰트/glyph 차이가 커서 수치만으로 blocker로 보지 않는다.
- p1: 첫 구역에는 미주 본문이 남지 않고 참조 위첨자만 남는다.
- p2: 구역0 미주 본문이 문서 끝인 두 번째 구역 페이지에 배치되고, 구분선은 기준 PDF처럼 전폭이다.
- p3: 구역0/구역1 미주 A/B가 마지막 구역 페이지에 순서대로 모인다.
- PR의 핵심 주장인 `END_OF_DOCUMENT` 미주 본문 문서 끝 배치, 전폭 구분선, 다중 미주 순서는 기준 PDF와 맞다.

## GitHub CI

merge 전 최신 head `a85f17ee8` 기준 GitHub checks:

- CI preflight: pass.
- Build default-feature tests: pass.
- Native Skia tests: pass.
- Build & Test: pass.
- CodeQL preflight / CodeQL / Analyze rust/python/javascript-typescript: pass.
- Render Diff preflight / Canvas visual diff: pass.
- WASM Build: skipped.

## 리뷰 판단

코드 구현은 특정 샘플명/페이지/임의 계수 하드코딩 없이 문서의 `endnote_shape.placement`, `noteLine.length`, `noteSpacing`, line segment 정보를 사용하고 있어 방향은 적절하다. 로컬 검증과 GitHub CI도 통과했다.

기준 PDF 기반 visual sweep도 핵심 페이지에서 자동 후보 0건이며, 사람 판정 기준으로 PR의 핵심 주장과 일치한다.

따라서 결론은 **merge 가능**이며, 최종 확인 후 admin merge 완료했다.

- merge commit: `2b21caef86ac1d3aa8918a30e937bcb7be5f6c18`.
- review 문서, 기준 PDF, 대표 visual asset은 옵션 2 후속 문서/asset PR 로 분리해 반영한다.
- #1875 PR description 의 GitHub auto-close 대상 이슈는 없었다.

권장 후속:

- 외부 gist fixture를 저장소 테스트 자산으로 반영하거나, `END_OF_DOCUMENT` 미주 본문이 마지막 구역에 모이는 회귀 테스트를 추가하면 장기 회귀 방어가 더 좋아진다. 현재 PR의 merge blocker로 보지는 않는다.

## 코멘트 초안

아래 코멘트는 초안이다. 후속 문서/asset PR merge 후 PR에 게시하기 전 작업지시자 승인을 받는다.

```markdown
검토 및 머지 완료했습니다. 감사합니다.

로컬에서 최신 head(`a85f17ee8`) 기준으로 기본 검증, 관련 미주 회귀 테스트, 한컴 2024 기준 PDF 기반 visual sweep을 확인했습니다.

- `cargo test --profile release-test --tests`: 통과 (`svg_snapshot` 포함)
- `cargo clippy --all-targets -- -D warnings`: 통과
- `issue_1139_inline_picture_duplicate`: 85 passed
- `issue_1082_endnote_multicolumn_drift`: 5 passed
- PR에 링크된 HWPX fixture 2개는 로컬에서 `dump-pages` / `export-svg` 경로를 통과했고, `END_OF_DOCUMENT` 미주 본문이 마지막 구역으로 모이는 것도 확인했습니다.
- 기준 PDF:
  - `endnote_end_of_document_fixture-2024.pdf`: 2 pages, Hancom PDF / Hwp 2024
  - `endnote_multi_note_fixture-2024.pdf`: 3 pages, Hancom PDF / Hwp 2024
- visual sweep:
  - 2구역 fixture p1-p2: flagged 0/2
  - 3구역 다중 미주 fixture p3: flagged 0/1

사람 판정 기준으로도 PR의 핵심 주장인 `END_OF_DOCUMENT` 미주 본문 문서 끝 배치, 전폭 구분선, 다중 미주 순서가 기준 PDF와 맞습니다.

최종적으로 PR의 핵심 주장인 `END_OF_DOCUMENT` 미주 본문 문서 끝 배치, 전폭 구분선, 다중 미주 순서가 기준 PDF와 맞는 것으로 판단해 merge 했습니다.

시각 검증 자료:

![PR #1875 EOD p1 visual sweep](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1875_endnote_eod_review_p001.png)

![PR #1875 EOD p2 visual sweep](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1875_endnote_eod_review_p002.png)

![PR #1875 multi-note p3 visual sweep](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1875_endnote_multi_review_p003.png)

다만 장기 회귀 방어를 위해, 가능하면 후속으로 현재 gist fixture를 저장소 테스트 자산으로 편입하거나 같은 동작을 검증하는 in-repo 회귀 테스트를 추가하면 좋겠습니다.

다음에 페이지 수나 시각 검증이 필요한 PR을 올려주실 때는, 가능하면 한컴 2020 또는 2024에서 저장한 기준 PDF도 함께 첨부해 주세요. 대조 기준이 있으면 review와 회귀 판단을 더 빠르고 정확하게 진행할 수 있습니다.

#1875 자체에는 자동 close 대상 이슈가 없어서 별도 issue close 는 수행하지 않았습니다.
```
