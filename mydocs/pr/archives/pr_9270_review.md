# PR #9270 검토 — feat(clipboard): 수식이 든 셀 문단의 논리 캐럿 복사·붙이기

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 셀 HTML 붙여넣기에서도 같은 캐럿 경계와 수식·글자 순서가 보존된다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9270](https://github.com/edwardkim/rhwp/pull/9270)의 head는 `3be4f59be784ca6a174ac88ab0d4224ee3435c0d`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

같은 문자 위치의 인접 수식 사이에서 셀 복사와 붙여넣기의 논리 경계를 확인했다. 새 셀 API 세 개가 공개 선언에서 빠져 있다. 같은 위치의 여러 개체를 논리 순서로 구분하고, 단일 수식 선택과 사이에 붙여넣기 및 타입 소비를 검증한다.

## 메인테이너 보완 검증 중간 결과

같은 논리 캐럿 경계를 셀 HTML 붙여넣기로 실행해 가[수식]X[수식]나를 확인했다. 새 관계 검사가 통과했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

표 셀의 인접한 수식 사이에 캐럿을 놓고 HTML을 붙여넣었습니다. 저장된 개체 순서를 보존하면서 새 글자가 선택한 두 수식 사이에 놓이는 것을 확인했습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `3be4f59be784ca6a174ac88ab0d4224ee3435c0d`이 병합 이력에 포함된 것을 확인했다. [원 PR #9270](https://github.com/edwardkim/rhwp/pull/9270) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
