# PR #2114 처리 계획

## 적용 전략

1. `upstream/devel` 기반 통합 검토 브랜치 `codex/planet6897-review-20260709` 생성.
2. conflict PR 제외:
   - #2107: `CONFLICTING/DIRTY`
   - #2109: `CONFLICTING/DIRTY`
   - #2119: `CONFLICTING/DIRTY`
   - #2104: GitHub은 `BEHIND`였으나 로컬 cherry-pick에서 `src/renderer/typeset.rs` 충돌 발생
3. #2114 최종 delta commit `07d7672e8adfa8c63329afc2509bc32988643818` cherry-pick.
4. 기준 PDF와 추가 테스트 검증.

## merge 전 조건

- PR head 최신 상태에서 GitHub Actions 통과 재확인.
- mergeable 상태 재확인.
- 사용자 승인 후 GitHub review/comment 또는 merge 처리.

## 후속 처리

merge 시 `devel` sync 후 PR 브랜치 정리. 이 PR은 검증 자료 보강 PR이므로 별도 이슈 close/comment 필요 여부는 #1921 상태와 함께 판단한다.
