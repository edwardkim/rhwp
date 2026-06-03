# Task M100-1063 Stage 1 완료 보고서 — 용지설정 방향 아이콘 식별성 정정

## 1. 작업 개요

용지 설정 대화창의 "용지 방향" 라디오 카드에서 세로/가로 가이드 아이콘이
시각적으로 구분되도록 정정했다.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/page-setup-dialog.ts` | 방향 SVG root에 `xmlns`, 방향별 class, `aria-hidden`, `focusable` 추가. landscape 아이콘을 40×28 가로 문서 형태로 보강 |
| `rhwp-studio/src/styles/dialogs.css` | `.icon-radio-icon svg` display/size 보존 규칙과 `.orient-icon-portrait`, `.orient-icon-landscape` 크기 명시 |
| `rhwp-studio/e2e/page-setup-orientation-icon.test.mjs` | 편집 용지 대화창을 열어 실제 DOM bounding box 비율을 검증하는 E2E 추가 |

## 3. 원인

`appendSvgMarkup()`은 SVG 문자열을 `DOMParser`로 파싱해 root를 import한다.
기존 방향 SVG root에는 `xmlns="http://www.w3.org/2000/svg"`가 없어서 headless
검증에서 namespace 없는 HTML 요소로 들어갔고, 실제 렌더 크기가 0×0으로 측정되었다.

이번 정정으로 방향 SVG root가 실제 SVG namespace를 갖고 렌더되며, CSS에서 방향별
실제 크기를 명시해 intrinsic size가 주변 flex/card 레이아웃에 의해 약해지지 않도록 했다.

## 4. 검증

### 4.1 Build

```bash
cd rhwp-studio
npm run build
```

결과:

- `tsc && vite build` 통과
- Vite build 완료
- 기존 chunk size warning만 출력

### 4.2 Studio 단위 테스트

```bash
cd rhwp-studio
npm test
```

결과:

- 실패
- 원인: 현재 `npm test` 스크립트가 `node --test tests/*.test.ts`로 `.ts` 파일을 직접 실행하며, Node v22.14.0에서 `ERR_UNKNOWN_FILE_EXTENSION` 발생
- 코드 정정 실패가 아니라 테스트 실행 방식 문제

우회 검증:

```bash
cd rhwp-studio
node --experimental-strip-types --test tests/*.test.ts
```

결과:

```text
1..26
# tests 26
# pass 26
# fail 0
```

### 4.3 E2E

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL=http://localhost:7700 \
node e2e/page-setup-orientation-icon.test.mjs --mode=headless
```

결과:

```text
PASS: 세로 방향 아이콘 존재
PASS: 가로 방향 아이콘 존재
PASS: 세로 아이콘은 height > width (28x36)
PASS: 가로 아이콘은 width > height (40x28)
PASS: 가로 아이콘 width가 세로 아이콘보다 큼 (40 > 28)
PASS: 세로 아이콘 height가 가로 아이콘보다 큼 (36 > 28)
```

스크린샷:

```text
rhwp-studio/e2e/screenshots/page-setup-orientation-icons.png
```

### 4.4 작업지시자 시각 판정

작업지시자 시각 판정:

```text
시각 판정 통과입니다.
```

결론: 용지 설정 대화창의 세로/가로 방향 아이콘 식별성 정정은 시각 게이트를 통과했다.

## 5. 범위 확인

- Rust/WASM 변경 없음
- HWP/HWPX/HWP3 파서 변경 없음
- PageDef 저장/폭길이 교환 로직 변경 없음
- frontend UI 시각 요소와 회귀 가드에 한정

## 6. 후속 Stage 2

Stage 1 시각 판정 통과 후, 작업지시자가 새 빈 문서의 편집 용지 대화창에서
기본 용지가 A4가 아니라 "사용자 정의"로 표시되는 문제를 추가 지시했다.

해당 정정은 `mydocs/working/task_m100_1063_stage2.md`에서 별도로 보고한다.
