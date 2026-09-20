# Issue #3513 수행·구현 계획

- Parent: #3512
- 기준: `upstream/devel@722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`
- 2026-09-20 사용자 승인: #3512 현행화 후 #3513 → #3515 진행, Chrome Phase 1 완료 후 증적 보고서 연동 후속 이슈 등록.

## 현황과 범위

#3514는 #5912로 완료됐다. #3512 본문을 현행화하고 #3514/#3513/#3515를 GitHub sub-issue로 연결했다.
현재 smoke는 UI hydration을, 다운로드 E2E는 ON 상태의 파일 분류·초기 저장 경합을 검증한다.
options에서 실제 저장한 OFF 값, worker 종료, 동일 profile 재시작, 과거 기록과 HWPX 경로는 추가 검증한다.

제품 권한·CSP·디버그 API를 변경하지 않는다. Puppeteer와 잠긴 Chrome for Testing을 사용한다.
각 시나리오에 독립 profile을 만들되 재시작 시나리오 내부에서는 같은 profile을 유지한다.
기준값은 issue의 사용자 관찰 계약인 options 표시 상태와 viewer 생성 0/1개다.
다운로드는 loopback 서버의 기존 tracked HWP/HWPX fixture로 발생시키고 완료 이벤트·저장 바이트를 확인한다.
중복 탭은 target 생성 시 즉시 실패시키며, 관찰 구간 내 생성 이력도 확인한다.
worker 종료 검증은 종료를 확인한 뒤 사용자 동작으로 재기동시키고, 내부 storage 조회는 실패 진단에만 사용한다.

## 단계

1. 누락 시나리오·초기 상태·종료 조건 확정 및 이 계획 커밋.
2. lifecycle E2E와 탭 예산·진단 기반 구현, 관련 Node 계약·실제 브라우저 focused 검증.
3. freshness/중복 방어 mutation 점검, 새 profile에서 retry 없는 10회 반복, smoke/download 기존 대조군.
4. 실제 검증 SHA·환경·시간·검증 한계를 보고서와 개발 매뉴얼에 기록, 제출 가능한 변경 준비.
5. #3515에서 현행 trusted impact classifier, Build & Test 집계, 조건부 Chrome 설치·캐시·실패 artifact 연결.

GitHub Actions 3회 실행과 원격 통합은 로컬 검증과 구분한다. #3515는 현재 branch push CI가 없다는
#7070 정책을 유지하며, cache seed 경로와 browser-only 시간/빌드 준비 시간을 따로 기록한다.
후속 증적 갤러리는 이번 Epic 완료 조건에 포함하지 않는다.

## 검증

- JS 구문과 탭 예산의 즉시 실패/종료 후 listener 정리 계약.
- options OFF 재진입/worker 종료/browser 재시작과 다운로드 0/1개 실제 브라우저 검사.
- 과거 기록은 확장을 설치하기 전 실제 다운로드로 준비, 확장 시작과 ON 전환 뒤 생성 이력 0개.
- HWP/HWPX 각각 ON/OFF/worker wakeup, 완전한 다운로드 확인 후 bounded quiet window 관찰.
- mutation은 임시 dist에서만 적용하고 원본을 복원한다. 이벤트가 발생하지 않아 mutation이 살아남는
  경우를 숨기지 않고 기존 Node freshness 계약의 검출 결과와 분리한다.
- 로컬 10회는 retry 없이 실행하며, 코드 수정 후에는 처음부터 다시 검증한다.
