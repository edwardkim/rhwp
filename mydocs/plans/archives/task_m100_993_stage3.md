# Task M100-993 Stage 3 계획서

## 1. 목적

Stage 2 판정으로 확정된 `mel-001` 파일손상 원인을 실제 HWPX to HWP adapter 구현에 반영한다.

확정 원인:

```text
2페이지 "󰊳  예산 현황" 직후 12x5 표의 CTRL_HEADER(Table).common_attr

oracle:    0x282a2311
generated: 0x082a2311
missing:   0x20000000
```

## 2. 구현 방향

현재 adapter는 table `common_attr`를 다음 값으로 materialize한다.

```text
0x082a2311
```

Stage 3에서는 `mel-001`에서 한컴 정답지가 요구하는 high bit `0x20000000`을 table
`CTRL_HEADER` materialization 규칙에 반영한다.

우선 구현 후보:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

관련 함수:

```text
materialize_table_ctrl_header_attr
```

## 3. 검증 대상

수정 후 다음 파일을 다시 생성한다.

```text
samples/hwpx/mel-001.hwpx
```

생성 결과:

```text
output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

비교/guard:

```text
- rhwp load pages=20 유지
- 한컴 에디터 파일손상 해소 여부 판정
- 기존 #949/#974에서 통과한 핵심 샘플 회귀 여부 확인
```

## 4. 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 2페이지 첫 1x1 표 배경 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp` |  |  |  |  |  |  |  |

## 5. 별도 추적

2페이지 첫 1x1 표 셀 배경이 검게 출력되는 문제는 Stage 3의 파일손상 수정과 분리한다.

이 문제는 다음 Stage에서 별도 비교한다.

```text
- oracle/generated DocInfo BorderFill payload
- 1x1 표 셀 border_fill_id=7 참조가 가리키는 실제 BorderFill record
- HWPX header의 borderFill 정의와 HWP5 BorderFill record 변환
```

## 6. 완료 조건

```text
- adapter 구현 결과가 Stage 2의 02 probe와 같은 파일손상 해소 효과를 낸다.
- 한컴 에디터에서 파일손상이 사라지는지 작업지시자 판정을 받는다.
- 검은 셀 배경 문제는 별도 Stage로 넘길 수 있게 후보를 정리한다.
```
