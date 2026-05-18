# Task m100 #949 Stage 31 계획 - id/instid + drawText envelope 조합 검증

## 1. 배경

Stage 30 수동 판정 결과는 Stage 28과 동일했다.

```text
파일손상과 중단된 페이지 위치가 동일하다.
```

따라서 Stage 30의 `id/instid` 분리 보정은 필요한 record 차이를 닫았지만,
한컴 파일손상 해소에는 충분하지 않았다.

현재까지의 중요한 분리:

| stage | 닫은 계약 | 한컴 결과 |
|---|---|---|
| Stage 23 | `#826 LIST_HEADER`, `#827 PARA_HEADER` size/hash | h03 파일손상 유지, hy-001 guard 손상 |
| Stage 28 | `renderingInfo -> SHAPE_COMPONENT raw_rendering` 일부 | h03/hy-001 파일손상 유지 |
| Stage 30 | `id/instid -> CTRL_HEADER/SHAPE_* tail` | Stage 28과 동일 |

이 결과를 곧바로 "다음 새 payload 축"으로 넘기면 논리적 구멍이 남는다.

```text
Stage 23은 id/instid가 틀린 상태에서 #826/#827만 맞췄다.
Stage 30은 #826/#827이 틀린 상태에서 id/instid만 맞췄다.
```

따라서 다음 단계는 두 축을 동시에 맞춘 상태를 확인해야 한다.

## 2. 목표

Stage 31의 목표는 production 구현 확정이 아니라 조합 검증이다.

```text
hwpx-h-03 손상 지점에서 다음 record들이 동시에 정답 HWP와 일치하는 후보를 만든다.

#824 CTRL_HEADER      rect id
#825 SHAPE_COMPONENT rect instid 포함
#826 LIST_HEADER      drawText text-box envelope
#827 PARA_HEADER      drawText inner paragraph envelope
#831 CTRL_HEADER      first pic id
#834 SHAPE_PICTURE    first pic instid
#835 CTRL_HEADER      second pic id
#837 SHAPE_PICTURE    second pic instid
```

이 조합으로도 한컴 파일손상이 동일하면, `id/instid`와 text-box envelope는 손상 해소의 충분조건에서
제외하고 `#825/#832/#836 SHAPE_COMPONENT payload`로 이동한다.

## 3. 구현 원칙

### 3.1 Stage 23 generic 적용을 그대로 되살리지 않는다

Stage 23은 `hy-001` guard를 깨뜨렸으므로 실패한 후보였다.

이번 단계에서 금지:

```text
모든 TextBox에 raw_list_header_extra/raw_header_extra를 일괄 materialize
```

이번 단계에서 허용:

```text
1. 문제 구간과 같은 HWPX drawText text-box record contract를 재현하는 probe-only 후보
2. h03 정답지와 record hash가 실제로 동시에 맞는지 확인하는 제한된 materialization
3. guard가 깨지면 production 후보로 승격하지 않음
```

### 3.2 먼저 record hash를 닫고, 그 다음 한컴 판정을 본다

한컴 수동 판정 전에 반드시 다음을 확인한다.

```text
1. h03 #824/#831/#835 CTRL_HEADER hash == oracle
2. h03 #834/#837 SHAPE_PICTURE hash == oracle
3. h03 #826 LIST_HEADER size/hash == oracle
4. h03 #827 PARA_HEADER size/hash == oracle
```

즉 "될 것 같은 파일"을 찍는 것이 아니라, 정답지와 닫힌 record 조합을 만든 뒤 판정한다.

## 4. 수정 후보

수정 후보 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

이미 Stage 30에서 `id/instid` parser 축은 반영되었으므로 유지한다.

Stage 31에서는 어댑터 단계에서 `drawText` text-box envelope를 제한적으로 materialize하는 후보를 만든다.
단, Stage 23처럼 문서 전체 TextBox에 넓게 적용하지 않는다.

후보 조건은 소스 구조를 기준으로 한다.

```text
1. HWPX-origin shape object
2. hp:drawText를 가진 rect/object
3. text_box.paragraphs가 존재
4. 해당 text_box 내부 paragraph가 TAC 그림 또는 GenShape control text를 포함
```

이 조건이 너무 넓어 guard를 깨면 즉시 reject하고, 다음 stage에서 `#825/#832/#836` payload로 이동한다.

## 5. 생성 파일

생성 경로:

```text
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/
```

생성 파일:

```text
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hy-001.hwp
```

## 6. 자동 검증

```text
cargo fmt --check
cargo test hwpx_h_03_rect_draw_text_contract_from_source
cargo build
target/debug/rhwp dump-pages <generated>
target/debug/rhwp hwp5-inventory-diff <oracle h03> <generated h03> --focus shape --window 8
```

필수 record 판정:

```text
#824 CTRL_HEADER   == oracle
#826 LIST_HEADER   == oracle
#827 PARA_HEADER   == oracle
#831 CTRL_HEADER   == oracle
#834 SHAPE_PICTURE == oracle
#835 CTRL_HEADER   == oracle
#837 SHAPE_PICTURE == oracle
```

## 7. 수동 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 8. 판정 기준

성공:

```text
1. h01/h02 guard 성공 유지
2. h03 파일손상 해소 또는 중단 위치 이동
3. hy-001 #974 guard 성공 유지
```

실패:

```text
1. h03 결과가 Stage 28/30과 동일
2. h01/h02 guard 회귀
3. hy-001이 Stage 23처럼 파일손상 발생
```

실패 시 결론:

```text
id/instid + drawText envelope 조합은 충분조건이 아니다.
다음 stage는 #825/#832/#836 SHAPE_COMPONENT payload를 byte offset별로 닫는다.
```
