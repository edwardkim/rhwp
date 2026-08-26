---
kind: pr_review_impl
status: active
canonical: mydocs/pr/archives/pr_4571_review.md
last_verified: 2026-08-11
---

# PR #4571 통합 및 후속 처리 계획

## 구현 경계

| 구분 | commit 또는 자료 |
| --- | --- |
| 기준 `devel` | `32ecfd113` |
| #4366 누적 source | `6f30732fa` .. `e00d1a2ba` |
| #4499 누적 source | `4586c1cd8` .. `1b0b5005b` |
| 메인터너 보정 | `7e37e5b08` - HWPX 실제 검정 음영 보존 |
| 원 PR 검토 증적 | `pr_4366_review.md`, `pr_4499_review.md`, `pr_4366_4499_review_impl.md` |
| 최신 devel 병합 | `559e170bf` |
| PR 생성 전 후보 | `b26a31c6a` |

## 단계

1. 두 원 PR을 최신 `devel` 위에 누적하고, 자동 병합 결과를 전체 nextest와 한컴 2020 PDF로 검증했다.
2. 검토 중 발견한 HWPX 검정 음영 소거를 `7e37e5b08`으로 보정하고 단위 계약을 추가했다.
3. 원 PR별 review, implementation 기록, PNG 증적을 archive에 포함해 별도 문서 전용 PR을 만들지 않았다.
4. `pr/devel-johndoekim-hwp3-chart`를 `upstream`에 push하고 PR #4571을 생성했다.
5. 이 문서·`pr_4571_review.md`·오늘할일을 같은 PR branch의 trailing 기록으로 push한다.
6. 최신 trailing head의 GitHub Actions와 approval 요구사항을 확인한다. 통과 뒤 작업지시자 승인으로 merge한다.
7. merge SHA가 `devel`에 포함된 것을 확인한 뒤 #4155/#4099 상태를 확인하고, 원 #4366/#4499에
   통합 PR·검증·감사 내용을 남긴 뒤 close한다.
8. root worktree를 최신 `upstream/devel`로 fast-forward하고, 이 review worktree와
   `review/johndoekim-20260811`, upstream head `pr/devel-johndoekim-hwp3-chart`를 정확히 확인해 제거한다.
   공유 `target/pr-review`는 보존한다.

## rollback

CI 또는 검토에서 문제를 발견하면 PR #4571은 merge하지 않는다. 메인터너 보정만 철회할 때는
`7e37e5b08`을 revert하고, 원 contributor commit은 amend, rebase, force-push하지 않는다.
