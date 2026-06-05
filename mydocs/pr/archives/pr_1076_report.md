# PR #1076 재검토 보고 — set_field_text_at 메타데이터 불일치

## 1. 현재 결정

**수용 가능 / maintainer 보정 포함.**

컨트리뷰터가 2026-05-27 추가 코멘트로 제시한 `FieldRange` 인덱스 좌표계 설명은
로컬 코드 확인 결과 타당하다. 이전 보고서의 "FieldRange는 `PARA_HEADER.char_count`
기준이고 `delete_text_at`은 순수 텍스트 기준이라 서로 불일치한다"는 진단은 철회한다.

다만 PR head에서 생성한 `output/field-01-modified.hwp`는 한컴 에디터에서 파일손상 판정을 받았다.
rhwp-studio에서는 정상 로드되므로, 한컴이 더 엄격하게 보는 HWP5 record contract 문제가 남아 있었다.
maintainer 보정 후보에서 Field `CTRL_DATA` 중복을 제거한 뒤 한컴 수동 판정을 통과했다.

| 항목 | 값 |
|------|-----|
| 번호 | #1076 |
| 제목 | fix: set_field_text_at 메타데이터 불일치 — ClickHere 필드 값 설정 시 파일 손상 (#838) |
| 작성자 | oksure (Hyunwoo Park) |
| base ← head | `devel` ← `contrib/fix-field-text-corruption` |
| 현재 head | `b214b1fe` |
| 상태 | PR open, maintainer 보정 포함 수용 가능 |

## 2. 추가 코멘트 검증

컨트리뷰터 주장:

```text
FieldRange.start_char_idx/end_char_idx는 para.text.chars()와 같은 좌표계다.
FIELD_BEGIN/FIELD_END 같은 확장 컨트롤은 parse_para_text의 local char_count를 증가시키지 않는다.
```

로컬 확인:

`src/parser/body_text.rs`의 `parse_para_text`는 다음처럼 동작한다.

```text
FIELD_BEGIN(0x0003): field_stack.push((char_count, ctrl_idx)); ctrl_idx += 1
FIELD_END(0x0004): field_ranges.push(start_char_idx, end_char_idx=char_count)
일반 텍스트/탭/개행: text.push(); char_count += 1
확장 컨트롤: pos += 16, char_count 미증가
```

따라서 `FieldRange.start_char_idx/end_char_idx`는 HWP `PARA_HEADER.char_count`
좌표가 아니라 `para.text.chars()` 기준 문자 인덱스다.

## 3. PR head 검증

검증 브랜치:

```text
local/pr1076-review
```

실행:

```text
cargo test set_field_value_removes_guide_text
cargo test diag_issue838_save_and_inspect -- --nocapture
cargo test test_issue838_two_field_setvalue_roundtrip -- --nocapture
```

결과:

```text
set_field_value_removes_guide_text: pass
diag_issue838_save_and_inspect: pass
test_issue838_two_field_setvalue_roundtrip: pass
```

진단 테스트 산출물:

```text
output/field-01-modified.hwp
```

작업지시자 판정:

```text
한컴 에디터: 파일손상
rhwp-studio: 정상
```

## 4. 한컴 파일손상 원인

`hwp5-ctrl-data-trace`로 정답지와 PR head 산출물을 비교했다.

```text
output/poc/pr1076-field/ctrl_data_trace.md
```

핵심 차이:

| side | CTRL_DATA records | total bytes |
|---|---:|---:|
| oracle `samples/field-01.hwp` | 11 | 198 |
| PR head generated `output/field-01-modified.hwp` | 16 | 290 |

PR head generated 파일은 Section0의 ClickHere Field `CTRL_HEADER` 아래에 같은 `CTRL_DATA`를 두 번 쓴다.

예:

```text
BodyText/Section0/PARA_HEADER#33/CTRL_HEADER#37/CTRL_DATA#38
BodyText/Section0/PARA_HEADER#33/CTRL_HEADER#37/CTRL_DATA#39
```

원인:

```text
1. Paragraph.ctrl_data_records에 보존된 원본 CTRL_DATA를 serialize_control() 마지막에서 복원
2. Control::Field 직렬화가 field.ctrl_data_name으로 CTRL_DATA를 다시 합성
```

rhwp-studio는 중복 `CTRL_DATA`를 관대하게 읽지만, 한컴 에디터는 Field control의 자식 record contract
위반으로 보고 파일손상 판정을 내리는 것으로 해석한다.

## 5. maintainer 보정 후보

Field 직렬화에서 원본 `ctrl_data_record`가 이미 전달된 경우에는 `field.ctrl_data_name` 기반
합성 `CTRL_DATA`를 추가하지 않도록 조정했다.

```text
src/serializer/control.rs
  - ClickHere Field CTRL_DATA 합성 조건에 ctrl_data_record.is_none() 추가
```

함께 정리한 항목:

```text
src/document_core/queries/field_query.rs
  - field_range_index 재조회 실패를 InvalidField 에러로 명시
  - rebuild_char_offsets 주석 정확화

src/wasm_api/tests.rs
  - output 파일 생성용 진단 테스트 제거

tests/issue_838_field_set_value.rs
  - DocumentCore 기반 회귀 테스트 2개로 정리
  - RHWP_ISSUE838_OUT 지정 시 한컴 판정용 파일 생성 가능
```

## 6. maintainer 후보 검증

실행:

```text
cargo fmt --all -- --check
cargo test --test issue_838_field_set_value
RHWP_ISSUE838_OUT=output/poc/pr1076-field/field-01-modified-candidate.hwp \
  cargo test --test issue_838_field_set_value \
  set_field_value_roundtrips_two_empty_click_here_fields -- --nocapture
target/debug/rhwp hwp5-ctrl-data-trace \
  samples/field-01.hwp \
  output/poc/pr1076-field/field-01-modified-candidate.hwp \
  --out output/poc/pr1076-field/ctrl_data_trace_candidate.md
```

결과:

```text
fmt: pass
issue_838_field_set_value: 2 passed
candidate generation: pass
```

후보 파일의 CTRL_DATA 비교:

| side | CTRL_DATA records | total bytes |
|---|---:|---:|
| oracle `samples/field-01.hwp` | 11 | 198 |
| candidate `field-01-modified-candidate.hwp` | 11 | 198 |

후보 파일에서는 PR head 산출물에 있던 Field `CTRL_DATA` 중복이 제거되었다.

## 7. 한컴 판정 게이트

현재 필요한 수동 확인 파일:

```text
output/poc/pr1076-field/field-01-modified-candidate.hwp
```

작업지시자 판정 항목:

| file | 한컴 열기 | 파일손상 | 첫 필드 이후 출력 | 필드 값 표시 | 비고 |
|---|---|---|---|---|---|
| `output/poc/pr1076-field/field-01-modified-candidate.hwp` | 성공 | 없음 | 성공 | 성공 | maintainer 보정 후보 |

한컴에서 정상 판정이면 PR #1076은 컨트리뷰터 변경 위에 maintainer 보정 커밋을 얹어 수용하는 방향이 적절하다.

## 8. 최종 결론

```text
2026-05-27 maintainer 수동 판정 통과
2026-05-27 한컴2020 편집기 정상 동작 확인
```

PR #1076의 핵심 방향은 타당하다. 단, PR head 그대로는 Field `CTRL_DATA` 중복으로 한컴 파일손상이 발생하므로
다음 보정까지 함께 반영한다.

```text
1. set_field_text_at은 para.text 직접 교체 대신 delete_text_at/insert_text_at 위임 유지
2. FieldRange 갱신 후 char_offsets 재생성 유지
3. Field CTRL_DATA는 원본 ctrl_data_record가 있을 때 중복 합성하지 않음
4. wasm_api의 산출물 생성용 진단 테스트는 제거하고 integration 회귀 테스트로 유지
```
