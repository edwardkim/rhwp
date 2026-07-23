# Task M100 #3126 Stage 4 — 인쇄 미리보기와 PDF 안내 흐름 분리

- 작성일: 2026-07-23
- 상태: **안내 생략 후속 구현 및 Chrome 자동 검증 완료, 사용자 수동 확인 대기**
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

## 6. PDF 메뉴 아이콘 후속 보정

사용자 수동 확인에서 인쇄 미리보기와 PDF 안내 흐름은 의도대로 동작함을 확인했다. 다만 파일 메뉴의
`PDF로 저장…`이 `인쇄`와 같은 프린터 아이콘을 사용해 두 진입점의 의미가 시각적으로 구분되지 않았다.

`public/images/icon_small_ko.svg`와 `icon_small_ko_dark.svg`를 확인한 결과, 첫 행 네 번째 칸
(`col=3`, `row=0`)에 한컴 파일 메뉴와 같은 문서 윤곽·빨간 `PDF` 표식 아이콘이 이미 포함되어 있었다.
따라서 새 자산을 만들거나 기존 스프라이트를 수정하지 않고 다음처럼 연결했다.

- `toolbar.css`에 `.icon-pdf` 의미 클래스를 추가한다.
- `PDF로 저장…`에는 `.icon-pdf`를 사용한다.
- `인쇄`에는 기존 `.icon-print`를 유지한다.
- 밝은/어두운 테마가 공유하는 스프라이트 좌표를 사용하므로 테마별 별도 자산은 추가하지 않는다.

계약 테스트는 PDF 메뉴의 클래스와 스프라이트 좌표를 함께 고정해 다시 프린터 아이콘으로 합쳐지는
회귀를 막는다.

후속 검증 결과:

- `print-command-contract.test.ts`: 6개 통과
- Studio/editor 전체 테스트: 515개 통과
- production build: 통과
- 7701 실제 어두운 테마 메뉴: PDF `-120px 0px`, 인쇄 `-160px 0px`로 분리 렌더 확인
- 밝은/어두운 스프라이트 모두 같은 `col=3`, `row=0` PDF 도형과 테마별 명암을 포함함을 원본에서 확인

## 7. 반복 안내 생략과 복원 경로

첫 안내를 이해한 사용자가 PDF 저장 때마다 같은 확인 단계를 반복하지 않도록 안내 모달에
`다음부터 이 안내를 표시하지 않기` 체크박스를 추가했다.

- 체크하지 않음: 종전처럼 저장 방법 안내와 `인쇄 창 열기` 확인을 거친다.
- 체크하고 실행: `rhwp-settings.dialog.showPdfPrintGuidance=false`를 저장한다.
- 이후 실행: 저장 방법 확인만 생략하고 즉시 `PDF 준비 중… (N/M)` 모달로 전환한다.
- 준비 진행률, 오류 표시, native 인쇄창 전 모달 제거는 생략 여부와 관계없이 유지한다.
- 취소한 경우에는 체크 상태를 저장하지 않는다.
- 비필수 설정 저장이 실패해도 PDF 준비와 인쇄 호출은 계속 진행한다.

영구 설정의 복원 경로는 `도구 → 환경 설정 → 파일 → PDF 저장`의
`PDF로 저장할 때 저장 방법 안내 표시` 체크박스로 제공한다. 따라서 사용자가 안내를 다시 보고 싶을 때
저장소를 직접 지우지 않아도 된다.

후속 검증:

- focused 설정/인쇄 계약 테스트: 11개 통과
- Chrome headless E2E:
  - 첫 실행에서 안내 숨김 체크 및 `rhwp-settings` 저장
  - 두 번째 실행에서 확인 안내 생략
  - 두 실행 모두 준비 진행률 표시, 안내 영역 비표시, print 전 모달 제거
  - 기존 PDF 회귀 2종과 인쇄 미리보기 회귀 통과
- 시각 확인:
  - `output/e2e/issue-3126/pdf-guidance-modal.png`
  - `도구 → 환경 설정 → 파일`의 PDF 안내 복원 옵션
