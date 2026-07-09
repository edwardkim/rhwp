# PR #2117 처리 계획

## 적용 전략

1. `upstream/devel` 기반 통합 검토 브랜치 `codex/planet6897-review-20260709` 사용.
2. conflict PR 제외 후 #2117 최종 delta commit `ed7f1df8d6188792723adaaac191a17b90918e0b` cherry-pick.
3. 실문서 기준 PDF, synthetic fixture PDF, README 설명, 테스트를 교차 확인.
4. 별도 MCP 경로로 실문서 PDF 변환이 1쪽 A4로 정상 산출되는지도 확인.

## merge 전 조건

- PR head 최신 상태에서 GitHub Actions 통과 재확인.
- mergeable 상태 재확인.
- 사용자 승인 후 GitHub review/comment 또는 merge 처리.

## 후속 처리

merge 시 `devel` sync 후 PR 브랜치 정리. 이 PR은 #2097 후속 검증자료 보강 PR이므로 issue close/comment 필요 여부는 #2097 상태와 함께 판단한다.
