---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4665 리뷰 - 배분 정렬 오른쪽 끝 정합

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4665](https://github.com/edwardkim/rhwp/pull/4665) · @planet6897 |
| 최신 원 head | `2fe04a6f97075c78a53fc84b1258c2587b8f944d` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 통합 commit | `8d25b718f` |
| 관련 이슈 | [#4657](https://github.com/edwardkim/rhwp/issues/4657) |

## 경로

```text
base route: collaborator 매개 외부 PR
modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적, 다수 PR·update branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## 검토와 시각 증적

배분 정렬의 남는 폭은 문자 수 `N`이 아니라 글자 사이 `N-1`곳에 나누어야 마지막 글자의 오른쪽 끝이
문단 폭에 닿는다. 말미 공백과 한 글자 문단은 분배 대상에서 제외한다. 합성 공개 fixture
`samples/issue4657/distribute-alignment-sample.hwpx`와 전후 SVG
`mydocs/pr/assets/issue4657_distribute_{before,after}.svg`가 저장소에 포함됐다.

현재 통합 head에서 다음을 실행했다.

```bash
target/pr-review/release-test/rhwp export-svg \
  samples/issue4657/distribute-alignment-sample.hwpx \
  -o /tmp/rhwp-pr4665-distribute-svg-20260812 --json
```

1쪽 SVG 1개(14,519 bytes), `overflowCellLines: 0`을 생성했다. fixture 회귀
`issue_4657_distribute_alignment`은 글자 수가 다른 다섯 줄의 좌우 끝 차이가 2px 이내인지 확인한다.
HWP 2020 기준 PDF는 원 이슈에 원본 문서가 없고 최소 HWPX를 합성한 검증이므로 만들지 않았다.

## 검증과 판정

- `issue_4657_distribute_alignment` focused test, merge tree, `git diff --check`를 통과했다.
- 현재 통합 head에서 전체 release-test nextest `5,782 passed / 36 skipped`, Clippy, Native Skia 58+2+4,
  WASM build를 통과했다.

**판정: 최신 통합 PR CI와 작업지시자 승인을 조건으로 수용한다. #4657 close는 통합 PR merge 뒤 처리한다.**
