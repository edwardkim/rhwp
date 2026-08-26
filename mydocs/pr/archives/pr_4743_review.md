---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-14
---

# PR #4743 검토 - 입력 경계와 WMF 초기화 안전성

| 항목 | 기록 |
| --- | --- |
| PR | [#4743](https://github.com/edwardkim/rhwp/pull/4743) |
| 작성자 | `jangster77` collaborator self PR |
| 관련 이슈 | [#4290](https://github.com/edwardkim/rhwp/issues/4290), [#4292](https://github.com/edwardkim/rhwp/issues/4292), [#4624](https://github.com/edwardkim/rhwp/issues/4624) |
| base / code candidate | `devel` / `2967509f0b92725a163fff4ade844e0e97e9457f` |
| 변경 규모 | 7 files, +302 / -55 |
| 문서 작성 시점 상태 | `MERGEABLE`, `BLOCKED` 참고값. 최신 head CI와 merge 전 상태 재확인이 필요하다. |
| 검토 방식 | collaborator self review. 별도 외부 reviewer request는 만들지 않았으며 작업지시자 승인 전 merge하지 않는다. |

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md
current code candidate: 2967509f0b92725a163fff4ade844e0e97e9457f
```

최신 `upstream/devel` `d871bb8ce` 위 `task_m100_4290_4292_4624-security-20260814` branch에서
code candidate를 만들고 upstream에 게시했다. 구현·로컬 검증 후 Open PR #4743을 생성한 뒤 이
검토 기록과 오늘할일을 같은 source branch의 trailing docs-only commit으로 추가한다.

## 변경 검토

- #4290은 polygon과 curve의 unsigned 양수 count가 실제 point payload를 초과하면 EOF의 기본값을
  반복 push하던 경로를 막는다. count를 남은 `INT32 x/y` 쌍 수로 clamp하므로 정상 payload의 기존
  parse 순서와 truncated payload의 fallback 의미를 넓히지 않는다.
- #4292는 `start - 1` 대신 `saturating_sub(1)`을 사용한다. malformed `start=0`이 panic하지 않고
  deterministic number string을 만들며, 양수 start의 기존 산술은 바뀌지 않는다.
- #4624는 manual atomic flag와 `static mut BTreeMap`을 immutable `OnceLock`으로 교체한다. 기존 map
  17개 codepage mapping과 symbol table 내용을 그대로 보존하고 unsafe shared mutation을 제거한다.

`src/renderer/layout/utils.rs`는 바뀌지만 이 변경은 start `0` malformed numbering의 text token 경계만
다룬다. 페이지 geometry, typeset, paint, fixture, 기준 PDF, golden은 바꾸지 않는다. unit test가 실제
사용자-visible token `1.`을 확인하므로 이번 안전성 PR에서는 visual sweep을 선택하지 않았다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| #4290 HWP5 huge positive point-count focused test | 1 passed |
| #4292 start=0 numbering focused test | 1 passed, `1.` 확인 |
| #4624 production WMF table 16-thread focused test | 1 passed, HANGUL `949` / GREEK `1253` 확인 |
| release-test nextest 전체 | 5,935 run 중 5,934 passed, 37 skipped, 7 slow; #2833 성능 test 1건만 병렬 시간 예산 실패 |
| #2833 단독 재실행 | 1 passed, 0.079초; 이번 변경과 무관한 일시적 병렬 측정 변동으로 분리 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --target-dir target/pr-review --all-targets -- -D warnings` | 통과 |
| `git diff --check` | 통과 |

문서 링크 검증의 첫 `python scripts/check_markdown_links.py ...` 호출은 이 Ubuntu host에 `python`
alias가 없어 exit 127로 시작되지 않았다. 같은 저장소 script를 `python3`로 재실행해 문서 자체의
검증 결과와 분리했다. `python3 scripts/check_markdown_links.py`는 변경 문서 5개의 내부 상대 링크를
모두 통과했다.

Cargo는 고정 `target/pr-review`를 재사용했고 `CARGO_INCREMENTAL=0`을 지정하지 않았다. source·test
Rust 변경이 있으므로 이 review 기록을 포함한 최신 PR head의 GitHub Full CI와 CodeQL은 merge 전
필수 조건이다.

## 이슈 선별과 비범위

- #4649는 upstream commit `7789998a5`에서 이미 해결되어 중복 수정하지 않았다.
- #4291은 최근 HWPX container guard가 반영됐고, 남은 #4730의 shared recursive-depth 설계는 별도
  issue로 유지한다.
- #4739, #4709, #4668, #4669, #4618은 font/rendering, serializer 또는 fallback 계약 검증을 요구하므로
  이번 input hardening PR에 섞지 않았다.

## 판정

**self-review 수용 후보.** 로컬 코드·focused 검증·lint에서 이번 변경의 blocker는 발견하지 못했다.
다만 최신 trailing docs-only head의 GitHub Actions, CodeQL, mergeability와 작업지시자 승인 전에는
merge하지 않는다. merge 뒤에는 PR 본문의 `Closes #4290`, `Closes #4292`, `Closes #4624`에 따른
이슈 종료 상태와 branch 정리를 post-merge 절차로 확인한다.
