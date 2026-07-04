# Task M100 #1456 Stage 2 완료보고서 — E2E 회귀 테스트 + 전후 대조 검증

- 이슈: #1456
- 브랜치: `local/task1456`
- 작성일: 2026-06-24
- 구현계획서: `mydocs/plans/task_m100_1456_impl.md` (2단계)

## 1. 수행 내용

E2E 회귀 테스트 [rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs](rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs) 신규 작성. 기존 `helpers.mjs`(`runTest`/`loadHwpFile`/`captureCanvasScreenshot`) 재사용 + 자체 픽셀 분석 헬퍼.

- **케이스 A (비공백)**: 새 세션에서 차트 HWP(`chart/세로막대형/묶은세로막대형.hwp`) 로드 → 재렌더 타이머·prefetch 발화 대기(~2.2s) → 첫 페이지 캔버스 PNG 의 **유채색 픽셀 비율 > 0.3%** 단언.
- **케이스 B (비재사용/고착)**: 같은 세션에서 다른 차트(`chart/원형/2차원원형.hwp`)로 교체 로드 → B 도 비공백 + **B↔A 픽셀 차이 > 2%** 단언(B가 A 캐시 픽셀 미재사용).
- 픽셀 헬퍼: `coloredPixelRatio`(채도>40 & 비백 → 유채색; 흑백 텍스트·백지 제외), `pixelDiffRatio`(채널합 차>30 픽셀 비율; 크기 불일치=1).

## 2. 실행 환경

- WASM: 본 수정은 **TS 전용**이라 기존 `pkg/`(빌드 산출물) 그대로 사용 — Docker 재빌드 불필요.
- 서버: `npx vite --host 127.0.0.1 --port 7700`.
- 브라우저: headless Chrome(`--mode=headless`, macOS `CHROME_PATH=/Applications/Google Chrome.app/...`).
- 산출물: `output/e2e/issue-1456/chart{A,B}.png`, `output/e2e/issue-1456-chart-rerender-report.html`(gitignore).

## 3. 검증 결과 — 전후(before/after) 대조

테스트의 회귀 유효성을 입증하기 위해 동일 테스트를 **수정 후 / 수정 전(negative control)** 양쪽에서 실행했다.

| 단언 | 수정 후(#1456 적용) | 수정 전(HEAD~1, rawSvgCount 제거) |
|---|---|---|
| 차트 A 유채색 픽셀 | **3.754%** > 0.3% → PASS | **0.011%** → **FAIL (공백)** |
| 차트 B 유채색 픽셀 | **2.834%** > 0.3% → PASS | **0.011%** → **FAIL (공백)** |
| 차트 B↔A 픽셀 차이 | **5.40%** > 2% → PASS | **0.00%** → **FAIL (고착)** |

- **수정 전**: 두 차트 모두 0.011%(사실상 백지)이며 B↔A 차이 0.00% — 이슈가 보고한 **첫 로드 공백 + 고착**이 그대로 재현됨.
- **수정 후**: 두 차트 모두 비공백으로 렌더(A 3.75% / B 2.83%)되고, 서로 다른 차트가 5.40% 차이로 구분됨.

→ 테스트가 결함을 결정적으로 포착하며, Stage 1 수정이 결함을 해소함이 **end-to-end 로 확인**됨.

## 4. Stage 3(#3 CanvasPool)에 대한 선결 관찰

수정 후 케이스 B 에서 **서로 다른 내용 차트**(세로막대형 ↔ 원형)가 5.40% 차이로 각각 정상 렌더됨(둘 다 비공백, 고착 없음). 즉 재렌더 트리거 보정만으로 *공백*뿐 아니라 *서로 다른 차트 간 고착*까지 해소된다 — 재렌더(`reRenderPageCanvases`)가 flow 캔버스를 통째로 다시 그려 CanvasPool 잔존 픽셀을 덮어쓰기 때문. **따라서 #3(CanvasPool 클리어)는 불필요**할 것으로 보인다. Stage 3 에서 정식 결론.

## 5. 다음 단계

Stage 3 — 고착 재현 여부 정식 판정(케이스 B 근거) → #3 불요 확정 + 기존 핵심 e2e 무회귀 확인.
