# PR #9047 리뷰 — sacru2red 표 간격·회귀 통합

## 최종 판정

**승인 — 병합 완료.** exact head `68c2441bfd947609676b841e7ff43bf1c64983ab`의 모든 필수 CI와
셀프 리뷰를 확인하고 merge commit `a785e23f6838d5452e8904cae82f81eb5e185efc`로 devel에 통합했다.
원 #8951은 MERGED이며 #8975의 새 head `66900a40be` 추가분도 후속 #9050으로 수용했다.
이 기록의 주된 검증 범위는 구 head이며 최신 head의 근거는 [#8975 최종 기록](pr_8975_review.md)으로 구분한다.

## source·검증·승인

- 사용자 승인: 후보 `68c2441bfd`의 push·Open 수용 PR·CI 성공·self-review·merge와 후속 댓글·기록·동기화·정리.
- 기준 devel `180a4ec97205b67c4dc4f90416063c01ecfdf396`, 제품 source `e5a6a1ff9f`, 회귀 보완 `b4b8c27260`.
- [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9047#pullrequestreview-5886317032)를 COMMENT로 게시했다. 34 SUCCESS·정상 SKIP 5건, MERGEABLE/CLEAN과 최신 base 병합 시뮬레이션을 직전 재확인하고 exact head를 지정해 merge했다.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/43364755431)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43364755447)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43364755162)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43364755161)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43364755178)·[최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/43367964439)가 모두 성공했다. 재실행·CI 정의 보정·검증 예외는 없었다. 실제 로그와 PR merge ref의 open CodeQL 경고 0건도 확인했다.
- 최소 글꼴 로컬 집중 8 PASS, 전체 release-test 13,432 PASS, Native 공식 4,073 PASS와 lint·build·unit tier·suite manifest 검사가 통과했다. 로컬 전체 nextest는 실제 최종 Summary·모든 PASS를 확인했고 wrapper SIGTERM으로 exit 영수증이 없는 점은 [원장](../assets/sacru2red-20261113/final/README.md)에 구분했다.
- Native/fresh WASM 목표 4쪽 각각 100.00%, footer 13쪽은 devel과 PNG 동일·최저 96.17%/96.11%, 원 a/None 각 1쪽 양쪽 100.00%다. SVG 실제 browser raster 목표 4쪽도 100.00%다. Native 편람 383쪽 gate와 보호 전 3,542쪽의 공식값 하락 0건을 확인했다. 원본·독립 Print·전쪽 TSV와 직접 판독은 위 원장에 있다.
- [병합 후 duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/43368245695)은 `ready:true`, `successful-pr-worker-measurements`와 정책 게시 성공을 확인했다. 게시 commit은 `8cee69b6fb8ffa58a4126b1b243d2d47617c73cd`이며 제품 CI를 병합 후 새로 실행하지 않았다.

## 조판 원칙과 새 head의 구분

공통 커서가 실제 소비한 빈 줄 간격을 lazy 상대 원점에서 다시 계상하지 않도록 했다.
원점이 있으면 그 축의 일치 검사를 유지하고, 합성·비연속·편집 경로는 기존 bridge를 보존했다.
문서 ID·목표 px·좌표 clamp로 맞추지 않았다. 원 입력은 rhwp 생성본이며 실제 한컴 저장본으로 표현하지 않았다.

기여자의 새 head는 기존 9개 HWP 스트림을 보존하고 `/RhwpLineSegTags` 40바이트 표지를 추가하며,
내용 순서·셀 소유·포함·반례 판정력 검사를 보강했다. 새 head는 승인 후보 제출 직전에 재확인하지 못했다.
`#9047`이 보존한 구 head `286eb32e08`과 새 head를 구분하고 추가분을 누락한 채 원 PR을 닫지 않는다.
표지 입력은 보정 전에도 정상이므로 그 성공을 기존 간격 보정의 효과로 세지 않는다.
구형 표지 없는 입력의 호환 보정과 새 표지 경로를 각각 검증하며, 실제 한컴 재저장 정상 대조군도 확인한다.

## 후속 처리

CI 실행 정의 변경이 없으므로 옵션 M으로 이 archive·원 #8951 archive·오늘할일을 보존한다.
원 #8951의 footer 11쪽 inline 증적·[감사 댓글](https://github.com/edwardkim/rhwp/pull/8951#issuecomment-6844996591)을 게시하고 API 본문을 재확인했다. #8975는 #9050 merge로 최신 head까지 MERGED다.
#7620은 기존 56345 실물 문서 등 전체 잔여 범위를 모두 재판정하지 않아 OPEN을 유지한다.
전체 종료 정리는 [정리 로그](../assets/sacru2red-20261113/cleanup-9047.md)에 실제 삭제와 최종 확인을 기록한다.
다른 worktree·원 fork·사용자 글꼴·로컬 crates.io 소비 검증 환경은 보존한다.
