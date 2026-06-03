# Task M100-1063 Stage 2 완료 보고서 — 새 빈 문서 A4 프리셋 매칭 정정

## 1. 작업 개요

새 빈 문서를 만든 뒤 편집 용지 대화창을 열면 기본 용지가 A4가 아니라
"사용자 정의"로 표시되는 문제를 정정했다.

## 2. 원인

새 빈 문서의 실제 PageDef:

```text
width=59528
height=84186
landscape=false
```

PageSetupDialog의 A4 프리셋:

```text
width=59528
height=84188
```

기존 `populateFields()`는 exact compare만 수행했으므로, 높이 2 HWPUNIT 차이 때문에
A4 프리셋으로 매칭되지 않았다. 2 HWPUNIT는 약 0.007mm라 사용자가 보는 용지
크기에서는 A4로 처리하는 것이 맞다.

## 3. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/page-setup-dialog.ts` | 프리셋 매칭 허용 오차 `PAPER_PRESET_TOLERANCE_HU = 3` 추가, exact compare를 tolerance 기반 매칭으로 교체 |
| `rhwp-studio/e2e/page-setup-orientation-icon.test.mjs` | 새 빈 문서의 용지 종류가 A4로 표시되는지 검증 추가 |
| `mydocs/orders/20260522.md` | Stage 2 진행 상태 갱신 |

## 4. 검증

### 4.1 Build

```bash
cd rhwp-studio
npm run build
```

결과:

- 통과
- 기존 chunk size warning만 출력

### 4.2 Studio 단위 테스트

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
PASS: 새 빈 문서의 용지 종류는 A4 (A4)
PASS: 세로 방향 아이콘 존재
PASS: 가로 방향 아이콘 존재
PASS: 세로 아이콘은 height > width (28x36)
PASS: 가로 아이콘은 width > height (40x28)
PASS: 가로 아이콘 width가 세로 아이콘보다 큼 (40 > 28)
PASS: 세로 아이콘 height가 가로 아이콘보다 큼 (36 > 28)
```

### 4.4 작업지시자 시각 판정

작업지시자 시각 판정:

```text
시각 판정 통과입니다.
```

결론: 새 빈 문서의 편집 용지 대화창이 A4로 표시되는 정정은 시각 게이트를 통과했다.

## 5. 범위 확인

- Rust/WASM 변경 없음
- `saved/blank2010.hwp` 템플릿 변경 없음
- `createBlankDocument`, `getPageDef`, `setPageDef` 변경 없음
- 편집 용지 대화창의 프리셋 표시 판정만 변경
