# Task M100 #1448 Stage 4: 새 문서 복구 정책 검증 보강

- 이슈: #1448 `rhwp-studio: 미저장 문서 자동 백업 및 복구 기능`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1448`
- 작성일: 2026-06-21

## 1. 목표

이슈 수용 기준 중 `새 문서, 기존 HWP 문서, HWPX 출처 문서에 대한 동작 정책이 테스트된다` 항목에서
새 문서 복구 정책이 E2E에 명시되지 않은 점을 보강한다.

## 2. 구현 범위

- 자동 백업 복구 E2E에 새 문서 draft 복구 케이스 추가
- 새 문서 복구본 파일명, 페이지 수, dirty 상태, 복구 후 draft 삭제 확인
- 기존 HWP/HWPX 출처 복구 E2E는 유지

## 3. 제외 범위

- 제품 로직 변경
- 새 문서 전용 복구 UI 분기
- 복구 후보 관리 전용 화면

## 4. 진행 기록

- 2026-06-21: #1448 수용 기준 재검토 중 새 문서 복구 정책 E2E 공백 확인.
- 2026-06-21: 자동 백업 복구 E2E에 새 문서 draft 복구 케이스 추가.

## 5. 검증

- `CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node e2e/autosave-recovery.test.mjs --mode=headless`
  - 새 문서 draft 복구 통과
  - 기존 HWP draft 복구 통과
  - HWPX 출처 draft가 HWP 복구본으로 열림 확인
- `npm test`
  - 84개 통과
- `npx tsc --noEmit`
  - 통과
- `npm run build`
  - 통과

## 6. 판정

- #1448 수용 기준의 새 문서/HWP/HWPX 정책 테스트 항목까지 충족.
