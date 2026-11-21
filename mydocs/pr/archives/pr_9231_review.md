# PR #9231 검토 — feat(render): 한컴 보기 > 그림처럼 편집 화면에서 그림 자리만 표시하는 setShowPictures

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 그림을 끈 편집 화면에는 자리표시선이 표시되고, 같은 상태에서 인쇄하면 원래 그림이 나온다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**머지 보류.** 화면에서 그림 표시를 끄면 명시적으로 요청한 Print 출력에서 그림이 사라진다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/show-pictures`
- 검토 head `e052d5212e457e382b8268d0a36362f6c290a233`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:41Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44695675392/job/136163464072)

그림 숨기기를 페이지 트리 단계에서 적용한다. 명시적 Print 프로필과 화면 프로필이 이 트리를 공유하므로 화면의 보기 설정이 인쇄에 미치는 영향을 확인한다. 제출된 검사는 화면에서 숨겼다가 다시 켜는 경우를 다룬다.

## 메인테이너 실행 결과

누적 적용 8번째, 적용 직후 `1cb9588707`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 1개가 통과했다.

그림이 하나인 문서로 `setShowPictures(false)` 전후의 Screen 및 Print SVG를 비교했다. 두 프로필 모두 이미지 1개·선 0개에서 이미지 0개·대체선 6개로 바뀌었다. 표시를 다시 켜면 인쇄 이미지도 돌아왔다. 화면은 대체 도형을 그리고, 같은 문서의 Print는 원래 그림을 유지하는 대조가 필요하다.

공개 API 대조 대상은 `getShowPictures, setShowPictures`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9231](https://github.com/edwardkim/rhwp/pull/9231)의 head는 `e052d5212e457e382b8268d0a36362f6c290a233`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

화면에서 그림을 숨기면 명시적으로 지정한 Print 출력에서도 그림이 사라지고 자리표시선이 생긴다. 화면 표시 상태를 Print에 적용하지 않도록 보완하고 양 프로필을 같은 문서로 대조한다.

## 메인테이너 보완 검증 중간 결과

동일 문서에서 그림 표시를 껐을 때 Screen은 그림 0개·자리표시선 6개, Print는 그림 1개·자리표시선 0개였다. 표시를 다시 켜면 두 프로필 모두 그림 1개였다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

편집 화면에서 그림을 숨기는 기능을 반영했습니다. 보기를 끈 같은 문서에서 화면에는 자리표시선 6개가 남고, 인쇄에는 원래 그림 한 개가 표시되는 것을 확인했습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `e052d5212e457e382b8268d0a36362f6c290a233`이 병합 이력에 포함된 것을 확인했다. [원 PR #9231](https://github.com/edwardkim/rhwp/pull/9231) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
