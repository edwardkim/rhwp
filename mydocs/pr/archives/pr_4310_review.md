---
kind: pr_review
status: ci-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4310 검토 — #4132 함수 게이트 native-skia test CI 회복

## 결론

**merge 후보.** 함수 단위 `native-skia` gate 때문에 CI 어디에서도 실행되지 않던 두 integration test를
파일 게이트 target으로 분리하고 Native Skia job·classifier에 연결했다. 새 target과 `#[path]` support의
배선 누락, 혼합 crate의 함수·module·`cfg_attr` 우회를 계약 테스트가 자동 감시한다.

초기 self-review가 발견한 재발 감시 공백과 CodeQL 비효율 정규식 경고를 code candidate
`a0e87055f3fc990e7684fb6e030b4e38783f12aa`에서 보정했다. 해당 head의 full CI, CodeQL, Canvas visual diff가
모두 통과했다. 이 문서는 trailing review-only commit으로 PR diff에 포함한다. 최종 merge 조건은 최신 PR
head의 required checks 통과, 실제 reviewer 확인, 작업지시자 승인이다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md(4.3 CI workflow)
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, codex/docs_and_git_workflow.md
code candidate head: a0e87055f3fc990e7684fb6e030b4e38783f12aa
current base: upstream/devel f94fe5e4f834867d830cb7dacfd1d0043d9383d5
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4310](https://github.com/edwardkim/rhwp/pull/4310) |
| 작성자 | `postmelee` (collaborator self-merge) |
| 대상 / source | `devel` / `postmelee:issue-4132-native-cli-exit` |
| code candidate | `a0e87055f` — 작성 시점 `MERGEABLE` / `CLEAN`, draft |
| 규모 | 14 files, +809 / -153 — Rust test·support 6, CI 계약·classifier 3, workflow 1, 문서 4 |
| 관련 issue | [#4132](https://github.com/edwardkim/rhwp/issues/4132), [#4040](https://github.com/edwardkim/rhwp/issues/4040), [#3790](https://github.com/edwardkim/rhwp/issues/3790) |
| review request | 작성 시점 없음 — draft 해제 전 실제 reviewer 지정 필요 |

## 변경 범위와 안전 계약

- `cli_exit_codes`와 `issue_1144`의 native 전용 test를 각각 파일 게이트 target으로 이동했다. default target의
  기존 test는 중복 실행하지 않는다.
- 두 target을 Native Skia job의 `release-test`·`release` 경로에 모두 추가했다.
- 두 target과 공용 support 두 파일을 classifier의 native-skia 소유 경로로 고정했다.
- 파일 게이트 target과 support를 저장소에서 자동 발견해 job·classifier 누락을 실패시킨다.
- 혼합 crate의 native 전용 test를 함수 cfg, inline module cfg, `cfg_attr(native-skia, test)` 세 형태에서
  탐지한다. 일반 함수 내부 item은 test harness 대상이 아니므로 제외한다.
- 제품 코드와 renderer·layout·paint·pagination·golden은 바꾸지 않는다.

## 초기 self-review 보정

[초기 코멘트](https://github.com/edwardkim/rhwp/pull/4310#issuecomment-5229270220)의 유효한 지적과
사실 오류를 다음처럼 처리했다.

| 항목 | 판정과 처리 |
| --- | --- |
| F1 review 문서 누락 | 이 문서를 `mydocs/pr/archives/pr_4310_review.md`에 추가한다. |
| F2 최상위 함수만 탐지 | 함수 body를 제외한 inline module 내부 test와 module cfg, `cfg_attr(..., test)`까지 확장했다. |
| CodeQL 비효율 정규식 | outer attribute 전체를 잡던 중첩 반복 정규식을 제거하고 구분자·body 범위를 구조적으로 읽는다. |
| F3 branch/base 문서 정합 | 계획서를 실제 `issue-4132-native-cli-exit` / `f94fe5e4f`로 정정했다. |
| F4 주석 속 `#[path]` 오탐 | Rust 비코드 마스킹 위치에서 attribute opener를 찾고 원문의 path 값만 대응시킨다. |
| F4 `#[doc = "a]b"]` 누락 주장 | 문자열 마스킹 뒤 정상 탐지됨을 재현해 지적을 철회했다. |

초기 코멘트의 `조건부 merge 승인`은 self-review가 부여할 수 있는 GitHub approval이 아니다. 후속
코멘트에서는 `merge 권고`로 정정하고 실제 reviewer·작업지시자 승인 조건을 분리한다.

## 검증

### 로컬 focused 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` | 27 passed |
| workflow 계약 테스트 5개 파일 결합 실행 | 68 passed |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 28 passed |
| 초기 리뷰의 inline module 함수 cfg, module cfg, `cfg_attr` 세 입력 | 모두 `t` 탐지 |
| `actionlint .github/workflows/ci.yml` | 통과 |
| `node --check scripts/ci-impact-classifier.cjs` | 통과 |
| `python3 scripts/check_markdown_links.py` | 525개 문서, 내부 상대 링크 이상 없음 |
| `git diff --check` | 통과 |

### code candidate 원격 검증

| workflow / check | 결과 |
| --- | --- |
| [CI run 31290225164](https://github.com/edwardkim/rhwp/actions/runs/31290225164) | Build & Test aggregate와 default slow·1/3·2/3·3/3 shard 통과 |
| Native Skia tests | 통과, 6분 46초 — 새 target 두 건 포함 |
| Lint / frontend package | 통과 |
| [CodeQL run 31290225012](https://github.com/edwardkim/rhwp/actions/runs/31290225012) | Python·JavaScript/TypeScript·Rust 분석과 최종 CodeQL check 통과 |
| [Render Diff run 31290225007](https://github.com/edwardkim/rhwp/actions/runs/31290225007) | Canvas visual diff 통과 |

## 시각·fixture 판단

별도 시각 asset은 적용하지 않았다. PR 고유 변경은 integration test 구조와 CI workflow·classifier·계약
테스트다. renderer 출력이나 HWP/HWPX/PDF fixture를 변경하지 않는다. Render Diff의 Canvas visual diff도
code candidate에서 통과했다.

## 잔여 조건

- 이 review-only trailing commit의 최신 head checks를 확인한다.
- draft 해제 전에 실제 reviewer를 지정한다.
- self-review는 approval이 아니므로 실제 reviewer와 작업지시자의 merge 승인을 별도로 받는다.
- merge 뒤에는 `post_merge.md`에 따라 merge SHA, devel 반영, issue #4132 상태와 후속 기록을 확인한다.

## 최종 권고

보정 code candidate는 로컬 계약 검증과 최신 full CI·CodeQL을 통과했다. review 문서 commit을 push한 뒤
최신 head의 required checks를 확인하고, 실제 reviewer 검토와 작업지시자 승인을 받아 merge 후보로 진행한다.
