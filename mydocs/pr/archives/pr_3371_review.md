# PR #3371 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토 및 문서 검증 경로를 적용했다. 작성 시점 참고값: `kevin9327`의
`pr/task-3370-agent-playbook` → `devel`, 최신 head `13b99acfae8b`, 보류 comment/review 없음,
검토 branch `review/kevin9327-20260726`.

## 변경 검토

에이전트가 업무 유형을 CLI 시퀀스와 기계 검증으로 연결하는 7개 실무 예제를 추가한다. #3345,
#3374, #3384의 실제 명령 계약을 설명하므로 통합본의 도움말·JSON 필드와 교차 확인했다.

## 검증·권고

문서와 명령 예제의 대상 기능은 통합 release-test 전수·clippy·fmt·diff check에서 검증됐다.
문서-only 변경은 별도 Cargo 재실행 대상이 아니며, 링크와 실제 명령 대응은
[통합 구현 기록](pr_3345_review_impl.md)에 남겼다.

**수용 가능**. #3370 이슈는 통합 PR merge 뒤 close 여부를 확인한다.
