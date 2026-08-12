---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4614 리뷰 - 자리차지 표 위 틈의 사다리 스냅 보정

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4614](https://github.com/edwardkim/rhwp/pull/4614) · @planet6897 |
| 최신 원 head | `e9ed37270b8e23c7b1153818de12f1d00dbcfeb7` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 통합 commit | `7a6c37b3c` |
| 관련 이슈 | [#4613](https://github.com/edwardkim/rhwp/issues/4613), [#4599](https://github.com/edwardkim/rhwp/issues/4599) |

## 경로

```text
base route: collaborator 매개 외부 PR
modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적, 다수 PR·update branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 검토

저장 사다리가 이미 낡았고 실제 흐름 줄은 TopAndBottom 표 위의 빈 zone에 들어가는 형상에서만 전방
스냅을 되돌린다. 직전 렌더 아이템이 zone 소유 문단인지와 잉크 있는 단일 저장 seg를 함께 확인해,
정당한 스냅이나 공백 문단까지 넓게 무효화하지 않는다. 별도 변경은 잉크 없는 문단이 exclusion zone을
소비하지 않게 해 후속 문단 누적 밀림을 막는다.

기여자 stage 기록의 기준 PDF 좌표와 sweep 산출물은 비공개 코퍼스에 속한다. 따라서 공개 HWPX/PDF를
새로 저장하거나 HWP 2020 MCP 대조로 판정하지 않았고, 검토 범위를 코드 경계·회귀·공개 가능한 통합
검증으로 한정했다.

## 검증과 판정

- `git merge-tree --write-tree upstream/devel HEAD` 및 `git diff --check upstream/devel...HEAD`를 통과했다.
- 전체 release-test nextest는 `5,782 passed / 36 skipped`, Clippy는 경고 없이 통과했다.
- Native Skia 58+2+4, `wasm-pack build --target web --out-dir pkg`도 현재 통합 head에서 통과했다.

**판정: 최신 통합 PR CI 성공과 작업지시자 승인을 조건으로 수용한다. #4613은 통합 PR merge 뒤 실제
후속 상태를 확인하고 close한다.**
