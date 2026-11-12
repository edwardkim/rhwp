# PR #8956 사전 판단 보고서

## 최종 판정

**승인 — 집중 회귀 4건과 reviewer의 필수 로컬 lint·workspace 빌드가 모두 통과했고, 수정이 필요한 코드 결함은 발견하지 않았다.**
상세 검사·결과·한계는 [리뷰 기록](pr_8956_review.md)에 따른다.
이 문서는 병합 후 보고서가 아니며 merge·이슈 종료를 완료 사실로 기록하지 않는다.

## 원인과 해결 범위

[이슈 #8880](https://github.com/edwardkim/rhwp/issues/8880)은 표 조회가 사용하지 않는 마지막
셀 필드를 필수로 요구하고 경로 형식 설명이 부족했던 문제다.
[PR #8956](https://github.com/edwardkim/rhwp/pull/8956)은 표 경로와 셀 경로 파싱을 분리하고
WASM API 문서 주석·소비자 가이드·회귀 4건을 추가했다. reviewer의 코드 보정은 없다.

## 출처와 반영 계획

| 항목 | 값 |
| --- | --- |
| Source PR | `edwardkim/rhwp#8956`, contributor `gitprogam` |
| 원 commit | `2acf7fecfc644a2fc2870c0ef31fb65872657a20` |
| 검증 head | 원 commit과 동일 |
| Source repository·branch | `gitprogam/rhwp`, `task/8880-bypath-table-path` |
| 검토 branch | `review/pr-8956-bypath-table-path` |
| 문서 | `mydocs/pr/archives/pr_8956_review.md`, `mydocs/pr/archives/pr_8956_report.md` |
| 기여 보존 | 원 commit의 author·메시지·history 변경 없음, 체리픽 mapping 없음 |

현재 확인된 `maintainer_can_modify=true`와 충돌 없는 원 PR을 근거로, 원 source head 위에
문서-only commit을 추가하는 경로로 진행한다. 별도 integration PR은 만들지 않았다.
2026-11-12 작업지시자의 승인 리뷰·조건부 병합 지시에 따라 필수 검토 기록을 먼저 반영한다.
원격 반영 전에는 원격 SHA가 검증 head와 같은지 재확인한다.
보정 코드나 추가 테스트가 생기면 기존 CI 재사용 판단을 다시 수행한다.

## 완료한 보류 해소

2026-11-12 KST에 원 code head에서 native Clippy, WASM32 Clippy, workspace build,
workspace all-target Clippy를 순차 실행해 모두 종료 코드 0으로 통과했다. 전체 target 검사
준비 중 발견한 희소 checkout의 정적 include 입력 누락은 추적 파일을 추가로 checkout해
해소했다. 코드·테스트·Cargo.lock은 수정하지 않았다. 상세 명령과 결과는 리뷰 기록에 남겼다.
갱신한 두 문서의 상대 링크·공백·변경 범위 검사도 통과했다.

## 남은 순서

1. 승인된 검토·병합 범위에서 문서만 별도 commit으로 반영한다.
   원 source SHA와 remote ref를 재확인하고 LFS 대상 여부를 먼저 판독한다.
2. push 이후 문서가 해당 PR diff에만 포함됐는지와 최신 head CI를 확인한다.
   review-only fast-pass는 당시 정본의 모든 조건을 만족할 때만 사용한다.
3. 별도로 승인된 GitHub review·merge 단계에서 최신 SHA·CI·mergeability를 다시 확인한다.
4. 병합된 경우에만 merge SHA와 #8880 상태를 읽어 실제 해결 범위와 자동 종료 여부를 확인한다.
   수동 issue close나 contributor comment는 승인된 범위에서 수행한다.
5. 최종 종료 시 이번 검토 worktree·임시 산출물을 정리한다. 공유 target과 다른 작업의
   사용 여부를 확인하며, 현재 진행 중인 작업의 파일을 삭제하거나 옮기지 않는다.

## Contributor comment 초안

> 경로 입력 계약과 문서 설명을 보완해 주셔서 감사합니다. 검토한 code head에서 마지막 필드
> 생략, 중첩 표 경로, 잘못된 입력 거부, bbox 조회의 native 집중 회귀 4건이 통과했습니다.
> native·WASM32·workspace all-target Clippy와 workspace 빌드도 모두 통과해 검토 판정을
> 승인으로 갱신했습니다. 실제 병합은 최신 head의 CI와 별도 작업지시자 승인을 확인한 뒤 진행합니다.

위 문안은 미게시 초안이다. 시각 개선 PR이 아니므로 Visual Sweep 수치·이미지를 넣지 않는다.
실제 게시 시점의 검증 상태에 맞게 수정하며, 완료되지 않은 검증이나 merge를 완료로 쓰지 않는다.
