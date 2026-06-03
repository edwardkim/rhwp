# Stage 1 완료보고서 — Task #1171

- **이슈**: [#1171](https://github.com/edwardkim/rhwp/issues/1171)
- **브랜치**: `local/task1171`
- **단계 목표**: 사각형 글상자(Shape text_box) 안 picture 가 controls JSON 에 cellPath
  (cell_index=0 sentinel)로 노출되도록 백엔드 식별자 생성. (B)의 식별 공백 해소.
- **작성일**: 2026-06-02

## 변경 내용

### 1. shape_layout.rs — picture 에 CellContext 전달 (계획대로)
`src/renderer/layout/shape_layout.rs` `layout_textbox_content` 의 `Control::Picture` 분기:
- text-run(1963행)/equation/table 경로와 동일한 `pic_cell_ctx` 빌드 추가:
  `CellContext { parent_para_index: para_index, path: parent_cell_path + CellPathEntry
  { control_index(바깥 Shape), cell_index: 0, cell_para_index: pi(글상자 문단), text_direction: 0 } }`
- inline/absolute 두 `layout_picture` 호출의 마지막 인자를 `None` → `Some(&pic_cell_ctx)` 로 교체.
- inner picture 인덱스는 기존 `control_index` 인자(= `ctrl_idx_in_para`)로 별도 전달(불변).

### 2. rendering.rs — collect_controls 사각형 자식 재귀 (★ 계획 외 추가 — 아래 "편차" 참조)
`src/document_core/queries/rendering.rs` `get_page_control_layout_native::collect_controls`:
- `RenderNodeType::Rectangle` (유효 좌표) 핸들러에서 "shape" 컨트롤 push 후의 `return;` 제거.
- Table 핸들러와 동일하게 자식으로 재귀 → 글상자 내부 picture/도형이 수집됨.
- 장식 노드(picture 테두리 등)는 `RectangleNode::new` 가 section/para/control 좌표를 `None`
  으로 두므로 좌표 가드(`if let Some`)에 의해 컨트롤로 방출되지 않음(회귀 안전).

## ★ 계획 대비 편차 (작업지시자 인지 필요)

수행/구현계획서는 rendering.rs 를 **"무변경(자동 직렬화 확인만)"** 으로 예상했다. 이는
"shape_layout 이 picture 에 cell_context 를 넣으면 rendering.rs 가 자동으로 cellPath 를
직렬화한다"는 전제였고, 직렬화 코드(rendering.rs:1539) 자체는 실제로 그대로다.

그러나 Stage 1 구현 중 진단으로 **collect_controls 가 사각형(Rectangle) 노드에서 조기
`return` 하여 글상자 내부로 재귀하지 않는 비대칭**을 발견했다(표 Table 노드는 이미 재귀).
이 때문에 cell_context 를 넣어도 picture 노드가 **수집 단계에서 누락**되어 JSON 에
나타나지 않았다. 즉 picture 직렬화 로직은 무변경이 맞으나, 그 직전의 **트리 순회(수집)**
한 줄(`return` 제거)이 추가로 필요했다.

- 영향 범위: 유효 좌표를 가진 모든 Rectangle(Shape) 노드의 controls 수집 — 글상자 안
  중첩 picture/도형이 새로 노출된다(기존 컨트롤은 불변, 누락분만 추가).
- 회귀 검증: 전체 `cargo test` 1947 passed / 0 failed 로 회귀 없음 확인.
- 셀 방식과의 정합: 이 수정은 "셀과 다른 방식"이 아니라 **표 셀 수집이 이미 하던 재귀를
  글상자에도 동일 적용**한 것이다(접근 B 통합 원칙 그대로).

## 검증

- `cargo build` 통과, `cargo clippy --tests` 변경 파일/라인 경고 0(나머지는 기존 경고).
- 신규 테스트 `tests/issue_1171_textbox_picture_cellpath.rs`
  (`textbox_picture_emits_cellpath_sentinel`) 통과:
  - 섹션0 문단25 picture 2개(inner ctrl 0,1) + 문단44 picture 1개(inner ctrl 0)가
    `cellPath=[{controlIndex:0, cellIndex:0, cellParaIndex:0}]` + `parentParaIdx`=각 25/44
    로 노출됨을 단정.
  - 렌더 위치: 문단25 → page index 5(6쪽), 문단44 → page index 6(7쪽) — 이슈의 p6/p7 일치.
- **전체 `cargo test`: 1947 passed, 0 failed.**
- 타깃 회귀: issue_1161_image_cellpath(1), issue_1139_inline_picture_duplicate(41),
  issue_717_table_cell_hit_test(3), issue_919_textbox_hit_test(5) 전부 통과.

## 다음 단계 (Stage 2)

백엔드 by_path picture getter/setter 가 글상자 path(마지막 세그먼트=Shape)도 해석하도록
`resolve_cell_by_path`/`resolve_cell_paragraph_mut` → 글상자 지원 resolver 로 교체.
이제 controls JSON 의 cellPath 가 올바르므로, 그 cellPath 로 속성 read/write 가 가능해야 한다.
