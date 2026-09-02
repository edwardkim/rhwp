---
kind: pr-review-implementation
status: mixed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
---

# Kevin PR #6398-#6446 체리픽 통합 검토 기록

## 범위와 기준

- review branch: `review/kevin9327-ci-green-20260830-complete`
- code candidate head: `dbbb8d8b2` (문서/증적 커밋 전)
- base: `upstream/devel@bd78a53122e4b532eeee330b2788cbc858dad2b0` (#6466)
- 포함 원 PR: #6398, #6399, #6400, #6401, #6402, #6404, #6405, #6406, #6407, #6410, #6411, #6414, #6416, #6418, #6419, #6421, #6425, #6426, #6429, #6433, #6436, #6437, #6446.
- 제외 원 PR: 사용자 지시에 따라 #6440, #6415, #6432, #6438, #6444 및 `oracle_pair_index.py`가 포함된 PR이다. 이 문서 이후 새로 열린 PR은 추가하지 않는다.
- reviewer `jangster77` 지정은 원 PR별로 완료돼 있다. 현재 source PR들은 Open/non-draft, head check `SUCCESS`이지만 `mergeStateStatus=UNKNOWN`은 통합 전 원 PR에 대한 작성 시점 참고값일 뿐 merge 보장은 아니다.

## 적용과 메인터너 보정

| 원 PR | source head | 통합 branch 최종 적용 commit |
| --- | --- | --- |
| #6398 | `d2bd1ef` | `3285e00` |
| #6399 | `cbb6117` | `4b3cfad` |
| #6400 | `8c6f266` | `a1991a3` |
| #6401 | `2d225cd` | `7e66506` |
| #6402 | `c084151` | `cd5fcec` |
| #6404 | `2956139` | `3fdadea` |
| #6405 | `c4b866b` | `32f87b4` |
| #6406 | `e109b21` | `c52f8e7` |
| #6407 | `bf6858e` | `fbddb00` |
| #6410 | `22f8527` | `ba1f35b` |
| #6411 | `b5ead38` | `d8adf77` |
| #6414 | `6786f96` | `3e21e7f` |
| #6416 | `9a3fca7` | `b577292` |
| #6418 | `245c176` | `cdd64c8` |
| #6419 | `73698f6` | `916206a` |
| #6421 | `2a3301f` | `429636f` |
| #6425 | `fd7680d` | `480614f` |
| #6426 | `7aced56` | `30b162e` |
| #6429 | `2070fb7` | `103b4b5` |
| #6433 | `8aea643` | `e442e14` |
| #6436 | `f2d95f4` | `662d9a6` |
| #6437 | `b8df11e` | `dab4d1f` |
| #6446 | `ac91035` | `3628324` |

- 기준선을 #6466으로 rebase했다. #6436은 정확한 fixture PDF를 새로 산출한 뒤 다시 포함했다. `dbbb8d8`은 `cargo fmt`가 요구한 #6436 import wrapping만 정리한 메인터너 보정이다. `upstream/devel`은 code candidate의 조상이며 `git diff --check upstream/devel...HEAD`도 통과했다.

## 완료한 검증

- rebase 전 동일 code candidate에서 `cargo fmt --all -- --check`, workspace clippy, wasm lib clippy, workspace build, wasm-pack, `npm test` (1,316 passed, 1 skipped), Studio build/responsive/password E2E가 통과했다.
- #6436 재포함 및 메인터너 포맷 보정 뒤 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 다시 실행해 8,763 passed, 3 slow, 43 skipped, 424.788초로 통과했다.
- Node workflow contract 115건, Python workflow contract 63건, suite manifest 및 tier check도 통과했다.
- #6466은 canonical PDF selection, tools, PDF/문서 변경이며 renderer Rust source를 변경하지 않는다. 사용자가 rebase 후 중복 회귀를 수행하지 말라고 지시했으므로 위 결과를 다시 실행하지 않았다.

## 시각 증적과 개별 판정

| PR | evidence | 권고 |
| --- | --- | --- |
| #6398 | HWP 2020/HWPX 2024 canonical PDF, p1 sweep 후보 0 | 범위 제한 수용 |
| #6399 | HWPX 2020 canonical PDF, p1-6 sweep 후보 0 | 수용 |
| #6400 | HWPX 2020 canonical PDF, p1-2 sweep 후보 0 | 수용 |
| #6425 | exact ZOOM input은 test 내부 합성, 장기 fixture 없음 | 증적 한계 수용 |
| #6436 | Hancom 2024 exact PDF, p1 sweep 후보 0, 표 band-본문 gap `27.2px`, 전체 nextest 통과 | 수용 |

- #6398/#6399/#6400/#6436의 info JSON, sweep summary, 직접 확인한 review PNG는 `mydocs/pr/assets/pr_6398_*`, `pr_6399_*`, `pr_6400_*`, `pr_6436_*`에 보존했다. #6436의 2페이지 exact PDF는 `pdf/issue6312/tab_host_own_line-2024.pdf`에 보존했다. 이 호스트에는 `google-chrome`이 없어 허용된 `rsvg` rasterizer를 사용했다. 글꼴 차이에 따른 ink match는 fidelity pass/fail 근거로 사용하지 않았다.

## Merge 후 증적 comment 계획

- visual 판단을 쓴 원 PR #6398, #6399, #6400, #6436에는 `devel` 반영 후에만 contributor comment를 게시한다. 각 개별 review의 `Merge 후 contributor PR comment 계획`에 대상 페이지, 후보 수, pixel/proxy 수치, 직접 확인 결론과 raw URL 템플릿을 기록했다.
- comment는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 direct link로 포함하고, 수치가 자동 일치율 보조값임을 명시한다. representative PNG는 branch URL이 아니라 실제 통합 merge SHA의 `raw.githubusercontent.com` URL로 표시한다.
- 통합 PR 번호, merge SHA, CI final aggregate를 확정하기 전에는 comment를 게시하지 않는다. 게시 시에는 UTF-8 without BOM `--body-file`을 사용하고, API 재조회로 literal `\\n` 또는 한글 인코딩 훼손이 없는지 확인한다.

## 다음 단계

1. 포함 PR만 새 `upstream/devel`에서 다시 source head/CI/mergeability를 확인해 통합 PR을 준비한다.
2. merge 뒤 원 contributor PR close comment, archive/오늘할일 갱신, branch/worktree 정리는 `post_merge.md`를 따른다. 이 단계는 아직 실행하지 않았다.
