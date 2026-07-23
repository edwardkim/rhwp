# Task M100 #3126 Stage 4 — 인쇄 미리보기와 PDF 안내 흐름 분리

- 작성일: 2026-07-23
- 상태: **구현 및 Chrome 자동 검증 완료, 사용자 수동 확인 대기**
- 선행 기록: `task_m100_3126_stage3.md`

## 1. 재검토 배경

Stage 3의 1차 UX는 `인쇄…`과 `PDF로 저장…`이 같은 hidden iframe `print()` 경로를 사용하고,
PDF 명령에만 우상단 안내 토스트를 추가했다. 실제 Chrome/Firefox 계열 수동 검증에서 다음 문제가
확인됐다.

1. 두 메뉴의 마지막 동작이 같아 기존 `인쇄…`이 제공하던 별도 인쇄 미리보기 기능이 사라졌다.
2. 웹 페이지의 안내 토스트는 native print UI보다 위에 표시할 수 없어 일부만 보이거나 가려졌다.
3. 웹 API는 인쇄 대상을 PDF로 강제할 수 있으므로 `PDF로 저장…`이라는 이름과 실제 브라우저 단계
   사이를 실행 전에 설명할 필요가 있다.

#2657 메인테이너 결정의 핵심은 직접 PDF bytes 생성을 v1.0.0에서 보류하고, 폰트·메모리·검색
가능한 텍스트 생성을 브라우저 인쇄 경로에 위임하는 것이다. 기존 미리보기 UX를 제거하거나 인쇄와
PDF 진입점을 같은 화면 흐름으로 만들라는 조건은 아니다.

## 2. 채택한 UX

### `인쇄…`

- 사용자 클릭의 동기 구간에서 same-origin `print.html` 창을 먼저 연다.
- 전용 창에서 `인쇄 준비 중… (N/M)` 상태를 표시한다.
- 준비가 끝나면 `인쇄`, `닫기`, 파일명과 쪽 수, 전체 페이지 미리보기를 제공한다.
- 사용자가 미리보기의 `인쇄`를 눌렀을 때만 native print UI를 연다.

기존 별도 창의 미리보기 의미는 유지하되 `about:blank` 구현은 복원하지 않았다. 이로써 popup
차단 위험을 줄이고 origin·초기 loading document·URL을 명시적으로 관리한다.

### `PDF로 저장…`

- 클릭하면 `PDF로 저장` 안내 모달을 연다.
- 모달에서 브라우저 인쇄 기능을 사용하는 이유와
  `대상 또는 프린터 → PDF로 저장` 단계를 설명한다.
- `인쇄 창 열기` 확인 뒤 같은 모달을 `PDF 준비 중… (N/M)` 진행 화면으로 전환한다.
- SVG·폰트·레이아웃이 준비되면 모달을 제거하고 hidden same-origin iframe의 `print()`를 호출한다.
- native print UI 위에 표시할 수 없는 runtime 안내 토스트와 인위적인 최소 노출 지연은 제거한다.

## 3. 공유 계약과 분리 경계

두 경로는 다음을 공유한다.

- `renderPageSvgWithProfile(page, 'print')`
- `createPrintPage`와 페이지별 named `@page`
- SVG id namespace
- font readiness와 2회 animation frame layout 확정
- file handle·파일명·dirty/save 상태 불변

마지막 surface만 분리한다.

| 진입점 | surface | native print 호출 |
|---|---|---|
| `인쇄…` | visible same-origin window | 미리보기의 `인쇄` 버튼 |
| `PDF로 저장…` | hidden same-origin iframe | 안내 모달 확인 및 준비 완료 뒤 자동 |

## 4. 자동 검증

- 전체 Studio/editor unit: 515개 통과
- focused print unit/contract: 14개 통과
- production build: 통과
- Chrome headless E2E:
  - #2524 embedded font/SVG text/PDF text 추출
  - #2525 7쪽/named `@page`/PDF text 추출
  - PDF 안내 모달 → 진행 상태 → 모달 제거 → `print()` 1회
  - same-origin 인쇄 미리보기 7쪽, 인쇄/닫기 버튼
  - 두 경로 모두 file handle·파일명·dirty 상태 불변
- 시각 QA:
  - `output/e2e/issue-3126/pdf-guidance-modal.png`
  - `output/e2e/issue-3126/print-preview.png`

## 5. 잔여 수동 확인

1. Chrome/Firefox에서 `PDF로 저장…` 모달 문구와 버튼 크기 확인
2. 확인 후 진행 상태가 보이고 모달이 사라진 다음 native print UI가 열리는지 확인
3. native print UI 뒤에 rhwp 안내 토스트가 남지 않는지 확인
4. `인쇄…` 클릭 시 새 same-origin 미리보기 창이 열리는지 확인
5. 미리보기의 `인쇄`와 `닫기`가 각각 native print UI 호출과 창 종료를 수행하는지 확인
6. Windows Edge에서 popup, iframe print, 혼합 용지 named `@page` 호환성 확인
