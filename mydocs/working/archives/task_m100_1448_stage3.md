# Task M100 #1448 Stage 3: HWPX 복구 정책과 실제 샘플 검증

- 이슈: #1448 `rhwp-studio: 미저장 문서 자동 백업 및 복구 기능`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1448`
- 작성일: 2026-06-21

## 1. 목표

Stage 1/2에서 구현한 자동 백업과 복구 UI를 실제 HWP/HWPX 샘플로 검증하고, HWPX 출처 문서의 복구본이
원본 HWPX를 덮어쓰는 것으로 오해되지 않도록 정책을 정리한다.

## 2. 구현 범위

- HWPX 출처 draft의 복구본 파일명을 `.hwp`로 생성
- 복구 후보 표시에서 HWPX 출처 문서가 HWP 복구본으로 열리는 점을 명시
- 실제 샘플 기반 E2E 추가
  - `samples/셀보호2.hwp`
  - `samples/셀보호2.hwpx` 출처 정책
- 복구 성공 후 dirty 상태와 draft 삭제 확인

## 3. 제외 범위

- HWPX 원본을 HWPX로 자동 복구 저장
- 복구 후보 관리 전용 화면
- 자동 저장 주기 사용자 설정 UI

## 4. 진행 기록

- 2026-06-21: Stage 3 착수.
- 2026-06-21: HWPX 출처 draft는 `exportHwp()` 기반 바이트이므로 복구본 파일명을 항상 `.hwp`로 생성하도록 정리.
- 2026-06-21: 복구 후보 메타 정보에 HWPX 출처 문서는 `HWPX → HWP 복구본`으로 표시되도록 보강.
- 2026-06-21: `셀보호2.hwp`, `셀보호2.hwpx`를 사용하는 실제 복구 E2E를 추가.

## 5. 검증

- `npm test`
  - 84개 통과
- `npx tsc --noEmit`
  - 통과
- `npm run build`
  - 통과
- `CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node e2e/autosave-recovery.test.mjs --mode=headless`
  - HWP draft 복구 통과
  - HWPX 출처 draft가 `.hwp` 복구본으로 열림 확인
  - 복구 후 dirty 상태 유지와 원본 draft 삭제 확인

## 6. 남은 판단

- Stage 1~3 범위 기준 자동 백업/복구 기본 흐름은 구현 및 검증 완료.
- 향후 별도 이슈 후보
  - 복구 후보 관리 전용 화면
  - 자동 저장 주기 사용자 설정
  - HWPX 원본을 HWPX 포맷 그대로 재저장하는 별도 복구 경로
