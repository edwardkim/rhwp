# PR #9096 — JinHo-von-Choi 3건 통합 완료

## 판정과 통합

판정은 **승인**이다. 사용자 승인 범위에서 [PR #9096](https://github.com/edwardkim/rhwp/pull/9096)을 2026-11-15 22:48 KST에 merge commit `e4f8ff2d637d5b0a313211455ae3ccc53444d969`로 통합했다. [공개 self-review](https://github.com/edwardkim/rhwp/pull/9096#pullrequestreview-5911029158)는 exact head `b6a91166761b212d8501faf70b4b13e19b26474e`를 검토했다. 예외·검증 기준 완화·admin 우회 없이 병합했다.

원 [PR #9070](https://github.com/edwardkim/rhwp/pull/9070)·[PR #9071](https://github.com/edwardkim/rhwp/pull/9071)·[PR #9072](https://github.com/edwardkim/rhwp/pull/9072)도 자동 MERGED다. 원 기여 SHA 세 개와 contributor fork branch를 보존했다. [PR 상태 영수증](../assets/jinho-von-choi-20261114/post-merge/pr9096-merged-prs.json).

## 검증과 최신 devel

- 로컬 Rust 전체 13,608건·Native 공식 4,073건·모든 lint가 통과했다. Native/fresh WASM 보호 3,542행의 공식값 하락은 0건이며 실제 상승 19행만 하한에 반영했다. 원 범위 134쪽씩 전쪽 raw 90% 이상과 직접 확대 판독을 완료했다. [최종 로컬 원장](../assets/jinho-von-choi-20261114/final-source-8ad/README.md).
- 새 devel #9093을 `1805d8f08cb3a54bccda8dfa6647432b7799f074`로 로컬 병합했다. Studio 2,798건·브라우저 26건과 같은 Chromium의 base/후보 실제 Canvas 글꼴 공급 12조합을 재검증했다. [기능 보존](../assets/jinho-von-choi-20261114/final-source-8ad/incoming9093/README.md).
- GitHub의 실제 선택 범위는 Archive 13,414 PASS·45 skip, Native focused 245 PASS, Studio 2,798 PASS·2 skip, Adapter 7 PASS, Proptest 63 PASS다. CodeQL 4개 언어와 Canvas 3사례·Direct PDF 3/3·CanvasKit 8/8이 완료됐다. 40 check 중 34 success·정상 조건부 skip 6개다. [CI 실행 감사](../assets/jinho-von-choi-20261114/post-merge/ci-actual-execution-audit.md) · [exact head 검사](../assets/jinho-von-choi-20261114/post-merge/exact-head-checks.json).
- PDF report-only 경고 4건은 실제 엄격한 gate 실패가 아니다. 이전 head의 동일 수치까지 대조했다고 주장하지 않는다. 코드·workflow·기준을 바꾸거나 경고를 숨기지 않았다.
- merge 직전 head/base 재조회와 `git merge-tree --write-tree`가 성공했고 원 기여 ancestry를 확인했다. [시뮬레이션](../assets/jinho-von-choi-20261114/post-merge/pr9096-final-merge-tree.json).

## 후속 처리

- 병합 후 [duration 자동 갱신](https://github.com/edwardkim/rhwp/actions/runs/43720966141)이 success다. 실측을 metrics commit `f48e944927735eace6230c9b699764f0e243a64b`으로 게시했다. 검증 CI를 다시 실행하지 않았다. [실행·단계](../assets/jinho-von-choi-20261114/post-merge/duration.json).
- 옵션 M으로 archive·오늘할일·검증 asset만 devel에 직접 반영한다. 테스트 주석의 기존 문서 경로 2곳은 완료 기록 안내문으로 유지한다. 제품·test·workflow·baseline 변경은 없다.
- 연결 이슈 [#8912](https://github.com/edwardkim/rhwp/issues/8912) · [#7631](https://github.com/edwardkim/rhwp/issues/7631) · [#8318](https://github.com/edwardkim/rhwp/issues/8318) · [#8307](https://github.com/edwardkim/rhwp/issues/8307) · [#9046](https://github.com/edwardkim/rhwp/issues/9046) · [#9049](https://github.com/edwardkim/rhwp/issues/9049) · [#7381](https://github.com/edwardkim/rhwp/issues/7381) · [#7711](https://github.com/edwardkim/rhwp/issues/7711) · [#8083](https://github.com/edwardkim/rhwp/issues/8083)는 병합 후 자동으로 completed 종료됐다. 부모 [#7445](https://github.com/edwardkim/rhwp/issues/7445)·[#7620](https://github.com/edwardkim/rhwp/issues/7620)은 OPEN을 유지한다. [실제 상태](../assets/jinho-von-choi-20261114/post-merge/issue-states.json).
- #9049는 기존 해결의 회귀 보강, #7381은 핵심 occurrence·도움말 계약의 완료다. 전체 옵션 리팩터링을 구현했다고 확대하지 않는다.
- 개별 archive의 댓글 계획에 따라 운영 기록 반영·devel 동기화 뒤 이슈 9건과 원 PR 3건에 결과 댓글을 게시했다. API 본문과 실제 GitHub 브라우저의 12개 댓글·48개 이미지 표시를 확인했다. 원 신고 쪽의 [추가 대응 증적](../assets/jinho-von-choi-20261114/final-source-8ad/issue-correspondence/README.md)은 이미 완료한 최종 캡처에서 생산했으며 새 검증이나 제품 변경이 아니다.
- 우리 전용 worktree 3곳·로컬/원격 통합 branch·루트 `output/`·공유 review debug를 실제 사용 종료 후 제거했다. 확보 공간은 약 66.39 GiB, 종료 시 여유 공간은 129.34 GiB다. 다른 worktree·fork branch·글꼴과 다른 target profile은 보존했다. [댓글·브라우저·삭제 경로·전후 용량 영수증](../assets/jinho-von-choi-20261114/post-merge/closure.json), 로컬 최신 동기화 기록은 `local/pr-9096-closure/closure.json`이다.

## 결과 댓글

- [이슈 #7381](https://github.com/edwardkim/rhwp/issues/7381#issuecomment-6895392803)
- [이슈 #7631](https://github.com/edwardkim/rhwp/issues/7631#issuecomment-6895393599)
- [이슈 #7711](https://github.com/edwardkim/rhwp/issues/7711#issuecomment-6895394324)
- [이슈 #8083](https://github.com/edwardkim/rhwp/issues/8083#issuecomment-6895395183)
- [이슈 #8307](https://github.com/edwardkim/rhwp/issues/8307#issuecomment-6895395919)
- [이슈 #8318](https://github.com/edwardkim/rhwp/issues/8318#issuecomment-6895396669)
- [이슈 #8912](https://github.com/edwardkim/rhwp/issues/8912#issuecomment-6895397431)
- [이슈 #9046](https://github.com/edwardkim/rhwp/issues/9046#issuecomment-6895398251)
- [이슈 #9049](https://github.com/edwardkim/rhwp/issues/9049#issuecomment-6895399109)
- [원 PR #9070](https://github.com/edwardkim/rhwp/pull/9070#issuecomment-6895400152)
- [원 PR #9071](https://github.com/edwardkim/rhwp/pull/9071#issuecomment-6895401143)
- [원 PR #9072](https://github.com/edwardkim/rhwp/pull/9072#issuecomment-6895402243)
