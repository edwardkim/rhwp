# PR #9222 검토 — feat(shape): 도형 그림 채우기·그림자 투명도를 주고받고, 표 셀·머리말 안 도형에 글상자를 붙인다

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 존재하지 않는 그림 자원 ID 177을 거절하고 HWP·HWPX로 저장해 다시 열어도 잘못된 참조가 생기지 않는다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**머지 보류.** 문서에 없는 이미지 참조를 그림 채우기로 받아들이고, 저장 형식에 따라 다른 값으로 재열린다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/shape-fill-textbox`
- 검토 head `2badf48da70b42edc3753f4c06c1d5bb634e57f8`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:30Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44662142662/job/136055606238)

도형 그림 채우기와 그림자 투명도를 속성에서 주고받고, 표 셀 및 머리말의 도형에 글상자를 붙이는 명령을 검토했다. 새 공개 메서드는 setTextBoxAtByPath이다. 기본 상태 SVG 대조 외에 새 명령으로 편집한 결과를 실제 소비 경로에서 확인해야 한다.

## 메인테이너 실행 결과

누적 적용 4번째, 적용 직후 `4b77ec0cd0`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 3개가 통과했다.

이미지 리소스가 0개인 문서의 도형에 `imageBinDataId=177`을 지정하면 명령과 조회가 모두 성공했다. HWP 저장·재열기에서는 리소스 0개에 ID 177이 남았고 HWPX에서는 ID 0이 됐다. 실제 저장소에 있는 이미지로 채우는 양성 사례와 없는 ID를 거절하고 이전 채우기를 보존하는 사례를 두 형식에서 확인해야 한다.

공개 API 대조 대상은 `setTextBoxAtByPath`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9222](https://github.com/edwardkim/rhwp/pull/9222)의 head는 `2badf48da70b42edc3753f4c06c1d5bb634e57f8`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

그림 자원이 없는 문서에서도 그림 채우기 ID 177을 받아들인다. 같은 도형을 저장하고 다시 열면 HWP는 177, HWPX는 0이 된다. 유효한 그림 자원만 참조하도록 검증하고, 거절 시 기존 속성과 양 형식 저장 결과가 보존되는지 확인한다.

## 메인테이너 보완 검증 중간 결과

없는 그림 ID 177을 설정하면 오류를 반환한다. HWP·HWPX 저장 후 재열기에서 모두 해당 ID가 없고 이미지 자원 수도 0임을 확인했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

중첩 도형의 그림 채우기와 글상자 편집을 반영했습니다. 그림 자원이 없는 문서에 자원 번호 177을 지정하면 거절되며, HWP와 HWPX로 저장해 다시 열어도 잘못된 참조가 생기지 않습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `2badf48da70b42edc3753f4c06c1d5bb634e57f8`이 병합 이력에 포함된 것을 확인했다. [원 PR #9222](https://github.com/edwardkim/rhwp/pull/9222) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
