# rhwp-studio e2e MANIFEST

e2e 스크립트의 **단일 권위 목록**이다. 파일 추가/변경/폐기 시 이 표를 함께
갱신한다 — `python3 scripts/check_e2e_manifest.py` 가 파일 목록(git tracked)과
이 표를 양방향 대조하고, 배선(npm script/CI)이 가리키는 파일의 실재를 검증한다.

- 분류: `상시`(회귀 게이트) / `진단`(프로브·디버그·보류 진단) / `유틸`(헬퍼·러너)
- 상태: `active` / `hold`(보류 이슈 종속) / `deprecated`(폐기 예고 — 사유 병기)
- 명명 규칙(신규): 상시 `<도메인>[-issue<N>].test.mjs` · 진단 `probe-*`/`debug-*`(비-test) ·
  유틸 무접미. 기존 비준수 파일은 비고 `legacy-name` 으로 면제 (개명하지 않음).
- 폐기 = 파일 삭제 + 행 제거 (git history 가 archive). 로컬 전용(gitignore 예:
  `kps-ai*.test.mjs`)은 이 목록 밖이다.
- 실행 방법은 [e2e-cdp.md](../../mydocs/manual/e2e-cdp.md) 참조.

| 파일 | 분류 | 상태 | 용도 | 샘플 | 배선 | 비고 |
|------|------|------|------|------|------|------|
| `autosave-recovery.test.mjs` | 상시 | active | Task #1448 — 미저장 문서 자동 백업 복구 E2E | — | 수동 |  |
| `blogform.test.mjs` | 상시 | active | BlogForm_BookReview.hwp 누름틀 안내문 | BlogForm_BookReview.hwp | 수동 |  |
| `body-outside-click-fallback.test.mjs` | 진단 | hold | 보류 ② 본문 외곽 클릭 fallback 결함 — 가설 (b) master page 글상자 hit 확정 e2e | hwpctl_Action_Table__v1.1.hwp | 수동 | legacy-name · 보류② 이슈 종속 |
| `canvas-render-diff.test.mjs` | 상시 | active | Browser canvas visual diff between the legacy PageRenderTree path  | — | npm+CI |  |
| `canvaskit-font-coverage.test.mjs` | 상시 | active | CanvasKit 번들 폰트 등록/커버리지 검증 (NotoSansKR) | — | npm+CI |  |
| `command-palette.test.mjs` | 상시 | active | /커맨드 팔레트 | — | 수동 |  |
| `copy-paste.test.mjs` | 상시 | active | 텍스트 블럭 복사/붙여넣기 버그 (Task 227) | — | 수동 |  |
| `debug-pagination.mjs` | 진단 | active | E2E 디버그: 50줄 입력 후 페이지네이션 확인 | — | 수동 | 수동 디버그 |
| `debug-table-pos.mjs` | 진단 | active | E2E 디버그: 표 삽입 후 텍스트 위치 확인 | — | 수동 | 수동 디버그 |
| `debug-textbox.mjs` | 진단 | active | E2E 디버그: 글상자 삽입 후 텍스트 위치 확인 | — | 수동 | 수동 디버그 |
| `dialog-theme.test.mjs` | 상시 | active | 다이얼로그 다크 테마 색상 정책 | — | 수동 |  |
| `drag-selection-autoscroll.test.mjs` | 상시 | active | 텍스트 드래그 선택 edge 자동 스크롤 | — | npm e2e:drag-autoscroll |  |
| `drop-confirm.test.mjs` | 상시 | active | #1439 드래그&드롭 로컬 파일 로딩 보안 게이트 | — | 수동 |  |
| `edit-pipeline.test.mjs` | 상시 | active | 편집 파이프라인 검증 (Issue #2) | — | 수동 |  |
| `embed-save-ack.test.mjs` | 상시 | active | Task #2660 호스트 저장 완료 통지와 dirty/autosave 정리 계약 | footnote-01.hwp | 수동 |  |
| `embed-transport.test.mjs` | 상시 | active | Issue #2186 @rhwp/editor MessageChannel v1 iframe transport | — | npm e2e:embed |  |
| `export-hwpx.test.mjs` | 상시 | active | Issue #557 — npm/editor RPC + Wrapper 에 exportHwpx / exportHwpVeri | — | 수동 |  |
| `footnote-delete-confirm.test.mjs` | 상시 | active | #598 본문 각주 삭제 확인창/취소/Undo | footnote-01.hwp | 수동 |  |
| `footnote-insert.test.mjs` | 상시 | active | footnote-01.hwp 각주 삽입 시 문단 위치 이상 확인 | footnote-01.hwp | 수동 |  |
| `footnote-vpos.test.mjs` | 상시 | active | footnote-01.hwp "원료를" 뒤 스페이스 입력 시 문단 위치 이상 / WASM API 직접 호출로 정확한 재 | footnote-01.hwp | 수동 |  |
| `form-control.test.mjs` | 상시 | active | 양식 컨트롤 — 셀 커서 진입(#111) + 체크박스 클릭 토글(#112) | form-002.hwpx | 수동 |  |
| `gen-screenshot.mjs` | 유틸 | active | README 용 렌더 스크린샷 생성기 | basic/KTX.hwp | 수동 |  |
| `global-shortcut.test.mjs` | 상시 | active | 전역 단축키 (문서 미로드 상태) | — | 수동 |  |
| `grid-mode-click-coord.test.mjs` | 진단 | hold | 보류 ① 그리드 좌표 결함 — 정량 e2e 측정 | exam_kor.hwp | 수동 | legacy-name · 보류① 이슈 종속 |
| `helpers.mjs` | 유틸 | active | E2E 테스트 헬퍼 — Puppeteer + Chrome CDP | — | 수동 |  |
| `hml-equation-embed.test.mjs` | 상시 | active | PR #2219 HML equation canvas edit/undo/export/reload | — | 수동 |  |
| `hml-open.check.mjs` | 상시 | active | Standalone HML browser regression. | — | 수동 | legacy-name |
| `hwpctl-basic.test.mjs` | 상시 | active | hwpctl 호환 레이어 기본 동작 | — | 수동 |  |
| `hwpx-direct-save.test.mjs` | 상시 | active | HWPX 직접 저장 (file:save) E2E — #1532 | — | 수동 |  |
| `issue-1280-textbox-text-input.test.mjs` | 상시 | active | E2E 회귀: #1280 — rhwp-studio가 삽입한 글상자가 text_box 없는 Rectangle로 생성되어  | — | 수동 |  |
| `issue-1456-chart-rerender.test.mjs` | 상시 | active | E2E 회귀 — #1456: rhwp-studio 캔버스 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망 | — | 수동 |  |
| `issue-2069-ole-object-selection.test.mjs` | 상시 | active | E2E: 한셀 OLE 미리보기는 표처럼 보이더라도 셀 내부 편집으로 진입하지 않는다. | 한셀OLE.hwp | 수동 |  |
| `issue-2214-page-local-repaint.test.mjs` | 상시 | active | Issue #2214 focused GREEN regression and optional diagnostic. | — | npm+CI |  |
| `issue-2318-master-page-zorder.test.mjs` | 상시 | active | Issue #2318: 바탕쪽 개체가 본문 텍스트를 가림 — studio 다층 canvas 합성 검증. /  / sho | basic/shortcut.hwp | 수동 |  |
| `issue-2635-rawsvg-first-paint.test.mjs` | 상시 | active | Issue #2635: 순수 RawSvg 차트가 첫 화면에 늦게 표시되는 회귀 | chart/원형/쪼개진원형.hwp | 수동 |  |
| `issue-270-set-field-persist.test.mjs` | 상시 | active | 이슈 #270 — set_field 후 저장/재오픈 시 필드 값 유실 회귀 | field-01.hwp | 수동 |  |
| `issue-2809-split-alignment.test.mjs` | 상시 | active | Issue #2809 위·아래 Split 문단 속성 및 WASM/editor 정렬 회귀 | issues/2809/jubo_20260104.hwp | npm e2e:issue-2809 |  |
| `issue-595.test.mjs` | 진단 | hold | Issue #595 진단 e2e | exam_math.hwp | 수동 | legacy-name · #595 진단 (assertion 0) |
| `line-spacing.test.mjs` | 상시 | active | 줄간격 변경에 따른 페이지 넘김 검증 | — | 수동 |  |
| `navigation-shortcuts.test.mjs` | 상시 | active | 플랫폼별 navigation shortcut | — | 수동 |  |
| `page-border-toggle.test.mjs` | 상시 | active | 쪽 테두리/배경 미리보기 버튼 토글 | — | 수동 |  |
| `page-break.test.mjs` | 상시 | active | biz_plan.hwp 강제 쪽 나누기 / "5. 사업추진조직" 문단 앞에 쪽 나누기 삽입 후 페이지 재배치 확인 | biz_plan.hwp | 수동 |  |
| `page-setup-orientation-icon.test.mjs` | 상시 | active | 편집 용지 대화창의 용지 방향 아이콘 식별성 | — | 수동 |  |
| `pdf-render-diff-report.mjs` | 상시 | active | Report-only visual diff between browser Canvas output and SVG-deri | — | npm+CI | legacy-name |
| `print-pdf-issue3126.test.mjs` | 상시 | active | #3126 same-origin iframe 인쇄/PDF UX, 상태 불변, #2524/#2525 browser PDF 회귀 | render-p35-font-native-bitmap.hwpx, hwpx/hwpx-02.hwpx | 수동 | native dialog는 Chrome/Edge 수동 절차 병행 |
| `pr2260-vscode-zoom-menu.test.mjs` | 상시 | active | [PR #2260 검증] rhwp-vscode 배율 메뉴 — 호스트 Chrome CDP 로 webview 하네스 구동. | — | 수동 |  |
| `renderer-baseline-native-diff.mjs` | 유틸 | active | 렌더러 baseline — studio vs native 산출 대조 | — | CI |  |
| `renderer-baseline.mjs` | 유틸 | active | 렌더러 baseline 스윕 러너 (manifest 기반 다문서 측정) | — | npm+CI |  |
| `renderer-contract.test.mjs` | 상시 | active | 렌더러 백엔드 계약 검증 (plane/replay 정합) | — | npm+CI |  |
| `report-generator.mjs` | 유틸 | active | E2E 테스트 HTML 보고서 생성기 | — | 수동 |  |
| `responsive.test.mjs` | 상시 | active | 반응형 레이아웃 검증 | — | 수동 |  |
| `run-render-diff.mjs` | 유틸 | active | render-diff CI 러너 (canvas/pdf diff 오케스트레이션) | — | npm+CI |  |
| `save-as-format.test.mjs` | 상시 | active | 저장 출력 포맷 선택 (file:save-as-hwp / file:save-as-hwpx) E2E — #1613 | biz_plan.hwp, hwpx/footnote-01.hwpx | 수동 |  |
| `scenario-runner.mjs` | 유틸 | active | 시나리오 실행기 + 렌더 트리 측정기 + 규칙 검증기 | — | 수동 |  |
| `shape-inline.test.mjs` | 상시 | active | 도형 인라인 컨트롤 — 커서 이동 및 텍스트 삽입 | — | 수동 |  |
| `shift-end.test.mjs` | 상시 | active | shift-return.hwp Shift+End 블록 선택 | shift-return.hwp | 수동 |  |
| `table-picture-resize-1282.test.mjs` | 상시 | active | E2E 테스트 (Issue #1282): 회전된 표 셀 내부 picture 리사이즈. | ta-pic-001-r-쪽영역안제한.hwp, ta-pic-001-r-쪽영역안제한 | 수동 |  |
| `tac-inline-create.test.mjs` | 상시 | active | 빈 문서에서 인라인 TAC 표 직접 생성 (Issue #32) | — | 수동 |  |
| `tac-inline-table.test.mjs` | 상시 | active | 인라인 TAC 표 배치 검증 (Issue #31) | tac-case-001.hwp | 수동 |  |
| `tac-verify.test.mjs` | 상시 | active | E2E 자동 검증: 인라인 TAC 표 조판 (Issue #33) | — | 수동 |  |
| `task-871-clipboard-priority.test.mjs` | 상시 | active | 외부 클립보드가 rhwp-studio 내부 클립보드보다 우선되어야 함 (Task 871) | — | npm e2e:clipboard-priority |  |
| `text-flow.test.mjs` | 상시 | active | 텍스트 플로우 (입력, 줄바꿈, 엔터, 페이지 넘김) | — | npm e2e |  |
| `textbox-insert-floating-1280v2.test.mjs` | 상시 | active | E2E 테스트 (Issue #1280 v2): 삽입 글상자 = floating + 글앞으로(InFrontOfText) | — | 수동 |  |
| `textbox-picture-1171.test.mjs` | 상시 | active | E2E 테스트 (Issue #1171): 사각형 글상자(Shape text_box) 안 picture | tac-img-02.hwp | 수동 |  |
| `textbox-picture-insert-1171.test.mjs` | 상시 | active | E2E 테스트 (Issue #1171 v2): 사각형 글상자 위에 이미지 드롭 → 본문(body) sibling 삽입 | tac-img-02.hwp | 수동 |  |
| `textbox-picture-ops-1273.test.mjs` | 상시 | active | E2E 테스트 (Issue #1273): 사각형 글상자(Shape text_box) 안 picture 의 / 마우스 드 | tac-img-02.hwp | 수동 |  |
| `theme-auto-dark.test.mjs` | 상시 | active | Chrome Auto Dark Mode 대응 | — | 수동 |  |
| `theme-bootstrap.test.mjs` | 상시 | active | 초기 테마 bootstrap | — | 수동 |  |
| `theme-mode.test.mjs` | 상시 | active | 보기 > 테마 | — | 수동 |  |
| `topmost-hittest.test.mjs` | 상시 | active | E2E 테스트 (Issue #1280 v2): 겹침 클릭 = "최상단 개체" 선택 | textbox-under-image.hwp | 수동 |  |
| `topmost-lifecycle.test.mjs` | 상시 | active | E2E 테스트 (Issue #1280 v2): 겹침 최상단 선택 → 연산 lifecycle | textbox-under-image.hwp | 수동 |  |
| `typesetting.test.mjs` | 상시 | active | 조판 품질 검증 (문단부호 표시 상태) | — | 수동 |  |
| `undo-contracts.test.mjs` | 상시 | active | 편집 undo 계약 실동작 검증 (Task #2301) | — | npm e2e:undo |  |
| `undo-object-selection.test.mjs` | 상시 | active | E2E: undo/redo 후 개체/표 선택 stale ref 해제 (Task #2303) /  / 계약: undo/r | — | npm e2e:undo-object-selection |  |
| `unsaved-changes-guard.test.mjs` | 상시 | active | #886 저장되지 않은 변경사항 보호 모달 | — | npm e2e:unsaved-guard |  |
| `unsupported-format-error.test.mjs` | 상시 | active | 미지원 문서 오류 알림 후 정상 문서 재로드 | field-01.hwp | 수동 |  |
