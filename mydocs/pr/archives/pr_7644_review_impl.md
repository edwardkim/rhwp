# PR #7644 메인터너 통합 실행계획

- 근거: 사용자 2026-10-08 시각 판정 통과 및 PR 머지 승인. 별도 메인터너 PR 통합 뒤 원 PR 종료 관례를 적용한다. #7681 통합 완료, 원 #7644는 ancestry에 의해 자동 MERGED 종료했다.
- 원 기여: sacru2red head `5cd52f83aaed82bbaad1d4331587002831c27392`; 원 commit author와 history를 보존한다.
- base: `f0e7228f6dd2ea1437724e53ad640c40c56d204b`; 통합 base commit `daa73a43…`.
- 메인터너 production: `dd1a2051e…` 저장 빈 문단 점유, `37caee9df…` 실제 컷 프레임의 padding/flow 공유.
- 정식 회귀·PNG: `61a8d3ff1…`. 진단 source와 입력은 같은 review branch에서 연속 보존한다.

1. 제한된 시각 수용과 남은 범위 기록, 해당 경계의 정식 회귀 FAIL/PASS 확인 — 완료.
2. 새 test 포함 Rust lint·manifest와 release-test 전체 10538 PASS — 완료. Native Skia 3종 4109 + 2 + 4 PASS — 완료.
3. current-base 충돌·문서 링크·fixture commit 확인 후 원본 저장소의 임시 head로 push, 한국어 Open 통합 PR 생성 — 완료.
4. #7681 head `c92d3f7cd…`의 required check·CI 성공과 최신 base simulation 확인 후 merge — 완료. merge `4c7092397…`.
5. 원 #7644 자동 MERGED 확인 — 완료. #7620 부분 해결 유지 및 merge SHA 이미지 안내 — 진행.
6. review·오늘할일의 확정값 반영, 기본 devel fast-forward, 작업 전용 branch 정리. 실행 중 검증 Studio worktree·공유 target은 사용자 확인을 위해 유지 사유를 기록 — 대기.

원 PR을 먼저 merge하거나 CI를 이전 contributor head로 대체하지 않는다. 실패한 테스트를 baseline 완화로 숨기지 않고 실제 실패의 원인과 독립 출력부터 확인한다.
