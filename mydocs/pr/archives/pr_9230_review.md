# PR #9230 검토 — fix(caption): 캡션 번호 자리표시자를 지우면 자동 번호도 함께 지운다

## 통합 후보의 검토 결과

**원 기여 수용 가능.** 캡션 번호 자리표시자를 지우고 HWP·HWPX로 두 번씩 저장한 후에도 뒤 문구가 남는다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

접수 당시에는 독립 검증이 남아 머지를 보류했다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/caption-number-delete`
- 검토 head `481054374f42b1dd535eb20841619d7531f0ffaf`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:38Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44691041142/job/136148452208)

사용자가 지운 자리표시자의 소유 컨트롤만 제거하고 파서와 미주 공백 정리의 글자 전용 삭제 경로는 보존한다. 캡션 자동 번호를 포함하거나 제외하는 두 경우와 HWP/HWPX 재열기를 확인한다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9230](https://github.com/edwardkim/rhwp/pull/9230)의 head는 `481054374f42b1dd535eb20841619d7531f0ffaf`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

그림 캡션의 번호 자리표시자를 삭제하면 자동 번호가 제거되고 뒤의 문구는 HWP와 HWPX 각각 두 번 저장·재열기해도 보존된다. 머지 직전에 원 head와 필수 CI를 다시 확인한다.

## 병합 후 기여자 PR 댓글 계획

캡션의 번호 자리표시자를 지우면 그 자리에 있던 자동 번호도 함께 지워집니다. 뒤 문구는 HWP와 HWPX로 각각 두 번 저장해 다시 연 후에도 남았습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `481054374f42b1dd535eb20841619d7531f0ffaf`이 병합 이력에 포함된 것을 확인했다. [원 PR #9230](https://github.com/edwardkim/rhwp/pull/9230) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
