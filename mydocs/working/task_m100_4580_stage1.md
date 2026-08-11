# task_m100_4580_stage1 — subsecond 핫패치 통합 정리 7건

한 적대 리뷰가 남긴 일곱 이슈를 한 브랜치로 처리한 단계 보고서다. 커밋은 이슈별로 나누고
**위험이 낮은 것부터** 쌓았다 — 메인테이너가 꼬리를 잘라도 앞쪽이 남는다.

| 순서 | 커밋 | 이슈 | 성격 |
|---|---|---|---|
| 1 | `docs(dev)` 검증 명령 | #4588 | 문서 |
| 2 | `docs(dev)` debug_assertions | #4596 | 문서 |
| 3 | `test(subsecond)` 결과 코드 드리프트 | #4589 | 테스트 + CI 라우팅 |
| 4 | `test(subsecond)` 소스 텍스트 단언 제거 | #4593 | 테스트 |
| 5 | `fix(studio)` 번들 격리 | #4580 | 스튜디오 배선 |
| 6 | `fix(build)` 버전 단일 출처 | #4580 | 빌드 |
| 7 | `refactor(wasm_api)` cfg_attr | #4580 | Rust 경계 |
| 8 | `refactor(lib)` doc(hidden) | #4580 | Rust 표면 |
| 9 | `refactor` 벤더→도메인 개명 | #4580 | 이름 (그룹 내 마지막) |
| 10 | `refactor(studio)` 해체 계약 | #4592 | 스튜디오 전역 |
| 11 | `fix(studio)` EventBus 배달 | #4591 | 스튜디오 전역 |

## 1. #4588 — 게이트 두 개, 둘 다 이미 통과한다

이슈가 지적한 두 게이트를 `upstream/devel`(298c2c1b2)에서 실측했다.

- clippy(`--features subsecond-dev`)는 **통과한다.** 이슈가 본 `wasm_api.rs` 의
  `clippy::needless_return` 은 #4577 이 그 블록을 `hot_render_boundaries!` 매크로로 옮기면서
  함께 사라졌다(저장소 전체 `needless_return` 0건).
- `--lib` 없는 wasm32 검사는 7개 오류로 죽는다. 다만 **`--features subsecond-dev` 를 빼도 같은
  7개**가 난다. CLI 바이너리가 wasm32 대상이 아니라서지 이 feature 탓이 아니다.

그래서 `src/main.rs` 를 wasm32 에서 컴파일되게 만들지 않았다. CLI 가 wasm32 대상이 아니라는
것은 사실이고, 브라우저에 실리는 것은 `[lib]` 뿐이며, CI 의 Lint 잡은 이미 `--lib` 을 쓴다.
남은 결함은 "그 사실이 적혀 있지 않다" 하나라 문서로 닫았다.

## 2. #4596 — 두 조건이 별개라는 사실

`HotFn::try_call` 이 `if !cfg!(debug_assertions)` 로 점프 테이블을 건너뛴다. feature 를 켜는
것과 디버그 프로파일로 빌드하는 것은 별개의 두 조건이고, 그 사실이 어디에도 없었다.

이슈가 선택지로 남긴 런타임 경고는 넣지 않았다. Rust→브라우저 콘솔 경로가 이 크레이트에 아직
없고(`web_sys::console` 사용처 0건), 술어가 `cfg!(debug_assertions)` 자신이라 테스트가
동어반복이 된다.

## 3. #4589 — 값이 있는가: 있다, 그리고 선례를 따랐다

같은 일곱 이름이 세 벌이었다(Rust `code()`, TS `SUBSECOND_OUTCOMES`, 테스트 `REJECTION_CODES`).
이슈가 요구한 대로 저장소에 같은 문제를 푼 곳이 있는지부터 봤다.

- `scripts/frontend-wasm-bindings.test.mjs` 가 `src/wasm_api.rs` 의 `js_name` 전부를 뽑아
  `pkg/rhwp.d.ts` 가 덮는지 본다(CI `Frontend package gates`).
- `rhwp-studio/tests/subsecond-runtime.test.ts` 자신이 이미 `src/wasm_api.rs`·`src/lib.rs` 를
  읽는다. `tests/undo-noop-skip.test.ts` 도 Rust 본문을 잘라 읽는다.
- 반대 방향(Rust 테스트가 `.ts` 를 읽는 것)은 저장소에 선례가 **없다.**

새 기제를 만들지 않고 그 모양을 따랐다. 사본은 1출처 + 2유도가 됐고, 검사는 두 방향을 본다.
**CI 배선이 없으면 이 검사는 장식이다** — `src/subsecond_dev.rs` 만 고친 변경은
`ci-impact-classifier` 에서 rust 로만 분류되어 frontend 잡이 아예 돌지 않는다. 검사가 가장
필요한 방향에서 지나가므로 `src/wasm_api.rs` 와 같은 근거로 fail-closed 목록에 올렸다.

## 4. #4593 — 코드 파일을 겨눈 10건은 전부 없앴다

27건 중 17건(매니페스트·설정)은 남겼다. 실행되는 코드가 아니고 cargo·vite·npm 이 그 문자열을
그대로 읽으므로 "이 자리에 이 문자열이 있다"가 곧 계약이다.

코드 파일을 겨눈 10건에는 이슈의 질문을 하나씩 했고 전부 "다르다"였다. 특히 두 건:

- `build.rs` 의 `librhwp-dioxus.rlib` — 그 이름을 쓰는 심링크 생성이 `#[cfg(unix)]` 뒤에 있어
  Windows 에서는 만들어지지 않는데 단언은 초록이었다. 상수 선언만 무조건 컴파일된다.
- `lib.rs` 의 feature 게이트 — **컴파일러가 증명한다.** 게이트를 지우고 기본 feature 로
  `cargo check --lib` 하면 `error[E0432]: unresolved import 'subsecond'` 로 죽는다(실측).

**새 소스 텍스트 단언은 더하지 않았다.** Rust `js_name` ↔ TS 속성 이름 짝은 지금 아무것도
검사하지 않는 상태로 남는다 — 확인하려면 `subsecond-dev` 로 빌드한 wasm 이 필요한데 어느
게이트도 그것을 만들지 않는다. 없는 보장을 초록으로 덮는 것보다 없다고 두는 편이 낫다.

## 5. #4580 — 다섯 갈래

### 번들 격리 (실측)

| 표지 | 전 | 후 |
|---|---|---|
| `_dioxus` | 1 | 0 |
| `subsecondProbe` | 2 | 0 |
| `applySubsecondDevtoolsMessage` | 1 | 0 |
| `getSubsecondPatchRevision` | 4 | 0 |
| `invalidateSubsecondRenderCaches` | 4 | 0 |
| 벤더 이름 전수(`[Ss]ubsecond`·`[Dd]ioxus`) | 22 | 0 |

`index-*.js` 1,271,397B → 1,266,369B. **DEV 게이트만으로는 모자랐다** — 능력 세 개가
`WasmBridge` 의 메서드였고, 클래스 메서드는 호출부가 없어도 트리셰이킹되지 않는다. 모듈만 뺀
중간 상태에서 벤더 이름 10건이 남는 것을 실측으로 잡았다.

검증은 소스가 아니라 **산출물**을 본다(`scripts/frontend-studio-dist.test.mjs`, CI 의
`Build Studio` 다음 단계). 표지가 개발 전용 모듈에 아직 있는지 되짚어 헛도는 감시를 막고,
번들을 실제로 읽었다는 증거도 함께 확인한다.

### 버전 단일 출처

계약을 정하는 사본을 아홉 개 지웠다. 남긴 `0.7.10` 은 계약이 아니라 **증거**다 —
`THIRD_PARTY_LICENSES.md` 의 대장, 그리고 벤더 소스를 줄 번호까지 인용하는 자리.

### `[lib] crate-type` — 바꾸지 않는다

아키텍처 문서 넷이 이 순서를 사실로 인용하고(`agent_architecture/invariants.md`,
`agent_security/detection_policy.md`, `agent_runtime/surface_spec.md`,
`wasm_agent_surface/self_description.md`), rhwp 자신에게도 맞는 순서다 — CLI 두 개와
`bindings/Native`, `tools/*` 가 전부 rlib 으로 링크한다. 주석에서 뺀 것은 버전뿐이고, 근거를
벤더 규칙이 아니라 rhwp 자신의 것부터 적었다.

### `tools/rhwp-subsecond` → `[[example]]` — 하지 않는다

이슈의 논거(예제는 `cargo build --workspace` 가 안 만드니 #3890 이 구조적으로 불가능해진다)는
맞다. 하지 않은 이유는 **검증할 수 없어서**다. `dx serve --package rhwp-subsecond` 를
`--example` 로 옮길 수 있는지는 dioxus-cli 를 실제로 띄워 봐야 알 수 있고, `build.rs` 의 심링크를
npm 스크립트의 `ln -sf` 로 옮기는 것도 dx 세션 없이는 확인이 안 된다. `cargo build --workspace`
는 지금 통과한다(7745f8e88 이 고쳤다). 검증하지 못한 구조 변경을 넣지 않는다.

### 개명 — 벤더가 바뀌면 무엇이 바뀌는가

| | 전 | 후 |
|---|---|---|
| 벤더 이름을 부르는 파일 | 14 | 17 |
| 그중 스튜디오 프로덕션 모듈의 벤더 언급 | wasm-bridge 22, canvas-view 7 | wasm-bridge 11, canvas-view 4 |
| 프로덕션 번들의 벤더 이름 | 22 | 0 |

파일 수가 는 것은 이번에 만든 벤더 전용 도구 셋 때문이다(`scripts/dioxus-cli-version.mjs`,
`frontend-studio-dist.test.mjs`, `frontend-wasm-bindings.test.mjs` 의 주석). 그 셋은 **벤더가
있어서 존재하는** 파일이라 벤더를 부르는 것이 맞다.

스튜디오 프로덕션 모듈에 남은 벤더 언급은 전부 모듈 경로(`subsecond-runtime`)와 DEV 게이트
한 메서드 안의 데브서버 프로토콜 이름이다. 벤더를 바꾸면 바뀌어야 하는 파일과 아닌 파일이 이제
이름으로 갈린다.

## 6. #4592 — 둘로 갈라 답했다

- `WasmBridge.dispose()` — **지웠다.** 호출부 0개이고 몸통이 `releaseDocument()`(실제로 불린다)의
  부분집합이었다. `disposed` 플래그가 없어 파급도 없다.
- `CanvasView.dispose()` — **남기고 계약을 적었다.** 죽은 사본이 아니라 온전한 해체 구현이고,
  지우려면 `disposed` 가드 10곳까지 함께 지워야 한다. `pagehide` 배선은 bfcache 복원 때문에
  적극적으로 해롭다. 해제 시점을 지어내지 않고, 그 시점이 오는 조건(문서 닫기·뷰 교체)을 적었다.

## 7. #4591 — 실측 후 격리, 다만 삼키지 않는다

구독 48곳, 발행 213곳. **되풀림에 기대는 발행 자리는 0곳**이다(`await` 된 `emit` 0, `emit` 을
위해 쓴 `catch` 0, 되풀림 단언 테스트 0). 반면 `try` 안의 `emit` 74곳 중 22곳은 구독자 예외를
`catch {}` 로 이미 삼키고, 13곳은 성공한 명령을 `return false` 로 뒤집는다.

그래서 바꾼 것은 **오직 "남은 구독자가 도는가"** 하나다. 배달을 끝낸 뒤 첫 실패를 그대로 다시
던지므로 전달 동작은 이전과 같고, 삼켜지던 22곳에서는 새 `console.error` 가 유일한 신호가 된다.

## 옛 브랜치에서 가져온 것과 다시 쓴 것

`fix/issue-4580-subsecond-isolation`(77커밋 뒤진 상태)의 다섯 커밋을 읽고 판단했다.

- **가져옴(수정해서)**: 번들 격리 접근(값 import → `import type` + DEV 동적 import, 능력 구현을
  개발 전용 모듈로 이동), `scripts/dioxus-cli-version.mjs`, `scripts/frontend-studio-dist.test.mjs`,
  `#[doc(hidden)]`, cfg_attr 정리.
- **다시 씀**: 개명의 동사(`invalidateDerivedRenderState` → `rebuildDerivedState` — #4576 이
  몸통을 "다시 만든다"로 바꿨으므로 `DocumentCore` 의 같은 이름을 위임한다), `patch_revision()`
  경유(#4577 의 매크로), `SubsecondPatchAccumulation` 배선(#4590 이후 추가된 것),
  `dioxus-cli-version.mjs` 의 main 판별, dist 테스트의 표지 목록.
- **버림**: 옛 브랜치가 테스트에 더한 `assert.match(runtime, /…/)` 세 줄 — #4593 이 없애기로 한
  바로 그 형태다.


## 후속 이슈 (2026-08-12)

작업 중 발견했지만 배정된 일곱 이슈 밖이라 **고치지 않고** 이슈로 분리했다.

- **[#4630](https://github.com/edwardkim/rhwp/issues/4630)** — wasm32 타깃 clippy 가
  기존 상태에서 **16건** 실패한다(`web_canvas.rs:2677` `identity_op` 등). CI 가 네이티브만
  lint 해서 wasm32 전용 lint 부채가 안 보인다. feature 와 무관하다.
- **[#4631](https://github.com/edwardkim/rhwp/issues/4631)** — CLI 바이너리가 wasm32 로
  컴파일되지 않는다. #4588 은 게이트 명령을 `--lib` 로 고정하는 것으로 처리했고, 구조적 제외
  여부는 미결이다.
- **[#4632](https://github.com/edwardkim/rhwp/issues/4632)** — 소스 텍스트 정규식 단언이
  `rhwp-studio/tests/` **164개 중 110개 파일**에 퍼져 있다. #4593 은 subsecond 파일의 코드
  대상 10건만 없앴다.
- **[#4633](https://github.com/edwardkim/rhwp/issues/4633)** — `async` EventBus 구독자
  7개가 배달 계약 밖에 있다. `emit` 이 동기라 거부가 도달하지 못한다 — #4591 전후 모두.
  `compare-dialog.ts:297-307` 의 90초 타임아웃이 같은 부류다.
- **[#4634](https://github.com/edwardkim/rhwp/issues/4634)** — CI 영향 분류기의 fail-closed
  목록이 손 유지다. #4589 의 드리프트 검사가 **정작 드리프트가 나는 방향에서 안 돌 뻔했다**.
- **[#4635](https://github.com/edwardkim/rhwp/issues/4635)** — 잔여물 3건(낡은 문서 인용,
  주석 처리된 구독, 번들 마커 오탐 가능성).
- **[#4636](https://github.com/edwardkim/rhwp/issues/4636)** — `wasm-bridge.ts` 가 아직
  devtools 소켓을 소유해 벤더 언급 11개가 프로덕션 모듈에 남는다.

## 강제된 의존 (범위 밖이지만 게이트가 막혀 최소 변경)

`scripts/frontend-wasm-bindings.test.mjs` 가 tautological `cfg_attr` 을 평범한
`#[wasm_bindgen]` 으로 바꾸는 순간 깨졌다 — 모든 `js_name` 이 `pkg/rhwp.d.ts` 에 있어야
하는데, 그 산출물은 optional feature 없이 빌드된다. 여러 줄 `cfg_attr` 이 그 정규식에서
**우연히 이름을 숨기고 있었다.** feature 게이트된 이름을 건너뛰고 역방향을 확인하며
비공허성 바닥을 두는 최소 수정을 했다. 단독 이슈였다면 *"wasm-bindings 게이트가 feature
게이트된 export 에서 거짓 실패한다"* 였을 것이다.
