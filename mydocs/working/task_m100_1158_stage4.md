# Stage 4 — 시각 검토 후 눈금자 다크톤 정정

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16
- 선행 커밋: `54f4519a` (`task 1158: 선택 오버레이와 보고서 정리`)

## 1. 목적

작업지시자 시각 판정에 바로 사용할 수 있도록 rhwp-studio dark mode 적용 결과를
대표 상태별 스크린샷과 간단한 점검 메모로 정리하고, 시각 검토에서 드러난 눈금자 dark tone
불일치를 정정한다.

## 2. 캡처 범위

- `http://localhost:7700`
- desktop viewport
  - light
  - dark
- mobile 계열 좁은 viewport
  - dark
- 필요 시 dialog 또는 메뉴가 열린 상태 1종 추가

## 3. 확인 포인트

- 앱 chrome이 light/dark 전환에 맞게 바뀌는지
- 편집 용지는 흰색을 유지하되, 눈금자 본문은 dark theme chrome 톤과 분리되어 보이는지
- 메뉴/툴바/상태바 대비가 깨지지 않는지
- console error/warn이 추가로 발생하지 않는지

## 4. 예정 산출물

- 스크린샷: `/tmp/task1158-visual-*.png`
- 점검 메모: 이 문서 5~6절 업데이트

## 5. 실행 결과

주요 산출물:

- 샘플 문서 로드 시각 검토 세트
  - `/tmp/task1158-visual-sample-light-desktop.png`
  - `/tmp/task1158-visual-sample-dark-desktop.png`
  - `/tmp/task1158-visual-sample-dark-mobile.png`
  - 요약 JSON: `/tmp/task1158-visual-sample-summary.json`
- 빈 시작 화면 보조 세트
  - `/tmp/task1158-visual-light-desktop.png`
  - `/tmp/task1158-visual-dark-desktop.png`
  - `/tmp/task1158-visual-dark-mobile.png`
  - 요약 JSON: `/tmp/task1158-visual-summary.json`

시각 검토 기준 샘플:

- `http://localhost:7700/?url=/samples/para-001.hwp&filename=para-001.hwp`

확인 결과:

- light desktop
  - `themeMode=light`, `themeEffective=light`
  - `theme-color=#f5f5f5`
  - workspace 배경은 밝은 회색, 문서 canvas 배경은 `rgb(255, 255, 255)`
- dark desktop
  - `themeMode=dark`, `themeEffective=dark`
  - `theme-color=#2b3037`
  - workspace 배경은 어두운 회색(`rgb(31, 35, 41)`), 문서 canvas 배경은 계속 `rgb(255, 255, 255)`
- dark mobile
  - `themeMode=dark`, `themeEffective=dark`
  - 좁은 viewport에서도 상단 chrome은 dark, 문서 본문 canvas는 흰색 유지
- 샘플 문서 상태바
  - `1 / 3 쪽`
  - `para-001.hwp — 3페이지`
- console 경고/오류
  - `/tmp/task1158-visual-sample-summary.json` 기준 없음

추가 판정 및 수정:

- 작업지시자 피드백: "눈금자가 정확히 다크모드로 동작 안함."
- 원인:
  - dark override에서 `--ruler-body`가 여전히 `#ffffff`로 남아 있었다.
  - 그래서 theme 변경 후 redraw는 되더라도, 눈금자 본문 band가 흰 종이처럼 보여 문서 용지와
    UI chrome 구분이 흐려졌다.
- 수정:
  - dark theme의 `--ruler-body`를 `--ui-surface-raised`(`#363c45`)로 정정
  - `theme-mode.test.mjs`에 dark ruler canvas 샘플 검증 추가
- 수정 후 확인:
  - dark 계산값
    - `--ruler-bg=#2d333b`
    - `--ruler-body=#363c45`
  - 편집 용지는 계속 `rgb(255, 255, 255)` 유지
  - 눈금자 본문은 dark tone으로 내려가고, 흰 종이와 시각적으로 분리
  - `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless` 통과
  - `cd rhwp-studio && npm run build` 통과

브라우저 검증 메모:

- in-app browser로 URL, DOM snapshot, console 상태는 확인했다.
- 다만 브라우저 플러그인 쪽 `Page.captureScreenshot`가 반복 타임아웃되어
  스크린샷 파일 생성은 로컬 headless Chrome(`puppeteer-core`)로 우회했다.

## 6. 작업지시자 확인 대기

- 눈금자 dark tone 정정 후 시각 재판정 진행
