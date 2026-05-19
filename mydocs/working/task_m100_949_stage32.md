# Task m100 #949 Stage 32 - h03 pagination divergence 재분석

## 1. 목표

Stage 32는 `hwpx-h-03`의 실패를 다시 파일 생성으로 probe하지 않고,
현재 실패가 어느 HWP5/IR 계약에서 처음 발생하는지 고정하는 단계다.

초기 계획은 `#825/#832/#836 SHAPE_COMPONENT` 차이를 우선 추적하는 것이었지만,
분석 중 rhwp-studio 페이지네이션 실패의 첫 divergence가 더 앞쪽에 있음을 확인했다.

## 2. 사용 파일

```text
source HWPX:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-03.hwp
```

## 3. 핵심 관찰

`paragraph 18`은 1페이지 마지막 표다.

```text
oracle HWP:
  paragraph 18 lineSeg vpos = 68258
  paragraph 19 starts on page 2

generated HWP:
  paragraph 18 lineSeg vpos = 61435
  paragraph 19 remains on page 1
```

차이:

```text
68258 - 61435 = 6823 HU
```

즉 generated HWP는 마지막 표가 6823 HU 위로 올라가면서 다음 문단이 1페이지에 들어간다.
이후 2페이지부터 page sequence가 oracle과 다르게 이어진다.

## 4. HWPX source 자체의 값

`samples/hwpx/hwpx-h-03.hwpx`의 `Contents/section0.xml`에는 다음 값이 존재한다.

```text
textpos="0" vertpos="68258" vertsize="600" textheight="600" baseline="510"
spacing="272" horzpos="0" horzsize="48188" flags="393216"
```

이 값은 oracle HWP의 `PARA_LINE_SEG`와 일치한다.

## 5. 도구별 차이

`ir-diff`는 `parse_document()` 결과를 비교한다.
이 경로에서는 HWPX source와 oracle HWP의 `paragraph 18`이 일치한다.

```text
target/debug/rhwp ir-diff samples/hwpx/hwpx-h-03.hwpx samples/hwpx/hancom-hwp/hwpx-h-03.hwp -s 0 -p 18

결과:
차이 0건
```

그러나 `dump`/`dump-pages`는 `DocumentCore::from_bytes()` 이후 상태를 본다.
이 경로에서는 HWPX source도 `paragraph 18`이 `61435`로 바뀐다.

```text
target/debug/rhwp dump samples/hwpx/hwpx-h-03.hwpx --section 0 --para 18

결과:
ls[0].vpos = 61435
```

따라서 문제는 HWPX XML 파싱 누락이 아니라, HWPX load 후 reflow 경로가 명시적
`lineSegArray.vertpos`를 덮어쓰는 것이다.

## 6. 코드 해석

관련 코드:

```text
src/document_core/commands/document.rs
  DocumentCore::from_bytes()
  reflow_zero_height_paragraphs()
```

`reflow_zero_height_paragraphs()`는 다음 조건에서 section vpos를 다시 계산한다.

```text
Control::Table(t)
  if !t.common.treat_as_char
  && t.common.text_wrap == TopAndBottom
  && t.common.height > 0
  && t.raw_ctrl_data.is_empty()
```

HWPX parser 출처 table은 `raw_ctrl_data`가 비어 있으므로 이 조건에 걸린다.
그 결과 source XML에 이미 계산 완료된 `vertpos=68258`이 있어도, loader가 `61435`로
덮어쓴다.

## 7. Stage 32 결론

```text
rhwp-studio 페이지네이션 실패의 1차 원인:
  HWPX load/reflow가 명시적 lineSegArray.vertpos를 덮어쓰는 문제

한컴 에디터 파일손상 후보:
  #825/#832/#836 SHAPE_COMPONENT 차이는 아직 남아 있으나, rhwp 첫 divergence는 아니다.
```

## 8. 산출물

```text
output/poc/hwpx2hwp/task949/stage32_h03_shape_component_trace/h03_failure_window.md
output/poc/hwpx2hwp/task949/stage32_h03_shape_component_trace/h03_rhwp_pagination_divergence.md
output/poc/hwpx2hwp/task949/stage32_h03_shape_component_trace/h03_source_to_component_mapping.md
output/poc/hwpx2hwp/task949/stage32_h03_shape_component_trace/h03_shape_component_field_diff.md
```

## 9. 다음 단계 제안

Stage 33은 최소 구현 후보를 다음 계약으로 잡아야 한다.

```text
HWPX lineSegArray가 명시적으로 있고 line_height/text_height/segment_width가 계산 완료 상태라면,
비-TAC TopAndBottom table의 raw_ctrl_data가 비어 있다는 이유만으로 section vpos를 재계산하지 않는다.
```

검증 순서:

```text
1. samples/hwpx/hwpx-h-03.hwpx 로드 후 paragraph 18 vpos가 68258로 유지되는지 확인
2. stage31 adapter 출력 HWP에서 paragraph 18 vpos가 68258로 저장되는지 확인
3. hwpx-h-01/hwpx-h-02 guard가 깨지지 않는지 확인
4. 이후에도 한컴 파일손상이 남으면 #825/#832/#836 SHAPE_COMPONENT 계약으로 이동
```

