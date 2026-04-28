# @rhwp/editor

**알(R), 모두의 한글** — 3줄로 HWP 에디터를 웹 페이지에 임베드

[![npm](https://img.shields.io/npm/v/@rhwp/editor)](https://www.npmjs.com/package/@rhwp/editor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

웹 페이지에 HWP 에디터를 통째로 임베드합니다.
메뉴, 툴바, 서식, 표 편집 — rhwp-studio의 모든 기능을 그대로 사용할 수 있습니다.

> **[온라인 데모](https://edwardkim.github.io/rhwp/)** 에서 먼저 체험해보세요.

## 설치

```bash
npm install @rhwp/editor
```

## 빠른 시작 — 3줄이면 충분합니다

```html
<!DOCTYPE html>
<html>
<head>
  <title>내 HWP 에디터</title>
  <style>
    #editor { width: 100%; height: 100vh; }
  </style>
</head>
<body>
  <div id="editor"></div>
  <script type="module">
    import { createEditor } from '@rhwp/editor';

    const editor = await createEditor('#editor');
  </script>
</body>
</html>
```

이것만으로 메뉴바, 툴바, 편집 영역, 상태 표시줄이 포함된 완전한 HWP 에디터가 표시됩니다.

## HWP 파일 로드

```javascript
import { createEditor } from '@rhwp/editor';

const editor = await createEditor('#editor');

// 파일 선택 또는 fetch로 HWP 데이터 가져오기
const response = await fetch('document.hwp');
const buffer = await response.arrayBuffer();

// 에디터에 로드
const result = await editor.loadFile(buffer, 'document.hwp');
console.log(`${result.pageCount}페이지 로드 완료`);
```

## API

### createEditor(container, options?)

에디터를 생성하고 컨테이너에 마운트합니다.

```javascript
const editor = await createEditor('#editor');
// 또는
const editor = await createEditor(document.getElementById('editor'));
```

**옵션:**

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `studioUrl` | `https://edwardkim.github.io/rhwp/` | rhwp-studio URL |
| `width` | `'100%'` | iframe 너비 |
| `height` | `'100%'` | iframe 높이 |

### editor.loadFile(data, fileName?)

HWP 파일을 로드합니다.

```javascript
const result = await editor.loadFile(buffer, 'sample.hwp');
// result = { pageCount: 5 }
```

### editor.pageCount()

현재 문서의 페이지 수를 반환합니다.

```javascript
const count = await editor.pageCount();
```

### editor.getPageSvg(page?)

특정 페이지를 SVG 문자열로 렌더링합니다.

```javascript
const svg = await editor.getPageSvg(0); // 첫 페이지
```

### editor.exportHwp()

현재 편집 중인 문서를 HWP 바이너리로 내보냅니다.

```javascript
const bytes = await editor.exportHwp();
const blob = new Blob([bytes], { type: 'application/x-hwp' });

const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'document.hwp';
a.click();
URL.revokeObjectURL(url);
```

### editor.destroy()

에디터를 제거합니다.

```javascript
editor.destroy();
```

## 폰트 안내

### 기본 동작 — 별도 설정 없이 사용 가능

`@rhwp/editor`는 오픈소스 폰트를 내장하고 있어 **별도 폰트 설정 없이 바로 사용**할 수 있습니다.

HWP 문서에서 사용된 한컴 전용 폰트(한컴바탕, HY명조 등)는 자동으로 오픈소스 폰트로 폴백됩니다.

### 내장 폴백 폰트

| HWP 원본 폰트 | 자동 폴백 |
|--------------|----------|
| 한컴바탕, HY명조, 바탕, 궁서 | Noto Serif KR → 나눔명조 → serif |
| 한컴돋움, HY고딕, 돋움, 굴림 | Noto Sans KR → 나눔고딕 → sans-serif |
| 함초롬바탕 | Noto Serif KR → 나눔명조 → serif |
| 함초롬돋움, 맑은 고딕 | Pretendard → Noto Sans KR → sans-serif |
| Arial, Calibri, Verdana | Pretendard → sans-serif |
| Times New Roman | Noto Serif KR → serif |
| Courier New | D2Coding → monospace |

### 폰트 품질

- **내장 폰트만으로 대부분의 HWP 문서를 원본에 가깝게 렌더링**할 수 있습니다
- 글자 간격이 원본과 미세하게 다를 수 있으나, 문서 내용 열람에는 지장이 없습니다
- 한컴 전용 폰트가 설치된 PC에서는 원본과 동일하게 표시됩니다

### 셀프 호스팅 시 폰트

셀프 호스팅 환경에서는 rhwp-studio 빌드에 포함된 `web/fonts/` 폴더의 오픈소스 폰트가 자동으로 사용됩니다. 추가 폰트를 원하면 `web/fonts/`에 woff2 파일을 추가하고 CSS `@font-face`를 등록하면 됩니다.

## 셀프 호스팅

기본적으로 `https://edwardkim.github.io/rhwp/`에 호스팅된 에디터를 사용합니다.
자체 서버에서 호스팅하려면:

```bash
# rhwp-studio 빌드
cd rhwp-studio
npm install
npx vite build --base=/your-path/

# 빌드 결과물(dist/)을 서버에 배포
```

```javascript
const editor = await createEditor('#editor', {
  studioUrl: 'https://your-domain.com/your-path/'
});
```

## 패키지 비교

| 패키지 | 용도 |
|--------|------|
| **@rhwp/core** | WASM 파서/렌더러 (직접 API 호출) |
| **@rhwp/editor** | 완전한 에디터 UI (iframe 임베드) |

- 뷰어만 필요하면 → `@rhwp/core`
- 편집 기능이 필요하면 → `@rhwp/editor`

## Third-Party Licenses

이 패키지는 iframe을 통해 rhwp-studio를 임베드하며, 내부적으로 `@rhwp/core` WASM 엔진을 사용합니다.

### Rust 크레이트 (WASM 엔진)

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

### 내장 웹 폰트

| 폰트 | 라이선스 |
|------|---------|
| Pretendard (Regular, Bold) | SIL Open Font License 1.1 |
| Noto Sans KR (Regular, Bold) | SIL Open Font License 1.1 |
| Noto Serif KR (Regular, Bold) | SIL Open Font License 1.1 |
| 나눔고딕 | SIL Open Font License 1.1 |
| 나눔명조 | SIL Open Font License 1.1 |
| 고운바탕 | SIL Open Font License 1.1 |
| 고운돋움 | SIL Open Font License 1.1 |
| D2Coding | SIL Open Font License 1.1 |

### 프론트엔드

| 패키지 | 라이선스 |
|--------|---------|
| TypeScript | Apache-2.0 |
| Vite | MIT |

전체 목록: [THIRD_PARTY_LICENSES.md](https://github.com/edwardkim/rhwp/blob/main/THIRD_PARTY_LICENSES.md)

> 모든 의존성은 MIT 라이선스와 호환됩니다.

## Notice

본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

## Trademark

"한글", "한컴", "HWP", "HWPX"는 주식회사 한글과컴퓨터의 등록 상표입니다.
본 패키지는 한글과컴퓨터와 제휴, 후원, 승인 관계가 없는 독립적인 오픈소스 프로젝트입니다.

## License

MIT
