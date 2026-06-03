# Task M100-1063 구현 계획서 — 용지 방향 아이콘 식별성 정정

## 1. 구현 방침

이번 작업은 `rhwp-studio`의 편집 용지 대화창 UI에 한정한다.

핵심 방침:

- SVG 문자열 자체의 방향 정보를 DOM/CSS에서 안정적으로 보존한다.
- CSS에서 아이콘 SVG의 display와 실제 width/height를 명시한다.
- `landscape` 아이콘은 세로 아이콘과 같은 형태로 오인되지 않도록 내부 선 구성도 가로형 문서처럼 보강한다.
- 기존 라디오 선택/폭길이 교환 로직은 변경하지 않는다.

## 2. 수정 대상

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/src/ui/page-setup-dialog.ts` | `ORIENT_ICONS` SVG root에 방향별 class 부여. 필요 시 landscape SVG 내부 선/외곽 표현 조정 |
| `rhwp-studio/src/styles/dialogs.css` | `.icon-radio-icon svg` display/size 보존 규칙 추가. `.orient-icon-portrait`, `.orient-icon-landscape` 크기 명시 |
| `rhwp-studio/e2e/page-setup-orientation-icon.test.mjs` | 신규 E2E. 용지 설정 대화창을 열고 두 방향 아이콘의 bounding box 비율을 검증 |

## 3. 세부 구현

### 3.1 `page-setup-dialog.ts`

현재:

```typescript
const ORIENT_ICONS: Record<string, string> = {
  portrait:  `<svg width="28" height="36" viewBox="0 0 28 36">...</svg>`,
  landscape: `<svg width="36" height="28" viewBox="0 0 36 28">...</svg>`,
};
```

계획:

- SVG root에 class를 추가한다.
  - `class="orient-icon orient-icon-portrait"`
  - `class="orient-icon orient-icon-landscape"`
- `landscape` SVG는 외곽 rect와 내부 텍스트 라인의 가로성이 더 분명하도록 유지/보강한다.
- 라벨 텍스트, radio value, event handler는 변경하지 않는다.

### 3.2 `dialogs.css`

추가 후보:

```css
.icon-radio-icon svg {
  display: block;
  flex: 0 0 auto;
}

.orient-icon-portrait {
  width: 28px;
  height: 36px;
}

.orient-icon-landscape {
  width: 36px;
  height: 28px;
}
```

필요 시 `.icon-radio-icon`에 안정적인 inline-size를 부여한다.

```css
.icon-radio-icon {
  width: 40px;
}
```

단, 이 값은 제본 아이콘까지 함께 영향을 받으므로 실제 DOM/시각 확인 후 적용 여부를 결정한다.

### 3.3 E2E 테스트

신규 파일:

```text
rhwp-studio/e2e/page-setup-orientation-icon.test.mjs
```

검증 흐름:

1. `helpers.mjs`의 `runTest`, `loadApp`, `createNewDocument`, `screenshot`, `assert` 사용.
2. 앱 로드 후 새 문서 생성.
3. F7 또는 command dispatch로 편집 용지 대화창 열기.
4. `.orient-icon-portrait`, `.orient-icon-landscape` 선택.
5. `getBoundingClientRect()`로 실제 렌더 크기 측정.
6. 다음 조건 assert:
   - portrait: `height > width`
   - landscape: `width > height`
   - landscape width가 portrait width보다 큼
   - portrait height가 landscape height보다 큼
7. screenshot 저장: `page-setup-orientation-icons`.

## 4. 검증 명령

기본 검증:

```bash
cd rhwp-studio
npm run build
```

E2E 검증:

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700
node e2e/page-setup-orientation-icon.test.mjs --mode=headless
```

필요 시 전체 관련 E2E:

```bash
cd rhwp-studio
node e2e/responsive.test.mjs --mode=headless
```

## 5. 완료 산출물

- 소스 정정 2파일 이내
- E2E 회귀 가드 1파일
- 단계 완료 보고서:

```text
mydocs/working/task_m100_1063_stage1.md
```

## 6. 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| CSS가 제본 아이콘에도 영향을 줌 | 방향 아이콘 전용 class 규칙을 우선 적용 |
| 모바일 대화상자에서 카드 폭이 흔들림 | desktop + mobile viewport screenshot 확인 |
| 실제 문제 원인이 SVG 크기가 아니라 시각 대비 부족 | landscape SVG 내부 표현을 가로형 문서로 더 명확히 조정 |
| E2E가 로컬 Chrome 경로에 의존 | `--mode=headless`와 기존 `helpers.mjs` 정책 사용 |

## 7. 승인 요청

본 구현 계획 승인 후 다음 순서로 진행한다.

1. 방향 SVG class와 CSS size 정책 정정
2. E2E 회귀 가드 추가
3. `npm run build` 및 E2E 검증
4. 단계 보고서 작성

---

## 8. Stage 2 구현 계획 — 빈 문서 기본 용지 A4 프리셋 매칭

## 8.1 증상

새 빈 문서를 생성한 뒤 편집 용지 대화창을 열면 용지 종류가 A4가 아니라
"사용자 정의"로 표시된다.

## 8.2 원인

브라우저에서 새 빈 문서의 PageDef를 확인했다.

```text
width=59528
height=84186
landscape=false
```

현재 `PAPER_PRESETS`의 A4는 다음 값이다.

```typescript
['A4', 59528, 84188]
```

`populateFields()`는 exact compare만 수행한다.

```typescript
const matched = PAPER_PRESETS.find(([, pw, ph]) =>
  (pw === pd.width && ph === pd.height) || (pw === pd.height && ph === pd.width)
);
```

따라서 높이 2 HWPUNIT 차이 때문에 A4가 아니라 `custom`으로 판정된다.
2 HWPUNIT는 약 0.007mm라 사용자 눈에는 같은 A4이며, blank template/한컴 저장
반올림 차이로 보는 것이 타당하다.

## 8.3 수정 방침

`PageSetupDialog` 내부의 프리셋 판정에 작은 HWPUNIT 허용 오차를 둔다.

후보 구현:

```typescript
const PAPER_PRESET_TOLERANCE_HU = 3;

function samePaperSize(a: number, b: number): boolean {
  return Math.abs(a - b) <= PAPER_PRESET_TOLERANCE_HU;
}

function matchesPaperPreset(width: number, height: number, presetWidth: number, presetHeight: number): boolean {
  return (
    samePaperSize(width, presetWidth) && samePaperSize(height, presetHeight)
  ) || (
    samePaperSize(width, presetHeight) && samePaperSize(height, presetWidth)
  );
}
```

`populateFields()`의 exact compare를 `matchesPaperPreset()`로 교체한다.

## 8.4 범위

포함:

- `rhwp-studio/src/ui/page-setup-dialog.ts`
  - 프리셋 매칭 tolerance helper 추가
  - `populateFields()` 매칭 로직 교체
- `rhwp-studio/e2e/page-setup-orientation-icon.test.mjs`
  - 기존 대화창 E2E에 새 빈 문서의 `paperSelect.value === 'A4'` 검증 추가

제외:

- `saved/blank2010.hwp` 템플릿 수정
- Rust `createBlankDocument` / `getPageDef` / `setPageDef` 변경
- HWP 저장 직렬화 변경

## 8.5 검증

```bash
cd rhwp-studio
npm run build
```

```bash
cd rhwp-studio
node --experimental-strip-types --test tests/*.test.ts
```

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL=http://localhost:7700 \
node e2e/page-setup-orientation-icon.test.mjs --mode=headless
```

추가 E2E 기대:

```text
PASS: 새 빈 문서의 용지 종류는 A4
```

## 8.6 승인 요청

Stage 2는 새 요구사항이므로, 본 구현 계획 승인 후 소스 수정에 들어간다.
