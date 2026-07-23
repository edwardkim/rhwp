# Task M100 #3126 Stage 3 — Chrome/CDP PDF 회귀 검증

- 작성일: 2026-07-23
- 상태: **Chrome 완료, Edge 수동 게이트 잔여**
- E2E: `rhwp-studio/e2e/print-pdf-issue3126.test.mjs`

## 1. 자동화 범위

native print dialog는 브라우저 자동화가 안정적으로 제어할 수 없으므로 검증을 두 층으로 분리했다.

1. 실제 macOS Chrome에서 `PDF로 저장…` 한 번 클릭 뒤 native print UI 자동 호출 확인
2. headless Chrome에서 iframe `print()`만 가로채 인쇄 DOM과 상태를 검사한 뒤, 동일 HTML을
   CDP `printToPDF`로 PDF화

가로채기는 제품 코드를 우회하지 않는다. 실제 명령이 생성한 same-origin iframe의 load 직후
`contentWindow.print`만 test double로 바꾸며 SVG/profile/CSS/font/layout pipeline은 그대로
실행한다.

## 2. fixture와 결과

| fixture | 문서/생성 PDF | 주요 검증 |
|---|---:|---|
| `render-p35-font-native-bitmap.hwpx` (#2524) | 1쪽 / 1쪽 | embedded `@font-face` data URI, SVG text, PDF text 추출 |
| `hwpx/hwpx-02.hwpx` (#2525) | 7쪽 / 7쪽 | 모든 페이지, 7개 named `@page`, 재래핑 회귀, PDF text 추출 |

두 PDF 모두 `%PDF` 매직과 `pdfinfo` 페이지 수가 정합했고 `pdftotext` 결과가 비어 있지 않았다.
#2524 PDF는 A4 594.96 × 841.92 pt, #2525 첫 페이지도 같은 A4 크기로 확인했다.

혼합 크기·방향 자체는 기존 `print-pages` test가 portrait/landscape 크기의 named `@page`를
페이지별로 단언한다. 브라우저 E2E는 이 CSS를 실제 print DOM에서 확인하고 #2525 다중 페이지
CDP 출력을 검증한다.

## 3. 상태·UX assertion

- 별도 `PDF로 저장…` label과 남은 단계 tooltip
- exact same-origin `/print.html`, `about:blank` 부재
- 한 번 클릭으로 `print()` 정확히 1회 자동 호출
- print 호출 시 상태바 `PDF 준비 완료`
- file handle·파일명·dirty 상태가 전/호출/후 모두 동일
- 호출 반환 뒤 iframe 제거
- searchable SVG `<text>`와 body text 보존

## 4. 전체 검증

| 명령 | 결과 |
|---|---|
| `npm test` | 513/513 |
| `npm run build` | 통과 |
| focused print tests | 12/12 |
| #2524 Rust regression | 4/4 |
| #2525 Rust regression | 1/1 |
| headless Chrome #3126 E2E | 두 시나리오 전 assertion 통과 |
| `git diff --check` | 통과 |

## 5. 잔여 게이트

현재 머신에 Microsoft Edge가 없어 다음 항목은 merge/issue close 전에 Windows Edge에서 수행한다.

1. 파일 메뉴 `PDF로 저장…` 클릭
2. 진행 표시 뒤 native print UI가 자동으로 열리는지 확인
3. 대상 `PDF로 저장`, A4/혼합 용지 크기와 7쪽 문서 페이지 수 확인
4. 취소 뒤 문서 제목과 dirty 표시가 그대로인지 확인

Edge에서 iframe print 또는 named `@page` 결함이 실증될 때만 Stage 0의 same-origin 전용 창
fallback을 구현한다.
