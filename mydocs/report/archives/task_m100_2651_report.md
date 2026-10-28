# task_m100_2651 처리결과 보고서 — 선택 영역 사각형의 중첩 표 셀 오매칭

- **이슈**: [#2651](https://github.com/edwardkim/rhwp/issues/2651)
- **브랜치**: `task/m100-2651-selection-rects-nested-cell` (base `devel`)
- **범위**: `src/document_core/queries/cursor_nav.rs`
- **분류**: 결함 수정 (중첩 표에서 잘못된 셀 선택)

## 1. 문제

`get_selection_rects_native`(선택 영역 사각형 계산)의 셀 매칭이 `cell_context.path` 의
**첫 항목만** 비교했다.

```rust
let matches_cell = tr.cell_context.as_ref().map_or(false, |ctx| {
    ctx.path.first().map_or(false, |entry| {
        ctx.parent_para_index == ppi
            && entry.control_index == ci
            && entry.cell_index == cei
            && entry.cell_para_index == cpi
    })
});
```

중첩 표 **내부** 셀의 TextRun 은 `path = [바깥 셀, 안쪽 셀]` 이고, 그 `path[0]` 이
**중첩 표를 품은 바깥 셀과 정확히 같다.** 따라서 바깥 셀을 질의해도 내부 run 들이 전부
매칭된다. `cpi` 비교 역시 바깥 셀의 문단 슬롯과 비교하는 것이라 형제 내부 셀들을
구분하지 못한다.

이는 이미 수정된 `cursor_rect.rs` 의 동일 클래스 버그(`path[0]`-only 매칭)와 같은
근본 원인이다. 같은 파일의 `cell_context_matches` 헬퍼가 `ctx.path.len() == path.len()`
를 계약으로 명시하고 있어, 설계 모호성이 아니라 계약 위반이다.

### 영향

`getSelectionRectsInCell`/`getSelectionRectsInCellEx` 로 중첩 표 안에서 드래그 선택을
시작·종료하면, 트리 순회 순서상 먼저 나온 아무 중첩 셀의 사각형이 선택되어 **엉뚱한 셀
위에 선택 영역이 그려진다.**

## 2. 분석 — 최소 안전 수정을 택한 이유

`get_selection_rects_native` 의 `cell_ctx` 파라미터 자체가 평면 3-튜플
`Option<(usize, usize, usize)>` 이라 **애초에 중첩 셀 경로 전체를 표현할 수 없다**
(이미 고친 `cursor_rect.rs` 의 리졸버들이 `Vec<CellPathEntry>` 전체를 받는 것과 대조적).

따라서 이번 수정은 API 를 넓히지 않고 `path.len() == 1` 가드만 추가한다. 효과:

- **틀린 셀을 잘못 고르는 것**(현재 결함) → 막힘
- **중첩 셀을 정확히 겨냥하는 것**(기존에도 불가) → 여전히 "못 찾음" 으로 안전하게 떨어짐

중첩 셀을 실제로 겨냥하려면 `cell_ctx` 를 `Vec<CellPathEntry>` 로 넓히고 wasm 진입점
(`getSelectionRectsInCell`/`Ex`)까지 바꿔야 하므로, 범위를 섞지 않기 위해 분리했다.

## 3. 변경

매칭 술어를 `flat_cell_ctx_matches` 헬퍼로 추출하고 `ctx.path.len() == 1` 가드를 추가했다.
헬퍼로 뺀 이유는 **단위 테스트 가능성** — 원래 위치가 메서드 내부의 중첩 함수라 직접
호출할 수 없었다.

## 4. 검증

### 신규 테스트 (`flat_cell_ctx_matches_tests`, 3건)

1. `matches_direct_single_level_cell` — 단일 depth 셀은 정상 매칭
2. `rejects_nested_cell_sharing_the_same_outer_path_entry` — **핵심**: `path[0]` 이
   질의와 동일한 중첩 셀 run 을 거부
3. `rejects_mismatched_outer_indices` — 인덱스 불일치 거부

### red→green 실증

가드(`ctx.path.len() == 1 &&`)를 제거하고 실행:
```
rejects_nested_cell_sharing_the_same_outer_path_entry ... FAILED
  assertion failed: !flat_cell_ctx_matches(&ctx, 0, 1, 2, 3)
test result: FAILED. 2 passed; 1 failed
```
가드 복원 후:
```
test result: ok. 3 passed; 0 failed
```
**중첩 셀 테스트만 정확히 실패**했고 나머지 2건은 통과 — 가드가 의도한 지점만 잡는다는
증거다.

### 회귀

```
cargo test --lib document_core::  →  257 passed / 0 failed / 2 ignored
```

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`)은 저장소
  규약상 작업지시자 별도 승인 사항이라 실행하지 않았다.
- 렌더 트리를 구성한 **행위 수준 E2E** 는 넣지 않았다. 이 매처는 메서드 내부 중첩
  함수라 직접 호출이 불가능했고, 그래서 술어를 헬퍼로 추출해 **결함의 핵심 판정
  로직을 직접 단위 테스트**하는 방식을 택했다. 렌더 트리 하네스 구축은 범위를 넘는다고
  판단했다.

## 5. 잔여

`cell_ctx` 를 `Vec<CellPathEntry>` 로 확장해 중첩 셀을 정확히 겨냥하는 작업은 wasm
진입점 변경을 수반하므로 별도 이슈로 분리한다.
