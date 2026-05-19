# Task M100 #949 Stage 26 계획서 — GenShape CommonObjAttr 계약 구현 후보

## 1. 배경

Stage 25에서 `hwpx-h-03` 파일손상 후보인 drawText/image group의 GenShape
`CTRL_HEADER` 차이를 필드 단위로 확인했다.

핵심 차이:

```text
#824 outer rect       : bit 14, bit 26 누락
#831 first child pic  : bit 13, bit 26, vertical_offset=-2429 누락
#835 second child pic : bit 13, bit 14, bit 26 누락
```

HWPX source와의 대응:

```text
bit 13 후보: hp:pos@flowWithText
bit 14 후보: hp:pos@allowOverlap
bit 26 후보: GenShape HWP5 storage high-bit
```

현재 `CommonObjAttr` 모델에는 `flowWithText`, `allowOverlap`을 담을 필드가 없고,
HWP5 공통 개체 attr 직렬화도 bit 13/14/26을 합성하지 않는다.

## 2. 목표

Stage 26은 Stage 25의 `CTRL_HEADER` field mapping을 실제 구현 후보로 반영하고,
guard 파일을 생성해 한컴 에디터 시각 판정을 요청할 수 있는 상태를 만든다.

이번 단계의 목표는 다음 두 가지다.

```text
1. HWPX object pos 계약을 IR의 CommonObjAttr에 보존한다.
2. HWP 저장 시 GenShape CTRL_HEADER attr/offset을 한컴 정답지에 가깝게 materialize한다.
```

## 3. 작업 원칙

```text
1. Stage 17/18에서 성공한 table-axis 계약을 깨뜨리지 않는다.
2. Stage 23의 generic TextBox LIST/PARA header 보강은 재사용하지 않는다.
3. #974 `hy-001` 글상자 TAC 그림/space 조판 guard를 반드시 포함한다.
4. `hwpx-h-01`을 sentinel로 먼저 성공 유지시킨 후 `hwpx-h-02`, `hwpx-h-03`로 확장한다.
5. `hwpx-h-03` 파일손상이 완전히 해소되지 않아도 중단 위치가 전진하면 부분 원인으로 기록한다.
```

## 4. 구현 범위

### 4.1 모델 확장

대상:

```text
src/model/shape.rs
```

`CommonObjAttr`에 HWPX object pos 계약 필드를 추가한다.

```text
flow_with_text: bool
allow_overlap: bool
```

bit 26은 source XML에 직접 같은 이름의 속성이 확인되지 않았으므로, 이번 단계에서는 별도
명명으로 다룬다.

후보:

```text
hwp5_gen_shape_attr_bit26: bool
```

이름은 구현 시 더 적절한 이름으로 조정할 수 있다. 단, table adapter의
`0x08000000`과 혼동하지 않도록 `numbering`이라는 이름은 쓰지 않는다.

### 4.2 HWP5 parser 보존

대상:

```text
src/parser/control/shape.rs
```

`parse_common_obj_attr()`에서 HWP5 attr를 읽을 때 다음 값을 모델 필드에 보존한다.

```text
flow_with_text = attr bit 13
allow_overlap = attr bit 14
hwp5_gen_shape_attr_bit26 = attr bit 26
```

HWP 출처 파일은 기존처럼 `common.attr` 원본 값을 우선 직렬화하므로, 이 변경은 주로 IR 관찰성과
HWPX 직렬화 경로의 모델 일관성을 위한 것이다.

### 4.3 HWPX parser 반영

대상:

```text
src/parser/hwpx/section.rs
```

`<hp:pos>` 계열 파서에서 다음 속성을 읽어 `CommonObjAttr`에 보존한다.

```text
flowWithText
allowOverlap
vertOffset
horzOffset
```

이미 `vertOffset`/`horzOffset` 파싱 코드는 있으나, `#831`에서 세로 오프셋이 generated에
반영되지 않았으므로 구현 후 반드시 재확인한다.

우선 적용 대상:

```text
hp:pic
hp:rect / hp:container / drawing shape 계열
```

table은 이미 별도 table-axis 계약으로 성공한 상태이므로, 이번 단계에서 table 동작을
불필요하게 바꾸지 않는다.

### 4.4 HWP5 writer 반영

대상:

```text
src/document_core/converters/common_obj_attr_writer.rs
src/serializer/control.rs
```

현재 공통 개체 attr 직렬화 함수가 두 곳에 존재한다. 둘 중 하나만 바꾸면 table adapter와
shape serializer가 서로 다른 attr를 만들 수 있으므로 둘 다 확인한다.

`pack_common_attr_bits()` 및 shape CTRL_HEADER 직렬화 경로에서 다음을 합성한다.

```text
flow_with_text=true       -> bit 13
allow_overlap=true        -> bit 14
hwp5_gen_shape_attr_bit26 -> bit 26
```

중요:

```text
- HWP 출처 `common.attr != 0`인 경우는 원본 attr를 보존한다.
- HWPX 출처 `common.attr == 0`인 경우에만 모델 필드로 attr를 합성한다.
- table adapter의 `0x08000000` 보강과 GenShape bit26 `0x04000000`을 혼동하지 않는다.
```

### 4.5 adapter 보강

대상:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

필요 시 HWPX 출처 shape/picture에 대해 GenShape bit26 후보를 materialize한다.

단계 순서:

```text
1. parser/writer만으로 Stage 25 diff가 줄어드는지 먼저 확인한다.
2. bit26이 source 파싱만으로 채워지지 않으면 adapter에서 HWPX shape/picture 대상으로 보강한다.
3. table에는 이 보강을 적용하지 않는다.
```

## 5. 생성 파일

Stage 26 구현 후보로 다음 파일을 생성한다.

```text
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp
```

추가 진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/h03_ctrl_header_after.md
output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/generation.md
```

## 6. 시각 판정 요청표

`mydocs/working/task_m100_949_stage26.md`에 다음 표를 만든다.

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage26_genshape_common_attr_candidate/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 7. 검증

로컬 검증:

```text
cargo fmt --check
cargo test
target/debug/rhwp info <generated.hwp>
target/debug/rhwp hwp5-inventory-diff <oracle> <generated> --section 0 --focus ctrl --report hints
```

필요 시 전체 `cargo test`가 길면, 우선 관련 단위 테스트와 `cargo test --no-default-features` 여부를
현재 저장소 관례에 맞춰 조정한다.

## 8. 판정 기준

Stage 26 결과는 다음 중 하나로 판정한다.

```text
A. h01/h02/hy-001 guard 유지 + h03 파일손상 해소
B. h01/h02/hy-001 guard 유지 + h03 중단 위치 전진
C. h03 변화 없음. 다음 단계에서 SHAPE_COMPONENT/SHAPE_PICTURE payload 디코드 필요
D. guard 회귀. 후보 폐기 또는 적용 조건 축소
```

## 9. 완료 조건

```text
1. 구현 변경 완료
2. guard HWP 파일 4개 생성
3. h03 CTRL_HEADER after diff 작성
4. mydocs/working/task_m100_949_stage26.md 작성
5. git diff --check 통과
6. 작업지시자에게 한컴 시각 판정 요청
```

## 10. 승인 요청

이 계획으로 Stage 26 구현 후보를 진행할지 승인 요청한다.
