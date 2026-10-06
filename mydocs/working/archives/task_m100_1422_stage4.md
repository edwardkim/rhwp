# Stage 4 완료 보고서 — Task M100-1422

- 이슈: #1422
- 브랜치: `local/task1422`
- 단계: Stage 4 — 추가 라이트 하드코딩 sweep
- 완료 시각: 2026-06-17 01:57

## 1. 작업 요약

Stage 4에서는 이슈 본문에 추가 후보로 등록된 popup/dialog의 남은 light UI surface 하드코딩을 정리했다.

핵심 기준은 다음과 같다.

- UI chrome: surface, border, hover, label, popup menu는 semantic token을 사용한다.
- 문서 preview: 실제 문서처럼 보여야 하는 preview는 `--doc-paper`와 고정 문서 텍스트 색을 사용한다.
- 실제 색상값: 색상 팔레트, 색상 견본, 문서 배경/테두리 기본값은 테마 token으로 바꾸지 않는다.

## 2. 수정 파일

- `rhwp-studio/src/ui/table-create-dialog.ts`
- `rhwp-studio/src/ui/endnote-shape-dialog.ts`
- `rhwp-studio/src/ui/grid-settings-dialog.ts`
- `rhwp-studio/src/ui/validation-modal.ts`
- `rhwp-studio/src/styles/para-shape-dialog.css`
- `rhwp-studio/src/ui/para-shape-dialog.ts`
- `rhwp-studio/src/ui/para-shape-tab-builders.ts`
- `rhwp-studio/src/ui/toolbar.ts`

## 3. 상세 변경

### 3.1 표 만들기 quick grid

- popup surface, border, shadow, label, footer hover를 token 기반으로 전환했다.
- quick grid cell 기본/hover 상태를 `--color-surface`, `--color-border`, `--color-accent-bg-light`, `--color-primary`로 정리했다.

### 3.2 미주 모양

- fieldset/legend, label, unit 텍스트를 token 기반으로 전환했다.
- 선 종류/굵기 preview button과 dropdown menu surface, option hover를 token 기반으로 전환했다.
- 색상 팔레트의 실제 색상값은 유지하고, swatch wrapper/active outline만 token 기반으로 정리했다.

### 3.3 문단 모양

- 문단 preview 배경을 `--doc-paper`로 분리했다.
- 문단 preview 텍스트는 문서 preview 성격에 맞춰 검은 문서 텍스트로 유지했다.
- 문단 테두리 preview 배경도 `--doc-paper`로 분리하고, 적용 전 guide는 연회색으로 유지했다.

### 3.4 글머리표 popup

- popup surface, border, shadow, cell 기본/hover 상태를 token 기반으로 전환했다.

### 3.5 validation/grid 관련 dialog

- validation modal의 summary/detail surface/link/hint 색상을 token 기반으로 전환했다.
- grid settings의 fieldset/legend/label/unit/input 색상을 token 기반으로 전환하고, 숫자 입력에는 `.dialog-input`을 적용했다.

## 4. 유지한 실제 색상값

다음 값들은 UI theme 색상이 아니라 문서 데이터 또는 사용자 선택 색상 의미를 가지므로 유지했다.

- 색상 팔레트 swatch 값
- 문단/미주 테두리 기본 색상 `#000000`
- 문단 배경 기본 색상 `#ffffff`
- 문서 preview 텍스트 `#111111`
- 문서 preview guide `#d0d0d0`

## 5. 검증 결과

실행한 검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

결과:

- Vite production build 통과
- `theme-mode.test.mjs` 전체 통과
- 브라우저 확인: `http://127.0.0.1:7701/` 로드 성공, effective theme `dark`
- 브라우저 확인: 콘솔 error/warn 0건
- 브라우저 확인: 글머리표 popup surface/cell이 dark token 계산값으로 표시됨
- 브라우저 확인: 표 만들기 quick grid popup surface/cell이 dark token 계산값으로 표시됨
- 브라우저 확인: 문단 모양 preview는 `rgb(255, 255, 255)` 배경과 `rgb(17, 17, 17)` 텍스트로 표시됨
- 브라우저 확인: grid settings fieldset/input이 dark token 계산값으로 표시됨
- 브라우저 확인: 미주 모양 preview button/dropdown이 dark token 계산값으로 표시됨

## 6. 잔여 작업

다음 승인 후 Stage 5를 진행한다.

- focused dialog theme 회귀 가드 추가 여부 판단
- 자동화 가능한 핵심 색상 정책을 DOM/e2e로 고정
