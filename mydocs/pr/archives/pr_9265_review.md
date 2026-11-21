# PR #9265 검토 — fix(clipboard): 각주·개체 뒤 논리 캐럿의 내부 붙이기와 표·개체 글자 모양 상속

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 본문의 인접한 두 수식 사이에 붙여넣은 글자가 두 수식 사이에 남는다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9265](https://github.com/edwardkim/rhwp/pull/9265)의 head는 `0a94a8710270d219590a5ae73678df0f220cbd64`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

인접한 수식 사이에 캐럿을 놓고 내부 붙여넣기할 때 입력이 어느 수식 뒤로 가는지 확인했다. 공개 선언에 pasteInternalLogical도 빠져 있다. 인접 개체 사이의 논리 위치를 보존하고 본문 붙여넣기 및 공개 선언을 검증한다.

## 메인테이너 보완 검증 중간 결과

두 인접한 수식 사이에 X를 본문 내부 붙여넣기로 삽입하면 가[수식]X[수식]나가 된다. 새 관계 검사가 통과했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

서로 붙어 있는 두 수식 사이에서 본문에 붙여넣기를 실행했습니다. 두 수식과 앞뒤 본문이 보존되고 새 글자가 실제 캐럿 위치인 두 수식 사이에 놓였습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `0a94a8710270d219590a5ae73678df0f220cbd64`이 병합 이력에 포함된 것을 확인했다. [원 PR #9265](https://github.com/edwardkim/rhwp/pull/9265) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
