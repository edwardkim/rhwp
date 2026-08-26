---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4623 리뷰 - TAC 표 줄 seg 사영과 공백 host 보정

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4623](https://github.com/edwardkim/rhwp/pull/4623) · @planet6897 |
| 최신 원 head | `7f1c1d9d736343bd53195c71890c9d7861bfacfa` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 통합 commit | `a6057472f`, `7280645f3` |
| 관련 이슈 | [#4622](https://github.com/edwardkim/rhwp/issues/4622), [#4599](https://github.com/edwardkim/rhwp/issues/4599) |

## 경로

```text
base route: collaborator 매개 외부 PR
modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적, 다수 PR·update branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 검토

음수 줄간격 TAC 문단의 흐름 리셋이 실제 표 줄이 아닌 seg를 집는 경우만 표 control의 line seg로
사영한다. 저장 layout, 비가시 선행 control, 선언 표 높이를 함께 요구해 일반 line-seg 선택을 바꾸지
않는다. 두 번째 commit은 공백 host/문단이 `fix_overlay` push를 재활성화하지 않도록 한다.

기여자 측 PDF와 3,418 문서 sweep은 비공개 코퍼스다. 공개 기준 PDF가 없으므로 HWP 2020 MCP 비교는
수행하지 않았고, 상세 좌표·반증은 `task_m100_4622_stage1.md`의 출처 제한 기록으로 남겼다.

## 검증과 판정

- 최신 devel과의 merge tree, `git diff --check`를 통과했다.
- 현재 통합 head에서 전체 release-test nextest `5,782 passed / 36 skipped`, Clippy, Native Skia 58+2+4,
  WASM build를 모두 통과했다.

**판정: 통합 PR의 최신 head CI와 작업지시자 승인을 조건으로 수용한다. #4622 close는 통합 PR merge
확인 뒤 처리한다.**
