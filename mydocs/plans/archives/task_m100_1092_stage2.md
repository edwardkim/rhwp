# Task M100-1092 Stage 2 수행 계획서

## 1. Stage 1 결론

Stage 1에서 `samples/aift.hwp` 정답지와 `saved/111aift.hwp` 현재 저장본을 비교했다.

핵심 차이:

```text
1. DocInfo MEMO_SHAPE count와 ID_MAPPINGS memo_shape_count는 이미 일치한다.
2. MEMO_SHAPE payload는 lineType 1바이트가 다르다.
3. HWPX MEMO field marker는 생성본에도 존재하지만, 정답지와 ctrl_id/properties가 다르다.
4. 정답지에는 MEMO_LIST 2개가 있으나 생성본에는 없다.
```

## 2. Stage 2 목표

Stage 2는 메모 컨트롤 저장 계약을 작은 후보로 나누어 검증한다.

출력 위치:

```text
output/poc/hwpx2hwp/task1092/stage2_memo_contract_candidates/
```

판정 대상:

```text
samples/hwpx/aift.hwpx
samples/aift.hwp
saved/111aift.hwp
```

## 3. 후보 축

### 3.1 MEMO field marker 후보

현재 생성본:

```text
ctrl_id = %%me
properties = 0x0001
```

정답지:

```text
ctrl_id = %unk
properties = 0x8001
```

수정 후보:

```text
HWPX fieldBegin type="MEMO"를 HWP5로 저장할 때만
  ctrl_id = %unk
  properties |= 0x8000
  Number parameter를 memo_index로 보존
```

예상 수정:

```text
src/parser/hwpx/section.rs
src/serializer/control.rs
```

### 3.2 MEMO_SHAPE lineType 후보

현재 생성본:

```text
MEMO_SHAPE lineType = 0
```

정답지:

```text
MEMO_SHAPE lineType = 1
```

수정 후보:

```text
hh:memoPr lineType="SOLID" 매핑을 정답지와 일치시킨다.
```

예상 수정:

```text
src/parser/hwpx/header.rs
```

### 3.3 MEMO_LIST materialize 후보

현재 생성본:

```text
MEMO_LIST 없음
```

정답지:

```text
MEMO_LIST payload=01 00 00 00
MEMO_LIST payload=02 00 00 00
```

수정 후보:

```text
HWPX fieldBegin type="MEMO" 목록을 기준으로 HWP5 MEMO_LIST record를 생성한다.
```

주의:

```text
MEMO_LIST 하위 paragraph list를 어떤 source에서 구성할지 먼저 코드상 표현을 확정한다.
정답지의 MEMO_LIST 하위 PARA_TEXT는 주변 제목 문단 텍스트로 시작한다.
이 값을 단순 복사하지 않고 HWPX source와 대응되는 규칙을 확인한다.
```

예상 수정:

```text
src/model/control.rs 또는 별도 memo model
src/parser/hwpx/section.rs
src/serializer/body_text.rs 또는 src/serializer/control.rs
```

## 4. 생성 후보 파일

```text
01_memo_field_marker_only.hwp
02_memo_shape_linetype_only.hwp
03_memo_field_marker_plus_shape.hwp
04_memo_list_only.hwp
05_all_memo_axes.hwp
```

## 5. 판정표

| file | 한컴 판정 유형 | 메모 표시 | 파일손상 여부 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| `01_memo_field_marker_only.hwp` |  |  |  |  | field marker |
| `02_memo_shape_linetype_only.hwp` |  |  |  |  | DocInfo shape |
| `03_memo_field_marker_plus_shape.hwp` |  |  |  |  | 01+02 |
| `04_memo_list_only.hwp` |  |  |  |  | MEMO_LIST |
| `05_all_memo_axes.hwp` |  |  |  |  | 전체 후보 |

## 6. 검증

필수:

```text
cargo fmt --check
cargo check
```

후보 생성 후:

```text
target/debug/rhwp hwp5-inventory ...
target/debug/rhwp hwp5-inventory-diff ...
```

시각 판정:

```text
1. 한컴 에디터에서 정상 로딩
2. 메모 컨트롤 표시/편집 가능 여부
3. rhwp-studio 재로드 후 메모 구조 유지
```

## 7. 승인 요청

Stage 2에서는 먼저 `01_memo_field_marker_only`와 `02_memo_shape_linetype_only`를 코드 후보로 구현해
판정 파일을 만든다. `MEMO_LIST`는 source 대응 규칙을 확인한 뒤 별도 후보로 넣는다.
