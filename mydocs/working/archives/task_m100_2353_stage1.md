# #2353 1단계 완료보고 — e2e 스크립트 전수 인벤토리 (76개)

- 계획서: `mydocs/plans/task_m100_2353.md` / 브랜치: `local/task2353`
- 방법: 76개 파일 전수 내용 추출(헤더 주석·runTest 제목·setTestCase·
  loadHwpFile 샘플·assert 수) + 애매 건 직접 정독. 파일명 추정 없음.

## 분류 집계

| 분류 | 수 | 정의 |
|------|---:|------|
| 상시(배선) | 12 | npm script 또는 CI 워크플로에 배선된 회귀 게이트 |
| 상시(수동) | 47 | 배선 없이 수동 실행하는 회귀 테스트 (이슈/기능 회귀 포함) |
| 일회성/진단 | 10 | 프로브·수동 디버그·보류 가설 확정·임시 검증 (폐기/보존 판단 대상) |
| 유틸 | 7 | 헬퍼·리포터·러너·생성기 |

## 검증 위생 발견 (본 타스크의 필요성 실증)

인벤토리 작성 중, 메인테이너 자신의 최근 PR 검증에서
**존재하지 않는 파일명**(`undo-object-selection-clear.test.mjs` — 단위 테스트
파일명과 혼동)을 e2e 로 실행해 node 오류가 "FAIL 0건"으로 집계된 **공허
통과**가 있었음을 발견. 실파일(`undo-object-selection.test.mjs`)로 정정
실행하여 현 devel 에서 전 단언 통과 확인(공백 메움). — 명명·manifest 체계가
있었으면 즉시 잡혔을 결함으로, 2단계 설계에서 "실행 대상 존재 검증"을 대조
검사 요건에 포함한다.

## 관찰 (2단계 설계 입력)

- assert() 0 인 상시 분류 파일 존재 — edit-pipeline(자체 리포터 사용),
  responsive/kps-ai(로그 판정형), tac-verify 등: "자동 판정 없음" 방치인지
  자체 리포터 사용인지 manifest 필드로 구분 필요
- `kps-ai-host` 는 `kps-ai` 의 host 모드 변형 — 헬퍼 mode 옵션으로 흡수
  가능한 중복 후보
- 이슈형 명명 혼재: `issue-1280-*` vs `textbox-insert-floating-1280v2` vs
  `table-picture-resize-1282` — 같은 이슈 회귀인데 접두 유무 불일치
- `.check.mjs` 2건(hml-open, task1315-load)의 의미(“standalone/임시”)가
  파일명 관례로 정의된 적 없음

## 전수 목록


### 상시(배선) (12개)

| 파일 | 용도 | 샘플 | assert | 비고 |
|------|------|------|-------:|------|
| `canvas-render-diff.test.mjs` | / Browser canvas visual diff between the legacy PageRenderTree path and  | 빈 문서 | 1 | npm+CI |
| `canvaskit-font-coverage.test.mjs` |  | 빈 문서 | 0 | npm+CI · assert() 0 — 자체 리포터/로그형 |
| `drag-selection-autoscroll.test.mjs` | / E2E 테스트: 텍스트 드래그 선택 edge 자동 스크롤 / | 빈 문서 | 6 | npm e2e:drag-autoscroll |
| `embed-transport.test.mjs` | Issue #2186 @rhwp/editor MessageChannel v1 iframe transport | 빈 문서 | 10 | npm e2e:embed |
| `issue-2214-page-local-repaint.test.mjs` | / Issue #2214 focused GREEN regression and optional diagnostic. / | 빈 문서 | 0 | npm+CI · assert() 0 — 자체 리포터/로그형 |
| `pdf-render-diff-report.mjs` | / Report-only visual diff between browser Canvas output and SVG-derived  | 빈 문서 | 0 | npm+CI · assert() 0 — 자체 리포터/로그형 |
| `renderer-contract.test.mjs` |  | 빈 문서 | 5 | npm+CI |
| `task-871-clipboard-priority.test.mjs` | / E2E 테스트: 외부 클립보드가 rhwp-studio 내부 클립보드보다 우선되어야 함 (Task 871) / | 빈 문서 | 7 | npm e2e:clipboard-priority |
| `text-flow.test.mjs` | / E2E 테스트: 텍스트 플로우 (입력, 줄바꿈, 엔터, 페이지 넘김) / | 빈 문서 | 5 | npm e2e |
| `undo-contracts.test.mjs` | / E2E 테스트: 편집 undo 계약 실동작 검증 (Task #2301) / | 빈 문서 | 24 | npm e2e:undo |
| `undo-object-selection.test.mjs` | E2E: undo/redo 후 개체/표 선택 stale ref 해제 (Task #2303) /  / 계약: undo/redo 는  | 빈 문서 | 14 | npm e2e:undo-object-selection |
| `unsaved-changes-guard.test.mjs` | / E2E 테스트: #886 저장되지 않은 변경사항 보호 모달 / | 빈 문서 | 6 | npm e2e:unsaved-guard |

### 상시(수동) (47개)

| 파일 | 용도 | 샘플 | assert | 비고 |
|------|------|------|-------:|------|
| `autosave-recovery.test.mjs` | / Task #1448 — 미저장 문서 자동 백업 복구 E2E / | 빈 문서 | 16 |  |
| `blogform.test.mjs` | / E2E 테스트: BlogForm_BookReview.hwp 누름틀 안내문 / | BlogForm_BookReview.hwp | 0 | assert() 0 — 자체 리포터/로그형 |
| `command-palette.test.mjs` | / E2E 테스트 — /커맨드 팔레트 / | 빈 문서 | 10 |  |
| `copy-paste.test.mjs` | / E2E 테스트: 텍스트 블럭 복사/붙여넣기 버그 (Task 227) / | 빈 문서 | 6 |  |
| `dialog-theme.test.mjs` | / E2E 테스트 — 다이얼로그 다크 테마 색상 정책 / | 빈 문서 | 43 |  |
| `drop-confirm.test.mjs` | / E2E 테스트: #1439 드래그&드롭 로컬 파일 로딩 보안 게이트 / | 빈 문서 | 4 |  |
| `edit-pipeline.test.mjs` | / E2E 테스트: 편집 파이프라인 검증 (Issue #2) / | 빈 문서 | 0 |  |
| `export-hwpx.test.mjs` | / Issue #557 — npm/editor RPC + Wrapper 에 exportHwpx / exportHwpVerify 노 | 빈 문서 | 11 |  |
| `footnote-delete-confirm.test.mjs` | / E2E 테스트: #598 본문 각주 삭제 확인창/취소/Undo / | footnote-01.hwp | 25 |  |
| `footnote-insert.test.mjs` | / E2E 테스트: footnote-01.hwp 각주 삽입 시 문단 위치 이상 확인 / | footnote-01.hwp | 2 |  |
| `footnote-vpos.test.mjs` | / E2E 테스트: footnote-01.hwp "원료를" 뒤 스페이스 입력 시 문단 위치 이상 / WASM API 직접 호출로  | footnote-01.hwp | 1 |  |
| `form-control.test.mjs` | / E2E 테스트: 양식 컨트롤 — 셀 커서 진입(#111) + 체크박스 클릭 토글(#112) / | form-002.hwpx | 10 |  |
| `global-shortcut.test.mjs` | / E2E 테스트 — 전역 단축키 (문서 미로드 상태) / | 빈 문서 | 2 |  |
| `hml-equation-embed.test.mjs` | PR #2219 HML equation canvas edit/undo/export/reload | 빈 문서 | 10 |  |
| `hml-open.check.mjs` | / Standalone HML browser regression. / | 빈 문서 | 24 |  |
| `hwpctl-basic.test.mjs` | / hwpctl 호환 레이어 E2E 테스트 — 기본 동작 / | 빈 문서 | 8 |  |
| `hwpx-direct-save.test.mjs` | / HWPX 직접 저장 (file:save) E2E — #1532 / | 빈 문서 | 10 |  |
| `issue-1280-textbox-text-input.test.mjs` | / E2E 회귀: #1280 — rhwp-studio가 삽입한 글상자가 text_box 없는 Rectangle로 생성되어 / 커서 | 빈 문서 | 4 |  |
| `issue-1456-chart-rerender.test.mjs` | / E2E 회귀 — #1456: rhwp-studio 캔버스 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망 / | 빈 문서 | 5 |  |
| `issue-2069-ole-object-selection.test.mjs` | / E2E: 한셀 OLE 미리보기는 표처럼 보이더라도 셀 내부 편집으로 진입하지 않는다. / | 한셀OLE.hwp | 23 |  |
| `issue-2318-master-page-zorder.test.mjs` | Issue #2318: 바탕쪽 개체가 본문 텍스트를 가림 — studio 다층 canvas 합성 검증. /  / shortcut. | basic/shortcut.hwp | 4 |  |
| `issue-270-set-field-persist.test.mjs` | / E2E 테스트: 이슈 #270 — set_field 후 저장/재오픈 시 필드 값 유실 회귀 / | field-01.hwp | 11 |  |
| `kps-ai.test.mjs` | / E2E 테스트: kps-ai.hwp 파일 로드 및 분할 표 렌더링 검증 / | 빈 문서 | 1 |  |
| `line-spacing.test.mjs` | / E2E 테스트: 줄간격 변경에 따른 페이지 넘김 검증 / | 빈 문서 | 4 |  |
| `navigation-shortcuts.test.mjs` | / E2E 테스트: 플랫폼별 navigation shortcut / | 빈 문서 | 12 |  |
| `page-border-toggle.test.mjs` | / E2E 테스트 — 쪽 테두리/배경 미리보기 버튼 토글 / | 빈 문서 | 19 |  |
| `page-break.test.mjs` | / E2E 테스트: biz_plan.hwp 강제 쪽 나누기 / "5. 사업추진조직" 문단 앞에 쪽 나누기 삽입 후 페이지 재배치  | biz_plan.hwp | 5 |  |
| `page-setup-orientation-icon.test.mjs` | / E2E 테스트: 편집 용지 대화창의 용지 방향 아이콘 식별성 / | 빈 문서 | 7 |  |
| `responsive.test.mjs` | / E2E 테스트: 반응형 레이아웃 검증 / | 빈 문서 | 0 |  |
| `save-as-format.test.mjs` | / 저장 출력 포맷 선택 (file:save-as-hwp / file:save-as-hwpx) E2E — #1613 / | biz_plan.hwp, hwpx/footnote-01.hwpx | 14 |  |
| `shape-inline.test.mjs` | / E2E 테스트: 도형 인라인 컨트롤 — 커서 이동 및 텍스트 삽입 / | 빈 문서 | 0 | assert() 0 — 자체 리포터/로그형 |
| `shift-end.test.mjs` | / E2E 테스트: shift-return.hwp Shift+End 블록 선택 / | shift-return.hwp | 2 |  |
| `table-picture-resize-1282.test.mjs` | / E2E 테스트 (Issue #1282): 회전된 표 셀 내부 picture 리사이즈. / | ta-pic-001-r-쪽영역안제한.hwp, ta-pic-001-r-쪽영 | 47 |  |
| `tac-inline-create.test.mjs` | / E2E 테스트: 빈 문서에서 인라인 TAC 표 직접 생성 (Issue #32) / | 빈 문서 | 5 |  |
| `tac-inline-table.test.mjs` | / E2E 테스트: 인라인 TAC 표 배치 검증 (Issue #31) / | tac-case-001.hwp | 7 |  |
| `tac-verify.test.mjs` | / E2E 자동 검증: 인라인 TAC 표 조판 (Issue #33) / | 빈 문서 | 0 | assert() 0 — 자체 리포터/로그형 |
| `textbox-insert-floating-1280v2.test.mjs` | / E2E 테스트 (Issue #1280 v2): 삽입 글상자 = floating + 글앞으로(InFrontOfText) / | 빈 문서 | 4 |  |
| `textbox-picture-1171.test.mjs` | / E2E 테스트 (Issue #1171): 사각형 글상자(Shape text_box) 안 picture / | tac-img-02.hwp | 7 |  |
| `textbox-picture-insert-1171.test.mjs` | / E2E 테스트 (Issue #1171 v2): 사각형 글상자 위에 이미지 드롭 → 본문(body) sibling 삽입 / | tac-img-02.hwp | 3 |  |
| `textbox-picture-ops-1273.test.mjs` | / E2E 테스트 (Issue #1273): 사각형 글상자(Shape text_box) 안 picture 의 / 마우스 드래그 조 | tac-img-02.hwp | 15 |  |
| `theme-auto-dark.test.mjs` | / E2E 테스트 — Chrome Auto Dark Mode 대응 / | 빈 문서 | 13 |  |
| `theme-bootstrap.test.mjs` | / E2E 테스트 — 초기 테마 bootstrap / | 빈 문서 | 6 |  |
| `theme-mode.test.mjs` | / E2E 테스트 — 보기 > 테마 / | 빈 문서 | 28 |  |
| `topmost-hittest.test.mjs` | / E2E 테스트 (Issue #1280 v2): 겹침 클릭 = "최상단 개체" 선택 / | textbox-under-image.hwp | 10 |  |
| `topmost-lifecycle.test.mjs` | / E2E 테스트 (Issue #1280 v2): 겹침 최상단 선택 → 연산 lifecycle / | textbox-under-image.hwp | 5 |  |
| `typesetting.test.mjs` | / E2E 테스트: 조판 품질 검증 (문단부호 표시 상태) / | 빈 문서 | 1 |  |
| `unsupported-format-error.test.mjs` | / E2E 테스트: 미지원 문서 오류 알림 후 정상 문서 재로드 / | field-01.hwp | 5 |  |

### 일회성/진단 (10개)

| 파일 | 용도 | 샘플 | assert | 비고 |
|------|------|------|-------:|------|
| `body-outside-click-fallback.test.mjs` | / 보류 ② 본문 외곽 클릭 fallback 결함 — 가설 (b) master page 글상자 hit 확정 e2e / | hwpctl_Action_Table__v1.1.hwp | 0 | 보류② 가설 확정용 (일회성 진단) |
| `debug-pagination.test.mjs` | / E2E 디버그: 50줄 입력 후 페이지네이션 확인 / | 빈 문서 | 0 | 수동 디버그 |
| `debug-table-pos.test.mjs` | / E2E 디버그: 표 삽입 후 텍스트 위치 확인 / | 빈 문서 | 0 | 수동 디버그 |
| `debug-textbox.test.mjs` | / E2E 디버그: 글상자 삽입 후 텍스트 위치 확인 / | 빈 문서 | 0 | 수동 디버그 |
| `grid-mode-click-coord.test.mjs` | / 보류 ① 그리드 좌표 결함 — 정량 e2e 측정 / | exam_kor.hwp | 3 | 보류① 정량 측정 (일회성 진단) |
| `issue-2021-probe.mjs` | Issue #2021 계측 프로브 — 대형 표 셀 입력 1회의 wasm 호출별 시간 분해. / 실행: CHROME_CDP=http | issue1949_giant_cell_nested_tables_perf. | 2 | #2021 계측 프로브 (해결 완료 이슈) |
| `issue-595.test.mjs` | / Issue #595 진단 e2e / | exam_math.hwp | 0 | #595 진단 e2e (assertion 0) |
| `kps-ai-host.test.mjs` | / E2E 테스트: kps-ai.hwp — 호스트 Windows Chrome CDP 연결 / | 빈 문서 | 1 | kps-ai 의 host 모드 변형 (중복 후보) |
| `pr2260-vscode-zoom-menu.test.mjs` | / [PR #2260 검증] rhwp-vscode 배율 메뉴 — 호스트 Chrome CDP 로 webview 하네스 구동. / | 빈 문서 | 11 | PR #2260 검증 하네스 (vscode webview — 재사용 가치로 존치 기록) |
| `task1315-load.check.mjs` | / Task #1315 4단계 — roundtrip 산출 HWPX의 rhwp-studio 로드 확인 (임시 검증 스크립트) / | 빈 문서 | 0 | 임시 검증 스크립트 (헤더에 명시) |

### 유틸 (7개)

| 파일 | 용도 | 샘플 | assert | 비고 |
|------|------|------|-------:|------|
| `gen-screenshot.mjs` |  | basic/KTX.hwp | 0 | README 스크린샷 생성 |
| `helpers.mjs` | / E2E 테스트 헬퍼 — Puppeteer + Chrome CDP / | 빈 문서 | 1 | 공통 헬퍼(러너/브라우저/로드/단언/보고서) |
| `renderer-baseline-native-diff.mjs` |  | 빈 문서 | 0 | baseline native 대조 |
| `renderer-baseline.mjs` |  | 빈 문서 | 0 | 렌더러 baseline 스윕 러너 |
| `report-generator.mjs` | / E2E 테스트 HTML 보고서 생성기 / | 빈 문서 | 0 | HTML 보고서 생성기 |
| `run-render-diff.mjs` |  | 빈 문서 | 0 | render-diff CI 러너 |
| `scenario-runner.mjs` | / 시나리오 실행기 + 렌더 트리 측정기 + 규칙 검증기 / | 빈 문서 | 0 | 시나리오 실행기+렌더 측정 |