# Stage 2 — 개별 다이얼로그와 보조 UI의 다크테마 잔여 색상 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16

## 1. 목적

Stage 1에서 앱 chrome과 공통 dialog token 기반은 갖췄지만, 여러 개별 다이얼로그 CSS는 아직
light 고정 색상을 직접 사용한다. 이번 Stage 2에서는 사용 빈도가 높고 눈에 띄는 dialog/overlay
계열을 우선 token 기반으로 바꿔 dark mode에서 이질감이 없게 만든다.

## 2. 이번 Stage 범위

- `char-shape-dialog.css`
- `bookmark-dialog.css`
- `symbols-dialog.css`
- `table-cell-props.css`
- `compare-dialog.css`
- `para-shape-dialog.css`
- `font-set-dialog.css`
- `picture-props.css`
- `find-dialog.css`
- `numbering-dialog.css`
- `shape-picker.css`
- `style-dialog.css`
- `form-overlay.css`

## 3. 현재 판단

- 모든 색상을 한 번에 semantic token으로 완전 치환하기보다, dialog 배경/테두리/선택/hover/primary
  상태를 우선 정리하는 편이 안전하다.
- 문서 선택 오버레이, 양식 오버레이처럼 의미 있는 강조색은 절대 색을 유지하더라도 dark 배경에서
  충분히 보이는지만 우선 확인한다.
- 비교 다이얼로그처럼 자체 팔레트가 많은 화면은 기존 hue는 유지하고 밝기/배경/테두리만 token과
  맞춘다.

## 4. 예정 검증

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

필요 시 in-app browser로 `localhost:7700` 화면을 다시 확인한다.

## 5. 실행 결과

반영 내용:

- 다이얼로그/패널/오버레이 계열의 입력창, 목록, 선택행, hover, 정보 패널, popup border를
  semantic token 위주로 전환
- 선택 항목의 강조색과 dark 배경 대비를 `--ui-selected`, `--color-primary`,
  `--ui-text-on-accent` 기준으로 정리
- 비교/양식/글꼴/도형 선택 계열에서 light 고정 border/background를 제거
- `--ui-on-accent-overlay` token 추가로 accent 위 badge/overlay도 dark/light 모두 대응

검증 결과:

- `cd rhwp-studio && npm run build` 통과
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless` 통과
  - system/dark/light 전환
  - localStorage 유지
  - `theme-color` 반영
  - 편집 용지 흰색 유지

## 6. 현재 판단

- Stage 2 범위에 넣은 주요 dialog CSS는 dark token 기준으로 1차 정리를 마쳤다.
- 아직 `options-dialog.css`, 일부 선택/렌더 전용 CSS, 토큰 정의 자체의 절대색은 남아 있지만,
  이번 Stage 2 범위 밖이거나 semantic token 정의 성격이라 별도 분리가 맞다.
- 다음 Stage가 필요하면 실제 시각 확인 후 덜 어울리는 dialog만 추려서 좁게 가는 편이 안전하다.

## 7. 작업지시자 확인 대기

- Stage 2 수정 후 기본 빌드/스모크를 다시 돌리고 시각 확인 포인트와 함께 승인 요청한다.
- Stage 2 변경분 커밋 승인 대기
