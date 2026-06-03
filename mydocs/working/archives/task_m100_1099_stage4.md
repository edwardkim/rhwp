# Task M100-1099 Stage 4 작업 기록

## 1. 목적

Stage 3에서 한컴 파일손상은 해결되었지만, 한컴 에디터에서 1~3항 문제 영역의 문단 경계선이
외곽만 이어지지 않고 문단 시작/끝에도 선이 출력되었다.

이번 단계는 다음 contract를 보강한다.

```text
HWPX paraPr border/@connect, @ignoreMargin
  -> HWP5 PARA_SHAPE attr1 bit 28, bit 29
```

## 2. 원인

HWPX 원본의 `지문` paragraph style은 다음과 같이 문단 테두리 연결을 명시한다.

```xml
<hh:border borderFillIDRef="10"
           offsetLeft="0"
           offsetRight="0"
           offsetTop="1133"
           offsetBottom="0"
           connect="1"
           ignoreMargin="0"/>
```

한컴 공식 HWP5 spec 기준:

```text
PARA_SHAPE attr1 bit 28 = 문단 테두리 연결 여부
PARA_SHAPE attr1 bit 29 = 문단 여백 무시 여부
```

기존 parser는 `borderFillIDRef`와 offset만 읽고, `connect`/`ignoreMargin`은 버렸다.
그 결과 생성 HWP에서는 `지문` style의 attr1 bit 28이 0으로 저장되어 한컴 에디터가 연속 문단을
하나의 문단 테두리 박스로 연결하지 못했다.

## 3. 구현

수정 파일:

```text
src/parser/hwpx/header.rs
```

수정 내용:

```text
1. hh:border/@connect=true|1  -> ParaShape.attr1 bit 28 set
2. hh:border/@connect=false|0 -> ParaShape.attr1 bit 28 clear
3. hh:border/@ignoreMargin=true|1  -> ParaShape.attr1 bit 29 set
4. hh:border/@ignoreMargin=false|0 -> ParaShape.attr1 bit 29 clear
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/01_stage3_baseline.hwp
output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/02_para_border_connect_attr.hwp
```

파일 크기:

| file | size |
|---|---:|
| `01_stage3_baseline.hwp` | 668K |
| `02_para_border_connect_attr.hwp` | 668K |

## 5. 구조 확인

`지문` style의 ParaShape는 HWPX `paraPr id="34"`에 대응한다.

생성 후보에서 해당 ParaShape의 attr1이 다음처럼 바뀌었다.

```text
Stage 3 baseline:
  attr1 = 0x00000000

Stage 4 candidate:
  attr1 = 0x10000000
  bit 28 set
```

정답지는 같은 ParaShape에서 다음 값을 가진다.

```text
oracle:
  attr1 = 0x10000080
  bit 28 set
```

따라서 이번 후보는 문단 테두리 연결 bit를 정답지와 같은 방향으로 맞춘다.
남은 bit 7 차이는 별도 줄나눔 contract로 분리한다.

비교 산출물:

```text
output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/candidate.section0.inventory.md
output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/contract_violation_hints.md
```

## 6. 검증

실행:

```text
cargo fmt --check
cargo test document_core::converters::hwpx_to_hwp::tests::header_footer_nested_tables_are_materialized
cargo build --bin rhwp
```

결과:

```text
success
```

## 7. 판정 요청

한컴 에디터 확인 대상:

| file | 한컴 판정 유형 | 문단 경계선 | 바탕쪽 출력 | 지문 박스 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/01_stage3_baseline.hwp` |  | 문단 시작/끝 선 출력 |  |  |  | baseline |
| `output/poc/hwpx2hwp/task1099/stage4_paragraph_border_connect_contract/02_para_border_connect_attr.hwp` |  |  |  |  |  | paragraph border connect 후보 |

## 8. 다음 판단

```text
1. 02 파일에서 한컴 문단 경계선이 정상화되면:
   - HWPX border/@connect -> HWP5 PARA_SHAPE attr1 bit 28 매핑으로 확정한다.
   - 2/3/4페이지 축소 샘플로 확장한다.

2. 02 파일에서도 경계선 문제가 남으면:
   - connect bit 매핑은 유지한다.
   - 다음 후보는 PARA_SHAPE attr1 bit 7 또는 58바이트 ParaShape tail 차이로 이동한다.
```

## 9. 판정 결과

작업지시자 한컴 에디터 판정:

```text
성공
```

결론:

```text
1. Stage 3에서 남았던 한컴 문단 경계선 시작/끝 출력 문제는 해결되었다.
2. HWPX hh:border/@connect -> HWP5 PARA_SHAPE attr1 bit 28 매핑을 확정한다.
3. 다음 단계는 2/3/4페이지 축소 샘플과 전체 exam_kor 샘플에 같은 contract를 적용해 회귀를 확인한다.
```
