# PR #3364 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토 경로를 적용했다. 작성 시점 참고값: `kevin9327`의
`pr/fix-issue-3357-capabilities-feature-truth` → `devel`, 최신 head `ad7d53889fc9`, 보류
comment/review 없음, 검토 branch `review/kevin9327-20260726`.

## 변경 검토

`capabilities`가 `export-png`의 feature 요구와 현재 사용 가능 여부를 자기서술한다. 지원한다고
광고하면서 runtime에서 실패하는 경로를 피하게 하며, CLI/MCP 도움말 충돌을 통합 시 보존했다.

## 검증·권고

통합 release-test 전수·clippy·fmt·diff check 통과. feature 감지 계약 변경이라 독립 visual sweep과
IR baseline은 적용 대상이 아니다. [통합 구현 기록](pr_3345_review_impl.md)을 참조한다.

**수용 가능**. #3357은 통합 PR merge 뒤 close 여부를 확인한다.
