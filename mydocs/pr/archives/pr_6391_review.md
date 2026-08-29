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
| 기준 devel | `2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31` |
| code candidate | `d8ab820b065618966dfb67969cf2c1b1ba26992a` |
| 규모 | 최초 구현 뒤 review 보정 6 files, trailing 기록 별도 |
| 원격 상태 | Open; 보정 candidate·trailing push와 최신 required checks 확인 전 |

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
성공 경로는 verification 증적 네 건까지 요구한다. 합성 fixture는 공개 `HwpDocument` API와 기존 PNG
asset을 사용하고 새 binary fixture를 추가하지 않는다.

## review 보정

[review comment](https://github.com/edwardkim/rhwp/pull/6391#issuecomment-5464292086)를 다음 순서로 반영했다.

- **해석 경계**: verifier가 본문 `Control::Picture`만 받던 결함을 고쳐 setter와 같이
  `Shape(Picture)`와 Endnote 가상 문단도 해석한다.
- **회귀 증명력**: 성공 CLI test가 `caption=Some(...)` 네 건을 요구해 verification block 삭제를
  잡는다. `Shape(Picture)` setter/getter 필드와 verifier topology도 별도 test로 고정한다.
- **구조 정리**: expectation·폭·간격과 JSON 생성을 한 곳에 모으고 vector 크기를 expectation 수에서
  계산하며 section lookup과 임시 폴더 정리를 단순화했다.
- **자기서술**: help·capabilities·CLI 정본에 “고정 fixture 캡션 라운드트립 검증”을 명시했다.

HWP5 export가 `Shape(Picture)`를 `Control::Picture`로 정규화해 subprocess fixture로 해당 표현을 보존할 수
없었다. 따라서 존재하지 않는 파일 왕복을 가장하지 않고 model setter/getter와 verifier topology의 두
계약으로 분리했다.

## 완료한 검증

검증 기준은 최신 devel merge `0240e043e` 뒤 보정 code candidate `d8ab820b0`이다.

| 검증 | 결과 |
| --- | --- |
| focused `test-caption` | 5/5 pass, run `bd1bcaa0-dd48-415d-ab11-5a325cdd718d` |
| focused CLI catalog | 20/20 pass, run `c7ba0e7a-ec8f-4b5b-a9ca-4643d1a6078e` |
| 전체 integration nextest | 8,686/8,686 pass, 43 skipped, 1 slow |
| 전체 nextest run | `554b5740-99fd-450e-982d-62c9c8810420` |
| lint·build | native/WASM32/workspace all-target Clippy와 workspace build 통과 |
| format | `cargo fmt --all`, `cargo fmt --all -- --check` 통과 |
| integration manifest | 1,032 sources / 4,535 attrs / 48/48 targets, 정책 검사 통과 |
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

최초 code candidate `988b9c85f`의 required GitHub Actions는 모두 성공했다.

- [CI run 33264074427](https://github.com/edwardkim/rhwp/actions/runs/33264074427): lint, archive A-D,
  shard A-D와 Build & Test aggregate 성공
- [CodeQL run 33264074414](https://github.com/edwardkim/rhwp/actions/runs/33264074414): Rust,
  JavaScript/TypeScript, Python 분석 성공
- [Adapter inter-diff run 33264074420](https://github.com/edwardkim/rhwp/actions/runs/33264074420): 성공
- [Proptest roundtrip run 33264074417](https://github.com/edwardkim/rhwp/actions/runs/33264074417): 성공

이 문서는 같은 PR의 보정 trailing docs-only commit으로 갱신한다. 최신 trailing head의 required aggregate와
mergeability를 다시 확인하기 전에는 merge하지 않는다. 실제 merge는 별도 작업지시자 승인 대상이다.
