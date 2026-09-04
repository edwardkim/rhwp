# Task M100 #1487 Stage 1

- 이슈: #1487 확장 옵션에 외부 웹폰트 사용 안 함 설정 추가
- 브랜치: `task_m100_1487`
- 작성일: 2026-06-23
- 상태: 구현/검증 완료

## 배경

내부망/오프라인 환경에서 CRX로 확장을 설치한 사용자가 viewer 진입 시
`웹폰트 로딩 중...` 상태에서 오래 대기한다는 피드백이 있었다.

현재 rhwp-studio의 font loader는 함초롬/한컴 계열 폰트를 `cdn.jsdelivr.net`
웹폰트로 등록하고, 초기 로딩에서 critical font로 로드한다. 인터넷 접근이
차단된 환경에서는 이 외부 웹폰트 요청이 viewer 초기 진입을 지연시킬 수 있다.

## 구현 방향

- 확장 options 페이지에 `외부 웹폰트 사용 안 함` 옵션을 추가한다.
- 기본값은 off로 유지하여 온라인 환경의 기존 렌더링 정합성을 보존한다.
- 옵션 안내에는 다음 문구를 포함한다.
  - `내부망/오프라인 환경에서 권장합니다. 일부 문서의 글꼴, 줄바꿈, 페이지 배치가 달라질 수 있습니다`
- 옵션이 켜진 viewer에서는 외부 웹폰트 `@font-face` 등록과 `FontFace.load()`를 모두 건너뛴다.
- 로컬 번들 폰트와 시스템 글꼴 사용은 유지한다.
- Chrome/Firefox/Safari 확장 설정 저장소 차이를 흡수하되, 일반 web/PWA 환경에서는 기본값 off로 동작한다.

## 검토 대상

- `rhwp-studio/src/core/font-loader.ts`
- `rhwp-studio/src/main.ts`
- Chrome/Firefox/Safari options 페이지와 설정 저장 기본값
- 확장 content/service worker의 `get-settings` 기본값
- i18n 메시지

## 검증 계획

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && npm test
cd rhwp-chrome && npm ci
cd rhwp-chrome && node build.mjs
node --check rhwp-chrome/options.js && node --check rhwp-chrome/background.js && node --check rhwp-chrome/sw/message-router.js && node --check rhwp-firefox/options.js && node --check rhwp-firefox/background.js && node --check rhwp-firefox/sw/message-router.js && node --check rhwp-safari/src/options.js && node --check rhwp-safari/src/background.js
node -e "for (const p of ['rhwp-chrome/_locales/ko/messages.json','rhwp-chrome/_locales/en/messages.json','rhwp-firefox/_locales/ko/messages.json','rhwp-firefox/_locales/en/messages.json']) JSON.parse(require('fs').readFileSync(p, 'utf8')); console.log('locale json ok')"
git diff --check
```

필요 시 확장 빌드까지 추가로 확인한다.

## 구현 내용

- Chrome/Firefox/Safari options 페이지에 `외부 웹폰트 사용 안 함` 옵션을 추가했다.
- Chrome/Firefox locale에 옵션 라벨과 안내 문구를 추가했다.
- 확장 설치 기본값과 `get-settings` 응답 기본값에 `disableExternalWebFonts: false`를 추가했다.
- viewer 공통 설정 브리지 `extension-settings.ts`를 추가해 Chrome/Firefox `storage.sync`, Safari `storage.local`을 읽도록 했다.
- `loadWebFonts()`에 옵션을 추가하고, 옵션이 켜진 경우 외부 URL 폰트의 `@font-face` 등록과 `FontFace.load()`를 모두 건너뛰도록 했다.
- viewer 초기화와 문서 로드 시 같은 설정을 font loader에 전달하도록 했다.
- 오프라인 옵션이 CDN `@font-face`와 `FontFace.load()`를 막는 회귀 테스트를 추가했다.

## 검증 결과

- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-studio && npm test`: 통과 (127 passed)
- `cd rhwp-chrome && npm ci`: 통과
  - 기존 의존성 기준 npm audit 경고 2건 표시
- `cd rhwp-chrome && node build.mjs`: 통과
- 확장 JS `node --check`: 통과
- Chrome/Firefox locale JSON parse: 통과
- `git diff --check`: 통과
