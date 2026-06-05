# PR #1080 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1080>
- 제목: `fix: set_cell_field_text 직접 대입 → delete+insert 사용 (#838 관련)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/838>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `contrib/fix-cell-field-text-metadata` |
| head sha | `6f6b594f029a6f50c97177fbd080b86ae763b7ba` |
| mergeable | true |
| 작성자 | `oksure` |
| 변경 파일 | 1개 |
| 변경 범위 | `src/document_core/queries/field_query.rs` |

CI 확인:

| workflow | conclusion |
|---|---|
| CI | success |
| CodeQL | success |

## 2. PR 주장

PR #1080은 셀 필드 값 설정 경로에서 다음 직접 대입을 제거한다.

```rust
cell_para.text = value.to_string();
rebuild_char_offsets(cell_para);
```

직접 대입은 `char_shapes`, `line_segs`, `range_tags`, `char_count` 등 문단 메타데이터를
새 텍스트 길이에 맞게 시프트하지 못한다.

PR은 이를 다음 방식으로 바꾼다.

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

## 3. 현재 local/devel 상태

현재 `local/devel`의 `set_cell_field_text`에는 직접 대입 코드가 남아 있다.

```text
src/document_core/queries/field_query.rs:
  if let Some(cell_para) = cell.paragraphs.first_mut() {
      cell_para.text = value.to_string();
      rebuild_char_offsets(cell_para);
  }
```

따라서 PR #1080의 대상 경로는 아직 코드에 반영되지 않았다.

## 4. #1076과의 차이

PR #1076은 `set_field_text_at`에서 `FieldRange.start_char_idx/end_char_idx`를
`delete_text_at` / `insert_text_at`에 그대로 넘겼고, 한컴 수동 검증에서 실패했다.

실패 원인은 다음이었다.

```text
FieldRange.start_char_idx/end_char_idx:
  FIELD_BEGIN/FIELD_END 컨트롤 문자를 포함한 문단 char_count 기준

delete_text_at/insert_text_at:
  순수 text.chars() 기준 char offset
```

즉 인덱스 의미가 달랐다.

PR #1080은 이와 다르다.

```text
set_cell_field_text:
  셀 field_name 기반의 가상 필드
  셀 첫 문단 전체 텍스트를 교체
  old_len = cell_para.text.chars().count()
  delete_text_at(0, old_len)
  insert_text_at(0, value)
```

여기서는 `FieldRange` offset을 사용하지 않는다.
따라서 #1076에서 확인된 "FieldRange char_count 기준 offset을 순수 텍스트 offset으로 잘못 사용"한
문제는 PR #1080에는 직접 적용되지 않는다.

## 5. Copilot 피드백 검토

Copilot은 `delete_text_at` / `insert_text_at`가 `char_offsets`를 변경할 수 있으므로,
`rebuild_char_offsets`의 baseline이 흔들릴 수 있다고 지적했다.

코드 확인 결과:

```text
Paragraph::delete_text_at:
  char_offsets를 실제로 삭제/시프트한다.

Paragraph::insert_text_at:
  char_offsets를 실제로 삽입/시프트한다.
```

따라서 컨트리뷰터의 "delete_text_at/insert_text_at가 char_offsets를 직접 변경하지 않는다"는
답변은 현재 코드 기준으로는 정확하지 않다.

다만 PR #1080의 대상은 셀 `field_name` 기반 가상 필드이며, 셀 첫 문단 전체 텍스트를 교체하는
경로다. 일반적인 셀 필드 값 문단은 inline control/FieldRange를 포함하지 않는 plain text 문단으로
취급된다. 이 경우 `delete_text_at(0, old_len)` + `insert_text_at(0, value)`는 직접 대입보다
안전하다.

위험 조건:

```text
셀 field_name 대상 첫 문단에 inline control이 섞여 있는 경우
```

이 경우 `delete_text_at` 이후 `char_offsets`가 비고, `insert_text_at`이 `controls.len() * 8`을
baseline으로 삼을 수 있다. 현재 셀 field_name 가상 필드 경로에서 일반적이지는 않지만,
후속 테스트로 고정하는 것이 좋다.

## 6. 판단

PR #1080의 문제 제기는 타당하다.

현재 `local/devel`에 동일 수정은 반영되어 있지 않다.
그리고 #1076의 실패 원인이었던 `FieldRange` 인덱스 의미 불일치가 이 PR에는 직접 존재하지 않는다.

따라서 권장 처리는 다음과 같다.

```text
1. PR #1080 커밋을 cherry-pick한다.
2. 가능하면 셀 field_name 가상 필드의 값 교체 회귀 테스트를 추가한다.
3. cargo fmt --check, cargo check, 관련 테스트를 실행한다.
4. 필요 시 WASM 빌드 후 메인테이너 시각/동작 판정을 받는다.
```

## 7. 권장안

```text
권장: cherry-pick 수용
단, #1076은 계속 open/수정요청 상태로 유지한다.
이슈 #838은 #1076과 #1080을 모두 고려해야 하므로 PR #1080만으로 닫지 않는다.
```
