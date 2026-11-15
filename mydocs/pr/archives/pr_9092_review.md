# PR #9092 — 작은 화면의 제품 정보 창 닫기

## 최종 판정

**승인.** 최신 devel에서도 재현되는 제품 정보 창 잘림을 원 기여가 해결한다. 메인테이너 제품 보정 없이 수용 가능하다. 원 head `c3b0b7191b91e7df57a80da81a43cf632059a609`의 CI가 성공했고, 최신 devel `d954632aab5b80aadbea39cece4f5a783634856b`와 로컬 병합한 `7fbed378bcf3fd2c20f85234610ec094867b2df7`에서 적용 검증을 완료했다.

작업지시자 승인 뒤 [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9092#pullrequestreview-5911674249)를 게시하고 2026-11-16 01:17 KST에 `4c4bd16a7a9bc8b00738e6c3cd74bca04ab3345c`로 원 PR을 정상 merge했다. #8905는 bot이 completed 자동 종료했다. 제품 보정이나 원 fork push 없이 원 기여를 통합했다.

## 접수와 범위

- [PR #9092](https://github.com/edwardkim/rhwp/pull/9092), 작성자 `JinHeeWond`, 원 fork `JinHeeWond/rhwp`, branch `task/8905-about-dialog-overflow`. rhwp PR 조회 결과 첫 PR이다. reviewer `edwardkim` 요청을 API로 확인한 뒤 상세 검토·fetch를 수행했다.
- 관련 이슈는 [#8905](https://github.com/edwardkim/rhwp/issues/8905)이며 현재 OPEN이다. Android Firefox의 「파일 → 제품 정보」에서 위아래가 잘려 닫을 수 없다는 제보다. 기존 해결 PR은 찾지 못했고, 최신 devel의 실제 실행에서도 같은 결함을 재현했다.
- 원 변경은 6개 파일·129줄 추가/1줄 삭제다. 제품 정보 창의 CSS, 기존 `closeOnOverlayClick` 인자, E2E·목록·npm 실행 항목을 변경한다. 다른 대화창의 공통 구현·문서 parser/조판/paint/저장·Rust·WASM API 변경은 없다.
- 경로는 maintainer_general + intake_and_review·local_validation·first_time_contributor·post_merge다. CI·문서·Git·개발 환경의 현행 정본을 확인했다.

## 구현 검토

`file:about` → `AboutDialog.show()` → 공통 `ModalDialog.show()`의 overlay/제목/본문/footer를 사용한다. 제품 정보 창에만 `about-dialog` class를 추가하고 flex column·viewport 최대 높이를 적용한다. 제목/footer는 줄어들지 않고, 본문은 `min-height: 0`·`overflow-y: auto`로 스크롤된다. 767px 이하에서는 기존 하단 시트 폭·90dvh 규칙을 유지한다.

배경 클릭은 기존 공통 옵션의 세 번째 인자 `true`로 활성화한다. 실제 overlay 자체를 클릭했을 때만 닫히므로 창 안의 클릭은 전파돼도 닫히지 않는다. 공통 모달의 기본값 `false`는 그대로다. 문서별 상수·허용치·렌더링 우회는 없다.

## 실제 검증

| 검증 | 결과 |
| --- | --- |
| 최신 devel과 원 head 병합 | 충돌 없음, tree `48ac736638d6e6e2411c9f01e6a784543bf925cd` |
| fresh dev WASM | locked wrapper, 공용 `/home/edward/mygithub/rhwp/target/pr-review`, 약 170초, 성공 |
| 실제 앱 초기화 WASM | HTTP 200과 SHA-256 `ceecc297f01a6e80dff2c6a86b434d2610e8ca8f8cbb97361dbc7f72a22043d3` 일치, 5개 관측 모두 성공 |
| TypeScript | `tsc --noEmit` 통과 |
| Studio 전체 | 2,798 PASS / 0 FAIL / 기존 2 SKIP |
| production build | 통과 |
| 기여자 E2E | `npm run e2e:issue-8905`, 19 PASS / 0 FAIL. base 대조 후 후보를 복원해 재실행해도 통과 |
| E2E manifest | tracked 192개와 목록 192행 일치 |
| 추가 실제 브라우저 | 800×360, 768×320, 767×360, 360×640, 320×240, 1280×900의 창·닫기 버튼 포함, 터치/배경/Escape 닫기 통과 |
| 메뉴·공통 기본값 | 실제 파일 메뉴에서 열기/닫기, 다른 기본 모달의 배경 클릭 비닫힘 통과 |
| 새 devel #9097 계약 | `frontend-canvas-gates.test.mjs` 4 PASS |

최신 devel에서는 800×360의 대화창이 y=-196.1875~556.1875, 제목 닫기가 -188.6875, 하단 버튼이 517.1875라 두 닫기 수단이 화면 밖이다. 같은 새 WASM과 브라우저에서 후보는 창이 y=12~348, 하단 버튼이 309~335로 화면 안에 들어간다. 같은 E2E는 base에서 해당 세 배치 검사 실패 후 실제 버튼 클릭으로 닫지 못해 종료 코드 1이고 후보는 종료 코드 0이다. 본문 끝 저작권 줄까지 스크롤되는 것도 기여자 검사에서 확인했다.

대표 스크린샷을 직접 열어 가로 화면의 제목·하단 닫기 유지, 세로 하단 시트의 본문 스크롤과 버튼 경계를 확인했다. [수정 전](../assets/pr9092/base-800x360.png) · [수정 후 가로](../assets/pr9092/candidate-800x360.png) · [세로](../assets/pr9092/candidate-360x640.png) · [데스크톱](../assets/pr9092/candidate-1280x900.png) · [측정·프로그램·해시](../assets/pr9092/validation.json).

처음 TypeScript를 package 생성 전에 호출해 `@wasm/rhwp.js` 부재로 실패했다. fresh 빌드 후 동일 명령과 전체 검사를 재실행해 통과했다. base 진단의 소스 3개는 실행 후 복원했고, wrapper가 생성한 tracked `public/rhwp.js`도 검증 종료 후 검토 HEAD로 복원했다. 환경 준비/생성물 상태를 제품 결함으로 분류하지 않았다.

## CI와 새 base 기능 보존

원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/43696397797)·[CodeQL 분석](https://github.com/edwardkim/rhwp/actions/runs/43696397799)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43696397769)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43696397766)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43696397891)와 정책 상태가 success다. Rust 전체/Native/lint의 조건부 skip은 Studio 전용 변경에 따른 실제 선택 범위이며 실행했다고 쓰지 않는다.

CodeQL의 네 언어 분석 job은 모두 success다. 별도 CodeQL 비교 check에는 현재 devel의 분석 구성 세 개를 찾지 못한다는 `neutral` 안내가 남는다. 이를 취약점 검출이나 네 분석 실패로 표시하지 않으며, 해당 비교 check를 success라고도 주장하지 않는다.

원 PR의 base 이후 들어온 #9083·#9088·#9096의 Rust 변경은 이 PR이 수정하지 않는다. 실제 빌드는 최신 devel의 Rust를 사용한다. #9093의 frontend 글꼴 선택은 최신 Studio 전체 2,798건과 실제 초기화로, #9097/#9099의 frontend Canvas 분리 계약은 4개 계약 검사로 확인했다. 원 PR의 공통 CSS/모달과 새 devel 사이에 충돌·소스 덮어쓰기는 없으며, 현재 개선과 새 base의 Studio 검사가 함께 통과했다.

## 공통 심사와 한계

| 항목 | 판정·근거 |
| --- | --- |
| 조판 원칙·분할·줄 메트릭·저장 정보·backend 출력 | 비해당. UI 모달 DOM/CSS만 변경하고 문서 출력 소비 경로는 수정하지 않음 |
| HWP/HWPX/PDF 입력 커밋·전쪽 TSV·편람·보호 하한 | 비해당. 문서 입력/fixture/시각 회귀를 사용하거나 추가하지 않음 |
| UI 검사의 독립성 | 충족. 수정 class 없이 구조 selector로 찾고 동일 검사가 최신 base에서 실제 실패함 |
| 주장과 검증 범위 | 충족. Chromium 152(1660786)와 터치 에뮬레이션, Node 24.15.0/WSL 호스트 검증. Android Firefox 실기기 검증이라고 주장하지 않음 |
| 새 code head·source 보정 | 없음. 로컬 병합과 review/asset 추가만 하며 원 fork/head는 보존 |

## Merge 후 contributor PR comment 계획

원 PR에는 첫 기여에 대한 감사와 환영을 포함한 한국어 존댓말 댓글을 게시한다. 창 높이/본문 스크롤/네 닫기 수단의 기여와 최신 devel 재현·검증을 설명하고, 메인테이너 제품 보정이 없었음을 정확히 적는다. 문서 Visual Sweep 수치를 UI 검사에 인용하지 않는다.

승인된 원 PR merge → duration 결과 확인 → archive·오늘할일·asset 운영 기록 반영 → devel 동기화 → #8905 자동 close 확인/필요한 완료 close와 근거 댓글 → 원 PR 감사 댓글 → 전용 worktree·branch·output/debug 정리 순서다. fork branch는 유지한다. 댓글에는 실제 merge SHA, 성공 CI, 안정 경로의 수정 전후 UI PNG를 사용하고 게시 후 API 본문·이미지 표시를 확인한다.

## 원격 통합과 후속 기록

- 정확한 원 head·필수 check·정책 상태를 다시 확인했고 최신 devel `d954632aab5b80aadbea39cece4f5a783634856b`의 병합 시뮬레이션이 exit 0이었다. 원 PR merge 직전 상태는 CLEAN이었다.
- [duration 자동 작업](https://github.com/edwardkim/rhwp/actions/runs/43734626476)은 success다. 새 실측이 없어 `no-verified-pr-duration-measurements`로 갱신을 보류하고 기존 metrics를 유지했다. [수집 근거](../assets/pr9092/duration.json)를 확인했으며 검증 CI는 재실행하지 않았다.
- 메인테이너 옵션 M으로 이 archive·UI asset·오늘할일만 `30f97a0edd5ba515e6a7f4055bc4ab54bfa6ee8d`로 devel에 반영했다. 소스·test·workflow·baseline 변경은 없다.
- [이슈 상태](../assets/pr9092/issue-state.json)는 CLOSED/completed다. 운영 기록 반영·devel 동기화 후 [이슈 완료 댓글](https://github.com/edwardkim/rhwp/issues/8905#issuecomment-6897580190)과 [첫 기여 환영·감사 댓글](https://github.com/edwardkim/rhwp/pull/9092#issuecomment-6897580874)을 게시했다. API 본문과 실제 GitHub 화면의 이미지 3개씩·총 6개 표시를 확인했다.
- 원 fork `JinHeeWond/rhwp`의 `task/8905-about-dialog-overflow`는 보존한다. 전용 worktree `/home/edward/mygithub/rhwp-worktrees/pr-9092-about-dialog-overflow`·로컬 review branch·전용 fetch ref·루트 `output/`·review debug를 사용 종료 후 제거했다. 약 12.17 GiB를 확보했고 종료 시 여유는 127.68 GiB다. [댓글·브라우저·정리 영수증](../assets/pr9092/closure.json)에 정확한 삭제 경로·전후 용량을 보존했으며 로컬 최신 동기화 로그는 `local/pr-9092-closure/closure.json`이다.
