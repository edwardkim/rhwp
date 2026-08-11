# #4582 Stage 1 — `set_document` 의 측정 캐시 누락 재현과 수정

- **Issue**: [#4582](https://github.com/edwardkim/rhwp/issues/4582)
- **브랜치**: `fix/renderer-measurement-unification`
- **기준**: `upstream/devel` `4f9e4ae69`

## 1. 재현 — 된다 (표·문단 둘 다)

이슈가 "확인이 먼저다"라고 요구한 실험을 그대로 했다. 같은 core 에 표를 가진 문서 둘을 연달아
넣고, **빈 core 에 같은 문서를 넣었을 때의 측정값**을 정답으로 잡았다.

RED 실행 (수정 전, `cargo test --profile release-test --lib set_document_tests`):

```
thread '...set_document_does_not_reuse_previous_documents_measured_table' panicked at
  src/document_core/commands/document.rs:2439:9:
assertion `left == right` failed: set_document 뒤 표 높이가 이전 문서의 측정값(160)을 재사용했다
  left: 160.0
 right: 53.333333333333336

thread '...set_document_does_not_reuse_previous_documents_measured_paragraph' panicked at
  src/document_core/commands/document.rs:2484:9:
assertion `left == right` failed: set_document 뒤 문단 높이가 이전 문서의 측정값(21.333333333333332)을 재사용했다
  left: 21.333333333333332
 right: 0.0

test result: FAILED. 0 passed; 2 failed
```

6행 표 문서를 넣은 core 에 2행 표 문서를 넣으면 표 높이가 **160.0px(6행 값) 그대로** 나온다.
정답은 53.33px 다.

### 1.1 기전 — `dirty_paragraphs` 가 clean 으로 남아 있다

이슈 본문은 `dirty_paragraphs = None` → `measure_section_incremental` → `!table.dirty` 표 재사용
경로를 지목했다. 실제 경로는 한 칸 더 나쁘다.

`paginate_pass` 는 매 패스 끝에서 `dirty_paragraphs[idx] = Some(vec![false; para_count])` 로
비트맵을 **전부 clean 으로** 되돌린다(`queries/rendering.rs:4619`). `set_document` 는 이 비트맵을
건드리지 않으므로, 두 번째 문서의 첫 페이지네이션은 `dirty_paras = Some(전부 false)` 로
`measure_section_selective` 에 들어간다. 거기서는

- 문단: `!is_dirty` → `prev_measured.fallback_paragraphs[para_idx]` 를 그대로 복사,
- 표: `!table.dirty` → `prev_measured.get_measured_table(para_idx, ctrl_idx)` 를 그대로 복사

한다. 즉 **새 문서의 문단 수 이하 구간 전체가 이전 문서의 측정값**이다. 새 문서의 문단이 더 많은
경우에만 초과분이 `unwrap_or(true)` 로 dirty 취급돼 재측정된다.

`table.dirty` 는 파싱·생성 직후 `false` 가 기본값이다(`model/table.rs:66` "Default: false").
"실무에서 `!table.dirty` 가 안 선다" 는 가설은 성립하지 않는다.

### 1.2 영향 범위

`set_document` 는 `pub` 이고 프로덕션 호출부는 `HwpDocument::create_empty`(`wasm_api.rs:595`)
하나인데, 거기서는 `DocumentCore::new_empty()` 직후라 캐시가 비어 있어 증상이 나지 않는다.
나머지 호출부는 전부 테스트다. 그래도 **API 계약이 틀린 것**이라 고친다 — "이 메서드는 빈 core
에만 안전하다"는 제약이 이름에도 문서에도 없다.

## 2. 수정 — 파생 상태 재구성을 한 이름으로 모은다

`set_document` 는 전면 재구성 순서를 손으로 적은 **세 번째 사본**이었다. 누락 여섯 줄을 손으로
채우면 사본이 셋으로 남는다. 그래서 단일 정의를 도입하고 두 호출부를 거기로 보냈다.

```rust
// queries/rendering.rs — 단일 정의
pub(crate) fn rebuild_derived_state(&mut self) { … }

// commands/document.rs
pub fn set_document(&mut self, doc: Document) {
    self.document = doc;
    self.bump_bin_data_epoch();
    self.rebuild_derived_state();
}
pub fn restore_snapshot_native(&mut self, id: u32) -> Result<String, HwpError> {
    …
    self.bump_bin_data_epoch();
    self.rebuild_derived_state();
    …
}
```

`bump_bin_data_epoch` 는 호출부에 남긴다 — "문서를 통째로 갈아끼웠다"는 사실은 파생 상태 재구성이
아니라 그 상위 연산의 성질이다(`bin_data_epoch` doc 이 그 셋을 명시한다).

### 2.1 #4584(#4576) 와의 관계 — 예측대로 rebase 됐다

작업 시작 시점의 `upstream/devel`(`4f9e4ae69`)에는 `rebuild_derived_state` 가 없었고, PR
#4584(브랜치 `origin/fix/issue-4576-subsecond-invalidation`)가 미머지 상태로 같은 이름·같은 본문의
메서드를 `queries/rendering.rs` 의 `mark_all_sections_dirty` 바로 뒤에 도입하고
`restore_snapshot_native` 를 그 호출로 바꾸고 있었다. 이 커밋은 **그 정의를 글자 그대로 같게** 넣고
두 호출부를 모두 그리로 보냈다 — 어느 쪽이 먼저 머지돼도 정의가 하나로 수렴하게 하려는 것이다.

작업 중 #4584 가 devel 에 머지됐고, 예측한 그대로 됐다.

```
$ git rebase upstream/devel
CONFLICT (content): Merge conflict in src/document_core/queries/rendering.rs
```

충돌은 **doc 주석 한 문장뿐**이었다(devel 은 "스냅샷 복원 + 핫패치", 이 커밋은
"`set_document` + 스냅샷 복원"). 메서드 본문은 같아서 충돌하지 않았고,
`restore_snapshot_native` 헝크는 이미 적용된 상태라 통째로 흡수됐다. 해소는 두 문장을 합친
"`set_document`·스냅샷 복원·핫패치" 한 줄이다.

rebase 뒤 이 커밋에 남은 실질은 예고한 대로다.

```
$ git show --stat HEAD
 mydocs/working/task_m100_4582_stage1.md | 122 +++++++++++
 src/document_core/commands/document.rs  | 131 +++++++++++++---
 src/document_core/queries/rendering.rs  |   4 +-
```

`rendering.rs` 4줄은 위 doc 주석 문장이고, `document.rs` 는 `set_document` 본문 6줄 + doc + RED
테스트 두 개다. **`rebuild_derived_state` 정의는 하나다.**

## 3. 검증

GREEN 실행:

```
$ cargo test --profile release-test --lib set_document_tests
running 2 tests
test ...set_document_does_not_reuse_previous_documents_measured_paragraph ... ok
test ...set_document_does_not_reuse_previous_documents_measured_table ... ok
test result: ok. 2 passed; 0 failed
```

## 4. 이번 범위 밖에서 본 것 (고치지 않음)

`DocumentCore::recompose_and_paginate`(`queries/rendering.rs:2465`)와 `set_section_def_native`
안의 같은 인라인 사본(`:2818`)은 문서 IR 을 **제자리 편집**한 뒤 전 구역을 recompose 하고
`mark_all_sections_dirty()` 만 부른다. `dirty_paragraphs` 는 직전 패스가 남긴
`Some(vec![false; n])` 그대로라, 이어지는 `measure_section_selective` 가 **옛 단 폭에서 잰 문단
측정값을 전부 재사용**한다. 단 폭이 바뀌는 `set_section_def_native` 에서는 그 재사용이 틀리다.
지금은 프로덕션 페이지네이션이 문단 측정값을 읽지 않아(#4605) 증상이 폴백·진단에 갇혀 있다.
