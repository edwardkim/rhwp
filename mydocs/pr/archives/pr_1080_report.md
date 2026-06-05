# PR #1080 처리 결과 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1080>
- 제목: `fix: set_cell_field_text 직접 대입 → delete+insert 사용 (#838 관련)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/838>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. 처리 결과

| 항목 | 결과 |
|---|---|
| 처리 방식 | cherry-pick 수용 |
| cherry-pick 원본 | `6f6b594f029a6f50c97177fbd080b86ae763b7ba` |
| local commit | `13cf3429 fix: set_cell_field_text 메타데이터 불일치 — #838 동일 패턴` |
| 추가 조치 | `69ec4f44 test: cover cell field text metadata update` |
| 관련 이슈 #838 | 유지, 별도 close하지 않음 |
| PR #1076 | 유지, 별도 close하지 않음 |

## 2. 반영 내용

PR #1080은 셀 field_name 기반 가상 필드 값 설정 경로에서 직접 대입을 제거했다.

기존 구현:

```rust
cell_para.text = value.to_string();
rebuild_char_offsets(cell_para);
```

반영 구현:

```rust
let old_len = cell_para.text.chars().count();
if old_len > 0 {
    cell_para.delete_text_at(0, old_len);
}
if !value.is_empty() {
    cell_para.insert_text_at(0, value);
}
rebuild_char_offsets(cell_para);
```

직접 대입은 `char_count`, `char_offsets`, `char_shapes`, `line_segs`, `range_tags` 같은 문단
메타데이터를 새 텍스트 길이에 맞게 갱신하지 못한다. `delete_text_at` / `insert_text_at` 경로를
사용하면 문단 내부 메타데이터 시프트 로직을 통과한다.

## 3. 추가 회귀 테스트

다음 테스트를 추가했다.

```text
set_cell_field_text_updates_text_metadata
```

검증 내용:

```text
1. table cell field_name="셀필드"를 가진 셀 첫 문단을 구성한다.
2. 기존 텍스트 "기존값"을 "새값"으로 교체한다.
3. text, char_count, char_offsets가 새 텍스트 기준으로 갱신되는지 확인한다.
```

이 테스트는 #1080 수정의 핵심인 "텍스트 교체 후 문단 메타데이터 갱신"을 직접 고정한다.

## 4. #1076 / #838 처리 판단

PR #1080은 #1076과 다르게 `FieldRange.start_char_idx/end_char_idx`를 사용하지 않는다.

```text
PR #1076:
  FieldRange char_count 기준 인덱스를 순수 text offset으로 사용해 실패

PR #1080:
  셀 field_name 기반 가상 필드의 첫 문단 전체 텍스트 교체
  delete_text_at(0, text.chars().count()) 사용
```

따라서 #1076에서 확인된 offset 의미 불일치가 PR #1080에는 직접 적용되지 않는다.

다만 #838은 #1076과 #1080을 모두 포함하는 넓은 범위의 필드 편집 안정성 이슈다.
그러므로 PR #1080 반영만으로 #838을 close하지 않는다.

## 5. 검증

실행한 검증:

```text
cargo fmt --check
cargo check
cargo test test_task230_get_field_value
cargo test test_task230_set_field_value
cargo test set_cell_field_text_updates_text_metadata
```

결과:

```text
success
```

기존 warning은 유지되었고, 이번 변경으로 새 실패는 발생하지 않았다.

## 6. 후속 조치

남은 절차:

```text
1. 결과 보고서 승인
2. PR #1080 처리 문서 커밋
3. 필요 시 PR #1080에 cherry-pick 수용 코멘트 작성 후 close
4. devel 동기화 전 전체 검증 및 wasm 빌드
```
