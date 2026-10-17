# #7831 이관 회귀 증적

#7445 후속 이슈 #7831의 사용자 승인 이관이다. `.rs.txt`는 변경 전 전체 source이며
활성 Rust test로 수집하지 않는다. source commit과 SHA256은 `archive-sha256.json`에 기록했다.
원문·독립 Print는 다른 정상 검사가 참조하므로 기존 공유 경로에 보존한다.

기존 base 미달과 현재 새 회귀는 이슈 본문에 구별했다. 검사 제거는 출력 개선이나 PR 승인
판정이 아니다. Native/fresh WASM 관련 전체 페이지90% 이상과 확대 판독 후 관계형 검사로
복원한다. 실행 로그·전쪽 TSV는 ignored `output/pr-review/planet6897-20261008/linux-validation-20261011/`에 보존한다.

이관: `issue_2430_cell_rewrap_threshold_no_oversplit` 전체 함수와
`issue_7095_terminal_fragment_box_takes_the_stored_cell_height_remainder`의 가정폭력 분기만
제거한다. 후자의7062 정상 분기와 전자의16쪽 빈 문단 검사는 유지한다.
현재39→37쪽·이어받기 손실은 이슈에서 미해결로 추적한다.
