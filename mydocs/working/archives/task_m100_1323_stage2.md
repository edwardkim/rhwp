# Stage 2 보고서 — Task M100-1323

- 이슈: #1323
- 작성일: 2026-06-10
- 브랜치: `local/task1323`

## 1. 작업 요약

Stage 1의 `merge_from` 보강이 실제 붙여넣기·병합 경로에서 end-to-end로 동작함을
통합 테스트로 고정했다. 표 셀/글상자/캡션/path 기반 셀의 그림 붙여넣기, 본문/셀
백스페이스 병합의 컨트롤 보존, HWP5 직렬화 round-trip을 검증한다. 소스 수정은
테스트·주석 한정이며 런타임 코드 변경은 없다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/wasm_api/tests.rs` | #1323 통합 테스트 6건 + `find_table_pos` 헬퍼 추가 |
| `src/document_core/commands/object_ops.rs` | `paste_picture_into_textbox` 테스트 추가, L8772 "별개 결함" 주석을 해소 사실로 갱신 |

## 3. 추가된 통합 테스트

| 테스트 | 경로 | 검증 내용 |
|--------|------|----------|
| `test_paste_picture_into_table_cell` | `paste_internal_in_cell_native` (표 셀) | 그림 컨트롤 보존 + CTRL_DATA 인덱스 정렬 보존 |
| `test_paste_picture_into_cell_by_path` | `paste_internal_in_cell_by_path_native` | path 기반 셀 경로 동일 보존 |
| `test_paste_picture_into_picture_caption` | `paste_internal_in_cell_native` (Picture 캡션 분기) | 캡션 문단 그림 보존 |
| `paste_picture_into_textbox` (object_ops) | `paste_internal_in_cell_native` (글상자 분기) | `insert_picture_native`(BinData 등록)→복사→글상자 붙여넣기 보존. #1280 텍스트 테스트와 나란히 배치 |
| `test_merge_paragraph_preserves_controls` | `merge_paragraph_native` (본문 백스페이스) | 컨트롤 보유 문단 병합 시 그림 보존 + `control_text_positions()==[0]` 위치 유지 |
| `test_merge_paragraph_in_cell_preserves_controls` | `merge_paragraph_in_cell_native` (셀 백스페이스) | 셀 문단 병합 시 그림 보존 |
| `test_paste_picture_into_table_cell_hwp5_roundtrip` | `export_hwp_native` → `from_bytes` | 셀에 붙여넣은 그림이 HWP5 직렬화·재파싱 후 보존 (char_count 역산·갭 인코딩 정합 실증) |

## 4. 구현계획서 항목별 처리

- ✅ 표 셀/글상자/캡션 round-trip — 위 테스트로 고정
- ✅ 중첩(path 기반) 셀 — `by_path` 경로 테스트 (단일 레벨 path로
  `get_cell_paragraphs_mut_by_path` + 동일 병합 함수 경유 검증)
- ✅ 백스페이스 병합 부수 해소 — 본문/셀 2건
- ✅ 본문 무회귀 — 기존 `test_paste_cascade_floating_picture`,
  `test_paste_inline_picture_no_cascade` 등 paste 테스트 15건 통과
- ✅ HWP5 저장 round-trip — 통과 (직렬화 계약 정합 실증)
- ✅ object_ops.rs:8772 주석 갱신 — "별개 결함" 인지 주석을 해소 사실 + 회귀
  테스트 참조로 교체
- ☑️ `reflow_cell_paragraph` 보정 — **보정 불필요로 판단**:
  `reflow_line_segs`(`renderer/composer/line_breaking.rs:902`)는 폰트 크기
  기반으로 line_segs를 계산하고, 셀 내부의 인라인 컨트롤 치수는 렌더 시점의
  셀 레이아웃이 처리한다(파싱 문서의 셀 내 그림이 정상 렌더되는 기존 동작과
  동일 경로). Stage 3 시각 검증으로 최종 확인한다.

## 5. 검증

통과:

- `cargo test --lib paste_picture` — 5 passed (신규)
- `cargo test --lib test_merge_paragraph` — 2 passed (신규)
- `cargo test` 전체 — **2138 passed, 0 failed** (lib + 통합 + doctest)
- `cargo clippy --all-targets` — 무경고
- `cargo fmt --check` (변경 파일 한정) — 통과

## 6. 비고

- Stage 3에서 WASM 빌드 후 rhwp-studio 실 UI(이미지 복사→글상자/표 셀 붙여넣기)
  시각 검증과 떠 있는 개체(treat_as_char=false) 셀 anchor 렌더링을 확인한다.
