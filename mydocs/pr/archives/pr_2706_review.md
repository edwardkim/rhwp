# PR #2706 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2706](https://github.com/edwardkim/rhwp/pull/2706) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| 관련 이슈 | [#2677](https://github.com/edwardkim/rhwp/issues/2677), [#2430](https://github.com/edwardkim/rhwp/issues/2430) |
| 원 commit / 누적 적용 | `cf1577e7` / `5d0b02716` (충돌 없음, 선행 [PR #2510](https://github.com/edwardkim/rhwp/pull/2510)) |
| 범위 | Windows+Hancom COM으로 재보존한 한양 4종 preflight identity artifact |
| 처리 경로 | collaborator 누적 통합 검토. merge 전 최신 원 PR 상태와 CI 재확인 필요 |

## 검증

- `preflight_report.tsv`가 한양 4종과 휴먼명조 5행을 모두 보존하도록 갱신됐고, 저작권 글꼴 파일은 포함하지 않는다.
- `win10-ted`에 Python 3.12.10을 준비한 뒤 최신 `upstream/devel`에서
  `tools/task2430/gen_metrics.py --verify`를 기본 SSH 셸, cmd, PowerShell로 각각 실행했다.
  세 환경 모두 5 face, 95/95 exact match로 일치했다.
- 전체 release-test integration과 clippy도 성공했다. 이 로컬 Windows 검증은 저장된 artifact와 메트릭의
  정합 확인이며, 원 PR이 수행한 Hancom COM 실측을 다시 대체한다고 주장하지 않는다.

## 권고

5행 artifact 재보존이라는 [#2677](https://github.com/edwardkim/rhwp/issues/2677)의 범위에는 부합한다.
최신 head CI와 작업지시자 승인이 충족되면 통합 PR로 merge하고, merge 뒤 #2677 close 여부를 최종 확인한다.
[#2430](https://github.com/edwardkim/rhwp/issues/2430)의 셀 재래핑 잔여는 별도이므로 open으로 유지한다.
