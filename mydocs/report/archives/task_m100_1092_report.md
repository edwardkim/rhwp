# Task M100-1092 완료 보고서

## 1. 이슈

GitHub Issue #1092:

```text
[hwpx2hwp] aift.hwpx 메모 컨트롤 HWP 저장 직렬화
```

## 2. 목표

`samples/hwpx/aift.hwpx`를 HWP로 저장할 때 HWPX 메모 컨트롤을 한컴 HWP5 메모 record contract에
맞게 직렬화한다.

비교 기준:

```text
source:    samples/hwpx/aift.hwpx
oracle:    samples/aift.hwp
baseline:  saved/111aift.hwp
```

## 3. 문제

기존 rhwp-studio 저장 결과는 메모 컨트롤을 온전하게 저장하지 못했다.

관찰된 문제:

```text
1. 한컴 에디터에서 Runtime Error R6025 pure virtual function call 발생
2. R6025 해결 후에도 메모 anchor 표시 스타일이 정답지와 다름
3. 첫 번째 메모가 개선된 뒤에도 두 번째 메모 스타일은 계속 풀림
```

## 4. 원인

문제는 단일 record 누락이 아니라 HWP5 메모 저장 계약의 여러 축이 동시에 필요한 형태였다.

확정한 계약:

```text
1. HWPX fieldBegin type="MEMO"는 HWP5 본문에서 %unk CTRL_HEADER로 저장해야 한다.
2. CTRL_HEADER properties에는 0x8001 패턴이 필요하다.
3. 마지막 구역 끝에는 MEMO_LIST 컨테이너와 메모 본문 paragraph list를 저장해야 한다.
4. MEMO_LIST 하위 메모 본문 문단에는 PARA_LINE_SEG를 쓰지 않는다.
5. HWPX charPr shadow offsetX/offsetY는 shadow type이 NONE이어도 보존해야 한다.
6. memo FIELD_END는 일반 %%me end marker가 아니라 한컴 전용 memo end marker를 사용해야 한다.
7. memo FIELD_END marker의 6번째 code unit에는 해당 memo index를 기록해야 한다.
```

최종 문제의 핵심:

```text
Stage 6에서는 FIELD_END marker의 memo index가 1로 고정되어 있었다.
따라서 첫 번째 메모는 정답지와 맞았지만, 두 번째 메모는 CTRL_HEADER memo_index=2와
PARA_TEXT FIELD_END memo_index=1이 불일치했다.
```

## 5. 수정

주요 수정 파일:

```text
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
src/model/control.rs
src/serializer/control.rs
src/serializer/body_text.rs
src/document_core/queries/field_query.rs
src/parser/control.rs
```

진단 도구:

```text
src/diagnostics/hwp5_anchor_trace.rs
src/diagnostics/mod.rs
src/main.rs
```

적용 내용:

```text
1. HWPX MEMO fieldBegin의 Number/ID/Command를 HWP5 Field 모델에 보존
2. HWPX MEMO subList 문단을 Field.memo_paragraphs로 보존
3. 마지막 구역 끝에 MEMO_LIST + LIST_HEADER + memo paragraph list 직렬화
4. MEMO field CTRL_HEADER를 %unk + properties 0x8001 형태로 직렬화
5. memo anchor fieldBegin/fieldEnd range를 Paragraph.field_ranges로 materialize
6. charPr shadow offsetX/offsetY 파싱 보존
7. memo FIELD_END marker를 한컴 정답지 패턴으로 직렬화
8. memo FIELD_END marker에 메모별 index를 기록
```

## 6. 산출물

최종 판정 파일:

```text
output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp
```

단계 보고서:

```text
mydocs/working/task_m100_1092_stage1.md
mydocs/working/task_m100_1092_stage2.md
mydocs/working/task_m100_1092_stage3.md
mydocs/working/task_m100_1092_stage4.md
mydocs/working/task_m100_1092_stage5.md
mydocs/working/task_m100_1092_stage6.md
mydocs/working/task_m100_1092_stage7.md
```

수행 계획서:

```text
mydocs/plans/task_m100_1092.md
```

## 7. 검증

실행한 검증:

```text
cargo fmt
cargo fmt --check
cargo check
cargo test -q test_parse_memo_field_begin_uses_id_as_hwp5_field_id
cargo test -q test_parse_memo_field_parameters_preserves_number_as_memo_index
cargo test -q test_parse_field_begin_end_materializes_field_range
cargo test -q test_parse_char_pr_preserves_shadow_offsets_even_when_shadow_is_none
cargo test -q test_memo_field_end_uses_hancom_marker_tail
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp
```

결과:

```text
success
```

Raw record 검증:

```text
cargo run --quiet --bin rhwp -- hwp5-anchor-trace \
  output/poc/hwpx2hwp/task1092/stage7_memo_field_end_index/aift-memo-field-end-index.hwp \
  --section 2 --needle 공동기관2 --window 4 \
  --out /tmp/aift_stage7_anchor2_trace.md
```

정답지와 Stage 7 후보의 두 번째 메모 anchor 비교:

| file | PARA_TEXT hash | CTRL_HEADER hash | memo index |
|---|---|---|---:|
| `samples/aift.hwp` | `5cffb518e5814aca` | `46a646257c075cf6` | 2 |
| `stage7/aift-memo-field-end-index.hwp` | `5cffb518e5814aca` | `46a646257c075cf6` | 2 |

## 8. 시각 판정

작업지시자 판정:

```text
이번 구현은 성공
```

확인 의미:

```text
1. 첫 번째 메모가 정답 HWP와 동일하게 출력됨
2. 두 번째 메모도 기존 문제 없이 정답 HWP와 동일하게 출력됨
3. memo FIELD_END index contract가 한컴 에디터 기준으로 유효함
```

## 9. 분리한 후속 문제

이번 이슈는 메모 컨트롤 저장 스타일을 해결했다.

다음 문제는 별도 이슈로 분리한다.

```text
1. aift 저장 HWP의 2페이지 표/셀 높이 차이
2. 한컴 에디터와 rhwp-studio의 페이지 배치 차이
3. rhwp-studio의 한컴 메모 렌더링 미지원
```

## 10. 결론

#1092는 HWPX memo field를 HWP5의 `%unk` memo field + MEMO_LIST + memo FIELD_END index contract로
정확히 materialize해야 하는 문제였다.

최종 해결의 결정적 조건은 memo `FIELD_END` marker에 메모별 index를 기록하는 것이다. 이 값이 `1`로
고정되면 첫 번째 메모만 정상 표시되고, 두 번째 이후 메모 스타일은 풀린다.

Stage 7 판정으로 메모 컨트롤 HWP 저장 직렬화는 완료 처리할 수 있다.
