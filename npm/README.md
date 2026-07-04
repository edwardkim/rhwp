# @rhwp/core

**알(R), 모두의 한글** — 브라우저에서 HWP 파일을 열어보세요

[![npm](https://img.shields.io/npm/v/@rhwp/core)](https://www.npmjs.com/package/@rhwp/core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust + WebAssembly 기반 HWP/HWPX 파서 & 렌더러입니다.
HWP 파일을 파싱하고 SVG로 렌더링하는 저수준 API를 제공합니다.

> 편집 기능(메뉴, 툴바, 서식)이 필요하면 **[@rhwp/editor](https://www.npmjs.com/package/@rhwp/editor)** 를 사용하세요.
> 3줄이면 완전한 HWP 에디터를 임베드할 수 있습니다.

| 패키지 | 용도 |
|--------|------|
| **@rhwp/core** (이 패키지) | WASM 파서/렌더러 — 직접 API 호출 |
| **@rhwp/editor** | 완전한 에디터 UI — iframe 임베드 |

## 빠른 시작 — 처음부터 따라하기

### 1. 프로젝트 생성

```bash
mkdir my-hwp-viewer
cd my-hwp-viewer
npm init -y
npm install @rhwp/core
npm install vite --save-dev
```

### 2. WASM 파일 복사

`@rhwp/core`에 포함된 WASM 바이너리를 웹 서버가 제공할 수 있는 위치에 복사합니다.

```bash
mkdir public
cp node_modules/@rhwp/core/rhwp_bg.wasm public/
```

### 3. HTML 작성 — `index.html`

```html
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8" />
  <title>HWP 뷰어</title>
</head>
<body>
  <h1>HWP 뷰어</h1>
  <input type="file" id="file-input" accept=".hwp,.hwpx" />
  <p id="status">파일을 선택해주세요.</p>
  <div id="viewer"></div>
  <script type="module" src="/main.js"></script>
</body>
</html>
```

### 4. JavaScript 작성 — `main.js`

```javascript
import init, { HwpDocument } from '@rhwp/core';

// ① 텍스트 폭 측정 함수 등록 (필수 — 아래 "왜 필요한가?" 참고)
let ctx = null;
let lastFont = '';
globalThis.measureTextWidth = (font, text) => {
  if (!ctx) ctx = document.createElement('canvas').getContext('2d');
  if (font !== lastFont) { ctx.font = font; lastFont = font; }
  return ctx.measureText(text).width;
};

// ② WASM 초기화
await init({ module_or_path: '/rhwp_bg.wasm' });
document.getElementById('status').textContent = '준비 완료!';

// ③ 파일 선택 시 렌더링
document.getElementById('file-input').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (!file) return;

  const buffer = new Uint8Array(await file.arrayBuffer());
  const doc = new HwpDocument(buffer);

  // SVG로 첫 페이지 렌더링
  const svg = doc.renderPageSvg(0);
  document.getElementById('viewer').innerHTML = svg;
  document.getElementById('status').textContent =
    `${file.name} — ${doc.pageCount()}페이지`;
});
```

### 5. 실행

```bash
npx vite --port 3000
```

브라우저에서 `http://localhost:3000` 을 열고 HWP 파일을 선택하면 렌더링됩니다.

## API

### 초기화

```javascript
import init, { HwpDocument } from '@rhwp/core';
await init({ module_or_path: '/rhwp_bg.wasm' });
```

### 문서 로드

```javascript
// 파일 입력에서 로드
const doc = new HwpDocument(new Uint8Array(buffer));

// fetch로 로드
const resp = await fetch('/sample.hwp');
const doc = new HwpDocument(new Uint8Array(await resp.arrayBuffer()));
```

### SVG 렌더링

```javascript
const svg = doc.renderPageSvg(0);   // 첫 페이지
document.getElementById('viewer').innerHTML = svg;
```

### 페이지 네비게이션

```javascript
const total = doc.pageCount();
for (let i = 0; i < total; i++) {
  const svg = doc.renderPageSvg(i);
  // ...
}
```

### 편집 API

`HwpDocument` 는 텍스트·표·그림·필드·서식을 편집하는 API 를 제공합니다. 대부분의
편집 메서드는 결과를 JSON 문자열로 반환합니다.

```javascript
// 텍스트 삽입 (구역 0, 문단 0, 오프셋 0)
doc.insertText(0, 0, 0, '안녕하세요');

// 표 생성 (2행 2열)
doc.createTable(0, 0, 0, 2, 2);
```

전체 편집 API 목록과 시그니처는 패키지에 포함된 타입 정의 `rhwp.d.ts` 를 참고하세요
(IDE 자동완성으로 인자·반환 타입을 확인할 수 있습니다).

### options object 변형 (`*Ex`) — 권장

인자가 많은 편집 API 는 **options object 변형 `<이름>Ex(optionsJson)`** 을 함께
제공합니다. positional 인자 대신 JSON 문자열로 호출하므로, 라이브러리 버전이 올라가며
인자가 추가·변경돼도 호출부가 덜 깨집니다(0.x 단계 권장).

```javascript
// positional — 인자 위치가 바뀌면 호출부를 모두 수정해야 함
doc.insertPicture(0, 0, 0, '', imageBytes, 4000, 3000, 100, 80, 'png', '', null, null);

// options object (권장) — 필드 추가/순서 변경에 안전
doc.insertPictureEx(
  JSON.stringify({
    sectionIdx: 0, paraIdx: 0, charOffset: 0,
    width: 4000, height: 3000, naturalWidthPx: 100, naturalHeightPx: 80,
    extension: 'png',
  }),
  imageBytes, // 이미지 바이너리는 별도 인자(Uint8Array)
);
```

- 키는 camelCase(`section_idx` → `sectionIdx`). 선택 키는 생략 시 기본값으로 처리됩니다.
- 반환값과 동작은 positional 버전과 동일합니다.
- 이미지 같은 바이너리 데이터는 JSON 이 아니라 별도 인자로 받습니다.
- 어떤 메서드에 `*Ex` 가 있는지는 `rhwp.d.ts` 에서 `Ex(options` 로 확인할 수 있습니다.

> 0.x 단계라 편집 API 시그니처가 바뀔 수 있습니다. 인자가 많은 API 는 `*Ex` 사용을
> 권장하며, 변경 사항은 CHANGELOG 의 `### API` 항목에 기록합니다.

## 필수 설정: measureTextWidth

WASM 내부에서 텍스트 레이아웃(줄바꿈, 정렬 등)을 계산할 때
브라우저의 Canvas 텍스트 측정 API가 필요합니다.
**WASM 초기화 전에 반드시 등록**해야 합니다.

```javascript
let ctx = null;
let lastFont = '';
globalThis.measureTextWidth = (font, text) => {
  if (!ctx) ctx = document.createElement('canvas').getContext('2d');
  if (font !== lastFont) { ctx.font = font; lastFont = font; }
  return ctx.measureText(text).width;
};
```

### 왜 필요한가?

HWP 문서의 텍스트 배치(줄바꿈 위치, 양쪽 정렬 간격)를 정확하게 계산하려면
각 글자의 실제 렌더링 폭을 알아야 합니다.
WASM 내부에는 브라우저 폰트에 접근할 수 없으므로,
JavaScript의 `Canvas.measureText()`를 콜백으로 호출합니다.

## 폰트 설정 가이드

### SVG 렌더링과 폰트

`renderPageSvg()`가 생성하는 SVG는 CSS `font-family` 속성으로 폰트를 지정합니다.
HWP 문서에서 사용된 폰트(한컴바탕, HY명조 등)가 사용자 환경에 없으면 **글자가 대체 폰트로 표시**되어 줄 바꿈 위치나 글자 간격이 원본과 달라질 수 있습니다.

### 권장: 오픈소스 폴백 폰트 로드

rhwp는 한컴 전용 폰트를 오픈소스 폰트로 자동 폴백합니다. 아래 폰트를 웹페이지에 로드하면 대부분의 HWP 문서를 원본에 가깝게 렌더링할 수 있습니다.

```html
<!-- Google Fonts CDN -->
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+KR:wght@400;700&family=Noto+Serif+KR:wght@400;700&family=Nanum+Gothic&family=Nanum+Myeongjo&display=swap">
```

또는 셀프 호스팅:

```css
@font-face {
  font-family: 'Pretendard';
  src: url('/fonts/Pretendard-Regular.woff2') format('woff2');
  font-weight: 400;
}
@font-face {
  font-family: 'Pretendard';
  src: url('/fonts/Pretendard-Bold.woff2') format('woff2');
  font-weight: 700;
}
```

### 폰트 폴백 매핑

rhwp가 SVG에 적용하는 자동 폴백 체인:

| HWP 원본 폰트 | 폴백 1 | 폴백 2 | 폴백 3 |
|--------------|--------|--------|--------|
| 한컴바탕, HY명조 | Noto Serif KR | 나눔명조 | serif |
| 한컴돋움, HY고딕 | Noto Sans KR | 나눔고딕 | sans-serif |
| 함초롬바탕 | Noto Serif KR | 나눔명조 | serif |
| 함초롬돋움 | Pretendard | Noto Sans KR | sans-serif |
| 맑은 고딕 | Pretendard | Noto Sans KR | sans-serif |
| Arial, Calibri | Pretendard | sans-serif | — |
| Times New Roman | Noto Serif KR | serif | — |
| 바탕, 궁서 | Noto Serif KR | serif | — |
| 돋움, 굴림 | Noto Sans KR | sans-serif | — |

### 폰트 없이도 동작합니다

폴백 폰트를 로드하지 않아도 SVG는 정상적으로 렌더링됩니다. 브라우저의 기본 serif/sans-serif 폰트가 사용되며, 글자 간격이 원본과 다소 다를 수 있습니다.

## 지원 기능

- **HWP 5.0** (바이너리) + **HWPX** (XML) 파싱
- 문단, 표, 수식, 이미지, 차트, 도형 렌더링
- 페이지네이션 (다단, 표 행 분할)
- 그림 자르기(crop), 이미지 테두리선
- SVG 출력
- 머리말/꼬리말/바탕쪽/각주/미주

## 링크

- **[온라인 데모](https://edwardkim.github.io/rhwp/)**
- **[GitHub](https://github.com/edwardkim/rhwp)**
- **[@rhwp/editor](https://www.npmjs.com/package/@rhwp/editor)** — 에디터 UI 임베드
- **[VS Code 확장](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode)**

## Third-Party Licenses

이 패키지는 다음 오픈소스 Rust 크레이트를 WASM으로 컴파일하여 포함합니다.

| 크레이트 | 라이선스 |
|---------|---------|
| wasm-bindgen / web-sys / js-sys | MIT OR Apache-2.0 |
| quick-xml | MIT |
| cfb | MIT |
| flate2 | MIT OR Apache-2.0 |
| encoding_rs | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| usvg / svg2pdf | Apache-2.0 OR MIT |
| pdf-writer | MIT OR Apache-2.0 |
| unicode-segmentation / unicode-width | MIT OR Apache-2.0 |
| image | MIT OR Apache-2.0 |

전체 목록: [THIRD_PARTY_LICENSES.md](https://github.com/edwardkim/rhwp/blob/main/THIRD_PARTY_LICENSES.md)

> 모든 의존성은 MIT 라이선스와 호환됩니다.

## Notice

본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

## Trademark

"한글", "한컴", "HWP", "HWPX"는 주식회사 한글과컴퓨터의 등록 상표입니다.
본 패키지는 한글과컴퓨터와 제휴, 후원, 승인 관계가 없는 독립적인 오픈소스 프로젝트입니다.

## License

MIT
