# Stage 1 — 테마 설정 기반과 UI chrome 토큰 연결

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16

## 1. 목적

`system | light | dark` 테마 설정을 저장하고, rhwp-studio의 앱 chrome이 해당 설정에 따라
일관되게 바뀌는 최소 기반을 만든다.

이번 Stage 1 범위:

- `rhwp-settings`에 theme mode 저장
- 앱 시작 시 `data-theme-mode`, `data-theme-effective` 반영
- system 모드의 `prefers-color-scheme` 연동
- 보기 메뉴 `테마` 항목 추가
- 메뉴바, 도구 상자, 서식 도구 모음, 편집 작업영역, 상태바, command palette, 공통 dialog 색상 토큰화 시작
- 편집 용지 흰색 유지

## 2. 현재 판단

- 기존 `user-settings.ts`가 단일 localStorage 서비스 역할을 하고 있어 theme도 같은 저장소에 넣는 것이 가장 작다.
- 테마의 저장값과 실제 적용값은 분리해야 한다.
  - 저장값: `system | light | dark`
  - 실제 적용값: `light | dark`
- CSS는 기존 `--color-*`를 완전히 제거하지 않고, 신규 semantic token을 alias로 연결해 회귀를 줄인다.
- 눈금자는 canvas 직접 렌더링이라 CSS만으로는 바뀌지 않으므로 theme 변경 시 다시 그리는 경로가 필요하다.

## 3. 구현 중 포인트

- 신규 `rhwp-studio/src/core/theme.ts`
- 수정:
  - `rhwp-studio/src/core/user-settings.ts`
  - `rhwp-studio/src/main.ts`
  - `rhwp-studio/src/command/commands/view.ts`
  - `rhwp-studio/index.html`
  - `rhwp-studio/src/view/ruler.ts`
  - `rhwp-studio/src/styles/base.css`
  - 주요 chrome CSS

## 4. 검증 대기

실행 검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

보조 확인:

```bash
Browser plugin(iab)로 http://localhost:7700 로드 확인
```

검증 결과:

- `npm run build` 통과
- `theme-mode.test.mjs --mode=headless` 통과
  - `system | dark | light` 전환
  - 메뉴 active 상태 반영
  - `localStorage` 저장/새로고침 유지
  - `meta[name="theme-color"]` 갱신
  - dark/light 모두 편집 용지 흰색 유지
- Browser plugin(iab)에서 `localhost:7700` 로드 및 메뉴바/테마 dataset 확인
- 호스트 Chrome CDP 경로 `http://172.21.192.1:19222` 는 현재 세션에서 `ENETUNREACH`로 직접 e2e 실행 불가

## 5. 현재 결과

이번 Stage 1에서 반영된 핵심:

- `theme.mode` 저장/로드 (`system | light | dark`)
- 앱 시작 시 `data-theme-mode`, `data-theme-effective`, `color-scheme` 반영
- system 모드의 OS 다크 설정 연동
- 보기 메뉴 `테마` 서브메뉴 및 active 상태 동기화
- 메뉴바/툴바/서식바/상태바/작업영역/command palette/공통 dialogs 주요 토큰화
- 눈금자 다크 테마 redraw
- e2e 스모크 테스트 `rhwp-studio/e2e/theme-mode.test.mjs` 추가

## 6. 다음 스테이지 후보

Stage 1 검증은 통과했지만, 개별 다이얼로그 CSS에는 아직 light 고정 색상이 남아 있다.

대표 잔여 파일:

- `rhwp-studio/src/styles/char-shape-dialog.css`
- `rhwp-studio/src/styles/bookmark-dialog.css`
- `rhwp-studio/src/styles/symbols-dialog.css`
- `rhwp-studio/src/styles/table-cell-props.css`
- `rhwp-studio/src/styles/compare-dialog.css`
- `rhwp-studio/src/styles/para-shape-dialog.css`
- `rhwp-studio/src/styles/font-set-dialog.css`
- `rhwp-studio/src/styles/picture-props.css`
- `rhwp-studio/src/styles/find-dialog.css`
- `rhwp-studio/src/styles/numbering-dialog.css`
- `rhwp-studio/src/styles/shape-picker.css`
- `rhwp-studio/src/styles/style-dialog.css`
- `rhwp-studio/src/styles/form-overlay.css`

다음 단계는 Stage 1 커밋 후 별도 Stage 문서에서 위 대화상자 계열을 dark token 기준으로 정리한다.

## 7. 작업지시자 확인 대기

- Stage 1 변경분 커밋 승인 대기
- 승인 후 다음 Stage 문서 생성 및 대화상자 잔여 색상 정리 진행
