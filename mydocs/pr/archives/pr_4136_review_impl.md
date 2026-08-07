# PR #4136 메인터너 보정 이행 기록

## 목적

외부 contributor `planet6897`의 한글 버전별 페이지네이션 오라클 PR을 원 commit을 재작성하지 않고
검토·보정·merge 가능한 상태로 만든다. 이 문서는 code 보정과 review 기록을 분리한 순서를 고정한다.

## 적용 기준과 commit 순서

| 단계 | SHA | 역할 |
| --- | --- | --- |
| 1 | `46f08e997` .. `0762ad241` | contributor: 오라클 도구, 재현 가이드, 측정 보고서 |
| 2 | `cdde6b815` | contributor source의 마지막 devel 병합 head |
| 3 | `c2d8a7039` | maintainer: dotted/comma `hwp.Version` major 판독과 BOM 보정 |
| 4 | `8c702735d` | maintainer: 외부 기여자용 운영 기록 정책을 `CONTRIBUTING.md`에 명시하고 보고서의 깨진 Markdown 경로 2건 보정 |
| 5 | `7f4c92c36` | maintainer: contributor가 넣은 오늘할일 항목 제거 |
| 6 | 이 문서와 `pr_4136_review.md` | maintainer: 검토 결과 기록 |

가시성 branch는 `review/planet6897-20260808`이며, source head `cdde6b815` 위에 maintainer commit만
연속으로 추가했다. 원 contributor commit을 rebase, amend, reset, force-push하지 않았다.

## 검증·push 순서

1. contributor fork의 `docs/hangul-version-oracle-r1` 원격 SHA, GitHub PR head SHA, local source ref가 모두 `cdde6b815`임을 확인했다.
2. 변경 경로의 Git LFS filter가 모두 `unspecified`이고 새 LFS object가 없음을 확인했다.
3. `GIT_LFS_SKIP_PUSH=1 git push --dry-run`으로 `cdde6b815..7f4c92c36` ref 갱신을 확인한 뒤 실제 push했다.
4. 실제 push 뒤 fork ref와 PR head가 `7f4c92c36`으로 일치함을 확인했다.
5. code 보정이 있으므로 review-only fast-pass를 적용하지 않고 해당 head의 Full CI와 CodeQL을 먼저 확인했다. [CI #4136](https://github.com/edwardkim/rhwp/actions/runs/31198985288)의 Native Skia, archive build, regular/slow shard와 CodeQL이 모두 성공했다.
6. Full CI 성공 뒤 이 review 기록만 trailing commit으로 push하고, 같은 code candidate SHA를 재사용한 fast-pass aggregate를 확인한다.
7. 작업지시자 승인 후 최신 PR head·mergeable·required check를 재확인해 merge하고, devel 동기화와 fork branch·local review branch 정리를 수행한다.

## 범위 경계

`mydocs/orders/YYYYMMDD.md`는 외부 contributor PR의 산출물이 아니다. 이번 PR에서 기여자가 추가한
항목은 제거했으며, merge 뒤 maintainer 운영 기록이 필요하면 `devel` 기준의 후속 절차에서 별도로 처리한다.
review 기록은 메인터너가 읽는 archive 문서이고, 외부 기여자가 따라야 할 정책은 `CONTRIBUTING.md`에만
명시했다.
