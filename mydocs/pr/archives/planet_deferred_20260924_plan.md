# planet6897 보류 PR 별도 재검토 준비

이번 승인분 통합 후보는 #7366·#7371·#7372의 코드를 포함하지 않는다. 원본·시도 증거는 기존
`/Users/tsjang/rhwp-planet-review-20260923` 작업 트리에 보존한다. 그 작업 트리의 실험적 보정은
승인분 후보에 복사하지 않았으며, 그대로 수용 판정이나 새 PR의 head로 쓰지 않는다.

| 원 PR | 현재 판정 | 재검토 입력과 해제 조건 |
| --- | --- | --- |
| [#7366](https://github.com/edwardkim/rhwp/pull/7366) | 머지 보류 | `samples/issue2006/1790387_prep_final_report.hwpx` p69·p70의 선행 본문·절단 행·caption을 기여자 branch에서 다시 보정한다. 원 한컴 PDF와 Native review·overlay에서 각각 76.164%, 57.662%였던 2px 관용 실루엣을 90% 이상으로 만들고 실제 행 경계를 판독한다. |
| [#7371](https://github.com/edwardkim/rhwp/pull/7371) | 머지 보류 | `samples/task2430/1382000_domestic_violence_survey.hwp` p19의 terminal fragment 빈 문단·하단 괘선을 다시 맞춘다. 기존 50.706%에서 90% 이상과 회귀 무증가를 입증한다. |
| [#7372](https://github.com/edwardkim/rhwp/pull/7372) | 개별 검토 승인, 통합 보류 | #7370 → #7371 선행 관계를 가진다. #7371이 위 gate를 해소하기 전에는 코드만 떼어 이번 후보에 넣지 않는다. 선행 변경의 새 head와 p16·p18 증거를 다시 확인한다. |

승인분 통합 PR이 `devel`에 들어간 뒤 최신 `upstream/devel`에서 별도의 새 검토 branch를 만든다.
먼저 #7366을 독립적으로, 다음 #7371과 그에 의존하는 #7372를 순서대로 검토한다. 원 PR head를
새로 fetch하고 이미 병합된 commit을 중복 cherry-pick하지 않는다. 각 PR의 reviewer 지정·입력
commit 확인·Native Visual Sweep·전체 회귀·개별 review 판정을 새 head에서 다시 수행한다.
90% 미만 또는 기하 차이가 남으면 source PR을 보류 상태로 유지하고, 승인분 통합 PR의 merge/comment
범위에 섞지 않는다.
