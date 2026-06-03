# 구현계획서 — Task M100-1161: 셀 안 picture 복사(Ctrl+C) cell_path 지원 (4단계)

## 설계 요약

- **공통 헬퍼 신설** — `resolve_control_para(sec, para, cell_path) -> &Paragraph` (cursor_nav.rs, `pub(crate)`). 빈 cell_path = 본문 `sections[sec].paragraphs[para]`, 아니면 기존 `resolve_paragraph_by_path`(cursor_nav.rs:574) 위임. 네 native 의 동일 프롤로그(`sections→paragraphs→controls`)를 이 헬퍼로 일원화하여 컨트롤 접근부만 교체.
- **native 4개 시그니처 확장** — `clipboard.rs` 의 `copy_control_native`(222)/`export_control_html_native`(924)/`get_control_image_data_native`(1284)/`get_control_image_mime_native`(1332) 에 `cell_path: &[(usize,usize,usize)]` 인자 추가. 본문은 빈 슬라이스로 기존 동작 보존.
- **wasm_api 래퍼 4개** — `insert_picture` 패턴 그대로: `cell_path_json: &str` 추가 → `if empty/"[]" {Vec::new()} else {DocumentCore::parse_cell_path(json)?}` → native 전달.
- **TS** — picture ref 스칼라(`outerTableControlIdx`,`cellIdx`,`cellParaIdx`) → 단일 레벨 cell_path JSON 조립 헬퍼 1개. 모든 복사/오려두기/이미지기록 지점에서 사용.
- **차용**: `copy_control_native`(&mut self) 는 헬퍼로 `&Paragraph`(불변 차용) 획득 → control clone + clip_para(owned) 구성 후 차용 종료 → `self.clipboard = Some(..)` 기록. 기존 함수와 동일 순서라 NLL 통과.

## Stage 1 — clipboard.rs 공통 헬퍼 + 4 native cell_path 확장 + RED 테스트

**목표**: native 셀 경로 지원 + 결함 박제(RED).

- `src/document_core/queries/cursor_nav.rs`: `resolve_control_para(&self, sec, para, cell_path) -> Result<&Paragraph, HwpError>` 추가.
- `src/document_core/commands/clipboard.rs`: 4 native 에 `cell_path: &[(usize,usize,usize)]` 인자 추가, 컨트롤 접근을 `let para = self.resolve_control_para(sec, para_idx, cell_path)?; let control = para.controls.get(control_idx)...` 로 교체. `copy_control_native` 의 `char_shape_id_at`/`para_shape_id`/`ctrl_data_records` 도 동일 `para`(셀 문단) 기준.
  - 이 단계에서 wasm_api 래퍼도 컴파일 위해 `cell_path` 인자 전달 추가(본문 호출 `&[]`).
- `tests/issue_1161_copy_picture_in_cell.rs` (신규):
  - `samples/pic-in-table-01.hwp` 파싱 → 셀 안 picture 위치(sec, parent_para, cell_path, inner_ctrl) 탐색 → `copy_control_native(sec, parent_para, &cell_path, inner_ctrl)` 호출 → `self.clipboard.paragraphs[0].controls[0]` 가 `Control::Picture` 인지 단언.
  - 본문 그림 회귀: 빈 cell_path 로 기존 본문 복사가 여전히 Picture 복사 성공.
  - RED 기준: cell_path 인자 도입 전엔 컴파일 자체가 신호이므로, **결함 재현 테스트**는 "기존 본문 전용 호출(outer para + inner ci)이 Picture 가 아님"을 명시적으로 단언하는 형태로 추가(고친 뒤 cell_path 경로가 Picture 반환).
- 검증: `cargo build` + `cargo test --test issue_1161_copy_picture_in_cell` (셀 경로 GREEN, 회귀 GREEN).
- 보고서: `mydocs/working/task_m100_1161_stage1.md`

## Stage 2 — wasm_api 래퍼 4개 cell_path_json 연결

**목표**: WASM 경계에 cell_path 노출.

- `src/wasm_api.rs`: `copyControl`/`exportControlHtml`/`getControlImageData`/`getControlImageMime` 4 래퍼에 `cell_path_json: &str` 인자 추가. `insert_picture` 와 동일하게 파싱 후 native 전달. 본문 호출 호환(`""`).
- 검증: `cargo build`(wasm feature 포함 가능 시) + `cargo test --tests` 전수 회귀.
- 보고서: `mydocs/working/task_m100_1161_stage2.md`

## Stage 3 — TS 브리지 + 호출 지점 + 타입 보정

**목표**: 프런트 전 복사 경로가 셀 컨텍스트 전달.

- `rhwp-studio/src/core/wasm-bridge.ts`: `copyControl`/`exportControlHtml`/`getControlImageData`/`getControlImageMime` 4 래퍼에 `cellPathJson: string = ''` 인자 추가 → `this.doc.*(... , cellPathJson)`.
- `rhwp-studio/src/engine/input-handler-keyboard.ts`:
  - cell_path JSON 조립 헬퍼 추가: ref 에 `cellIdx`·`cellParaIdx`·`outerTableControlIdx` 가 모두 있으면 `JSON.stringify([{controlIndex: outerTableControlIdx, cellIndex: cellIdx, cellParaIndex: cellParaIdx}])`, 아니면 `''`.
  - onCopy(1265)/onCut(1347)/onKeyDown 복사·오려두기(625,652,755,776) 의 `copyControl`·`exportControlHtml` 호출에 cellPathJson 전달.
  - `writeImageToClipboard`(178) 에 `cellPathJson` 인자 추가 → 내부 `getControlImageData`/`getControlImageMime` 에 전달. 호출처도 갱신.
- `rhwp-studio/src/engine/input-handler.ts`: 컨텍스트 메뉴 복사/오려두기(2385,2407) 동일 전달 + `getSelectedPictureRef()` 반환 타입(2139)에 `outerTableControlIdx?: number` 보정.
- 검증: `cd rhwp-studio && npx tsc --noEmit`(타입) + 빌드.
- 보고서: `mydocs/working/task_m100_1161_stage3.md`

## Stage 4 — WASM 빌드 + 통합 검증 + 시각 판정 + 최종 보고서

**목표**: 전 경로 회귀 가드 + 작업지시자 수동 게이트.

- `cargo test --tests` 전수 + `cargo clippy`(변경 파일) + `cargo fmt --check`(신규/수정 파일).
- WASM 빌드: `docker compose --env-file .env.docker run --rm wasm`.
- **작업지시자 수동/시각 판정**: `samples/pic-in-table-01.hwp` page 22 셀 그림 선택 → Ctrl+C → 본문/다른 셀 Ctrl+V 로 그림 붙여넣기 정상 + 외부 앱에 image/png 붙여넣기 정상.
- 회귀: 본문 inline 그림, PR #1177 floating 셀 그림 복사 정상 유지.
- 최종 보고서: `mydocs/report/task_m100_1161_report.md`.

## 단계별 커밋 규칙

각 stage 소스 + `_stage{N}.md` 보고서를 `local/task1161` 에서 함께 커밋. 기능/포맷 분리, 무관 rustfmt diff 금지. 최종 보고서·orders 갱신도 머지 전 커밋(`git status` 확인).

## 리스크 점검 항목

- 4 native 시그니처 변경 → 호출처는 wasm_api 4 래퍼뿐(Stage 1·2 에서 동시 갱신). 다른 내부 호출 없는지 `grep` 확인.
- `resolve_control_para` 가 셀/본문 모두 동일 `&Paragraph` 반환 → 본문 경로 동작 불변(테스트 가드).
- `copy_control_native` 의 `&mut self` + 불변 차용 충돌 → owned 추출 후 기록 순서 유지로 회피.
- 단일 레벨 cell_path 한계(중첩 표/글상자 안 그림) → picture 선택 ref 가 스칼라만 저장하는 기존 제약. 본 작업 범위 밖(별도 이슈 후보), 회귀 아님.
- `feedback_image_renderer_paths_separate`: 클립보드 전용 native 만 수정, 렌더 경로 무변경 확인.
