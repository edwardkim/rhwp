# 원격 처리 계획 — 사용자 승인 완료·실행 단계

대상은 #8515·#8599·#8608·#8631·#8656이다. 원 fork `kidsnote/rhwp`의 실제 push 권한이 없으므로 본 저장소의 `review/integration-20261106-hyunmyungjanelee`에 원 기여 SHA를 보존한 후보를 올리고 devel 대상 통합 Open PR로 처리한다. 원 PR 브랜치와 fork는 변경하거나 삭제하지 않는다. 로컬 필수 검증 완료 뒤 작업지시자가 기존 차이 한정 예외와 push·Open PR·새 head CI·self-review·merge·원 PR 완료·댓글·운영 기록·동기화·정리를 승인했다. remote-approval.json을 따른다.

1. 로컬 필수 검증의 완료와 후보 SHA를 확정하고 사용자에게 통합 branch push·Open PR·새 head CI·self-review·merge 및 아래 후속 범위를 함께 제시한다. 성공하지 않은 항목을 CI로 넘기지 않는다.
2. 승인 뒤 최신 원 PR head, upstream/devel, 본인 계정의 reviewer 없는 Open PR을 재조회한다. 원 head가 바뀌면 기존 검증을 새 head에 그대로 적용하지 않는다. 변경된 기준은 실제 merge-tree와 기능 보존으로 대조한다.
3. 원 commit 5개가 모두 후보 조상인지 확인하고 upstream 통합 branch에 push한다. 본문은 각 원 PR·관련 이슈·출력/예산 보정과 정확한 한계, 독립 Desktop Print·전쪽 TSV·공개 PNG·검증 code SHA를 구분한다.
4. 통합 PR의 해당 head 필수 CI와 실제 self-review를 완료한다. 원 PR의 과거 성공 check로 대신하지 않는다. CI 실패는 실제 로그를 읽어 원인에 따라 보정하고, workflow 취소·재실행은 GitHub 운영 규칙의 별도 승인 범위를 따른다. workflow·검사 기준을 임의로 완화하지 않는다.
5. 승인된 merge는 새 base simulation·head CI·mergeable 재조회 뒤 `--match-head-commit`으로 원 기여를 보존하는 merge commit을 사용한다. 원 PR별 간접 MERGED 여부를 확인한다. 자동 반영되지 않은 원 PR의 통합 수용 close는 그 범위의 명시 승인 뒤 수행한다.
6. 개정 post_merge.md 순서대로 archive review·asset·오늘할일을 운영 기록으로 반영한 뒤 devel을 동기화한다. #8513·#8584·#8594는 실제 해결 범위를 확인해 완료한다. #8593은 soft-edge 꼬리의 원본 크기·alpha 오류 해결과 blur/다른 효과 지원을 구분한다. #8579는 개선된 두 원 증상을 기록하되 본문 회피·행 높이·기준선의 명시된 잔여 기전이 있으므로 OPEN을 유지한다. 승인된 이 유지 범위에서 자동 close가 발생하면 잔여 근거와 함께 다시 OPEN으로 맞춘다.
7. 각 원 PR에 한국어 존댓말로 감사·수용 범위·보정·실제 검증과 devel에 존재하는 시각 asset URL을 게시한다. 문서별/페이지별 증거가 해당 PR과 일치하는지 확인한다.
8. 필수 후속 처리가 끝나면 이번 작업 전용 branch·두 worktree·임시 upstream head를 안전 조건에 따라 정리한다. 루트 output 전체와 기본 target/pr-review/debug는 사용 종료 후 정리한다. 다른 작업 worktree와 contributor fork는 보존한다.

승인된 원격 범위와 실제 완료를 구분한다. 최종 로컬 검증은 final-validation.md에 있으며 새 head GitHub CI 성공을 확인하기 전에는 병합하지 않는다.
