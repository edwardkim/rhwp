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
같은 계열의 반대 결함도 함께 막았다 — `schedule()` 이 `frameId` 를 무조건 덮어쓰고 있어, 재도색 안에서
`stop()`·`start()` 가 불리면 `finally` 의 재예약이 그 예약을 덮어써 **취소할 수 없는 두 번째 루프**가
남았다. 예약이 이미 있으면 새로 잡지 않는다.

`lastRevision` 은 재도색보다 **먼저** 올린다(의도). 실패한 리비전을 매 프레임 다시 그리면 초당 60번
실패하므로, 그 리비전은 다음 패치까지 다시 시도하지 않는다. 세션은 살아 있고 다음 저장이 만드는 새
리비전부터 정상으로 돌아온다. 실패를 표면화하는 일은 #4578 의 몫이다.

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

- `SubsecondPatchAccumulation` 이 적용한 패치를 세고 32건마다 경고한다. 경고에는 그 순간 측정한 wasm
  선형 메모리 크기를 함께 담는다(추정값이 아니라 실측값). 임계 기준은 패치 수다 — 선형 메모리는 큰
  문서를 열어도 커지므로 그것으로 임계를 잡으면 핫패치가 아닌 사용을 핫패치 탓으로 돌리게 된다.
- 세는 자리는 소켓의 `onmessage` 다. `applySubsecondDevtoolsMessage` 가 참을 돌려준 메시지, 즉 이
  build 를 위한 `HotReload` 중 점프 테이블 역직렬화까지 성공한 것만 센다. wasm 에서 `apply_patch` 는
  future 를 띄우고 바로 `Ok(())` 를 돌려주고(`lib.rs:551`) 그 future 는 `.wasm` 이 아닌 경로에서 조용히
  빠져나갈 수 있으므로(`:565-567`), 이 수는 실제 메모리 증가 횟수의 **상한**이다.
- 세션 길이 제약은 [개발 환경 가이드](../manual/dev_environment_guide.md#subsecond-핫패치-세션은-길이를-관리한다)에 적었다.
- 세는 자리가 `onmessage` 라서 #4578(메시지 처리 경로 소유)과 **같은 hunk 를 건드린다.** 셈이 정확하려면
  반환값을 봐야 해서 피할 수 없었다. #4578 이 그 반환값의 의미를 다시 정의하면 이 한 줄도 같이 바뀐다.

패치 1건당 증가량은 **실측하지 못했다**. 이 장비에는 dx 가 만든 패치 산출물이 없고(`target/dx/` 에
베이스 모듈만 있다), 새로 만들려면 워크트리에서 wasm 디버그 전체 빌드부터 필요하다. 대신 확실한 값만
적는다 — 이 장비의 베이스 모듈은 123,051,645 byte(serve 산출물, `target/dx/.../wasm/`)이고, 패치 1건이
더하는 선형 메모리는 `(ceil(패치 byte / 64KiB) + 1) × 64KiB` 로 코드에서 정확히 결정된다(`lib.rs:623`).
패치 크기 자체는 상류가 "patches routinely cross that[8MB]"(`lib.rs:604`)라 적어 둔 기술에 의존한다 →
**8.45MB/건은 추정이다.** 런타임 경고는 이 추정을 쓰지 않고 선형 메모리를 직접 읽는다.

(이슈 본문이 인용한 `lib.rs:659-663` 은 이 문장이 아니다. 그 자리는 동기 `WebAssembly.Instance` 의 8MB
상한을 설명한다. 원문은 `:604`.)

## 4. 소켓 해제와 백오프 — 백오프 리셋을 추가

`reconnectDelay` 는 `onclose` 마다 두 배로 늘기만 하고 줄어드는 자리가 없었다(`onopen` 핸들러가 없고
`WebSocketConnection` 에 선언조차 없었다). `dx serve` 를 껐다 켜면 백오프가 상한 4초에 붙은 채 남아
남은 세션 내내 첫 패치가 최대 4초 늦는다.

되돌리는 조건은 "핸드셰이크 성공"이 아니라 **"오래 붙어 있던 연결이 끊겼다"** 로 잡았다. 첫 구현은
`onopen` 에서 바로 되돌렸는데, 적대 리뷰가 그 경우의 회귀를 잡아냈다 — Vite 프록시는 살아 있고
`dx serve` 만 죽어 열자마자 끊기는 흔한 상황에서, 4초 폴링이 **250ms 무한 재연결**로 바뀐다(측정:
`[250,500,1000,2000,4000,…]` → `[250,250,250,…]`). 최대 백오프(4초)보다 오래 버틴 연결만 되돌리므로
두 경우가 갈린다. 테스트가 두 실패를 모두 잡는다.

소켓 해제 함수는 그대로 뒀다. realm 하나에 소켓 하나이고 `wasm-bridge.ts` 의 중복 연결 guard 가 그
사실을 지킨다. 호출부가 없다는 사실 자체는 주석으로 적었다 — 없는 종료 경로를 만들어 내지 않았다.

## 검증

| 게이트 | 결과 |
| --- | --- |
| `npx tsc --noEmit` (rhwp-studio) | 통과(출력 없음) |
| `npm test` (rhwp-studio) | 843개 중 842 통과, 0 실패, 1 skip(환경: pkg-node 빌드 없음, 기존) |
| `python3 scripts/check_document_metadata.py` | 557개 문서, 이상 없음 |
| `python3 scripts/check_markdown_links.py` | 562개 문서, 이상 없음 |

수정 전 RED 확인 — `upstream/devel` 의 `subsecond-runtime.ts` 로 되돌려 새 테스트만 돌린 결과
(12개 중 6 실패):

- 1번 — `재도색이 던져도 감시 루프는 다음 프레임을 다시 예약해야 한다`: `0 !== 1`
- 1번(중복 예약) — `예약된 프레임은 언제나 한 개여야 한다`: `2 !== 1`
- 4번 — 백오프: `[250, 500, 1000]` ≠ `[250, 500, 250]`
- 3번 — `TypeError: SubsecondPatchAccumulation is not a constructor` (2건)
- 3번(측정 실패 내성) — 같은 이유로 RED
- 2번 — 대체 테스트는 행동 계약이라 수정 전후 모두 초록이다. RED 근거는 지운 단언 쪽에 있다:
  `CanvasView.dispose()` 호출부가 0개인데도 그 단언은 통과한다.

첫 구현(`onopen` 즉시 리셋)으로 되돌려 돌리면 백오프 테스트가 다시 RED 다:
`핸드셰이크만 성공한 연결로 백오프를 되돌리면 250ms 재연결이 영원히 돈다` — `[250, 250]` ≠ `[250, 500]`.

## 적대 리뷰에서 고치지 않기로 한 것

- `EventBus.emit` 핸들러 격리 — 위 1번 참고. 전역 계약이라 이 이슈에서 넓히지 않는다.
- `CanvasView.dispose()` 호출부 신설 — 위 2번 참고.
- 같은 테스트 파일에 남은 소스·매니페스트 텍스트 단언 27건(`:489-515`). 이슈가 지목한
  `build.rs` 의 `librhwp-dioxus.rlib`(`:495`, Windows 에서 `#[cfg]` 로 사라져도 통과)은 빌드 배선이라
  #4577·#4580 쪽이지만, `:499-504` 의 `wasm-bridge`·`canvas-view` 배선 확인은 **스튜디오 런타임
  배선**이라 그 분류가 맞지 않는다. 이 이슈에서 지운 것은 없는 수명 관리를 덮던 단언 하나뿐이고,
  나머지를 행동 검증으로 바꾸는 일은 별도 이슈로 남긴다.
- 무한 재연결 자체(시도 횟수 상한·로그 없음) — 로그는 #4578, 상한은 `dx serve` 재기동을 못 잡게 만든다.

브라우저 실동작은 확인하지 못했다. `dx serve --hot-patch` 로 핫패치 세션을 띄우려면 이 워크트리에서
wasm 디버그 전체 빌드가 선행되어야 한다.


## 후속 이슈 (2026-08-11)

작업 중 발견했지만 저장소 전역 계약이라 범위 밖으로 남긴 것을 이슈로 분리했다.

- **[#4591](https://github.com/edwardkim/rhwp/issues/4591)** — `EventBus.emit`
  (`event-bus.ts:16`)이 핸들러를 격리하지 않아 모든 `emit` 자리가 암묵적 rethrow 지점이다.
  이번 감시 루프 결함의 원인이 여기였고, 루프 쪽에서 `try`/`finally` 로 막았다. 다만
  구독자가 약 30곳이라 그냥 `try`/`catch` 로 감싸면 무관한 실패가 조용해진다 — 실측이 먼저다.
- **[#4592](https://github.com/edwardkim/rhwp/issues/4592)** — `CanvasView.dispose()`
  (`canvas-view.ts:950`)와 `WasmBridge.dispose()`(`wasm-bridge.ts:3157`) 둘 다 호출부가
  0개다. 스튜디오에 뷰 해체 경로 자체가 없다. `pagehide` 배선은 bfcache 복원과 충돌하므로
  답이 아니라는 것까지 적었다.
- **[#4593](https://github.com/edwardkim/rhwp/issues/4593)** — 이 파일에 남은 소스 텍스트
  정규식 단언 27건. `:495` 는 `#[cfg(unix)]` 안의 문자열을 찾으므로 Windows 에서 코드가
  아예 없어도 초록색이다. 매니페스트 대상 17건과 코드 파일 대상 10건은 성격이 달라 갈라야
  한다.

## 인용 정정

이슈 본문이 상류의 "8MB" 문장을 `lib.rs:659-663` 으로 적었으나 그 줄은 Chrome 의 동기
instantiate 상한이다. 패치 크기 문장은 **`lib.rs:604`**, 영구 증가 식은 `lib.rs:623` 이다.
이슈에 코멘트로 정정했다.
