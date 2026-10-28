# Task M100-258 Stage 27 — 일반 선택 색상 회귀 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `5b861c3f` (`task 258: 누름틀 선택 표시 반전 보정`)

## 1. 문제

Stage26에서 selection layer 전체를 검은 반전 방식으로 바꾸면서 일반 본문 텍스트와 누름틀 선택
색상까지 기존 파란 반투명 스타일에서 달라졌다.

작업지시자 기준은 일반 본문 텍스트와 누름틀 선택 모두 기존 rhwp-studio의 동일한 선택 색상으로
보여야 한다.

## 2. 수정 방향

- `SelectionRenderer`는 선택 스타일을 기존 파란 반투명으로 되돌린다.
- 일반 본문, 각주, 표 셀, 누름틀 선택 모두 동일한 선택 색상을 유지한다.
- 선택 중 누름틀 marker 숨김 보정은 유지한다.

## 3. 검증 계획

- 일반 본문 `dd|dddd...` 선택이 기존 파란 반투명 스타일인지 확인
- `abc[123][123]`의 `123123` 선택도 일반 본문과 같은 선택 색상인지 확인
- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `cargo fmt --check`
- `git diff --check`

## 4. 수행 결과

- `SelectionRenderer`의 `mix-blend-mode:difference`와 흰색 highlight를 제거하고 Stage25의 파란 반투명
  highlight(`rgba(51,144,255,0.35)`)로 복원했다.
- 선택 중 누름틀 marker를 숨기는 Stage26 보정은 유지했다.
- 검증용 직접 삽입 API는 인접 누름틀 범위를 실제 UI 생성 흐름과 다르게 합치는 부작용이 있어,
  색상 확인에는 실제 샘플 `samples/누름틀-2024.hwp`도 함께 사용했다.

## 5. 검증 결과

- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음
- Playwright 일반 본문 선택:
  - selection layer `mix-blend-mode: normal`
  - highlight background `rgba(51, 144, 255, 0.35)`
  - screenshot: `/tmp/rhwp-task258-stage27-normal-selection.png`
- Playwright 누름틀 샘플 선택:
  - `samples/누름틀-2024.hwp` 첫 누름틀 `11223344` 선택
  - selection layer `mix-blend-mode: normal`
  - highlight background `rgba(51, 144, 255, 0.35)`
  - screenshot: `/tmp/rhwp-task258-stage27-sample-clickhere-selection.png`
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (11 passed)
- `cd rhwp-studio && npm run build`: 통과
  - 기존 Vite chunk size warning만 표시
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
