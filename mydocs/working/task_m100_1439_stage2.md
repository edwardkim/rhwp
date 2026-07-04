# Task M100 #1439 — 2단계 완료 보고서 (빌드 + 확장/웹 공통 + 회귀)

- 브랜치: `local/task1439`
- 작성일: 2026-06-19

## 1. 빌드 (확장/PWA)

- `npm run build` (tsc && vite build): **성공** (✓ built, PWA SW 생성, exit 0).
  - chunk 크기 경고는 기존 WASM/CanvasKit 청크 — 본 변경과 무관.
- 산출물 `dist/assets/index-*.js` 에 드롭 확인 게이트 문구("로컬 파일 열기 확인" 등)
  **포함 확인** → 확장/PWA 빌드 모두 게이트 동작.

## 2. 확장/웹 공통 동작

- 드롭 핸들러·대화상자는 `main.ts` 같은 페이지 컨텍스트. `chrome` API 의존 없는 순수
  DOM 모달이라 확장(standalone 탭 `/rhwp/`)·웹 동일 코드 경로. 빌드 산출물 단일.

## 3. 기존 e2e 회귀

테스트 인프라 의존성 누락 정정: `pixelmatch`/`pngjs` 가 package.json 선언돼 있으나
node_modules 미설치 → `npm install` 로 동기화(2 packages added).

- `e2e/unsaved-changes-guard.test.mjs` (host CDP, localhost:19222) **전부 PASS**:
  - dirty 상태 저장 확인 모달 표시 / 저장·저장 안 함·취소 버튼 / 취소 후 모달 닫힘·내용
    유지 / 저장 안 함 후 새 문서 전환.
  - → drop 핸들러 변경이 unsaved 가드 시나리오에 **회귀 없음** 확인 (unsaved-guard 는
    새 문서 입력 경로라 drop 미사용).

## 4. 다음 단계

- 3단계: 드롭 게이트 e2e 신규(드롭 시뮬 → 확인 표시 / [열기] 로딩 / [취소] 미로딩) +
  보안 가이드/감사 문서 반영 + 최종 보고서.
