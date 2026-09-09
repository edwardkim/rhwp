# Task m100 #1307 Stage 3 보고서

## 목적

썸네일 추출 결과가 `data:image/...` URI로 page DOM에 직접 삽입될 경우, 신뢰할 수 없는 페이지 스크립트가 `MutationObserver`, `querySelector`, `img.src` 접근으로 미리보기 데이터를 읽을 수 있다.

Stage 1/2는 내부망 fetch 차단을 담당했고, Stage 3는 DOM 인터셉트 방어를 담당한다.

## 구현

수정 파일:

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`

변경:

- hover card를 page DOM 직속 card가 아니라 host + closed Shadow DOM 구조로 렌더링
- thumbnail `img.src`는 closed shadow 내부에만 존재
- page DOM에는 `data-rhwp-hover-host="true"` host만 노출
- content script는 내부 참조를 통해 card 크기 계산, 위치 지정, thumbnail 교체를 계속 수행

## 방어 효과

페이지 스크립트가 관찰 가능한 것:

- hover host 요소가 추가/삭제되는 사실
- host의 위치와 크기

페이지 스크립트가 직접 읽기 어려운 것:

- closed shadow 내부 DOM
- 자동 추출 thumbnail `img.src`
- service worker가 반환한 `dataUri`

즉, 자동 추출된 PrvImage가 page DOM의 일반 `img` 요소로 노출되는 경로를 제거했다.

## 한계

- closed Shadow DOM도 브라우저 렌더링 표면에는 표시된다. 픽셀 기반 스크린 캡처까지 막는 기능은 아니다.
- 페이지가 제공한 `data-hwp-thumbnail` URL은 원래 페이지가 알고 있는 값이므로 비밀 데이터로 보지 않는다.
- 더 강한 격리가 필요하면 후속으로 extension iframe 기반 미리보기 UI를 검토할 수 있다.

## 검증

실행:

```bash
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node rhwp-chrome/sw/fetch-security.test.mjs
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

결과:

- Chrome content script 문법 체크 통과
- Firefox content script 문법 체크 통과
- fetch security policy test 통과
- Chrome extension build 통과
- Firefox extension build 통과

## 브라우저 수동 확인 항목

- hover card가 기존과 같은 위치/크기로 표시되는지 확인
- 자동 추출 thumbnail이 hover card 안에 표시되는지 확인
- 페이지 콘솔에서 `document.querySelector('[data-rhwp-hover-host]')`는 가능하지만 내부 `img.src`가 직접 노출되지 않는지 확인
- 제보 PoC에서 private/internal URL fetch가 차단되고 dataUri 로그가 수집되지 않는지 확인

## 릴리스 계획

- rhwp core 0.7.15 패치 버전 릴리즈 후 Chrome/Firefox 확장 0.2.4에 포함해 배포한다.
- 제보자 회신에는 0.7.15 → 0.2.4 순서와 0.2.4 배포 전후 재검증 요청을 명시한다.
- 공개 기여자 목록에는 이메일과 PoC 세부 없이 `Dangel`을 보안 제보 기여자로 추가한다.
