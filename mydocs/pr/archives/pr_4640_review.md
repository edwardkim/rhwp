---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4640 리뷰 - TAC host 동거 float의 사다리 스냅 회복

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4640](https://github.com/edwardkim/rhwp/pull/4640) · @planet6897 |
| 최신 원 head | `097625b6f4f1b7c5b17297a92d9662448c5b92ed` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 통합 commit | `3063265ac` |
| 관련 이슈 | [#4639](https://github.com/edwardkim/rhwp/issues/4639), [#4599](https://github.com/edwardkim/rhwp/issues/4599) |

## 경로

```text
base route: collaborator 매개 외부 PR
modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적, 다수 PR·update branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 충돌 해소와 검토

`src/renderer/layout.rs`에서 #4614/#4623의 사다리 역보정과 같은 조건부를 수정해 충돌했다.
메인터너는 `y_before_vpos_adjust`를 보존하고, 직전 TAC host가 TopAndBottom 또는 Square 비-TAC
Shape/Picture float를 함께 가진 때만 `prev_tac_seg_applied`의 스냅 생략을 해제하도록 두 조건을 함께
보존했다. 따라서 TAC 표의 전진 효과는 유지하면서 동거 float 밴드만 흐름에서 사라지지 않게 한다.

원 PR의 PDF 좌표 증적은 비공개 코퍼스에 한정된다. 공개 HWPX/PDF가 없어서 HWP 2020 MCP PDF를 새 기준으로
만들지 않았으며, 검토는 충돌 해소의 조건 합성과 전체 회귀를 근거로 한다.

## 검증과 판정

- merge tree와 `git diff --check`를 통과했다.
- 현재 통합 head에서 전체 release-test nextest `5,782 passed / 36 skipped`, Clippy, Native Skia 58+2+4,
  WASM build를 통과했다.

**판정: 최신 통합 PR CI와 작업지시자 승인을 조건으로 수용한다. #4639은 통합 PR merge 확인 뒤 close한다.**
