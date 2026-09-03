# Task M100 #4771 구현 계획 — derived layout ownership

- Issue: #4771
- 수행 계획: `mydocs/plans/task_m100_4771.md`
- 브랜치: `ir/derived-layout-state`
- 작성일: 2026-09-03

## 불변식

1. HWP/HWPX 저장기는 같은 함수가 돌려준 source LineSeg만 직렬화한다.
2. renderer가 만든 suffix, 측정 dirty bit, 단일줄 overflow memo와 local resize projection은
   `Document`의 영속 의미로 취급하지 않는다.
3. #2004 projection은 같은 가로·세로 reference frame에서 양 축이 실제로 겹칠 때만 허용한다.
4. 저장 성공·실패·비밀번호·검증 API 호출 전후 live `Document`는 구조적으로 동일하다.
5. 안정된 section/paragraph는 기존 measured/page tree cache를 계속 재사용한다.

## Stage 1 — source LineSeg 단일 view

`Paragraph::serializable_line_segs()`를 중심 변경으로 둔다.

```rust
pub fn serializable_line_segs(&self) -> &[LineSeg] {
    let source_len = self.line_segs.len().saturating_sub(self.layout_only_fill_lines);
    &self.line_segs[..source_len]
}
```

HWP5와 HWPX가 이 view에 각 포맷의 text-axis 변환만 적용한다. `layout_only_fill_lines > 0`인 문서를
HWPX 저장·재파싱해 suffix가 파일 경계를 통과하지 않는 red→green 회귀를 둔다.

## Stage 2 — #2004 2D admission

`floating_image_stack_extents`가 다음을 모두 증명하도록 좁힌다.

- 모든 그림의 `horz_rel_to`, `vert_rel_to`, `horz_align`, `vert_align`이 같다.
- 가로 interval의 공통 교집합이 양수다.
- 세로 offset spread는 최소 그림 높이보다 엄격히 작다.

가로로 분리된 그림, 서로 다른 reference frame, 세로 경계값을 negative regression으로 둔다. 기존
#2004 HWP/HWPX는 8쪽뿐 아니라 page별 `ImageNode.bin_data_id`와 bbox를 고정한다.

## Stage 3 — renderer cache ownership

- `Table.dirty`를 제거하고 `dirty_paragraphs`를 measurement invalidation owner로 사용한다.
  `mark_cell_control_dirty`가 외곽 paragraph revision을 올리며, selective measurement는 dirty paragraph의
  모든 nested table을 재측정한다.
- `single_line_overflow_memo`를 source `Paragraph`에서 제거한다. Layout/measurement session이
  `(source paragraph identity, cell_inner_width)` key의 cache를 소유하고 source/style revision 경계에서
  일괄 clear하므로 mutation site의 수동 invalidation 호출을 없앤다.
- stored LineSeg partition provenance는 cache와 분리된 명시적 boolean으로 유지한다.

## Stage 4 — 표현 불가능한 local resize 경계

`local_resize_rows/cols/cell_widths/cell_heights`와 `inferred_local_resize_rows`를 제거한다. 완결된 행·열이
파일에 선언한 cell width 합으로 표 외곽과 닫히면 그 선언을 공유 grid evidence로 사용하고, 테이블
폭보다 큰 행은 format-validity outlier로 제외한다. 다만 HWP/HWPX에는 어느 행·열만 독립적으로 바꿨다는
편집 의도를 저장하는 슬롯이 없으므로 `localResize:true`는 source mutation 전에 명시적 오류로 반환한다.
일반 grid resize와 save/reopen 동작은 유지한다.

## Stage 5 — snapshot-only HWP lowering

`prepare_hwp_export_snapshot` 한 곳에서 `Document`를 clone하고 HWPX/HWP3 lowering, raw DocInfo 재밀봉을
수행한다. serializer와 CLI verify는 같은 `HwpExportSnapshot` instance를 소비한다. 기존 mutating API
이름은 호환을 위해 유지하되 live IR을 변경하지 않는다.
일반·password·verify 및 report 변형을 구조 동일성 테스트로 묶는다.

## 검증

- 새 integration source: `tests/cases/issue_4771_derived_layout_state.rs`
- 관련 회귀: #2004, #2308, #1491/#6557, #4325, #4677/#5847, HWP adapter tests
- `node scripts/rust-test-suite-manifest.mjs --prepare` 후 저장소 필수 Rust lint 묶음
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast`
- Native Skia 3종, WASM build
- `samples/issue2004_cell_image_stack.{hwp,hwpx}` OVR 및 page별 render tree 수동 판정

파생 suite와 manifest는 stage하지 않는다.
