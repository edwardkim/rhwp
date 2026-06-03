# task_m100_903 Stage 6 — 1페이지 paragraph/line_seg/table diff inventory

## 1. 목적

Stage 5 산출물은 한컴 에디터에서 여전히 파일 손상 판정을 받았고,
rhwp-studio에서도 1페이지 문단과 표 배치가 무너졌다.

이번 단계는 구현보다 먼저 정답 HWP와 Stage 5 HWP의 1페이지 조판 차이를
문단, line_seg, 표 배치 단위로 비교하여 원인을 좁힌다.

비교 대상:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/stage5/hwpx-h-01_adapter.hwp
samples/hwpx/hwpx-h-01.hwpx
```

## 2. 페이지 배치 요약

정답 HWP 1페이지:

```text
items=26
used=834.9px
```

Stage 5 HWP 1페이지:

```text
items=17
used=509.9px
```

HWPX 원본 파싱 IR 1페이지:

```text
items=26
used=903.1px
```

즉 HWPX 원본 파싱 IR은 1페이지 item 수 관점에서 정답 HWP와 같은 26개 item을 가진다.
Stage 5 HWP로 저장/재로드하는 순간 17개 item으로 줄어든다.

## 3. 첫 붕괴 지점

정답 HWP와 HWPX 원본은 문단 `0:10` 표가 `TopAndBottom` 표로 배치된다.

HWPX 원본 문단 `0:10`:

```text
ls[0].vpos=26514
표: 4행×10열
common: treat_as_char=false, wrap=TopAndBottom, vert=Para(off=289), horz=Para(off=501)
outer_margin=(141,141,141,141)
```

Stage 5 HWP 재로드 문단 `0:10`:

```text
ls[0].vpos=26514
표: 4행×10열
common: treat_as_char=false, wrap=Square, vert=Paper(off=289), horz=Paper(off=501)
outer_margin=(0,0,0,0)
raw CTRL_HEADER attr=0
```

`line_seg.vpos` 자체는 문단 `0:10`에서는 아직 원본과 같다.
하지만 표의 common attr가 저장 후 손실되어 표가 HWPX/정답과 다른 방식으로 배치되고,
이후 문단들의 `line_seg.vpos`가 연쇄적으로 앞당겨진다.

## 4. 1페이지 주요 연쇄 차이

정답 HWP:

```text
pi=10 table vpos=32205, wrap=TopAndBottom
pi=11 text  vpos=33557..37757
pi=14 table vpos=48600, wrap=TopAndBottom
pi=16 text  vpos=50252..56552
pi=21 table vpos=69055, wrap=TopAndBottom
```

Stage 5 HWP:

```text
pi=10 table vpos=26514, wrap=Square
pi=14 table vpos=40425, wrap=Square
pi=21 table vpos=61112, wrap=Square
```

Stage 5는 표들이 문단 흐름을 밀어내는 `TopAndBottom` 객체로 해석되지 않고
`Square/Paper` 객체로 재로드된다. 그 결과 텍스트 문단들이 정답지보다 위로 당겨지고,
페이지 1 item 수가 26개에서 17개로 줄어든다.

## 5. 전체 IR diff 패턴

정답 HWP vs Stage 5 HWP 전체 `ir-diff` 주요 패턴:

- 다수 표에서 `outer_margin`: 정답 `(141 또는 283)` vs Stage 5 `(0,0,0,0)`
- 다수 표에서 `wrap`: 정답 `TopAndBottom` vs Stage 5 `Square`
- 다수 표에서 `vert_rel`: 정답 `Para` vs Stage 5 `Paper`
- 다수 표에서 `horz_rel`: 정답 `Para` 또는 `Column` vs Stage 5 `Paper`
- 이후 문단의 `line_seg.vpos`가 연쇄적으로 달라짐

HWPX 원본 vs Stage 5 HWP에서도 같은 표 common attr 손실이 반복된다.

## 6. 원인 가설

HWPX 파서는 표의 `CommonObjAttr`를 갖고 있다.

하지만 `convert_hwpx_to_hwp_ir()`의 `adapt_table()`은 특별히 materialize 대상으로
판정하지 않은 표에 대해 다음 처리를 한다.

```text
table.raw_ctrl_data = serialize_common_obj_attr(&table.common)
...
else {
    table.raw_ctrl_data[0..4].copy_from_slice(&0u32.to_le_bytes());
    table.attr = 0;
}
```

이 로직이 HWPX 원본이 가진 표 배치 attr를 HWP 저장 직전에 0으로 만든다.
따라서 저장 후 재로드하면 표가 기본값인 `Square/Paper`로 해석된다.

Stage 5에서 그림/도형 `CommonObjAttr` packing은 고쳤지만,
표 adapter가 일반 표의 CTRL_HEADER attr를 다시 0으로 지우고 있어
페이지 조판 붕괴가 계속된다.

## 7. Stage 6 접근 제안

구현 전에 RED 테스트를 먼저 만든다.

테스트 후보:

```text
task903_hwpx_h_01_page1_flow_tables_common_attrs_survive_hwp_save_reload
```

검증 대상:

- 문단 `0:10`, `0:14`, `0:21`의 표
- HWPX 원본과 adapter HWP 재로드 후 다음 필드가 일치해야 함
  - `common.text_wrap`
  - `common.vert_rel_to`
  - `common.horz_rel_to`
  - `outer_margin_left/right/top/bottom`

추가 layout gate:

```text
task903_hwpx_h_01_page1_item_count_matches_source_after_hwp_save_reload
```

검증:

- HWPX 원본 1페이지 item count: 26
- adapter HWP 재로드 1페이지 item count도 26이어야 함

## 8. 구현 후보

`adapt_table()`에서 일반 표의 attr를 무조건 0으로 지우는 처리를 제거하거나,
조건을 좁혀야 한다.

후보 정책:

1. HWPX 출처 표는 기본적으로 `CommonObjAttr` enum에서 attr를 합성해 보존한다.
2. 특수한 손상 회피 케이스가 필요한 표만 명시적으로 0으로 둔다.
3. `outer_margin`은 HWPX `<hp:outMargin>` 값을 `CommonObjAttr.margin`에도 승격하여
   HWP CTRL_HEADER에서 재로드 가능하게 한다.

이 접근은 Stage 5의 첫 표 국소 수정이 아니라, 1페이지 흐름 표 전체의 배치 속성을
저장/재로드에서 보존하는 방향이다.

## 9. Stage 6 구현 결과

RED 테스트를 먼저 추가했다.

```text
task903_hwpx_h_01_page1_flow_tables_common_attrs_survive_hwp_save_reload
```

초기 RED:

```text
문단 0:10 표 text_wrap
left: Square
right: TopAndBottom
```

적용한 핵심 변경:

- HWPX 출처 표의 `raw_ctrl_data` 합성 시 일반 표의 CTRL_HEADER attr를 더 이상 0으로 지우지 않는다.
- `CommonObjAttr` enum 필드에서 packed attr를 합성하고 `table.attr`에도 반영한다.
- 표 `outer_margin`을 `CommonObjAttr.margin`으로 항상 승격한다.
- `hwpx-h-02`는 한컴 정답 HWP 기준 페이지 수가 10페이지임을 확인했다.
  - HWPX 원본 rhwp 렌더링: 9페이지
  - 한컴 2020 변환 정답 HWP: 10페이지
  - 따라서 기존 테스트의 “원본 HWPX 페이지 수와 같아야 함” 기준을 한컴 HWP 저장 기준으로 보정했다.

## 10. Stage 6 산출물

```text
output/poc/hwpx2hwp/task903/stage6/hwpx-h-01_adapter.hwp
```

내부 재로드 결과:

```text
페이지 수: 9
1페이지 items=26
1페이지 used=834.9px
```

문단 `0:10` 표 재로드 결과:

```text
wrap=TopAndBottom
vert=Para(off=289)
horz=Para(off=501)
outer_margin=(141,141,141,141)
raw CTRL_HEADER attr=[10, 03, 2A, 00, ...]
```

Stage 5에서 보이던 `Square/Paper/outer_margin=0` 손실은 사라졌다.

## 11. 테스트 결과

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> 37 passed

cargo test --test hwpx_roundtrip_integration -- --nocapture
=> 17 passed
```

## 12. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage6/hwpx-h-01_adapter.hwp
```

판정 항목:

- 한컴 에디터에서 파일 손상 판정이 사라지는지
- rhwp-studio에서 1페이지 문단/표 배치가 Stage 5보다 회복됐는지
- 1페이지 표 안 이미지와 후반 묶음 그림이 유지되는지
- 전체 페이지 수가 한컴 정답 HWP와 같은 9페이지인지

## 13. 작업지시자 판정 결과

```text
한컴 에디터: 파일손상 판정
rhwp-studio: 렌더링 성공
```

판단:

- Stage 6은 rhwp-studio 렌더링/IR 회복에는 성공했다.
- 한컴 손상은 표 배치 IR 자체보다는 HWP 바이너리 레코드 호환성 문제로 분리한다.

추가 레코드 덤프 단서:

```text
정답 HWP:
  PARA_HEADER sz=24
  TABLE sz=24 또는 30
  LIST_HEADER sz=47 또는 65
  SectionDef CTRL_HEADER sz=47

Stage 6 산출물:
  PARA_HEADER sz=22
  TABLE sz=22 또는 28
  LIST_HEADER sz=34
  SectionDef CTRL_HEADER sz=28
```

특히 표 셀의 `LIST_HEADER`가 정답 HWP보다 짧다.
rhwp 파서는 이를 허용하지만 한컴 에디터는 일부 레코드의 tail 필드를 더 엄격하게
검증할 가능성이 높다.
