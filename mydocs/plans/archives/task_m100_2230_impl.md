# 구현계획서 — Task M100 #2230: placeholder 선택 + 그림 삽입 편집 기능

- 이슈: #2230 / 수행계획서: `task_m100_2230.md` (승인됨) / 브랜치: `local/task2230`
- 작성일: 2026-07-12
- 검증 문서: `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생
  알림(화재번호 2026-177).hwpx` — 1페이지 "심볼" 필드 (bin_id=0 미지정 그림)

## 정찰 실증 (구현 전 확정 사실)

1. **studio 개체 선택의 데이터 소스는 `getPageControlLayout`** 이다.
   `findPictureAtClick`(input-handler-picture.ts:135)가
   `wasm.getPageControlLayout(pageIdx)` 의 `controls[]` 를 순회해 bbox
   hit-test 한다. 커서 쪽이 아니라 이 경로가 개체 선택의 본선이다.
2. **미지정 그림은 현재 이 컨트롤 목록에서 빠진다** (실측: 36389312 페이지 0
   컨트롤 5개 = table 4 + image 1(좌측 로고), 심볼 placeholder 부재).
   원인: `get_page_control_layout_native` 의 Placeholder 분기
   (rendering.rs:2097)가 `control_ref.kind == "ole"` 만 방출.
3. **placeholder 렌더 노드에 문서 좌표가 없다**. `missing_picture()`
   생성자(render_tree.rs:404)가 `control_ref: None` 으로 고정. 방출 지점
   2곳(picture_footnote.rs 의 layout_picture_full :170대,
   layout_body_picture :470대)에는 `section_index / para_index /
   control_index / cell_ctx` 가 모두 스코프에 있음 (바로 아래 ImageNode
   생성부가 동일 값 사용) — 배선만 하면 된다.
4. **셀 안 그림의 hit 에는 cellPath 가 필요**하다. Image 분기는
   `image_node.cell_context`(CellContext) 로 `cellIdx/cellParaIdx/cellPath`
   를 방출 — placeholder 도 동일 필드가 필요 (심볼 필드는 표 셀 안).
5. **BinData 추가 규칙은 `insert_picture_native`(picture.rs:1280)에 완비** —
   storage id 채번·확장자 처리 재사용 가능.
6. **undo 는 studio TS 스냅샷 패턴** (`executeOperation({kind:'snapshot',
   ...})` + wasm 커맨드, input-handler-keyboard.ts:1714 그림 삽입과 동일) —
   Rust 측 undo 스택 작업 불필요.

## 단계 구성 (4단계)

### 1단계 — 선택 가능화: placeholder 에 문서 좌표 배선 + hit-test 소스 방출 (Rust)

- `PlaceholderNode`:
  - `missing_picture()` → `missing_picture(section_index, para_index,
    control_index, cell_context)` 로 변경. `control_ref =
    Some(ObjectControlRef { kind: "picture", .. })` 채움.
  - `cell_context: Option<CellContext>` 필드 추가 (Image 노드 대칭.
    직렬화 제외 — 레이어 JSON 부풀림 방지, 컨트롤 레이아웃에서만 사용).
- 방출 2곳(layout_picture_full / layout_body_picture)에서 스코프의
  section/para/control_index + cell_ctx 전달.
- `get_page_control_layout_native` Placeholder 분기 확장:
  `kind == "picture"` 이면 **`type:"image"` 컨트롤로 방출** + Image 분기와
  동일한 `cellIdx/cellParaIdx/cellPath` + **`"missing":true` 마커**.
  → `findPictureAtClick` 는 기존 'image' 로직 그대로 hit — 클릭 선택·선택
  테두리·핸들이 TS 무수정으로 성립. `missing` 마커는 3단계 진입 판별용.
- 표적 테스트 신설 `tests/issue_2230_placeholder_selection.rs`:
  36389312 페이지 0 컨트롤 레이아웃에 x≈646 image(missing=true, cellPath
  동반) 존재 단언. **수정 전 FAILED 실증(patch-revert 방식) 후 커밋.**

### 2단계 — 그림 지정 커맨드 (Rust + wasm)

- `assign_picture_image_native(section_idx, para_idx, control_idx,
  cell_path, image_data, natural_w_px, natural_h_px, extension)` 신설
  (object_ops/picture.rs):
  - cell_path 로 대상 Picture 컨트롤 탐색 (기존 컨트롤 탐색 헬퍼 재사용).
  - BinData 등록은 insert_picture_native 의 규칙을 헬퍼로 추출해 공유.
  - `picture.image_attr.bin_data_id` 갱신 + `shape_attr.original_width/
    height` 는 natural px→HU 기록. **틀 크기(선언 w/h)는 유지** — 한컴
    placeholder 는 틀에 그림을 맞추는 동작이므로 레이아웃 불변.
  - 재조판 무효화: `invalidate_page_tree_cache_from` + layer_tree_json_cache
    (#2222) 연동 — 기존 커맨드들의 무효화 패턴 그대로.
- wasm_api 에 `assign_picture_image(...)` 노출.
- 표적 테스트 확장: 지정 후 렌더 트리에서 placeholder 소멸 + Image 노드
  등장 + bin_data_id > 0 단언.

### 3단계 — studio UI: 선택 → 더블클릭 → 파일 선택 → 지정 (TS)

- 클릭 선택은 1단계로 자동 성립 (기존 개체 선택 UI 재사용).
- `picHit.missing === true` 인 image 선택 상태에서 **더블클릭** 시 파일
  선택(`<input type="file" accept="image/*">`) 진입 — 기존 그림 삽입 흐름
  (input-handler-keyboard.ts insertPicture 패턴)의 파일 읽기·자연 크기
  측정 로직 재사용.
- `executeOperation({kind:'snapshot', operationType:'assignPictureImage',
  ...})` + `wasm.assignPictureImage(...)` → 재렌더. undo/redo 는 스냅샷
  패턴으로 자동 정합.
- 미지정이 아닌 일반 그림의 더블클릭 동작(개체 속성)은 불변 — `missing`
  마커로만 분기.

### 4단계 — 저장 왕복 + 게이트 + 실사용 판정

- 코어 왕복 테스트: 지정 → HWPX 저장 → 재파싱에서 그림 유지(BinData
  매니페스트 포함), HWP(5) 저장 경로도 동일 검증.
- 게이트: fmt --check / clippy 0 / `cargo test --profile release-test
  --tests --no-fail-fast` / golden svg_snapshot 8 / OVR 5샘플(±2px,
  샘플별 분리 출력 폴더) / studio `npm ci` + build + tsc.
- WASM 빌드 → **작업지시자 실사용 판정**: 심볼 placeholder 클릭 선택 →
  더블클릭 → 그림 선택 → 표시 → 저장 → 재열기 유지.

## 위험/주의

- 컨트롤 레이아웃에 placeholder 를 'image' 로 추가하면
  `getObjectProperties`/`setObjectProperties`/`deleteObjectControl` 등
  기존 image 경로가 함께 노출된다 — 대상이 실존 Picture 컨트롤이므로 모델
  조회는 성립하지만, 1단계에서 속성 대화상자 열림/삭제 동작을 점검하고
  이상 시 `missing` 가드로 제한한다.
- 레이어 JSON 캐시(#2222): placeholder 는 이미 레이어 트리에 있으므로
  1단계는 컨트롤 레이아웃만 변화 — 캐시 지문 무영향. 2단계 지정은 문서
  변형이므로 기존 무효화 훅 경유 확인.
- OLE placeholder(kind="ole") 기존 동작 불변 — 분기 추가만, 수정 없음.
