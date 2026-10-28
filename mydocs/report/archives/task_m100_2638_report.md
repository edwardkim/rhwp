# task_m100_2638 처리결과 보고서 — update_neighbor_borders DocInfo passthrough 무효화

- **이슈**: [#2638](https://github.com/edwardkim/rhwp/issues/2638)
- **브랜치**: `task/m100-2638-neighbor-border-invalidate` (base `devel` @ `3c54abfd`)
- **범위**: `src/document_core/commands/table_ops.rs` 한 지점
- **분류**: 결함 수정 (dangling border_fill_id 방지)

## 1. 문제

`set_cell_properties` 가 이웃 셀 테두리를 갱신하는 `update_neighbor_borders`(`table_ops.rs:1337`)
가 새 `BorderFill` 을 DocInfo 에 push 하면서 **DocInfo passthrough 를 무효화하지 않았다**.
섹션 스트림만 지웠고(`:971`), 섹션과 DocInfo 는 서로 다른 passthrough 레이어다.

`src/serializer/doc_info.rs:24-33` — `!raw_stream_dirty && raw_stream.is_some()` 이면 DocInfo
스트림 전체를 **원본 그대로** 반환하므로, 방금 push 한 `BORDER_FILL` 레코드는 저장에서 통째로
사라지고 `ID_MAPPINGS` 도 예전 개수를 유지한다.

형제 호출부 4곳(`html_table_import.rs:769-770,904-905`, `object_ops/table.rs:450-451,885-886`)은
모두 push 직후 무효화하므로, 이 지점만의 누락이었다.

## 2. 재현 시나리오

`create_border_fill_from_json` 이 dedup 히트 시 무효화 없이 조기 반환하는 경로가 있어(범위 밖):
1. DocInfo 에 이미 일치하는 테두리 스타일이 있는 문서를 연다
2. 셀 A 에 그 스타일 적용 → dedup 히트(무효화 없음)
3. 이웃 셀 B 의 결합 테두리(A 의 새 변 + B 의 나머지 3변)는 새 조합이라 `update_neighbor_borders`
   가 push → `table.cells[B].border_fill_id = N+1` 이 본문에 기록됨
4. 저장 → DocInfo 는 N 개만 있음 → 재로드 시 이웃 셀이 **범위 밖 border_fill_id** 를 가짐

## 3. 변경

`table_ops.rs:1337` push 직후 `self.document.doc_info.raw_stream_dirty = true;` 추가.

## 4. 검증

### 신규 테스트

`neighbor_border_raw_data_tests::neighbor_border_push_invalidates_doc_info_passthrough` — 파싱된
문서를 흉내(`raw_stream = Some(...)`, `raw_stream_dirty = false`)내고 `update_neighbor_borders`
호출 후 `raw_stream_dirty == true` 를 단언.

### red→green 실증

수정 라인(`raw_stream_dirty = true;`)을 제거 → **FAILED**(panic at assert). 복원 → **2건 통과**.

```
FAILED (fix 제거 시): 0 passed; 1 failed
GREEN (fix 복원 후):  2 passed; 0 failed
```

### 회귀

```
cargo test --lib document_core::commands::table_ops  →  5 passed / 0 failed
```

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약상
  작업지시자 별도 승인 사항이라 실행하지 않았다.
- `create_border_fill_from_json` 의 dedup-hit 무효화 보강(이슈 본문의 "보강" 항목)은 **본 PR
  범위에 포함하지 않았다** — 이슈에서 지목한 핵심 누락(`table_ops.rs:1337`)만 최소 수정했다.
  필요시 별도 이슈로 처리한다.
