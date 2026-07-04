# Task 1293 Stage 63: 0mm 미주 Shape 실제 잉크 bbox 분석

## 목적

Stage62에서 `문13` 제목 자체를 당기는 실험은 모두 폐기했다. 남은 핵심은 display 11쪽
`pi=537` 비TAC Shape가 renderer에서 선언 bbox 하단까지 flow advance를 소비하면서 `pi=538`,
`pi=539`가 PDF/Hancom보다 아래로 밀리는 현상이다.

이번 단계는 코드 수정 전에 다음 값을 분리한다.

1. render tree의 `Image` 선언 bbox
2. rhwp PNG의 실제 잉크 bbox
3. PDF PNG의 실제 잉크 bbox
4. `layout_column_item`의 `Shape pi=537` y_in/y_out
5. `pi=538`, `pi=539`의 render tree 위치

## 대상

- 샘플: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- 기준 PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- target: `2024-11-practice-above0-between0-below0`
- page: display 11쪽, 내부 page index 10
- 문단:
  - `pi=537`: Shape item
  - `pi=538`: 문12 tail
  - `pi=539`: 문13 제목

## 분석 계획

- Stage61 HEAD 기준 산출물을 사용한다.
- compare PNG만으로 좌우 page offset을 추정하지 않고, rhwp SVG PNG와 PDF PNG를 각각 같은 page
  좌표계에서 분석한다.
- graph 주변 ROI에서 검은색/색상 잉크 픽셀의 bbox를 산출한다.
- render tree bbox와 잉크 bbox의 bottom 차이를 계산한다.
- 차이가 충분히 크면 `Shape` flow advance가 선언 bbox가 아니라 실제 image ink/visible bbox를 써야
  하는지 검토한다.

## 검증 계획

- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0`
- 필요 시 `rsvg-convert`로 SVG page 11을 PNG화한다.
- 분석 스크립트는 임시 명령으로 수행하고, 재사용 가치가 있으면 sweep script 보강 stage로 분리한다.

## 실행 결과

### baseline sweep

```sh
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage63_zero_baseline \
  --rhwp-bin target/debug/rhwp
```

결과:

- page count: SVG/PDF/render tree `21/21/21`
- renderer `LAYOUT_OVERFLOW`: 0
- hard gate: frame/title/order/equation 0
- qflow 후보: `[11, 12, 13, 17, 19, 20]`

### render tree bbox

display 11쪽 render tree:

| 항목 | bbox |
|---|---|
| `pi=535` TextLine | `x=34.0 y=842.0 w=357.2 h=30.6` |
| `pi=536` TextLine | `x=34.0 y=878.7 w=357.2 h=12.0` |
| `pi=537` Image | `x=34.9 y=898.0 w=220.6 h=155.9` |
| `pi=538` TextLine | `x=34.0 y=1072.0 w=357.2 h=35.9` |
| `pi=539` TextLine | 우측 단 `x=402.5 y=90.7 w=357.2 h=12.0` |

### 실제 잉크 bbox

`rhwp_png/rhwp_011.png`와 `pdf_png/pdf-11.png`에서 graph ROI의 connected component를 비교했다.

| 기준 | graph ink bbox | 해석 |
|---|---|---|
| rhwp | `(40, 899) - (248, 1049)` | render tree image bbox `y=898.0 h=155.9`와 거의 일치 |
| PDF/Hancom | `(40, 868) - (248, 1019)` | 같은 크기의 graph가 약 30px 위에 있음 |

따라서 이 문제는 image 내부 투명 여백/선언 bbox 하단 문제가 아니다. graph 자체가 rhwp에서
PDF/Hancom보다 약 30px 아래에 배치된다.

### cursor와 vpos base

현재 `RHWP_DEBUG_TAC_CURSOR=1` 기준:

```text
FullPara pi=536 y_in=878.7 y_out=896.7 dy=18.0
FullPara pi=537 y_in=896.7 y_out=896.7 dy=0.0
Shape pi=537 ci=0 y_in=896.7 y_out=1072.0 dy=175.2
FullPara pi=538 y_in=1072.0 y_out=1113.9 dy=41.9
FullPara pi=539 y_in=90.7 y_out=108.7 dy=18.0
```

`RHWP_VPOS_DEBUG=1` 기준:

```text
pi=514 path=page base=150253 y=108.73
pi=516 path=lazy base=147097 y=178.00
pi=537 path=lazy base=147097 y=896.72
pi=540 path=page base=222481 y=108.73
```

`pi=539`는 우측 단 첫 문항 제목이므로 page path의 시작 위치로 배치된다. display 11쪽의
실제 문제는 `pi=537` graph와 이어지는 `pi=538` tail이 좌측 단 하단에서 약 30px 늦게 시작하는
것이다.

`pi=537`의 저장 vpos는 `207548`이다. 이를 page 좌표로 환산하면:

| base | 계산 y | 의미 |
|---:|---:|---|
| `147097` | `896.7px` | 현재 rhwp lazy base |
| `147549` | `890.7px` | reset 억제 실험에 가까운 base |
| `148901` | `872.7px` | `pi=514` 저장 vpos |
| `149251` | `868.0px` | PDF graph top에 해당하는 역산 base |
| `150253` | `854.6px` | page path base |

PDF/Hancom graph top은 현재 lazy base보다 `약 2250HU`, 즉 `약 30px` 높은 base를 쓴 경우에
가깝다. 특히 `pi=514`의 저장 vpos `148901`이 PDF에 근접한다.

## 판단

Stage62에서 가정한 “선언 bbox 하단과 실제 잉크 하단 차이”는 틀렸다. rhwp와 PDF는 graph 크기가
거의 같고, 차이는 y anchor 전체가 약 30px 내려간 데 있다.

원인은 `pi=516`에서 `HeightCursor`가 page path에서 lazy path로 전환하면서 base를 `147097`까지
낮게 재산출하는 데 있다. 이 lazy base가 `pi=537` Shape까지 유지되어 graph와 후속 `pi=538` tail이
PDF/Hancom보다 아래로 밀린다.

단순한 해결은 아직 금지한다.

- 제목만 현재 단에 남기면 frame 밖 또는 수식 겹침이 생긴다.
- vpos reset 억제를 전역 적용하면 display 14쪽 등 다른 page가 회귀한다.
- `pi=537` shape advance를 줄이는 것은 graph 자체 위치를 고치지 못한다.

## 다음 단계

Stage64에서는 `HeightCursor` lazy base 산출 조건을 좁게 분석한다.

- `pi=516` lazy base가 `prev_vpos_end - y_delta`로 재산출되는 과정에서 0/0/0 미주 빈 spacer가
  과도하게 반영되는지 확인한다.
- `prev_para`가 빈 paragraph이고 current paragraph가 같은 미주 flow의 후속 텍스트일 때,
  `vpos_page_base` 또는 이전 안정 base를 유지해야 하는지 실험한다.
- 실험은 `2024-11-practice-above0-between0-below0`뿐 아니라 `stage59` 전체 qflow 후보에서
  회귀 여부를 확인한다.
