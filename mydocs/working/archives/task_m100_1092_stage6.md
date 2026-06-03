# Task M100-1092 Stage 6 작업 기록

## 1. 단계 목표

Stage 5 후보는 HWPX `charPr`의 shadow offset 누락을 보정했지만, 한컴 에디터의 메모 표시
스타일은 개선되지 않았다. Stage 6에서는 메모 anchor 문단을 HWP5 raw record 단위로 정답지와
비교해, `PARA_TEXT` 안의 `FIELD_END` marker 계약을 맞춘다.

## 2. 핵심 발견

대상 문단:

```text
section = 2
needle = 공동기관1
anchor text = "    - 공동기관1 : 두아즈"
```

정답지 `samples/aift.hwp`의 메모 anchor 문단은 다음 구조다.

```text
PARA_TEXT:
  FIELD_BEGIN marker: 03 00 65 6d 25 25 00 00 00 00 00 00 00 00 03 00
  text: "    - 공동기관1 : 두아즈"
  FIELD_END marker:   04 00 65 6d 25 00 01 ff ff 00 01 00 00 00 04 00

CTRL_HEADER:
  ctrl_id = %unk
  properties = 0x00008001
  command = MEMO/65535/1/1517431184/31247371/user/\;;
  field_id = 2135782115
  memo_index = 1
```

Stage 5 생성본은 `CTRL_HEADER`는 정답지와 일치했지만, `FIELD_END` marker를 일반 확장 컨트롤
형식으로 저장했다.

```text
generated Stage 5 FIELD_END:
  04 00 65 6d 25 25 00 00 00 00 00 00 00 00 04 00

oracle FIELD_END:
  04 00 65 6d 25 00 01 ff ff 00 01 00 00 00 04 00
```

따라서 메모 스타일 미적용의 다음 후보는 `MEMO_SHAPE`, `MEMO_LIST`, `CTRL_HEADER`, char shape가
아니라 `PARA_TEXT` 내부 memo `FIELD_END` marker contract다.

## 3. 구현 내용

수정 소스:

```text
src/serializer/body_text.rs
```

적용 내용:

```text
1. `FieldType::Memo` 또는 command가 `MEMO/`로 시작하는 필드의 FIELD_END를 memo 전용 marker로 저장한다.
2. 일반 필드의 FIELD_END는 기존 `push_extended_ctrl(0x0004, ctrl_id)` 경로를 유지한다.
3. memo FIELD_END marker contract를 단위 테스트로 고정한다.
```

추가 테스트:

```text
test_memo_field_end_uses_hancom_marker_tail
```

추가 진단 명령:

```text
rhwp hwp5-anchor-trace <파일.hwp> --needle <텍스트> [--section N] [--window N] [--out <path>]
```

이 명령은 특정 `PARA_TEXT` 주변 raw record를 출력해, IR diff에서 드러나지 않는 HWP5 marker
차이를 확인하기 위한 도구다.

## 4. 생성 후보

출력 파일:

```text
output/poc/hwpx2hwp/task1092/stage6_memo_field_end_marker/aift-memo-field-end.hwp
```

파일 정보:

```text
size = 4,605,952 bytes
rhwp info = ok, sections=3, pages=76
```

## 5. Raw record 검증

정답지와 Stage 6 후보의 메모 anchor `PARA_TEXT` hash:

| file | PARA_TEXT hash | 판정 |
|---|---|---|
| `samples/aift.hwp` | `e665b9e5dc593fc2` | oracle |
| `stage6/aift-memo-field-end.hwp` | `e665b9e5dc593fc2` | 일치 |

Stage 6 후보의 `PARA_TEXT`:

```text
03 00 65 6d 25 25 00 00 00 00 00 00 00 00 03 00
20 00 20 00 20 00 20 00 2d 00 20 00 f5 ac d9 b3
30 ae 00 ad 31 00 20 00 3a 00 20 00 50 b4 44 c5
88 c9
04 00 65 6d 25 00 01 ff ff 00 01 00 00 00 04 00
0d 00
```

이제 정답지와 같은 memo `FIELD_END` marker를 저장한다.

## 6. 실행한 검증

```text
cargo test -q test_memo_field_end_uses_hancom_marker_tail
cargo fmt --check
cargo check
cargo run --quiet --bin rhwp -- info output/poc/hwpx2hwp/task1092/stage6_memo_field_end_marker/aift-memo-field-end.hwp
cargo run --quiet --bin rhwp -- hwp5-anchor-trace output/poc/hwpx2hwp/task1092/stage6_memo_field_end_marker/aift-memo-field-end.hwp --section 2 --needle 공동기관1 --window 4 --out /tmp/aift_stage6_anchor_trace.md
```

결과:

```text
success
```

## 7. 판정 요청

다음 파일을 한컴 에디터에서 판정한다.

| file | 한컴 판정 유형 | 메모 표시 스타일 | 표/페이지 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage6_memo_field_end_marker/aift-memo-field-end.hwp` |  |  |  |  | memo FIELD_END marker 정합 후보 |

판정 포인트:

```text
1. 메모 anchor 왼쪽 텍스트의 초록 배경/메모 표시가 정답지와 가까워졌는지 확인한다.
2. 오른쪽 메모 박스/연결선 스타일이 정답지와 가까워졌는지 확인한다.
3. 표 높이/페이지 배치 차이는 별도 이슈로 분리했으므로, 이번 판정에서는 메모 표시 스타일만 본다.
```
