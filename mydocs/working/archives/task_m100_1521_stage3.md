# Task #1521 Stage 3 완료보고서 — 구현 + 집중 검증

## 범위

- Chrome/Firefox/Safari content-script의 hover card lifecycle 정정
- show/hide 타이머 분리
- pending/active anchor 상태 명시
- stale show timer가 카드 또는 hide 예약을 덮는 race 제거
- 썸네일 비동기 응답의 active card/anchor 검증 보강
- JS 문법 검사와 기존 확장 SW 테스트 실행

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-chrome/content-script.js` | `hoverTimeout` 단일 상태 제거, show/hide 타이머 분리, `activeAnchor`/`pendingAnchor` 추가 |
| `rhwp-firefox/content-script.js` | Chrome과 같은 lifecycle 모델 적용, Promise thumbnail 응답 guard 보강 |
| `rhwp-safari/src/content-script.js` | 기존 `activeAnchor`를 공통 모델로 정렬, show/hide 타이머 분리, thumbnail async guard 추가 |

## 구현 내용

### 1. 타이머 분리

기존 단일 상태:

```js
let hoverTimeout = null;
```

신규 상태:

```js
let activeCard = null;
let activeAnchor = null;
let pendingAnchor = null;
let showHoverTimeout = null;
let hideHoverTimeout = null;
```

Chrome/Firefox는 show 300ms, hide 200ms를 유지했다. Safari는 기존 show 250ms, hide 150ms를 유지했다.

### 2. lifecycle helper 추가

세 플랫폼에 같은 의미의 helper를 추가했다.

- `clearShowHoverTimer()`
- `clearHideHoverTimer()`
- `removeActiveHoverCard()`
- `scheduleHideHoverCard()`

`hideHoverCard()`는 전체 lifecycle 취소 함수로 유지했다.

```js
function hideHoverCard() {
  clearShowHoverTimer();
  clearHideHoverTimer();
  pendingAnchor = null;
  removeActiveHoverCard();
}
```

핵심은 `showHoverCard()` 내부에서 더 이상 `hideHoverCard()`를 직접 호출하지 않는 것이다. 대신 `removeActiveHoverCard()`만 호출해 stale show가 future hide timer까지 지우는 문제를 없앴다.

### 3. `showHoverCard(anchor)` stale guard

카드 생성 전 다음 조건을 확인한다.

- hover preview 설정이 켜져 있는가
- `pendingAnchor === anchor`인가
- anchor가 아직 DOM에 연결되어 있는가
- anchor가 현재 `:hover` 상태인가

이 조건을 통과하지 않으면 카드 DOM을 만들지 않는다.

### 4. 이벤트 전이

링크 `mouseenter`:

- show/hide timer를 각각 취소
- 이미 같은 anchor의 active card가 있으면 유지
- 새 anchor면 기존 active card 제거
- `pendingAnchor` 설정 후 show timer 예약

링크 `mouseleave`:

- 해당 anchor가 pending이면 pending 해제
- show timer 즉시 취소
- 이미 active card가 있으면 hide timer 예약

카드 `mouseenter`:

- hide timer만 취소

카드 `mouseleave`:

- hide timer 예약

### 5. thumbnail async guard

Chrome/Firefox는 기존 `activeCard === card` 조건을 다음처럼 보강했다.

```js
activeCard === card && activeAnchor === anchor
```

Safari는 기존에 비동기 썸네일 응답이 닫힌 카드의 detached DOM에도 `insertThumbnail()`을 실행할 수 있었다. 이번 변경으로 현재 active card/anchor일 때만 삽입한다.

## 검증 결과

### 문법 검사

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-safari/src/content-script.js
```

결과: 모두 통과.

### 기존 SW 테스트

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
```

결과: 통과.

```bash
node --test rhwp-chrome/sw/download-interceptor.test.mjs
```

결과: 13개 테스트 통과.

```bash
node --test rhwp-firefox/sw/download-interceptor.test.mjs
```

결과: 11개 테스트 통과.

참고: 계획서에 후보로 적은 `rhwp-firefox/sw/fetch-security.test.mjs`는 현재 저장소에 존재하지 않아 실행 대상에서 제외했다. Firefox 쪽에는 `sw/fetch-security.js`는 있으나 대응 test 파일은 없다.

### whitespace 검사

```bash
git diff --check
```

결과: 통과.

## 빌드 검증

### 로드용 dist 준비

표준 `npm run build`는 아래 환경 문제로 실패했지만, 추적 파일을 변경하지 않고 Vite JS API를 `configFile: false`로 직접 호출해 viewer bundle을 생성한 뒤 기존 `build.mjs`의 복사 단계를 수동으로 동일 적용했다.

생성된 로드용 산출물:

```text
rhwp-chrome/dist/
rhwp-firefox/dist/
rhwp-safari/dist/
```

확인 결과:

- `rhwp-chrome/dist/manifest.json` 파싱 성공, MV3, version `0.2.7`
- `rhwp-firefox/dist/manifest.json` 파싱 성공, MV3, version `0.2.7`
- `rhwp-safari/dist/manifest.json` 파싱 성공, MV3, version `0.2.1`
- 각 dist의 `content-script.js` 문법 검사 통과
- 각 dist의 `content-script.js`는 수정된 source와 byte-for-byte 일치
- `viewer.html`, `background.js`, `content-script.js`, `manifest.json`, `wasm/rhwp_bg.wasm` 존재 확인
- `dist/`는 git ignored 상태라 PR 변경 목록에는 포함되지 않음

### Chrome

```bash
cd rhwp-chrome
npm run build
```

결과: 실패.

1차 실행은 Vite가 `/Users/melee/node_modules/.vite-temp`에 임시 config 파일을 쓰려다 sandbox `EPERM`으로 실패했다.

승격 권한으로 재실행했으나 다음 환경 문제로 실패했다.

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'vite' imported from
/Users/melee/node_modules/.vite-temp/vite.config.ts.timestamp-...
```

`rhwp-chrome/vite.config.ts`가 `vite`를 import하지만, Vite가 config를 `/Users/melee/node_modules/.vite-temp` 아래 임시 파일로 번들링하면서 해당 위치 기준으로 `vite` 패키지를 찾지 못한다.

### Firefox

```bash
cd rhwp-firefox
npm run build
```

결과: 실패.

Chrome과 같은 Vite config 해석 문제다.

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'vite' imported from
/Users/melee/node_modules/.vite-temp/vite.config.ts.timestamp-...
```

이 빌드 실패는 content-script 변경 구간까지 도달하기 전에 발생했으며, Stage 3 코드 변경으로 인한 문법/런타임 오류는 아니다. Stage 4에서 빌드 환경 정리가 가능하면 재검증한다.

## 수동 hover 검증 상태

현재 CLI 환경에서는 브라우저 확장을 실제 로드한 상태의 수동 hover 확인을 수행하지 않았다.

Stage 4에서 다음 fixture로 확인한다.

- `rhwp-chrome/test/05-gov-site-sim.html`
- `rhwp-chrome/test/06-security.html`
- `rhwp-firefox/test/05-gov-site-sim.html`
- `rhwp-firefox/test/06-security.html`
- `rhwp-safari/test/test-hwp-link.html`

확인 항목:

- show delay 이전 leave → 카드 미생성
- 정상 hover → 카드 표시
- 링크에서 카드로 이동 → 카드 유지
- 카드 leave → 카드 닫힘
- 빠른 여러 링크 통과 → 이전 링크 카드 미표시
- hover card click → 기존 `open-hwp` 요청 유지
- XSS fixture → data attribute가 실행되지 않고 텍스트로 표시

## 남은 위험

| 위험 | 상태 |
|------|------|
| `anchor.matches(':hover')`가 특정 브라우저에서 정상 표시를 과하게 막을 가능성 | Stage 4 수동 확인 필요 |
| Chrome/Firefox closed Shadow DOM 때문에 자동 hover 검증이 제한됨 | 수동 fixture 확인 필요 |
| 확장 build script의 Vite config 해석 실패 | 환경 문제로 분리, Stage 4에서 가능 시 재시도 |
| Safari 실제 확장 런타임 동작 | `node --check` 통과, 실제 Safari 확인은 Stage 4 |

## 결론

Stage 3 구현은 완료했다.

코드상 단일 `hoverTimeout` race는 제거되었고, stale show timer가 더 이상 카드 생성 또는 hide 예약을 덮을 수 없도록 show/hide timer와 pending/active anchor 상태를 분리했다.

다음 단계는 Stage 4 회귀 검증이다. 특히 실제 브라우저에서 `anchor.matches(':hover')` guard가 정상 hover 표시를 막지 않는지 확인해야 한다.
