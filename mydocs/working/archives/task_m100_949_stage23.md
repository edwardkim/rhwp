# Task M100 #949 Stage 23 작업 보고서 — drawText TextBox HWP5 envelope 보강

## 1. 목적

Stage 22에서 `hwpx-h-03` 파일손상 후보는 `hp:rect > hp:drawText` 내부 HWP5 record
contract로 좁혀졌다.

이번 단계의 목적은 #974에서 확인한 글상자 TAC 계열 계약을 HWPX -> HWP 저장 어댑터에 연결해,
`drawText` TextBox가 한컴 HWP 로더가 기대하는 HWP5 record envelope를 갖도록 보강하는 것이다.

## 2. 수정 범위

후보 구현 당시 수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

수정하지 않은 파일:

```text
src/serializer/control.rs
src/serializer/body_text.rs
```

이 단계에서는 serializer를 확장하지 않았다. HWPX 출처 IR을 저장 직전에 HWP5 저장 계약에 맞게
materialize하는 방식으로만 처리했다.

판정 후 정리:

```text
이 후보는 hy-001 guard를 깨뜨렸으므로 최종 소스에는 남기지 않았다.
Stage 23 결과는 실패한 후보의 관찰 기록으로만 유지한다.
```

## 3. 후보 구현 내용

추가한 adapter report 카운터:

```text
text_box_list_header_tail_materialized
text_box_para_header_tail_materialized
```

추가한 보강 규칙:

```text
1. HWPX drawText TextBox가 문단을 가지고 있고 raw_list_header_extra가 비어 있으면
   13B zero tail을 materialize한다.

2. TextBox 내부 paragraph의 raw_header_extra가 12B보다 짧으면
   serializer가 24B PARA_HEADER를 쓰도록 12B raw_header_extra를 materialize한다.
```

핵심 byte shape:

```text
TextBox LIST_HEADER:
  20B base + 13B zero tail = 33B

TextBox 내부 PARA_HEADER:
  raw_header_extra =
    numCharShapes(2B)
    numRangeTags(2B)
    numLineSegs(2B)
    tail 00 00 00 80 00 00

serializer는 raw_header_extra[6..]을 header tail로 쓰므로 최종 PARA_HEADER는 24B가 된다.
```

## 4. 정적 계약 검증

생성된 `hwpx-h-03.hwp`의 `/BodyText/Section0` record를 확인했다.

검증 파일:

```text
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-03_section0.jsonl
```

핵심 결과:

```text
#826 LIST_HEADER
  size = 33
  payload_hash = blake3:737c0b8b32c2d22dd194625fa8389a8eb415679d8279b7dfa4f19c56f56504e3

#827 PARA_HEADER
  size = 24
  payload_hash = blake3:a80a25be90041751289411f831d3c67ed1d9d3a85cbe117a07edd0a0950e2e8d
```

Stage 22에서 정답지와 차이가 있던 `#826 LIST_HEADER`, `#827 PARA_HEADER`는 이번 단계
생성 결과에서 정답지와 size/hash가 일치한다.

## 5. 생성 결과

시각 판정 대상 파일:

```text
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hy-001.hwp
```

파일 크기:

| file | size |
|---|---:|
| `hwpx-h-01.hwp` | 366K |
| `hwpx-h-02.hwp` | 32K |
| `hwpx-h-03.hwp` | 38K |
| `hy-001.hwp` | 87K |

## 6. 한컴/렌더러 판정표

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | Stage 18 성공 guard |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | Stage 18 성공 guard |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hwpx-h-03.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 글상자 전까지만 출력, 실패 | 1페이지 페이지네이션 실패 | 파일손상 대상 |
| `output/poc/hwpx2hwp/task949/stage23_drawtext_hwp5_contract/hy-001.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 마지막 표전까지만 출력, 실패 | 정상 | #974 글상자 TAC guard |

주의:

```text
정적 record 계약상 #826/#827은 정답지와 일치한다.
하지만 한컴 에디터 판정에서 hwpx-h-03 파일손상은 해소되지 않았고,
#974 guard인 hy-001에서 새 파일손상이 발생했다.
```

## 7. 실행한 검증

```text
cargo fmt --all -- --check
cargo test --quiet hwpx_h_03_drawtext_hwp5_textbox_contract_from_source
cargo test --quiet test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx
cargo test --quiet hwpx_h_03_rect_draw_text_contract_from_source
cargo build --quiet
git diff --check
```

결과:

```text
모두 통과
```

기존 warning은 있었지만 이번 변경과 무관한 경고다.

## 8. 판정 해석

이번 단계에서 닫힌 항목:

```text
1. hp:rect > hp:drawText TextBox LIST_HEADER 33B envelope
2. hp:rect > hp:drawText 내부 paragraph PARA_HEADER 24B envelope
3. 정적 테스트 수준의 #974 글상자 TAC 렌더링 회귀 없음
```

하지만 시각 판정 기준으로 Stage 23 후보는 reject한다.

reject 사유:

```text
1. target인 hwpx-h-03 파일손상이 남았다.
2. guard인 hy-001에서 새 한컴 파일손상이 발생했다.
3. 따라서 TextBox 전체에 LIST_HEADER/PARA_HEADER tail을 generic하게 보강하는 전략은
   한컴 HWP 로더 contract에 맞지 않는다.
```

확정된 규칙:

```text
1. #826 LIST_HEADER, #827 PARA_HEADER 차이는 실재한다.
2. 그러나 이 두 레코드를 정답지와 맞추는 것만으로 hwpx-h-03 파일손상은 해결되지 않는다.
3. TextBox envelope 보강은 문서 전체 TextBox에 일괄 적용하면 안 된다.
4. 특히 #974 hy-001처럼 글상자 내부 TAC picture/space 조판이 정상이어야 하는 문서는
   HWP5 record envelope 보강이 별도 guard를 반드시 통과해야 한다.
```

아직 단정하지 않는 항목:

```text
1. hwpx-h-03 한컴 파일손상 최종 해소 여부
2. 파일손상이 남을 경우, 원인이 #824/#831/#835 GenShape/ShapeComponent 계열인지 여부
```

다음 판단 기준:

```text
1. Stage 23 code change는 성공 후보가 아니므로 다음 구현 후보에 그대로 승계하지 않는다.
2. hwpx-h-03 파일손상은 #826/#827을 원인에서 제외하고 추적한다.
3. 다음 후보는 h03의 2페이지 글상자 직전/주변 record bundle 중
   #824/#831/#835 GenShape/ShapeComponent/Picture 계열을 정답지와 비교한다.
4. hy-001은 #974 guard로 계속 유지한다.
```
