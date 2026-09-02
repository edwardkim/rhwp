# PR #4763 검토 기록 - #3820 저장 frame 페이지 소유권과 대형 문서 쪽수 정합

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4763](https://github.com/edwardkim/rhwp/pull/4763) |
| 제목 | `fix(renderer): #3820 저장 frame 페이지 소유권과 대형 문서 쪽수를 정합한다` |
| 작성자 | `jangster77` (repository collaborator) |
| base | `devel@d1dade398f893c9a3c7f464137d19160f6dabf93` |
| 검증한 code candidate | `080f7270c443dd217b1c04b64a19fd36c00ed4ea` |
| 규모 | 135 files, +8,069/-766, 96 commits |
| reviewer | `edwardkim` 요청 완료 |
| 작성 시점 참고 상태 | Open, non-draft, MERGEABLE. 최신 review-only head의 상태는 merge 전에 다시 확인한다. |

## 라우팅과 기준선

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md,
           review_only_fast_pass.md, post_merge.md, rework_and_exceptions.md
code candidate: 080f7270c443dd217b1c04b64a19fd36c00ed4ea
upstream/devel: d1dade398f893c9a3c7f464137d19160f6dabf93
```

1,000줄을 넘는 대형 renderer PR이므로 즉시 merge하지 않고 code candidate 전체 CI, current-base
merge simulation, 시각·fixture 증적, trailing review-only CI를 분리해 확인한다. `upstream/devel`은
code candidate의 조상이고 `git merge-tree --write-tree upstream/devel HEAD`가 clean tree
`7add85e09b8acd8443b290679b50aa0711389e1c`를 생성했다. `git diff --check
upstream/devel...HEAD`도 통과했다.

## 관련 이슈와 완료 범위

[이슈 #3820](https://github.com/edwardkim/rhwp/issues/3820)의 완료 기준은 HWP/HWPX 페이지 수 정합과
전체 회귀 실패 0건으로 확정했다. 이슈는 PR 병합 전에 완료 상태로 닫았고, 페이지 수에 영향을 주지 않는
raster·폰트·paint 잔여는 [후속 #4764](https://github.com/edwardkim/rhwp/issues/4764)로 분리했다.

- 2025 행정업무운영 편람 HWP/HWPX를 각각 383페이지로 고정했다.
- 76076 규제분석 문서를 82페이지로 고정하고 주요 연속·중첩 표 fragment의 물리 페이지 소유권을 정합했다.
- issue4090 HWPX를 17페이지로 고정하고 꼬리 페이지 줄 소유권을 정합했다.
- 저장 LineSeg, RowBreak, TAC, rowspan, 각주, float stack과 object frame의 절단·이월 판정을 구조화했다.
- 문서명·페이지 번호·fixture 전용 수치를 런타임 판정 조건으로 추가하지 않았다.

PR 본문은 `Closes #3820`을 사용하고 #4764를 후속으로 참조한다. #3820은 이미 수동 종료됐으므로 merge 뒤
종료 상태와 후속 링크가 유지되는지 재확인한다.

## 최신 upstream 정합과 PR #4755 보호

PR 생성 직전 최신 병합 PR 두 건의 파일을 교차 확인했다. PR #4754는 15개 파일 중 겹침이 없었고,
[PR #4755](https://github.com/edwardkim/rhwp/pull/4755)는 28개 파일 중 `typeset.rs`,
`table_layout.rs`, `composer.rs` 등 10개 파일이 겹쳤다.

`upstream/devel@d1dade398`로 94개 기존 commit을 rebase하면서 `composer.rs` import 충돌 1건을 양쪽
심볼을 모두 유지해 해소했다. PR #4755가 추가한 physical-frame 호출부는 #3820이 일반화한
`tokenize_paragraph_with_regenerated_space_metric`에 연결하되 `false` 플래그, inline-control 전달,
frame carve와 row commit 조건을 바꾸지 않았다.

최종 전체 회귀에서 LayoutFrame 단위 테스트 9건, frame reflow, 실제 p325 picture band,
table owner-width 테스트가 모두 통과해 PR #4755의 계약이 유지됨을 확인했다. 세부 결합 과정은
[Stage 243](../../working/task_m100_3820_stage243_post_4755_rustfmt_convergence.md)과
[Stage 244](../../working/task_m100_3820_stage244_pr4755_tokenizer_contract_rebase.md)에 기록했다.

## 완료한 로컬 검증

아래 결과는 code candidate `080f7270c443dd217b1c04b64a19fd36c00ed4ea`의 작성자 사전 검증이다.
`CARGO_INCREMENTAL=0`은 설정하지 않았다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --profile release-test --lib` | 3,673 passed, 0 failed, 13 ignored |
| `cargo test --profile release-test --tests` | 종료 코드 0 |
| 전체 `test result: ok` 묶음 | 546개, 실패 표식 0 |
| `cargo clippy -- -D warnings` | 통과 |
| `issue_3820_rowbreak_rowspan_band` | 4 passed, 0 failed |
| PR #4755 LayoutFrame·frame reflow·picture band·table owner-width | 모두 통과 |
| current-base merge tree | clean tree `7add85e09b8acd8443b290679b50aa0711389e1c` |
| `git diff --check upstream/devel...HEAD` | 통과 |

같은 code candidate의 GitHub Full CI가 성공했고 이 뒤에는 review와 오늘할일만 추가하므로 전체 Rust 회귀와
Native Skia 묶음을 로컬에서 다시 반복하지 않는다. 최신 review-only head에서는 preflight가 동일 PR의 녹색
candidate를 채택했는지와 Build & Test aggregate가 성공했는지 다시 확인한다.

## 시각·fixture 증적

renderer/layout/typeset과 기준 PDF를 변경하므로 시각·fixture 증적 경로를 적용했다.

- PR에는 HWP 2020 MCP로 산출한 비교 PDF 17개가 포함된다.
- 핵심 기준은 `pdf/issue3820/2025-administration-final-hwp-hwp2020-20260814.pdf`,
  `pdf/issue3820/2025-administration-final-hwpx-hwp2020-20260814.pdf`,
  `pdf/issue3820/76076-regulatory-analysis-hwp2020-20260814.pdf`,
  `pdf/issue3820/156492236-regulatory-sandbox-min-hwp2020-20260814.pdf`다.
- 입력 HWP/HWPX 17개, 새 PDF 17개, #4490·#4491 직접 기준 PDF의 전체 대응표는
  [후속 #4764](https://github.com/edwardkim/rhwp/issues/4764)에 보존했다.
- HWP 2020 MCP 기준은 `file`에서 일반 PDF 1.7로 식별되는 결과이며 `zip deflate encoded` 결과는 제외한다.
- 383페이지 전체 raster 완전 일치, issue4090 p5 object wrap, 76076 paint 잔여, #4490·#4491 글꼴 차이는
  이 PR의 페이지 수 완료 기준과 분리해 #4764에서 추적한다.

최종 rebase HEAD에서 별도 `wasm-pack build`는 다시 실행하지 않았다. 이전 #3820 head에서 작업지시자가
WASM build를 완료했고, code candidate의 GitHub `Lint (fmt, clippy, WASM check)`와 Native Skia가 성공했다.
별도 `WASM Build` job은 preflight 판정에 따라 skipped였으므로 실제 최신-head wasm-pack 성공으로 과장하지 않는다.

## GitHub code candidate CI

code candidate `080f7270c`의 GitHub checks는 문서 commit 전 모두 완료됐다.

| 축 | 결과 |
| --- | --- |
| [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/31786239651) | 성공. slow shard와 regular 1/3, 2/3, 3/3 모두 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31786239390) | Rust, Python, JavaScript/TypeScript 분석 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31786239317) | Canvas visual diff 성공 |
| Lint | fmt, Clippy, WASM check 성공 |
| Native Skia | 성공 |
| `gh pr checks --watch` | 종료 코드 0 |

## 위험과 후속 범위

- 96개 commit과 135개 파일의 대형 변경이므로 merge 직전 최신 head·base·aggregate를 다시 확인한다.
- 전체 raster 일치를 완료 주장에 포함하지 않는다. 남은 시각 차이는 #4764의 입력·PDF 대응표와 수용 기준을 따른다.
- 최신 review-only head에서 fast-pass가 아니라 Full CI fallback이 선택되면 그 전체 CI 완료까지 기다린다.
- #3820의 완료 상태와 #4764의 열린 상태를 merge 후 확인하고 중복 close comment를 남기지 않는다.

## 권고

code candidate의 로컬 전체 회귀, GitHub Build & Test, CodeQL, Render Diff, Native Skia와 current-base
merge simulation에서 차단 결함을 발견하지 못했다. 이 review와 오늘할일만 추가한 trailing commit의 최신
preflight·Build & Test aggregate가 성공하고, PR이 mergeable이며 reviewer와 작업지시자 승인을 충족하면
squash merge를 권고한다. merge 뒤에는 devel 반영, #3820·#4764 상태, PR 후속 comment와 정확한 head branch
정리를 `post_merge.md` 순서로 수행한다.
