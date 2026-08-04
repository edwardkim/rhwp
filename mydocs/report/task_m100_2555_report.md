# task_m100_2555 처리결과 보고서 — 이웃 셀 BorderFill push 시 DocInfo 패스스루 무효화

- **이슈**: [#2555](https://github.com/edwardkim/rhwp/issues/2555)
- **브랜치**: `task/m100-2555-neighbor-border-docinfo-dirty` (base `devel` @ `3c54abfd`)
- **범위**: `src/document_core/commands/table_ops.rs` `update_neighbor_borders` 1줄 + 테스트
- **분류**: 결함 수정 (저장 시 dangling border_fill_id)

## 1. 문제

`update_neighbor_borders` 가 새 `BorderFill` 을 DocInfo 에 push 하면서 **DocInfo 패스스루를
무효화하지 않았다**. 이 함수는 섹션 스트림만 지우는데(`:971`), 섹션과 DocInfo 는 서로 다른
패스스루 계층이다.

직렬화 근거(`src/serializer/doc_info.rs:23-33`): `!raw_stream_dirty` 이고 `raw_stream` 이
`Some` 이면 **DocInfo 스트림 전체를 원본 그대로 반환**한다. 따라서 push 된 `BORDER_FILL`
레코드는 기록되지 않고 `ID_MAPPINGS` 도 여전히 N 개인데, 본문에는 `border_fill_id = N+1` 이
쓰인다(`:1347`) → 재열기 시 **범위 밖 id**.

## 2. 분석 — 규약이 아니라 누락

형제 호출부는 모두 무효화한다. 즉 이 코드가 규약을 벗어난 것이다.

| 호출부 | 무효화 |
|---|---|
| `object_ops/table.rs:451` | `raw_stream = None` |
| `html_table_import.rs:769-770` | `raw_stream_dirty = true` |
| `html_import.rs:923, 988` | `raw_stream_dirty = true` |
| **`table_ops.rs:1337`** | **없음 ← 결함** |

도달 가능성: `create_border_fill_from_json` 은 dedup 히트에서 dirty 를 세우지 않고 조기
반환한다(`html_table_import.rs:761-765`). 실제 .hwp 는 BORDER_FILL 을 10개 이상 갖고 있어
dedup 히트가 흔하므로, "대상 셀은 dedup 히트(무효화 없음) + 이웃 셀은 새 조합(push)" 조합이
자연스럽게 발생한다.

## 3. 변경

`table_ops.rs` push 직후 `self.document.doc_info.raw_stream_dirty = true;` 1줄 추가.

## 4. 검증

### red→green 실증

`neighbor_border_push_marks_doc_info_dirty` 추가 — 파싱된 문서 상태(`raw_stream = Some(..)`,
`raw_stream_dirty = false`)를 재현하고 새 조합을 만든 뒤, BorderFill 이 push 되었고
`raw_stream_dirty` 가 섰는지 단언한다.

- 수정 제거 시 → **FAILED** (`DocInfo 패스스루가 무효화되지 않으면 …` 로 panic)
- 복원 시 → **2 passed**

> 검증 과정 기록: 처음에 `perl` 로 수정을 제거해 RED 를 확인하려 했으나 저장소가 CRLF 라
> `$` 앵커가 매칭되지 않아 **수정이 제거되지 않은 채 통과**했다. 이를 오탐 신호로 보고
> 프로덕션 코드의 `raw_stream_dirty = true` 발생 개수를 세어 원인을 특정한 뒤, 편집기로
> 정확히 제거해 RED 를 재확인했다.

### 회귀

`cargo test --lib document_core` → **255 passed / 0 failed**.

### 미실행 항목 (투명 고지)

- **저장→재파싱 왕복 테스트 미추가.** 이슈 본문에 적었던 "export → from_bytes → 이웃 셀 id 가
  범위 내로 해소" 까지 확인하려면 유효한 CFB 왕복 픽스처가 필요하다. 본 PR 은 결함의 직접
  원인인 무효화 누락을 단언하는 데 그쳤다(그 지점이 red→green 으로 갈리는 유일한 지점이다).
  왕복 하네스까지 원하시면 별도로 진행하겠다.
- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약상
  작업지시자 별도 승인 사항이라 실행하지 않았다.
