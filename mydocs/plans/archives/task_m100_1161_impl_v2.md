# 구현계획서 v2 — Task M100-1161: 셀 안 picture 복사 + ImageNode 다단계 cellPath 기반 (5단계)

> v1(task_m100_1161_impl.md) 대비 변경: #1171 연관 발견(중첩 picture 는 ImageNode 가
> 다단계 cellPath 를 못 들고 다님 — TextRun 만 보유)을 반영. ImageNode 에 전체 cellPath
> 를 싣는 **공유 기반**을 #1161 안에 구축(작업지시자 승인 옵션 1). Stage 1 은 v1 그대로 완료.

## 배경 — 발견된 공통 뿌리 원인

- TextRun: `cell_context: Option<CellContext>`(전체 다단계 `path`) 보유 → 레이아웃이
  `cellPath:[...]` 방출(rendering.rs:1313-1327). 중첩 셀 텍스트 편집 정상.
- ImageNode: **단일 레벨 스칼라**(`cell_index`/`cell_para_index`/`outer_table_control_index`)만
  (render_tree.rs:652-658). `make_picture_image_node`(paragraph_layout.rs:4163)가
  in-scope `cell_ctx` 를 `last_image_indices()`로 **innermost 만 투영**(4176-4189) → 다단계 손실.
- 결과: `samples/pic-in-table-01.hwp` p22(외부표→셀→내부표→셀→그림, 2단계) 의 picture 는
  프런트가 전체 경로 복원 불가 → 복사/선택 불가. #1171(사각형→글상자→picture)도 동일 뿌리.

## 작업지시자 확정 계약 (A)(B)(C)

- **(A) cellPath 단일 진실원**: ImageNode `cell_context` 를 권위값으로. 기존 스칼라는
  `last_image_indices()` 파생(단일 레벨 투영)으로 **유지**(하위호환·회귀 0).
- **(B) 공유 채움 경로**: ImageNode path 를 TextRun 과 **동일한 `cell_ctx`** 로 채움.
  단일 chokepoint `make_picture_image_node` 한 곳만 수정(픽처 전용 임시코드 금지).
- **(C) hit-test 불간섭**: findPictureAtClick 의 Shape-vs-picture 우선순위는 **#1171 영역**.
  #1161 은 *이미 선택된* picture 복사만. 표-중첩 picture 선택 가능 여부는 Stage 4 에서 실측.

---

## Stage 1 — clipboard native 4종 cell_path + 공통 헬퍼  ✅ 완료 (commit 4bae87cc)

`resolve_control_para` + copy/export/image native 4종 cell_path 확장 + 회귀 테스트 4종.
(상세: task_m100_1161_stage1.md)

## Stage 2 — wasm_api 래퍼 4개 cell_path_json 연결

**목표**: WASM 경계에 cell_path 노출(클립보드 4 API).

- `src/wasm_api.rs`: `copyControl`/`exportControlHtml`/`getControlImageData`/`getControlImageMime`
  에 `cell_path_json: &str` 추가 → `insert_picture` 와 동일 파싱(`parse_cell_path`, `""`/`"[]"`=본문) 후 native 전달.
- 검증: `cargo build` + `cargo test --tests`(기존 회귀) + 새 wasm_api 단위테스트(셀 cellPath JSON → "[그림]").
- 보고서: `mydocs/working/task_m100_1161_stage2.md`

## Stage 3 — ImageNode 다단계 cellPath 기반 (#1171 공유) [신규]

**목표**: ImageNode 가 전체 다단계 cellPath 를 보유·방출. (계약 A/B)

- `src/renderer/render_tree.rs`: `ImageNode` 에 `cell_context: Option<CellContext>` 추가
  (TextRun 정합). `ImageNode::new` 기본 None. 기존 스칼라 필드는 **유지**(파생값).
  - `CellContext` 가 layout 정의이므로 의존 방향 확인 — 필요 시 `Vec<CellPathEntry>` 만 보관.
- `src/renderer/layout/paragraph_layout.rs`: `make_picture_image_node` 에서
  `cell_context: cell_ctx.cloned()` 설정(스칼라는 `last_image_indices()` 그대로 = 계약 A).
  **chokepoint 단일 수정**(계약 B). 우회 ImageNode 생성 site 감사(layout.rs:4795 본문=None 정상,
  table_cell_content.rs:753 등 픽처 site 가 helper 경유인지 점검).
- `src/document_core/queries/rendering.rs`: ImageNode 방출부(1515-1529)에 `cellPath:[...]`
  추가 — TextRun 방출(1313-1327)과 **동일 포맷 공유**(가능하면 헬퍼 추출). 스칼라도 계속 방출.
- 회귀 테스트 `tests/issue_1161_image_cellpath.rs`(신규): `get_page_control_layout_native` 로
  p22 페이지 렌더 → image 컨트롤 JSON 에 **2-엔트리 cellPath** 존재 단언. 본문/단일레벨 picture 는
  cellPath 없음/단일 엔트리 회귀 가드.
- 검증: `cargo test --tests` 전수 + fmt/clippy. **렌더 출력(픽셀) 무변경**(메타데이터만 추가) 확인.
- 보고서: `mydocs/working/task_m100_1161_stage3.md`

## Stage 4 — TS: 선택 ref 가 cellPath 보유 + 복사 전 지점 배선

**목표**: 프런트가 다단계 cellPath 로 복사(시스템+내부 클립보드 전 경로).

- `findPictureAtClick`(input-handler-picture.ts): layout image 의 `cellPath` 를 읽어 반환값에 포함.
- `cursor.ts` `getSelectedPictureRef`/`enterPictureObjectSelectionDirect`: `cellPath` 보유(스칼라는
  하위호환 유지). `input-handler.ts:2139` 타입에 `outerTableControlIdx`/`cellPath` 보정.
- `wasm-bridge.ts`: 4 클립보드 래퍼에 `cellPathJson` 인자.
- 호출 지점: onCopy/onCut/onKeyDown 복사·오려두기/컨텍스트 메뉴 + `writeImageToClipboard` 가
  ref.cellPath 를 JSON 으로 전달(없으면 `""`).
- **실측(계약 C)**: 표-중첩 picture 가 현재 선택 가능한지 확인. 불가 시(=Shape 가로채기 등)
  별도 이슈로 분리(#1171 영역), 복사 배선은 유지.
- 검증: `cd rhwp-studio && npx tsc --noEmit`.
- 보고서: `mydocs/working/task_m100_1161_stage4.md`

## Stage 5 — WASM 빌드 + 통합 검증 + 시각 판정 + 최종 보고서

- `cargo test --tests` 전수 + clippy/fmt + `tsc`. WASM 빌드(docker).
- **작업지시자 수동/시각**: p22 셀 그림 선택→Ctrl+C→본문/다른 셀 Ctrl+V + 외부 앱 image/png.
- 회귀: 본문 inline 그림, 단일레벨 셀 그림(있으면), PR #1177 floating 셀 그림.
- **#1171 회귀 영향 절**: ImageNode cellPath 가 additive(스칼라 유지)임을 명시, #1171 가
  소비할 cellPath 계약 문서화.
- 최종 보고서: `mydocs/report/task_m100_1161_report.md`(표 70 bit 0 inline 근거 인용 포함).

## 단계별 커밋 규칙

각 stage 소스 + `_stage{N}.md` 를 `local/task1161` 에서 함께 커밋. 기능/포맷 분리, 무관 rustfmt diff 금지.

## 리스크 점검

- **#1171 회귀**: additive(새 필드+새 방출, 스칼라 불변, chokepoint 1곳) → 위험 낮음, 오히려
  #1171 기반 제공. 계약 A/B/C 준수로 이중 출처·우회 site·hit-test 충돌 방지.
- `cell_context` 가 layout↔render_tree 모듈 의존 경계 — 순환 의존 시 `Vec<CellPathEntry>` 만 보관.
- 우회 ImageNode site(helper 미경유)가 있으면 cellPath 누락 → Stage 3 감사 항목.
- 렌더러별(skia/canvaskit/svg) ImageNode 사용 — 새 필드는 무시되므로 무영향(빌드만 확인).
