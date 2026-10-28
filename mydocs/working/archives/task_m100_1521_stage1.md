# Task #1521 Stage 1 완료보고서 — 진단 확정

## 범위

- Chrome/Firefox/Safari content-script의 hover lifecycle 현재 구조 확인
- show/hide 타이머 덮어쓰기와 stale timer 발생 순서 확정
- `showHoverCard()` 실행 직전 검증 부재 확인
- 썸네일 비동기 응답의 카드 생존 검증 상태 확인
- Stage 2 구현 계획서에서 다룰 검증 fixture와 게이트 후보 정리

소스 코드는 수정하지 않았다.

## 이슈 요약

이슈 #1521의 증상은 브라우저 확장 hover card 표시 지연 타이머 race다.

정상 후보 HWP/HWPX 링크에 마우스를 올리면 content-script가 카드 표시를 지연 예약한다. 카드가 표시되기 전에 링크에서 마우스가 빠져도 기존 show 예약이 안정적으로 취소되지 않아, 나중에 `showHoverCard(anchor)`가 실행되고 화면에 카드가 남을 수 있다.

관련 이슈 #1520은 URL classifier 오탐 정정이다. 현재 브랜치 기준으로 Chrome/Firefox/Safari content-script에는 이미 `classifyDocumentHref()` 기반 URL 판별 코드가 들어와 있다. 따라서 이번 작업 범위는 URL 판별이 아니라 정상 후보 링크의 hover lifecycle 정정으로 확정한다.

## Chrome 진단

대상 파일: `rhwp-chrome/content-script.js`

현재 hover 관련 상태는 다음과 같다.

```text
278: let activeCard = null;
279: let hoverTimeout = null;
```

`mouseenter`에서는 기존 타이머를 clear하고 카드 표시를 예약한다.

```text
446: anchor.addEventListener('mouseenter', () => {
447:   clearTimeout(hoverTimeout);
448:   hideHoverCard();
449:   hoverTimeout = setTimeout(() => showHoverCard(anchor), 300);
450: });
```

`mouseleave`에서는 기존 show 예약을 취소하지 않고 같은 변수에 hide 예약을 덮어쓴다.

```text
451: anchor.addEventListener('mouseleave', () => {
452:   hoverTimeout = setTimeout(() => hideHoverCard(), 200);
453: });
```

`showHoverCard(anchor)`는 실행 직전에 해당 anchor가 여전히 hover 대상인지 확인하지 않는다. 또한 첫 동작으로 `hideHoverCard()`를 호출한다.

```text
282: function showHoverCard(anchor) {
283:   if (!settings.hoverPreview) return;
285:   hideHoverCard();
```

`hideHoverCard()`는 `activeCard` 제거 후 현재 `hoverTimeout`을 clear한다.

```text
431: function hideHoverCard() {
...
440:   clearTimeout(hoverTimeout);
441: }
```

### Chrome race 순서

예시 1 — 매우 빠른 leave:

1. t=0 링크 `mouseenter` → show timer A 예약, 300ms 뒤 실행 예정
2. t=50 링크 `mouseleave` → hide timer B 예약, 200ms 뒤 실행 예정, `hoverTimeout`은 B로 덮임
3. t=250 B 실행 → active card가 없으므로 제거할 것이 없음, B만 clear
4. t=300 A 실행 → `showHoverCard(anchor)`가 카드 생성
5. 현재 마우스는 링크/카드 밖이므로 추가 leave 이벤트가 없어 카드가 남음

예시 2 — hide가 show보다 늦게 예약되는 leave:

1. t=0 링크 `mouseenter` → show timer A 예약
2. t=150 링크 `mouseleave` → hide timer B 예약, t=350 실행 예정
3. t=300 A 실행 → `showHoverCard()` 내부 `hideHoverCard()`가 현재 `hoverTimeout`인 B를 clear
4. 카드 생성 후 B가 실행되지 않음
5. 현재 마우스는 링크/카드 밖이므로 카드가 남음

즉 Chrome은 pending show 미취소뿐 아니라, stale show 실행 시점에 future hide까지 취소할 수 있다. 단일 `hoverTimeout`으로 show/hide를 같이 관리하는 것이 직접 원인이다.

## Firefox 진단

대상 파일: `rhwp-firefox/content-script.js`

Firefox도 Chrome과 같은 구조다.

```text
282: let activeCard = null;
283: let hoverTimeout = null;
```

`mouseenter` / `mouseleave`의 타이머 관리도 동일하다.

```text
461: anchor.addEventListener('mouseenter', () => {
462:   clearTimeout(hoverTimeout);
463:   hideHoverCard();
464:   hoverTimeout = setTimeout(() => showHoverCard(anchor), 300);
465: });
466: anchor.addEventListener('mouseleave', () => {
467:   hoverTimeout = setTimeout(() => hideHoverCard(), 200);
468: });
```

`showHoverCard(anchor)`도 실행 직전 anchor hover 상태를 검증하지 않고 `hideHoverCard()`를 먼저 호출한다.

```text
286: function showHoverCard(anchor) {
287:   if (!settings.hoverPreview) return;
289:   hideHoverCard();
```

따라서 Firefox도 Chrome과 같은 stale show / hide cancellation race를 가진다.

Firefox의 썸네일 비동기 응답은 `activeCard === card` 조건을 사용하므로 닫힌 Chrome/Firefox 카드에 썸네일을 덧붙이는 문제는 이미 방어되어 있다.

```text
420: if (activeCard === card) {
421:   const thumbDiv = card.querySelector('.rhwp-thumb-loading');
```

## Safari 진단

대상 파일: `rhwp-safari/src/content-script.js`

Safari는 `activeAnchor`를 이미 갖고 있지만 show/hide 타이머는 하나다.

```text
142: let activeCard = null;
143: let activeAnchor = null;
144: let hoverTimeout = null;
```

`showHoverCard(anchor)`는 같은 anchor의 기존 카드 중복 생성만 막는다.

```text
178: function showHoverCard(anchor) {
179:   if (!settings.hoverPreview) return;
180:   if (activeAnchor === anchor && activeCard) return;
182:   hideHoverCard();
```

이 조건은 “이미 표시된 같은 카드”만 다루며, “예약 당시 anchor가 아직 유효한 hover 대상인지”는 확인하지 않는다.

Safari의 `mouseleave`는 기존 단일 타이머를 clear한 뒤 hide를 예약한다.

```text
313: anchor.addEventListener('mouseenter', () => {
314:   clearTimeout(hoverTimeout);
315:   hoverTimeout = setTimeout(() => showHoverCard(anchor), 250);
316: });
317: anchor.addEventListener('mouseleave', () => {
318:   clearTimeout(hoverTimeout);
319:   hoverTimeout = setTimeout(() => hideHoverCard(), 150);
320: });
```

이 구조는 이슈의 “show 예약 미취소” 케이스는 Chrome/Firefox보다 덜하지만, 여전히 show/hide 상태가 하나의 변수에 묶여 있다. 또한 카드 영역 `mouseenter`도 같은 타이머를 clear한다.

```text
288: card.addEventListener('mouseenter', () => clearTimeout(hoverTimeout));
289: card.addEventListener('mouseleave', () => {
290:   hoverTimeout = setTimeout(() => hideHoverCard(), 150);
291: });
```

Safari는 구현 계획에서 같은 상태 전이 모델로 정렬하는 것이 맞다. 특히 show 직전 active/pending anchor 검증을 넣어야 Chrome/Firefox와 같은 의미를 보장할 수 있다.

Safari의 썸네일 비동기 응답은 현재 카드 생존 여부를 확인하지 않고 `thumbContainer`에 직접 삽입한다.

```text
206: browser.runtime.sendMessage({ type: 'extract-thumbnail', url: anchor.href })
207:   .then(response => {
208:     if (response && response.dataUri) {
209:       thumbnailCache.set(anchor.href, response);
210:       insertThumbnail(thumbContainer, response.dataUri);
```

닫힌 카드에 붙은 detached DOM을 갱신하는 수준이면 사용자 visible 문제는 작지만, Stage 2에서 `activeCard === card && activeAnchor === anchor` 형태의 보강 여부를 함께 결정한다.

## 원인 확정

원인은 다음 3개로 정리한다.

1. Chrome/Firefox가 show 타이머와 hide 타이머를 단일 `hoverTimeout`으로 관리한다.
2. Chrome/Firefox의 `mouseleave`가 pending show를 취소하지 않는다.
3. 세 플랫폼 모두 `showHoverCard(anchor)` 실행 직전에 “이 anchor가 현재 표시되어야 하는 대상인가”를 명시적으로 검증하지 않는다.

부가적으로 Chrome/Firefox는 stale show 실행 시점에 `hideHoverCard()`가 현재 변수에 저장된 hide timer를 clear할 수 있어, 카드가 닫힐 마지막 기회까지 사라질 수 있다.

## 구현 계획 후보

Stage 2에서 다음 모델을 구체화한다.

- `showTimer`와 `hideTimer`를 분리한다.
- `pendingAnchor` 또는 `activeAnchor`를 명시적으로 관리한다.
- 링크 `mouseenter`:
  - hide timer 취소
  - 기존 카드 제거 또는 전환 정책 적용
  - pending anchor 설정
  - show timer 예약
- 링크 `mouseleave`:
  - show timer 즉시 취소
  - pending anchor가 해당 링크면 해제
  - active card가 있으면 hide timer 예약
- 카드 `mouseenter`:
  - hide timer만 취소
  - show timer는 건드리지 않음
- 카드 `mouseleave`:
  - hide timer 예약
- `showHoverCard(anchor)` 시작부:
  - 설정 확인
  - pending/active anchor가 현재 anchor인지 확인
  - anchor가 DOM에서 여전히 유효한지 확인하는 방어 조건 검토

## 테스트 fixture 후보

자동 hover 전용 테스트는 현재 없다. 기존 테스트 자산은 다음과 같다.

| 플랫폼 | 후보 | 용도 |
|------|------|------|
| Chrome | `rhwp-chrome/test/05-gov-site-sim.html` | 여러 HWP/HWPX 링크가 가까이 있어 빠른 hover 전환 수동 확인 |
| Firefox | `rhwp-firefox/test/05-gov-site-sim.html` | Chrome과 같은 수동 확인 |
| Safari | `rhwp-safari/test/test-hwp-link.html` | Safari 기본 HWP/HWPX 링크 hover 확인 |
| Chrome/Firefox | `test/06-security.html` | hover card DOM API / XSS 방어 회귀 확인 |

`05-gov-site-sim.html`에는 `data-hwp="true"` 링크와 확장자 기반 링크가 함께 있다. 빠른 leave, 정상 hover, 여러 링크 통과를 확인하기에 적합하다.

Safari fixture는 링크 수가 적고 metadata가 부족하므로 Stage 2에서 수동 확인만 할지, 임시 로컬 재현 페이지를 `output/` 또는 `/private/tmp`에 둘지 결정한다. PR 포함 fixture 추가는 별도 승인 전에는 하지 않는다.

## 검증 게이트 후보

Stage 2에서 다음 명령과 수동 체크를 구현 계획에 확정한다.

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-safari/src/content-script.js
```

```bash
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
node --test rhwp-chrome/sw/download-interceptor.test.mjs
node rhwp-firefox/sw/fetch-security.test.mjs
node --test rhwp-firefox/sw/download-interceptor.test.mjs
```

Safari는 `rhwp-safari/build.sh`가 Chrome build, Safari JS syntax check, Xcode build까지 포함한다. 로컬 Xcode/Safari 환경 의존성이 있으므로 Stage 2에서 실행 가능 범위와 실패 시 대체 게이트를 명시한다.

## 수동 회귀 체크리스트

- 링크에 300ms 이상 hover하면 카드가 표시된다.
- show 지연 시간 전에 링크를 벗어나면 카드가 표시되지 않는다.
- 링크에서 카드로 이동하면 카드가 유지된다.
- 카드에서 벗어나면 카드가 닫힌다.
- 여러 HWP/HWPX 링크를 빠르게 지나갈 때 이전 링크 카드가 늦게 표시되지 않는다.
- hover card 클릭 시 기존 `open-hwp` 동작이 유지된다.
- `test/06-security.html`에서 data attribute 텍스트가 실행되지 않고 텍스트로 처리된다.

## 결론

이슈 #1521은 코드 기준으로 재현 가능한 hover lifecycle race로 확정한다.

수정 범위는 Chrome/Firefox/Safari content-script의 hover 상태 관리이며, URL classifier, 링크 감지 규칙, hover card UI, 썸네일 fetch 정책은 본 작업 범위에서 제외한다.

다음 단계는 구현 계획서에서 타이머 분리와 active/pending anchor 상태 전이 규칙을 확정하는 것이다.
