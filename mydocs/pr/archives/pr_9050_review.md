# PR #9050 리뷰 — #8975 최신 검사·입력 표지 후속 수용

## 최종 판정

**승인 — 병합 완료.** 제출 head `2201c894aa527661d2a28ad194688b2738196070`의 필수 CI·셀프 리뷰 뒤
merge `c680b2a58eaade4f3185880fd21d72c2480c4f0a`로 통합했다(2026-11-13 15:12:57 KST).
원 #8975의 최신 head `66900a40bec0a45d5dd3124538d824c483d631f0`도 간접 MERGED다.
사용자가 승인한 #8951·#8975 처리 범위의 완료이며 새 제품 수정·이슈 해결 범위를 추가하지 않았다.

## 검증과 병합 근거

- 최신 devel `1003349996897c6ff097271e7296ae4d1a3d807f`와의 로컬/GitHub merge tree가 모두 `faecbc28e1a385e02e7c5af4e56270172b18b7ce`다. 직전 MERGEABLE/CLEAN·원 PR head 불변·정확한 head CI를 확인하고 `--match-head-commit`으로 merge했다.
- 31 SUCCESS·정상 SKIP 5건. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/43373482321)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43373481590)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43373481978)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43373481782)·[최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/43376885483)가 모두 성공했다. 실제 로그를 확인했으며 재실행·판정 예외는 없었다.
- GitHub 회귀 A/B/C/D/E는 각각 3,796/2,151/3,145/1,944/2,203 PASS, 합계 13,239 PASS·45 정상 skip이다. Native와 Frontend·Lint도 성공했다. Render Diff는 이번 변경 경로의 workflow 필터·trusted 정책에서 실행 대상이 아니었다.
- [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9050#pullrequestreview-5886931071)는 본인 PR의 COMMENT다. 실제 게시 본문·commit ID를 API 재조회로 확인했다. 열린 PR merge ref CodeQL 경고는 0건이다.
- 최소 글꼴 집중 9 PASS, 보안 입력 샘플 6개를 포함한 래칫·쪽수 82 PASS, SVG snapshot·반복 출력 결정성 4 PASS와 필수 lint/build/manifest가 통과했다. 제품 파일 변경은 0건이다.
- Native/fresh WASM 각 12쪽은 모두 raw 100.00%다. 기여자의 합성 표지 입력·구형 표지 없는 입력·실제 Hancom 2020 재저장본을 각각 동일 입력의 Desktop 2020 Print와 대조하고 직접 판독했다. [전쪽 TSV·입력/PDF 해시·직접 판독 원장](../assets/pr8975-updated-66900a40/README.md)을 보존했다.
- 보호 3,542쪽·이전 전체 제품 회귀·Native 공식 4,073건은 제품·producer·해당 입력이 동일한 #9047의 근거를 구분해 재사용한다. 새로운 실행으로 표현하지 않았다. 기존 차이·글꼴 예외나 기준값 완화는 없다.

## 조판 원칙과 후속 처리

원 기여의 라벨별 내용 순서·소유·포함과 반례 판정력을 보존했다. 표지 없는 구 입력도 기존 검사에서 계속 실행한다.
쪽수는 독립 Print의 내용 대응과 함께 보호하며 절대 px 위치를 고정하지 않았다. 제품의 분할·측정·배치·저장 줄 수용 조건은 변경하지 않았다.
[원 PR 최종 검토](pr_8975_review.md)와 앞선 [공통 보정 #9047](pr_9047_review.md)의 범위를 구분한다.

병합 후 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/43377144591)은 ready:true·successful-pr-worker-measurements와 게시 성공을 확인했다. source PR/run은 9050/43373482321, attempt 1이며 게시 commit은 50d327549214e9cbe13c6b3f33d64614f489189b다.
제품 CI를 병합 후 다시 시작하지 않았다. 옵션 M으로 archive·asset·오늘할일만 직접 반영한다.
원 #8975의 두 대조군 전 4쪽 inline 비교 PNG·[감사 댓글](https://github.com/edwardkim/rhwp/pull/8975#issuecomment-6845852907)을 게시하고 API 본문과 실제 브라우저 이미지 표시를 확인했다.
원 #8951의 [감사 댓글](https://github.com/edwardkim/rhwp/pull/8951#issuecomment-6844996591)은 이미 게시·확인했다.
#7620은 전체 실물 잔여 범위를 재판정하지 않아 OPEN을 유지한다.
[정리 로그](../assets/sacru2red-20261113/cleanup-9047.md)에 자체 branch/worktree·루트 output 전체·공용 debug의 실제 종료 정리 완료를 기록했다.
원 fork·다른 worktree·사용자 글꼴·로컬 crates.io 소비 환경은 보존한다.
