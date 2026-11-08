---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-11
---

# PR #7706 — Linux IME 조합 종료 후 탐색키 중복 실행 방지

## 최종 판정

**승인 — 사용자 후속 처리 승인 후 원 contributor 코드에 보정 없이 병합했다.** 병합 전 exact head·필수 CI·최신 devel merge-tree를 재확인했고 원 기여 commit을 보존하는 merge 방식을 사용했다. 병합 SHA는 `f07b6bd4a5dcdde3d536eab5d4df52231955007c`, 병합 시각은 `2026-10-11T02:07:56Z`다. #7691은 devel push 종료 작업에 의해 `2026-10-11T02:08:08Z`에 자동 종료됐다.

## 접수와 트리아지

| 항목 | 확인 결과 |
| --- | --- |
| PR / 작성자 | [#7706](https://github.com/edwardkim/rhwp/pull/7706) / studyreadbook4ever; 기존 merged PR 0건으로 첫 기여 절차 적용 |
| 관련 이슈 | [#7691](https://github.com/edwardkim/rhwp/issues/7691), OPEN; PR 본문 `Fixes #7691` |
| 원 head | `666a8288075ebd4d1d2166b6f825ffdcf0065589` |
| PR base | devel / `21ed64071f81f35f37618717693bd30f727392c9` |
| 최신 검토 base | `19245412a43a19baa9ca03a7ef5ca99013fc3242` |
| 로컬 통합 검증 후보 | `1ccfa2fff7b169ffeb49b1b6e06c2385b392c694`; 원 기능 commit 1개를 최신 base에 충돌 없이 cherry-pick |
| 작업공간 | 주 작업공간의 `review/studyreadbook4ever-20261011`; 별도 worktree 생성 없음 |
| 트리아지 | assignee=studyreadbook4ever, milestone=v1.0.0, labels=bug / rhwp-studio / typescript; reviewer=edwardkim. REST 응답 재확인 |
| 변경 규모 | Studio TypeScript·테스트·E2E·npm script 12파일, +477/-102 |
| 최종 조회 참고값 | 원 head와 fork remote SHA 동일, OPEN / MERGEABLE / CLEAN. 최신 base와 원 head의 merge-tree 성공: `c81995e098ed3f079bfdc963ce4407ace19035ef` |

원 PR 자체는 maintainer 일반 경로와 첫 기여자 절차로 처리한다. 로컬 최신-base 통합은 외부 PR 가이드 9.1.1의 가시성 방식을 사용한 추가 검증이다. contributor history를 rewrite하거나 fork에 최신 devel을 push하지 않았다. 로컬 통합 후보는 원 PR head 또는 원격 CI 대상 SHA로 표기하지 않는다.

## 구현과 호출 경로 검토

- `onKeyDown`의 IME 분기는 물리 `code`와 수정자를 `_pendingNavAfterIME` 및 `ImeNavigationGuard.queue`에 보존한다. `compositionend` 뒤 일반 `keydown`이 다시 도착하는 #7691의 실제 fcitx 이벤트 기록이 독립적인 재현 근거다.
- `onCompositionEnd`는 조합 내용을 Undo 이력에 먼저 기록하고 `processPendingNav`를 실행한다. 기존의 별도 본문 탐색 구현을 제거하고 `onKeyDown`을 재사용하여 본문·머리말/꼬리말·각주의 실제 커서 소유자와 Shift 선택, 플랫폼 수정자, 화면 이동 처리를 공유한다.
- 조합 확정 Enter는 문단 분할을 재생하지 않고 pagination barrier만 실행한다. 일반 키 처리에서 `preventDefault`가 호출된 경우에만 `markReplayed`를 설정한다. 머리말/각주 Tab 및 Ctrl+PageUp/Down 등 미처리 키는 브라우저 기본 동작을 유지한다.
- 다음 비조합 keydown은 같은 code와 수정자이고 repeat가 아닌 경우에만 한 번 소비한다. keyup·textarea/window blur·deactivate·dispose에서 상태를 해제한다. 시간 제한, OS 문자열, 문서 ID에 의존하는 분기는 없다.
- 이 PR은 입력 이벤트 라우팅을 바꾼다. Rust, parser/model/serializer, 편집 command 구현, 줄 메트릭·조판 속성·Canvas/SVG/PNG backend는 바뀌지 않는다. 본문 Tab 등의 예약 입력도 기존 일반 command를 실행한다. 같은 논리 편집을 저장·조판하는 계약 자체의 변경은 없으므로 이번 입력 계약 검증에 한컴 Print PDF/페이지별 Visual Sweep은 비해당이다. 한컴 출력 피델리티를 검증했다고 주장하지 않는다.

## 실제 검증 결과

실행 대상은 위 로컬 통합 후보다. 원 PR 변경 파일 중 engine 및 새 테스트의 내용은 원 head와 동일함을 `git diff`로 확인했다. 실행 당시 원문 로그는 ignored `output/pr-review/pr7706-20261011/`에 보존했다. 아래 결과를 영구 기록하고 댓글 본문을 API 대조한 뒤, 종료 절차에 따라 이 PR 전용 임시 로그를 정리했다. 표의 로그 이름은 당시 실행 자료 식별자다.

| 검증 | 결과와 증적 |
| --- | --- |
| TypeScript | `npx tsc --noEmit` PASS (`tsc.log`) |
| Studio 전체 테스트 | `npm test`: 1834 PASS / 0 FAIL / 2 skipped, 총 1836 (`npm-test.log`). 두 skip은 기존 Node WASM 왕복 검사 #6788·대기 서식이며 이번 변경 검사 모두 실행 |
| focused 입력 계약 | guard 3건 및 실제 CursorState/onCompositionEnd/onKeyDown을 사용하는 runner PASS. 본문·머리말·꼬리말·각주 Left/Right/Home/End, Shift/Ctrl, repeat·early keyup·다음 입력, 화면 이동, 미처리 키 기본 동작, Enter 확인 |
| 실제 Chrome CDP | `CHROME_CDP=http://localhost:19222 VITE_URL=http://localhost:7794 node e2e/ime-navigation.test.mjs --mode=host` PASS (`ime-navigation-host-final.log`): 중복 전달/전달 없음, 다음 물리 키, Shift 선택, early keyup, Enter, 조합 Undo |
| 기존 단축키 대조군 | 같은 환경의 `navigation-shortcuts.test.mjs --mode=host` PASS (`navigation-shortcuts-host.log`): Windows/Linux 및 macOS 매핑의 단어·줄 이동과 선택. 실제 macOS/Windows IME 시험으로 분류하지 않음 |
| 수정 전 음성 대조 | 동일 WASM·서버·Chrome, base `19245412…`에서 동일 E2E가 예상 원인으로 FAIL: `6 !== 7` (`base-ime-navigation-host.log`). 조합 후 offset 8에서 Left 한 번에 6으로 이동. 후보 복귀 후 재실행 PASS, offset 7 |
| production build | `npm run build` PASS, 기존 chunk 크기 warning (`build.log`) |
| E2E manifest | `python3 scripts/check_e2e_manifest.py`: tracked 152 / manifest 152, 이상 없음 (`e2e-manifest.log`) |
| diff·호환성 | `git diff --check upstream/devel...HEAD` PASS; current-base merge-tree 성공 |

WASM은 앞선 표준 Docker 빌드(`docker compose --env-file .env.docker run --rm wasm`)의 산출물을 재사용했다. Rust/Cargo/toolchain/wrapper 변경이 없는 것을 확인하고 root `pkg/rhwp_bg.wasm`과 Studio `public/rhwp_bg.wasm`의 SHA-256 동일성을 확인했다: `cc5e9c9d86d2d3ac117a9184510bca661fb8cfaa6aef660707de2c9063575de2`. Docker 빌드 출처는 로컬 #7714 검토 기록에 연결된다. 이번 검토에서 새 WASM 빌드를 실행한 것으로 보고하지 않는다. Rust lint·회귀는 Rust source/test 변경이 없어 비해당이다.

원 head GitHub [Build & Test run 38046513949](https://github.com/edwardkim/rhwp/actions/runs/38046513949)은 success이고 `head_sha`도 원 head와 동일하다. Frontend package gates, Canvas visual diff, CodeQL JavaScript/TypeScript, adapter inter-diff, prop roundtrip이 성공했다. Rust lint/Native Skia/WASM Build는 impact 정책에 따라 skipped다. Frontend unit gates의 별도 job도 skipped지만 package gates 로그에 Studio 전체 테스트 실행이 있다: **1829 PASS / 2 skipped / 0 FAIL, 총 1831**. PR 본문의 기여자 로컬 기록(1,831건 전수 통과, Node WASM 준비)과 Actions/검토자 환경의 결과를 구분한다. 로컬 최신 base에서는 별도 추가된 테스트를 포함해 위 1836건이다.

## 조판 원칙 준수 판정

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 실제 fcitx 이벤트 기록, 입력 상태 전이와 동일 물리 키 기준. 샘플·timeout·UA 예외 없음 |
| 측정·배치 일관성 | 비해당 | 줄 측정·배치 구현 변경 없음 |
| 최종 출력 소비 계약 | 비해당 | 출력 backend·metric/resource 변경 없음; 실제 브라우저 입력 동작은 별도 검증 |
| 분할·이어받기 계약 | 비해당 | pagination 분할 규칙 변경 없음; 기존 입력 barrier 유지 검사 PASS |
| 줄 소속·점유 높이 | 비해당 | LineSeg·줄 구성·높이 계산 변경 없음 |
| 사례·증거 독립성 | 충족 | 기존 OS 신고의 이벤트 기록과 독립 커서 기대값, 최신 base FAIL / 후보 PASS. mock runner와 실제 브라우저를 구분 |
| 기준값 변경 | 비해당 | baseline/golden/허용치 변경 없음 |
| 렌더링 회귀 추가 선행 조건 | 비해당 | 렌더링 fixture·조판 회귀를 추가하지 않음; 새 검사는 입력·커서 소유·중복 소비 계약 |
| 90% 후 확대 판독 | 비해당 | 한컴 픽셀 일치율 판정이나 조판 개선 주장을 하지 않음 |
| 주장·검증 범위 | 충족 | 원 head CI와 최신-base 로컬 결과 분리. 아래 실제 OS·영역 렌더링 한계 명시 |

HWP/HWPX/PDF 검증 입력 커밋 조건은 비해당이다. 이번 직접 재현은 브라우저에서 생성한 `ABCDEF` 본문과 합성 조합 이벤트다. 특정 실제 문서의 조판을 정답지와 대조한 검증이 아니다.

## 검증 한계와 후속 처리

- 검토자는 Arch X11/fcitx 실제 물리 IME 환경을 재구축하지 않았다. Chrome CDP에서 신고된 이벤트 순서를 재생했다. Wayland, 브라우저 확장 내부, 실제 macOS·Windows IME는 미검증이다.
- 머리말/꼬리말·각주의 커서 소유·선택은 실제 CursorState를 사용하는 행위 runner로 확인했으며 WASM 및 화면 geometry는 mock이다. 실제 해당 영역의 글리프/배치를 확인한 것으로 보고하지 않는다.
- 원 head의 적용 코드에서 차단 결함은 발견하지 않았다. 위 미검증 환경을 전체 OS 보장으로 확대하지 않는다.
- 작업지시자의 merge·운영 기록 반영·후속 댓글 승인을 받아 원 head와 필수 CI 및 최신 base를 재확인한 뒤 병합했다. merge parent에 원 기여 SHA가 포함됨을 확인했다. 원 코드 PR의 CI는 병합 전에 끝났으며 병합 후 검증 CI는 시작하거나 재실행하지 않았다. 검토 기록은 maintainer 일반 경로의 운영 기록 직접 반영으로 archive review와 오늘할일만 devel에 보존한다.
- 기록 반영 뒤 기여자·이슈 댓글에 실제 merge SHA, CI 링크, 검증 요약과 미검증 범위를 남긴다. 본문 파일을 `--body-file`로 전달하고 API로 UTF-8 본문·BOM/치환 문자 여부를 재확인한다. 첫 기여 감사와 원 commit 보존을 명시한다. contributor fork와 공용 target은 보존하고 이 PR의 검토 서버·임시 branch/ref·로그만 종료 절차에 따라 정리한다.

## 후속 처리 완료

- 사용자 승인 후 원 PR merge와 devel 반영을 확인했고, archive review·오늘할일만 운영 기록으로 직접 반영했다. contributor source에 추가 push·rewrite는 하지 않았다.
- [이슈 검증 요약 댓글](https://github.com/edwardkim/rhwp/issues/7691#issuecomment-6104488961), [첫 기여 감사·병합 댓글](https://github.com/edwardkim/rhwp/pull/7706#issuecomment-6104490984)을 게시했다. 각 본문은 API 응답과 UTF-8 파일이 완전히 같고 BOM·치환 문자·`??`가 없으며 작성자가 edwardkim임을 확인했다.
- 코드 병합 후 [duration 갱신 run 38104148616](https://github.com/edwardkim/rhwp/actions/runs/38104148616)은 success다. 실제 수집 결과는 `ready:false / no-verified-pr-duration-measurements`로 자료 부족에 따른 갱신 보류이며, 해당 PR의 Rust worker가 skipped인 결과와 일치한다. 필수 검증 CI의 실패로 분류하지 않는다. 병합 후 검증 CI를 재실행하지 않았다.
- 이 작업이 만든 `review/studyreadbook4ever-20261011` branch와 `upstream/pr7706-head` 임시 fetch ref를 정리했다. 코드 후보와 실제 병합본의 mydocs 밖 diff가 없고 검토 기록이 devel에 존재함을 먼저 확인했다. 별도 worktree는 만들지 않았다. 검토 Vite 7794 서버를 종료했다.
- 종료 시 이 PR 전용 ignored 로그·스크립트 폴더를 정리했다. 기본 작업공간·기존 다른 검토 worktree·공용 `target/pr-review`와 WASM 산출물은 보존했다. `studyreadbook4ever/rhwp`의 `fix/linux-ime-navigation-devel` fork branch는 삭제하지 않았다. 최종 로컬 branch는 devel이다.
