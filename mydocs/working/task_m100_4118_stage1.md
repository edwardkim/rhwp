# Task M100 #4118 — 셀 뮤테이터 O(n²) 배치 지연 재페이지네이션

## 무엇을

셀 블록 전체 서식·속성 적용(`applyParaFormatInCell`·`applyCharFormatInCell`·
`setCellProperties`를 셀 수만큼 호출)이 표가 커질수록 제곱으로 느려지는 결함
(#4118, 1000셀 정렬 ~5.2초 · 1500셀 12.8초)을 고친다.

## 왜 — 근본 원인

- 코어에는 이미 두 가지 지연 계약이 있었다.
  - `batch_mode` 플래그와 이를 존중하는 `paginate_if_needed()`(rendering.rs),
    `begin_batch_native`/`end_batch_native`(document.rs), wasm 표면
    `beginBatch`/`endBatch`.
  - 셀 **텍스트** 편집의 deferred pagination(#2424): `mark_section_pagination_dirty`
    만 하고 paginate 를 flush 로 미룬다.
- 그런데 셀 **서식** 뮤테이터 꼬리는 이 게이트들을 우회해 매 호출
  `rebuild_section`(resolve_styles + recompose_section + paginate 전체 패스)을
  했다. studio 는 beginBatch 를 한 번도 쓰지 않으므로 호출 수 × O(문서) = O(n²).
- setCellProperties 는 이미 `paginate_if_needed`(table_ops.rs)라 배치로만 묶으면
  되었고, 서식 뮤테이터는 꼬리 자체가 계약 밖이었다.

## 어떻게

1. **코어**(formatting.rs): 공통 꼬리 헬퍼
   `rebuild_section_deferred_in_batch(sec)` 추가 — batch_mode 면
   `resolve_styles` 즉시 갱신(새 서식 id 반영, O(스타일 수))+`mark_section_dirty`
   로 미루고, 아니면 종전대로 전체 rebuild. 다음 6곳의 꼬리를 이 헬퍼로 교체:
   - `apply_char_format_in_cell_native`
   - `apply_char_format_in_cell_by_path` (깊이≥2 분기)
   - `apply_para_format_in_cell_native`
   - `apply_cell_style_native` (두 분기)
   - `set_char_shape_id_in_cell_by_path` (undo 복원 루프가 셀 수만큼 부른다)
2. **studio**: bridge 에 `runInBatch(fn)`(begin/endBatch try-finally 묶음, 낡은
   pkg 호환 폴백) 추가하고 블록 적용 루프 5곳을 묶음:
   - input-handler: `applyCharFormatToCellBlock`, `applyCopiedCellPropsToSelection`
   - command: `ApplyCharFormatCommand.execute`(셀 분기), `ApplyParaFormatCommand.execute`
   - cell-border-bg-dialog: 전체/선택 범위 스코프 루프

비배치 단일 호출 경로는 동작이 완전히 동일하다(else 분기가 종전 코드).

## 검증 실측

- 신규 통합 테스트 `tests/cases/issue_4118_cell_format_batch_deferral.rs`:
  실제 샘플 문서의 본문 표 전체 셀에 서식 3종 적용 시 — 배치 여부와 무관하게
  **쪽 수·1쪽 지오메트리(getPageInfo)·1쪽 SVG 렌더 바이트·저장 바이트가 동일**.
- `cargo test --lib`: 3,893 passed / 0 failed
- `cargo clippy --lib -- -D warnings`: clean
- `cargo fmt --all -- --check`: pass
- manifest `--prepare`/`--check`: pass(신규 source 등록)
- `rust-unit-test-tiers --check`: 4225 유지(source-side 테스트 증가 없음 —
  신규 테스트는 tests/cases 원본으로만 추가)
- rhwp-studio `npm test`: 1,065 passed / 0 failed / 1 skipped(기존 skip)
- tsc: 변경 파일 오류 없음(devel 원래 존재하는 5건 제외)

## 성능 실측

네이티브 release 빌드, 이슈 프로브와 동일 형태(샘플 문서에 100×10 표 생성,
셀마다 "AB" 입력, 1000셀 × `apply_para_format_in_cell({"alignment":"center"})`):

| 경로 | 총 시간 | 호출당 |
|---|---|---|
| 종전(호출마다 rebuild_section) | **5.854s** | 5.854ms |
| 수정(begin_batch~end_batch) | **0.130s** | 0.130ms |

- **약 45배 단축**, 호출당 비용이 셀 수에 더 이상 비례하지 않는다(평탄).
- 두 경로의 결과는 동일하다 — 쪽 수 동일(assert). 저장 바이트·1쪽 지오메트리
  동일성은 통합 테스트 `issue_4118_cell_format_batch_deferral` 이 보장한다.
- WASM 브라우저 수치는 리뷰 환경에서 wasm-pack 재빌드 후 이슈 프로브로
  재측정해 첨부할 것(이슈 보고자의 재측정 관례 준용).

## 잔여 / 후속

- body 경로 서식 뮤테이터(`apply_para_format_native` 등 formatting.rs 잔여
  rebuild_section 꼬리)와 `restoreCharShapeIds` undo 루프의 배치 묶음 — 동일
  클래스이나 블록 루프 표적이 아니어서 이번 범위에서 제외.
- 중첩 표 블록 적용은 applyCharFormatToCellBlock 의 byPath 경로가 함께 묶인다.
