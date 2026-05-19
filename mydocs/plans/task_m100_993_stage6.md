# Task M100-993 Stage 6 계획서

## 1. 목적

Stage 5에서 `mel-001` 조직도 셀 내부 세로 방향 윗쪽 맞춤 문제를 해결했다.

이제 다시 원래 목표인 HWPX to HWP 저장 문제로 돌아간다. Stage 6의 목적은 현재 adapter
저장 결과를 새 기준선으로 만들고, 셀 속성의 HWP5 저장 계약을 정답 HWP와 record 단위로
검증하는 것이다.

이번 단계의 핵심 관점은 다음과 같다.

```text
HWPX 셀 속성은 HWP5 셀 record 계약으로 정확하게 materialize되어야 한다.
```

즉, XML 속성을 단순 복사하는 것이 아니라 HWPX 셀의 의미 속성을 HWP5 `LIST_HEADER`,
셀 고정 payload, tail/extra, DocInfo 참조 구조로 빠짐없이 매핑해야 한다.

남은 관찰:

```text
1. 2페이지 첫 1x1 표 셀 배경이 검정/비정상으로 저장된다.
2. 1페이지 처음 부분에 정체를 알 수 없는 선이 나타난다.
```

이번 단계에서는 1번을 직접 BorderFill 문제로 단정하지 않는다. 먼저 해당 셀의 HWP5 셀
계약 전체를 비교한 뒤, 실제 차이가 BorderFill 참조인지, `LIST_HEADER.list_attr`인지,
셀 payload/tail인지 분리한다. 2번은 별도 축으로 남긴다.

## 2. 현재 기준선

Stage 3에서 파일손상 해소 조건은 이미 확인했다.

```text
caption table CTRL_HEADER.common_attr:
generated 0x082a2311 -> oracle 0x282a2311
조건: table.caption.is_some()
```

Stage 5에서 조직도 셀 렌더링 기준선도 고쳤다.

```text
병합 셀 vertical_align=Top일 때만 LINE_SEG.vpos를 셀 상단 기준으로 앵커링
```

따라서 Stage 6은 다음 상태를 기준으로 한다.

```text
1. caption table common_attr 보강이 적용된 HWP 저장 결과
2. Stage 5 시각 판정을 통과한 셀 세로 정렬 수정
3. Center/Bottom 셀은 LINE_SEG.vpos 때문에 Top처럼 렌더링되지 않아야 함
```

## 3. 입력

원본:

```text
samples/hwpx/mel-001.hwpx
```

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

보조 렌더링 기준:

```text
samples/hwpx/mel-001.hwpx
pdf-large/hwpx/mel-001.pdf
```

## 4. 작업 범위

이번 단계에서는 다음을 수행한다.

```text
1. 현재 adapter로 mel-001.hwpx를 HWP로 저장한다.
2. 한컴 문제 지점인 2페이지 첫 1x1 표를 IR dump와 HWP5 record inventory에서 식별한다.
3. 해당 표/셀의 HWP5 `LIST_HEADER`와 셀 고정 payload를 정답 HWP와 생성 HWP에서 비교한다.
4. `list_attr`, `width_ref`, `padding`, `border_fill_id`, `raw_list_extra/tail` 차이를 분리한다.
5. `border_fill_id` 차이가 확인되면 참조 DocInfo BorderFill record payload까지 비교한다.
6. 차이가 확인된 축만 사용해 probe를 생성한다.
```

검토 대상 셀 속성:

```text
- LIST_HEADER.list_attr
  - text_direction
  - vertical_align
  - apply_inner_margin
  - is_header
- LIST_HEADER.width_ref
- 셀 고정 payload
  - row/col, row_span/col_span
  - width/height
  - padding left/right/top/bottom
  - border_fill_id
- raw_list_extra/tail
- table-level border_fill_id 및 zone border_fill_id
- DocInfo BorderFill payload
```

현재 확인된 구현 위험:

```text
1. serializer는 셀 list_attr를 text_direction + vertical_align 중심으로 재구성한다.
2. adapter는 apply_inner_margin을 text_direction=1로 우회 materialize한다.
3. 이 방식은 서로 다른 의미의 속성을 같은 비트 경로에 섞으므로 장기적으로 제거해야 한다.
4. HWPX 출처 셀은 raw_list_extra가 비어 있을 수 있으므로 HWP5 저장 계약을 별도로 생성해야 한다.
```

probe 후보:

```text
01_current_adapter
02_target_cell_list_attr_from_oracle
03_target_cell_fixed_payload_from_oracle
04_target_cell_border_fill_id_from_oracle
05_target_border_fill_record_from_oracle
06_target_cell_list_attr_plus_payload
07_target_cell_border_fill_bundle
08_target_cell_full_contract_bundle
```

## 5. 성공 기준

1차 성공:

```text
한컴 에디터에서 파일손상 없이 열리고, 2페이지 첫 1x1 표 배경이 정답과 같아진다.
```

2차 성공:

```text
rhwp-studio에서도 동일 표 배경이 정답 HWP/PDF와 일치한다.
```

3차 성공:

```text
Stage 5에서 해결한 조직도 셀 세로 방향 윗쪽 맞춤이 회귀하지 않는다.
```

4차 성공:

```text
셀 vertical_align=Center/Bottom이 LINE_SEG.vpos 보정 때문에 Top처럼 렌더링되지 않는다.
```

## 6. 산출물

작업 기록:

```text
mydocs/working/task_m100_993_stage6.md
```

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/
```

예상 산출물:

```text
mel-001-current.hwp
cell_contract_target_trace.md
cell_list_header_diff.md
cell_payload_diff.md
borderfill_payload_diff.md
probe_generation.md
판정용 HWP 파일들
```

## 7. 비목표

이번 단계에서 다음은 해결하지 않는다.

```text
- 1페이지 처음 부분 정체 불명 선
- 모든 BorderFill/Fill serializer 전면 개편
- 문서 전체 표 배치 재조정
```

단, 1x1 표 배경 문제가 셀 `LIST_HEADER` 또는 BorderFill serializer의 구조적 문제로 확인되면,
그때 별도 단계에서 일반화 구현 계획을 세운다.

## 8. 구현 원칙

이번 단계에서는 다음 원칙을 지킨다.

```text
1. 성공한 렌더링 보정과 HWP 저장 계약 보정을 섞지 않는다.
2. apply_inner_margin을 text_direction으로 우회하는 방식은 정답 후보로 취급하지 않는다.
3. 셀 속성은 명시적인 HWP5 list_attr/materialized payload 생성으로 해결한다.
4. BorderFill은 셀 속성 계약 검증 뒤에 참조 payload 문제로 좁혀졌을 때만 직접 수정한다.
5. probe는 hwpx-h 계열처럼 성공/실패 패턴을 섞어 해석하지 않고, mel-001 문제 셀 기준으로 정답 HWP와 비교한다.
```

## 9. 승인 요청

이 계획으로 Stage 6을 진행한다.
