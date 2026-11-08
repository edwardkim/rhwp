# #8104 Studio 명령 오류·복구 계약 결과

Issue: [#8104](https://github.com/edwardkim/rhwp/issues/8104).
브랜치: `codex/issue-8104-command-error-contract`.
기준: `6097253468`. 검증한 제품 코드·테스트 SHA:
`fc77d9e7e5ac6218f8a43d8f4da032333d9bfd0d`.
후속 커밋은 MANIFEST 열거값 보정과 이 결과보고만 포함한다.

## 결과

공통 명령 경계가 실패를 typed exception으로 유지하면서 버전 1 payload를 통지한다.
명령 소유자가 확인한 snapshot rollback 또는 mutation 전 거부에 커서·선택·history·dirty·
실제 화면 갱신·엄격한 커서 질의를 결합한 경우에만 `succeeded`다. rollback 실패는 `failed`,
부분 변경·trap·검증 실패·미지원 복구 경로는 `unknown`으로 구분한다.
같은 실패를 다시 throw하거나 이벤트와 응답에서 받더라도 이벤트 ID는 같다.
문서 세대가 달라진 늦은 완료는 현재 문서를 복원·차단하지 않는다.

직접 호스트는 `window.rhwpCommandErrors`와 `rhwp-command-error`를 사용한다.
MessagePort는 `command-error-v1` capability와 `getCommandErrorState`를 사용한다.
소유한 typed error만 전역 중복 처리를 막고 일반 초기화·로드·리소스 오류는 기존 경로에 남긴다.
상세 payload, 지원 범위, host 소비·차단 책임은
[계약 문서](../tech/studio_command_error_contract.md)에 기록했다.

## 선행 구현 보존

#3350 원 PR #3656 → 통합 #3661(`e5d3049199`) → #3662 source guard 강화(`2deb282081`)의
원인·검토 기록·종료 댓글을 확인했다. before 복원·ID 해제·원 오류 전파, 이중 실패 AggregateError와
#5769의 after 지연 저장을 유지한다. 기존 source guard의 restore/discard/rethrow 조건을 유지하고
그 사이에 추가한 outcome 기록도 확인하도록 바꿨다. 일반 정상 명령의 undo/redo 정책을 바꾸지 않았다.

## 실제 검증

환경: macOS arm64, Node `v24.15.0`, Rust `1.93.1`, 로컬 Google Chrome headless.
로그 기준 위치: `output/pr-review/issue-8104/`.

| 실행 | 결과 |
| --- | --- |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` | PASS. 현재 base의 fresh WASM·Studio 동기화 |
| `npx --prefix rhwp-studio tsc --project rhwp-studio/tsconfig.json --noEmit` | PASS |
| `npm --prefix rhwp-studio test` | **2,440 PASS / 0 FAIL / 0 skip** |
| `npm --prefix rhwp-studio run build` | PASS. chunk 크기 안내 경고 있음 |
| `node e2e/run-with-vite.mjs -- node e2e/command-error-contract.test.mjs --mode=headless` | PASS. 아래 실패·복구·호스트 시나리오 |
| `node e2e/run-with-vite.mjs -- node e2e/cell-target-issue8102.test.mjs --mode=headless` | PASS. 기존 정상 셀 진입·선택 삭제·undo/redo, errors 0 |
| `git diff --check` | PASS |
| E2E MANIFEST 대조 | 신규 오류 0. 기준 SHA에도 있던 #7998 파일 미등재 1건은 동일 |
| 변경 문서 메타데이터 검사 | PASS. 전체 검사에는 변경하지 않은 기존 5개 문서의 누락 20건이 있음 |
| 변경 문서 내부 링크 검사 | PASS |

실행 시 `CHROME_PATH`는 설치된 Chrome 실행 파일로 지정했다. 전체 npm test는 자식 프로세스
제약을 피하도록 저장소 지침에 따라 샌드박스 밖에서 실행했다. 초기 pkg의 오래된 WASM 타입 오류는
현재 소스의 fresh 빌드로 해결했다. 검증용으로 생성한 tracked public glue는 커밋하지 않는다.

| 브라우저 시나리오 | 판정·관측 |
| --- | --- |
| clean 문서 snapshot 부분 변경 실패 | succeeded. HWP 바이트·cursor/selection·양 history 스택·dirty 보존, 후속 편집/undo 성공 |
| dirty 문서와 redo가 있는 상태의 실패 | succeeded. 동일 보존 검사와 후속 편집/undo 성공 |
| 양식 모드의 mutation 전 거부 | COMMAND_REJECTED, 내용·편집 상태 보존 |
| snapshot restore 자체의 실패 | ROLLBACK_FAILED, 실패와 부분 변경을 성공으로 응답하지 않음 |
| rollback 지원 없는 명령의 부분 변경 | INTEGRITY_UNKNOWN, 공통 명령 경계 blocked |
| RuntimeError/trap | snapshot이 복원돼도 INTEGRITY_UNKNOWN |
| 실제 render 함수 실패 | INTEGRITY_UNKNOWN |
| cursor 조회 실패·기존 fallback | 엄격한 복구 질의가 실패하므로 INTEGRITY_UNKNOWN |
| 표 셀 `alpha` 부분 삭제 후 예외 | 바이트와 선택 복원, 후속 실제 선택 삭제·undo 성공 |
| 복구 render 대기 중 실제 문서 교체 | 오래된 payload는 stale, 새 문서는 ready 유지 |
| 실제 window error | typed failure의 defaultPrevented 확인, 구조화 보고 succeeded. Vite 진단 리스너의 로그는 별개 |
| MessagePort 호스트 | capability·현재 문서 세션 일치, accept/duplicate/stale/invalid/unsupported 판정 |

새 단위 검사는 snapshot/history 실제 실행과 실패 주입, 양 스택 보존, 후속 redo/undo,
편집·history·dirty·render·probe 실패, 이전 세션, 전역 오류 소유권, 미협상 연결과 실패 RPC의
error-only 응답을 검사한다. 사용자 정의 실패를 주입했으며 실제 엔진 panic을 유발했다는 주장은 하지 않는다.
기존 #8102 회귀는 정상 셀 생성·클릭·키보드·undo 진입·legacy flat·다중 문단·중첩 경로를 확인한다.

Rust 조판·모델·serializer·정상 출력 규칙과 픽셀 fixture는 바꾸지 않았다. 화면 검증은 오류 뒤
실제 visible canvas 갱신과 상태 보존에 관한 것으로, 독립 한컴 PDF와의 조판 일치율 개선을 주장하지 않는다.

## 전달 상태와 남은 통합 경계

로컬 구현과 검증을 완료했다. remote push, PR 생성, upstream 반영 및 이슈 close는 수행하지 않았다.
실제 alhangeul WKWebView 소비 구현·앱 번들 반영·배포는 이 저장소에서 수행하지 않았으며 #594의
별도 통합 검증 대상이다. 호스트는 계약을 지원하는 번들인지 확인하고 미지원·잘못된 payload를
복구 성공으로 추측하지 않아야 한다. raw WASM·별도 저장 API 차단과 안전한 복구본·재시작 UX도
호스트 책임이다.
