# 구현 계획서 — Task #1171: 사각형 글상자 안 picture click hit-test / 속성 지원

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171) (M100 / v1.0.0)
- **브랜치**: `local/task1171` (base: `local/devel`)
- **수행계획서**: `mydocs/plans/task_m100_1171.md`
- **작성일**: 2026-06-02

각 단계 완료 후 `mydocs/working/task_m100_1171_stage{N}.md` 보고서 작성 + 해당 단계 소스 커밋,
승인 후 다음 단계 진행.

---

## Stage 1 — shape_layout picture에 CellContext 전달 (백엔드 식별자 생성)

**목적**: 글상자 안 picture의 ImageNode가 `cell_context`(cell_index=0 sentinel path)를 갖도록 하여
controls JSON에 `cellPath`가 직렬화되게 한다. (B)의 식별 공백 #1 해소.

**변경**:
- `src/renderer/layout/shape_layout.rs:2204-2215`(inline), `2226-2237`(absolute):
  `layout_picture(..., None)` → text-run 경로(1963-1975행) 패턴을 복제한 `pic_cell_ctx` 빌드 후
  `Some(&pic_cell_ctx)` 전달.
  ```rust
  let pic_cell_ctx = CellContext {
      parent_para_index: para_index,
      path: { let mut p = parent_cell_path.to_vec();
              p.push(CellPathEntry { control_index, cell_index: 0,
                                     cell_para_index: pi, text_direction: 0 }); p },
  };
  ```
  - 주의: cellPath sentinel의 `control_index`는 **바깥 Shape**의 본문 control 인덱스(현 scope의
    `control_index` 변수), ImageNode의 `control_index`(JSON `controlIdx` = inner ctrl)는 기존
    `Some(ctrl_idx_in_para)` 유지. 두 값을 혼동하지 않는다.

**검증**:
- `cargo build`.
- `rhwp export-svg samples/tac-img-02.hwp -p 5 -p 6 --debug-overlay`로 picture 렌더 위치 회귀 없음 확인.
- 단위 테스트 신규: 글상자→paragraph→picture fixture를 `build_page_render_tree` 후 해당 ImageNode의
  `cell_context.path` 마지막 엔트리가 `{control_index: Shape, cell_index:0, cell_para_index: 글상자문단}`
  인지 검증. (셀 안 picture cell_context 테스트 패턴 참조.)

**게이트**: picture 렌더 위치 무회귀 + cell_context 정상 생성 확인.

---

## Stage 2 — 백엔드 by_path getter/setter 글상자 지원 (속성 read/write)

**목적**: by_path picture getter/setter가 마지막 세그먼트가 글상자인 path도 해석. (B)의 공백 #2 해소.

**변경**:
- `src/document_core/commands/object_ops.rs:3104-3129` `get_cell_picture_properties_by_path_native`:
  `resolve_cell_by_path` + `cell.paragraphs[last.2]` → `resolve_paragraph_by_path`(글상자/셀 양쪽
  지원, `cursor_nav.rs:584`)로 paragraph를 직접 얻은 뒤 `cell_para.controls[inner_control_idx]`.
- `src/document_core/commands/object_ops.rs:3166-3208` `set_cell_picture_properties_by_path_native`:
  `resolve_cell_paragraph_mut`(Table 전용) → 신규 `resolve_paragraph_by_path_mut`(immutable
  `resolve_paragraph_by_path`의 mut 미러, 회귀 격리)로 교체.
- 신규 헬퍼 `resolve_paragraph_by_path_mut`(`cursor_nav.rs` 또는 `object_ops.rs`): `Control::Table`
  + `Control::Shape`(get_textbox_from_shape mut) arm을 모두 처리.
- `src/document_core/queries/rendering.rs:1539-1558`: **무변경**(Stage 1 cell_context로 자동 직렬화).

**검증**:
- `cargo test` 신규 round-trip 테스트 `picture_in_textbox_get_set_by_path`: 글상자 picture를 path로
  조회 → 속성 변경(width/height) → 재조회 일치.
- 기존 표 셀 picture 테스트(`object_ops.rs:7525` 등) + 전체 `cargo test` 회귀 0.
- `cargo clippy`.

**게이트**: round-trip 성공 + 표 셀 회귀 0.

---

## Stage 3 — 프론트엔드 picture 우선 hit-test (A)

**목적**: 글상자 내부 클릭이 텍스트 편집으로 단락되기 전에 글상자 안 picture를 선제 선택.

**변경**:
- `rhwp-studio/src/engine/input-handler-mouse.ts`: 글상자 경계선 검사(705-720) 직후, 텍스트 편집
  진입(744) 직전에 분기 추가 —
  `if (hit.isTextBox)` 일 때 `findPictureAtClick(pageIdx, pageX, pageY)`를 선제 호출하여
  반환 picHit이 `type==='image'|'equation'`이고 `cellPath`(글상자 sentinel 포함) 동반이면
  760-857의 picture dispatch 로직(enterPictureObjectSelectionDirect)으로 보내고 `return`.
  - 외곽 경계선은 위 705 블록이 이미 Shape 선택 처리 → 본 분기는 내부 picture만.
  - picHit이 없거나 picture가 아니면 기존 744행 텍스트 편집 fall-through 유지.

**검증**:
- `cd rhwp-studio && npx tsc --noEmit`.
- E2E 신규 시나리오: tac-img-02.hwp 로드 → 글상자 안 picture 클릭 → `picture-object-selection-changed`
  true + 선택 ref가 글상자 picture(cellPath 보유) 확인.
- 표 셀 picture 클릭, picture 없는 글상자 텍스트 편집 진입 회귀 확인.

**게이트**: 글상자 picture 객체선택 진입 + 기존 클릭 동작 무회귀.

---

## Stage 4 — dialog cellPath 정합 (insert.ts)

**목적**: 선택된 글상자 picture의 속성 대화상자가 올바른 by_path API를 호출하도록 cellPath 전달.

**변경**:
- `rhwp-studio/src/command/commands/insert.ts:272-287`: cellPath 소스를 스칼라
  (`outerTableControlIdx/cellIdx/cellParaIdx`) 재구성 대신 `ref.cellPath`(findPictureAtClick이 준
  full array) 우선 사용. `innerControlIdx=ref.ci` 유지. `ref.cellPath` 부재 시 기존 스칼라
  재구성으로 fallback(표 셀 단일 레벨 안전망).
- `picture-props-dialog.ts` / `cursor.ts` / `input-handler-picture.ts`: **무변경**(정합 확인만).

**검증**:
- tsc.
- E2E: 글상자 picture 선택 → 개체 속성 dialog open → width/height read 일치 → 변경 적용 후
  재렌더 반영.
- 표 셀 picture 속성 dialog read/write 회귀 확인.

**게이트**: 글상자 picture 속성 read/write 정상 + 표 셀 dialog 무회귀.

---

## Stage 5 — 통합 + 수동 검증 (tac-img-02.hwp p6/p7)

**목적**: 전체 정합 및 작업지시자 시각 판정.

**변경**: 없음(검증·문서화·최종 보고서).

**검증**:
- 전체 `cargo test`, `cargo clippy`, studio `npx tsc --noEmit` + `npm run build`.
- (필요 시) WASM 빌드(Docker) 후 studio 로드.
- 수동(호스트 Chrome CDP, `--remote-debugging-port=9222`): tac-img-02.hwp p6/p7 —
  ① 글상자 안 picture 클릭 → picture 객체선택
  ② 속성 dialog 크기 변경 → 반영
  ③ 글상자 외곽 클릭 → Shape 선택(기존 유지)
  ④ picture 없는 글상자 영역 클릭 → 텍스트 편집 진입(기존 유지).
- **산출**: `mydocs/report/task_m100_1171_report.md`.

**게이트**: 4개 수동 시나리오 통과 + 전체 회귀 0.
