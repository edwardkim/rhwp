# task_m100_4326 Stage 1 — PartialTable 행 좌표계를 데이터로 고정

- **이슈**: [#4326](https://github.com/edwardkim/rhwp/issues/4326)
- **PR**: [#4374](https://github.com/edwardkim/rhwp/pull/4374)
- **브랜치**: `fix/issue-4326-partial-table-row-coords`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 + Skia 3종 + wasm-pack + 시각 증적 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 결함 — 값으로 되추론했다

투명 1×1 래퍼 안에 든 표는 페이지네이션이 래퍼를 벗기고 **중첩 표의 행**으로 커서를 잡을 수 있다.
그런데 `PageItem::PartialTable`(`renderer/pagination.rs:488`)은 `start_row`/`end_row`/`start_cut`/
`end_cut` 만 실을 뿐 **그 값이 어느 좌표계인지 담지 않았다.**

렌더러가 값으로 되추론했다(`layout/table_partial.rs:80` 부근):

```rust
fn fragment_row_geometry_table(table: &Table, end_row: usize) -> &Table {
    if end_row <= table.row_count as usize { table } else { transparent_nested_table(table) }
}
```

**중첩 표의 진짜 단일 행 조각(`end_row == 1`)은 "바깥 래퍼의 자기 행 0"과 값으로 구분되지 않는다.**
래퍼를 벗겨야 할 때 안 벗기고, 표가 두 쪽에 중복 인쇄되며 본문 영역을 넘는다.

## 2. 재현

`samples/76076_regulatory_analysis.hwp` para 36(3×3 중첩 표를 감싼 투명 1×1 RowBreak 래퍼),
`page_def.margin_bottom` +500 HWPUNIT:

```
수정 전: LAYOUT_OVERFLOW: page=5, type=PartialTable, y=1104.8, bottom=1040.2, overflow=64.5px
         p5 에 세 행 전부 재출력, p6 에 "현행유지안" 중복 인쇄
수정 후: overflow 없음. p5 = 제목행만, p6 = 나머지 두 행
```

`git stash` 로 수정을 껐다 켜서 양방향 확인했다.

## 3. 구현 — 결정 시점에 데이터화

`PageItem::PartialTable` 에 `row_cursor_is_nested: bool` 추가. 값은 페이지네이션 결정 시점에 포인터
비교로 계산한다(`typeset.rs`, `step_block_table_continuation`):

```rust
let row_cursor_is_nested = !std::ptr::eq(row_geometry_table, table);
```

`fragment_row_geometry_table` **삭제**. 플래그를 `PageItem` → `layout_partial_table_item` →
`layout_partial_table` → `layout_partial_table_resolved` 로 관통 전달하고, 캐럿 fast-path
(`cursor_rect.rs`)도 같은 필드를 받는다.

**핵심 적대 검증**: `row_geometry_table` 은 `hwp5_single_cell_rowbreak_wrapper` 분기에서 `table`
**자신**을 대입한다(`typeset.rs:19406-19410`). 따라서 네이티브 RowBreak 래퍼가 자기 프레임을 지켜야
하는 경로(#1921 p16, #3637 HWP 2020 p7)는 `ptr::eq` 가 참 → 플래그 `false` → 종전 동작 그대로다.
값 기반 되추론이 구분하지 못하던 것을 결정 자체가 기록한다.

fallback `Paginator`(`pagination/engine.rs`, `RHWP_USE_PAGINATOR=1`)도 같은 이중 표 모델을 공유해
같은 결함이 잠재해 있어 함께 채웠다.

`PageItem` 은 `#[derive(Debug)]` 뿐이라 필드 추가에 직렬화·캐시 호환성 영향이 없음을 확인했다.

## 4. 시각 증적 — 두 경로 각각

처음에는 옵션 없는 `export-svg` 대조만 하고 "시각 증적"이라 적었는데, 그것이 paint 계층을 거치지
않는 legacy 경로임이 드러나(→ #4379) 표현을 좁히고 두 경로를 각각 다시 쟀다.

| 경로 | 명령 | 결과 |
|---|---|---|
| legacy(렌더 트리 직행) | `export-svg` | 82/82 바이트 동일 |
| paint 계층 | `export-svg --profile print` | 82/82 바이트 동일 |

baseline 은 `upstream/devel` `e48fe8694`. 무변경 문서에서 렌더 변화가 없다 — 결함은 쪽 기하가
흔들릴 때만 드러난다.

## 5. 범위 밖

fallback `Paginator` 의 기존 발산(#1921 p11/p36)은 손대지 않았다 — `git stash` 로 이 변경을 뺀
상태에서도 동일하게 실패하는 사전 결함이다.

## 6. 검증 (완료)

- 회귀 테스트 `tests/issue_4326_partial_table_nested_row_coords.rs` 신설(baseline + margin+500).
  수정 전 코드에서 실패함을 확인했다.
- `cargo test --profile release-test --tests` 전체 통과. `table` 필터 635건 통과.
- Native Skia 3종 통과, `wasm-pack build` 성공.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.
- 시각 증적 두 경로(위 4절).

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
