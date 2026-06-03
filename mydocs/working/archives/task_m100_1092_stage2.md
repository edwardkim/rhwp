# Task M100-1092 Stage 2 작업 기록

## 1. 단계 목표

Stage 1에서 확인한 메모 컨트롤 저장 차이 중, HWPX source에서 바로 확정할 수 있는 두 축을 먼저
구현 후보로 반영했다.

```text
1. HWPX fieldBegin type="MEMO"의 HWP5 field marker 저장 형태
2. HWPX hh:memoPr lineType="SOLID"의 HWP5 MEMO_SHAPE lineType 매핑
```

`MEMO_LIST` 생성은 아직 source 대응 규칙이 확정되지 않았으므로 이번 단계에서 구현하지 않았다.

## 2. 수정한 소스

```text
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
src/serializer/control.rs
```

적용 내용:

```text
1. hh:memoPr lineType="SOLID"를 HWP5 MEMO_SHAPE lineType=1로 저장
2. hp:integerParam name="Number" 값을 Field.memo_index로 보존
3. FieldType::Memo 저장 시 CTRL_HEADER를 정답지와 같은 형태로 저장
   - ctrl_id = %unk (0x25756e6b)
   - properties |= 0x8000
```

## 3. 생성 후보

출력 위치:

```text
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/
```

생성 파일:

```text
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_memo_field_marker_plus_shape.hwp
```

파일 정보:

```text
size = 4,604,928 bytes
rhwp info = ok, sections=3, pages=76
```

## 4. 정답지 비교

정답지:

```text
samples/aift.hwp
```

생성한 비교 산출물:

```text
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_inventory.md
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_inventory.jsonl
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_inventory_diff.md
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_contract_hints.md
```

### 4.1 MEMO_SHAPE

정답지와 생성 후보의 `MEMO_SHAPE` payload가 일치한다.

```text
oracle:
  head22=e7 3c 00 00 01 03 a9 a9 a9 00 cb ff 99 00 fd bc dd 00 00 00 00 00
  hash=1de1259eebaba551

generated:
  head22=e7 3c 00 00 01 03 a9 a9 a9 00 cb ff 99 00 fd bc dd 00 00 00 00 00
  hash=1de1259eebaba551
```

따라서 Stage 1의 `lineType=0` 문제는 해결됐다.

### 4.2 MEMO field marker

두 개의 메모 field marker가 이제 정답지와 같은 `ctrl_id/properties` 형태로 저장된다.

```text
ctrl_id = 0x25756e6b (%unk)
properties = 0x8001
```

후보 inventory:

```text
BodyText.Section2#11257 CTRL_HEADER ctrl=0x25756e6b/Unknown size=101
BodyText.Section2#11265 CTRL_HEADER ctrl=0x25756e6b/Unknown size=101
```

다만 payload hash는 정답지와 아직 다르다.

```text
oracle    hash=67156d3678a125ee / 46a646257c075cf6
generated hash=a0d82bcb4f39b35d / 99e8713e44b61e8a
```

앞 32바이트는 `ctrl_id`, `properties`, `MEMO/...` command prefix까지 정답지와 같은 형태다. 남은 차이는
command tail 또는 memo field 부가 값 축으로 별도 추적한다.

### 4.3 MEMO_LIST

`MEMO_LIST`는 아직 생성되지 않는다.

```text
oracle:
  BodyText.Section2#25924 MEMO_LIST payload=01 00 00 00
  BodyText.Section2#25929 MEMO_LIST payload=02 00 00 00

generated:
  MEMO_LIST 없음
```

이번 단계의 후보는 field marker와 MEMO_SHAPE 축만 처리한 것이므로, `MEMO_LIST` 누락은 남은 핵심
작업으로 유지한다.

## 5. 검증

실행한 명령:

```text
cargo fmt --check
cargo test -q test_parse_hwpx_memo_shape_solid_line_type_uses_hwp5_value
cargo test -q test_parse_memo_field_parameters_preserves_number_as_memo_index
cargo build
cargo check
```

결과:

```text
success
```

기존 경고는 있었지만 이번 수정으로 새 실패는 발생하지 않았다.

## 6. 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| file | 한컴 판정 유형 | 메모 표시 | 파일손상 여부 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/03_memo_field_marker_plus_shape.hwp` |  |  |  |  | field marker + MEMO_SHAPE |

## 7. 판정 결과와 정정

작업지시자 판정:

```text
03_memo_field_marker_plus_shape.hwp:
  한컴 에디터에서 Runtime Error
  R6025 pure virtual function call
```

추가 분석 결과, `03` 후보는 `ctrl_id/properties`와 `MEMO_SHAPE`는 맞았지만 MEMO field marker의
마지막 8바이트 중 `field_id`가 틀렸다.

정답지:

```text
... e3 72 4d 7f 01 00 00 00
... e4 72 4d 7f 02 00 00 00
```

`e3 72 4d 7f`, `e4 72 4d 7f`는 HWPX `fieldBegin@id` 값이다.

`03` 후보:

```text
... 65 6d 25 25 01 00 00 00
... 65 6d 25 25 02 00 00 00
```

`65 6d 25 25`는 `%%me` ctrl_id 바이트다. 즉, HWPX MEMO field에서 HWP5 marker tail로 들어가야 할
`fieldBegin@id` 대신 기존 field ctrl_id가 들어가고 있었다.

정정:

```text
FieldType::Memo는 fieldid가 아니라 id 속성을 HWP5 field_id로 저장한다.
일반 field는 기존처럼 fieldid 우선 정책을 유지한다.
```

정정 후보 파일:

```text
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/06_memo_field_id_fix.hwp
```

`06` 후보의 메모 field marker는 정답지와 payload hash까지 일치한다.

```text
BodyText.Section2#11257 CTRL_HEADER hash=67156d3678a125ee
BodyText.Section2#11265 CTRL_HEADER hash=46a646257c075cf6
```

`MEMO_SHAPE`도 정답지와 일치한다.

```text
MEMO_SHAPE hash=1de1259eebaba551
```

남은 차이:

```text
MEMO_LIST 2개는 아직 생성되지 않는다.
```

중요한 판정 기준:

```text
rhwp-studio는 현재 한컴 메모를 렌더링하지 않는다.
따라서 이번 단계의 1차 성공 기준은 한컴 에디터 정상 로딩과 한컴 메모 컨트롤 계약 유지다.
rhwp-studio의 메모 렌더링은 별도 구현 축으로 분리한다.
```

## 8. 추가 판정 요청

다음 파일을 다시 한컴 에디터에서 판정한다.

| file | 한컴 판정 유형 | 메모 표시 | 파일손상 여부 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/06_memo_field_id_fix.hwp` |  |  |  |  | field marker payload hash 일치 |

## 9. 다음 단계

시각 판정 후 다음을 결정한다.

```text
1. 06 후보가 한컴 에디터에서 Runtime Error 없이 열리는지 확인
2. 메모 컨트롤이 여전히 불완전하면 MEMO_LIST materialize 규칙을 Stage 3에서 구현
3. rhwp-studio 메모 렌더링은 HWP5 저장 계약과 분리해 별도 이슈/단계로 처리
```
