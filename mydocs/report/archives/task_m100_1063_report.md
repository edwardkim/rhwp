# Task M100-1063 최종 보고서 — 용지설정 방향 아이콘 식별성 정정

## 1. 결과

완료.

용지 설정 대화창의 "용지 방향" 라디오 카드에서 세로/가로 가이드 아이콘이
실제 SVG로 렌더되고, 세로는 28×36, 가로는 40×28 비율로 명확히 구분되도록
정정했다.

추가로 새 빈 문서의 PageDef가 `59528×84186`으로 들어오는 경우에도 A4 프리셋으로
표시되도록, 편집 용지 대화창의 프리셋 매칭에 작은 HWPUNIT 허용 오차를 적용했다.

작업지시자 시각 판정:

```text
시각 판정 통과입니다.
```

Stage 2 작업지시자 시각 판정:

```text
시각 판정 통과입니다.
```

## 2. 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/page-setup-dialog.ts` | 방향 SVG root에 `xmlns`, 방향별 class, 접근성 제외 속성 추가. landscape 아이콘을 40×28 가로형 문서로 보강 |
| `rhwp-studio/src/ui/page-setup-dialog.ts` | 프리셋 매칭 허용 오차 3 HU 추가 |
| `rhwp-studio/src/styles/dialogs.css` | SVG display/flex size 보존 규칙과 방향별 크기 명시 |
| `rhwp-studio/e2e/page-setup-orientation-icon.test.mjs` | 방향 아이콘 실제 bounding box 비율 + 새 빈 문서 A4 프리셋 회귀 가드 추가 |
| `mydocs/working/task_m100_1063_stage1.md` | Stage 1 완료 보고 |
| `mydocs/working/task_m100_1063_stage2.md` | Stage 2 완료 보고 |

## 3. 원인

`appendSvgMarkup()` 경로에서 SVG 문자열을 DOMParser로 파싱해 root를 import한다.
기존 방향 아이콘 SVG에는 `xmlns="http://www.w3.org/2000/svg"`가 없어, 검증 과정에서
namespace 없는 HTML 요소로 들어가 실제 크기가 0×0으로 측정되었다.

이번 정정으로 SVG namespace를 명시하고, CSS에서 방향별 렌더 크기를 고정해 카드
레이아웃 안에서도 세로/가로 비율이 보존되도록 했다.

Stage 2에서는 새 빈 문서의 `height=84186`과 A4 프리셋 `height=84188`의 2 HU
차이 때문에 exact compare가 실패하는 것을 확인했다. 2 HU는 약 0.007mm라 같은
A4로 판정하도록 프리셋 매칭 허용 오차를 적용했다.

## 4. 검증

```bash
cd rhwp-studio
npm run build
```

- 통과
- 기존 chunk size warning만 출력

```bash
cd rhwp-studio
node --experimental-strip-types --test tests/*.test.ts
```

- 26/26 통과

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL=http://localhost:7700 \
node e2e/page-setup-orientation-icon.test.mjs --mode=headless
```

- 통과
- 측정값:
  - 새 빈 문서 용지 종류: A4
  - 세로 아이콘: 28×36
  - 가로 아이콘: 40×28
- 작업지시자 시각 판정 통과

참고: `npm test`는 현재 `node --test tests/*.test.ts`가 Node v22.14.0에서
`.ts` 확장자를 직접 로드하지 못해 `ERR_UNKNOWN_FILE_EXTENSION`으로 실패한다.
우회 실행인 `node --experimental-strip-types --test tests/*.test.ts`는 통과했다.

## 5. 범위

- frontend UI 한정
- Rust/WASM 변경 없음
- HWP/HWPX/HWP3 파서 변경 없음
- PageDef 저장/폭길이 교환 로직 변경 없음
