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

### editor.exportHwpx()

현재 편집 중인 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다.

```javascript
const bytes = await editor.exportHwpx();
const blob = new Blob([bytes], { type: 'application/vnd.hancom.hwpx' });

const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'document.hwpx';
a.click();
URL.revokeObjectURL(url);
```

### editor.exportHwpVerify()

HWP 직렬화 + 자기 재로드 검증 메타데이터를 반환합니다 (#178).
검증 메타데이터만 반환하며, 실제 HWP bytes 가 필요하면 `exportHwp()` 를 별도 호출하세요.

```javascript
const verify = await editor.exportHwpVerify();
// {
//   bytesLen: 678912,
//   pageCountBefore: 9,
//   pageCountAfter: 9,
//   recovered: true
// }
if (!verify.recovered || verify.pageCountBefore !== verify.pageCountAfter) {
  console.warn('HWP 직렬화 검증 실패', verify);
}
```

### editor.destroy()

에디터를 제거합니다.

```javascript
editor.destroy();
```

## 디버깅 도구 — rhwpDev

iframe 안의 rhwp-studio에는 **개발 모드 전용 디버깅 헬퍼** `rhwpDev`가 내장되어 있습니다. 검색, 커서 이동, 문단 ID 점검 등을 콘솔에서 직접 호출할 수 있습니다.

### 접근 방법

`rhwpDev`는 **iframe 내부 window**에 등록됩니다. 브라우저 DevTools의 콘솔 컨텍스트를 iframe으로 전환한 후 호출하세요.

**Chrome / Edge:** DevTools 콘솔 좌측 상단의 컨텍스트 드롭다운(`top ▾`)에서 `rhwp-studio iframe` 선택

**Firefox:** 콘솔 우측 상단의 `iframe` 버튼 → iframe URL 선택

```javascript
// iframe 컨텍스트로 전환 후
rhwpDev.help();   // 사용법 안내
```

> **주의:** rhwp-studio가 DEV 모드(vite dev server)로 빌드된 경우에만 `rhwpDev`가 등록됩니다. 프로덕션 빌드(기본 호스팅 URL `https://edwardkim.github.io/rhwp/`)에는 포함되지 않습니다. 디버깅 도구가 필요하면 셀프 호스팅 환경에서 `vite dev` 모드로 실행하세요.

### 주요 메서드

#### rhwpDev.search(text, includeCells?)

문서 영역에서 모든 매치를 일괄 검색합니다. 반환: `SearchHit[]`

```javascript
const hits = rhwpDev.search("사업계획");
// → 9 match(es), console.table 자동 표시
//
// [{sec: 0, para: 2, charOffset: 0, length: 13}, ...]

// 표 셀/글상자 내부 포함
const allHits = rhwpDev.search("사업계획", true);
```

#### rhwpDev.goto(hit, options?)

SearchHit으로 커서를 이동하고 화면을 스크롤합니다. 반환: `boolean`

```javascript
const hits = rhwpDev.search("사업계획");

// 3번째 매치로 이동 + 선택 (인덱스 2)
rhwpDev.goto(hits[2]);

// 캐럿만 이동 (선택 없음)
rhwpDev.goto(hits[2], { select: false });

// 한 줄 패턴
rhwpDev.goto(rhwpDev.search("text")[0]);
```

#### rhwpDev.showAllIds(pageNum?)

페이지의 모든 문단 ID를 `console.table`로 표시합니다.

```javascript
rhwpDev.showAllIds(0);    // 첫 페이지만
rhwpDev.showAllIds();     // 전체 페이지
```

#### rhwpDev.findNearest(targetId, pageNum?)

가장 가까운 paragraph 인덱스를 검색합니다.

```javascript
rhwpDev.findNearest(618);
// → { paraIdx: 0, distance: 618, text: "...", container: "cell[p0,c2,i0]" }
```

#### rhwpDev.help()

전체 사용법과 예시를 콘솔에 출력합니다.

### SearchHit 구조

```typescript
interface SearchHit {
  sec: number;          // 구역 인덱스
  para: number;         // 본문: 문단 인덱스 / 셀: parentPara
  charOffset: number;   // 문단 내 글자 오프셋
  length: number;       // 매치 길이
  cellContext?: {       // 셀/글상자 매치 시
    parentPara: number;
    ctrlIdx: number;
    cellIdx: number;
    cellPara: number;
  };
}
```

### 활용 예시 — 매치 일괄 강조

```javascript
// 모든 "회사" 매치를 순차적으로 보여주기
const hits = rhwpDev.search("회사");
let i = 0;
setInterval(() => {
  if (i >= hits.length) return;
  rhwpDev.goto(hits[i++]);
}, 1500);
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
