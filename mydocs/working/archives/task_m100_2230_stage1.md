# 1단계 완료보고서 — Task M100 #2230: placeholder 선택 가능화

- 이슈: #2230 / 구현계획서: `task_m100_2230_impl.md` (승인됨) / 브랜치: `local/task2230`
- 작성일: 2026-07-12

## 수행 내용

studio 개체 선택의 데이터 소스인 `get_page_control_layout` 에 그림 미지정
placeholder 를 방출해 클릭 선택을 성립시켰다.

### 변경 파일 3개

1. **`src/renderer/render_tree.rs`**
   - `ObjectControlRef::picture(si, pi, ci)` 생성자 신설 (kind="picture").
   - `PlaceholderNode.cell_context: Option<CellContext>` 필드 추가
     (`#[serde(skip)]` — 레이어 JSON 부풀림 방지, 컨트롤 레이아웃 전용).
   - `missing_picture()` → `missing_picture(section_index, para_index,
     control_index, cell_context)` (Option 인자 — 좌표 미전파 경로는 기존
     #2225 표시-만 동작 유지).

2. **`src/renderer/layout/picture_footnote.rs`** — 방출 2곳 배선:
   - `layout_picture_full`: 스코프의 section/para/control_index +
     `cell_ctx.cloned()` 전달 (셀 안 그림 — 심볼 필드가 이 경로).
   - `layout_body_picture`: 본문 그림 — 좌표 3종 전달, 셀 경로 None.

3. **`src/document_core/queries/rendering.rs`** — 컨트롤 수집기 Placeholder
   분기에 kind=="picture" 케이스 추가: **`type:"image"` + `missing:true`**
   + Image 분기와 동일 포맷의 `cellIdx/cellParaIdx/parentParaIdx/cellPath`
   방출. 기존 kind=="ole" 분기는 무수정.

### 설계 근거

- `type:"image"` 로 방출하므로 `findPictureAtClick` → hit →
  `enterPictureObjectSelectionDirect` 선택 흐름이 **TS 무수정으로 성립**.
- `missing:true` 마커는 3단계에서 더블클릭 시 "그림 지정 진입"과 "일반
  그림 속성 대화상자"를 분기하는 근거.

## 검증

### 표적 테스트 (신설: `tests/issue_2230_placeholder_selection.rs`, 2건)

- `missing_picture_placeholder_emitted_as_selectable_image_control`:
  36389312 페이지 0 에 missing image 컨트롤 1건 + 실측 bbox(x≈646.2,
  y≈54.9) + secIdx/paraIdx/controlIdx/cellPath 존재.
  **수정 전 FAILED 실증 완료** (missing 컨트롤 0건 — 방출 부재 재현).
- `normal_image_control_has_no_missing_marker`: 좌측 로고(실존 그림,
  x≈84.1)에 missing 마커 미부착 — 마커 오염 회귀 가드.

### 방출 실측 (수정 후)

```json
{"type":"image","missing":true,"x":646.2,"y":54.9,"w":75.6,"h":75.6,
 "secIdx":0,"paraIdx":0,"controlIdx":0,"cellIdx":3,"cellParaIdx":0,
 "parentParaIdx":0,"cellPath":[{"controlIndex":2,"cellIndex":3,"cellParaIndex":0}],
 "plane":2,"zOrder":0,"stableIndex":19}
```

### 계획서 명시 리스크 점검 — 기존 image 경로 성립 확인

- `findPictureBbox`: 컨트롤 레이아웃 재순회 방식 — 방출된 bbox 로 성립.
- `getObjectProperties`(image + cellPath →
  `get_cell_picture_properties_by_path_native`): 미지정 그림에서 정상 반환
  실측 (`{"width":5669,"height":5669,"treatAsChar":true,...}`) — 모델의
  Picture 컨트롤 조회이므로 bin 데이터 불요.
- 레이어 JSON: `cell_context` serde skip + paint JSON 은 control_ref
  미방출 — 지문/캐시(#2222) 무영향.

### 게이트

- `cargo fmt --all -- --check` 통과 / clippy(release-test all-targets) 경고 0
- `cargo test --profile release-test --tests --no-fail-fast`:
  **3054 passed / 0 failed** (golden svg_snapshot 포함)

## 다음 단계

2단계 — `assign_picture_image_native` 커맨드 (BinData 등록 규칙 헬퍼 추출
공유, bin_data_id 갱신, 틀 크기 유지, 캐시 무효화) + wasm API 노출.
