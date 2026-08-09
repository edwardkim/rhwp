---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 45 — 전체 회귀 SVG snapshot 실패 분리 분석

## 발생 및 판정

Stage 44의 현재 source로 전용 target에서 자연 종료까지 실행한

```sh
CARGO_TARGET_DIR=target/task-3820-3821-fidelity \
  CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

은 마지막 `tests/svg_snapshot.rs`에서 8개 중 7개 mismatch로 **실패**했다. 따라서 focused
`issue_2007_nested_cell_pagination` 9/9, `issue_1921_59043_pagination_pin` 4/4,
`issue_rowbreak_chart_overlap` 20/20, Stage 42 `overflow_cell_lines_do_not_grow`
(674 fixture, non-zero 21문서/709줄) 통과와 별개로 현재 worktree 전체 integration은 통과
상태가 아니다.

실패 항목은 다음과 같다.

| golden | actual 증적 |
| --- | --- |
| `form-002/page-0` | `tests/golden_svg/form-002/page-0.actual.svg` |
| `issue-147/aift-page3` | `tests/golden_svg/issue-147/aift-page3.actual.svg` |
| `issue-157/page-1` | `tests/golden_svg/issue-157/page-1.actual.svg` |
| `issue-267/ktx-toc-page` | `tests/golden_svg/issue-267/ktx-toc-page.actual.svg` |
| `issue-617/exam-kor-page5` | `tests/golden_svg/issue-617/exam-kor-page5.actual.svg` |
| `issue-677/bokhakwonseo-page1` | `tests/golden_svg/issue-677/bokhakwonseo-page1.actual.svg` |
| `table-text/page-0` | `tests/golden_svg/table-text/page-0.actual.svg` |

`.actual.svg`는 test harness가 mismatch 분석을 위해 생성한 임시 비교 산출물이다.
`UPDATE_GOLDEN=1`은 raster/XML 구조 대조 없이 사용하지 않고, 판정과 갱신이 끝나면
`.actual.svg`는 tracked 증적에 포함하지 않는다.

## Stage 44와의 관계

Stage 44가 고정한 42065 p10/p11의 source owner는 현재 RenderTree에서 정확하다. 이번 실패는
그 테스트보다 나중에 수행된 전역 SVG byte snapshot gate이며, 서로 다른 7 fixture를 동시에
포함한다. 따라서 #2007 보정을 되돌려 golden을 맞추는 것은 금지한다.

현재 branch의 commit `0ec8a14eb`은 `src/renderer/mod.rs`에서 첫 font family도 작은따옴표로
감싸도록 바꿨지만 SVG golden은 갱신하지 않았다. Stage 42/43 table pagination 변경과 이 직렬화
변경을 분리해 판정한다.

## 판정 절차

1. expected/actual SVG를 정규화하지 않은 원문 diff와 page raster diff로 나눠, byte-only 차이와
   실제 layout/ink 차이를 분리한다.
2. 각 fixture의 RenderTree bbox/text line을 비교해 table pagination 변경이 영향을 준 노드와
   CSS font-family serialization 변경을 구분한다.
3. 실제 layout 변화가 Hancom PDF 또는 기존 fixture contract와 일치하는 경우에만, 변경 근거와
   최소 갱신 범위를 별도 문서에 기록한 뒤 golden 갱신을 검토한다.
4. layout 차이가 원본 계약을 깨면 golden을 갱신하지 않고 원인 코드만 좁혀 수정한다. 코드 수정 뒤에는
   affected snapshot을 먼저 실행하고, 그 다음 전체 integration gate를 실행한다.

## 현재 금지 사항

- 원인 판정 전 `.actual.svg` 삭제 또는 `UPDATE_GOLDEN=1` 사용
- Stage 42 overflow predicate 및 Stage 43 H5 owner 보정 rollback
- golden의 byte 일치만을 목표로 한 renderer 정책 변경

## 1차 판정 — 7건 모두 visual-equivalent serialization drift

각 expected/actual SVG를 librsvg로 같은 2,381×3,367 raster로 변환하고 ImageMagick absolute-error
metric으로 비교했다. 일곱 fixture 모두 changed pixel은 `0 (0)`이었다.

| golden | raster changed pixel | byte mismatch의 직접 원인 |
| --- | ---: | --- |
| `form-002/page-0` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 |
| `issue-147/aift-page3` | 0 | 첫 font family quoting만으로 정규화 후 동일 |
| `issue-157/page-1` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 |
| `issue-267/ktx-toc-page` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 + body clip 과확장 제거 |
| `issue-617/exam-kor-page5` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 |
| `issue-677/bokhakwonseo-page1` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 |
| `table-text/page-0` | 0 | 첫 font family quoting + 부동소수 문자열 정밀도 |

예를 들어 기존 SVG는 `font-family="맑은 고딕,..."`이고 현재 SVG는
`font-family="'맑은 고딕',..."`이다. 일반 Korean family에서는 같은 computed font를 선택한다.
Raster 0-pixel 결과는 이 seven golden에서 visual/layout contract가 변하지 않았음을 보인다.

`issue-267`은 body clip 폭도 `673.306666px`에서 `646.046666px`로 바뀌었다. 이는 commit
`25fc978b3`이 이미 clip된 `TableCell` 자손을 body clip 확장 근거에서 제외한 결과다. 현재 visible
paint의 최대 우측은 `720.533px`, 새 clip 우측은 `721.633px`라 1.1px 안쪽에 안전하게 들어가고,
기존 폭은 보이지 않는 자손 때문에 과확장된 값이다.

따라서 Stage 42/43 pagination 변경의 시각 회귀가 아닌, 의도된 CSS serialization 및 clip-owner
개선에 따른 byte golden drift로 분류한다. 다만 golden 갱신 전에 아래 실제 CSS 결함을 먼저
수정한다.

## 추가 결함 — apostrophe가 있는 font family의 CSS 문자열 파손

`issue-677` actual에는 `Tom's Handwriting`이 다음처럼 직렬화됐다.

```text
'Tom's Handwriting','Malgun Gothic',...
```

첫 작은따옴표 안의 apostrophe가 escape되지 않아 유효한 CSS family string이 아니다. 따라서 이
actual을 그대로 승인하지 않았다. `css_single_quoted_font_family()`를 추가해 역슬래시를 `\\`,
작은따옴표를 `\'`로 escape하고, `Tom's Handwriting` 및 `Legacy\Face`를 단위 테스트로 고정했다.
focused renderer test는 1/1 통과했다.

## golden 갱신 및 focused 재검증

위 raster evidence에 근거해 다음 documented harness 명령으로 seven golden을 재생성했다.

```sh
UPDATE_GOLDEN=1 CARGO_TARGET_DIR=target/task-3820-3821-stage44 \
  CARGO_INCREMENTAL=0 cargo test --profile release-test --test svg_snapshot
```

tracked golden은 표의 7개 경로만 변경됐다. 이어 같은 compiled binary를 `UPDATE_GOLDEN` 없이
실행해 `svg_snapshot` **8/8 통과**를 확인했다. 이 결과는 byte contract가 새 CSS escaping 형식과
일치함을 뜻하며, visual-equivalence 판정의 근거는 위 0-pixel raster 비교다. 실패 때 생성된
`.actual.svg` 7개는 임시 산출물이므로 제거했다.

다음 단계는 `target/task-3820-3821-stage44` 전용 target에서
`cargo test --profile release-test --tests`를 처음부터 다시 실행해 최종 종료 상태와 전체 요약을
확인하는 것이다.

## 전체 integration 최종 결과

CSS escape 수정과 golden 갱신 뒤 같은 전용 target에서 다음 순서로 처음부터 재실행했다.

```sh
cargo fmt --all -- --check
git diff --check
CARGO_TARGET_DIR=target/task-3820-3821-stage44 \
  CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

최종 결과는 exit code `0`이다.

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| release-test test summary | 468개 summary / 5,304 passed / 0 failed / 28 ignored |
| `overflow_cell_lines_do_not_grow` | 통과 |
| `svg_snapshot` | 8/8 통과 |
| `visual_roundtrip_baseline` | 3/3 통과 |

ignored 28건은 기존 진단·명시적 opt-in 테스트이며 실패가 아니다. snapshot 실패 때 생성된
`.actual.svg`는 남아 있지 않고, `issue-677` golden은 `Tom\'s Handwriting`을 CSS escape한 문자열을
고정한다. 이로써 Stage 45의 전체 회귀 실패는 해소됐다.
