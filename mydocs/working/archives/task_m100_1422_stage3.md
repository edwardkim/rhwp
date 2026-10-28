# Stage 3 완료 보고서 — Task M100-1422

- 이슈: #1422
- 브랜치: `local/task1422`
- 단계: Stage 3 — 수식 편집 및 쪽 테두리/배경 정책 반영
- 완료 시각: 2026-06-17 01:30
- 보정 시각: 2026-06-17 01:36

## 1. 작업 요약

Stage 3에서는 수식 편집 미리보기와 쪽 테두리/배경 대화상자의 다크모드 잔여 대비 문제를 정리했다.

핵심 정책은 다음과 같다.

- 수식 미리보기는 저장 색상 또는 수식 SVG 색상을 임의 반전하지 않고, preview 배경만 문서 종이 토큰으로 분리한다.
- 쪽 테두리/배경 중앙 SVG 문서 preview는 실제 문서 preview 의미를 유지한다.
- 쪽 테두리/배경 주변 fieldset, legend, 라벨, 입력, 사방 버튼은 UI token을 사용한다.

## 2. 수정 파일

- `rhwp-studio/src/styles/dialogs.css`
- `rhwp-studio/src/ui/page-border-dialog.ts`

## 3. 상세 변경

### 3.1 수식 편집 미리보기

- `.eq-preview` 배경을 `var(--color-surface)`에서 `var(--doc-paper)`로 변경했다.
- 기본 검은 수식 SVG가 다크 UI 위에서 사라지는 문제를 preview 배경 정책으로 해결했다.
- `equation-editor-dialog.ts`의 preview 색상 전달 로직은 변경하지 않았다. 사용자가 지정한 수식 색상 의미를 유지하기 위해서다.

### 3.2 쪽 테두리/배경 대화상자

- 중앙 SVG preview의 배경과 내부 종이 rect fill을 `var(--doc-paper)`로 유지했다.
- preview 보조선 stroke를 문서 preview guide 전용 연회색으로 유지했다.
- fieldset border, legend, 라벨, checkbox/radio row 텍스트를 semantic token 기반으로 전환했다.
- 위치 입력과 비활성 숫자 입력에 `.dialog-input`을 적용해 Stage 1 공통 disabled/read-only 스타일을 재사용했다.
- 사방 적용 버튼에 `.page-border-side-btn` 클래스를 추가하고, 배경/테두리/글자색/hover를 token 기반으로 정리했다.
- 배경 탭의 `#ffffff` 기본값은 실제 문서 채움색 기본값이므로 유지했다.

## 4. 검증 결과

실행한 검증:

```bash
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm run build
cd rhwp-studio && CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

결과:

- TypeScript 타입 체크 통과
- Vite production build 통과
- `theme-mode.test.mjs` 전체 통과
- 브라우저 확인: `http://127.0.0.1:7701/` 로드 성공, title `rhwp-studio`
- 브라우저 확인: 콘솔 error/warn 0건
- 브라우저 확인: `.eq-preview` 배경 규칙이 `var(--doc-paper)`로 로드됨
- 브라우저 확인: `.page-border-side-btn` 배경/테두리/글자색/color-scheme이 token 기반으로 로드됨

## 4.1 보정 검증

작업지시자 확인으로 쪽 테두리/배경 미리보기 사방 버튼 클릭 시 dark mode에서 적용 전 안내선과 적용 후 테두리의 시각 차이가 부족한 문제가 확인되었다.

원인:

- 중앙 문서 preview는 dark mode에서도 흰 종이로 유지된다.
- 적용 전 안내선 stroke를 dark UI token인 `var(--ui-border-light)`로 바꾸면서 흰 preview 위에서 이미 적용된 테두리처럼 진하게 보였다.
- 실제 적용선은 `border.color`를 사용하므로, guide stroke와 실제 문서 테두리 색상은 분리되어야 한다.

보정:

- preview guide stroke를 `#d0d0d0`로 고정했다.
- 중앙 문서 preview와 실제 적용선 색상 의미는 유지했다.
- dark mode에서도 버튼 클릭 후 검은 적용선이 연한 guide 위에 명확히 보이도록 했다.

보정 후 검증:

- `npm run build` 통과
- `theme-mode.test.mjs --mode=headless` 통과
- 브라우저 확인: dark mode에서 guide stroke가 `rgb(208, 208, 208)`로 로드됨
- 브라우저 확인: 위쪽 사방 버튼 클릭 후 실제 적용선 `line`이 1개 추가되고 stroke가 `#000000`으로 유지됨

## 5. 잔여 작업

다음 승인 후 Stage 4를 진행한다.

- 표 만들기 quick grid popup
- 미주 모양 preview button/menu
- 문단 모양 preview
- 글머리표 popup
- validation/grid 관련 dialog
