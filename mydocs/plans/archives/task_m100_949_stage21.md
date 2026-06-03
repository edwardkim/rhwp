# task_m100_949 Stage 21 계획

## 목표

Stage 20에서 분리한 `hwpx-h-03`의 2페이지 도형/그림 묶기 주변 계약 중,
`hp:rect > hp:drawText` 경로에서 HWPX 원천에 명시되어 있으나 HWP 저장 시 빠진 값을
정확히 materialize한다.

## 선행 결론

Stage 20 판정:

```text
1. GenShape #824의 14 bytes 차이는 unknown tail이 아니라 shapeComment 텍스트다.
   원천 HWPX: <hp:shapeComment>사각형입니다.</hp:shapeComment>

2. LIST_HEADER #826은 oracle 33B, generated 20B다.
   generated는 list_attr bit 5가 빠져 있었다.
   원천 HWPX: <hp:subList vertAlign="CENTER">

3. PARA_HEADER #827은 oracle 24B, generated 22B다.
   이 축은 Stage 21에서 단정하지 않고 후속 확인 대상으로 둔다.
```

## 구현 범위

이번 단계는 다음 두 항목만 반영한다.

```text
1. hp:rect/hp:shapeComment -> ShapeObject.common.description
2. hp:drawText/hp:subList@vertAlign -> TextBox.list_attr bits 5..6
   CENTER = 1 << 5
   BOTTOM = 2 << 5
```

범위 밖:

```text
- LIST_HEADER tail 13 bytes 임의 graft
- PARA_HEADER 22B -> 24B 보정
- group/shape/picture 전체 payload 보정
```

## 검증

```text
cargo test --quiet hwpx_h_03_rect_draw_text_contract_from_source
cargo test --quiet hwpx_h_03_href_ctrl_data_from_source_contract
cargo test --quiet table_axis_materializes_hancom_record_contract
cargo build --quiet
```

## 산출물

```text
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp
```

