# Task m100-#2825: 중첩 표 최내곽 셀 붙여넣기(cellPath) 재래핑 보강

## 이슈

[#2825](https://github.com/edwardkim/rhwp/issues/2825) 중첩 표 최내곽 셀 붙여넣기(cellPath)가 셀 폭으로 재래핑되지 않음

## 근거

`src/document_core/commands/clipboard.rs`의 `paste_internal_in_cell_by_path_native`
(붙여넣기 대상이 `cellPath`로 지정되는 깊이 ≥2 중첩 표/글상자/캡션 셀 경로)는
`paste_paragraphs_into_cell_paragraphs`로 텍스트/문단을 삽입한 뒤 곧바로
`mark_cell_control_dirty` → `mark_section_dirty` → `paginate_if_needed`로 넘어갔다.
문단의 `line_segs`(줄바꿈 캐시)를 최내곽 셀 폭으로 다시 계산하는 호출이 전혀 없었다.

같은 파일의 형제 함수(깊이 1 flat 버전) `paste_internal_in_cell_native`는 붙여넣기
직후 명시적으로

```rust
for i in cell_para_idx..=last_para_idx {
    self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, i);
}
```

를 호출해 셀 폭 기준 재래핑을 수행한다. 또한 `src/document_core/commands/text_editing.rs`의
`delete_text_in_cell_by_path`는 이슈 #2755에서 정확히 같은 클래스의 문제(깊이 ≥2 중첩
셀은 flat `reflow_cell_paragraph`로 재래핑할 수 없음)를 인지하고 path 전용
`reflow_cell_paragraph_by_path`(`text_editing.rs:1354`, `pub(crate)`)를 도입해 이미
고쳐졌다. 그러나 이 헬퍼가 `paste_internal_in_cell_by_path_native`에는 적용되지 않은
채 남아 있었다 — #2755가 삭제/분할/병합/서식 경로에는 반영됐지만 붙여넣기 경로는
누락됐다.

## 재현

바깥 표(셀 폭 5000 HWPUNIT, 넉넉함) 문단 안에 안쪽 표(셀 폭 200 HWPUNIT, 좁음)를
중첩시키고, 안쪽 최내곽 셀에 40자 텍스트를 내부 클립보드로 붙여넣으면(`cellPath`
경로 API), 셀 폭 200 기준으로는 여러 줄로 재래핑돼야 하지만 실제로는
`insert_text_at` 직후의 미보정 상태(`line_segs.len() == 0`, 즉 재래핑이 전혀
수행되지 않은 원본 빈 문단 상태)가 그대로 남았다.

### red → green

- 수정 전(재래핑 호출 임시 제거) 테스트 결과: `실제 0줄` → 패닉.
- 수정 후: `line_segs.len() > 1` 통과.

```
test document_core::commands::clipboard::nested_cell_paste_reflow_tests::paste_in_nested_cell_by_path_reflows_inner_cell ... ok
```

## 수정

`paste_internal_in_cell_by_path_native`에서 붙여넣기 직후, 형제 delete 경로와 동일한
패턴으로 붙여넣기가 영향을 준 문단 범위(`cell_para_idx..=last_para_idx`)를
`reflow_cell_paragraph_by_path`로 재래핑한다.

## 영향 범위

- `src/document_core/commands/clipboard.rs`만 수정. 다른 워크트리/파일은 건드리지 않음.
- 이 브랜치는 `reflow_cell_paragraph_by_path` 헬퍼를 제공하는 선행 커밋
  (`task/m100-2755-cell-bypath-reflow`, PR #2821)이 이미 로컬에 존재하는 워크트리 상태
  위에서 작업했다. 즉 본 PR은 #2821 위에 스택된다 — #2821이 머지되기 전에는 devel
  기준 diff에 #2821의 변경도 함께 표시될 수 있다.

## 검증

- `cargo build --lib`: 성공.
- `cargo test --lib paste_in_nested_cell_by_path_reflows_inner_cell`: red→green 확인.
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음.
- `rustfmt --edition 2021` 적용 후 `git diff --name-only`에 서식 변경으로 인한 추가
  파일 없음(clipboard.rs만 변경).
