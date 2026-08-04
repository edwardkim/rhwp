# PR #3347 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토 경로를 적용했다. 작성 시점 참고값으로 `kevin9327`의
`pr/task-batch-axes` → `devel`, 최신 head `58b1792aa317`, maintainer 보류 comment/review 없음,
검토 branch `review/kevin9327-20260726`이다.

## 변경 검토

batch CLI에 `search`·`export-tables`·`fields` 축을 추가해 코퍼스 처리 범위를 맞춘다. 단일
search(#3354)와 batch search가 같은 helper를 공유하므로, 통합 과정에서 `totalMatchCount`와
`truncated`가 두 JSON 응답에 동일하게 노출되는지 확인했다. 1,000개 절단을 성공처럼 숨기지 않는다.

## 검증·권고

통합 전수 release-test, clippy, fmt, diff check가 통과했다. 자세한 교차 계약과 명령은
[통합 구현 기록](pr_3345_review_impl.md)을 따른다.

**메인터너 계약 보정 포함 수용 가능**. #3346 관련 batch 계약은 통합 PR의 CI 성공 후 반영한다.
