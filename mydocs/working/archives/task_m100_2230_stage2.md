# 2단계 완료보고서 — Task M100 #2230: 그림 지정 커맨드

- 이슈: #2230 / 구현계획서: `task_m100_2230_impl.md` / 브랜치: `local/task2230`
- 작성일: 2026-07-12

## 수행 내용

기존 Picture 컨트롤에 이미지를 지정하는 커맨드를 신설하고 wasm 으로 노출했다.

### 변경 파일 2개

1. **`src/document_core/commands/object_ops/picture.rs`**
   - `register_embedded_bin_data(image_data, extension) -> u16` 헬퍼 추출:
     `insert_picture_native` 의 BinData 등록 1·2절(콘텐츠 push + 메타데이터
     push + storage id 최댓값+1 채번 + raw_stream 리셋)을 그대로 옮기고
     원 함수는 호출로 대체 — **규칙 단일 원천 확보** (동작 불변).
   - `assign_picture_image_native(section, parent_para, cell_path, control,
     image_data, natural_w_px, natural_h_px, extension)` 신설:
     - **대상 검증을 BinData 등록보다 먼저** 수행 — 실패 시 문서 무변형.
     - 대상 탐색: cell_path 비면 `resolve_picture_control_ref/_mut`(본문·
       미주), 있으면 `resolve_cell_paragraph_mut`(셀/글상자 — 기존
       set_cell_picture_properties_by_path_native 와 동일 패턴).
     - 갱신: `image_attr.bin_data_id` = 신규 위치 id, `external_path` 소거,
       crop = 원본 전체(natural px × 75 HU@96dpi — insert 와 동일 규약).
       **개체 틀 크기(common.width/height)·배치 속성은 유지** — 한컴
       placeholder 는 틀에 그림을 맞추므로 레이아웃 불변.
     - 후처리: `raw_stream=None` → `recompose_section` →
       `paginate_if_needed` → `invalidate_page_tree_cache`(#2222 레이어
       JSON 캐시 연쇄 무효화 확인 완료) → `PictureResized` 이벤트.
     - 반환: `{"ok":true,"binDataId":N}`.

2. **`src/wasm_api.rs`** — `assignPictureImage(sectionIdx, parentParaIdx,
   cellPathJson, controlIdx, imageData, naturalWidthPx, naturalHeightPx,
   extension)` 노출. cellPath 파싱은 insertPicture 와 동일 규약(빈
   문자열/"[]" = 본문).

## 검증

### 표적 테스트 확장 (`tests/issue_2230_placeholder_selection.rs`, +2건 = 총 4건)

- `assign_picture_image_converts_placeholder_to_image`: 심볼 placeholder
  (cellPath [{2,3,0}], ci=0)에 1×1 PNG 지정 → ①반환 ok+binDataId ②컨트롤
  레이아웃에서 missing 마커 소멸 ③image 컨트롤 2건(로고+심볼) ④심볼 자리
  (x≈646.2)에 **틀 크기(75.6px) 유지**된 image 컨트롤 존재.
- `assign_picture_image_invalid_target_leaves_document_unchanged`: 범위
  초과 컨트롤/빈 이미지 → 오류 + **레이아웃 완전 무변형** (BinData 선등록
  방지 검증).

참고: 테스트는 wasm 래퍼가 아닌 `assign_picture_image_native` 를 직접
호출한다 — wasm 래퍼의 오류 경로는 JsValue 생성으로 비-wasm 타겟에서
abort 하기 때문 (기존 object_ops 테스트들과 동일 관행).

### 게이트

- `cargo fmt --all -- --check` 통과 / clippy(release-test all-targets) 0
- `cargo test --profile release-test --tests --no-fail-fast`:
  **3056 passed / 0 failed** (1단계 3054 + 신규 2)

## 다음 단계

3단계 — studio UI: `picHit.missing` image 선택 상태에서 더블클릭 → 파일
선택 → snapshot undo 패턴 + `wasm.assignPictureImage` 호출 → 재렌더.
