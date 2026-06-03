# Task M100-1092 Stage 4 작업 기록

## 1. 단계 목표

Stage 3 후보는 한컴 에디터에서 Runtime Error 없이 열리지만, 정답지 `samples/aift.hwp`와 비교하면
메모 앵커/메모 표시 스타일에 차이가 남았다.

작업지시자 시각 판정 요약:

```text
정답지:
  본문 왼쪽 메모 앵커 텍스트가 메모 범위처럼 표시되고,
  오른쪽 메모 박스에 메모 번호/라벨과 배경 스타일이 표시된다.

생성 HWP:
  본문 왼쪽 메모 앵커 범위 표시가 약하고,
  오른쪽 메모 표시 스타일이 정답지와 다르다.
```

따라서 Stage 4에서는 HWPX `fieldBegin`/`fieldEnd` 쌍이 HWP5 `PARA_TEXT`에 정확히 복원되는지 확인했다.

## 2. 원인

HWPX 파서는 `<hp:fieldEnd>`를 읽어 문단의 내부 위치 계산에는 반영하고 있었다.

하지만 모델의 `Paragraph.field_ranges`에는 메모 필드 범위를 남기지 않았다.
그 결과 HWP 저장 시 serializer가 `FIELD_END` 제어 문자를 다시 삽입할 근거가 없어졌다.

문제 문단 비교:

```text
정답지 samples/aift.hwp, section 2 para 482:
  cc=34, text_len=17, controls=1

기존 생성 HWP:
  cc=26, text_len=17, controls=1
```

차이 `8 code unit`은 HWP5 extended control 하나에 해당한다.
즉, 메모 본문 field marker는 존재하지만 본문 쪽 메모 범위 끝 `FIELD_END`가 빠져 있었다.

## 3. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
```

수정 내용:

```text
1. HWPX paragraph의 text_parts를 다시 순회한다.
2. FIELD_BEGIN marker(`0x0003`)를 만나면 현재 visible text index와 controls[] index를 stack에 저장한다.
3. FIELD_END marker(`0x0004`)를 만나면 stack에서 꺼내 `Paragraph.field_ranges`에 기록한다.
4. 기존 char_offsets/char_count 계산은 유지한다.
```

추가 테스트:

```text
test_parse_field_begin_end_materializes_field_range
```

검증 내용:

```text
HWPX fieldBegin + text + fieldEnd 구조가
Paragraph.field_ranges = [{ start=0, end=3, control_idx=0 }]
형태로 materialize되는지 확인한다.
```

## 4. 생성 후보

시각 판정 대상:

```text
output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp
```

생성 명령:

```text
cargo run --quiet --bin rhwp -- convert samples/hwpx/aift.hwpx output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp
```

## 5. 정답지 비교

문제 문단 `section 2 para 482` 비교:

| file | char count | text len | controls | field marker |
|---|---:|---:|---:|---|
| `samples/aift.hwp` | 34 | 17 | 1 | `%unk` CTRL_HEADER + MEMO command |
| `aift-field-range.hwp` | 34 | 17 | 1 | `%unk` CTRL_HEADER + MEMO command |

두 번째 메모 문단 `section 2 para 484` 비교:

| file | char count | text len | controls | field marker |
|---|---:|---:|---:|---|
| `samples/aift.hwp` | 38 | 21 | 1 | `%unk` CTRL_HEADER + MEMO command |
| `aift-field-range.hwp` | 38 | 21 | 1 | `%unk` CTRL_HEADER + MEMO command |

## 6. 실행한 검증

```text
cargo fmt --check
cargo test -q test_parse_field_begin_end_materializes_field_range
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp
cargo run --quiet --bin rhwp -- dump output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp --section 2 --para 482
cargo run --quiet --bin rhwp -- dump output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp --section 2 --para 484
```

결과:

```text
success
```

## 7. 판정 요청

다음 파일을 한컴 에디터에서 확인한다.

| file | 한컴 판정 유형 | 메모 앵커 스타일 | 메모 박스 스타일 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage4_memo_anchor_style/aift-field-range.hwp` |  |  |  |  | FIELD_END 복원 후보 |

판정 기준:

```text
1. Runtime Error 없이 열리는지 확인
2. 왼쪽 본문 메모 앵커 범위 표시가 정답지와 가까워졌는지 확인
3. 오른쪽 메모 박스/라벨 스타일이 정답지와 가까워졌는지 확인
```
