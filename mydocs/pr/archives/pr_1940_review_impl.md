# PR #1940 처리 계획

## 적용 커밋

- `83d94bcfdd63388b2d035f49de99c06b3d025b0f` Issue #1917 잔여(XML 축): MAX_XML_SIZE 32→256MB

## 처리 기록

- `upstream/devel` 기준 누적 검토 브랜치 `review/planet-1940-1960` 생성.
- PR #1940 커밋을 첫 번째로 cherry-pick.
- 이후 #1942 적용 시 `src/parser/hwpx/reader.rs` 테스트 영역 충돌 발생, #1940 테스트와 #1942 테스트를 모두 유지하도록 해결.

## 후속 절차

- merge 전 `gh pr view 1940`로 최신 head, mergeability, checks 재확인.
- PR #1940 merge 후 관련 이슈 #1917 상태와 후속 코멘트 필요 여부 확인.
