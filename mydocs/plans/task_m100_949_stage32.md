# Task m100 #949 Stage 32 계획 - h03 SHAPE_COMPONENT와 rhwp 페이지네이션 실패 연결

## 1. 배경

Stage 31 수동 판정은 Stage 30과 동일했다.

```text
파일손상과 중단된 페이지 위치가 동일하다.
```

Stage 31에서 이미 닫힌 계약:

| record | 계약 |
|---:|---|
| `#824 CTRL_HEADER` | outer rect id/common attr |
| `#826 LIST_HEADER` | drawText text-box list envelope |
| `#827 PARA_HEADER` | drawText inner paragraph envelope |
| `#831 CTRL_HEADER` | first picture id/common attr |
| `#834 SHAPE_PICTURE` | first picture payload/tail |
| `#835 CTRL_HEADER` | second picture id/common attr |
| `#837 SHAPE_PICTURE` | second picture payload/tail |

아직 정답 HWP와 다른 record:

| record | 의미 |
|---:|---|
| `#825 SHAPE_COMPONENT` | outer text-box rect component |
| `#832 SHAPE_COMPONENT` | first inner picture component |
| `#836 SHAPE_COMPONENT` | second inner picture component |

작업지시자가 추가로 확인한 중요한 관찰:

```text
hy-001.hwp:    rhwp-studio에서 정상 조판
hwpx-h-03.hwp: rhwp-studio에서 1페이지 마지막 표 다음 페이지네이션 실패
```

따라서 Stage 32는 한컴 에디터 수동 판정으로 바로 가는 probe가 아니라,
`hwpx-h-03`의 rhwp 재로드 조판 실패를 먼저 설명하는 분석 단계로 진행한다.

## 2. 목표

Stage 32의 목표는 다음 질문에 답하는 것이다.

```text
hwpx-h-03 generated HWP가 rhwp-studio에서도 1페이지 마지막 표 다음에서 페이지네이션을 실패하는
구체적인 HWP5/IR 계약 차이는 무엇인가?
```

성공 기준은 다음 중 하나다.

```text
1. #825/#832/#836 SHAPE_COMPONENT payload 차이 중 페이지네이션 실패와 연결되는 필드를 특정한다.
2. 특정 필드가 HWPX source 또는 rhwp IR의 어느 값에서 생성되어야 하는지 설명한다.
3. 필드 특정이 아직 불가능하면, 다음 stage에서 검증할 미해석 byte range를 1개 이상의 작은 군으로
   분리한다.
```

## 3. 금지 사항

이번 단계에서 금지:

```text
1. Stage 31에서 이미 닫힌 LIST_HEADER/PARA_HEADER/id/instid/CTRL_DATA/SHAPE_PICTURE tail 재검증
2. 정답 HWP byte payload를 이유 없이 통째로 graft하는 후보 생성
3. hwpx-h-01에서 이미 성공한 계약을 깨뜨릴 수 있는 광범위 adapter 변경
4. 한컴 수동 판정을 먼저 요구하는 "될 것 같은" HWP 파일 생성
```

## 4. 작업 절차

### 4.1 실패 지점의 문서 구조 고정

대상:

```text
samples/hwpx/hwpx-h-03.hwpx
samples/hwpx/hancom-hwp/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage31_id_instid_textbox_envelope/hwpx-h-03.hwp
```

확인할 항목:

```text
1. 1페이지 마지막 표 이후 paragraph/control sequence
2. 문제 text-box rect와 내부 picture 2개의 source XML path
3. oracle HWP와 generated HWP의 해당 control tuple index
4. rhwp reload 후 page/control sequence의 첫 divergence
```

### 4.2 SHAPE_COMPONENT payload field decode

대상 record:

```text
#825 outer rect SHAPE_COMPONENT
#832 first inner picture SHAPE_COMPONENT
#836 second inner picture SHAPE_COMPONENT
```

각 record에 대해 다음을 분리한다.

```text
1. component base shape id/name
2. local position/size
3. coordinate origin/anchor 관련 값
4. rotation/scale/transform/renderingInfo 계열 값
5. group/text-box child coordinate에 영향을 줄 수 있는 값
6. raw-only tail 또는 아직 모델화되지 않은 field
```

### 4.3 rhwp 페이지네이션 실패와 연결

분석 기준:

```text
1. rhwp-studio에서도 실패하는 차이를 우선한다.
2. hy-001은 rhwp-studio 정상 조판이므로, hy-001을 깨뜨리는 후보는 reject한다.
3. hwpx-h-01/hwpx-h-02 guard에서 이미 성공한 table-axis 계약은 유지한다.
```

## 5. 산출물

생성 디렉터리:

```text
output/poc/hwpx2hwp/task949/stage32_h03_shape_component_trace/
```

예상 산출물:

```text
h03_failure_window.md
h03_shape_component_field_diff.md
h03_source_to_component_mapping.md
h03_rhwp_pagination_divergence.md
```

working 문서:

```text
mydocs/working/task_m100_949_stage32.md
```

## 6. 다음 단계 분기

Stage 32 결과에 따라 다음처럼 분기한다.

```text
1. 필드가 특정되면 Stage 33에서 최소 구현 후보를 만든다.
2. 필드가 특정되지 않으면 Stage 33은 미해석 byte range를 더 작은 구조 단위로 분해한다.
3. 어떤 경우에도 닫힌 축을 다시 넓게 probe하지 않는다.
```

## 7. 승인 요청

위 계획대로 Stage 32를 진행하려면 승인한다.

## 8. 진행 중 정정

Stage 32 진행 중 초기 계획을 다음처럼 정정했다.

```text
초기 계획:
  #825/#832/#836 SHAPE_COMPONENT 차이를 rhwp 페이지네이션 실패의 첫 원인 후보로 분석한다.

실제 관찰:
  rhwp 페이지네이션 첫 divergence는 paragraph 33 글상자보다 앞선 paragraph 18에서 발생한다.
  원인은 paragraph 18의 PARA_LINE_SEG.vertical_pos가 oracle/source XML의 68258에서
  DocumentCore load/reflow 이후 61435로 덮어써지는 것이다.
```

따라서 Stage 32 산출물은 SHAPE_COMPONENT byte 후보 생성이 아니라,
HWPX `lineSegArray.vertpos` 보존 계약과 `DocumentCore::from_bytes()` reflow 경로의
충돌을 문서화하는 방향으로 조정한다.
