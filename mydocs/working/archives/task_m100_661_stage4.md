# Task M100 #661 Stage 4 완료 보고서

## 개요

- 이슈: #661 `rhwp-studio: 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상`
- 기준 브랜치: `local/task661`
- 단계 목표: 실제 브라우저 드래그 동작 검증 및 회귀 E2E 추가

## 변경 내용

### 1. 드래그 edge 자동 스크롤 전용 E2E 추가

- 파일: `rhwp-studio/e2e/drag-selection-autoscroll.test.mjs`
- 새 문서에 70줄 텍스트를 입력한다.
- 첫 문단 시작점에서 편집 영역 하단 edge까지 마우스 드래그 후 일정 시간 유지한다.
- 다음 조건을 검증한다.
  - `scrollTop`이 80px 이상 증가
  - 선택 상태 유지
  - `.selection-layer > div` 하이라이트 표시
  - 선택 focus가 하단 문단까지 확장

### 2. npm script 추가

- 파일: `rhwp-studio/package.json`
- `e2e:drag-autoscroll` script를 추가했다.

## Browser 실제 조작 검증

- URL: `http://127.0.0.1:7700/`
- 화면: `rhwp-studio`
- 흐름: 앱 로드 → Alt+N 새 문서 → 70줄 텍스트 입력 → 첫 줄에서 하단 edge로 드래그 → 스크롤 및 선택 하이라이트 확인
- 결과:
  - 앱은 빈 화면/프레임워크 오류 없이 렌더링됐다.
  - 하단 edge 드래그 시 스크롤바가 내려갔다.
  - 선택 하이라이트가 line 03부터 line 35 부근까지 이어졌다.
  - console error는 없었다.
  - 긴 입력 중 `CursorState`의 캐럿 페이지 불일치 fallback warning 1건이 관찰됐으나, 드래그 선택 검증을 막지는 않았다.

## 자동화 검증

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" npm run e2e:drag-autoscroll -- --mode=headless
```

- 결과: 성공
- 주요 결과:
  - `scrollTop`: `0 → 1529`
  - 선택 상태: `true`
  - 선택 focus: `paragraphIndex=69`
  - 선택 하이라이트: `70`

```bash
cd rhwp-studio
npm run build
```

- 결과: 성공
- 비고: Vite의 기존 chunk size 경고는 발생했지만 빌드 실패는 없었다.

```bash
git diff --check
```

- 결과: 성공

## 보정 사항

- 첫 E2E 실행에서는 선택 하이라이트 selector를 `.selection-highlight`로 잘못 잡아 실패했다.
- 실제 렌더러는 `.selection-layer` 아래 div를 풀링하므로, 표시 중인 `.selection-layer > div` 개수를 세도록 테스트를 정정했다.
- 제품 코드 추가 보정은 필요하지 않았다.

## 남은 작업

1. Stage 5에서 최종 회귀 검증 명령을 한 번 더 묶어 실행한다.
2. 최종 보고서와 오늘할일 완료 상태를 정리한다.
3. 작업 브랜치 상태를 확인하고 승인 후 다음 통합 절차로 넘긴다.

## 승인 요청

Stage 4 검증 결과와 전용 E2E 추가를 승인해 주시면 Stage 5 최종 검증 및 보고서 작성으로 진행한다.
