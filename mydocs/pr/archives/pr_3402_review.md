# PR #3402 검토 기록 — 바탕쪽 Para/Column 부동 도형 여백 정합

| 항목 | 내용 |
| --- | --- |
| PR | [#3402](https://github.com/edwardkim/rhwp/pull/3402) — `fix(renderer): 바탕쪽 Para/Column 기준 개체를 본문 여백 안에 배치` |
| 작성자·처리자 | `@kevin9327` (external contributor) · `@jangster77` (collaborator) |
| base / head | `devel` / `pr/task-header-float-horz-margin` (fork: `kevin9327/rhwp`) |
| code candidate | `c1bfe4cb5426a4083a7fc1581ab4e80ac7404b25`; 이 기록 작성 전 최신 `upstream/devel` `52c3bb493`을 merge한 local update head는 `39230e589` |
| 관련 이슈 | 별도 close 대상 이슈 없음 |
| 원 변경 규모 | 4 files, +110 / -1 (renderer 2, p8 시각 증적 PNG 2) |
| 라우팅 | base: collaborator external PR; modifiers: intake/review, local validation, visual/fixture evidence, multi-PR/update branch |

## 변경 범위와 코드 판정

`LayoutEngine::build_master_page`는 바탕쪽 `Shape`/`Equation` 배치에 기존 `paper_area`를
`col_area`로 넘겼다. 따라서 `Para`/`Column` 상대의 좌·우 정렬 부동 도형은 물리 용지 끝을 기준으로
계산됐다. PR은 바탕쪽용 `master_col_area`의 가로 `x`/`width`만 `body_area`로 교정하고, 세로
`y`/`height`는 기존 `paper_area` 값을 보존한다.

`compute_object_position`의 `Para`는 문단 컨테이너, `Column`은 `col_area`를 기준으로 하므로 두 경로가
함께 본문 가로 여백을 사용한다. `Paper`/`Page` 상대 위치는 별도 `paper_area`/`body_area`를 사용하므로
영향이 없다. 재현 문서의 문제 개체는 `Shape` GSO이며 `Picture`의 별도 배치 경로를 변경하지 않는다.

회귀 test `test_master_page_header_shapes_stay_within_body_margins`는 page 8 머리말 글상자 clip의 좌·우가
용지 끝에 닿지 않음을 확인한다. 전용 release-test binary에서 정확한 test 이름으로 재실행해
`1 passed; 0 failed`를 확인했다.

## 시각·fixture 증적

- 재현 원본: `samples/21_언어_기출_편집가능본.hwp`
  (`SHA-256 905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`).
- 한컴 기준 PDF: `pdf/21_언어_기출_편집가능본-2022.pdf`, 15 pages
  (`SHA-256 f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72`).
- 대표 검토 asset: `mydocs/pr/assets/pr_3402_kevin9327_header_float_p008_review.png`
  (`SHA-256 fc17dc1b6b813ed226504467c10516db8424769f727ab8b11be92f15c9884e82`). 원본 정답지·수정 전·수정 후를
  page 8 한 장에 3단으로 배치한 실제 PNG다.

![PR #3402 page 8 시각 검토: 기준 PDF·수정 전·수정 후](../assets/pr_3402_kevin9327_header_float_p008_review.png)

사람 검토에서 수정 전에는 쪽번호가 좌측 물리 용지 끝, “홀수형” 상자가 우측 물리 용지 끝에 붙어 있고,
수정 후에는 두 개체가 기준 PDF의 본문 여백 안으로 이동한다. Page 기준 밑줄과 단 구분선은 유지된다.
추가 local visual sweep은 작업지시자 지시에 따라 실행하지 않았다. 이 PNG는 시각 판정의 대표 근거이며,
최종 한컴 판정 권위는 작업지시자에게 있다.

## 검증·CI 상태

- `c1bfe4cb`를 당시 `upstream/devel`에 merge simulation한 결과 conflict 0, `git diff --check` 통과.
- 이후 최신 `upstream/devel` `52c3bb493`을 source branch에 merge하는 update branch도 conflict 0.
- `c1bfe4cb`의 GitHub Actions는 CI Build & Test(8 shard 포함), Native Skia tests, CodeQL, Render Diff를
  모두 통과했다. external fork이므로 stale-run reaper는 의도적으로 skipped였고 이전 SHA run은 모두
  이미 완료돼 수동 force-cancel 대상이 없었다.
- renderer 변경이므로 문서·asset trailing commit 및 update branch 뒤의 **최신 head full CI**가 최종
  merge 조건이다. 작업지시자 지시에 따라 추가 local Cargo/Native Skia/WASM 검증은 완료까지 진행하지
  않고 GitHub CI 결과를 사용한다.

## 최종 권고

**merge 권고**. 이 기록·대표 PNG·오늘할일을 source head에 추가한 뒤, 최신 head가 `MERGEABLE`이고
CI, CodeQL, Render Diff가 모두 통과한 것을 다시 확인한 후 merge한다. merge 뒤에는 asset이 `devel`에
반영된 SHA를 사용해 PR comment에 실제 PNG를 표시하고, #3399의 playbook 예시 6 후속 보완을 시작한다.
