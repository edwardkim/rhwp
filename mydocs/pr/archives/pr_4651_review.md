---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4651 리뷰 - Square wrap의 다줄 전폭 꼬리 분리

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4651](https://github.com/edwardkim/rhwp/pull/4651) · @planet6897 |
| 최신 원 head | `c6f363b963ce5525d681b0bd7a0c6e9c07d70501` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 통합 commit | `6b8feffb4` |
| 관련 이슈 | [#4650](https://github.com/edwardkim/rhwp/issues/4650), [#4599](https://github.com/edwardkim/rhwp/issues/4599) |

## 경로

```text
base route: collaborator 매개 외부 PR
modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적, 다수 PR·update branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 검토

반폭 Square 표 옆 흐름에서 문단 prefix는 표 옆에, 저장 seg와 조판 줄이 1:1 대응하는 전폭 꼬리는
표 아래에 둘 때만 분리한다. “꼬리가 한 줄”이라는 이전 제약을 “꼬리 전체가 전폭”으로 확장한 것이며,
저장 seg가 없는 일반 wrap 문단에는 적용되지 않는다.

원 PR의 PDF 비교와 sweep 결과는 비공개 코퍼스다. `task_m100_4650_stage1.md`에 원 관측을 보존하며,
공개 HWP 2020 MCP 기준 PDF가 없으므로 fidelity 수치를 일반화하지 않는다.

## 검증과 판정

- merge tree와 `git diff --check`를 통과했다.
- 현재 통합 head에서 전체 release-test nextest `5,782 passed / 36 skipped`, Clippy, Native Skia 58+2+4,
  WASM build를 통과했다.

**판정: 최신 통합 PR CI와 작업지시자 승인을 조건으로 수용한다. #4650 close는 통합 PR merge 뒤 확인한다.**
