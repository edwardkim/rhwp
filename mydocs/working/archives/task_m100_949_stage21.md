# task_m100_949 Stage 21 작업 기록

## 1. 목적

Stage 20에서 확인한 `hp:rect > hp:drawText` 계약을 최소 범위로 반영한다.

이번 단계는 `hwpx-h-03` 파일손상의 근본 원인을 단정하는 단계가 아니라,
HWPX 원천에 명시된 도형 설명과 텍스트박스 세로 정렬 정보를 HWP IR/HWP record에 누락 없이
반영하는 단계다.

## 2. 변경 사항

### 2.1 `hp:shapeComment`

`hp:rect` 계열 shape 내부의 다음 값을 `ShapeObject.common.description`으로 보존한다.

```xml
<hp:shapeComment>사각형입니다.</hp:shapeComment>
```

Stage 20의 `BodyText.Section0#824` 차이 14 bytes는 이 문자열의 HWP 저장 표현이었다.

### 2.2 `hp:drawText > hp:subList@vertAlign`

다음 HWPX 속성을 `TextBox.list_attr`의 세로 정렬 bit로 materialize한다.

```text
TOP    -> 0 << 5
CENTER -> 1 << 5
BOTTOM -> 2 << 5
```

`hwpx-h-03` 원천의 `vertAlign="CENTER"`는 `LIST_HEADER #826`의 `list_attr`에
`0x20`으로 반영되어야 한다.

## 3. 검증

실행한 테스트:

```text
cargo test --quiet hwpx_h_03_rect_draw_text_contract_from_source
cargo test --quiet hwpx_h_03_href_ctrl_data_from_source_contract
cargo test --quiet table_axis_materializes_hancom_record_contract
cargo build --quiet
```

결과:

```text
통과
```

기존 warning은 이번 변경 범위 밖의 기존 warning이다.

## 4. 생성 결과

```text
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp
```

파일 크기:

| file | size |
|---|---:|
| `hwpx-h-01.hwp` | 374,784 bytes |
| `hwpx-h-02.hwp` | 32,256 bytes |
| `hwpx-h-03.hwp` | 38,400 bytes |

`rhwp info` 기준 `hwpx-h-03.hwp`:

```text
sections=2
pages=9
BinData=3
```

## 5. 정적 trace

생성 파일:

```text
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03_inventory.md
output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/h03_shape_bundles_w8.md
```

`hwpx-h-03_inventory.md`에서 확인한 핵심 record:

```text
BodyText.Section0#824 CTRL_HEADER GenShape size=60
BodyText.Section0#826 LIST_HEADER size=20 head20=01 00 00 00 20 00 ...
BodyText.Section0#827 PARA_HEADER size=22
BodyText.Section0#833 CTRL_DATA size=76 hash=024e873ad9c2bd92
```

해석:

```text
1. #824 shapeComment 반영은 성공했다.
2. #826 list_attr bit 5 반영은 성공했다.
3. #826 LIST_HEADER tail 13 bytes와 #827 PARA_HEADER 24B 계약은 아직 닫히지 않았다.
4. #833 CTRL_DATA는 Stage 15와 동일하게 oracle hash와 일치한다.
```

## 6. 시각 판정표

작업지시자 판정 대기:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  | Stage 17/18 guard |
| `output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-02.hwp` |  |  |  |  |  |  | Stage 18 guard |
| `output/poc/hwpx2hwp/task949/stage21_rect_drawtext_candidate/hwpx-h-03.hwp` |  |  |  |  |  |  | Stage 21 target |

## 7. 다음 판단 기준

```text
1. h01/h02가 계속 성공해야 한다.
2. h03 파일손상이 해소되면 Stage 20의 drawText 계약이 주 원인 후보가 된다.
3. h03 파일손상이 유지되면 다음 단계는 #826 LIST_HEADER tail 13 bytes와 #827 PARA_HEADER 24B
   차이를 HWPX 원천/정답지 기준으로 다시 좁힌다.
```

