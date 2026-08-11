# task_m100_4579_stage1 — subsecond 수명 관리 4건

이슈 [#4579](https://github.com/edwardkim/rhwp/issues/4579) 의 네 항목을 수명(lifecycle) 범위에서만
처리한 단계 보고서다. 무효화 계약(#4576), 핫패치 경계(#4577), 진단·로깅(#4578), 번들·명명(#4580)은
건드리지 않았다.

## 1. 재도색 예외가 감시 루프를 영구히 멎게 하던 문제 — 고침

`rhwp-studio/src/core/subsecond-runtime.ts` 의 `SubsecondRevisionWatcher.schedule()` 이
`checkRevision()` **뒤에서** 재무장하고 있었다. 재도색 경로(`onPatched` → `eventBus.emit` →
`EventBus.emit` → `CanvasView.refreshPages()`)에는 보호가 없어 예외가 재무장 전에 rAF 콜백 밖으로
풀리고, `running=true`·`frameId=null` 이 남는다. 이 상태에서 `start()` 는 `if (this.running)` 로 조기
반환하고 `stop()` 은 취소할 것을 못 찾는다 — 세션이 끝날 때까지 감시가 멎고 로그도 없다.

`try`/`finally` 로 재무장을 보장했다. 예외는 삼키지 않고 그대로 흘려보내 브라우저가 보고하게 둔다.

`EventBus.emit`(`rhwp-studio/src/core/event-bus.ts:17`)의 핸들러 격리는 **하지 않았다.** 이벤트 버스는
저장소 전역 계약이고, 한 핸들러의 예외를 삼키면 subsecond 와 무관한 30여 개 구독자의 실패가 조용해진다.
감시 루프의 생존은 루프 자신이 보장하는 것이 맞다. 별도 판단이 필요하면 전용 이슈로 분리한다.

## 2. `stop()` 이 죽은 코드이고 테스트가 덮던 문제 — 테스트를 고침

`tests/subsecond-runtime.test.ts` 가 `canvas-view.ts` 의 소스 텍스트에서
`subsecondRevisionWatcher\.stop\(\)` 를 찾던 단언을 지우고, 감시자의 해제 계약을 행동으로 검증하는
테스트로 바꿨다(정지가 프레임을 실제로 해제하고, 정지 뒤에는 그리지 않으며, 다시 시작할 수 있다).

호출부는 새로 만들지 않았다. 근거:

- `CanvasView.dispose()` 는 호출부가 0개다. `canvasView` 는 `main.ts` 모듈 바인딩이고 `initialize()` 는
  모듈 로드 때 한 번 실행된다. 스튜디오에는 문서 닫기도 뷰 폐기도 없다(`destroy`/`teardown` 검색 결과 0).
- `pagehide`/`beforeunload` 에 `dispose()` 를 거는 것은 더 나쁘다. bfcache 로 복원되는 페이지에서
  폐기된 `CanvasView` 가 되살아나고, 어차피 realm 이 사라지는 시점의 해제는 의식일 뿐이다.
- 그래서 감시자의 수명은 realm 과 같다는 사실을 `SubsecondRevisionWatcher` 주석에 적고, 해제선
  (`stop()`)은 그대로 뒀다 — 종료 경로가 생기면 그 자리가 유일한 해제선이다.

## 3. 패치 누적 — 셈과 경고를 추가

`commit_patch`(`subsecond-0.7.10/src/lib.rs:308-312`)는 이전 `Box<JumpTable>` 을 버리고,
`memory.grow`(`:628`)/`funcs.grow`(`:632`)로 늘린 선형 메모리와 간접 함수 테이블은 줄어들 수 없다.
회수는 플랫폼 제약상 불가능하므로 누적을 보이게만 한다.

- `SubsecondPatchBudget` 이 적용한 패치를 세고 32건마다 경고한다. 경고에는 그 순간 측정한 wasm 선형
  메모리 크기를 함께 담는다(추정값이 아니라 실측값).
- 세는 자리는 소켓의 `onmessage` 다. `applySubsecondDevtoolsMessage` 가 참을 돌려준 메시지, 즉 이
  build 를 위한 `HotReload` 만 센다. wasm 에서 적용은 비동기라 이 수는 "적용을 시작한 패치 수"다.
- 세션 길이 제약은 [개발 환경 가이드](../manual/dev_environment_guide.md#subsecond-핫패치-세션은-길이를-관리한다)에 적었다.

패치 1건당 증가량은 **실측하지 못했다**. 이 장비에는 dx 가 만든 패치 산출물이 없고(`target/dx/` 에
베이스 모듈만 있다), 새로 만들려면 워크트리에서 wasm 디버그 전체 빌드부터 필요하다. 대신 확실한 값만
적는다 — 이 장비의 베이스 모듈은 123,051,645 byte(serve 산출물)이고, 패치 1건이 더하는 선형 메모리는
`(ceil(패치 byte / 64KiB) + 1) × 64KiB` 로 코드에서 정확히 결정된다. 패치 크기는 상류가 "routinely
cross 8MB"(`lib.rs:659-663`)라 적어 둔 추정에 의존한다 → 8.45MB/건은 추정이다. 런타임 경고는 이
추정을 쓰지 않고 선형 메모리를 직접 읽는다.

## 4. 소켓 해제와 백오프 — 백오프 리셋을 추가

`reconnectDelay` 는 `onclose` 마다 두 배로 늘기만 하고 줄어드는 자리가 없었다(`onopen` 핸들러가 없고
`WebSocketConnection` 에 선언조차 없었다). `dx serve` 를 껐다 켜면 백오프가 상한 4초에 붙은 채 남아
남은 세션 내내 첫 패치가 최대 4초 늦는다. 연결이 열린 순간 최소 대기로 되돌린다.

소켓 해제 함수는 그대로 뒀다. realm 하나에 소켓 하나이고 `wasm-bridge.ts` 의 중복 연결 guard 가 그
사실을 지킨다. 호출부가 없다는 사실 자체는 주석으로 적었다 — 없는 종료 경로를 만들어 내지 않았다.

## 검증

| 게이트 | 결과 |
| --- | --- |
| `npx tsc --noEmit` (rhwp-studio) | 통과(출력 없음) |
| `npm test` (rhwp-studio) | 841개 중 840 통과, 0 실패, 1 skip(환경: pkg-node 빌드 없음, 기존) |
| `python3 scripts/check_document_metadata.py` | 557개 문서, 이상 없음 |
| `python3 scripts/check_markdown_links.py` | 562개 문서, 이상 없음 |

수정 전 RED 확인(`node --test tests/subsecond-runtime.test.ts`):

- 1번 — `재도색이 던져도 감시 루프는 다음 프레임을 다시 예약해야 한다`: `0 !== 1`
- 4번 — 백오프 리셋: `[250, 500, 1000]` ≠ `[250, 500, 250]`
- 3번 — `TypeError: SubsecondPatchBudget is not a constructor`
- 2번 — 대체 테스트는 행동 계약이라 수정 전후 모두 초록이다. RED 근거는 지운 단언 쪽에 있다:
  `CanvasView.dispose()` 호출부가 0개인데도 그 단언은 통과한다.

브라우저 실동작은 확인하지 못했다. `dx serve --hot-patch` 로 핫패치 세션을 띄우려면 이 워크트리에서
wasm 디버그 전체 빌드가 선행되어야 한다.
