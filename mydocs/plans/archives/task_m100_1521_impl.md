# Task #1521 구현 계획서 — hover card show/hide 타이머 분리

## 기준

- **수행 계획서**: `mydocs/plans/task_m100_1521.md`
- **Stage 1 보고서**: `mydocs/working/task_m100_1521_stage1.md`
- **브랜치/worktree**: `local/task1521` / `/Users/melee/Documents/projects/forks/rhwp`
- **base**: `upstream/devel` `a94e2051`
- **작업 범위**: Chrome/Firefox/Safari content-script hover lifecycle race 정정
- **비범위**: URL classifier(#1520), 링크 감지 규칙, hover card UI, thumbnail fetch 정책, manifest/CSP 변경

## 구현 원칙

1. show 예약과 hide 예약을 별도 타이머로 분리한다.
2. `pendingAnchor`와 `activeAnchor`를 명시적으로 관리한다.
3. 링크 `mouseleave`에서 pending show를 즉시 취소한다.
4. `showHoverCard(anchor)` 실행 직전에 해당 anchor가 아직 표시 대상인지 확인한다.
5. stale show가 future hide를 clear하지 못하도록 카드 제거와 타이머 취소 책임을 분리한다.
6. 카드 DOM 생성 방식, DOM API/textContent 보안 흐름, `open-hwp` 클릭 동작은 변경하지 않는다.
7. Chrome/Firefox/Safari는 같은 상태 전이 모델을 쓰되 API 네임스페이스 차이는 유지한다.

## 상태 모델

### 공통 상태

Chrome/Firefox에는 `activeAnchor`를 새로 추가하고, 세 플랫폼 모두 다음 상태를 갖도록 정렬한다.

```js
let activeCard = null;
let activeAnchor = null;
let pendingAnchor = null;
let showHoverTimeout = null;
let hideHoverTimeout = null;
```

### Helper

구현 시 다음 helper를 둔다. 이름은 기존 코드 스타일에 맞춰 짧게 조정할 수 있다.

```js
function clearShowTimer() {
  if (showHoverTimeout) {
    clearTimeout(showHoverTimeout);
    showHoverTimeout = null;
  }
}

function clearHideTimer() {
  if (hideHoverTimeout) {
    clearTimeout(hideHoverTimeout);
    hideHoverTimeout = null;
  }
}

function removeActiveCard() {
  if (!activeCard) return;
  if (activeCard.__rhwpHost) {
    activeCard.__rhwpHost.remove();
  } else {
    activeCard.remove();
  }
  activeCard = null;
  activeAnchor = null;
}
```

기존 외부 호출용 `hideHoverCard()`는 전체 lifecycle 취소로 유지한다.

```js
function hideHoverCard() {
  clearShowTimer();
  clearHideTimer();
  pendingAnchor = null;
  removeActiveCard();
}
```

중요: `showHoverCard(anchor)`는 기존처럼 `hideHoverCard()`를 직접 부르지 않고 `removeActiveCard()`를 사용한다. `hideHoverCard()`를 부르면 새 pending 상태까지 지워져 정상 show가 막힐 수 있기 때문이다.

## 상태 전이

### 링크 `mouseenter`

```js
anchor.addEventListener('mouseenter', () => {
  clearShowTimer();
  clearHideTimer();
  pendingAnchor = anchor;

  if (activeAnchor !== anchor) {
    removeActiveCard();
  }

  showHoverTimeout = setTimeout(() => {
    showHoverTimeout = null;
    showHoverCard(anchor);
  }, HOVER_SHOW_DELAY_MS);
});
```

Chrome/Firefox는 기존 300ms를 유지한다. Safari는 기존 250ms를 유지한다.

### 링크 `mouseleave`

```js
anchor.addEventListener('mouseleave', () => {
  if (pendingAnchor === anchor) {
    pendingAnchor = null;
  }
  clearShowTimer();

  if (activeAnchor === anchor && activeCard) {
    scheduleHideHoverCard();
  }
});
```

카드가 아직 뜨지 않은 경우에는 hide 예약을 만들지 않는다. 카드가 이미 떠 있고 링크에서 카드로 이동하는 경우에는 hide timer가 예약되지만, 카드 `mouseenter`가 이를 취소한다.

### 카드 `mouseenter` / `mouseleave`

```js
card.addEventListener('mouseenter', () => {
  clearHideTimer();
});

card.addEventListener('mouseleave', () => {
  scheduleHideHoverCard();
});
```

카드 이벤트는 hide timer만 다룬다. show timer를 건드리지 않아 stale show/hide 교차 간섭을 없앤다.

### `scheduleHideHoverCard()`

```js
function scheduleHideHoverCard() {
  clearHideTimer();
  hideHoverTimeout = setTimeout(() => {
    hideHoverTimeout = null;
    hideHoverCard();
  }, HOVER_HIDE_DELAY_MS);
}
```

Chrome/Firefox는 기존 200ms를 유지한다. Safari는 기존 150ms를 유지한다.

## `showHoverCard(anchor)` 변경

함수 시작부를 다음 의미로 정리한다.

```js
function showHoverCard(anchor) {
  if (!settings.hoverPreview) return;
  if (pendingAnchor !== anchor) return;
  if (!anchor.isConnected) {
    pendingAnchor = null;
    return;
  }
  if (typeof anchor.matches === 'function' && !anchor.matches(':hover')) {
    pendingAnchor = null;
    return;
  }

  clearHideTimer();
  removeActiveCard();

  // 기존 카드 생성 로직 유지
}
```

`pendingAnchor === anchor`를 lifecycle의 1차 권위로 삼고, `anchor.matches(':hover')`는 browser DOM 상태 방어로 사용한다. `matches(':hover')`가 실제 플랫폼에서 지나치게 엄격하면 Stage 3에서 해당 조건은 제거하고 `pendingAnchor` 검증만 유지한다. 구현 중 이 판단은 수동 fixture로 확인한다.

카드 생성 후 상태 설정은 다음 의미로 맞춘다.

```js
activeCard = card;
activeAnchor = anchor;
pendingAnchor = null;
```

## 플랫폼별 수정 계획

### 1. `rhwp-chrome/content-script.js`

현재:

- `activeCard`
- `hoverTimeout`
- `showHoverCard()` 시작부의 `hideHoverCard()`
- 카드 `mouseenter`가 `hoverTimeout` clear
- 링크 `mouseleave`가 pending show 미취소

수정:

- `activeAnchor`, `pendingAnchor`, `showHoverTimeout`, `hideHoverTimeout` 추가
- `clearShowTimer()`, `clearHideTimer()`, `removeActiveCard()`, `scheduleHideHoverCard()` 추가
- `hideHoverCard()`를 전체 lifecycle 취소 함수로 재정의
- `showHoverCard()` 시작부에 pending/connected/hover 검증 추가
- `showHoverCard()` 내부 기존 `hideHoverCard()` 호출을 `removeActiveCard()`로 교체
- 카드 이벤트는 hide timer만 제어
- 링크 이벤트는 pending show 취소와 hide 예약을 분리
- thumbnail callback 조건을 `activeCard === card && activeAnchor === anchor`로 보강

### 2. `rhwp-firefox/content-script.js`

Chrome과 같은 구조로 수정한다.

차이:

- `browser.runtime.sendMessage(...).then(...).catch(...)` Promise 흐름은 유지
- thumbnail callback의 `.then()` / `.catch()` 내부 조건을 `activeCard === card && activeAnchor === anchor`로 보강

### 3. `rhwp-safari/src/content-script.js`

현재:

- `activeCard`
- `activeAnchor`
- `hoverTimeout`
- Safari delay: show 250ms, hide 150ms
- thumbnail async 응답이 active card 여부를 확인하지 않음

수정:

- `pendingAnchor`, `showHoverTimeout`, `hideHoverTimeout` 추가
- 단일 `hoverTimeout` 제거
- Chrome/Firefox와 같은 helper와 상태 전이 적용
- 기존 250ms/150ms delay 유지
- `showHoverCard()` 시작부에 pending/connected/hover 검증 추가
- 카드 생성 전 기존 `hideHoverCard()` 호출을 `removeActiveCard()`로 교체
- thumbnail async 응답은 `activeCard === card && activeAnchor === anchor`일 때만 `insertThumbnail()` 수행

## 구현 단계

### Stage 3-1 — Chrome hover lifecycle 정정

- Chrome content-script 상태 변수와 helper 추가
- link/card 이벤트 handler 정리
- `showHoverCard()` stale guard 추가
- thumbnail callback active anchor guard 추가
- `node --check rhwp-chrome/content-script.js`

### Stage 3-2 — Firefox 동형 정정

- Chrome 수정 구조를 Firefox API 스타일에 맞춰 반영
- Promise callback guard 보강
- `node --check rhwp-firefox/content-script.js`

### Stage 3-3 — Safari 정렬

- Safari 단일 `hoverTimeout` 제거
- 기존 `activeAnchor`를 공통 모델에 맞게 재사용
- Safari thumbnail async guard 보강
- `node --check rhwp-safari/src/content-script.js`

### Stage 3-4 — 집중 검증 및 완료보고

- 수정한 content-script 전체 문법 검사
- Chrome/Firefox 빌드 가능 여부 확인
- 대표 fixture에서 수동 hover 체크
- `mydocs/working/task_m100_1521_stage3.md` 작성

## 검증 계획

### 문법 검사

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-safari/src/content-script.js
```

### 확장 빌드

```bash
cd rhwp-chrome
npm run build
```

```bash
cd rhwp-firefox
npm run build
```

`npm run build`는 Vite와 `pkg/` 산출물 상태에 의존한다. 현재 로컬 `pkg/`가 없거나 의존성 문제가 있으면, 실패를 환경 문제로 분리하고 `node --check` 및 targeted SW 테스트를 우선 게이트로 삼는다.

Safari 전체 빌드는 `rhwp-safari/build.sh`가 Xcode/Safari 변환 도구에 의존한다. Stage 3에서는 우선 `node --check`를 필수 게이트로 두고, Stage 4에서 환경이 허용하면 `rhwp-safari/build.sh`까지 실행한다.

### 기존 SW 테스트

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
node --test rhwp-chrome/sw/download-interceptor.test.mjs
```

```bash
node rhwp-firefox/sw/fetch-security.test.mjs
node --test rhwp-firefox/sw/download-interceptor.test.mjs
```

hover lifecycle 수정이 service worker를 직접 건드리지는 않지만, 확장 작업의 기존 보안 게이트로 유지한다.

### 수동 hover 체크

Chrome:

- `rhwp-chrome/test/05-gov-site-sim.html`
- `rhwp-chrome/test/06-security.html`

Firefox:

- `rhwp-firefox/test/05-gov-site-sim.html`
- `rhwp-firefox/test/06-security.html`

Safari:

- `rhwp-safari/test/test-hwp-link.html`
- 필요 시 `/private/tmp`에 임시 metadata-rich HWP 링크 페이지를 만들어 수동 확인한다. 이 임시 파일은 PR에 포함하지 않는다.

수동 판정 항목:

- show delay 이전 leave → 카드 미생성
- 정상 hover → 카드 표시
- 링크에서 카드로 이동 → 카드 유지
- 카드 leave → 카드 닫힘
- 빠른 여러 링크 통과 → 이전 링크 카드 미표시
- hover card click → 기존 `open-hwp` 요청 유지
- XSS fixture → data attribute가 실행되지 않고 텍스트로 표시

## Stage 4 회귀 검증 계획

Stage 3 승인 후 다음을 실행한다.

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-safari/src/content-script.js
node rhwp-chrome/sw/fetch-security.test.mjs
node --test rhwp-chrome/sw/download-interceptor.test.mjs
node rhwp-firefox/sw/fetch-security.test.mjs
node --test rhwp-firefox/sw/download-interceptor.test.mjs
```

가능하면 추가 실행:

```bash
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
cd rhwp-safari && ./build.sh
```

Safari/Xcode 빌드가 환경 문제로 실패하면, 실패 원인과 대체 검증(`node --check`, 수동 Safari fixture)을 Stage 4 보고서에 분리 기록한다.

## 성공 기준

- show delay 이전에 링크를 벗어나면 카드가 생성되지 않는다.
- 정상 hover에서는 카드가 기존처럼 생성된다.
- 링크에서 카드로 이동하면 카드가 유지된다.
- 카드에서 벗어나면 카드가 닫힌다.
- 여러 링크를 빠르게 지나가도 이전 링크 stale card가 늦게 표시되지 않는다.
- Chrome/Firefox/Safari content-script 문법 검사가 통과한다.
- Chrome/Firefox 기존 SW 테스트가 통과한다.
- DOM API/textContent 기반 hover card 보안 흐름이 유지된다.
- URL classifier 관련 동작은 변경하지 않는다.

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| `anchor.matches(':hover')`가 특정 브라우저에서 너무 엄격해 정상 표시를 막음 | Stage 3 수동 fixture에서 확인. 문제가 있으면 `pendingAnchor` guard만 유지 |
| `hideHoverCard()` 책임 변경으로 클릭 후 카드가 남음 | `hideHoverCard()`는 전체 lifecycle 취소로 유지하고 click handler는 그대로 호출 |
| 링크에서 카드로 이동할 때 hide가 실행됨 | 카드 `mouseenter`에서 hide timer만 취소. hide delay는 기존 값 유지 |
| 빠른 링크 전환 시 이전 카드가 남음 | 새 링크 `mouseenter`에서 hide timer 취소 후 active anchor가 다르면 `removeActiveCard()` |
| Safari thumbnail 응답이 닫힌 카드 DOM을 갱신 | `activeCard === card && activeAnchor === anchor` guard 추가 |
| Chrome/Firefox closed Shadow DOM 때문에 자동 검증 한계 | 문법/빌드/SW 테스트 + 수동 fixture 체크를 보고서에 명확히 남김 |

## 승인 후 다음 작업

작업지시자 승인 후 Stage 3 구현을 시작한다.
