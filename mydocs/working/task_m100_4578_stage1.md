---
kind: working
status: active
canonical: mydocs/manual/dev_environment_guide.md
last_verified: 2026-08-11
---

# Task #4578 Stage 1 — 핫패치 네 층의 침묵을 깬다

이슈가 지목한 침묵은 넷이고, 그중 하나는 **구조적으로 보고할 수 없는 것**이었다. 파이프라인은
그대로 두고 신호만 낸다.

## 1. 보고할 수 없는 `bool` — 없앴다

`src/subsecond_dev.rs`

`apply_subsecond_devtools_message` 의 `bool` 은 두 가지를 동시에 거짓말했다.

- 다섯 거절 사유를 전부 `false` 하나로 접었다.
- `true` 는 wasm32 에서 **구조적으로** 참일 수 없었다. `subsecond::apply_patch` 는 wasm32 에서
  patch wasm 의 fetch/compile/instantiate future 를 띄우고 즉시 `Ok(())` 를 돌려주므로
  (subsecond 0.7.10 `src/lib.rs:551`, `:690`), 적용 성공은 이 함수가 돌아온 **뒤에** 결정된다.

두 선택지 중 "진실을 보고하게 만든다"는 불가능하다 — 값이 정해지는 시점이 반환 시점보다 뒤다.
future 를 기다리려면 `Promise` 를 돌려주는 다른 함수가 되어야 하고 그건 파이프라인 재설계다.
그래서 **거짓말하는 `bool` 을 없애는 쪽**을 골랐다. 반환값은
`DevtoolsMessageOutcome::code()` 문자열이고, 성공값의 이름은 `patch-dispatched` —
"넘겼다"까지만 말하고 "적용됐다"고 말하지 않는다. 없는 정보를 지어내는 대신 **없다는 사실을
값의 이름과 문서로 드러낸다.**

### wasm 실패는 어디로 나가는가 (조사 결과)

이슈는 "unhandled promise rejection"이라고 적었는데, 실제 경로는 둘이고 대개는 rejection 이
아니다.

- future 안의 실패는 전부 `.unwrap()`/`panic!`(subsecond `lib.rs:578-582`)이다.
- 이 panic 은 먼저 `console_error_panic_hook` 을 통과한다. 이 hook 은 루트 `Cargo.toml` 의
  **기본 feature**이고 `src/lib.rs:38` 의 `#[wasm_bindgen(start)]` 로 자동 설치되므로, panic
  **메시지 자체는 `console.error` 로 나온다** — 완전한 침묵은 아니었다. 다만 그 메시지에는
  subsecond·핫패치라는 단어가 없어 "왜 화면이 안 바뀌나"와 연결되지 않는다.
- panic 은 abort → wasm trap 이 되어 호출자에게 던져진다. 이 future 를 돌리는 것은
  `wasm_bindgen_futures::spawn_local` 이 직접 돌리는 것이 아니라 js-sys 0.3 의 마이크로태스크 큐다
  (`js-sys/src/futures/queue.rs:63-74`). 현대 브라우저에는 `queueMicrotask` 가 있으므로 trap 은
  **`queueMicrotask` 콜백에서 던져진 예외** — HTML 명세상 uncaught exception 으로 보고되어
  전역 `error` 이벤트가 된다. `queueMicrotask` 가 없는 폴백 경로에서만 promise reaction 이
  되어 `unhandledrejection` 이 된다.

**결론: 잡을 수는 있다. 단, 전역에서만, 그리고 귀속은 불가능하다.** 그래서 스튜디오는 두 이벤트를
모두 듣되, "이 오류가 패치 탓이다"라고 단정하지 않고 "패치를 넘긴 뒤에 도달했다"는 사실과 다음에
볼 곳만 말한다. 이미 abort 된 뒤라 복구 개념도 없다.

## 2. 반환값을 읽지 않던 호출부

`rhwp-studio/src/core/subsecond-runtime.ts`

`applyMessage(event.data);` 가 결과를 버렸다. 이제 결과 코드를 표(`SUBSECOND_OUTCOMES`)로 옮겨
수준별로 보고한다.

| 코드 | 수준 | 이유 |
|---|---|---|
| `patch-dispatched` | info | 넘겼다는 사실은 "안 바뀐다"를 진단할 때 필요하다 |
| `not-hot-reload` | debug | 데브서버 정상 제어 트래픽 — 경고로 올리면 소음이다 |
| 나머지 다섯 | warn | 전부 서로 다른 문구 + 다음에 볼 곳 |
| 표에 없는 값 | warn | 옛 번들(불리언 판)이 로드된 경우까지 드러낸다 |

프로덕션 격리는 두 겹이다. (1) `applySubsecondDevtoolsMessage` 자체가 `subsecond-dev` feature
빌드에만 있어 프로덕션 런타임은 이 경로에 오지 않는다. (2) 기본 출력이 `import.meta.env.DEV`
뒤에 있어 Vite 가 프로덕션 번들에서 분기를 통째로 제거한다.

**(2)는 처음에 성립하지 않았다.** 첫 구현은 메시지 경로에서 곧바로 완성된 문구를 넘겼는데,
그러면 문구 표가 항상 살아 있는 코드에서 참조되어 `import.meta.env.DEV` 가 지운 것은
`console` 호출뿐이었다. `npm run build` 산출물을 실제로 grep 해서 확인했다.

```
$ grep -c "점프 테이블을 subsecond" dist/assets/index-*.js   # 1  ← 프로덕션에 들어갔다
$ grep -c "\[subsecond\]"           dist/assets/index-*.js   # 0  ← 콘솔 호출만 지워졌다
```

그래서 메시지 경로가 넘기는 것을 **사실만 담은 신호**(`SubsecondSignal`)로 바꾸고, 문구 생성은
`describeSubsecondSignal` 로 미뤘다. 이 함수는 DEV 가드 안쪽에서만 참조되므로 프로덕션에서는
아무 데서도 참조되지 않아 표와 함께 사라진다. 재측정 결과는 다음과 같다.

```
$ grep -c "점프 테이블을 subsecond" dist/assets/index-*.js   # 0
$ grep -c "건을 넘긴 뒤"            dist/assets/index-*.js   # 0
$ grep -c "읽지 못한 결과 값"       dist/assets/index-*.js   # 0
$ grep -c "subsecond:serve"        dist/assets/index-*.js   # 0
$ grep -c "_dioxus"                dist/assets/index-*.js   # 1  ← 메시지 경로는 그대로 있다
$ grep -c "patch-dispatched"       dist/assets/index-*.js   # 1
```

**#4579 와의 경계**: 같은 파일을 건드리지만 rAF 워처와 소켓 재접속 로직은 그대로다. 소켓
생명주기 쪽 변경은 disposer 에 전역 리스너 해제 두 줄을 더한 것뿐이다.

## 3. `tools/rhwp-subsecond/build.rs` 의 네 가지 조용한 실패

| 지점 | 종전 | 지금 |
|---|---|---|
| `#[cfg(unix)]` | 비-unix 에서 함수 자체가 없음 | `cargo:warning` — unix 전용 이유와 WSL 대안 |
| `ancestors().nth(3)` | 조용히 `return` | `cargo:warning` + 실제 OUT_DIR 출력 |
| `alias.exists()` | 심링크를 따라가 끊긴 별칭을 "없음"으로 읽음 → `symlink()` EEXIST → `let _` 가 삼킴 | `symlink_metadata()` 로 링크 자체를 보고, 생성 실패는 `cargo:warning` |
| `rerun-if-changed=build.rs` 뿐 | `deps/` 정리 후 별칭이 영영 복구 안 됨 | 별칭 경로 자체를 추적 — 사라지면 다음 빌드에서 재생성 |

끊긴 별칭 자체는 경고하지 않는다. 대상 rlib 은 이 스크립트가 끝난 뒤에 생기고 `cargo check` 만
돌면 아예 생기지 않으므로, "끊김"은 정상 상태이기도 하다. 여기서 경고하면 wasm 대상
`cargo check` 마다 거짓 경고가 난다.

비-unix 를 심링크 대신 복사로 지원하지 않은 이유: 별칭 대상(`librhwp.rlib`)은 이 빌드 스크립트가
끝난 뒤 rustc 가 만든다. 복사할 원본이 아직 없다. 지원은 진단이 아니라 기능이므로 이 작업 범위
밖이고, 대신 제약을 말한다.

## 4. 문서

`mydocs/manual/dev_environment_guide.md` 에 "Subsecond 핫패치 (개발 전용, unix 호스트 전용)"
절을 넣었다. 실행 절차, 플랫폼 제약과 그 이유, **층별 실패 신호를 어디서 읽는지**(표), 그리고
마지막 층의 구조적 한계를 적었다.

## 검증

RED 를 먼저 확인했다.

- Rust: 새 테스트 5건이 `E0277 can't compare bool with &str` 외 6개 오류로 컴파일 실패 →
  구현 후 5 passed.
- Studio: 새 테스트 4건이 **어서션 수준**으로 실패(`0 !== 5` 등) → 구현 후 9/9 통과.
  프로덕션 번들 격리는 위 grep 으로 실측했다.
- build.rs: 끊긴 별칭 상태에서 `cargo check --target wasm32` 출력에 경고 0건, 별칭을 지우고
  다시 빌드해도 재생성 안 됨 → 수정 후 경고가 뜨고 별칭이 재생성된다.
