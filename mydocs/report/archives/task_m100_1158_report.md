# Task M100-1158 최종 보고서 — rhwp-studio 다크테마 지원

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16
- 기준 브랜치: `upstream/devel`

## 1. 완료 범위

- rhwp-studio에 `system | light | dark` 테마 설정을 추가했다.
- 설정 저장값과 실제 적용값을 분리해
  - 저장값: `system | light | dark`
  - 실제 적용값: `light | dark`
  구조로 정리했다.
- 앱 시작 시 `document.documentElement.dataset.themeMode`,
  `document.documentElement.dataset.themeEffective`,
  `color-scheme`를 반영하도록 했다.
- system 모드에서 `prefers-color-scheme` 변경을 따라가도록 했다.
- 보기 메뉴에 `테마 > 시스템 설정 / 밝게 / 어둡게` 항목을 추가하고 active 상태를 동기화했다.
- `meta[name="theme-color"]`가 현재 테마에 맞게 갱신되도록 했다.
- 메뉴바, 툴바, 서식바, 상태바, 작업영역, command palette, 공통 dialog, 주요 개별 dialog를
  semantic token 기반으로 전환했다.
- dark mode에서도 편집 용지는 흰색을 유지하고, 눈금자 body는 dark chrome 톤으로 분리했다.
- 눈금자 canvas는 theme 변경 시 palette를 다시 읽고 redraw 하도록 했다.
- 표/셀 선택 오버레이도 token 기반으로 정리했다.
- dark mode에서 검정 스프라이트 아이콘이 묻히던 문제를 dark 전용 스프라이트 교체 방식으로 정리했다.
- 테마 스모크 E2E `rhwp-studio/e2e/theme-mode.test.mjs`를 추가했다.

## 2. 비범위 / 유지한 값

- 문서 내용, 인쇄, export SVG/WASM 렌더 색은 테마에 따라 반전하지 않았다.
- `style-bar.css`의 글자색/형광펜 미리보기 막대는 실제 선택 색상 샘플이라 절대색을 유지했다.

## 3. 구현 스테이지

- Stage 1: 테마 설정 저장, DOM dataset 반영, 보기 메뉴 추가, 앱 chrome token화
- Stage 2: 개별 dialog/overlay 색상 정리
- Stage 3: 표 선택 오버레이 token화 및 잔여 절대색 최소화
- Stage 4: 시각 검토 후 눈금자 dark tone 정정 및 e2e 가드 보강
- Stage 5: dark toolbar/menu 검정 아이콘을 전용 스프라이트로 치환하고 desktop/mobile 시각 자료 확보

## 4. 검증

Stage 1 검증:

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`
- in-app browser에서 `http://localhost:7700` 로드 확인

Stage 2 검증:

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`

Stage 3 검증:

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`

Stage 4 검증:

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`
- dark 계산값 확인
  - `--ruler-bg=#2d333b`
  - `--ruler-body=#363c45`

Stage 5 검증:

- `cd rhwp-studio && npm run build`
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless`
- Chrome headless 모바일 캡처로 dark mobile 메인 화면/파일 메뉴 시각 확인

## 5. 시각 검토 자료

- desktop toolbar: `mydocs/report/assets/task_m100_1158_dark_toolbar_top.png`
- desktop file menu: `mydocs/report/assets/task_m100_1158_dark_menu_file.png`
- mobile main: `mydocs/report/assets/task_m100_1158_dark_mobile_main.png`
- mobile file menu: `mydocs/report/assets/task_m100_1158_dark_mobile_file_menu.png`

## 6. PR 준비 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `git diff --check`

## 7. 최종 판단

- theme 설정/저장/새로고침 유지/system 연동은 구현 완료
- dark mode의 주요 UI chrome과 대표 dialog는 token 기반으로 정리 완료
- 편집 용지는 dark mode에서도 흰색 유지
- 눈금자 본문은 dark mode에서 흰 종이처럼 남지 않도록 별도 dark tone으로 정정 완료
- dark toolbar/menu 아이콘은 desktop/mobile 모두 전용 스프라이트로 가시성 확보 완료
- 남은 절대색은 실제 색상 샘플 성격만 남겨 의도된 값으로 판단
