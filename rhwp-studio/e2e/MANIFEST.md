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
| `automation-commands.test.mjs` | 상시 | active | studio 자동화 표면 — 커맨드 질의·실행·메뉴 모델·드리프트 가드·다이얼로그 정책 | — | npm e2e:automation |  |
| `autosave-recovery.test.mjs` | 상시 | active | Task #1448 — 미저장 문서 자동 백업 복구 E2E | — | 수동 |  |
| `blogform.test.mjs` | 상시 | active | BlogForm_BookReview.hwp 누름틀 안내문 | BlogForm_BookReview.hwp | 수동 |  |
| `body-outside-click-fallback.test.mjs` | 진단 | hold | 보류 ② 본문 외곽 클릭 fallback 결함 — 가설 (b) master page 글상자 hit 확정 e2e | hwpctl_Action_Table__v1.1.hwp | 수동 | legacy-name · 보류② 이슈 종속 |
| `bridge-lifecycle.test.mjs` | 상시 | active | Studio Bridge — 부모 페이지에서 createStudio 로 chrome·커맨드·hwpctrl 제어와 destroy 회수 | table-001.hwp | npm e2e:bridge |  |
| `bridge-perf.test.mjs` | 상시 | active | 브리지 성능 게이트 — 배치 RPC 1왕복·배치 이득·undo 1스텝·수명 잔여 (계획서 §6) | table-001.hwp | npm e2e:bridge-perf |  |
| `canvas-render-diff.test.mjs` | 상시 | active | Browser canvas visual diff between the legacy PageRenderTree path  | — | npm+CI |  |
| `canvaskit-font-coverage.test.mjs` | 상시 | active | CanvasKit 번들 폰트와 exact TTC GlyphRun 등록/픽셀 replay 검증 | — | npm+CI |  |
| `chart-data-structure-issue6053.test.mjs` | 상시 | active | #6053 차트 행·열·라벨 구조 편집 UI — 우클릭 행 추가·저장본 반영·Ctrl+Z 원복·무편집 무흔적·ESC 는 메뉴만 닫음·종류별 사전 판정(원형 안내/주식형 캔들 양끝 비활성) | chart/세로막대형/묶은세로막대형.hwp, chart/원형/2차원원형.hwp, chart/기타/시가고가저가종가.hwp | npm e2e:issue-6053 | dev server 필요 — run-with-vite.mjs 경유 |
| `cell-enter-pagination-issue4031.test.mjs` | 상시 | active | Issue #4031 — pending 중 셀 Enter의 pre-navigation full flush 0회·split 1회·barrier 대조군 계약 | issue1949_giant_cell_nested_tables_perf.hwp/.hwpx | npm e2e:issue-4031-cell-enter |  |
| `command-palette.test.mjs` | 상시 | active | /커맨드 팔레트 | — | 수동 |  |
| `content-loss-save-issue4430.test.mjs` | 상시 | active | Issue #4430/#5986 — serializer 내용 손실 보고의 명시 저장·fallback·암호 저장, 저장 실패의 보호 상태 보존 및 취소 무알림 계약 | test-image.hwpx | npm e2e:issue-4430-content-loss | fresh WASM 필수 · synthetic 보고서 주입 없음 |
| `copy-paste.test.mjs` | 상시 | active | 텍스트 블럭 복사/붙여넣기 버그 (Task 227) | — | 수동 |  |
| `debug-pagination.mjs` | 진단 | active | E2E 디버그: 50줄 입력 후 페이지네이션 확인 | — | 수동 | 수동 디버그 |
| `debug-table-pos.mjs` | 진단 | active | E2E 디버그: 표 삽입 후 텍스트 위치 확인 | — | 수동 | 수동 디버그 |
| `debug-textbox.mjs` | 진단 | active | E2E 디버그: 글상자 삽입 후 텍스트 위치 확인 | — | 수동 | 수동 디버그 |
| `dialog-theme.test.mjs` | 상시 | active | 다이얼로그 다크 테마 색상 정책 | — | 수동 |  |
| `document-agent-command.test.mjs` | 상시 | active | HWP/HWPX exact command apply·strict render·revert·focus·native typing·일반 Ctrl+Z·modal 0회 | para-001.hwp, hwpx/para-001.hwpx | npm e2e:document-agent | fresh WASM 필수 |
| `drag-selection-autoscroll.test.mjs` | 상시 | active | 텍스트 드래그 선택 edge 자동 스크롤 | — | npm e2e:drag-autoscroll |  |
| `drop-confirm.test.mjs` | 상시 | active | 드롭 확인 대화상자 경계 (문서=없음, 이미지=#1439 게이트) | — | 수동 |  |
| `edit-pipeline.test.mjs` | 상시 | active | 편집 파이프라인 검증 (Issue #2) | — | 수동 |  |
| `embed-save-ack.test.mjs` | 상시 | active | Task #2660 호스트 저장 완료 통지와 dirty/autosave 정리 계약 | footnote-01.hwp | 수동 |  |
| `embed-transport.test.mjs` | 상시 | active | Issue #2186 @rhwp/editor MessageChannel v1 iframe transport | — | npm e2e:embed |  |
| `export-hwpx.test.mjs` | 상시 | active | Issue #557 — npm/editor RPC + Wrapper 에 exportHwpx / exportHwpVeri | — | 수동 |  |
| `footnote-delete-confirm.test.mjs` | 상시 | active | #598 본문 각주 삭제 확인창/취소/Undo | footnote-01.hwp | 수동 |  |
| `footnote-insert.test.mjs` | 상시 | active | footnote-01.hwp 각주 삽입 시 문단 위치 이상 확인 | footnote-01.hwp | 수동 |  |
| `footnote-vpos.test.mjs` | 상시 | active | footnote-01.hwp "원료를" 뒤 스페이스 입력 시 문단 위치 이상 / WASM API 직접 호출로 정확한 재 | footnote-01.hwp | 수동 |  |
| `form-control.test.mjs` | 상시 | active | 양식 컨트롤 — 셀 커서 진입(#111) + 체크박스 클릭 토글(#112) | form-002.hwpx | 수동 |  |
| `form-edit-escape-cancel.test.mjs` | 상시 | active | #2375 Edit 양식 필드 Escape는 blur 뒤에도 취소·무기록 | form-01.hwp | npm e2e:form-edit-escape |  |
| `gen-screenshot.mjs` | 유틸 | active | README 용 렌더 스크린샷 생성기 | basic/KTX.hwp | 수동 |  |
| `global-shortcut.test.mjs` | 상시 | active | 시작 시 빈 문서 + 전역 단축키 | — | 수동 |  |
| `grid-mode-click-coord.test.mjs` | 진단 | hold | 보류 ① 그리드 좌표 결함 — 정량 e2e 측정 | exam_kor.hwp | 수동 | legacy-name · 보류① 이슈 종속 |
| `header-footer-selection-issue4121.test.mjs` | 상시 | active | #4121 HF 선택 생성·반복 페이지 투영과 delete/type/paste/copy/cut/format history | biz_plan.hwp | npm e2e:issue-4121 | Stage 2~3 선택 생성·소비 계약 |
| `home-end-key.test.mjs` | 상시 | active | Home/End 줄 처음·끝, Ctrl+Home/End 문서 처음·끝 — 포커스 밖·머리말·각주 모드 포함 | biz_plan.hwp, footnote-01.hwp | npm e2e:home-end-key |  |
| `helpers.mjs` | 유틸 | active | E2E 테스트 헬퍼 — Puppeteer + Chrome CDP | — | 수동 |  |
| `hml-equation-embed.test.mjs` | 상시 | active | PR #2219 HML equation canvas edit/undo/export/reload | — | 수동 |  |
| `hml-open.check.mjs` | 상시 | active | Standalone HML browser regression. | — | 수동 | legacy-name |
| `hwpctl-basic.test.mjs` | 상시 | active | hwpctl 호환 레이어 기본 동작 | — | 수동 |  |
| `hwp-password-open.test.mjs` | 상시 | active | #3474/#3481/#5986 HWP3·HWP5·HWPX 암호 문서: 대화상자·오답·취소·Enter·저장소 비보존·저장 보호 상태 수명주기, HWP3 A4 144dpi Canvas 경계 | HWP3-password-123456.hwp, hwp3-sample16-hwp5-2024-password-123456.hwp, HWP5-password-123456.hwpx, HWP5-nopassword-123456.hwpx | npm e2e:hwp-password-open | 실제 암호 fixture; 비밀번호와 로컬 E2E 보고서는 비커밋 |
| `hwpctrl-plugin.test.mjs` | 상시 | active | hwpctrl 플러그인 — studio 문서 공유·배치 편집·좌표 변환·unload 생존 | — | npm e2e:hwpctrl-plugin |  |
| `hwpx-direct-save.test.mjs` | 상시 | active | HWPX 직접 저장 (file:save) E2E — #1532 | — | 수동 |  |
| `issue-1280-textbox-text-input.test.mjs` | 상시 | active | E2E 회귀: #1280 — rhwp-studio가 삽입한 글상자가 text_box 없는 Rectangle로 생성되어  | — | 수동 |  |
| `issue-1456-chart-rerender.test.mjs` | 상시 | active | E2E 회귀 — #1456: rhwp-studio 캔버스 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망 | — | 수동 |  |
| `issue-2069-ole-object-selection.test.mjs` | 상시 | active | E2E: 한셀 OLE 미리보기는 표처럼 보이더라도 셀 내부 편집으로 진입하지 않는다. | 한셀OLE.hwp | 수동 |  |
| `issue-2214-page-local-repaint.test.mjs` | 상시 | active | #2214 focused GREEN과 #3815 stack #3937·#3822 연속 IME·두 번 wrap 통합 회귀 | issue1949_giant_cell_nested_tables_perf.hwp, issue1949_giant_cell_nested_tables_perf.hwpx | npm e2e:issue-2214, npm e2e:issue-3815, CI node-check | production WASM·Chrome의 시간·쪽수·revision 결과는 로컬 증적 |
| `issue-2318-master-page-zorder.test.mjs` | 상시 | active | Issue #2318: 바탕쪽 개체가 본문 텍스트를 가림 — studio 다층 canvas 합성 검증. /  / sho | basic/shortcut.hwp | 수동 |  |
| `issue-2635-rawsvg-first-paint.test.mjs` | 상시 | active | Issue #2635: 순수 RawSvg 차트가 첫 화면에 늦게 표시되는 회귀 | chart/원형/쪼개진원형.hwp | 수동 |  |
| `issue-270-set-field-persist.test.mjs` | 상시 | active | 이슈 #270 — set_field 후 저장/재오픈 시 필드 값 유실 회귀 | field-01.hwp | 수동 |  |
| `issue-2809-split-alignment.test.mjs` | 상시 | active | Issue #2809 위·아래 Split 문단 속성 및 WASM/editor 정렬 회귀 | issues/2809/jubo_20260104.hwp | npm e2e:issue-2809 |  |
| `issue-3820-hwp-hwpx-page-count.test.mjs` | 상시 | active | Issue #3820 — 2025 행정업무운영 편람의 HWP/HWPX 물리 페이지 수와 주요 page owner를 383쪽 기준으로 고정 | 2025 행정업무운영 편람(최종).hwp/.hwpx | npm e2e:issue-3820 | fresh WASM 필수 |
| `issue-3682-chart-object-probe.test.mjs` | 진단 | active | #3682 차트 개체 P1~P5 행동 현황 수집 프로브 | chart/세로막대형/묶은세로막대형.hwp | 수동 | legacy-name · 최신 devel의 기존 미등재 항목 보완 |
| `issue-3953-large-document-goto.test.mjs` | 상시 | active | #3953 대형 HWP 후반부 찾아가기, 오류 재입력과 상태 표시줄 진입 | 정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp | 수동 | |
| `issue-4026-footnote-global-shortcuts.test.mjs` | 상시 | active | #4026 각주 편집 중 Cmd+Z 되돌리기와 Option+G 찾아가기 | footnote-01.hwp | 수동 | |
| `issue-4030-footnote-goto-transition.test.mjs` | 상시 | active | #4030 실제 대형 HWP 각주에서 Option+G 200쪽 이동 시 본문 모드·상태 표시·viewport 전환 | 정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp | 수동 | |
| `issue-4158-char-overlap-boxed-pua-canvas2d.test.mjs` | 상시 | active | #4158 실제 CharOverlap의 사각 안 숫자 PUA를 Canvas2D가 숫자와 사각형으로 합성 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4158 | 17쪽·물리 10쪽 raw IR 보존 계약 포함 |
| `issue-4159-terminal-nested-bottom-border-canvas2d.test.mjs` | 상시 | active | #4159 실제 HWP 물리 3쪽 종료 재귀 중첩 표 bottom 선의 Canvas2D clip 포섭 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4159 | #4069 17쪽 pagination 계약 포함 |
| `issue-4252-nested-partial-table-object-selection.test.mjs` | 상시 | active | #4252 재귀 분할 중첩 표의 실제 IR cellPath·bbox 조회·Esc 객체 선택과 브라우저 성능 증적 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4252 | 물리 5쪽 외부→래퍼→자식 표 경로와 `output/4252/perf-*.json` |
| `issue-4694-chart-data-edit.test.mjs` | 상시 | active | #4694 차트 데이터 편집 — 메뉴 노출·더블클릭 진입·값 수정 반영·Ctrl+Z 원복·무편집 무흔적 | chart/세로막대형/묶은세로막대형.hwp | 수동 | |
| `issue-4969-shaping-replay.test.mjs` | 상시 | active | #4969 common GlyphRun의 strict CanvasKit 선택·Source Han exact face·affine advance와 √ratio ink 계약 | SourceHanSerifK-OldHangul-subset.otf | npm e2e:issue-4969 | software CanvasKit pixel 대조 |
| `issue-4272-page11-nested-cell-copy.test.mjs` | 상시 | active | #4272 물리 11쪽 자식 표 셀 문단 22의 실제 드래그·Ctrl+C 최내곽 문단 축 검증 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4272-page11 | `output/4272/page11-child-cell-copy.{json,png}` |
| `issue-4272-nested-table-object-copy.test.mjs` | 상시 | active | #4272 물리 5쪽 3중 중첩 표 객체 선택의 owner path·control index 분리와 Ctrl+C 검증 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4272-table-copy | `output/4272/nested-table-object-copy.{json,png}` |
| `issue-4272-nested-cell-text-selection.test.mjs` | 상시 | active | #4272 중첩 표 안쪽 셀의 전체 cellPath 선택 rect, 실제 마우스 드래그와 Ctrl+C/V | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-4272 | 물리 5쪽 `23,504`, `output/4272/nested-cell-*.{json,png}` |
| `issue-536-boxed-pua-canvas2d.test.mjs` | 상시 | active | #536 실제 HWP의 사각 안 숫자 PUA를 기본 Canvas2D가 결정적으로 합성 | basic/issue2007_nested_cell_pagination_42065.hwp | npm e2e:issue-536 | #4122 17쪽 pagination stack 계약 포함 |
| `local-font-partial-enumeration-issue4741.test.mjs` | 상시 | active | #4741 Local Font Access 부분 열거를 강제해 raw probe·exact Canvas2D face·폭·cache·편람 383쪽 계약 검증 | 2025 행정업무운영 편람(최종).hwp | npm e2e:issue-4741 | KoPub바탕체 Light 설치 호스트 Chrome 전용 |
| `issue-595.test.mjs` | 진단 | hold | Issue #595 진단 e2e | exam_math.hwp | 수동 | legacy-name · #595 진단 (assertion 0) |
| `issue-6099-probe.mjs` | 진단 | active | #6099 90° 회전 그림 DOM frame/img 실측과 스크린샷 생성 프로브 | samples 하위 지정 파일 | 수동 | legacy-name · 일회성 실측 프로브 |
| `issue-6117-cell-underline-canvas2d.test.mjs` | 상시 | active | #6117 표 칸 안 밑줄이 우측 괘선을 넘어 그려지지 않는 Canvas2D 잉크 경계 | issue6117/52690_higher_education_decree.hwp | 수동 | 9쪽 실제 fixture, `output/6117` 증적 |
| `line-spacing.test.mjs` | 상시 | active | 줄간격 변경에 따른 페이지 넘김 검증 | — | 수동 |  |
| `loading-busy-cursor.test.mjs` | 상시 | active | #5740 대형 문서 로딩 중 busy 상태와 wait cursor 표시 계약 | 2025 행정업무운영 편람(최종).hwp | npm e2e:loading-busy-cursor |  |
| `merged-cell-boundary-drag.test.mjs` | 상시 | active | #6557 세로 병합 셀 표에서 하위 행만 선택하고 열 경계를 드래그 — 선택 필터의 병합 셀 포함·걸친 모든 행의 이웃 보상·균일 결과 무마킹 세 층이 함께 고쳐져야 경계가 전 행에서 같은 x 로 이동 | — | npm e2e:issue-6557-merged-col | dev server 필요 — run-with-vite.mjs 경유 · 증적 [assets/merged-cell-resize-evidence](https://github.com/jeong-sik/rhwp/tree/assets/merged-cell-resize-evidence) |
| `merged-cell-row-boundary-drag.test.mjs` | 상시 | active | #6557 가로 병합 셀 표의 행 경계 드래그 — 병합 셀이 걸친 모든 열의 아래 이웃이 보상을 받아 표 전체 높이가 보존되는지 검증 | — | npm e2e:issue-6557-merged-row | dev server 필요 — run-with-vite.mjs 경유 · 증적 [assets/merged-cell-resize-evidence](https://github.com/jeong-sik/rhwp/tree/assets/merged-cell-resize-evidence) |
| `navigation-shortcuts.test.mjs` | 상시 | active | 플랫폼별 navigation shortcut | — | 수동 |  |
| `page-border-toggle.test.mjs` | 상시 | active | 쪽 테두리/배경 미리보기 버튼 토글 | — | 수동 |  |
| `page-break.test.mjs` | 상시 | active | biz_plan.hwp 강제 쪽 나누기 / "5. 사업추진조직" 문단 앞에 쪽 나누기 삽입 후 페이지 재배치 확인 | biz_plan.hwp | 수동 |  |
| `page-break-caret-reveal.test.mjs` | 상시 | active | Cmd+Enter 쪽 나누기 뒤 새 쪽 캐럿 DOM 재배치와 편집 영역 viewport 자동 스크롤 | — | npm e2e:page-break-caret | dev server 필요 — run-with-vite.mjs 경유 |
| `page-key-scroll.test.mjs` | 상시 | active | PgUp/PgDn 쪽 단위 스크롤 — 건너뜀 없음·쪽 머리 착지·캐럿 추종·포커스 밖 동작 | biz_plan.hwp | npm e2e:page-key-scroll |  |
| `page-setup-orientation-icon.test.mjs` | 상시 | active | 편집 용지 대화창의 용지 방향 아이콘 식별성 | — | 수동 |  |
| `pdf-render-diff-report.mjs` | 상시 | active | Report-only visual diff between browser Canvas output and SVG-deri | — | npm+CI | legacy-name |
| `plugin-lifecycle.test.mjs` | 상시 | active | 플러그인 호스트 — allowlist·트랜잭션 1스냅샷·롤백·unload 회수 | — | npm e2e:plugin-lifecycle |  |
| `print-pdf-issue3126.test.mjs` | 상시 | active | #3126 same-origin iframe 인쇄/PDF UX, 상태 불변, #2524/#2525 browser PDF 회귀 | render-p35-font-native-bitmap.hwpx, hwpx/hwpx-02.hwpx | 수동 | native dialog는 Chrome/Edge 수동 절차 병행 |
| `probe-image-repaint-issue3315.mjs` | 진단 | active | Issue #3315 Track 4 종결 측정 — 그림 1장 문서의 타이핑·`document-changed`·개체 이동 비용을 그림 없음과 대조 (#2520 프로브 형식, 브리지 메서드별 내역 포함) | images/tiger01.jpg | npm e2e:issue-3315-perf | 시간 수치는 비-CI 진단 |
| `probe-input-perf-issue3137.mjs` | 진단 | active | Issue #3137 거대 표 셀 입력의 mutation·cursor update·focused repaint·operation·2-rAF·long task 성능 매트릭스 | issue1949_giant_cell_nested_tables_perf.hwp/.hwpx | npm e2e:issue-3137-perf | 시간 수치는 비-CI, 문서·cursor·focused repaint·flush 계약만 hard assertion |
| `pr2260-vscode-zoom-menu.test.mjs` | 상시 | active | [PR #2260 검증] rhwp-vscode 배율 메뉴 — 호스트 Chrome CDP 로 webview 하네스 구동. | — | 수동 |  |
| `issue-4224-pua-f02fb-small-right-triangle-canvas2d.test.mjs` | 상시 | active | #4224 한컴 문자표 U+F02FB 일반 TextRun을 작은 오른쪽 방향 삼각형으로 Canvas2D 투영 | basic/pau-004.hwp | npm e2e:issue-4224 | raw IR 보존·공개 글꼴 tofu 차단 |
| `renderer-baseline-native-diff.mjs` | 유틸 | active | 렌더러 baseline — studio vs native 산출 대조 | — | CI |  |
| `renderer-baseline-contract.mjs` | 유틸 | active | CanvasKit runtime 실패 진단 정규화 helper | — | 수동 | `renderer-baseline.mjs`에서 import |
| `renderer-baseline.mjs` | 유틸 | active | 렌더러 baseline 스윕 러너 (manifest 기반 다문서 측정) | — | npm+CI |  |
| `renderer-contract.test.mjs` | 상시 | active | 렌더러 백엔드 계약 검증 (plane/replay 정합) | — | npm+CI |  |
| `report-generator.mjs` | 유틸 | active | E2E 테스트 HTML 보고서 생성기 | — | 수동 |  |
| `responsive.test.mjs` | 상시 | active | #6118 서식 바 경계·문단 더보기, #6138 도구 한 줄 스크롤, #6187 모든 너비·낮은 높이의 눈금자 표시와 grid 정렬 검증 (resize 프레임 공백 검증은 별도) | — | npm e2e:responsive + CI |  |
| `ruler-document-switch.test.mjs` | 상시 | active | 문서를 바꿔 열면 눈금자가 새 문서의 쪽을 다시 그린다 (`document-view-loaded`) — 문단 여백이 같은 문서를 잇달아 열 때 앞 문서 눈금이 남는 회귀 가드 | 누름틀-2024.hwp · 253E164F57A1BC6934-empty.hwp | npm e2e:ruler-document-switch |  |
| `ruler-resize.test.mjs` | 상시 | active | #6187 resize 경계의 눈금자 grid·정렬·실제 화면 snapshot 검사; 전체 합성 프레임 보증과 구분, browser-client driver 주입 지원 | exam_kor.hwp | 수동 |  |
| `page-virtualization-image-failure.test.mjs` | 상시 | active | #6042 실제 Chromium에서 첫 embedded-image decode 실패를 주입하고 PageRenderer fallback 뒤 scheduler·image job·flow layer 정착을 검사 | test-image.hwp | 수동 |  |
| `run-render-diff.mjs` | 유틸 | active | render-diff CI 러너 (canvas/pdf diff 오케스트레이션) | — | npm+CI |  |
| `run-with-vite.mjs` | 유틸 | active | Vite dev server 기동 + 임의 명령 실행 공용 러너 (VITE_URL 주입, 종료 코드 전파) | — | npm e2e:undo-depth |  |
| `save-as-format.test.mjs` | 상시 | active | 저장 출력 포맷 선택 (file:save-as-hwp / file:save-as-hwpx) E2E — #1613 | biz_plan.hwp, hwpx/footnote-01.hwpx | 수동 |  |
| `scenario-runner.mjs` | 유틸 | active | 시나리오 실행기 + 렌더 트리 측정기 + 규칙 검증기 | — | 수동 |  |
| `vite-server.mjs` | 유틸 | active | Vite dev server 기동·대기·종료 공용 헬퍼 — node 직접 기동(win32 .cmd EINVAL 우회), taskkill 트리 정리 | — | 수동 | `run-render-diff.mjs`·`run-with-vite.mjs`에서 import |
| `shape-inline.test.mjs` | 상시 | active | 도형 인라인 컨트롤 — 커서 이동 및 텍스트 삽입 | — | 수동 |  |
| `shift-end.test.mjs` | 상시 | active | shift-return.hwp Shift+End 블록 선택 | shift-return.hwp | 수동 |  |
| `status-page-number.test.mjs` | 상시 | active | #5749 상태 표시줄 쪽 번호가 물리 순번이 아니라 문서 쪽번호를 따르는 계약 | 쪽기준.hwp | npm e2e:status-page-number |  |
| `table-border-hover-resize-issue4117.test.mjs` | 상시 | active | #4117 셀 선택 모드 클릭 없이 표 경계 hover → 리사이즈 커서·드래그 동작 — 이동 스톰 60회 중 엔진 호출 ≤2 단정으로 task 2010 랙 재발 방지 | — | npm e2e:issue-4117-border-hover | dev server 필요 — run-with-vite.mjs 경유 |
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
| `toolbox-visibility.test.mjs` | 상시 | active | 기본 도구 상자 접기/펴기와 표시 상태 저장·복원 | — | npm e2e:toolbox-visibility |  |
| `topmost-hittest.test.mjs` | 상시 | active | E2E 테스트 (Issue #1280 v2): 겹침 클릭 = "최상단 개체" 선택 | textbox-under-image.hwp | 수동 |  |
| `topmost-lifecycle.test.mjs` | 상시 | active | E2E 테스트 (Issue #1280 v2): 겹침 최상단 선택 → 연산 lifecycle | textbox-under-image.hwp | 수동 |  |
| `typesetting.test.mjs` | 상시 | active | 조판 품질 검증 (문단부호 표시 상태) | — | 수동 |  |
| `undo-contracts.test.mjs` | 상시 | active | 편집 undo 계약 실동작 검증 (Task #2301) | — | npm e2e:undo |  |
| `undo-delete-fragment-bytes.test.mjs` | 상시 | active | [#5769] 다문단 선택 삭제 조각 undo 저장 바이트 왕복 동일성 (Delete 키 → Ctrl+Z → exportHwp 대조) | — | npm e2e:undo-delete-fragment-bytes |  |
| `undo-depth-issue5769.test.mjs` | 상시 | active | [#5769] 혼합 세션(선택 삭제+재입력 R라운드) 실효 undo 깊이 무축출 계약 — 슬롯 합 0·조각 경로 다수 라운드(≥⌈R/2⌉, 짧아진 문단은 refill 분기)·스택 전량 복원·새 문서 상태 복귀. 회귀 시 예산 축출로 ③④가 깨진다 | — | npm e2e:undo-depth | E2E_DEPTH_ROUNDS 로 스모크 |
| `undo-object-selection.test.mjs` | 상시 | active | E2E: undo/redo 후 개체/표 선택 stale ref 해제 (Task #2303) /  / 계약: undo/r | — | npm e2e:undo-object-selection |  |
| `unsaved-changes-guard.test.mjs` | 상시 | active | #886 저장되지 않은 변경사항 보호 모달 | — | npm e2e:unsaved-guard |  |
| `unsupported-format-error.test.mjs` | 상시 | active | 미지원 문서 오류 알림 후 정상 문서 재로드 | field-01.hwp | 수동 |  |
| `zoom-fit-mode-persistence.test.mjs` | 상시 | active | 쪽 맞춤/폭 맞춤 선택 저장과 문서 로드 시 복원 — 새 문서 쪽 크기로 재계산, 수치 배율은 맞춤 해제 | 2010-01-06.hwp · 253E164F57A1BC6934-empty.hwp | npm e2e:zoom-fit-mode |  |
| `zoom-dialog-transaction.test.mjs` | 상시 | active | #6109 사용자 배율 오류·ARIA·Enter/Escape/취소와 배치+이동+배율 단일 transaction — 최종 상태 `recalcLayout()` 1회 | — | npm e2e:zoom-dialog-transaction | dev server 필요 — run-with-vite.mjs 경유 |
