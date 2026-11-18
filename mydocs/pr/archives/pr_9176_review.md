# PR #9176 검토 기록

## 최종 판정

[PR #9176](https://github.com/edwardkim/rhwp/pull/9176)의 v0.9.1 main 승격을 승인하고 병합했다.
GitHub의 승인 리뷰 1건 요건은 작업지시자가 이번 PR에 한해 예외를 승인했다.
필수 CI와 승격 검증을 모두 완료한 뒤 GitHub의 관리자 병합을 사용했다.
[edwardkim의 셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9176#pullrequestreview-5958611371).

| 항목 | 확인값 |
| --- | --- |
| 작성자 | `edwardkim` |
| 검증한 devel head | `08b38c12a75fa5cabfe5fbbfde9388aceb3fcca3` |
| 병합 직전 main | `7213ed1f96266b95e22cd5645a07851c9c18298e` |
| 병합 SHA 및 v0.9.1 태그 | `981ce66b4183b13c1a3b5907e04a8969c8c42034` |
| 검증·병합 tree | `fd066bb4f6db75e02176ed5e5e2f98372f0c895d` |
| 병합 시각 | 2026-11-18 23:32:33 KST |

## 검증 근거

- 고정된 devel에서 승격 workflow 12개가 성공했다. 승격 정책의 15개 필수 항목도 예외 0건으로 통과했다.
  [승격 판정](https://github.com/edwardkim/rhwp/actions/runs/44316168569/artifacts/14431234583).
- main PR의 승격 사전검사·Build & Test·CodeQL·Render Diff·Adapter·Proptest가 통과했다.
  [승격 사전검사](https://github.com/edwardkim/rhwp/actions/runs/44316168569/job/134843083996) ·
  [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44316168569/job/134862190552).
- GitHub 병합 시뮬레이션과 실제 merge commit의 tree는 검증한 devel tree와 같다.
  병합 요청에 HTTP 502가 반환됐지만 GitHub API에서 병합 완료와 정확한 SHA를 확인했다.
- 병합 뒤 v0.9.1을 태그하고 실제 공개 배포를 확인했다. 태그 CI의 Native Skia 시간 초과와
  브라우저 확장의 메인테이너 수동 제출 상태는 [배포 결과](../../report/release_0_9_1_execution_report.md)에 기록했다.
- CI 변경은 [옵션 2](../../manual/pr_review/post_merge.md#711-ci-변경-pr은-옵션-2로-후속-기록)에 따라
  별도 문서 PR에 기록한다. [#9154](https://github.com/edwardkim/rhwp/issues/9154)는 main의 정기 Fuzz 결과를 확인하기까지 OPEN이다.
