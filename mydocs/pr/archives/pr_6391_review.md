---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6391
issue: 6381
author: postmelee
---

# PR #6391 review - `test-caption` false-pass 제거

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/review_only_fast_pass.md`,
  `codex/docs_and_git_workflow.md`
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.

## 메타데이터와 범위

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#6391](https://github.com/edwardkim/rhwp/pull/6391) |
| 관련 issue | [#6381](https://github.com/edwardkim/rhwp/issues/6381) |
| base / head | `devel` / `task_m100_6381-test-caption-false-pass` |
| 기준 devel | `f5440811042f9c5ab7580d3a64204cf1d1e39dd8` |
| code candidate | `988b9c85f021a082c96713ce16c51c97ba7f4864` |
| 규모 | 10 files, `+827/-69`, 8 commits |
| 원격 상태 | Draft, `MERGEABLE`; code candidate required checks 성공, trailing push 전 |

PR은 내부 진단 명령 `test-caption`이 고정 fixture의 캡션 변경을 검증하지 못해도 SVG와 `완료`를 남기고
exit 0을 반환하던 false-pass를 제거한다. CLI command, 세 subprocess 회귀, 내부 CLI 문서와 작업 증적만
변경한다.

고정 좌표를 일반 문서의 그림 자동 탐색으로 바꾸지 않으며 caption setter, renderer, layout, document model,
Render Diff workflow와 공개 CLI schema는 범위 밖이다. PR 본문은 `Closes #6381`을 포함한다.

## self-review 판단

**로컬 기준 수용 권고.** 네 mutation 결과를 개별 추적하고 mutation 성공 대상의 Picture 종류와 caption
방향·세로 정렬·폭·간격을 다시 확인하는 경계가 이슈의 실제 false-pass를 직접 차단한다. 실패를 stderr와
exit 1로 돌리고 렌더·출력 폴더 생성 전에 종료하므로, 일부 mutation 성공이 전체 성공으로 승격되지 않는다.
네 대상이 모두 통과할 때의 기존 stdout, SVG 파일명과 `완료`는 유지한다.

회귀는 고정 대상이 없는 임의 실문서, 일부만 유효한 합성 HWP, 네 대상이 모두 유효한 합성 HWP를 분리한다.
합성 fixture는 공개 `HwpDocument` API와 기존 PNG asset을 사용하고 새 binary fixture를 추가하지 않는다.

## 완료한 검증

검증 기준은 최신 devel merge 뒤 code candidate의 제품 tree인 `143e3032d`다. 뒤의 `988b9c85f`는 검증
결과만 갱신한 docs-only checkpoint다.

| 검증 | 결과 |
| --- | --- |
| focused nextest | 3/3 pass, run `9178a2dd-86d3-4842-a44b-cfe6e6132b96` |
| 전체 integration nextest | 8,660/8,660 pass, 43 skipped, 4 slow |
| 전체 nextest run | `f5122360-2c28-47fa-a8a6-0824129d7d47` |
| clippy | `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과 |
| format | `cargo fmt --all`, `cargo fmt --all -- --check` 통과 |
| integration manifest | 1,032 sources / 4,533 attrs / 48/48 targets, 정책 검사 통과 |
| source-side unit tier | 4,221 tests / 299 modules, 정책 검사 통과 |
| 문서·diff | Markdown 상대 링크와 `git diff --check` 통과 |

`tests/generated/`, `tests/suites/manifest.json`, `target/`, `output/`은 ignored 로컬 검증 산출물이며 PR diff에
포함하지 않았다.

## 렌더 영향과 위험

renderer·layout·paint·pagination·sample·기준 PDF·golden을 변경하지 않는다. SVG 생성은 모든 validation이
성공한 뒤에만 실행되도록 제어 흐름을 좁혔으며 렌더 결과 자체의 의미는 바꾸지 않는다. 따라서 visual
sweep은 적용 대상이 아니다.

잔여 범위는 고정 fixture 좌표가 바뀌면 명령이 의도적으로 exit 1을 반환한다는 점이다. 이는 내부 진단 명령의
검증 계약이며 자동 그림 탐색으로 일반화하는 것은 별도 설계 대상이다.

## 원격 조건과 권고

code candidate `988b9c85f`의 required GitHub Actions는 모두 성공했다.

- [CI run 33264074427](https://github.com/edwardkim/rhwp/actions/runs/33264074427): lint, archive A-D,
  shard A-D와 Build & Test aggregate 성공
- [CodeQL run 33264074414](https://github.com/edwardkim/rhwp/actions/runs/33264074414): Rust,
  JavaScript/TypeScript, Python 분석 성공
- [Adapter inter-diff run 33264074420](https://github.com/edwardkim/rhwp/actions/runs/33264074420): 성공
- [Proptest roundtrip run 33264074417](https://github.com/edwardkim/rhwp/actions/runs/33264074417): 성공

이 문서는 같은 PR의 trailing docs-only commit으로 추가한다. 최신 trailing head의 review-only fast-pass,
required aggregate와 mergeability를 다시 확인하기 전에는 merge하지 않는다.

현재 PR은 작업지시자가 승인한 Draft다. Draft 해제와 실제 merge는 각각 별도 작업지시자 승인 대상이다.
