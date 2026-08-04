# PR #3352 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토 경로를 적용했다. 작성 시점 참고값: `kevin9327`의
`pr/fix-issue-3349-export-text-option-order` → `devel`, 최신 head `74a80bd9b07f`, 보류
comment/review 없음, 검토 branch `review/kevin9327-20260726`.

## 변경 검토

`export-text`가 위치 인자보다 앞에 온 옵션도 일관되게 해석한다. #3360의 export 계열 파서
통일과 함께 적용해, 한 명령만 예외적으로 파일 앞 옵션을 거부하는 회귀가 없도록 했다.

## 검증·권고

통합 release-test 전수·clippy·fmt·diff check가 통과했다. CLI 파싱만의 변경이므로 visual sweep,
IR fixture baseline은 적용 대상이 아니다. 전체 검증 맥락은
[통합 구현 기록](pr_3345_review_impl.md)에 있다.

**수용 가능**. #3349를 닫는 통합 PR의 CI 성공이 병합 조건이다.
