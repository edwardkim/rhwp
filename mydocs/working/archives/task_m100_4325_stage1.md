# task_m100_4325 Stage 1 — table.dirty clear 범위 한정 + 무효화 플래그 kill 규칙

- **이슈**: [#4325](https://github.com/edwardkim/rhwp/issues/4325), [#4335](https://github.com/edwardkim/rhwp/issues/4335)
- **PR**: [#4360](https://github.com/edwardkim/rhwp/pull/4360)
- **브랜치**: `fix/issue-4325-table-dirty-scope`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 결함 — 지우는 범위와 소비하는 범위가 어긋났다

`table.dirty` 를 **소비**하는 곳은 구역 범위다. `rendering.rs:4037` 의
`if !self.dirty_sections[idx] { … continue; }` 가 dirty 아닌 구역의 측정을 건너뛰고, 건너뛴 구역의
표는 `measure_section_incremental`(`height_measurer.rs:2325`)이 아직 `!table.dirty` 로 소비하지
않은 상태다.

**지우는** 곳은 문서 범위였다. 같은 패스 끝에서 `for section in &mut self.document.sections` 로
전 구역의 `table.dirty = false` 를 지웠다 — 방금 건너뛴 구역까지.

## 2. 실측

2구역 문서, 각 구역에 동일한 2×2 표, 스타일 글자모양 36pt:

```
BEFORE   구역0 h=55.52   구역1 h=55.52
AFTER    구역0 h=487.52  구역1 h=55.52   ← stale
재실행·본문편집 후에도 구역1 = 55.52. 그 표를 직접 편집해야만 복구.
```

셀 텍스트는 새 글꼴로 그려지는데 행 높이는 옛 값이라 `clip: true` 에 글자가 잘리고, 쪽 나눔도 옛
높이로 계산돼 저장·인쇄·PDF 까지 틀린다.

## 3. 두 선택지 중 (a) 를 골랐다

- **(a)** clear 를 이번 패스에서 재측정한 구역으로 한정 ← **채택**
- (b) `mark_cell_control_dirty` 가 구역도 함께 dirty 로 만들게 함

(b) 는 `update_style_shapes` 한 호출자의 증상만 가린다. clear 루프 자체가 "건너뛴 구역까지
지운다"는 결함을 안고 있어, `table.dirty = true` 를 직접 세우는 나머지 사이트 중 구역 dirty 를
세우지 않는 경로가 추가되면 같은 결함을 물려받는다.

저장소 관례도 (a) 다 — `rendering.rs:3815` 와 `:3826-3832` 가 이미 같은 스코핑을 쓴다.

## 4. 구현

`paginate_pass` 에 `remeasured_sections: Vec<bool>` 를 두고, 측정을 마치는 지점
(`dirty_sections[idx] = false` 옆)에서 표시한다. 건너뛴 구역은 그 지점에 도달하지 않아 `false` 로
남는다. 패스 말미 clear 루프가 이 플래그를 보고 건너뛴다.

플래그가 남는 방향은 안전하다 — 다음 패스에서 한 번 더 측정할 뿐 stale 을 읽지 않는다.

**적대 검증**: `paginate_pass` 안에서 `document.sections` 길이를 바꾸는 코드가 없음을 확인해
`remeasured_sections` 인덱스 범위 초과가 불가능함을 근거로 남겼다.

## 5. #4335 — 같은 결함의 일반형

개별 버그가 아니라 **규칙 부재**였다. 같은 파일 안에서도 두 방식이 공존했다(`:3815` 는 한 구역만,
`:4627` 은 전 구역).

`mydocs/tech/rendering_engine_design.md` §12 뒤에 규칙을 기록했다 — *"무효화 플래그의 clear 는 그
플래그를 읽는 모든 지점의 immediate dominator 에서만 수행한다."*

플래그 6종 + revision 3단계를 전수 감사했고 **새 위반은 없었다.** `Table.dirty_flag` 는 HWPX
`<hp:tc dirty>` 라운드트립 속성이라 제외했다.

후속 후보로 남긴 것: `RenderNode.dirty` 는 `RenderNode::new` 가 항상 `true` 로 세우는데
`has_dirty_nodes`/`mark_clean`/`needs_render` 의 프로덕션 호출부가 0건이고, `#[serde]` skip 이
없어 모든 노드가 JSON 에 `dirty:true` 를 싣고 나간다.

## 6. 검증 (완료)

- 회귀 테스트 `issue_4325_style_update_second_section_table_not_left_stale` 신설. 수정 전 코드에서
  `left: 1207.52` vs `right: 114.19` 로 실패함을 확인했다.
- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.
- 문서: `check_markdown_links.py`, `check_document_metadata.py`, `git diff --check` 통과.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
