# PR #9552 검토 기록

#7445의 하위 이슈 #7813·#7821·#7822·#7830·#7831·#7834·#7863·#7864·#7922·#7955·#8100과
Draft CI 개선 #9561을 검토했다. 기준 `devel`은 `274ae9d4c83154e06da049c90eed6904a8b2ec89`이며
제출 커밋 `e231fcafa7ba16925d722420487cf132716679e6`는 `77b3b0d69aaec3d30bc6e701dde29830bea5d4be`로
병합됐다. 병합 트리가 제출 트리와 일치했다. [PR #9552](https://github.com/edwardkim/rhwp/pull/9552)

## 출력과 회귀

- 저장 줄과 표 조각의 물리 점유를 따라 앞뒤 표·셀 내용·그림이 원본의 순서를 유지하는지 확인했다.
  사무분장표의 91·92쪽에서 두 조각이 원본의 선언 높이를 보존하고 표 순서를 유지하는지 검사했다. #7922에서는 공개 클릭 동작이 끝 탭의 양쪽 문자 4와 5를 구분했다.
  [회귀 테스트](../../../tests/cases/issue_7445_restored_source_layout.rs) · [측정과 편집 검증](../assets/issues7813_8100/README.md)
- 원본 19개를 Native와 Chrome fresh WASM 각각 512쪽에서 한컴 Print와 대조했다. 두 경로의 최저는
  98.04%였다. 같은 조건의 최신 devel과 비교한 1,024쪽에서 하락이 없었다. 정본 보호 23개 문서의
  두 경로 합계 4,046쪽과 공개 글꼴 검사 8건이 기준을 충족했다.
  [Native](../assets/issues7813_8100/native/README.md) · [Chrome](../assets/issues7813_8100/wasm/README.md) · [정본 보호](../assets/issues7813_8100/protected/README.md)
- 최소 글꼴 환경에서 Rust 회귀 14,303건과 Native Skia 4,230건이 통과했다. #9495에서 반영한
  연구노트 편집의 표 순서와 저장 후 재열기도 유지됐다. Draft CI 커밋을 추가한 뒤 조판 코드와
  회귀 입력·증적을 담은 파일의 blob이 모두 일치했다. [시각 비교와 로컬 검증](../assets/issues7813_8100/local-validation.json)

## Draft CI와 GitHub Actions

Draft에서 검증 job을 생략하고 같은 커밋의 Ready 전환에서 검증을 시작한다. Draft 집계의 표시 이름을
필수 `Build & Test`와 구분했고 CI Impact Policy가 상태를 발행하기 직전 현재 PR이 Draft인지 확인한다.
로컬 워크플로·정책 계약 571건과 actionlint를 통과했다. 기본 브랜치에서 실행하는 컨트롤러의 변경은
정상적인 `main` 반영 시 활성화된다. [#9561](https://github.com/edwardkim/rhwp/issues/9561) · [운영 절차](../../manual/github_operations.md)

제출 SHA의 Full CI와 최종 CI Impact Policy가 통과했다. GitHub의 추가 Gym 계약과 Skill Router도
통과했다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46156917984/job/141027825593) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46156917925) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46156917980) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/46156917937) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46156917891) · [Gym](https://github.com/edwardkim/rhwp/actions/runs/46156917944) · [Skill Router](https://github.com/edwardkim/rhwp/actions/runs/46156917908) · [최종 CI 정책](https://github.com/edwardkim/rhwp/actions/runs/46161028427)
